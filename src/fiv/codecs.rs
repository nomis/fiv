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

mod generic;
mod heif;
mod jpeg;

use super::{Orientation, image::AFPoint, image::ImageData, numeric::DimensionsU32};
use anyhow::{Error, anyhow};
use enum_dispatch::enum_dispatch;
use std::{fmt, sync::LazyLock};

/// Exiv2 initialisation is not thread-safe
static EXIV2_INIT: LazyLock<()> = LazyLock::new(|| rexiv2::initialize().unwrap());

#[enum_dispatch]
pub trait Codec {
	fn metadata(&self, file: &[u8]) -> Result<CodecMetadata, Error>;
	fn primary(&self, file: &[u8], metadata: &CodecMetadata) -> Result<CodecPrimary, Error>;
}

#[derive(Debug)]
pub struct CodecMetadata {
	pub dimensions: DimensionsU32,
	pub orientation: Orientation,
	pub af_points: Option<Vec<AFPoint>>,
}

#[derive(Debug)]
pub struct CodecPrimary {
	pub image_data: ImageData,
}

#[enum_dispatch(Codec)]
#[derive(strum::AsRefStr)]
pub enum Codecs {
	Generic,
	Heif,
	Jpeg,
}

impl Codecs {
	pub fn new(file: &[u8]) -> Result<Self, Error> {
		let mime_type = tree_magic_mini::from_u8(file);

		if let Some(codec) = match mime_type {
			"image/avif" | "image/heic" | "image/heif" => Some(Codecs::from(Heif::default())),
			"image/jpeg" => Some(Codecs::from(Jpeg::default())),
			_ => None,
		} {
			Ok(codec)
		} else if mime_type.starts_with("image/") {
			Ok(Codecs::from(Generic::default()))
		} else {
			Err(anyhow!("Unsupported type {}", mime_type))
		}
	}
}

impl fmt::Debug for Codecs {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_ref())
	}
}

#[derive(Debug, Default)]
pub struct Generic {}

#[derive(Debug, Default)]
pub struct Heif {}

#[derive(Debug, Default)]
pub struct Jpeg {}
