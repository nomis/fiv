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

#ifndef fiv__MAINWINDOW_HPP_
#define fiv__MAINWINDOW_HPP_

#include <memory>
#include <gtkmm.h>

#include "Fiv.hpp"

class MainWindow: public Gtk::ApplicationWindow {
public:
	MainWindow(std::shared_ptr<Fiv> fiv_);

private:
	const std::string title;
	std::shared_ptr<Fiv> fiv;
	std::shared_ptr<Fiv::Images> images;
};

#endif /* fiv__MAINWINDOW_HPP_ */
