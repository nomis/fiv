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

#ifndef fiv__FIV_HPP_
#define fiv__FIV_HPP_

#include <condition_variable>
#include <deque>
#include <list>
#include <memory>
#include <mutex>
#include <string>

#include "Image.hpp"

class Codec;

class Image;

class Fiv: public std::enable_shared_from_this<Fiv> {
public:
	class Images: public std::enable_shared_from_this<Images> {
	public:
		Images(std::shared_ptr<Fiv> fiv);
		std::shared_ptr<Image> current();
		void orientation(Image::Orientation modify);
		bool first();
		bool previous();
		bool next();
		bool last();

	private:
		std::shared_ptr<Fiv> fiv;
		std::list<std::shared_ptr<Image>>::const_iterator it;
	};

	Fiv();
	bool init(int argc, char *argv[]);
	void exit();
	std::shared_ptr<Fiv::Images> getImages();

	static const std::string appName;
	static const std::string appId;

private:
	bool initImagesInBackground(std::unique_ptr<std::list<std::string>> filenames);
	void initImagesThread(std::unique_ptr<std::list<std::string>> filenames);
	void initImagesFromDir(const std::string &dirname, std::deque<std::shared_ptr<Image>> &dirImages);
	bool addImage(std::shared_ptr<Image> image);

	std::mutex mtxImages;
	std::list<std::shared_ptr<Image>> images;
	std::condition_variable imageAdded;
	bool initImagesComplete;
	bool initStop;
};

#endif /* fiv__FIV_HPP_ */
