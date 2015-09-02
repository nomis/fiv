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
	id = 0;
}

Window::~Window() {
	if (id)
		glutDestroyWindow(id);
}

bool Window::init(int argc, char *argv[]) {
	glutInit(&argc, argv);
	glutSetOption(GLUT_ACTION_ON_WINDOW_CLOSE, GLUT_ACTION_GLUTMAINLOOP_RETURNS);
	glClearColor(0.0f, 0.0f, 0.0f, 0.0f);
	glEnable(GL_TEXTURE_2D);
	return true;
}

void Window::glutDisplay() {
	int id = glutGetWindow();

	if (id)
		windows.at(id)->display();
}

void Window::glutKeyboard(unsigned char key, int x, int y) {
	int id = glutGetWindow();

	if (id)
		windows.at(id)->keyboard(key, x, y);
}

void Window::glutClose() {
	int id = glutGetWindow();

	if (id)
		windows.at(id)->closed();
}

void Window::mainLoop() {
	glutMainLoop();
}

void Window::create() {
	glutInitDisplayMode(GLUT_DOUBLE | GLUT_RGBA);
	glutInitWindowSize(1536, 1024);
	// TODO maximise window
	id = glutCreateWindow(title.c_str());
	windows[id]=shared_from_this();
	glutDisplayFunc(&Window::glutDisplay);
	glutKeyboardFunc(&Window::glutKeyboard);
	glutCloseFunc(&Window::glutClose);
}

void Window::destroy() {
	if (id) {
		glutDestroyWindow(id);
		id = 0;
	}
}

void Window::display() {
	glClear(GL_COLOR_BUFFER_BIT);
	glutSwapBuffers();
}

void Window::keyboard(unsigned char key __attribute__((unused)), int x __attribute__((unused)), int y __attribute__((unused))) {

}

void Window::closed() {
	if (id) {
		windows.erase(id);
		id = 0;
	}
}
