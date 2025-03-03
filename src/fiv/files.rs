/*
 * fiv - Fast Image Viewer
 * Copyright 2015,2020,2025  Simon Arlott
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use super::{CommandLineArgs, CommandLineFilenames, Image, Mark, Orientation, Rotate, Waitable};
use async_notify::Notify;
use gtk::glib::clone::Downgrade;
use itertools::interleave;
use log::{debug, error, trace};
use pariter::IteratorExt;
use std::collections::{HashSet, VecDeque};
use std::iter;
use std::path::PathBuf;
use std::sync::atomic::{self, AtomicBool};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;
use threadpool::ThreadPool;

#[derive(Debug)]
pub struct Files {
	args: CommandLineArgs,
	startup: Mutex<Startup>,
	state: Mutex<State>,
	notify: Notify,
	seq_pool: ThreadPool,

	/// `start()` has finished or loaded at least one image
	start_ready: Waitable<bool>,
	start_finished: Waitable<bool>,
	shutdown: Arc<AtomicBool>,
}

#[derive(Debug)]
struct Startup {
	begin: Instant,
	image_loaded: bool,
	image_ready: bool,
}

#[derive(Debug, Default)]
pub struct Current {
	pub image: Option<Arc<Image>>,
	pub filename: PathBuf,
	pub position: usize,
	pub total: usize,
	pub mark: Option<bool>,
}

#[derive(Debug, Copy, Clone)]
pub enum Navigate {
	First,
	Previous,
	Next,
	Last,
}

impl Startup {
	pub fn new(begin: Instant) -> Self {
		Self {
			begin,
			image_loaded: false,
			image_ready: false,
		}
	}
}

impl Files {
	pub fn new(args: CommandLineArgs, startup: Instant) -> Arc<Files> {
		let preload = args.preload as usize;
		let shutdown = Arc::new(AtomicBool::new(false));
		let files = Arc::new(Files {
			args,
			startup: Mutex::new(Startup::new(startup)),
			state: Mutex::new(State::new(preload, shutdown.clone())),
			notify: Notify::new(),
			seq_pool: ThreadPool::new(1),
			start_ready: Waitable::new(false),
			start_finished: Waitable::new(false),
			shutdown,
		});

		files.state.lock().unwrap().start(&files);
		files
	}

	pub fn mark_supported(&self) -> bool {
		self.args.mark_directory.is_some()
	}

	pub fn begin(&self) -> Instant {
		self.startup.lock().unwrap().begin
	}

	pub fn start(self: &Arc<Self>) -> bool {
		let self_copy = self.clone();
		let shutdown_copy = self.shutdown.clone();

		std::thread::spawn(move || {
			pariter::scope(|scope| {
				let canonical_mark_directory =
					self_copy
						.args
						.mark_directory
						.as_ref()
						.and_then(|directory| {
							directory
								.canonicalize()
								.map_err(|err| error!("{}: {err}", directory.display()))
								.ok()
						});

				CommandLineFilenames::new(&self_copy.args, shutdown_copy.clone())
					.parallel_map_scoped(scope, move |filename| {
						if shutdown_copy.load(atomic::Ordering::Acquire) {
							None
						} else {
							Image::new(canonical_mark_directory.as_ref(), &filename)
								.map_err(|err| error!("{}: {err}", filename.display()))
								.ok()
						}
					})
					.flatten()
					.for_each(|image| self_copy.add(image));

				debug!(
					"Files added from command line in {:?}",
					self_copy.startup.lock().unwrap().begin.elapsed()
				);

				self_copy.start_ready.set(true);
				self_copy.start_finished.set(true);
				self_copy.update_ui();
			})
			.unwrap();
		});

		self.start_ready.wait(&true);

		let state = self.state.lock().unwrap();
		!state.images.is_empty()
	}

	pub fn shutdown(&self) {
		if self
			.shutdown
			.compare_exchange(
				false,
				true,
				atomic::Ordering::Release,
				atomic::Ordering::Acquire,
			)
			.is_ok()
		{
			self.state.lock().unwrap().shutdown();
			self.update_ui();
		}
	}

	pub fn join(&self) {
		self.shutdown();

		let begin = Instant::now();
		debug!("Waiting for background tasks to finish...");
		self.seq_pool.join();
		debug!("Background tasks complete in {:?}", begin.elapsed());
	}

	fn add(&self, image: Arc<Image>) {
		if self.shutdown.load(atomic::Ordering::Acquire) {
			return;
		}

		let mut state = self.state.lock().unwrap();
		if state.add(image) {
			debug!(
				"First image added after {:?}",
				self.startup.lock().unwrap().begin.elapsed()
			);

			self.start_ready.set(true);
		}
		self.update_ui();
	}

	pub fn starting(&self) -> bool {
		!self.start_finished.get()
	}

	pub async fn ui_wait(&self) -> bool {
		if !self.shutdown.load(atomic::Ordering::Acquire) {
			self.notify.notified().await;
		}
		!self.shutdown.load(atomic::Ordering::Acquire)
	}

	pub fn update_ui(&self) {
		self.notify.notify();
	}

	pub fn current(&self) -> Current {
		self.state.lock().unwrap().current()
	}

	fn position(&self) -> usize {
		self.state.lock().unwrap().position()
	}

	pub fn loaded(&self, image: &Image) {
		let state = self.state.lock().unwrap();
		let current = state.current();

		// It's possible to have unloaded the image between releasing the
		// preload mutex and acquiring the state mutex
		if !image.loaded() || self.shutdown.load(atomic::Ordering::Acquire) {
			return;
		}

		if let Ok(mut startup) = self.startup.lock() {
			if !startup.image_loaded {
				startup.image_loaded = true;

				debug!("First image loaded after {:?}", startup.begin.elapsed());
			}
		}

		if let Some(current_image) = current.image {
			if *current_image == *image {
				if let Ok(mut startup) = self.startup.lock() {
					if !startup.image_ready {
						startup.image_ready = true;

						debug!(
							"First image ready for display after {:?}",
							startup.begin.elapsed()
						);
					}
				}

				self.update_ui();
			}
		}
	}

	/// Run a long task sequentially in the background (for file I/O)
	fn seq_execute<F: FnOnce(&Image) + Send + 'static>(
		self: &Arc<Self>,
		current: Current,
		always: bool,
		func: F,
	) {
		let self_copy = self.clone();

		self.seq_pool.execute(move || {
			let shutdown = self_copy.shutdown.load(atomic::Ordering::Acquire);

			// To avoid wasting resources doing something that is no longer
			// needed, check that we're still on the same image, unless this
			// task must always be run
			if always || (!shutdown && current.position == self_copy.position()) {
				if let Some(image) = current.image {
					func(&image);

					// After running the task, if we're still on the same image
					// then the UI for it needs to be updated
					if current.position == self_copy.position() {
						self_copy.update_ui();
					}
				}
			}
		});
	}

	pub fn navigate(self: &Arc<Self>, action: Navigate) {
		let mut state = self.state.lock().unwrap();

		state.navigate(action);

		if self.args.mark_directory.is_some() {
			self.seq_execute(state.current(), false, Image::refresh_mark);
		}
		self.update_ui();
	}

	pub fn orientation(self: &Arc<Self>, rotate: Rotate, horizontal_flip: bool) {
		let mut state = self.state.lock().unwrap();

		state.orientation(Orientation::new(rotate, horizontal_flip));
		self.update_ui();
	}

	pub fn mark(self: &Arc<Self>, mark: Mark) {
		if self.args.mark_directory.is_some() {
			self.seq_execute(self.state.lock().unwrap().current(), true, move |image| {
				image.mark(mark);
			});
		}
	}
}

#[derive(Debug)]
struct State {
	images: Vec<Arc<Image>>,
	position: usize,
	preload: Arc<Preload>,
}

impl State {
	fn new(preload: usize, shutdown: Arc<AtomicBool>) -> Self {
		Self {
			images: Vec::new(),
			position: 0,
			preload: Arc::new(Preload::new(preload + 1, shutdown)),
		}
	}

	pub fn start(&self, files: &Arc<Files>) {
		self.preload.start(files);
	}

	/// Returns true if this is the first image
	pub fn add(&mut self, image: Arc<Image>) -> bool {
		self.images.push(image);

		let first = self.images.len() == 1;
		self.preload(!first);
		first
	}

	fn preload(&self, only_if_starved: bool) {
		self.preload
			.update(&self.images, self.position, only_if_starved);
	}

	pub fn current(&self) -> Current {
		if let Some(image) = self.images.get(self.position) {
			Current {
				image: Some(image.clone()),
				filename: image.filename.clone(),
				position: self.position + 1,
				total: self.images.len(),
				mark: image.marked(),
			}
		} else {
			Current::default()
		}
	}

	pub fn position(&self) -> usize {
		self.images
			.get(self.position)
			.map_or(0, |_| self.position + 1)
	}

	pub fn navigate(&mut self, action: Navigate) {
		match action {
			Navigate::First => {
				self.position = 0;
			}

			Navigate::Previous => {
				if self.position > 0 {
					self.position -= 1;
				}
			}

			Navigate::Next => {
				if !self.images.is_empty() && self.position < self.images.len() - 1 {
					self.position += 1;
				}
			}

			Navigate::Last => {
				if !self.images.is_empty() {
					self.position = self.images.len() - 1;
				}
			}
		}

		self.preload(false);
	}

	pub fn orientation(&mut self, add: Orientation) {
		if let Some(image) = self.images.get(self.position) {
			image.add_orientation(add);
		}
	}

	pub fn shutdown(&self) {
		self.preload.shutdown();
	}
}

#[derive(Debug)]
struct Preload {
	capacity: usize,
	pool: ThreadPool,
	state: Mutex<PreloadState>,
	loading_required: Condvar,
	shutdown: Arc<AtomicBool>,
}

#[derive(Debug)]
struct PreloadState {
	queue: VecDeque<Arc<Image>>,
	load: HashSet<Arc<Image>>,
	loading: HashSet<Arc<Image>>,
	loaded: HashSet<Arc<Image>>,
}

impl PreloadState {
	pub fn new(capacity: usize) -> Self {
		Self {
			queue: VecDeque::with_capacity(capacity),
			load: HashSet::with_capacity(capacity),
			loading: HashSet::with_capacity(capacity),
			loaded: HashSet::with_capacity(capacity),
		}
	}
}

impl Preload {
	pub fn new(capacity: usize, shutdown: Arc<AtomicBool>) -> Self {
		Self {
			capacity,
			pool: threadpool::Builder::new().build(),
			state: Mutex::new(PreloadState::new(capacity)),
			loading_required: Condvar::new(),
			shutdown,
		}
	}

	pub fn start(self: &Arc<Self>, files: &Arc<Files>) {
		for _ in 0..self.pool.max_count() {
			let self_ref = self.downgrade();
			let files_ref = files.downgrade();

			self.pool.execute(move || {
				loop {
					let Some(self_copy) = self_ref.upgrade() else {
						return;
					};

					let Some(files_copy) = files_ref.upgrade() else {
						return;
					};

					if self_copy.shutdown.load(atomic::Ordering::Acquire) {
						return;
					}

					self_copy.load_one_or_wait(&files_copy);
				}
			});
		}
	}

	pub fn update(&self, images: &[Arc<Image>], current: usize, only_if_starved: bool) {
		if images.is_empty() || self.shutdown.load(atomic::Ordering::Acquire) {
			return;
		}

		let mut state = self.state.lock().unwrap();

		// Don't repeatedly update the queue on startup after preloading the
		// maximum number of images
		if only_if_starved && state.load.len() == self.capacity {
			return;
		}

		state.queue.clear();

		// Preload images forward and backward
		let forward = images.iter().skip(current + 1);
		let backward = images.iter().rev().skip(images.len() - current);
		let mut images =
			itertools::chain(iter::once(&images[current]), interleave(forward, backward));
		#[expect(clippy::mutable_key_type, reason = "Key is immutable")]
		let mut load = HashSet::<Arc<Image>>::with_capacity(self.capacity);

		while load.len() < self.capacity {
			if let Some(image) = images.next() {
				load.insert(image.clone());
				if !image.loaded() && !state.loading.contains(image) {
					state.queue.push_back(image.clone());
				}
			} else {
				break;
			}
		}

		// Unload images that will not be preloaded
		state.loaded.retain(|image| {
			if load.contains(image) {
				true
			} else {
				image.unload();
				false
			}
		});
		state.load = load;

		// Start background loading for images that are not loaded
		for _ in 0..state.queue.len() {
			self.loading_required.notify_one();
		}
	}

	fn load_one_or_wait(&self, files: &Files) {
		let mut state = self.state.lock().unwrap();

		if let Some(image) = state.queue.pop_front() {
			state.loading.insert(image.clone());
			drop(state);

			image.load();

			state = self.state.lock().unwrap();
			state.loading.remove(&image);

			if state.load.contains(&image) {
				state.loaded.insert(image.clone());

				trace!(
					"Loaded {} image{}",
					state.loaded.len(),
					if state.loaded.len() == 1 { "" } else { "s" }
				);

				// Release preload mutex before acquiring the state mutex
				drop(state);
				files.loaded(&image);
			} else {
				image.unload();
			}
		} else {
			drop(self.loading_required.wait(state).unwrap());
		}
	}

	pub fn shutdown(self: &Arc<Self>) {
		{
			let mut state = self.state.lock().unwrap();

			state.queue.clear();
			state.load.clear();
			self.loading_required.notify_all();
		}

		// Unload images to save memory, but do it in the background so that the
		// process exit isn't delayed if there's nothing else to do
		let self_copy = self.clone();

		std::thread::spawn(move || {
			let mut state = self_copy.state.lock().unwrap();

			for image in &state.loaded {
				image.unload();
			}
			state.loaded.clear();
		});
	}
}
