/*
 Copyright 2015,2020  Simon Arlott

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
	explicit JpegCodec(std::shared_ptr<const Image> image);
	std::unique_ptr<Codec> getInstance(std::shared_ptr<const Image> image) const override;
	int getWidth() const override;
	int getHeight() const override;
	Image::Orientation getOrientation() const override;
	const Image::Properties getProperties() const override;
	Cairo::RefPtr<Cairo::ImageSurface> getPrimary() const override;
	Cairo::RefPtr<Cairo::ImageSurface> getThumbnail() const override;

	static const std::string MIME_TYPE;

private:
	void initHeader();
	void initExiv2();
	std::unique_ptr<Exiv2::Image> getExiv2Data() const;
	void getCanonAF(const Exiv2::ExifData &exif);

	int width;
	int height;
	Image::Orientation orientation;
	Image::Properties properties;
};

#endif /* fiv__JPEGCODEC_HPP_ */
