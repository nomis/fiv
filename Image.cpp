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

Image::Image(string filename_) :
		filename(filename_) {
	data = nullptr;
	length = 0;
}

Image::~Image() {
	if (data != nullptr) {
		munmap(data, length);

		data = nullptr;
		length = 0;
	}
}

bool Image::openFile() {
	struct stat st;
	int fd;

	if (data != nullptr)
		return true;

	fd = open(filename.c_str(), O_RDONLY|O_NONBLOCK|O_CLOEXEC);
	if (fd < 0)
		goto err;

	if (fstat(fd, &st))
		goto err;

	data = static_cast<uint8_t*>(mmap(nullptr, st.st_size, PROT_READ, MAP_SHARED, fd, 0));
	if (data == nullptr)
		goto err;

	length = st.st_size;
	close(fd);

	mimeType = Magic::identify(data, length);
	codec = Fiv::getCodec(shared_from_this(), mimeType);
	return (bool)codec;

err:
	perror(filename.c_str());
	if (fd >= 0)
		close(fd);
	return false;
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
	return stream << "Image(filename=" << image.filename << ")";
}

void Image::getThumbnail() {
	codec->getThumbnail();
}
