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

use super::CommandLineArgs;
use super::Image;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

#[derive(Debug)]
pub struct Files {
	args: Arc<CommandLineArgs>,
	images: Mutex<Vec<Image>>,

	/// start() has finished or loaded at least one image
	start_ready: (Mutex<bool>, Condvar),
}

fn file_err<P: AsRef<Path>, E: std::error::Error>(path: P, err: E) {
	eprintln!("{}: {}", path.as_ref().display(), err);
}

fn sorted_dir_list(path: &Path) -> Vec<PathBuf> {
	match fs::read_dir(path) {
		Err(err) => {
			file_err(path, err);
			Vec::<PathBuf>::new()
		}

		Ok(dir) => {
			let mut files: Vec<PathBuf> = dir
				.flat_map(|res| {
					res.map(|entry| entry.path().to_path_buf())
						.map_err(|err| file_err(path, err))
				})
				.collect();
			files.sort();
			files
		}
	}
}

impl Files {
	pub fn new(args: Arc<CommandLineArgs>) -> Arc<Files> {
		Arc::new(Files {
			args,
			images: Mutex::new(Vec::new()),
			start_ready: (Mutex::new(false), Condvar::new()),
		})
	}

	pub fn start(self: &Arc<Self>) -> bool {
		let self_copy = self.clone();

		thread::spawn(move || {
			for file in &self_copy.args.filenames {
				self_copy.load(file, true)
			}

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

	fn load(&self, filename: &PathBuf, recurse: bool) {
		match fs::metadata(filename) {
			Err(err) => file_err(filename, err),

			Ok(metadata) => {
				if metadata.is_file() {
					let mut images = self.images.lock().unwrap();

					images.push(Image::new(filename));

					if images.len() == 1 {
						self.start_set_ready();
					}
				} else if recurse && metadata.is_dir() {
					for filename in &sorted_dir_list(filename) {
						self.load(filename, false);
					}
				}
			}
		}
	}
}
