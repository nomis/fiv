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

#ifndef fiv__IMAGE_HPP_
#define fiv__IMAGE_HPP_

#include <cairomm/cairomm.h>
#include <stddef.h>
#include <cstdint>
#include <iostream>
#include <memory>
#include <string>

class DataBuffer;
class TextureDataBuffer;

class Codec;

class Image: public std::enable_shared_from_this<Image> {
	friend class Codec;

public:
	enum Orientation {
		AUTO,

		/* Horizontal (normal) */
		NORMAL,

		/* Mirror horizontal */
		MIRROR_HORIZONTAL,

		/* Rotate 180 */
		ROTATE_180,

		/* Mirror vertical */
		MIRROR_VERTICAL,

		/* Mirror horizontal and rotate 270 CW */
		MIRROR_HORIZONTAL_ROTATE_270,

		/* Rotate 90 CW */
		ROTATE_90,

		/* Mirror horizontal and rotate 90 CW */
		MIRROR_HORIZONTAL_ROTATE_90,

		/* Rotate 270 CW */
		ROTATE_270
	};

	Image(const std::string &name, std::unique_ptr<DataBuffer> buffer, Orientation orientation = AUTO);
	friend std::ostream& operator<<(std::ostream &stream, const Image &image);
	static Image::Orientation rotateLeft(Image::Orientation orientation);
	static Image::Orientation rotateRight(Image::Orientation orientation);

	bool load();
	const uint8_t *begin() const;
	const uint8_t *end() const;
	size_t size() const;
	int width();
	int height();

	bool loadPrimary();
	Cairo::RefPtr<Cairo::Surface> getPrimary();
	Image::Orientation getOrientation();

	bool loadThumbnail();
	std::shared_ptr<Image> getThumbnail() const;
	void unloadThumbnail();

	const std::string name;

private:
	std::unique_ptr<DataBuffer> buffer;
	std::string mimeType;
	Orientation orientation;
	std::unique_ptr<Codec> codec;
	Cairo::RefPtr<Cairo::Surface> primary;
	bool primaryFailed;
	std::shared_ptr<Image> thumbnail;
	bool thumbnailFailed;
};

#endif /* fiv__IMAGE_HPP_ */
