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

#ifndef fiv__WINDOW_HPP_
#define fiv__WINDOW_HPP_

#include <map>
#include <memory>
#include <string>

class Fiv;

class Window: public std::enable_shared_from_this<Window> {
public:
	Window(const std::string &title);
	virtual ~Window();
	void create();
	void destroy();
	virtual void display();
	virtual void keyboard(unsigned char key, int x, int y);
	virtual void closed();
	static int init(int argc, char *argv[]);
	static void mainLoop();

private:
	static void glutDisplay();
	static void glutKeyboard(unsigned char key, int x, int y);
	static void glutClose();

	static std::map<int,std::shared_ptr<Window>> windows;

	std::string title;
	int id;
};

#endif /* fiv__WINDOW_HPP_ */
