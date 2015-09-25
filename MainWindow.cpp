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

//#include <sigc++/functors/mem_fun.h>
//#include <memory>
#include <string>
#include <utility>

#include "Fiv.hpp"
#include "Image.hpp"
#include "ImageDrawable.hpp"

using namespace std;

MainWindow::MainWindow(shared_ptr<Fiv> fiv) : Gtk::ApplicationWindow(), title(Fiv::appName) {
	images = fiv;
	fullScreen = false;

	set_default_size(1920/2, 1080/2);

	drawImage.setImages(images);
	add(drawImage);

	add_action("edit.rotateLeft", sigc::mem_fun(this, &MainWindow::action_edit_rotateLeft));
	add_action("edit.rotateRight", sigc::mem_fun(this, &MainWindow::action_edit_rotateRight));
	add_action("edit.flipHorizontal", sigc::mem_fun(this, &MainWindow::action_edit_flipHorizontal));
	add_action("edit.flipVertical", sigc::mem_fun(this, &MainWindow::action_edit_flipVertical));
	add_action("view.first", sigc::mem_fun(this, &MainWindow::action_view_first));
	add_action("view.previous", sigc::mem_fun(this, &MainWindow::action_view_previous));
	add_action("view.next", sigc::mem_fun(this, &MainWindow::action_view_next));
	add_action("view.last", sigc::mem_fun(this, &MainWindow::action_view_last));
	add_action("view.zoomActual", sigc::mem_fun(drawImage, &ImageDrawable::zoomActual));
	add_action("view.zoomFit", sigc::mem_fun(drawImage, &ImageDrawable::zoomFit));
	add_action("view.fullScreen", sigc::mem_fun(this, &MainWindow::action_view_fullScreen));

	update();
}

void MainWindow::loadedCurrent() {
	drawImage.loaded();
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

void MainWindow::action_view_previous() {
	if (images->previous())
		update();
}

void MainWindow::action_view_next() {
	if (images->next())
		update();
}

void MainWindow::action_view_first() {
	if (images->first())
		update();
}

void MainWindow::action_view_last() {
	if (images->last())
		update();
}

void MainWindow::action_view_fullScreen() {
	if (fullScreen) {
		unfullscreen();
	} else {
		fullscreen();
	}
	redraw();
}

void MainWindow::redraw() {
	drawImage.redraw();
}

void MainWindow::update() {
	drawImage.update();
	set_title(title + ": " + images->current()->name);
}

bool MainWindow::on_window_state_event(GdkEventWindowState *event) {
	fullScreen = (event->new_window_state & GDK_WINDOW_STATE_FULLSCREEN) != 0;
	return false;
}

