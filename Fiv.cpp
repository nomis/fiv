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

#include "Fiv.hpp"

#include <dirent.h>
#include <sys/stat.h>
#include <unistd.h>
#include <algorithm>
#include <chrono>
#include <condition_variable>
#include <cstdio>
#include <deque>
#include <functional>
#include <iostream>
#include <iterator>
#include <list>
#include <memory>
#include <mutex>
#include <string>
#include <thread>
#include <unordered_map>
#include <unordered_set>
#include <utility>

#include "Codec.hpp"
#include "Events.hpp"
#include "FileDataBuffer.hpp"
#include "Image.hpp"

using namespace std;

const string Fiv::appName = "fiv";
const string Fiv::appId = "uk.uuid.fiv";

Fiv::Fiv() {
	initImagesComplete = false;
	initStop = false;
	maxPreload = 100;
	itCurrent = images.cend();

	mtxLoad = make_shared<mutex>();
	loadingRequired = make_shared<condition_variable>();
	preloadStarved = false;
}

Fiv::~Fiv() {
	loadingRequired->notify_all();
}

bool Fiv::init(int argc, char *argv[]) {
	unique_ptr<list<string>> args(make_unique<list<string>>());

	while (--argc > 0)
		args->emplace_back((const char *)(++argv)[0]);

	if (!args->size())
		args->emplace_back(".");


	if (!initImagesInBackground(move(args)))
		return false;

	unique_lock<mutex> lckImages(mtxImages);
	itCurrent = images.cbegin();
	preload();

	weak_ptr<Fiv> wSelf = shared_from_this();
	for (unsigned int i = 0; i < thread::hardware_concurrency(); i++)
		thread([wSelf]{ runLoader(wSelf); }).detach();

	return true;
}

void Fiv::exit() {
	unique_lock<mutex> lckImages(mtxImages);
	initStop = true;
	while (!initImagesComplete)
		imageAdded.wait(lckImages);
}

bool Fiv::initImagesInBackground(unique_ptr<list<string>> filenames_) {
	using namespace std::placeholders;

	thread(bind(&Fiv::initImagesThread, shared_from_this(), _1), move(filenames_)).detach();

	unique_lock<mutex> lckImages(mtxImages);
	if (!initImagesComplete)
		imageAdded.wait(lckImages);

	return images.size();
}

void Fiv::initImagesThread(unique_ptr<list<string>> filenames) {
	for (auto filename : *filenames) {
		struct stat st;

		if (access(filename.c_str(), R_OK)) {
			perror(filename.c_str());
			continue;
		}

		if (stat(filename.c_str(), &st))
			continue;

		if (S_ISREG(st.st_mode)) {
			unique_ptr<DataBuffer> buffer(make_unique<FileDataBuffer>(filename));
			shared_ptr<Image> image(make_shared<Image>(filename, move(buffer)));

			if (!addImage(image))
				goto stop;
		} else if (S_ISDIR(st.st_mode)) {
			deque<shared_ptr<Image>> dirImages;

			initImagesFromDir(filename, dirImages);

			for (auto image : dirImages)
				if (!addImage(image))
					goto stop;
		}
	}

stop:
	unique_lock<mutex> lckImages(mtxImages);
	initImagesComplete = true;
	imageAdded.notify_all();
}

static bool compareImage(const shared_ptr<Image> &a, const shared_ptr<Image> &b) {
	return a->name < b->name;
}

void Fiv::initImagesFromDir(const string &dirname, deque<shared_ptr<Image>> &dirImages) {
	DIR *dir = opendir(dirname.c_str());
	struct dirent *entry;

	if (dir == nullptr) {
		perror(dirname.c_str());
		return;
	}

	while ((entry = readdir(dir)) != NULL) {
		if (entry->d_type == DT_REG || entry->d_type == DT_LNK) {
			string filename = dirname + "/" + entry->d_name;

			if (entry->d_type == DT_LNK) {
				struct stat st;

				if (stat(filename.c_str(), &st))
					continue;

				if (!S_ISREG(st.st_mode)) // links must be to regular files
					continue;
			}

			if (access(filename.c_str(), R_OK)) {
				perror(filename.c_str());
				continue;
			}

			unique_ptr<DataBuffer> buffer(make_unique<FileDataBuffer>(filename));
			dirImages.push_back(make_shared<Image>(dirname == "." ? entry->d_name : filename, move(buffer)));
		}
	}

	closedir(dir);

	sort(dirImages.begin(), dirImages.end(), compareImage);
}

bool Fiv::addImage(shared_ptr<Image> image) {
	if (image->load()) {
		unique_lock<mutex> lckImages(mtxImages);

		if (initStop)
			return false;

		images.push_back(image);
		preload(true);
		imageAdded.notify_all();
	} else {
		unique_lock<mutex> lckImages(mtxImages);

		if (initStop)
			return false;
	}
	return true;
}

shared_ptr<Image> Fiv::current() {
	unique_lock<mutex> lckImages(mtxImages);
	return *itCurrent;
}

void Fiv::orientation(Image::Orientation modify) {
	unique_lock<mutex> lckImages(mtxImages);
	auto image = *itCurrent;
	image->setOrientation(modify);
	if (image->loadThumbnail())
		image->getThumbnail()->setOrientation(modify);
}

bool Fiv::first() {
	unique_lock<mutex> lckImages(mtxImages);
	if (itCurrent != images.cbegin()) {
		itCurrent = images.cbegin();
		preload();
		return true;
	}
	return false;
}

bool Fiv::previous() {
	unique_lock<mutex> lckImages(mtxImages);
	if (itCurrent != images.cbegin()) {
		itCurrent--;
		preload();
		return true;
	}
	return false;
}

bool Fiv::next() {
	unique_lock<mutex> lckImages(mtxImages);
	auto itNext = itCurrent;
	itNext++;
	if (itNext != images.cend()) {
		itCurrent++;
		preload();
		return true;
	}
	return false;
}

bool Fiv::last() {
	unique_lock<mutex> lckImages(mtxImages);
	auto itLast = images.cend();
	itLast--;
	if (itCurrent != itLast) {
		itCurrent = itLast;
		preload();
		return true;
	}
	return false;
}

void Fiv::addListener(weak_ptr<Events> listener) {
	listeners.push_back(listener);
}

vector<shared_ptr<Events>> Fiv::getListeners() {
	vector<shared_ptr<Events>> activeListeners;

	auto itListener = listeners.cbegin();

	while (itListener != listeners.cend()) {
		shared_ptr<Events> listener = itListener->lock();

		if (listener) {
			activeListeners.push_back(listener);
			itListener++;
		} else {
			itListener = listeners.erase(itListener);
		}
	}

	return activeListeners;
}

#ifndef __cpp_lib_make_reverse_iterator
template<class Iterator>
::std::reverse_iterator<Iterator> make_reverse_iterator(Iterator i) {
    return ::std::reverse_iterator<Iterator>(i);
}
#endif

void Fiv::preload(bool checkStarved) {
	unique_lock<mutex> lckLoad(*mtxLoad);

	if (checkStarved && !preloadStarved)
		return;

	auto start = chrono::steady_clock::now();

	unsigned int preload = maxPreload;
	auto itForward = itCurrent;
	auto itBackward = make_reverse_iterator(itCurrent);

	backgroundLoad.clear();
	backgroundLoad.push_back(*itCurrent);

	// Preload images forward and backward
	itForward++;
	while (true) {
		bool stop = true;

		if (itForward != images.cend()) {
			backgroundLoad.push_back(*(itForward++));

			if (--preload == 0)
				break;
			stop = false;
		}

		if (itBackward != images.crend()) {
			backgroundLoad.push_back(*(itBackward++));

			if (--preload == 0)
				break;
			stop = false;
		}

		if (stop)
			break;
	}

	preloadStarved = preload > 0;

	// Unload images that will not be preloaded
	unordered_set<shared_ptr<Image>> keep(backgroundLoad.cbegin(), backgroundLoad.cend());
	auto itLoaded = loaded.cbegin();
	while (itLoaded != loaded.cend()) {
		if (keep.find(*itLoaded) == loaded.end()) {
			(*itLoaded)->unloadPrimary();
			itLoaded = loaded.erase(itLoaded);
		} else {
			itLoaded++;
		}
	}

	// Start background loading for images that are not loaded
	auto itQueue = backgroundLoad.cbegin();
	while (itQueue != backgroundLoad.cend()) {
		if (loaded.find(*itQueue) == loaded.end()) {
			loadingRequired->notify_one();
			itQueue++;
		} else {
			itQueue = backgroundLoad.erase(itQueue);
		}
	}

	auto stop = chrono::steady_clock::now();
	cout << "preload queued " << backgroundLoad.size() << " in " << chrono::duration_cast<chrono::milliseconds>(stop - start).count() << "ms" << endl;
}

void Fiv::runLoader(weak_ptr<Fiv> wSelf) {
	shared_ptr<mutex> mtxLoad;
	shared_ptr<condition_variable> loadingRequired;

	{
		shared_ptr<Fiv> self = wSelf.lock();
		if (!self)
			return;

		mtxLoad = self->mtxLoad;
		loadingRequired = self->loadingRequired;
	}

	while (true) {
		shared_ptr<Image> image;

		{
			unique_lock<mutex> lckLoad(*mtxLoad);
			shared_ptr<Fiv> self = wSelf.lock();
			if (!self)
				return;

			if (self->backgroundLoad.empty())
				loadingRequired->wait(lckLoad);

			if (!self->backgroundLoad.empty()) {
				image = self->backgroundLoad.front();
				self->backgroundLoad.pop_front();
			}
		}

		if (!image)
			continue;

		bool loaded = image->loadPrimary();

		shared_ptr<Fiv> self = wSelf.lock();
		if (!self)
			return;

		if (loaded) {
			unique_lock<mutex> lckLoad(*mtxLoad);

			self->loaded.insert(image);
		}

		if (image.get() == self->current().get())
			for (auto listener : self->getListeners())
				listener->loadedCurrent();
	}
}

