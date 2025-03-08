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

use super::{Codec, CodecMetadata, CodecPrimary, Heif, ImageData};
use crate::fiv::{Orientation, numeric::DimensionsU32};
use anyhow::{Error, ensure};
use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};
use std::sync::LazyLock;

impl From<&libheif_rs::ImageHandle> for DimensionsU32 {
	fn from(handle: &libheif_rs::ImageHandle) -> Self {
		DimensionsU32::new(handle.width().into(), handle.height().into())
	}
}

static LIB_HEIF: LazyLock<LibHeif> = LazyLock::new(LibHeif::new);

impl Codec for Heif {
	fn metadata(&self, file: &[u8]) -> Result<CodecMetadata, Error> {
		LazyLock::force(&super::EXIV2_INIT);
		LazyLock::force(&LIB_HEIF);
		let context = HeifContext::read_from_bytes(file)?;
		let handle = context.primary_image_handle()?;
		let exiv = rexiv2::Metadata::new_from_buffer(file).ok();
		let dimensions = DimensionsU32::from(&handle);
		let orientation = Orientation::from(exiv.as_ref());

		Ok(CodecMetadata {
			dimensions,
			orientation,
			af_points: None,
		})
	}

	fn primary(&self, file: &[u8], metadata: &CodecMetadata) -> Result<CodecPrimary, Error> {
		let context = HeifContext::read_from_bytes(file)?;
		let handle = context.primary_image_handle()?;
		let dimensions = DimensionsU32::from(&handle);

		ensure!(
			dimensions == metadata.dimensions,
			"Image dimensions have changed: {} != {}",
			dimensions,
			metadata.dimensions,
		);

		let mut image_data = ImageData::builder(dimensions)?;
		let image = LIB_HEIF.decode(&handle, ColorSpace::Rgb(RgbChroma::Rgb), None)?;
		let plane = image.planes().interleaved.unwrap();
		let pixel_stride = usize::try_from(i32::from(image_data.width)).unwrap();

		// Decoding images as RGB and then converting them to XBGR wastes time
		// compared to decoding to XBGR directly ☹️ (and libheif's stride varies)
		for (src_row, dst_row) in plane
			.data
			.chunks_exact(plane.stride)
			.zip(AsMut::<[u32]>::as_mut(&mut image_data).chunks_exact_mut(pixel_stride))
		{
			for (src, dst) in src_row.chunks_exact(3).zip(dst_row.iter_mut()) {
				*dst = (u32::from(src[0]) << 16) | (u32::from(src[1]) << 8) | u32::from(src[2]);
			}
		}

		Ok(CodecPrimary {
			image_data: image_data.into(),
		})
	}
}
