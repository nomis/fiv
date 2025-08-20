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

use super::{Codec, CodecMetadata, CodecPrimary, ImageData, Jpeg};
use crate::fiv::{
	AFPoint, ByteOrder, Orientation, byte_order_of,
	numeric::{DimensionsF64, DimensionsU32, PointF64, Xf64, Xu32, Yf64, Yu32},
};
use anyhow::{Error, anyhow, ensure};
use bitfield::Bit;
use std::sync::LazyLock;

impl TryFrom<&turbojpeg::DecompressHeader> for DimensionsU32 {
	type Error = Error;

	fn try_from(header: &turbojpeg::DecompressHeader) -> Result<Self, Error> {
		Ok(DimensionsU32::new(
			u32::try_from(header.width)?.into(),
			u32::try_from(header.height)?.into(),
		))
	}
}

impl Codec for Jpeg {
	fn metadata(&self, file: &[u8]) -> Result<CodecMetadata, Error> {
		LazyLock::force(&super::EXIV2_INIT);
		let header = turbojpeg::read_header(file)?;
		let exiv = rexiv2::Metadata::new_from_buffer(file).ok();
		let dimensions = DimensionsU32::try_from(&header)?;
		let orientation = Orientation::from(exiv.as_ref());
		let af_points = exiv.and_then(|exiv| read_canon_af_points(dimensions, &exiv).ok());

		Ok(CodecMetadata {
			dimensions,
			orientation,
			af_points,
		})
	}

	fn primary(&self, file: &[u8], metadata: &CodecMetadata) -> Result<CodecPrimary, Error> {
		let mut decompressor = turbojpeg::Decompressor::new()?;
		let header = decompressor.read_header(file)?;
		let dimensions = DimensionsU32::try_from(&header)?;

		ensure!(
			dimensions == metadata.dimensions,
			"Image dimensions have changed: {} != {}",
			dimensions,
			metadata.dimensions,
		);

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

#[derive(Debug, derive_more::Constructor)]
struct CanonAFVec {
	data: Vec<u8>,
	bo: ByteOrder,
}

impl CanonAFVec {
	fn get<F: FnOnce() -> Error>(&self, index: usize, err: F) -> Result<[u8; 2], Error> {
		match self.data.get(2 * index) {
			Some(b0) => match self.data.get(2 * index + 1) {
				Some(b1) => Ok([*b0, *b1]),
				None => anyhow::bail!(err()),
			},
			None => anyhow::bail!(err()),
		}
	}

	pub fn get_i16<F: FnOnce() -> Error>(&self, index: usize, err: F) -> Result<i16, Error> {
		let bytes = self.get(index, err)?;
		Ok(match self.bo {
			ByteOrder::BE => i16::from_be_bytes(bytes),
			ByteOrder::LE => i16::from_le_bytes(bytes),
		})
	}

	pub fn get_u16<F: FnOnce() -> Error>(&self, index: usize, err: F) -> Result<u16, Error> {
		let bytes = self.get(index, err)?;
		Ok(match self.bo {
			ByteOrder::BE => u16::from_be_bytes(bytes),
			ByteOrder::LE => u16::from_le_bytes(bytes),
		})
	}

	pub fn get_u32<F: FnOnce() -> Error>(&self, index: usize, err: F) -> Result<u32, Error> {
		Ok(u32::from(self.get_u16(index, err)?))
	}

	pub fn get_usize<F: FnOnce() -> Error>(&self, index: usize, err: F) -> Result<usize, Error> {
		Ok(usize::from(self.get_u16(index, err)?))
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn get_dimensions_u32<Fx: FnOnce() -> Error, Fy: FnOnce() -> Error>(
		&self,
		x_index: usize,
		x_err: Fx,
		y_index: usize,
		y_err: Fy,
	) -> Result<DimensionsU32, Error> {
		Ok(DimensionsU32::new(
			Xu32::from(self.get_u32(x_index, x_err)?),
			Yu32::from(self.get_u32(y_index, y_err)?),
		))
	}

	pub fn get_dimensions_f64<Fx: FnOnce() -> Error, Fy: FnOnce() -> Error>(
		&self,
		x_index: usize,
		x_err: Fx,
		y_index: usize,
		y_err: Fy,
	) -> Result<DimensionsF64, Error> {
		Ok(DimensionsF64::from(
			self.get_dimensions_u32(x_index, x_err, y_index, y_err)?,
		))
	}

	pub fn get_point_f64<Fx: FnOnce() -> Error, Fy: FnOnce() -> Error>(
		&self,
		x_index: usize,
		x_err: Fx,
		y_index: usize,
		y_err: Fy,
	) -> Result<PointF64, Error> {
		Ok(PointF64::new(
			Xf64::try_from(f64::from(self.get_i16(x_index, x_err)?)).unwrap(),
			0.0 - Yf64::try_from(f64::from(self.get_i16(y_index, y_err)?)).unwrap(),
		))
	}

	pub fn get_bit<F: FnOnce() -> Error>(
		&self,
		index: usize,
		bit: usize,
		err: F,
	) -> Result<bool, Error> {
		let index = 2 * index
			+ match self.bo {
				// [0-7, 8-15, 16-23, 24-31, ...]
				ByteOrder::LE => bit / 8,

				// [8-15, 0-7, 24-31, 16-23, ...]
				ByteOrder::BE => 2 * (bit / 16) + (((bit / 8) & 1) ^ 1),
			};

		Ok(self.data.get(index).ok_or_else(err)?.bit(bit % 8))
	}
}

fn read_canon_af_points(
	dimensions: DimensionsU32,
	exiv: &rexiv2::Metadata,
) -> Result<Vec<AFPoint>, Error> {
	let af_info = CanonAFVec::new(
		exiv.get_tag_raw("Exif.Canon.AFInfo")
			.map_err(|_| anyhow!("Exif.Canon.AFInfo not found"))?,
		byte_order_of(exiv)?,
	);

	let count = af_info.get_usize(0, || anyhow!("Missing AFInfoSize"))?;

	ensure!(
		count == af_info.len(),
		"Invalid count {count} != length of data {}",
		af_info.len()
	);

	let _af_area_mode = af_info.get_u16(1, || anyhow!("Missing AFAreaMode"))?;
	let num_af_points = af_info.get_usize(2, || anyhow!("Missing NumAFPoints"))?;
	let num_af_bitfields = num_af_points.div_ceil(16);
	let valid_af_points = af_info.get_usize(3, || anyhow!("Missing ValidAFPoints"))?;
	let img_dimensions = af_info.get_dimensions_u32(
		4,
		|| anyhow!("Missing CanonImageWidth"),
		5,
		|| anyhow!("Missing CanonImageHeight"),
	)?;
	let af_img_dimensions = af_info.get_dimensions_f64(
		6,
		|| anyhow!("Missing AFImageWidth"),
		7,
		|| anyhow!("Missing AFImageHeight"),
	)?;

	ensure!(
		img_dimensions == dimensions,
		"Image dimensions don't match: {img_dimensions} != {dimensions}"
	);

	ensure!(
		img_dimensions.non_zero(),
		"Image dimensions are zero: {img_dimensions}"
	);

	let img_dimensions = DimensionsF64::from(img_dimensions);
	let af_img_centre = af_img_dimensions.centre();
	let af_img_scale = (
		af_img_dimensions.width / img_dimensions.width,
		af_img_dimensions.height / img_dimensions.height,
	);

	let af_area_width_offset = 8;
	let af_area_height_offset = af_area_width_offset + num_af_points;
	let af_area_x_pos_offset = af_area_height_offset + num_af_points;
	let af_area_y_pos_offset = af_area_x_pos_offset + num_af_points;
	let af_points_active_offset = af_area_y_pos_offset + num_af_points;
	let af_points_selected_offset = af_points_active_offset + num_af_bitfields;

	let mut af_points = Vec::with_capacity(valid_af_points);

	for i in 0..valid_af_points {
		af_points.push(AFPoint {
			dimensions: af_info.get_dimensions_f64(
				af_area_width_offset + i,
				|| anyhow!("Missing AFAreaWidth[{i}]"),
				af_area_height_offset + i,
				|| anyhow!("Missing AFAreaHeight[{i}]"),
			)? * af_img_scale,
			position: (af_info.get_point_f64(
				af_area_x_pos_offset + i,
				|| anyhow!("Missing AFAreaXPositions[{i}]"),
				af_area_y_pos_offset + i,
				|| anyhow!("Missing AFAreaYPositions[{i}]"),
			)? + af_img_centre)
				* af_img_scale,
			selected: af_info.get_bit(af_points_selected_offset, i, || {
				anyhow!("Missing AFPointsSelected[{}]", i / 16)
			})?,
			active: af_info.get_bit(af_points_active_offset, i, || {
				anyhow!("Missing AFPointsInFocus[{}]", i / 16)
			})?,
		});
	}

	Ok(af_points)
}
