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

use anyhow::Error;
use image::ImageReader;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Image {
	pub filename: PathBuf,
	pub _width: u32,
	pub _height: u32,
}

impl Image {
	pub fn new<P: AsRef<Path>>(filename: P) -> Result<super::Image, Error> {
		let image = ImageReader::open(&filename)?.with_guessed_format()?;
		let (width, height) = image.into_dimensions()?;

		Ok(Image {
			filename: filename.as_ref().to_path_buf(),
			_width: width,
			_height: height,
		})
	}
}
