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

use super::{CommandLineArgs, CommandLineFilenames, Image, Mark, Waitable};
use async_notify::Notify;
use log::{debug, error};
use pariter::IteratorExt;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use threadpool::ThreadPool;

#[derive(Debug)]
pub struct Files {
	args: Arc<CommandLineArgs>,
	state: Mutex<State>,
	notify: Notify,
	seq_pool: ThreadPool,

	/// start() has finished or loaded at least one image
	start_ready: Waitable<bool>,
	start_finished: Waitable<bool>,
}

#[derive(Debug, Default)]
pub struct Current {
	pub image: Option<Arc<Image>>,
	pub filename: PathBuf,
	pub position: usize,
	pub total: usize,
	pub mark: Option<bool>,
}

#[derive(Debug)]
pub enum Navigate {
	First,
	Previous,
	Next,
	Last,
}

pub fn file_err<P: AsRef<Path>, E: Display>(path: P, err: E) {
	error!("{}: {}", path.as_ref().display(), err);
}

impl Files {
	pub fn new(args: Arc<CommandLineArgs>) -> Arc<Files> {
		Arc::new(Files {
			args,
			state: Mutex::new(State::default()),
			notify: Notify::new(),
			seq_pool: ThreadPool::new(1),
			start_ready: Waitable::new(false),
			start_finished: Waitable::new(false),
		})
	}

	pub fn start(self: &Arc<Self>) -> bool {
		let self_copy = self.clone();
		self.state.lock().unwrap().start_begin = Instant::now();

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
								.map_err(|err| file_err(directory, err))
								.ok()
						});

				CommandLineFilenames::new(&self_copy.args)
					.parallel_map_scoped(scope, move |filename| {
						Image::new(&canonical_mark_directory, &filename)
							.map_err(|err| file_err(&filename, err))
							.ok()
					})
					.flatten()
					.for_each(|image| self_copy.load(image));

				debug!(
					"Files loaded from command line in {:?}",
					self_copy.state.lock().unwrap().start_begin.elapsed()
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

	fn load(&self, image: Image) {
		let mut state = self.state.lock().unwrap();
		if state.add(image) {
			debug!("First image loaded after {:?}", state.start_begin.elapsed());

			self.start_ready.set(true);
		}
		self.update_ui();
	}

	pub fn is_loading(&self) -> bool {
		!self.start_finished.get()
	}

	pub fn mark_supported(&self) -> bool {
		self.args.mark_directory.is_some()
	}

	pub async fn ui_wait(&self) {
		self.notify.notified().await;
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

	/// Run a long task sequentially in the background (for file I/O)
	fn seq_execute<F: FnOnce(&Image) + Send + 'static>(
		self: &Arc<Self>,
		current: Current,
		always: bool,
		func: F,
	) {
		let self_copy = self.clone();

		self.seq_pool.execute(move || {
			// To avoid wasting resources doing something that is no longer
			// needed, check that we're still on the same image, unless this
			// task must always be run
			if always || current.position == self_copy.position() {
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
			self.seq_execute(state.current(), false, |image| image.refresh_mark());
		}
		self.update_ui();
	}

	pub fn mark(self: &Arc<Self>, mark: Mark) {
		if self.args.mark_directory.is_some() {
			self.seq_execute(self.state.lock().unwrap().current(), true, move |image| {
				image.mark(mark)
			});
		}
	}
}

#[derive(Debug)]
struct State {
	start_begin: Instant,
	images: Vec<Arc<Image>>,
	position: usize,
}

impl Default for State {
	fn default() -> Self {
		Self {
			start_begin: Instant::now(),
			images: Vec::new(),
			position: 0,
		}
	}
}

impl State {
	/// Returns true if this is the first image
	pub fn add(&mut self, image: Image) -> bool {
		self.images.push(Arc::new(image));
		self.images.len() == 1
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
	}
}
