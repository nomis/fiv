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

use std::io::{BufReader, Cursor};

use super::{Codec, CodecMetadata, CodecPrimary, ImageData, Jpeg};
use crate::fiv::numeric::DimensionsU32;
use anyhow::{Error, anyhow};

impl TryFrom<turbojpeg::DecompressHeader> for DimensionsU32 {
	type Error = Error;

	fn try_from(header: turbojpeg::DecompressHeader) -> Result<Self, Error> {
		Ok(DimensionsU32::new(
			u32::try_from(header.width)?.into(),
			u32::try_from(header.height)?.into(),
		))
	}
}

impl Codec for Jpeg {
	fn metadata(&self, file: &[u8]) -> Result<CodecMetadata, Error> {
		let header = turbojpeg::read_header(file)?;
		let exif = exif::Reader::new()
			.read_from_container(&mut BufReader::new(Cursor::new(file)))
			.ok();

		Ok(CodecMetadata {
			dimensions: DimensionsU32::try_from(header)?,
			orientation: exif.into(),
		})
	}

	fn primary(&self, file: &[u8], metadata: &CodecMetadata) -> Result<CodecPrimary, Error> {
		let mut decompressor = turbojpeg::Decompressor::new()?;
		let header = decompressor.read_header(file)?;
		let dimensions = DimensionsU32::try_from(header)?;

		if dimensions != metadata.dimensions {
			Err(anyhow!(
				"Image dimensions have changed: {} != {}",
				dimensions,
				metadata.dimensions,
			))?;
		}

		let mut image_data = ImageData::builder(dimensions)?;
		let pitch = usize::try_from(image_data.stride)?;
		let mut image = turbojpeg::Image {
			pixels: image_data.as_mut(),
			width: header.width,
			pitch,
			height: header.height,
			format: if cfg!(target_endian = "little") {
				turbojpeg::PixelFormat::BGRX
			} else {
				turbojpeg::PixelFormat::XRGB
			},
		};

		decompressor.decompress(file, image.as_deref_mut())?;

		Ok(CodecPrimary {
			image_data: image_data.into(),
		})
	}
}
