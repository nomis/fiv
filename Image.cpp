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

#include "Image.hpp"

#include <cairomm/refptr.h>
#include <cairomm/surface.h>
#include <stddef.h>
#include <algorithm>
#include <chrono>
#include <cstdint>
#include <iostream>
#include <memory>
#include <string>
#include <utility>

#include "Codec.hpp"
#include "Codecs.hpp"
#include "DataBuffer.hpp"
#include "Magic.hpp"

using namespace std;

Image::Image(const string &name_, unique_ptr<DataBuffer> buffer_) :
		name(name_), buffer(move(buffer_)), autoOrientation(true), orientation(Image::Rotate::ROTATE_NONE, false) {
	primaryFailed = false;
	thumbnailFailed = false;
}

Image::Image(const string &name_, unique_ptr<DataBuffer> buffer_, Orientation orientation_) :
		name(name_), buffer(move(buffer_)), autoOrientation(true), orientation(orientation_) {
	primaryFailed = false;
	thumbnailFailed = false;
}

ostream& operator<<(ostream &stream, const Image &image) {
	return stream << "Image(name=" << image.name << ",type=" << image.mimeType << ")";
}

bool Image::load() {
	if (!buffer->load())
		return false;

	if (!mimeType.length())
		mimeType = Magic::identify(buffer->begin(), buffer->size());

	if (!codec)
		codec = Codecs::create(shared_from_this(), mimeType);

	if (!codec) {
		cerr << name << ": Unsupported type " << mimeType << endl;
		return false;
	}

	return true;
}

const uint8_t *Image::begin() const {
	return buffer->begin();
}

const uint8_t *Image::end() const {
	return buffer->end();
}

size_t Image::size() const {
	return buffer->size();
}

int Image::width() {
	return codec->getWidth();
}

int Image::height() {
	return codec->getHeight();
}

bool Image::loadPrimary() {
	if (primary)
		return true;

	if (primaryFailed)
		return false;

	auto start = chrono::steady_clock::now();
	primary = codec->getPrimary();
	auto stop = chrono::steady_clock::now();
	cout << "load " << chrono::duration_cast<chrono::milliseconds>(stop - start).count() << "ms" << endl;
	if (primary)
		return true;

	primaryFailed = true;
	return false;
}

Cairo::RefPtr<Cairo::Surface> Image::getPrimary() {
	return primary;
}

Image::Orientation Image::getOrientation() {
	if (autoOrientation) {
		orientation = codec->getOrientation();
		autoOrientation = false;
	}

	return orientation;
}

void Image::setOrientation(Image::Orientation modify) {
	static const Image::Rotate ROTATE_MAP[4][4] = {
			                 /*                ROTATE_NONE                 ROTATE_90                   ROTATE_180                  ROTATE_270  */
			/* ROTATE_NONE */ { Image::Rotate::ROTATE_NONE, Image::Rotate::ROTATE_90,   Image::Rotate::ROTATE_180,  Image::Rotate::ROTATE_270  },
			/* ROTATE_90   */ { Image::Rotate::ROTATE_90,   Image::Rotate::ROTATE_180,  Image::Rotate::ROTATE_270,  Image::Rotate::ROTATE_NONE },
			/* ROTATE_180  */ { Image::Rotate::ROTATE_180,  Image::Rotate::ROTATE_270,  Image::Rotate::ROTATE_NONE, Image::Rotate::ROTATE_90   },
			/* ROTATE_270  */ { Image::Rotate::ROTATE_270,  Image::Rotate::ROTATE_NONE, Image::Rotate::ROTATE_90,   Image::Rotate::ROTATE_180  }
	};

	getOrientation();
	orientation.first = ROTATE_MAP[orientation.first][modify.first];
	orientation.second = orientation.second ^ modify.second;
}

bool Image::loadThumbnail() {
	if (thumbnail)
		return true;

	if (thumbnailFailed)
		return false;

	thumbnail = codec->getThumbnail();
	if (thumbnail)
		return true;

	thumbnailFailed = true;
	return false;
}

shared_ptr<Image> Image::getThumbnail() const {
	if (thumbnail)
		return thumbnail;

	return shared_ptr<Image>();
}

void Image::unloadThumbnail() {
	thumbnail = nullptr;
}
