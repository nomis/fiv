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

#ifndef IMAGE_HPP_
#define IMAGE_HPP_

#include <iostream>
#include <memory>
#include <string>

class Image : public std::enable_shared_from_this<Image> {
public:
	Image(std::string filename);
	~Image();
	bool openFile();
	friend std::ostream& operator<<(std::ostream &stream, const Image &image);

	const std::string filename;

private:
	void *data;
	size_t length;
};

#endif /* IMAGE_HPP_ */
