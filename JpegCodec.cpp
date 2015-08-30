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

#include <libexif/exif-data.h>
#include <libexif/exif-loader.h>
#include <stddef.h>
#include <cstdint>
#include <iostream>
#include <memory>
#include <string>
#include <exiv2/image.hpp>

#include "Image.hpp"

using namespace std;

const string JpegCodec::MIME_TYPE = "image/jpeg";
const Exiv2::ExifKey JpegCodec::Exif_Thumbnail_JPEGInterchangeFormat("Exif.Thumbnail.JPEGInterchangeFormat");

JpegCodec::JpegCodec() : Codec() {
	image = nullptr;
	exiv2Error = false;
}

JpegCodec::~JpegCodec() {

}

JpegCodec::JpegCodec(shared_ptr<const Image> image_) : Codec(image_) {
	exiv2Error = false;
}

unique_ptr<Codec> JpegCodec::getInstance(shared_ptr<const Image> image_) const {
	return make_unique<JpegCodec>(image_);
}

shared_ptr<Image> JpegCodec::getThumbnail() {
	if (!initExiv2())
		return shared_ptr<Image>();

	auto dataTag = exif.findKey(Exif_Thumbnail_JPEGInterchangeFormat);

	if (dataTag == exif.end())
		return shared_ptr<Image>();

	pair<Exiv2::byte*,long> dataArea = dataTag->dataArea().release();
	unique_ptr<const uint8_t[]> data = unique_ptr<const uint8_t[]>(dataArea.first);
	size_t length = dataArea.second;
	shared_ptr<Image> thumbnail = make_shared<Image>(image->name + " <Exif_Thumbnail>", move(data), length);

	if (thumbnail->openMemory())
		return thumbnail;

	return shared_ptr<Image>();
}

bool JpegCodec::initExiv2() {
	if (exiv2Error)
		return false;

	if (exiv2)
		return true;

	try {
		unique_ptr<Exiv2::Image> tmp = Exiv2::ImageFactory::open(image->begin(), image->size());
		if (tmp && tmp->good()) {
			tmp->readMetadata();
			exif = tmp->exifData();
			exiv2 = move(tmp);
		}
	} catch (const Exiv2::Error& e) {
		cerr << image->name << ": Exiv2: " << e.what() << endl;
		exiv2Error = true;
	}

	return (bool)exiv2;
}
