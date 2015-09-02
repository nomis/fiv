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

#include "Window.hpp"

#include <GL/freeglut_std.h>
#include <GL/freeglut_ext.h>
#include <GL/gl.h>
#include <cstdlib>
#include <map>
#include <memory>
#include <string>

using namespace std;
using std::remove;

map<int,shared_ptr<Window>> Window::windows;

Window::Window(const string &title_) {
	title = title_;
}

Window::~Window() {

}

bool Window::init() {
	return true;
}

void Window::mainLoop() {

}

void Window::create() {

}

void Window::destroy() {

}

void Window::display() {

}

void Window::keyboard(unsigned char key __attribute__((unused)), int x __attribute__((unused)), int y __attribute__((unused))) {

}

void Window::closed() {

}
