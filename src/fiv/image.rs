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
use gtk::cairo;
use image::ImageReader;
use pathdiff::diff_paths;
use std::cell::RefCell;
use std::fs::{read_link, remove_file};
use std::io;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Mutex;

#[derive(Debug)]
pub struct Image {
	pub filename: PathBuf,
	pub width: u32,
	pub height: u32,
	mark_link: Option<Link>,
	marked: Mutex<Option<bool>>,
	data: Mutex<Option<ImageData>>,
	orientation: Mutex<Orientation>,
}

#[derive(Debug)]
pub struct ImageData {
	data: Option<Box<[u8]>>,
	format: cairo::Format,
	width: i32,
	height: i32,
	stride: i32,
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

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Orientation {
	pub rotate: Rotate,
	pub horizontal_flip: bool,
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum Rotate {
	#[default]
	Rotate0,
	Rotate90,
	Rotate180,
	Rotate270,
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
			width,
			height,
			mark_link,
			marked: Mutex::new(None),
			data: Mutex::new(None),
			orientation: Mutex::new(Orientation::default()),
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

	pub fn loaded(&self) -> bool {
		self.data.lock().unwrap().is_some()
	}

	pub fn orientation(&self) -> Orientation {
		*self.orientation.lock().unwrap()
	}

	pub fn with_surface<F: FnOnce(&Option<cairo::ImageSurface>, bool)>(&self, func: F) {
		let mut data = self.data.lock().unwrap();

		match &mut *data {
			Some(data) => data.with_surface(func),
			None => func(&None, false),
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

/*
 * ImageData, ImageHolder based on https://github.com/gtk-rs/gtk3-rs/examples/cairo_threads/image/
 *
 * Copyright 2021 Julian Hofer
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

impl ImageData {
	pub fn new(format: cairo::Format, width: u32, height: u32) -> Self {
		assert!(width <= i32::MAX as u32);
		assert!(height <= i32::MAX as u32);
		let stride = format.stride_for_width(width).unwrap();

		Self {
			data: Some(vec![0; stride as usize * height as usize].into()),
			format,
			width: width.try_into().unwrap(),
			height: height.try_into().unwrap(),
			stride,
		}
	}

	pub fn failed() -> Self {
		Self {
			data: None,
			format: cairo::Format::Invalid,
			width: -1,
			height: -1,
			stride: -1,
		}
	}

	/// Calls the given closure with a temporary Cairo image surface. After the closure has returned
	/// there must be no further references to the surface.
	pub fn with_surface<F: FnOnce(&Option<cairo::ImageSurface>, bool)>(&mut self, func: F) {
		// Temporarily move out the pixels
		if let Some(image) = self.data.take() {
			// A new return location that is then passed to our helper struct below
			let return_location = Rc::new(RefCell::new(None));
			{
				let holder = ImageHolder::new(Some(image), return_location.clone());
				let surface = Some(
					cairo::ImageSurface::create_for_data(
						holder,
						self.format,
						self.width,
						self.height,
						self.stride,
					)
					.expect("Can't create surface"),
				);

				func(&surface, true);

				// Now the surface will be destroyed and the pixels are stored in the return_location
			}

			// Move the pixels back again
			self.data = Some(
				return_location
					.borrow_mut()
					.take()
					.expect("Image not returned"),
			);
		} else {
			func(&None, true);
		}
	}
}

/// Helper struct that allows passing the pixels to the Cairo image surface and once the
/// image surface is destroyed the pixels will be stored in the return_location.
///
/// This allows us to give temporary ownership of the pixels to the Cairo surface and later
/// retrieve them back in a safe way while ensuring that nothing else still has access to
/// it.
pub struct ImageHolder {
	image: Option<Box<[u8]>>,
	return_location: Rc<RefCell<Option<Box<[u8]>>>>,
}

impl ImageHolder {
	pub fn new(image: Option<Box<[u8]>>, return_location: Rc<RefCell<Option<Box<[u8]>>>>) -> Self {
		Self {
			image,
			return_location,
		}
	}
}

/// This stores the pixels back into the return_location as now nothing
/// references the pixels anymore
impl Drop for ImageHolder {
	fn drop(&mut self) {
		*self.return_location.borrow_mut() = Some(self.image.take().expect("Holding no image"));
	}
}

impl AsRef<[u8]> for ImageHolder {
	fn as_ref(&self) -> &[u8] {
		self.image.as_ref().expect("Holding no image").as_ref()
	}
}

/// Needed for `cairo::ImageSurface::create_for_data()` to be able to access the pixels
impl AsMut<[u8]> for ImageHolder {
	fn as_mut(&mut self) -> &mut [u8] {
		self.image.as_mut().expect("Holding no image").as_mut()
	}
}
