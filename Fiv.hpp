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
#include <map>
#include <memory>
#include <mutex>
#include <string>

#include "Codec.hpp"

class Image;

class Fiv : public std::enable_shared_from_this<Fiv> {
public:
	int main(int argc, char *argv[]);
	static std::unique_ptr<Codec> getCodec(std::shared_ptr<const Image> image, std::string mimeType);

private:
	void initCodecs();
	int initImages(int argc, char *argv[]);
	int initImagesInBackground(std::unique_ptr<std::deque<std::string>> filenames);
	void initImagesThread(std::unique_ptr<std::deque<std::string>> filenames);
	void initImagesFromDir(const std::string &dirname, std::deque<std::shared_ptr<Image>> &dirImages);

	static std::map<std::string,std::shared_ptr<Codec>> codecs;

	std::mutex mtxImages;
	std::deque<std::shared_ptr<Image>> images;
	std::condition_variable imageAdded;
	bool initImagesComplete;
};



#endif /* fiv__FIV_HPP_ */
