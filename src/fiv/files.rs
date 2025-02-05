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

use super::{CommandLineArgs, CommandLineFilenames, Image, Waitable};
use async_notify::Notify;
use log::debug;
use pariter::IteratorExt;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug)]
pub struct Files {
	args: Arc<CommandLineArgs>,
	state: Mutex<State>,
	notify: Notify,

	/// start() has finished or loaded at least one image
	start_ready: Waitable<bool>,
	start_finished: Waitable<bool>,
}

#[derive(Debug, Default)]
pub struct Current {
	pub filename: PathBuf,
	pub position: usize,
	pub total: usize,
}

pub fn file_err<P: AsRef<Path>, E: Display>(path: P, err: E) {
	eprintln!("{}: {}", path.as_ref().display(), err);
}

impl Files {
	pub fn new(args: Arc<CommandLineArgs>) -> Arc<Files> {
		Arc::new(Files {
			args,
			state: Mutex::new(State::default()),
			notify: Notify::new(),
			start_ready: Waitable::new(false),
			start_finished: Waitable::new(false),
		})
	}

	pub fn start(self: &Arc<Self>) -> bool {
		let self_copy = self.clone();
		self.state.lock().unwrap().start_begin = Instant::now();

		std::thread::spawn(move || {
			pariter::scope(|scope| {
				CommandLineFilenames::new(&self_copy.args)
					.parallel_map_scoped(scope, |filename| match Image::new(&filename) {
						Err(err) => {
							file_err(filename, err);
							None
						}

						Ok(image) => Some(image),
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

	pub async fn ui_wait(&self) {
		self.notify.notified().await;
	}

	pub fn update_ui(&self) {
		self.notify.notify();
	}

	pub fn current(&self) -> Current {
		self.state.lock().unwrap().current()
	}
}

#[derive(Debug)]
struct State {
	start_begin: Instant,
	images: Vec<Image>,
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
		self.images.push(image);
		self.images.len() == 1
	}

	pub fn current(&self) -> Current {
		if let Some(image) = self.images.get(self.position) {
			Current {
				filename: image.filename.clone(),
				position: self.position + 1,
				total: self.images.len(),
			}
		} else {
			Current::default()
		}
	}
}
