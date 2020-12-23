/*
 Copyright 2015,2020  Simon Arlott

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

#ifndef fiv__DATABUFFER_HPP_
#define fiv__DATABUFFER_HPP_

#include <stddef.h>
#include <cstdint>
#include <string>

class DataBuffer {
public:
	virtual ~DataBuffer() = default;
	virtual bool load() = 0;
	virtual void unload() = 0;
	virtual std::string getFilename();
	const uint8_t *begin() const;
	const uint8_t *end() const;
	size_t size() const;

protected:
	DataBuffer() = default;

	const uint8_t *data = nullptr;
	size_t length = 0;
};

#endif /* fiv__DATABUFFER_HPP_ */
