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

use clap::Parser;

#[derive(Debug, Parser)]
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
	pub mark_directory: Option<std::path::PathBuf>,

	/// Image files or directories of image files to display
	#[arg(value_names = ["FILE"], default_value = ".")]
	pub files: Vec<std::path::PathBuf>,
}
