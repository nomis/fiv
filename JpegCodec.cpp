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

#include <exiv2/error.hpp>
#include <exiv2/exif.hpp>
#include <exiv2/image.hpp>
#include <exiv2/tags.hpp>
#include <exiv2/types.hpp>
#include <GL/freeglut_std.h>
#include <GL/glext.h>
#include <turbojpeg.h>
#include <algorithm>
#include <iostream>
#include <list>
#include <memory>
#include <string>

#include "Image.hpp"
#include "MemoryDataBuffer.hpp"
#include "TextureDataBuffer.hpp"

using namespace std;

const Exiv2::ExifKey Exif_Thumbnail_JPEGInterchangeFormat("Exif.Thumbnail.JPEGInterchangeFormat");

const string JpegCodec::MIME_TYPE = "image/jpeg";

JpegCodec::JpegCodec() : JpegCodec(nullptr) {
	atexit(&Exiv2::XmpParser::terminate);
}

JpegCodec::~JpegCodec() {

}

JpegCodec::JpegCodec(shared_ptr<const Image> image_) : Codec(image_) {
	width = 0;
	height = 0;
	headerInit = false;
	headerError = false;
	exiv2Error = false;
}

unique_ptr<Codec> JpegCodec::getInstance(shared_ptr<const Image> image_) const {
	return make_unique<JpegCodec>(image_);
}

int JpegCodec::getWidth() {
	initHeader();
	return width;
}

int JpegCodec::getHeight() {
	initHeader();
	return height;
}

unique_ptr<TextureDataBuffer> JpegCodec::getPrimary() {
	if (!initHeader())
		return unique_ptr<TextureDataBuffer>();

	tjhandle tj = tjInitDecompress();

	if (!tj)
		return unique_ptr<TextureDataBuffer>();

	const int format = TJPF_BGRA;
	const int pitch = width * tjPixelSize[format];
	const int size = pitch * height;
	unique_ptr<uint8_t[]> data = unique_ptr<uint8_t[]>(new uint8_t[size]);
	unique_ptr<TextureDataBuffer> buffer;

	if (!tjDecompress2(tj, const_cast<uint8_t*>(image->begin()), image->size(), data.get(), width, pitch, height, format, TJFLAG_BOTTOMUP|TJFLAG_NOREALLOC))
		buffer = make_unique<TextureDataBuffer>(move(data), size, GL_BGRA, GL_UNSIGNED_BYTE);

	tjDestroy(tj);
	return buffer;
}

shared_ptr<Image> JpegCodec::getThumbnail() {
	if (!initExiv2())
		return shared_ptr<Image>();

	auto dataTag = exif.findKey(Exif_Thumbnail_JPEGInterchangeFormat);

	if (dataTag == exif.end())
		return shared_ptr<Image>();

	unique_ptr<MemoryDataBuffer> buffer = make_unique<MemoryDataBuffer>(dataTag->dataArea());
	shared_ptr<Image> thumbnail = make_shared<Image>(image->name + " <Exif_Thumbnail>", move(buffer));

	if (!thumbnail->load())
		return shared_ptr<Image>();

	return thumbnail;
}

bool JpegCodec::initHeader() {
	if (headerInit)
		return true;

	if (headerError)
		return false;

	tjhandle tj = tjInitDecompress();
	if (tj) {
		int subsamp;

		if (!tjDecompressHeader2(tj, const_cast<uint8_t*>(image->begin()), image->size(), &width, &height, &subsamp)) {
			headerInit = true;
		} else {
			cerr << image->name << ": TurboJPEG: error reading header" << endl;
			headerError = true;
		}
		tjDestroy(tj);
	} else {
		headerError = true;
	}

	return headerInit;
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
