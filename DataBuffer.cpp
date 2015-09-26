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

#include "DataBuffer.hpp"

#include <stddef.h>
#include <cstdint>
#include <iostream>

using namespace std;

DataBuffer::DataBuffer() {
	data = nullptr;
	length = 0;
}

DataBuffer::~DataBuffer() {

}

bool DataBuffer::load() {
	return data != nullptr;
}

void DataBuffer::unload() {
	data = nullptr;
	length = 0;
}

string DataBuffer::getFilename() {
	return "";
}

const uint8_t *DataBuffer::begin() const {
	return data;
}

const uint8_t *DataBuffer::end() const {
	if (data == nullptr)
		return nullptr;

	return data + length;
}

size_t DataBuffer::size() const {
	return length;
}
