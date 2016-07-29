/*
 Copyright 2015-2016  Simon Arlott

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

#include "Magic.hpp"

#include <magic.h>
#include <stddef.h>
#include <stdio.h>
#include <cstdint>
#include <iostream>
#include <string>

#include "ThreadLocalStream.hpp"

using namespace std;

thread_local Magic Magic::instance;

Magic::Magic() {
	cookie = magic_open(MAGIC_MIME_TYPE|MAGIC_ERROR|MAGIC_NO_CHECK_BUILTIN);
	if (cookie == nullptr) {
		ThreadLocalEStream::perror("magic_open");
		return;
	}

	if (magic_load(cookie, NULL)) {
		cerr << "magic_load: " << magic_error(cookie) << endl;
		return;
	}
}

Magic::~Magic() {
	if (cookie != nullptr) {
		magic_close(cookie);
		cookie = nullptr;
	}
}

string Magic::identify(const uint8_t *data, size_t size) {
	if (instance.cookie == nullptr)
		return "";

	return magic_buffer(instance.cookie, data, size);
}
