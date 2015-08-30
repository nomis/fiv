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
#include <map>
#include <memory>
#include <stdexcept>
#include <string>

#include "Codec.hpp"
#include "Fiv.hpp"
#include "Magic.hpp"

using namespace std;

Image::Image(string filename) :
		file(true), name(filename) {
	fileData = nullptr;
	data = nullptr;
	length = 0;
}

Image::Image(string name_, unique_ptr<const uint8_t[]> data_, size_t length_) :
		file(false), name(name_) {
	fileData = nullptr;
	memoryData = move(data_);
	data = memoryData.get();
	length = length_;
}

Image::~Image() {
	if (fileData != nullptr) {
		munmap(fileData, length);

		fileData = nullptr;
		length = 0;
	}
}

bool Image::openFile() {
	struct stat st;
	int fd = -1;

	if (!file)
		return false;

	if (data == nullptr) {
		fd = open(name.c_str(), O_RDONLY|O_NONBLOCK|O_CLOEXEC);
		if (fd < 0)
			goto perr;

		if (fstat(fd, &st))
			goto perr;

		fileData = mmap(nullptr, st.st_size, PROT_READ, MAP_SHARED, fd, 0);
		if (fileData == nullptr)
			goto perr;

		data = static_cast<uint8_t*>(fileData);
		length = st.st_size;
		close(fd);
	}

	if (!mimeType.length())
		mimeType = Magic::identify(data, length);

	if (!codec)
		codec = Fiv::getCodec(shared_from_this(), mimeType);

	if (!codec) {
		cerr << name << ": Unsupported type " << mimeType << endl;
		goto err;
	}
	return true;

perr:
	perror(name.c_str());
err:
	if (fd >= 0)
		close(fd);
	return false;
}

bool Image::openMemory() {
	if (file)
		return false;

	if (!mimeType.length())
		mimeType = Magic::identify(data, length);

	if (!codec)
		codec = Fiv::getCodec(shared_from_this(), mimeType);

	if (!codec) {
		cerr << name << ": Unsupported type " << mimeType << endl;
		return false;;
	}
	return true;
}

const uint8_t *Image::begin() const {
	return data;
}

const uint8_t *Image::end() const {
	if (data == nullptr)
		return nullptr;

	return data + length;
}

size_t Image::size() const {
	return length;
}

ostream& operator<<(ostream &stream, const Image &image) {
	return stream << "Image(name=" << image.name << ",type=" << image.mimeType << ")";
}

shared_ptr<Image> Image::getThumbnail() {
	if (thumbnail)
		return thumbnail;

	return thumbnail = codec->getThumbnail();
}
