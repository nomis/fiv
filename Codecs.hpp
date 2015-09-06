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

#ifndef fiv__CODECS_HPP_
#define fiv__CODECS_HPP_

#include <map>
#include <memory>
#include <string>

class Image;

class Codec;

class Codecs {
public:
	static std::unique_ptr<Codec> create(std::shared_ptr<const Image> image, std::string mimeType);

private:
	Codecs();

	std::map<std::string,std::shared_ptr<Codec>> codecs;
};

#endif /* fiv__CODECS_HPP_ */
