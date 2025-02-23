/*
 * fiv - Fast Image Viewer
 * Copyright 2025  Simon Arlott
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

use super::{Codec, CodecMetadata, CodecPrimary, Generic, ImageData};
use crate::fiv::numeric::DimensionsU32;
use anyhow::{anyhow, Error};
use image::{DynamicImage, ImageDecoder, ImageReader};
use std::path::Path;

impl Codec for Generic {
	fn metadata(&self, filename: &Path) -> Result<CodecMetadata, Error> {
		let mut decoder = ImageReader::open(filename)?
			.with_guessed_format()?
			.into_decoder()?;

		Ok(CodecMetadata {
			dimensions: decoder.dimensions().into(),
			orientation: decoder.orientation().unwrap().into(),
		})
	}

	fn primary(&self, filename: &Path, metadata: &CodecMetadata) -> Result<CodecPrimary, Error> {
		let decoder = ImageReader::open(filename)?
			.with_guessed_format()?
			.into_decoder()?;

		let dimensions: DimensionsU32 = decoder.dimensions().into();

		if dimensions != metadata.dimensions {
			Err(anyhow!(
				"Image dimensions have changed: {} != {}",
				dimensions,
				metadata.dimensions,
			))?;
		}

		let image = DynamicImage::from_decoder(decoder)?.into_rgb8();
		let samples = image.as_flat_samples().samples;
		let mut image_data = ImageData::builder(dimensions);

		for (src, dst) in samples.chunks_exact(3).zip(image_data.buffer.iter_mut()) {
			*dst = (u32::from(src[0]) << 16) | (u32::from(src[1]) << 8) | u32::from(src[2]);
		}

		Ok(CodecPrimary {
			image_data: image_data.into(),
		})
	}
}
