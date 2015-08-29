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

#ifndef IMAGE_HPP_
#define IMAGE_HPP_

#include <stddef.h>
#include <cstdint>
#include <iostream>
#include <map>
#include <memory>
#include <string>

#include "Codec.hpp"

class Image : public std::enable_shared_from_this<Image> {
	friend class Codec;

public:
	Image(std::string filename);
	~Image();
	bool openFile(const std::map<std::string,std::shared_ptr<Codec>> codecs);
	const uint8_t *begin() const;
	const uint8_t *end() const;
	size_t size() const;
	friend std::ostream& operator<<(std::ostream &stream, const Image &image);
	void getThumbnail();

	const std::string filename;

private:
	bool identifyFile(const std::map<std::string,std::shared_ptr<Codec>> codecs);

	std::unique_ptr<Codec> codec;
	uint8_t *data;
	size_t length;
};

#endif /* IMAGE_HPP_ */
