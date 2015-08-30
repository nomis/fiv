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

#ifndef fiv__JPEGCODEC_HPP_
#define fiv__JPEGCODEC_HPP_

#include <exiv2/image.hpp>
#include <stddef.h>
#include <cstdint>
#include <memory>
#include <string>

#include "Codec.hpp"

class JpegCodec: public Codec {
public:
	JpegCodec();
	JpegCodec(std::shared_ptr<const Image> image);
	virtual ~JpegCodec();
	virtual std::unique_ptr<Codec> getInstance(std::shared_ptr<const Image> image) const;
	virtual std::shared_ptr<Image> getThumbnail();

	static const std::string MIME_TYPE;

private:
	bool initExiv2();
	bool initExif();

	static const Exiv2::ExifKey Exif_Thumbnail_JPEGInterchangeFormat;
	static const Exiv2::ExifKey Exif_Thumbnail_JPEGInterchangeFormatLength;

	std::unique_ptr<Exiv2::Image> exiv2;
	Exiv2::ExifData exif;
	bool exiv2Error;
};

#endif /* fiv__JPEGCODEC_HPP_ */
