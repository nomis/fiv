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

#include <cairomm/refptr.h>
#include <cairomm/surface.h>
#include <stddef.h>
#include <cstdint>
#include <iostream>
#include <memory>
#include <mutex>
#include <string>
#include <utility>

class DataBuffer;
class TextureDataBuffer;

class Codec;

class Image: public std::enable_shared_from_this<Image> {
	friend class Codec;

public:
	enum Rotate {
		ROTATE_NONE,
		ROTATE_90,
		ROTATE_180,
		ROTATE_270
	};

	typedef bool HFlip;

	typedef std::pair<Rotate,HFlip> Orientation;

	Image(const std::string &name, std::unique_ptr<DataBuffer> buffer);
	Image(const std::string &name, std::unique_ptr<DataBuffer> buffer, Orientation orientation);
	friend std::ostream& operator<<(std::ostream &stream, const Image &image);

	bool load();
	const uint8_t *begin() const;
	const uint8_t *end() const;
	size_t size() const;
	int width();
	int height();

	bool loadPrimary();
	bool isPrimaryLoaded();
	Cairo::RefPtr<Cairo::Surface> getPrimary();
	void unloadPrimary();
	Image::Orientation getOrientation();
	void setOrientation(Image::Orientation modify);

	bool loadThumbnail();
	std::shared_ptr<Image> getThumbnail() const;
	void unloadThumbnail();

	const std::string name;

private:
	std::unique_ptr<DataBuffer> buffer;
	std::string mimeType;
	bool autoOrientation;
	Orientation orientation;
	std::unique_ptr<Codec> codec;
	std::mutex mtxPrimary;
	std::mutex mtxPrimaryLoad;
	Cairo::RefPtr<Cairo::Surface> primary;
	bool primaryUnload;
	bool primaryFailed;
	std::shared_ptr<Image> thumbnail;
	bool thumbnailFailed;
};

#endif /* fiv__IMAGE_HPP_ */
