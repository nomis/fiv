/*
	Copyright 2015  Simon Arlott

	This program is free software: you can redistribute it and/or modify
	it under the terms of the GNU Affero General Public License as published by
	the Free Software Foundation, either version 3 of the License, or
	(at your option) any later version.

	This program is distributed in the hope that it will be useful,
	but WITHOUT ANY WARRANTY; without even the implied warranty of
	MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
	GNU Affero General Public License for more details.

	You should have received a copy of the GNU Affero General Public License
	along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

#include "Fiv.hpp"

#include <dirent.h>
#include <sys/stat.h>
#include <unistd.h>
#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <deque>
#include <iostream>
#include <memory>
#include <string>
#include <thread>
#include <condition_variable>

#include "Image.hpp"

using namespace std;

int Fiv::main(int argc, char *argv[]) {
	shared_ptr<deque<shared_ptr<Image>>> loadImages(make_unique<deque<shared_ptr<Image>>>());
	int ret;

	ret = initImages(argc, argv, loadImages);
	if (ret != EXIT_SUCCESS)
		return ret;

	ret = loadImagesInBackground(loadImages);
	if (ret != EXIT_SUCCESS)
		return ret;

	loadImages = nullptr;

	return EXIT_SUCCESS;
}

int Fiv::loadImagesInBackground(shared_ptr<deque<shared_ptr<Image>>> loadImages) {
	auto self(shared_from_this());
	shared_ptr<condition_variable> imageLoaded(make_shared<condition_variable>());

	thread([this, self, loadImages, imageLoaded] {
		bool opened = false;

		for (auto image : *loadImages) {
			if (image->openFile()) {
				unique_lock<mutex> lckImages(mtxImages);

				images.push_back(image);
				imageLoaded->notify_all();
				opened = true;
			}
		}

		if (!opened) {
			unique_lock<mutex> lckImages(mtxImages);
			imageLoaded->notify_all();
		}
	}).detach();

	unique_lock<mutex> lckImages(mtxImages);
	imageLoaded->wait(lckImages);

	for (auto image : images)
		cout << *image << endl;

	return images.size() ? EXIT_SUCCESS : EXIT_FAILURE;
}

int Fiv::initImages(int argc, char *argv[], shared_ptr<deque<shared_ptr<Image>>> loadImages) {
	deque<string> args;

	while (--argc > 0)
		args.emplace_back((const char *)(++argv)[0]);

	if (!args.size())
		args.emplace_back(".");

	return initImages(args, loadImages);
}

int Fiv::initImages(deque<string> filenames, shared_ptr<deque<shared_ptr<Image>>> loadImages) {
	for (auto filename : filenames) {
		struct stat st;

		if (access(filename.c_str(), R_OK)) {
			perror(filename.c_str());
			continue;
		}

		if (stat(filename.c_str(), &st))
			continue;

		if (S_ISREG(st.st_mode)) {
			loadImages->emplace_back(make_shared<Image>(filename));
		} else if (S_ISDIR(st.st_mode)) {
			initImagesFromDir(filename, loadImages);
		}
	}

	return loadImages->size() ? EXIT_SUCCESS : EXIT_FAILURE;
}

static bool compareImage(const shared_ptr<Image> &a, const shared_ptr<Image> &b) {
	return a->filename < b->filename;
}

void Fiv::initImagesFromDir(const string &dirname, shared_ptr<deque<shared_ptr<Image>>> loadImages) {
	DIR *dir = opendir(dirname.c_str());
	struct dirent *entry;
	auto pos = loadImages->end();

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

			loadImages->emplace_back(make_shared<Image>(filename));
		}
	}

	closedir(dir);

	sort(pos, loadImages->end(), compareImage); // sort newly added images only
}
