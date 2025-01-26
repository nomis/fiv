/*
 * fiv - Fast Image Viewer
 * Copyright 2025  Simon Arlott
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

mod imp;

use gio::ApplicationFlags;
use gtk::{gio, glib};

glib::wrapper! {
	pub struct Application(ObjectSubclass<imp::Application>)
		@extends gio::Application, gtk::Application;
}

impl Default for Application {
	fn default() -> Self {
		glib::Object::builder()
			.property("application-id", "uk.uuid.fiv")
			.property("flags", ApplicationFlags::HANDLES_COMMAND_LINE)
			.build()
	}
}
