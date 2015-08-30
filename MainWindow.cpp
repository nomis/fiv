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

#include "MainWindow.hpp"

#include <GL/freeglut_std.h>
#include <iostream>
#include <memory>

#include "Fiv.hpp"

using namespace std;

MainWindow::MainWindow(shared_ptr<Fiv> fiv_) : Window("fiv") {
	fiv = fiv_;
}

void MainWindow::display() {
	glClear(GL_COLOR_BUFFER_BIT);
	glutSwapBuffers();
}

void MainWindow::keyboard(unsigned char key, int x __attribute__((unused)), int y __attribute__((unused))) {
	switch (key) {
	case 'q':
	case 'Q':
	case 17:
		destroy();
		break;
	}

}
