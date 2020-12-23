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

#include "JpegCodec.hpp"

#include <cairomm/enums.h>
#include <cairomm/refptr.h>
#include <cairomm/surface.h>
#include <cairomm/types.h>
#include <exiv2/easyaccess.hpp>
#include <exiv2/error.hpp>
#include <exiv2/exif.hpp>
#include <exiv2/image.hpp>
#include <exiv2/properties.hpp>
#include <exiv2/tags.hpp>
#include <exiv2/types.hpp>
#include <exiv2/xmp.hpp>
#include <turbojpeg.h>
#include <algorithm>
#include <chrono>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <ctime>
#include <iomanip>
#include <iostream>
#include <iterator>
#include <list>
#include <memory>
#include <sstream>
#include <string>
#include <utility>
#include <vector>

#include "Image.hpp"
#include "MemoryDataBuffer.hpp"

using namespace std;

const Exiv2::ExifKey Exif_Thumbnail_JPEGInterchangeFormat("Exif.Thumbnail.JPEGInterchangeFormat");
const Exiv2::ExifKey Exif_Image_DateTime("Exif.Image.DateTime");
const Exiv2::ExifKey Exif_Photo_SubSecTime("Exif.Photo.SubSecTime");
const Exiv2::ExifKey Exif_Image_DateTimeOriginal("Exif.Image.DateTimeOriginal");
const Exiv2::ExifKey Exif_Photo_SubSecTimeOriginal("Exif.Photo.SubSecTimeOriginal");
const Exiv2::ExifKey Exif_Photo_DateTimeDigitized("Exif.Photo.DateTimeDigitized");
const Exiv2::ExifKey Exif_Photo_SubSecTimeDigitized("Exif.Photo.SubSecTimeDigitized");
const Exiv2::ExifKey Exif_Image_ExposureBiasValue("Exif.Image.ExposureBiasValue");
const Exiv2::ExifKey Exif_Image_Flash("Exif.Image.Flash");
const Exiv2::XmpKey Xmp_xmp_Rating("Xmp.xmp.Rating");
const Exiv2::ExifKey Exif_Canon_AFInfo("Exif.Canon.AFInfo");

const string JpegCodec::MIME_TYPE = "image/jpeg";

JpegCodec::JpegCodec() {
	Exiv2::XmpParser::initialize();
	atexit(&Exiv2::XmpParser::terminate);

	width = 0;
	height = 0;
	orientation = Image::Orientation(Image::Rotate::ROTATE_NONE, false);
	properties = Image::Properties();
}

JpegCodec::JpegCodec(shared_ptr<const Image> image_) : Codec(image_) {
	width = 0;
	height = 0;
	orientation = Image::Orientation(Image::Rotate::ROTATE_NONE, false);
	properties = Image::Properties();
	initHeader();
	initExiv2();
}

unique_ptr<Codec> JpegCodec::getInstance(shared_ptr<const Image> image_) const {
	return make_unique<JpegCodec>(image_);
}

int JpegCodec::getWidth() const {
	return width;
}

int JpegCodec::getHeight() const {
	return height;
}

Image::Orientation JpegCodec::getOrientation() const {
	return orientation;
}

const Image::Properties JpegCodec::getProperties() const {
	return properties;
}

Cairo::RefPtr<Cairo::ImageSurface> JpegCodec::getPrimary() const {
	Cairo::RefPtr<Cairo::ImageSurface> surface;
	tjhandle tj;

	if (width <= 0 || height <= 0)
		goto err;

	tj = tjInitDecompress();
	if (!tj)
		goto err;

	surface = Cairo::ImageSurface::create(Cairo::Format::FORMAT_RGB24, width, height);
	if (!surface)
		goto err_tj;

	surface->flush();
	if (!tjDecompress2(tj, const_cast<uint8_t*>(image->begin()), image->size(), surface->get_data(),
			width, surface->get_stride(), height, TJPF_BGRX, TJFLAG_NOREALLOC)) {
		surface->mark_dirty();
	} else {
		surface = Cairo::RefPtr<Cairo::ImageSurface>();
	}

err_tj:
	tjDestroy(tj);
err:
	return surface;
}

Cairo::RefPtr<Cairo::ImageSurface> JpegCodec::getThumbnail() const {
	unique_ptr<Exiv2::Image> exiv2 = getExiv2Data();
	if (!exiv2)
		return Cairo::RefPtr<Cairo::ImageSurface>();

	auto exif = exiv2->exifData();
	auto dataTag = exif.findKey(Exif_Thumbnail_JPEGInterchangeFormat);
	if (dataTag == exif.end())
		return Cairo::RefPtr<Cairo::ImageSurface>();

	unique_ptr<MemoryDataBuffer> buffer = make_unique<MemoryDataBuffer>(dataTag->dataArea());
	shared_ptr<Image> thumbnail = make_shared<Image>(image->name + " <Exif_Thumbnail>", move(buffer));

	if (!thumbnail->load())
		return Cairo::RefPtr<Cairo::ImageSurface>();

	return thumbnail->getPrimary();
}

void JpegCodec::initHeader() {
	tjhandle tj = tjInitDecompress();
	if (tj) {
		int subsamp;

		if (!tjDecompressHeader2(tj, const_cast<uint8_t*>(image->begin()), image->size(), &width, &height, &subsamp)) {
		} else {
			cerr << image->name << ": TurboJPEG: error reading header" << endl;
		}
		tjDestroy(tj);
	}
}

unique_ptr<Exiv2::Image> JpegCodec::getExiv2Data() const {
	try {
		unique_ptr<Exiv2::Image> tmp = Exiv2::ImageFactory::open(image->begin(), image->size());
		if (tmp && tmp->good()) {
			tmp->readMetadata();
			return tmp;
		}
	} catch (const Exiv2::Error& e) {
		cerr << image->name << ": Exiv2: " << e.what() << endl;
	}
	return unique_ptr<Exiv2::Image>();
}

static bool getTimestamp(const Exiv2::ExifData &exif, Exiv2::ExifKey datetimeKey, Exiv2::ExifKey subsecKey, chrono::high_resolution_clock::time_point &timestamp) {
	auto timestampTag = exif.findKey(datetimeKey);
	if (timestampTag != exif.end()) {
		tm tm;
		char *ret = strptime(timestampTag->toString().c_str(), "%Y:%m:%d %H:%M:%S", &tm);

		if (ret != nullptr && strlen(ret) == 0) {
			timestamp = chrono::high_resolution_clock::from_time_t(mktime(&tm));

			auto timestampSubSecTag = exif.findKey(subsecKey);
			if (timestampSubSecTag != exif.end()) {
				stringstream subsec;
				const char *nptr;
				char *endp;

				subsec << left << setw(9) << setfill('0') << timestampSubSecTag->toString();
				nptr = subsec.str().substr(0, 9).c_str();
				chrono::nanoseconds ns(strtoull(nptr, &endp, 10));
				if (strlen(endp) == 0)
					timestamp += ns;
			}

			return true;
		}
	}

	return false;
}

static void getShort(const Exiv2::ExifData &exif, const Exiv2::ExifKey &exifKey, unsigned short &value) {
	auto exifTag = exif.findKey(exifKey);
	if (exifTag != exif.end())
		value = exifTag->toLong();
}

static void getLong(const Exiv2::ExifData &exif, Exiv2::ExifData::const_iterator (*func)(const Exiv2::ExifData& ed), long &value) {
	auto exifTag = func(exif);
	if (exifTag != exif.end())
		value = exifTag->toLong();
}

static void getLong(const Exiv2::XmpData &xmp, const Exiv2::XmpKey &xmpKey, long &value) {
	auto xmpTag = xmp.findKey(xmpKey);
	if (xmpTag != xmp.end())
		value = xmpTag->toLong();
}

static void getRational(const Exiv2::ExifData &exif, Exiv2::ExifData::const_iterator (*func)(const Exiv2::ExifData& ed), double &value) {
	auto exifTag = func(exif);
	if (exifTag != exif.end()) {
		Exiv2::Rational r = exifTag->toRational();
		value = (double)r.first / r.second;
	}
}

static void getRational(const Exiv2::ExifData &exif, const Exiv2::ExifKey &exifKey, double &value) {
	auto exifTag = exif.findKey(exifKey);
	if (exifTag != exif.end()) {
		Exiv2::Rational r = exifTag->toRational();
		value = (double)r.first / r.second;
	}
}

void JpegCodec::getCanonAF(const Exiv2::ExifData &exif) {
	vector<Cairo::Rectangle> afPoints;

	auto exifTag = exif.findKey(Exif_Canon_AFInfo);
	if (exifTag == exif.end())
		return;

	long count = exifTag->toLong(0) / 2;

	int pos = 2;
	if (count < pos + 6)
		return;

	long numAFPoints = exifTag->toLong(pos++);
	long validAFPoints = exifTag->toLong(pos++);
	long imgWidth = exifTag->toLong(pos++);
	long imgHeight = exifTag->toLong(pos++);
	long afWidth = exifTag->toLong(pos++);
	long afHeight = exifTag->toLong(pos++);

	if (imgWidth != width || imgHeight != height)
		return;

	if (count < pos + numAFPoints * 4)
		return;

	for (long i = 0; i < validAFPoints; i++) {
		Cairo::Rectangle rect;

		rect.width = (int16_t)exifTag->toLong(8 + i);
		rect.height = (int16_t)exifTag->toLong(8 + numAFPoints + i);
		rect.x = (int16_t)exifTag->toLong(8 + numAFPoints * 2 + i);
		rect.y = -(int16_t)exifTag->toLong(8 + numAFPoints * 3 + i);

		rect.x += (double)afWidth / 2;
		rect.y += (double)afHeight / 2;

		rect.width *= (double)afWidth / imgWidth;
		rect.height *= (double)afHeight / imgHeight;
		rect.x *= (double)afWidth / imgWidth;
		rect.y *= (double)afHeight / imgHeight;

		afPoints.push_back(rect);
		properties.focusPoints.push_back(rect);
	}

	pos += numAFPoints * 4;

	long bitfieldAFPoints = (numAFPoints + 15) / 16;
	if (count < pos + bitfieldAFPoints)
		return;

	for (long i = 0; i < validAFPoints; i++)
		if ((exifTag->toLong(pos + i/16) & 1 << (i % 16)) != 0)
			properties.focusPointsActive.insert(afPoints.at(i));

	pos += bitfieldAFPoints;
	if (count < pos + bitfieldAFPoints)
		return;

	for (long i = 0; i < validAFPoints; i++)
		if ((exifTag->toLong(pos + i/16) & 1 << (i % 16)) != 0)
			properties.focusPointsSelected.insert(afPoints.at(i));
}

void JpegCodec::initExiv2() {
	unique_ptr<Exiv2::Image> exiv2 = getExiv2Data();
	if (!exiv2)
		return;

	auto exif = exiv2->exifData();
	auto xmp = exiv2->xmpData();

	auto orientationTag = Exiv2::orientation(exif);
	if (orientationTag != exif.end() && orientationTag->toLong() >= 1 && orientationTag->toLong() <= 8) {
		static const Image::Orientation ORIENTATION_MAP[8] = {
				/* Horizontal (normal) */
				Image::Orientation(Image::Rotate::ROTATE_NONE, false),

				/* Mirror horizontal */
				Image::Orientation(Image::Rotate::ROTATE_NONE, true),

				/* Rotate 180 */
				Image::Orientation(Image::Rotate::ROTATE_180, false),

				/* Mirror vertical */
				Image::Orientation(Image::Rotate::ROTATE_180, true),

				/* Mirror horizontal and rotate 270 CW */
				Image::Orientation(Image::Rotate::ROTATE_270, true),

				/* Rotate 90 CW */
				Image::Orientation(Image::Rotate::ROTATE_90, false),

				/* Mirror horizontal and rotate 90 CW */
				Image::Orientation(Image::Rotate::ROTATE_90, true),

				/* Rotate 270 CW */
				Image::Orientation(Image::Rotate::ROTATE_270, false)
		};

		orientation = ORIENTATION_MAP[orientationTag->toLong() - 1];
	}

	if (!getTimestamp(exif, Exif_Image_DateTimeOriginal, Exif_Photo_SubSecTimeOriginal, properties.timestamp))
		if (!getTimestamp(exif, Exif_Image_DateTime, Exif_Photo_SubSecTime, properties.timestamp))
			getTimestamp(exif, Exif_Photo_DateTimeDigitized, Exif_Photo_SubSecTimeDigitized, properties.timestamp);

	getLong(exif, &Exiv2::isoSpeed, properties.isoSpeed);
	getRational(exif, &Exiv2::fNumber, properties.fAperture);
	getRational(exif, &Exiv2::focalLength, properties.mmFocalLength);
	getRational(exif, &Exiv2::exposureTime, properties.sExposureTime);
	getRational(exif, Exif_Image_ExposureBiasValue, properties.evExposureBias);
	getShort(exif, Exif_Image_Flash, properties.flash);
	getRational(exif, &Exiv2::flashBias, properties.evFlashBias);
	getLong(xmp, Xmp_xmp_Rating, properties.rating);
	getCanonAF(exif);
}
