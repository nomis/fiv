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

#include <fcntl.h>
#include <stddef.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>
#include <cstdint>
#include <cstdio>
#include <iostream>
#include <memory>
#include <string>

#include "Codec.hpp"
#include "DataBuffer.hpp"
#include "Fiv.hpp"
#include "Magic.hpp"

using namespace std;

Image::Image(string name_, unique_ptr<DataBuffer> buffer_) :
		name(name_), buffer(move(buffer_)) {

}

bool Image::load() {
	if (!buffer->load())
		return false;

	if (!mimeType.length())
		mimeType = Magic::identify(buffer->begin(), buffer->size());

	if (!codec)
		codec = Fiv::getCodec(shared_from_this(), mimeType);

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

ostream& operator<<(ostream &stream, const Image &image) {
	return stream << "Image(name=" << image.name << ",type=" << image.mimeType << ")";
}

shared_ptr<Image> Image::getThumbnail() {
	if (thumbnail)
		return thumbnail;

	return thumbnail = codec->getThumbnail();
}
