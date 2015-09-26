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

#include <cairomm/refptr.h>
#include <cairomm/surface.h>
#include <exiv2/exif.hpp>
#include <memory>
#include <string>

#include "Codec.hpp"
#include "Image.hpp"

namespace Exiv2 {
class Image;
} /* namespace Exiv2 */

class JpegCodec: public Codec {
public:
	JpegCodec();
	JpegCodec(std::shared_ptr<const Image> image);
	virtual ~JpegCodec();
	virtual std::unique_ptr<Codec> getInstance(std::shared_ptr<const Image> image) const;
	virtual int getWidth();
	virtual int getHeight();
	virtual Image::Orientation getOrientation();
	virtual const Image::Properties getProperties();
	virtual Cairo::RefPtr<Cairo::ImageSurface> getPrimary();
	virtual Cairo::RefPtr<Cairo::ImageSurface> getThumbnail();

	static const std::string MIME_TYPE;

private:
	void initHeader();
	void initExiv2();
	std::unique_ptr<Exiv2::Image> getExiv2Data();
	void getCanonAF(const Exiv2::ExifData &exif);

	int width;
	int height;
	Image::Orientation orientation;
	Image::Properties properties;
};

#endif /* fiv__JPEGCODEC_HPP_ */
