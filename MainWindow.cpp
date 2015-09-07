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

#include <sigc++/functors/mem_fun.h>
#include <memory>
#include <string>

#include "Fiv.hpp"

using namespace std;

MainWindow::MainWindow(shared_ptr<Fiv> fiv_) : Gtk::ApplicationWindow(), title(Fiv::appName) {
	fiv = fiv_;
	images = fiv->getImages();

	set_title(title);
	set_default_size(1920/2, 1080/2);
}
