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
use rayon::{prelude::*, ThreadPool};
use std::fmt::Display;
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};

#[derive(Debug)]
pub struct Files {
	pool: Arc<ThreadPool>,
	args: Arc<CommandLineArgs>,
	images: Mutex<Vec<Image>>,

	/// start() has finished or loaded at least one image
	start_ready: (Mutex<bool>, Condvar),
}

pub fn file_err<P: AsRef<Path>, E: Display>(path: P, err: E) {
	eprintln!("{}: {}", path.as_ref().display(), err);
}

impl Files {
	pub fn new(pool: Arc<ThreadPool>, args: Arc<CommandLineArgs>) -> Arc<Files> {
		Arc::new(Files {
			pool,
			args,
			images: Mutex::new(Vec::new()),
			start_ready: (Mutex::new(false), Condvar::new()),
		})
	}

	pub fn start(self: &Arc<Self>) -> bool {
		let self_copy = self.clone();

		self.pool.install(move || {
			let filenames = CommandLineFilenames::new(&self_copy.args);

			filenames
				.par_bridge()
				.filter_map(|filename| match Image::new(&filename) {
					Err(err) => {
						file_err(filename, err);
						None
					}

					Ok(image) => Some(image),
				})
				.for_each(|image| self.load(image));

			self_copy.start_set_ready();
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
}
