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

#include "Image.h"

#include <unistd.h>
#include <iostream>
#include <string>

using namespace std;

Image::Image(string filename_) :
		filename(filename_) {
	fd = -1;
}

Image::~Image() {
	if (fd >= 0) {
		close(fd);
		fd = -1;
	}
}

ostream& operator<<(ostream &stream, const Image &image) {
	return stream << "Image(filename=" << image.filename << ")";
}
