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
#include <condition_variable>
#include <cstdio>
#include <cstdlib>
#include <deque>
#include <iostream>
#include <memory>
#include <mutex>
#include <string>
#include <thread>

#include "Image.hpp"

using namespace std;

int Fiv::main(int argc, char *argv[]) {
	int ret;

	ret = initImages(argc, argv);
	if (ret != EXIT_SUCCESS)
		return ret;

	return EXIT_SUCCESS;
}

int Fiv::initImages(int argc, char *argv[]) {
	unique_ptr<deque<string>> args(make_unique<deque<string>>());

	while (--argc > 0)
		args->emplace_back((const char *)(++argv)[0]);

	if (!args->size())
		args->emplace_back(".");

	return initImagesInBackground(move(args));
}

int Fiv::initImagesInBackground(unique_ptr<deque<string>> filenames_) {
	auto self(shared_from_this());

	thread([this, self, filenames = move(filenames_)] () mutable {
		this->initImagesThread(move(filenames));
	}).detach();

	unique_lock<mutex> lckImages(mtxImages);
	if (!initImagesComplete)
		imageAdded.wait(lckImages);

	for (auto image : images)
		cout << *image << endl;

	return images.size() ? EXIT_SUCCESS : EXIT_FAILURE;
}

void Fiv::initImagesThread(unique_ptr<deque<string>> filenames) {
	for (auto filename : *filenames) {
		struct stat st;

		if (access(filename.c_str(), R_OK)) {
			perror(filename.c_str());
			continue;
		}

		if (stat(filename.c_str(), &st))
			continue;

		if (S_ISREG(st.st_mode)) {
			shared_ptr<Image> image(make_shared<Image>(filename));

			if (image->openFile()) {
				unique_lock<mutex> lckImages(mtxImages);

				images.push_back(image);
				imageAdded.notify_all();
			}
		} else if (S_ISDIR(st.st_mode)) {
			deque<shared_ptr<Image>> dirImages;

			initImagesFromDir(filename, dirImages);

			for (auto image : dirImages) {
				if (image->openFile()) {
					unique_lock<mutex> lckImages(mtxImages);

					images.push_back(image);
					imageAdded.notify_all();
				}
			}
		}
	}

	unique_lock<mutex> lckImages(mtxImages);
	initImagesComplete = true;
	imageAdded.notify_all();
}

static bool compareImage(const shared_ptr<Image> &a, const shared_ptr<Image> &b) {
	return a->filename < b->filename;
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

			dirImages.emplace_back(make_shared<Image>(filename));
		}
	}

	closedir(dir);

	sort(dirImages.begin(), dirImages.end(), compareImage);
}
