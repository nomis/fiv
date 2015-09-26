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

#include <glibmm/refptr.h>
#include <gdk/gdk.h>
#include <gtkmm/applicationwindow.h>
#include <gtkmm/gesturedrag.h>
#include <gtkmm/gesturezoom.h>
#include <memory>
#include <mutex>
#include <string>

#include "Events.hpp"
#include "Fiv.hpp"
#include "ImageDrawable.hpp"

class MainWindow: public Gtk::ApplicationWindow, public Events {
public:
	MainWindow(std::shared_ptr<Fiv> fiv_);
	virtual void addImage();
	virtual void loadedCurrent();

private:
	void action_edit_mark();
	void action_edit_toggleMark();
	void action_edit_unmark();
	void action_image_rotateLeft();
	void action_image_rotateRight();
	void action_image_flipHorizontal();
	void action_image_flipVertical();
	void action_view_previous();
	void action_view_next();
	void action_view_first();
	void action_view_last();
	void action_view_fullScreen();
	void redraw();
	void updateAll();
	void updateTitle();
	virtual bool on_window_state_event(GdkEventWindowState *event);

	std::mutex mtxUpdate;
	std::shared_ptr<Fiv> images;
	ImageDrawable drawImage;
	Glib::RefPtr<Gtk::GestureDrag> drag;
	Glib::RefPtr<Gtk::GestureZoom> zoom;
	bool fullScreen;
};

#endif /* fiv__MAINWINDOW_HPP_ */
