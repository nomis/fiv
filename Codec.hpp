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

#ifndef fiv__CODEC_HPP_
#define fiv__CODEC_HPP_

#include <cairomm/refptr.h>
#include <cairomm/surface.h>
#include <memory>

#include "Image.hpp"

class TextureDataBuffer;

class DataBuffer;

class Image;

class Codec {
public:
	Codec();
	virtual ~Codec();
	virtual std::unique_ptr<Codec> getInstance(std::shared_ptr<const Image> image) const;
	virtual int getWidth();
	virtual int getHeight();
	virtual Image::Orientation getOrientation();
	virtual const Image::Properties getProperties();
	virtual Cairo::RefPtr<Cairo::ImageSurface> getPrimary();
	virtual Cairo::RefPtr<Cairo::ImageSurface> getThumbnail();

protected:
	Codec(std::shared_ptr<const Image> image);

	std::shared_ptr<const Image> image;
};

#endif /* fiv__CODEC_HPP_ */
