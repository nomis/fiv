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

#ifndef JPEGCODEC_HPP_
#define JPEGCODEC_HPP_

#include <libexif/exif-data.h>
#include <libexif/exif-loader.h>
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
	virtual void getThumbnail();

	static const std::string MIME_TYPE;

private:
	class Exif {
	public:
		Exif(std::shared_ptr<const Image> image);
		Exif(uint8_t *data, size_t size);
		~Exif();
		void getThumbnail();

	private:
		ExifLoader *exifLoader;
		ExifData *exifData;
	};
};

#endif /* JPEGCODEC_HPP_ */
