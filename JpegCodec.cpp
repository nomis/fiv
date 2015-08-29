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
#include <cstdint>
#include <iostream>
#include <memory>
#include <string>

#include "Image.hpp"

using namespace std;

const string JpegCodec::MIME_TYPE = "image/jpeg";

JpegCodec::JpegCodec() {
	image = nullptr;
}

JpegCodec::~JpegCodec() {

}

JpegCodec::JpegCodec(shared_ptr<const Image> image_) : Codec(image_) {

}

unique_ptr<Codec> JpegCodec::getInstance(shared_ptr<const Image> image_) const {
	return make_unique<JpegCodec>(image_);
}

void JpegCodec::getThumbnail() {
	Exif(image).getThumbnail();
}

JpegCodec::Exif::Exif(shared_ptr<const Image> image_) : JpegCodec::Exif::Exif(const_cast<uint8_t*>(image_->begin()), image_->size()) {

}

JpegCodec::Exif::Exif(uint8_t *data, size_t size) {
	exifLoader = exif_loader_new();
	exifData = nullptr;

	if (exifLoader == nullptr)
		return;

	if (exif_loader_write(exifLoader, data, size))
		return;

	exifData = exif_loader_get_data(exifLoader);
}

JpegCodec::Exif::~Exif() {
	if (exifData != nullptr) {
		exif_data_unref(exifData);
		exifData = nullptr;
	}

	if (exifLoader != nullptr) {
		exif_loader_unref(exifLoader);
		exifLoader = nullptr;
	}
}

void JpegCodec::Exif::getThumbnail() {
	if (exifData == nullptr)
		return;

	cout << &exifData->data << "," << exifData->size << endl;
}
