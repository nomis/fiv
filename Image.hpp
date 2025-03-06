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
#include <cairomm/types.h>
#include <stddef.h>
#include <chrono>
#include <cstdint>
#include <iostream>
#include <list>
#include <memory>
#include <mutex>
#include <set>
#include <string>
#include <utility>

class DataBuffer;
class TextureDataBuffer;

class Codec;

struct RectCompare {
	bool operator() (const Cairo::Rectangle &a, const Cairo::Rectangle &b) const {
		if (a.x == b.x) {
			if (a.y == b.y) {
				if (a.width == b.width) {
					if (a.height == b.height) {
						return false;
					} else {
						return a.height < b.height;
					}
				} else {
					return a.width < b.width;
				}
			} else {
				return a.y < b.y;
			}
		} else {
			return a.x < b.x;
		}
	}
};

class Image: public std::enable_shared_from_this<Image> {
	friend class Codec;
	friend class Fiv;

public:
	enum Rotate {
		ROTATE_NONE,
		ROTATE_90,
		ROTATE_180,
		ROTATE_270
	};

	typedef bool HFlip;

	typedef std::pair<Rotate,HFlip> Orientation;

	class Properties {
	public:
		Properties();

		std::chrono::high_resolution_clock::time_point timestamp;
		long isoSpeed;
		double fAperture;
		double mmFocalLength;
		double sExposureTime;
		double evExposureBias;
		unsigned short flash;
		double evFlashBias;

		long rating;
		std::list<Cairo::Rectangle> focusPoints;
		std::set<Cairo::Rectangle, RectCompare> focusPointsSelected;
		std::set<Cairo::Rectangle, RectCompare> focusPointsActive;
	};

	Image(const std::string &name, std::unique_ptr<DataBuffer> buffer);
	Image(const std::string &name, std::unique_ptr<DataBuffer> buffer, Orientation orientation);
	friend std::ostream& operator<<(std::ostream &stream, const Image &image);
	std::string getFilename();

	bool load();
	const uint8_t *begin() const;
	const uint8_t *end() const;
	size_t size() const;
	int width();
	int height();

	Image::Orientation getOrientation();
	void setOrientation(Image::Orientation modify);
	const Image::Properties getProperties();

	bool loadPrimary();
	bool isPrimaryFailed();
	Cairo::RefPtr<Cairo::ImageSurface> getPrimary();
	void unloadPrimary();

	bool loadThumbnail();
	bool isThumbnailFailed();
	Cairo::RefPtr<Cairo::ImageSurface> getThumbnail();
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
	Cairo::RefPtr<Cairo::ImageSurface> primary;
	bool primaryUnload;
	bool primaryFailed;

	std::mutex mtxThumbnail;
	std::mutex mtxThumbnailLoad;
	Cairo::RefPtr<Cairo::ImageSurface> thumbnail;
	bool thumbnailUnload;
	bool thumbnailFailed;
};

#endif /* fiv__IMAGE_HPP_ */
