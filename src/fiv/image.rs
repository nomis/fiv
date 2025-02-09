/*
 * fiv - Fast Image Viewer
 * Copyright 2015,2025  Simon Arlott
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

use super::files::file_err;
use anyhow::Error;
use image::ImageReader;
use pathdiff::diff_paths;
use std::fs::{read_link, remove_file};
use std::io;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug)]
pub struct Image {
	pub filename: PathBuf,
	pub _width: u32,
	pub _height: u32,
	mark_link: Option<Link>,
	marked: Mutex<Option<bool>>,
}

#[derive(Debug)]
struct Link {
	name: PathBuf,
	target: PathBuf,
}

#[derive(Debug, Copy, Clone)]
pub enum Mark {
	Set,
	Toggle,
	Unset,
}

impl Image {
	pub fn new<P: AsRef<Path>>(
		canonical_mark_directory: &Option<PathBuf>,
		filename: P,
	) -> Result<super::Image, Error> {
		let reader = ImageReader::open(&filename)?.with_guessed_format()?;
		let path = filename.as_ref().to_path_buf();
		let mark_link = mark_link(&canonical_mark_directory, &path);
		let (width, height) = reader.into_dimensions()?;
		let image = Image {
			filename: path,
			_width: width,
			_height: height,
			mark_link,
			marked: Mutex::new(None),
		};

		image.refresh_mark();
		Ok(image)
	}

	pub fn refresh_mark(&self) {
		*self.marked.lock().unwrap() = self.read_mark_link();
	}

	pub fn marked(&self) -> Option<bool> {
		*self.marked.lock().unwrap()
	}

	pub fn mark(&self, mark: Mark) {
		let mut marked = self.marked.lock().unwrap();

		self.write_mark_link(
			match mark {
				Mark::Set => true,
				Mark::Toggle => !self.read_mark_link().unwrap_or(false),
				Mark::Unset => false,
			},
			marked.unwrap_or(false),
		);

		*marked = self.read_mark_link();
	}

	fn read_mark_link(&self) -> Option<bool> {
		self.mark_link
			.as_ref()
			.and_then(|link| match read_link(&link.name) {
				Err(err) => {
					if err.kind() == io::ErrorKind::NotFound {
						Some(false)
					} else {
						file_err(&link.name, err);
						None
					}
				}
				Ok(target) => {
					if target == link.target {
						Some(true)
					} else {
						None
					}
				}
			})
	}

	fn write_mark_link(&self, mark: bool, suppress_error: bool) {
		if let Some(link) = &self.mark_link {
			if mark {
				symlink(&link.target, &link.name).unwrap_or_else(|err| {
					if err.kind() != io::ErrorKind::AlreadyExists || !suppress_error {
						file_err(&link.name, err)
					}
				});
			} else {
				remove_file(&link.name).unwrap_or_else(|err| {
					if err.kind() != io::ErrorKind::NotFound {
						file_err(&link.name, err)
					}
				});
			}
		}
	}
}

fn mark_link(mark_directory: &Option<PathBuf>, filename: &PathBuf) -> Option<Link> {
	if let Some(mut directory) = mark_directory.clone() {
		match filename.canonicalize() {
			Ok(abs_filename) => {
				if let Some(basename) = filename.file_name() {
					if let Some(target) = diff_paths(abs_filename, &directory) {
						directory.push(basename);
						Some(Link {
							name: directory,
							target,
						})
					} else {
						// One of the arguments is not absolute, which can't happen
						None
					}
				} else {
					// Image filename ends in "..", which can't happen
					None
				}
			}

			Err(err) => {
				file_err(filename, err);
				None
			}
		}
	} else {
		None
	}
}
