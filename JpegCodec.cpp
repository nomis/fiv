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
const Exiv2::ExifKey JpegCodec::Exif_Thumbnail_JPEGInterchangeFormatLength("Exif.Thumbnail.JPEGInterchangeFormatLength");

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

void JpegCodec::getThumbnail() {
	if (!initExiv2())
		return;

	Exiv2::ExifData exif = exiv2->exifData();

	auto data = exif.findKey(Exif_Thumbnail_JPEGInterchangeFormat);
	auto length = exif.findKey(Exif_Thumbnail_JPEGInterchangeFormatLength);

	if (data != exif.end() && length != exif.end())
		cout << &data->value() << "," << length->value() << endl;
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
			exiv2 = move(tmp);
		}
	} catch (const Exiv2::Error& e) {
		cerr << image->filename << ": Exiv2: " << e.what() << endl;
		exiv2Error = true;
	}

	return (bool)exiv2;
}
