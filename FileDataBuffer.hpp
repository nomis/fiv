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

#ifndef fiv__FILEDATABUFFER_HPP_
#define fiv__FILEDATABUFFER_HPP_

#include <string>

#include "DataBuffer.hpp"

class FileDataBuffer: public DataBuffer {
public:
	FileDataBuffer(const std::string &filename);
	virtual ~FileDataBuffer();
	virtual bool load();
	virtual void unload();
	virtual std::string getFilename();

private:
	std::string filename;
	void *mapping;
};

#endif /* fiv__FILEDATABUFFER_HPP_ */
