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
#include <fcntl.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <dirent.h>
#include <unistd.h>
#include <cstdlib>
#include <vector>
#include <deque>
#include <iostream>
#include <memory>
#include <string>
#include <cstring>
#include <algorithm>
#include <errno.h>

#include "Image.hpp"

using namespace std;

Fiv::Fiv() {

}

int Fiv::main(int argc, char *argv[]) {
	initImages(argc, argv);

	return EXIT_SUCCESS;
}

int Fiv::initImages(int argc, char *argv[]) {
	deque<string> args;

	while (--argc > 0)
		args.emplace_back((const char *)(++argv)[0]);

	if (!args.size())
		args.emplace_back(".");

	return initImages(args);
}

int Fiv::initImages(deque<string> filenames) {
	for (auto filename : filenames) {
		struct stat st;

		if (access(filename.c_str(), R_OK)) {
			perror(filename.c_str());
			continue;
		}

		if (stat(filename.c_str(), &st))
			continue;

		if (S_ISREG(st.st_mode)) {
			images.emplace_back(make_shared<Image>(filename));
		} else if (S_ISDIR(st.st_mode)) {
			initImagesFromDir(filename);
		}
	}

	for (auto image : images)
		cout << *image << endl;

	// TODO init images in order in background thread and wait for at least one successful image before returning

	return images.size() ? EXIT_SUCCESS : EXIT_FAILURE;
}

static bool compareImage(const shared_ptr<Image> &a, const shared_ptr<Image> &b) {
	return a->filename < b->filename;
}

void Fiv::initImagesFromDir(const string &dirname) {
	DIR *dir = opendir(dirname.c_str());
	struct dirent *entry;
	auto pos = images.end();

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

			images.emplace_back(make_shared<Image>(filename));
		}
	}

	closedir(dir);

	sort(pos, images.end(), compareImage); // sort newly added images only
}
