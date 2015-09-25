/*
 Copyright 2015  Simon Arlott

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

#include "JpegCodec.hpp"

#include <cairomm/enums.h>
#include <cairomm/refptr.h>
#include <cairomm/surface.h>
#include <exiv2/error.hpp>
#include <exiv2/exif.hpp>
#include <exiv2/image.hpp>
#include <exiv2/tags.hpp>
#include <exiv2/types.hpp>
#include <exiv2/xmp.hpp>
#include <turbojpeg.h>
#include <algorithm>
#include <cstdint>
#include <cstdlib>
#include <iostream>
#include <list>
#include <memory>
#include <string>
#include <utility>

#include "Image.hpp"
#include "MemoryDataBuffer.hpp"

using namespace std;

const Exiv2::ExifKey Exif_Thumbnail_JPEGInterchangeFormat("Exif.Thumbnail.JPEGInterchangeFormat");
const Exiv2::ExifKey Exif_Image_Orientation("Exif.Image.Orientation");

const string JpegCodec::MIME_TYPE = "image/jpeg";

JpegCodec::JpegCodec() {
	atexit(&Exiv2::XmpParser::terminate);

	width = 0;
	height = 0;
	orientation = Image::Orientation(Image::Rotate::ROTATE_NONE, false);
}

JpegCodec::~JpegCodec() {

}

JpegCodec::JpegCodec(shared_ptr<const Image> image_) : Codec(image_) {
	width = 0;
	height = 0;
	orientation = Image::Orientation(Image::Rotate::ROTATE_NONE, false);
	initHeader();
	initExiv2();
}

unique_ptr<Codec> JpegCodec::getInstance(shared_ptr<const Image> image_) const {
	return make_unique<JpegCodec>(image_);
}

int JpegCodec::getWidth() {
	return width;
}

int JpegCodec::getHeight() {
	return height;
}

Image::Orientation JpegCodec::getOrientation() {
	return orientation;
}

Cairo::RefPtr<Cairo::Surface> JpegCodec::getPrimary() {
	Cairo::RefPtr<Cairo::ImageSurface> surface;
	tjhandle tj;

	if (width <= 0 || height <= 0)
		goto err;

	tj = tjInitDecompress();
	if (!tj)
		goto err;

	surface = Cairo::ImageSurface::create(Cairo::Format::FORMAT_RGB24, width, height);
	if (!surface)
		goto err_tj;

	surface->flush();
	if (!tjDecompress2(tj, const_cast<uint8_t*>(image->begin()), image->size(), surface->get_data(),
			width, surface->get_stride(), height, TJPF_BGRX, TJFLAG_NOREALLOC)) {
		surface->mark_dirty();
	} else {
		surface = Cairo::RefPtr<Cairo::ImageSurface>();
	}

err_tj:
	tjDestroy(tj);
err:
	return surface;
}

shared_ptr<Image> JpegCodec::getThumbnail() {
	Exiv2::ExifData exif = getExifData();

	auto dataTag = exif.findKey(Exif_Thumbnail_JPEGInterchangeFormat);
	if (dataTag == exif.end())
		return shared_ptr<Image>();

	unique_ptr<MemoryDataBuffer> buffer = make_unique<MemoryDataBuffer>(dataTag->dataArea());
	shared_ptr<Image> thumbnail = make_shared<Image>(image->name + " <Exif_Thumbnail>", move(buffer), orientation);

	if (!thumbnail->load())
		return shared_ptr<Image>();

	return thumbnail;
}

void JpegCodec::initHeader() {
	tjhandle tj = tjInitDecompress();
	if (tj) {
		int subsamp;

		if (!tjDecompressHeader2(tj, const_cast<uint8_t*>(image->begin()), image->size(), &width, &height, &subsamp)) {
		} else {
			cerr << image->name << ": TurboJPEG: error reading header" << endl;
		}
		tjDestroy(tj);
	}
}

Exiv2::ExifData JpegCodec::getExifData() {
	try {
		unique_ptr<Exiv2::Image> tmp = Exiv2::ImageFactory::open(image->begin(), image->size());
		if (tmp && tmp->good()) {
			tmp->readMetadata();
			return tmp->exifData();
		}
	} catch (const Exiv2::Error& e) {
		cerr << image->name << ": Exiv2: " << e.what() << endl;
	}
	return Exiv2::ExifData();
}

void JpegCodec::initExiv2() {
	Exiv2::ExifData exif = getExifData();

	auto orientationTag = exif.findKey(Exif_Image_Orientation);
	if (orientationTag != exif.end() && orientationTag->toLong() >= 1 && orientationTag->toLong() <= 8) {
		static const Image::Orientation ORIENTATION_MAP[8] = {
				/* Horizontal (normal) */
				Image::Orientation(Image::Rotate::ROTATE_NONE, false),

				/* Mirror horizontal */
				Image::Orientation(Image::Rotate::ROTATE_NONE, true),

				/* Rotate 180 */
				Image::Orientation(Image::Rotate::ROTATE_180, false),

				/* Mirror vertical */
				Image::Orientation(Image::Rotate::ROTATE_180, true),

				/* Mirror horizontal and rotate 270 CW */
				Image::Orientation(Image::Rotate::ROTATE_270, true),

				/* Rotate 90 CW */
				Image::Orientation(Image::Rotate::ROTATE_90, false),

				/* Mirror horizontal and rotate 90 CW */
				Image::Orientation(Image::Rotate::ROTATE_90, true),

				/* Rotate 270 CW */
				Image::Orientation(Image::Rotate::ROTATE_270, false)
		};

		orientation = ORIENTATION_MAP[orientationTag->toLong() - 1];
	}
}
