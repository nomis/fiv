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

use super::codecs::{Codec, Codecs, Generic};
use super::files::file_err;
use anyhow::Error;
use bytemuck::{cast_slice, cast_slice_mut};
use gtk::cairo;
use log::trace;
use pathdiff::diff_paths;
use std::cell::RefCell;
use std::fs::{read_link, remove_file};
use std::io;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug)]
pub struct Image {
	pub filename: PathBuf,
	pub width: u32,
	pub height: u32,
	codec: Codecs,
	mark_link: Option<Link>,
	marked: Mutex<Option<bool>>,
	data: Mutex<Option<ImageData>>,
	orientation: Mutex<Orientation>,
}

type Pixel = u32;

#[derive(derive_more::Debug)]
pub struct ImageData {
	#[debug("{:?}", data.as_ref().map(|x| Some(x.len())))]
	data: Option<Box<[Pixel]>>,
	format: cairo::Format,
	width: i32,
	height: i32,
	stride: i32,
}

#[derive(derive_more::Debug)]
pub struct ImageDataBuilder {
	#[debug("{}", buffer.len())]
	pub buffer: Box<[Pixel]>,
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
	/// Blocking on CPU, I/O
	pub fn new<P: AsRef<Path>>(
		canonical_mark_directory: Option<&PathBuf>,
		filename: P,
	) -> Result<super::Image, Error> {
		let codec = Codecs::from(Generic::default());
		let metadata = codec.metadata(filename.as_ref())?;
		let path = filename.as_ref().to_path_buf();
		let mark_link = mark_link(canonical_mark_directory, &path);
		let image = Image {
			filename: path,
			width: metadata.width,
			height: metadata.height,
			codec,
			mark_link,
			marked: Mutex::new(None),
			data: Mutex::new(None),
			orientation: Mutex::new(metadata.orientation),
		};

		image.refresh_mark();
		Ok(image)
	}

	/// Blocking on I/O
	pub fn refresh_mark(&self) {
		*self.marked.lock().unwrap() = self.read_mark_link();
	}

	pub fn marked(&self) -> Option<bool> {
		*self.marked.lock().unwrap()
	}

	/// Blocking on I/O
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

	/// Blocking on I/O
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

	/// Blocking on I/O
	fn write_mark_link(&self, mark: bool, suppress_error: bool) {
		if let Some(link) = &self.mark_link {
			if mark {
				symlink(&link.target, &link.name).unwrap_or_else(|err| {
					if err.kind() != io::ErrorKind::AlreadyExists || !suppress_error {
						file_err(&link.name, err);
					}
				});
			} else {
				remove_file(&link.name).unwrap_or_else(|err| {
					if err.kind() != io::ErrorKind::NotFound {
						file_err(&link.name, err);
					}
				});
			}
		}
	}

	/// Blocking on CPU, I/O
	pub fn load(&self) {
		let begin = Instant::now();

		*self.data.lock().unwrap() = Some(
			match self.codec.primary(&self.filename, self.width, self.height) {
				Ok(primary) => primary.image_data,
				Err(err) => {
					file_err(&self.filename, err);
					ImageData::failed()
				}
			},
		);

		trace!(
			"{}: loaded in {:?}",
			self.filename.display(),
			begin.elapsed()
		);
	}

	pub fn loaded(&self) -> bool {
		self.data.lock().unwrap().is_some()
	}

	pub fn unload(&self) {
		*self.data.lock().unwrap() = None;
	}

	pub fn orientation(&self) -> Orientation {
		*self.orientation.lock().unwrap()
	}

	/// Blocks other accesses to image data and load/unload/loaded state
	pub fn with_surface<F: FnOnce(Option<&cairo::ImageSurface>, bool)>(&self, func: F) {
		let mut data = self.data.lock().unwrap();

		match &mut *data {
			Some(data) => data.with_surface(func),
			None => func(None, false),
		}
	}
}

fn mark_link(mark_directory: Option<&PathBuf>, filename: &PathBuf) -> Option<Link> {
	if let Some(directory) = mark_directory {
		match filename.canonicalize() {
			Ok(abs_filename) => {
				if let Some(basename) = filename.file_name() {
					diff_paths(abs_filename, directory).map(|target| Link {
						name: {
							let mut directory = directory.clone();
							directory.push(basename);
							directory
						},
						target,
					})
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

impl Orientation {
	pub fn new(rotate: Rotate, horizontal_flip: bool) -> Self {
		Self {
			rotate,
			horizontal_flip,
		}
	}
}

impl From<image::metadata::Orientation> for Orientation {
	fn from(orientation: image::metadata::Orientation) -> Self {
		match orientation {
			image::metadata::Orientation::NoTransforms => Orientation::new(Rotate::Rotate0, false),
			image::metadata::Orientation::Rotate90 => Orientation::new(Rotate::Rotate90, false),
			image::metadata::Orientation::Rotate180 => Orientation::new(Rotate::Rotate180, false),
			image::metadata::Orientation::Rotate270 => Orientation::new(Rotate::Rotate270, false),
			image::metadata::Orientation::FlipHorizontal => Orientation::new(Rotate::Rotate0, true),
			image::metadata::Orientation::FlipVertical => Orientation::new(Rotate::Rotate180, true),
			image::metadata::Orientation::Rotate90FlipH => Orientation::new(Rotate::Rotate90, true),
			image::metadata::Orientation::Rotate270FlipH => {
				Orientation::new(Rotate::Rotate270, true)
			}
		}
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

impl From<ImageDataBuilder> for ImageData {
	fn from(builder: ImageDataBuilder) -> Self {
		Self {
			data: Some(builder.buffer),
			format: builder.format,
			width: builder.width,
			height: builder.height,
			stride: builder.stride,
		}
	}
}

impl ImageData {
	pub fn builder(width: u32, height: u32) -> ImageDataBuilder {
		assert!(i32::try_from(width).is_ok());
		assert!(i32::try_from(height).is_ok());
		let format = cairo::Format::Rgb24;
		let stride = u32::try_from(format.stride_for_width(width).unwrap()).unwrap();

		assert!(stride as usize == width as usize * size_of::<Pixel>());
		let elements = stride as usize / size_of::<Pixel>() * height as usize;

		ImageDataBuilder {
			buffer: vec![0; elements].into(),
			format,
			width: width.try_into().unwrap(),
			height: height.try_into().unwrap(),
			stride: stride.try_into().unwrap(),
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
	pub fn with_surface<F: FnOnce(Option<&cairo::ImageSurface>, bool)>(&mut self, func: F) {
		// Temporarily move out the pixels
		if let Some(image) = self.data.take() {
			// A new return location that is then passed to our helper struct below
			let return_location = Rc::new(RefCell::new(None));
			{
				let holder = ImageHolder::new(Some(image), return_location.clone());
				let surface = cairo::ImageSurface::create_for_data(
					holder,
					self.format,
					self.width,
					self.height,
					self.stride,
				)
				.expect("Can't create surface");

				func(Some(&surface), true);

				// Now the surface will be destroyed and the pixels are stored in the return location
			}

			// Move the pixels back again
			self.data = Some(
				return_location
					.borrow_mut()
					.take()
					.expect("Image not returned"),
			);
		} else {
			func(None, true);
		}
	}
}

/// Helper struct that allows passing the pixels to the Cairo image surface and once the
/// image surface is destroyed the pixels will be stored in the `return_location`.
///
/// This allows us to give temporary ownership of the pixels to the Cairo surface and later
/// retrieve them back in a safe way while ensuring that nothing else still has access to
/// it.
pub struct ImageHolder {
	image: Option<Box<[Pixel]>>,
	return_location: Rc<RefCell<Option<Box<[Pixel]>>>>,
}

impl ImageHolder {
	pub fn new(
		image: Option<Box<[Pixel]>>,
		return_location: Rc<RefCell<Option<Box<[Pixel]>>>>,
	) -> Self {
		Self {
			image,
			return_location,
		}
	}
}

/// This stores the pixels back into the `return_location` as now nothing
/// references the pixels
impl Drop for ImageHolder {
	fn drop(&mut self) {
		*self.return_location.borrow_mut() = Some(self.image.take().expect("Holding no image"));
	}
}

impl AsRef<[u8]> for ImageHolder {
	fn as_ref(&self) -> &[u8] {
		cast_slice::<Pixel, u8>(self.image.as_ref().expect("Holding no image").as_ref())
	}
}

/// Needed for `cairo::ImageSurface::create_for_data()` to be able to access the pixels
impl AsMut<[u8]> for ImageHolder {
	fn as_mut(&mut self) -> &mut [u8] {
		cast_slice_mut::<Pixel, u8>(self.image.as_mut().expect("Holding no image").as_mut())
	}
}
