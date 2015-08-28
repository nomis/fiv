/*
 Copyright 2015  Simon Arlott

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU Affero General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU Affero General Public License for more details.

 You should have received a copy of the GNU Affero General Public License
 along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

#include "Image.hpp"

#include <fcntl.h>
#include <unistd.h>
#include <iostream>
#include <string>
#include <sys/mman.h>
#include <sys/stat.h>

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

	fd = open(filename.c_str(), O_RDONLY|O_CLOEXEC);
	if (fd < 0)
		goto err;

	if (fstat(fd, &st))
		goto err;

	data = mmap(nullptr, st.st_size, PROT_READ, MAP_SHARED, fd, 0);
	if (data == nullptr)
		goto err;

	length = st.st_size;
	close(fd);
	return true;

err:
	perror(filename.c_str());
	if (fd >= 0)
		close(fd);
	return false;
}

ostream& operator<<(ostream &stream, const Image &image) {
	return stream << "Image(filename=" << image.filename << ")";
}
