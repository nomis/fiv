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

#include <cassert>
#include <chrono>
#include <condition_variable>
#include <deque>
#include <iostream>
#include <iterator>
#include <list>
#include <memory>
#include <mutex>
#include <thread>
#include <unordered_map>
#include <unordered_set>
#include <utility>

#include "Fiv.hpp"
#include "Image.hpp"

using namespace std;

Fiv::Images::Images(shared_ptr<Fiv> fiv_) : fiv(fiv_) {
	mtxLoad = make_shared<mutex>();
	loadingRequired = make_shared<condition_variable>();

	unique_lock<mutex> lckImages(fiv->mtxImages);
	assert(fiv->images.size());
	itCurrent = fiv->images.cbegin();
	preload();
}

void Fiv::Images::start() {
	for (unsigned int i = 0; i < thread::hardware_concurrency(); i++) {
		weak_ptr<Fiv::Images> wSelf = shared_from_this();

		thread([wSelf]{ runLoader(wSelf); }).detach();
	}
}

Fiv::Images::~Images() {
	loadingRequired->notify_all();
}

shared_ptr<Image> Fiv::Images::current() {
	unique_lock<mutex> lckImages(fiv->mtxImages);
	return *itCurrent;
}

void Fiv::Images::orientation(Image::Orientation modify) {
	unique_lock<mutex> lckImages(fiv->mtxImages);
	auto image = *itCurrent;
	image->setOrientation(modify);
	if (image->loadThumbnail())
		image->getThumbnail()->setOrientation(modify);
}

bool Fiv::Images::first() {
	unique_lock<mutex> lckImages(fiv->mtxImages);
	if (itCurrent != fiv->images.cbegin()) {
		itCurrent = fiv->images.cbegin();
		preload();
		return true;
	}
	return false;
}

bool Fiv::Images::previous() {
	unique_lock<mutex> lckImages(fiv->mtxImages);
	if (itCurrent != fiv->images.cbegin()) {
		itCurrent--;
		preload();
		return true;
	}
	return false;
}

bool Fiv::Images::next() {
	unique_lock<mutex> lckImages(fiv->mtxImages);
	auto itNext = itCurrent;
	itNext++;
	if (itNext != fiv->images.cend()) {
		itCurrent++;
		preload();
		return true;
	}
	return false;
}

bool Fiv::Images::last() {
	unique_lock<mutex> lckImages(fiv->mtxImages);
	auto itLast = fiv->images.cend();
	itLast--;
	if (itCurrent != itLast) {
		itCurrent = itLast;
		preload();
		return true;
	}
	return false;
}

#ifndef __cpp_lib_make_reverse_iterator
template<class Iterator>
::std::reverse_iterator<Iterator> make_reverse_iterator(Iterator i) {
    return ::std::reverse_iterator<Iterator>(i);
}
#endif

void Fiv::Images::preload() {
	unique_lock<mutex> lckLoad(*mtxLoad);

	auto start = chrono::steady_clock::now();

	unsigned int preload = fiv->getMaxPreload();
	auto itForward = itCurrent;
	auto itBackward = make_reverse_iterator(itCurrent);

	backgroundLoad.clear();
	backgroundLoad.push_back(*itCurrent);

	// Preload images forward and backward
	itForward++;
	while (true) {
		bool stop = true;

		if (itForward != fiv->images.cend()) {
			backgroundLoad.push_back(*(itForward++));

			if (--preload == 0)
				break;
			stop = false;
		}

		if (itBackward != fiv->images.crend()) {
			backgroundLoad.push_back(*(itBackward++));

			if (--preload == 0)
				break;
			stop = false;
		}

		if (stop)
			break;
	}

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

void Fiv::Images::runLoader(weak_ptr<Images> wSelf) {
	shared_ptr<mutex> mtxLoad;
	shared_ptr<condition_variable> loadingRequired;

	{
		shared_ptr<Images> self = wSelf.lock();
		if (!self)
			return;

		mtxLoad = self->mtxLoad;
		loadingRequired = self->loadingRequired;
	}

	while (true) {
		shared_ptr<Image> image;

		{
			unique_lock<mutex> lckLoad(*mtxLoad);
			shared_ptr<Images> self = wSelf.lock();
			if (!self)
				return;

			if (self->backgroundLoad.empty())
				loadingRequired->wait(lckLoad);

			if (!self->backgroundLoad.empty()) {
				image = self->backgroundLoad.front();
				self->backgroundLoad.pop_front();
			}
		}

		if (image && image->loadPrimary()) {
			unique_lock<mutex> lckLoad(*mtxLoad);
			shared_ptr<Images> self = wSelf.lock();
			if (!self)
				return;

			self->loaded.insert(image);
		}
	}
}
