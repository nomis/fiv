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
use std::sync::Mutex;

#[derive(Debug)]
pub struct Files<'app> {
	args: &'app CommandLineArgs,
	images: Mutex<Vec<Image>>,
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

impl Files<'_> {
	pub fn new<'app>(args: &'app CommandLineArgs) -> Files<'app> {
		Files::<'app> {
			args,
			images: Mutex::new(Vec::new()),
		}
	}

	pub fn start(&self) -> bool {
		for file in &self.args.files {
			self.load(file, true)
		}

		!self.images.lock().unwrap().is_empty()
	}

	fn load(&self, file: &PathBuf, recurse: bool) {
		match fs::metadata(file) {
			Err(err) => file_err(file, err),

			Ok(metadata) => {
				if metadata.is_file() {
					self.images.lock().unwrap().push(Image::new(file))
				} else if recurse && metadata.is_dir() {
					for file in &sorted_dir_list(file) {
						self.load(file, false)
					}
				}
			}
		}
	}
}
