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

#include "Codecs.hpp"

#include <map>
#include <memory>
#include <stdexcept>
#include <string>

#include "Image.hpp"
#include "JpegCodec.hpp"

using namespace std;

Codecs::Codecs() {
	codecs[JpegCodec::MIME_TYPE] = make_shared<JpegCodec>();
}

unique_ptr<Codec> Codecs::create(shared_ptr<const Image> image, string mimeType) {
	static Codecs instance;
	shared_ptr<Codec> codec;
	try {
		codec = instance.codecs.at(mimeType);
	} catch (const out_of_range &oor) {
		return unique_ptr<Codec>();
	}
	return codec->getInstance(image);
}

