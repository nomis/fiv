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

// Using rexiv2::Metadata.get_tag_raw() calls gexiv2_metadata_get_tag_raw()
// which calls Exiv2::ExifData::iterator.copy(..., Exiv2::invalidByteOrder)
// which calls Exiv2::us2Data() on u16 values without checking that the
// byte order is valid so the data is always big-endian instead of the original
// raw endian.
//
// Test the library at runtime to find out what byte order it's using otherwise
// raw tag parsing will break if Exiv2 and gexiv2 are fixed.

use anyhow::{Error, anyhow};
use std::sync::LazyLock;

#[derive(Debug, Copy, Clone)]
pub enum ByteOrder {
	BE,
	LE,
}

#[derive(Debug, Copy, Clone)]
enum NumberFormat {
	U16,
	U32,
}

#[derive(Debug, Copy, Clone)]
enum SizeFormat {
	One,
	U8,
	U16,
}

#[derive(Debug, Copy, Clone)]
enum TagType {
	AsciiString = 2,
	UnsignedShort = 3,
	UnsignedLong = 4,
	Undefined = 7,
}

#[derive(Debug)]
struct Offset {
	position: usize,
	bo: ByteOrder,
	format: NumberFormat,
}

#[derive(Debug)]
struct Count {
	position: usize,
	bo: ByteOrder,
	format: NumberFormat,
	value: usize,
}

#[derive(Debug)]
struct Size {
	position: usize,
	bo: ByteOrder,
	input: SizeFormat,
	output: NumberFormat,
}

#[derive(Debug)]
struct DataBuilder {
	position: usize,
}

#[derive(Debug)]
struct Data {
	position: usize,
	length: usize,
}

impl Count {
	fn add(&mut self) {
		self.value += 1;
	}
}

trait TestVecExt {
	fn begin_offset(&mut self, bo: ByteOrder, format: NumberFormat) -> Offset;
	fn end_offset(&mut self, offset: Offset, relative_to: &DataBuilder, data: Data) -> Data;
	fn begin_ifd_count(&mut self, bo: ByteOrder) -> Count;
	fn end_ifd_count(&mut self, count: Count) -> Offset;
	fn begin_size(&mut self, bo: ByteOrder, input: SizeFormat, output: NumberFormat) -> Size;
	fn end_size(&mut self, size: Size, data: Data) -> Data;
	fn begin_data(&self) -> DataBuilder;
	fn end_data(&self, data_builder: DataBuilder) -> Data;

	fn push_u16(&mut self, bo: ByteOrder, value: u16);
	fn push_u32(&mut self, bo: ByteOrder, value: u32);
	fn push_tag_type(&mut self, bo: ByteOrder, value: TagType);
	fn push_zero(&mut self, bo: ByteOrder, format: NumberFormat) -> usize;
	fn splice_value(&mut self, position: usize, bo: ByteOrder, format: NumberFormat, value: usize);

	fn push_tiff_header(&mut self, bo: ByteOrder);
	fn push_ifd_entry(
		&mut self,
		count: &mut Count,
		tag: u16,
		tag_type: TagType,
		size_format: SizeFormat,
	) -> (Size, Offset);
	fn end_ifd_entry(&mut self, size: Size, offset: Offset, relative_to: &DataBuilder, data: Data);
	fn push_ifd_value(&mut self, count: &mut Count, tag: u16, tag_type: TagType, value: Vec<u8>);
	fn push_ascii(&mut self, value: &str, null_terminated: bool) -> Data;
}

impl TestVecExt for Vec<u8> {
	fn begin_offset(&mut self, bo: ByteOrder, format: NumberFormat) -> Offset {
		Offset {
			position: self.push_zero(bo, format),
			bo,
			format,
		}
	}

	fn end_offset(&mut self, offset: Offset, relative_to: &DataBuilder, data: Data) -> Data {
		assert!(data.position >= relative_to.position);
		let position = data.position - relative_to.position;
		self.splice_value(offset.position, offset.bo, offset.format, position);
		data
	}

	fn begin_ifd_count(&mut self, bo: ByteOrder) -> Count {
		let format = NumberFormat::U16;

		Count {
			position: self.push_zero(bo, format),
			bo,
			format,
			value: 0,
		}
	}

	fn end_ifd_count(&mut self, count: Count) -> Offset {
		self.splice_value(count.position, count.bo, count.format, count.value);
		self.begin_offset(count.bo, NumberFormat::U32)
	}

	fn begin_size(&mut self, bo: ByteOrder, input: SizeFormat, output: NumberFormat) -> Size {
		Size {
			position: self.push_zero(bo, output),
			bo,
			input,
			output,
		}
	}

	fn end_size(&mut self, size: Size, data: Data) -> Data {
		let length = match size.input {
			SizeFormat::One => 1,
			SizeFormat::U8 => data.length,
			SizeFormat::U16 => {
				assert!(data.length % 2 == 0);
				data.length / 2
			}
		};
		match size.input {
			SizeFormat::One => (),
			SizeFormat::U8 | SizeFormat::U16 => {
				// IFD values shorter than 5 bytes are inlined
				assert!(length > 4);
			}
		}
		self.splice_value(size.position, size.bo, size.output, length);
		data
	}

	fn begin_data(&self) -> DataBuilder {
		DataBuilder {
			position: self.len(),
		}
	}

	fn end_data(&self, data_builder: DataBuilder) -> Data {
		Data {
			position: data_builder.position,
			length: self.len() - data_builder.position,
		}
	}

	fn push_u16(&mut self, bo: ByteOrder, value: u16) {
		match bo {
			ByteOrder::BE => self.extend(value.to_be_bytes()),
			ByteOrder::LE => self.extend(value.to_le_bytes()),
		}
	}

	fn push_u32(&mut self, bo: ByteOrder, value: u32) {
		match bo {
			ByteOrder::BE => self.extend(value.to_be_bytes()),
			ByteOrder::LE => self.extend(value.to_le_bytes()),
		}
	}

	fn push_tag_type(&mut self, bo: ByteOrder, value: TagType) {
		self.push_u16(bo, u16::try_from(value as usize).unwrap());
	}

	fn push_zero(&mut self, bo: ByteOrder, format: NumberFormat) -> usize {
		let position = self.len();

		match format {
			NumberFormat::U16 => self.push_u16(bo, 0),
			NumberFormat::U32 => self.push_u32(bo, 0),
		};

		position
	}

	fn splice_value(&mut self, position: usize, bo: ByteOrder, format: NumberFormat, value: usize) {
		match format {
			NumberFormat::U16 => {
				let value = u16::try_from(value).unwrap();
				self.splice(
					position..position + size_of_val(&value),
					match bo {
						ByteOrder::BE => value.to_be_bytes(),
						ByteOrder::LE => value.to_le_bytes(),
					},
				);
			}
			NumberFormat::U32 => {
				let value = u32::try_from(value).unwrap();
				self.splice(
					position..position + size_of_val(&value),
					match bo {
						ByteOrder::BE => value.to_be_bytes(),
						ByteOrder::LE => value.to_le_bytes(),
					},
				);
			}
		};
	}

	fn push_tiff_header(&mut self, bo: ByteOrder) {
		self.push_ascii(
			match bo {
				ByteOrder::BE => "MM",
				ByteOrder::LE => "II",
			},
			false,
		); // TIFF Header
		self.push_u16(bo, 0x2A); // TIFF Version
	}

	fn push_ifd_entry(
		&mut self,
		count: &mut Count,
		tag: u16,
		tag_type: TagType,
		format: SizeFormat,
	) -> (Size, Offset) {
		self.push_u16(count.bo, tag);
		self.push_tag_type(count.bo, tag_type);
		let size = self.begin_size(count.bo, format, NumberFormat::U32);
		let offset = self.begin_offset(count.bo, NumberFormat::U32);
		count.add();
		(size, offset)
	}

	fn end_ifd_entry(&mut self, size: Size, offset: Offset, relative_to: &DataBuilder, data: Data) {
		let data = self.end_size(size, data);
		self.end_offset(offset, relative_to, data);
	}

	fn push_ifd_value(&mut self, count: &mut Count, tag: u16, tag_type: TagType, value: Vec<u8>) {
		assert!(value.len() <= 4);
		self.push_u16(count.bo, tag);
		self.push_tag_type(count.bo, tag_type);
		self.push_u32(count.bo, u32::try_from(value.len()).unwrap());
		self.extend(&value);
		for _ in value.len()..4 {
			self.push(0);
		}
		count.add();
	}

	fn push_ascii(&mut self, value: &str, null_terminated: bool) -> Data {
		assert!(value.is_ascii());
		let data = self.begin_data();
		self.extend(value.as_bytes());
		if null_terminated {
			self.push(0);
		}
		self.end_data(data)
	}
}

fn test_image(bo: ByteOrder) -> Vec<u8> {
	let mut buffer = Vec::new();

	buffer.extend([0xFF, 0xD8]); // Start of image

	buffer.extend([0xFF, 0xE1]); // Application Marker
	let exif_data = buffer.begin_data();
	let exif_size = buffer.begin_size(ByteOrder::BE, SizeFormat::U8, NumberFormat::U16); // Data Size (Big Endian)
	buffer.extend([0x45, 0x78, 0x69, 0x66, 0x00, 0x00]); // Exif Header
	let tiff_data = buffer.begin_data();
	buffer.push_tiff_header(bo);

	let ifd0_offset = buffer.begin_offset(bo, NumberFormat::U32);
	// IFD0 (main image)
	// -----------------
	let ifd0_data = buffer.begin_data();
	let mut ifd0_count = buffer.begin_ifd_count(bo); // Number of directory entries

	// Make
	let (make_size, make_offset) = buffer.push_ifd_entry(
		&mut ifd0_count,
		0x010F,
		TagType::AsciiString,
		SizeFormat::U8,
	);

	// Model
	let (model_size, model_offset) = buffer.push_ifd_entry(
		&mut ifd0_count,
		0x0110,
		TagType::AsciiString,
		SizeFormat::U8,
	);

	// Exif SubIFD
	let (exif_sub_ifd_size, exif_sub_ifd_offset) = buffer.push_ifd_entry(
		&mut ifd0_count,
		0x8769,
		TagType::UnsignedLong,
		SizeFormat::One,
	);

	let ifd1_offset = buffer.end_ifd_count(ifd0_count);

	let make_data = buffer.push_ascii("Canon", true);
	buffer.end_ifd_entry(make_size, make_offset, &tiff_data, make_data);

	let model_data = buffer.push_ascii("Canon EOS", true);
	buffer.end_ifd_entry(model_size, model_offset, &tiff_data, model_data);

	let ifd0_data = buffer.end_data(ifd0_data);
	buffer.end_offset(ifd0_offset, &tiff_data, ifd0_data);

	// IFD1 (thumbnail image)
	// ----------------------
	let ifd1_data = buffer.begin_data();
	let ifd1_count = buffer.begin_ifd_count(bo); // Number of directory entries
	buffer.end_ifd_count(ifd1_count);

	let ifd1_data = buffer.end_data(ifd1_data);
	buffer.end_offset(ifd1_offset, &tiff_data, ifd1_data);

	// Exif SubIFD
	// -----------
	let exif_sub_ifd_data = buffer.begin_data();
	let mut exif_sub_ifd_count = buffer.begin_ifd_count(bo); // Number of directory entries

	// Exif Version
	buffer.push_ifd_value(
		&mut exif_sub_ifd_count,
		0x9000,
		TagType::Undefined,
		"0230".as_bytes().to_vec(),
	);

	// MakerNote
	let (makernote_size, makernote_offset) = buffer.push_ifd_entry(
		&mut exif_sub_ifd_count,
		0x927C,
		TagType::Undefined,
		SizeFormat::U8,
	);

	buffer.end_ifd_count(exif_sub_ifd_count);
	let exif_sub_ifd_data = buffer.end_data(exif_sub_ifd_data);
	buffer.end_ifd_entry(
		exif_sub_ifd_size,
		exif_sub_ifd_offset,
		&tiff_data,
		exif_sub_ifd_data,
	);

	// MakerNote SubIFD
	// ----------------
	let makernote_data = buffer.begin_data();
	let mut makernote_count = buffer.begin_ifd_count(bo); // Number of directory entries

	// CanonAFInfo2
	let (afinfo_size, afinfo_offset) = buffer.push_ifd_entry(
		&mut makernote_count,
		0x0026,
		TagType::UnsignedShort,
		SizeFormat::U16,
	);

	buffer.end_ifd_count(makernote_count);

	// CanonAFInfo2
	let afinfo_data = buffer.begin_data();
	let afinfo_data_size = buffer.begin_size(bo, SizeFormat::U8, NumberFormat::U16); // AFInfoSize
	buffer.push_u16(bo, 0x4D49); // AFAreaMode
	buffer.push_u16(bo, 1); // NumAFPoints
	buffer.push_u16(bo, 0); // ValidAFPoints
	buffer.push_u16(bo, 1); // CanonImageWidth
	buffer.push_u16(bo, 1); // CanonImageHeight
	buffer.push_u16(bo, 1); // AFImageWidth
	buffer.push_u16(bo, 1); // AFImageHeight
	buffer.push_u16(bo, 0); // AFAreaWidths
	buffer.push_u16(bo, 0); // AFAreaHeights
	buffer.push_u16(bo, 0); // AFAreaXPositions
	buffer.push_u16(bo, 0); // AFAreaYPositions
	buffer.push_u16(bo, 0); // AFPointsInFocus
	buffer.push_u16(bo, 0); // AFPointsSelected
	let afinfo_data = buffer.end_data(afinfo_data);
	let afinfo_data = buffer.end_size(afinfo_data_size, afinfo_data);
	buffer.end_ifd_entry(afinfo_size, afinfo_offset, &tiff_data, afinfo_data);

	// Footer
	buffer.push_tiff_header(bo);
	let makernote_offset_copy = buffer.begin_offset(bo, NumberFormat::U32);
	let makernote_data = buffer.end_data(makernote_data);
	let makernote_data = buffer.end_offset(makernote_offset_copy, &tiff_data, makernote_data);
	buffer.end_ifd_entry(makernote_size, makernote_offset, &tiff_data, makernote_data);

	let exif_data = buffer.end_data(exif_data);
	buffer.end_size(exif_size, exif_data);

	buffer.extend([0xFF, 0xD9]); // End of image

	buffer
}

fn test_image_byte_order(bo: ByteOrder) -> Result<ByteOrder, Error> {
	let test_image = test_image(bo);
	let exiv = rexiv2::Metadata::new_from_buffer(&test_image)?;
	let af_info = exiv.get_tag_raw("Exif.Canon.AFInfo")?;

	match af_info.get(2) {
		Some(b'I') => Ok(ByteOrder::LE),
		Some(b'M') => Ok(ByteOrder::BE),
		_ => Err(anyhow!("Invalid AFInfo in {bo:?} test image: {af_info:?}")),
	}
}

static BE_TEST_IMAGE_BYTE_ORDER: LazyLock<Option<ByteOrder>> =
	LazyLock::new(|| test_image_byte_order(ByteOrder::BE).ok());
static LE_TEST_IMAGE_BYTE_ORDER: LazyLock<Option<ByteOrder>> =
	LazyLock::new(|| test_image_byte_order(ByteOrder::LE).ok());

pub fn byte_order_of(exiv: &rexiv2::Metadata) -> Result<ByteOrder, Error> {
	match exiv
		// Exiv2 doesn't provide access to the Exif byte order, this is a
		// synthesised tag that happens to be a copy of it (but is also present
		// as its own value in the Canon MakerNote footer, which Exiv2 ignores)
		.get_tag_string("Exif.MakerNote.ByteOrder")
		.map_err(|_| anyhow!("Missing image ByteOrder"))?
		.as_str()
	{
		"II" => LE_TEST_IMAGE_BYTE_ORDER.ok_or_else(|| anyhow!("Little-endian test image failed")),
		"MM" => BE_TEST_IMAGE_BYTE_ORDER.ok_or_else(|| anyhow!("Big-endian test image failed")),
		value => Err(anyhow!("Invalid image ByteOrder: {value:?}")),
	}
}
