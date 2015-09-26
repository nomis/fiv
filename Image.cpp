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
#include <cmath>
#include <cstdint>
#include <iostream>
#include <memory>
#include <mutex>
#include <string>
#include <utility>

#include "Codec.hpp"
#include "Codecs.hpp"
#include "DataBuffer.hpp"
#include "Magic.hpp"

using namespace std;

Image::Image(const string &name_, unique_ptr<DataBuffer> buffer_) :
		name(name_), buffer(move(buffer_)), autoOrientation(true), orientation(Image::Rotate::ROTATE_NONE, false) {
	primaryUnload = false;
	primaryFailed = false;
	thumbnailUnload = false;
	thumbnailFailed = false;
}

Image::Image(const string &name_, unique_ptr<DataBuffer> buffer_, Orientation orientation_) :
		name(name_), buffer(move(buffer_)), autoOrientation(true), orientation(orientation_) {
	primaryUnload = false;
	primaryFailed = false;
	thumbnailUnload = false;
	thumbnailFailed = false;
}

ostream& operator<<(ostream &stream, const Image &image) {
	return stream << "Image(name=" << image.name << ",type=" << image.mimeType << ")";
}

string Image::getFilename() {
	return buffer->getFilename();
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

Image::Properties::Properties() {
	isoSpeed = 0;
	fAperture = NAN;
	mmFocalLength = NAN;
	sExposureTime = NAN;
	evExposureBias = NAN;
	flash = false;
	evFlashBias = NAN;
	rating = 0;
}

const Image::Properties Image::getProperties() {
	return codec->getProperties();
}

bool Image::loadPrimary() {
	unique_lock<mutex> lckPrimary(mtxPrimary);

	if (primary)
		return true;

	if (primaryFailed)
		return false;

	unique_lock<mutex> lckPrimaryLoad(mtxPrimaryLoad, defer_lock);
	if (lckPrimaryLoad.try_lock()) {
		Cairo::RefPtr<Cairo::ImageSurface> loadedPrimary;

		primaryUnload = false;

		try {
			lckPrimary.unlock();

			//auto start = chrono::steady_clock::now();
			loadedPrimary = codec->getPrimary();
			//auto stop = chrono::steady_clock::now();
			//cout << "load " << name << " in " << chrono::duration_cast<chrono::milliseconds>(stop - start).count() << "ms" << endl;

			if (!loadedPrimary)
				primaryFailed = true;
		} catch (...) {
			lckPrimary.lock();
			lckPrimaryLoad.unlock();
			primaryFailed = true;
			throw;
		}

		lckPrimary.lock();
		lckPrimaryLoad.unlock();
		if (!primaryUnload)
			primary = loadedPrimary;
		primaryUnload = false;
	}

	if (primary)
		return true;

	return false;
}

bool Image::isPrimaryFailed() {
	lock_guard<mutex> lckPrimary(mtxPrimary);

	if (primaryFailed)
		return true;

	return false;
}

void Image::unloadPrimary() {
	lock_guard<mutex> lckPrimary(mtxPrimary);

	if (!primary)
		return;

	primary = Cairo::RefPtr<Cairo::ImageSurface>();
	primaryUnload = true;
}

Cairo::RefPtr<Cairo::ImageSurface> Image::getPrimary() {
	lock_guard<mutex> lckPrimary(mtxPrimary);
	return primary;
}

bool Image::loadThumbnail() {
	unique_lock<mutex> lckThumbnail(mtxThumbnail);

	if (thumbnail)
		return true;

	if (thumbnailFailed)
		return false;

	unique_lock<mutex> lckThumbnailLoad(mtxThumbnailLoad, defer_lock);
	if (lckThumbnailLoad.try_lock()) {
		Cairo::RefPtr<Cairo::ImageSurface> loadedThumbnail;

		thumbnailUnload = false;

		try {
			lckThumbnail.unlock();

			//auto start = chrono::steady_clock::now();
			loadedThumbnail = codec->getThumbnail();
			//auto stop = chrono::steady_clock::now();
			//cout << "load " << name << " thumbnail in " << chrono::duration_cast<chrono::milliseconds>(stop - start).count() << "ms" << endl;

			if (!loadedThumbnail)
				thumbnailFailed = true;
		} catch (...) {
			lckThumbnail.lock();
			lckThumbnailLoad.unlock();
			thumbnailFailed = true;
			throw;
		}

		lckThumbnail.lock();
		lckThumbnailLoad.unlock();
		if (!thumbnailUnload)
			thumbnail = loadedThumbnail;
		thumbnailUnload = false;
	}

	if (thumbnail)
		return true;

	return false;
}

bool Image::isThumbnailFailed() {
	lock_guard<mutex> lckThumbnail(mtxThumbnail);

	if (thumbnailFailed)
		return true;

	return false;
}

void Image::unloadThumbnail() {
	lock_guard<mutex> lckThumbnail(mtxThumbnail);

	if (!thumbnail)
		return;

	thumbnail = Cairo::RefPtr<Cairo::ImageSurface>();
	thumbnailUnload = true;
}

Cairo::RefPtr<Cairo::ImageSurface> Image::getThumbnail() {
	lock_guard<mutex> lckThumbnail(mtxThumbnail);
	return thumbnail;
}
