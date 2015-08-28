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

#ifndef FIV_HPP_
#define FIV_HPP_

#include <deque>
#include <memory>
#include <string>

class Image;

class Fiv {
public:
	Fiv();

	int main(int argc, char *argv[]);

private:
	int initImages(int argc, char *argv[]);
	int initImages(std::deque<std::string> filenames);
	void initImagesFromDir(const std::string &dirname);

	std::deque<std::shared_ptr<Image>> images;
};



#endif /* FIV_HPP_ */
