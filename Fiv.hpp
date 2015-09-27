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
#include <future>
#include <list>
#include <memory>
#include <mutex>
#include <string>
#include <tuple>
#include <unordered_set>
#include <vector>

#include "Image.hpp"

class Application;

class Events;

class Fiv: public std::enable_shared_from_this<Fiv> {
	friend Application;

public:
	Fiv();
	virtual ~Fiv();
	bool init(int argc, char *argv[]);
	void exit();
	std::shared_ptr<Image> current();
	void orientation(Image::Orientation modify);
	bool first();
	bool previous();
	bool next();
	bool last();
	std::tuple<int,int,bool> position();

	bool hasMarkSupport();
	bool isMarked(std::shared_ptr<Image> image);
	bool mark(std::shared_ptr<Image> image);
	bool toggleMark(std::shared_ptr<Image> image);
	bool unmark(std::shared_ptr<Image> image);

	void addListener(std::weak_ptr<Events> listener);

	static const std::string appName;
	static const std::string appId;

private:
	bool initImagesInBackground(std::unique_ptr<std::list<std::string>> filenames);
	void initImagesThread(std::unique_ptr<std::list<std::string>> filenames);
	bool initImagesFromDir(const std::string &dirname, std::deque<std::shared_ptr<Image>> &dirImages);
	bool backgroundInitImage(std::deque<std::future<std::shared_ptr<Image>>> &bgImages, std::shared_ptr<Image> image);
	bool processBackgroundInitImages(std::deque<std::future<std::shared_ptr<Image>>> &bgImages, bool all = true);
	bool addImage(std::shared_ptr<Image> image);
	void preload(bool checkStarved = false);
	bool getMarkStatus(std::shared_ptr<Image> image, std::string &filename, std::string &linkname, bool &marked);

	std::vector<std::shared_ptr<Events>> getListeners();

	static void runLoader(std::weak_ptr<Fiv> wSelf);

	std::mutex mtxImages;
	std::list<std::shared_ptr<Image>> images;
	std::condition_variable imageAdded;
	std::list<std::shared_ptr<Image>>::const_iterator itCurrent;
	bool initImagesComplete;
	bool initStop;
	std::string markDirectory;

	int maxPreload;
	std::shared_ptr<std::mutex> mtxLoad;
	std::unordered_set<std::shared_ptr<Image>> loaded;
	std::deque<std::shared_ptr<Image>> backgroundLoad;
	std::shared_ptr<std::condition_variable> loadingRequired;
	bool preloadStarved;

	std::mutex mtxListeners;
	std::vector<std::weak_ptr<Events>> listeners;
};

#endif /* fiv__FIV_HPP_ */
