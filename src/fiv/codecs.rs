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

use super::{image::ImageData, numeric::DimensionsU32, Orientation};
use anyhow::Error;
use enum_dispatch::enum_dispatch;
use std::path::Path;

#[enum_dispatch]
pub trait Codec {
	fn metadata(&self, filename: &Path) -> Result<CodecMetadata, Error>;
	fn primary(&self, filename: &Path, metadata: &CodecMetadata) -> Result<CodecPrimary, Error>;
}

#[derive(Debug)]
pub struct CodecMetadata {
	pub dimensions: DimensionsU32,
	pub orientation: Orientation,
}

#[derive(Debug)]
pub struct CodecPrimary {
	pub image_data: ImageData,
}

#[derive(Debug)]
#[enum_dispatch(Codec)]
pub enum Codecs {
	Generic,
}

#[derive(Debug, Default)]
pub struct Generic {}
