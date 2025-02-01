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

use super::{CommandLineArgs, CommandLineFilenames, Image};
use pariter::IteratorExt;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};

#[derive(Debug)]
pub struct Files {
	args: Arc<CommandLineArgs>,
	images: Mutex<Vec<Image>>,
	position: AtomicUsize,

	/// start() has finished or loaded at least one image
	start_ready: (Mutex<bool>, Condvar),
	start_finished: (Mutex<bool>, Condvar),
}

#[derive(Debug)]
pub struct Current {
	pub filename: PathBuf,
	pub position: usize,
	pub total: usize,
	pub loading: bool,
}

pub fn file_err<P: AsRef<Path>, E: Display>(path: P, err: E) {
	eprintln!("{}: {}", path.as_ref().display(), err);
}

impl Default for Files {
	fn default() -> Files {
		Files {
			args: Arc::new(CommandLineArgs::default()),
			images: Mutex::new(Vec::new()),
			position: AtomicUsize::new(0),
			start_ready: (Mutex::new(false), Condvar::new()),
			start_finished: (Mutex::new(false), Condvar::new()),
		}
	}
}

impl Files {
	pub fn new(args: Arc<CommandLineArgs>) -> Arc<Files> {
		Arc::new(Files {
			args,
			..Default::default()
		})
	}

	pub fn start(self: &Arc<Self>) -> bool {
		let self_copy = self.clone();

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

				self_copy.start_set_ready();
				self_copy.start_set_finished();
			})
			.unwrap();
		});

		self.wait_for_start_ready();
		!self.images.lock().unwrap().is_empty()
	}

	fn start_set_ready(&self) {
		let (lock, cv) = &self.start_ready;
		let mut result = lock.lock().unwrap();

		*result = true;
		cv.notify_all();
	}

	fn start_set_finished(&self) {
		let (lock, cv) = &self.start_finished;
		let mut result = lock.lock().unwrap();

		*result = true;
		cv.notify_all();
	}

	fn wait_for_start_ready(&self) {
		let (lock, cv) = &self.start_ready;
		let mut result = lock.lock().unwrap();

		while !*result {
			result = cv.wait(result).unwrap();
		}
	}

	fn load(&self, image: Image) {
		let mut images = self.images.lock().unwrap();

		images.push(image);

		if images.len() == 1 {
			self.start_set_ready();
		}
	}

	pub fn current(&self) -> Current {
		let images = self.images.lock().unwrap();
		let position = self.position.load(Ordering::Acquire);
		let image = &images[position];

		Current {
			filename: image.filename.clone(),
			position: position + 1,
			total: images.len(),
			loading: !*self.start_finished.0.lock().unwrap(),
		}
	}
}
