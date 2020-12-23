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

#include "ThreadLocalStream.hpp"

#include <errno.h>
#include <string.h>
#include <iostream>
#include <sstream>
#include <streambuf>

using namespace std;

thread_local ostringstream ThreadLocalOStream::buffer(stringstream::binary);
ThreadLocalOStream ThreadLocalOStream::instance;

thread_local ostringstream ThreadLocalEStream::buffer(stringstream::binary);
ThreadLocalEStream ThreadLocalEStream::instance;

ThreadLocalOStream::ThreadLocalOStream() : output(cout.rdbuf()) {

}

char_traits<char>::int_type ThreadLocalOStream::overflow(char_traits<char>::int_type c) {
	buffer.put(c);
	if (c == '\n') {
		unique_lock<mutex> lckOutput(mtxOutput);
		output << buffer.str() << flush;
		buffer.str("");
		buffer.clear();
	}
	return char_traits<char>::not_eof(c);
}

ThreadLocalEStream::ThreadLocalEStream() : output(cerr.rdbuf()) {

}

char_traits<char>::int_type ThreadLocalEStream::overflow(char_traits<char>::int_type c) {
	buffer.put(c);
	if (c == '\n') {
		unique_lock<mutex> lckOutput(mtxOutput);
		output << buffer.str() << flush;
		buffer.str("");
		buffer.clear();
	}
	return char_traits<char>::not_eof(c);
}

void ThreadLocalEStream::perror(const std::string &msg) {
	thread_local char buf[4096];
	cerr << msg << ": " << strerror_r(errno, buf, sizeof(buf)) << endl;
}
