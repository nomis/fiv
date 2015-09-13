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
#include "ImageDrawable.hpp"

using namespace std;

MainWindow::MainWindow(shared_ptr<Fiv> fiv_) : Gtk::ApplicationWindow(), title(Fiv::appName) {
	fiv = fiv_;
	images = fiv->getImages();

	set_default_size(1920/2, 1080/2);
	add_action("view.first", sigc::mem_fun(this, &MainWindow::action_view_first));
	add_action("view.previous", sigc::mem_fun(this, &MainWindow::action_view_previous));
	add_action("view.next", sigc::mem_fun(this, &MainWindow::action_view_next));
	add_action("view.last", sigc::mem_fun(this, &MainWindow::action_view_last));
	add_action("edit.rotateLeft", sigc::mem_fun(this, &MainWindow::action_edit_rotateLeft));
	add_action("edit.rotateRight", sigc::mem_fun(this, &MainWindow::action_edit_rotateRight));
	add_action("edit.flipHorizontal", sigc::mem_fun(this, &MainWindow::action_edit_flipHorizontal));
	add_action("edit.flipVertical", sigc::mem_fun(this, &MainWindow::action_edit_flipVertical));

	drawImage.setImages(images);
	add(drawImage);

	update();
}

void MainWindow::action_view_previous() {
	if (images->previous())
		update();
}

void MainWindow::action_view_next() {
	if (images->next()) {
		update();
	}
}

void MainWindow::action_view_first() {
	if (images->first())
		update();
}

void MainWindow::action_view_last() {
	if (images->last())
		update();
}

void MainWindow::action_edit_rotateLeft() {
	images->orientation(Image::Orientation(Image::Rotate::ROTATE_270, false));
	update();
}

void MainWindow::action_edit_rotateRight() {
	images->orientation(Image::Orientation(Image::Rotate::ROTATE_90, false));
	update();
}

void MainWindow::action_edit_flipHorizontal() {
	images->orientation(Image::Orientation(Image::Rotate::ROTATE_NONE, true));
	update();
}

void MainWindow::action_edit_flipVertical() {
	images->orientation(Image::Orientation(Image::Rotate::ROTATE_180, true));
	update();
}

void MainWindow::update() {
	if (drawImage.is_visible()) {
		drawImage.update();
	} else {
		drawImage.show();
	}
	set_title(title + ": " + images->current()->name);
}
