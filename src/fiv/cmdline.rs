/*
 * fiv - Fast Image Viewer
 * Copyright 2015,2018,2025  Simon Arlott
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
use clap::Parser;
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Default, Parser)]
#[command(
	version,
	display_name = "Fast Image Viewer",
	about = "Display image files"
)]
pub struct Args {
	/// Number of images to preload
	#[arg(short, long, value_names = ["COUNT"], default_value_t = 100)]
	pub preload: u32,

	/// Location to use to mark images using symlinks
	#[arg(short, long, value_names = ["PATH"])]
	pub mark_directory: Option<PathBuf>,

	/// Image files or directories of image files to display
	#[arg(value_names = ["FILE"], default_value = ".")]
	pub filenames: Vec<PathBuf>,
}

pub struct Filenames<'a> {
	filenames: core::slice::Iter<'a, PathBuf>,
	dir_filenames: Option<VecDeque<PathBuf>>,
}

impl Filenames<'_> {
	pub fn new(args: &Arc<Args>) -> Filenames<'_> {
		Filenames {
			filenames: args.filenames.iter(),
			dir_filenames: None,
		}
	}
}

impl Iterator for Filenames<'_> {
	type Item = PathBuf;

	/// Return filenames that are accessible files and sorted accessible files
	/// within filenames that are accessible directories (without recursion)
	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let (filename, recurse) = match &mut self.dir_filenames {
				None => match self.filenames.next() {
					None => {
						return None;
					}

					Some(filename) => (filename, true),
				},

				Some(dir_filenames) => (
					&match dir_filenames.pop_front() {
						None => {
							self.dir_filenames = None;
							continue;
						}

						Some(filename) => filename,
					},
					false,
				),
			};

			match fs::metadata(filename) {
				Err(err) => {
					file_err(filename, err);
					continue;
				}

				Ok(metadata) => {
					if metadata.is_file() {
						return Some(filename.to_path_buf());
					} else if metadata.is_dir() && recurse {
						self.dir_filenames = Some(sorted_dir_list(filename));
						continue;
					} else {
						continue;
					}
				}
			}
		}
	}
}

fn sorted_dir_list(path: &Path) -> VecDeque<PathBuf> {
	match fs::read_dir(path) {
		Err(err) => {
			file_err(path, err);
			VecDeque::<PathBuf>::new()
		}

		Ok(dir) => {
			let mut files: VecDeque<PathBuf> = dir
				.flat_map(|res| {
					res.map(|entry| entry.path().to_path_buf())
						.map_err(|err| file_err(path, err))
				})
				.collect();
			files.make_contiguous().sort();
			files
		}
	}
}
