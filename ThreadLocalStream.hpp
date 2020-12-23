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

#ifndef fiv__THREADLOCALSTREAM_HPP_
#define fiv__THREADLOCALSTREAM_HPP_

#include <mutex>
#include <ostream>
#include <streambuf>

class ThreadLocalOStream : public std::basic_streambuf<char, std::char_traits<char>> {
public:
	~ThreadLocalOStream() = default;

	static ThreadLocalOStream instance;

private:
	ThreadLocalOStream();
	ThreadLocalOStream& operator=(const ThreadLocalOStream&);
	ThreadLocalOStream(const ThreadLocalOStream&);
	int_type overflow(int_type c = traits_type::eof()) override;

	static thread_local std::ostringstream buffer;

	std::mutex mtxOutput;
	std::ostream output;
};

class ThreadLocalEStream : public std::basic_streambuf<char, std::char_traits<char>> {
public:
	~ThreadLocalEStream() = default;
	static void perror(const std::string &msg);

	static ThreadLocalEStream instance;

private:
	ThreadLocalEStream();
	ThreadLocalEStream& operator=(const ThreadLocalEStream&);
	ThreadLocalEStream(const ThreadLocalEStream&);
	int_type overflow(int_type c = traits_type::eof()) override;

	static thread_local std::ostringstream buffer;

	std::mutex mtxOutput;
	std::ostream output;
};

#endif /* fiv__THREADLOCALSTREAM_HPP_ */
