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

use std::io::{BufReader, Cursor};

use super::{Codec, CodecMetadata, CodecPrimary, Generic, ImageData};
use crate::fiv::numeric::DimensionsU32;
use anyhow::{Error, ensure};
use image::{DynamicImage, ImageDecoder, ImageReader};

impl Codec for Generic {
	fn metadata(&self, file: &[u8]) -> Result<CodecMetadata, Error> {
		let mut decoder = ImageReader::new(BufReader::new(Cursor::new(file)))
			.with_guessed_format()?
			.into_decoder()?;

		Ok(CodecMetadata {
			dimensions: decoder.dimensions().into(),
			orientation: decoder.orientation().unwrap().into(),
			af_points: None,
		})
	}

	fn primary(&self, file: &[u8], metadata: &CodecMetadata) -> Result<CodecPrimary, Error> {
		let decoder = ImageReader::new(BufReader::new(Cursor::new(file)))
			.with_guessed_format()?
			.into_decoder()?;

		let dimensions: DimensionsU32 = decoder.dimensions().into();

		ensure!(
			dimensions == metadata.dimensions,
			"Image dimensions have changed: {} != {}",
			dimensions,
			metadata.dimensions,
		);

		let image = DynamicImage::from_decoder(decoder)?.into_rgb8();
		let samples = image.as_flat_samples().samples;
		let mut image_data = ImageData::builder(dimensions)?;

		// Decoding images as RGB and then converting them to XBGR adds 33% to
		// the total time compared to decoding to XBGR directly ☹️
		for (src, dst) in samples.chunks_exact(3).zip(image_data.iter_mut()) {
			*dst = (u32::from(src[0]) << 16) | (u32::from(src[1]) << 8) | u32::from(src[2]);
		}

		Ok(CodecPrimary {
			image_data: image_data.into(),
		})
	}
}
