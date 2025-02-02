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

mod app;

use super::Files;
use gio::ApplicationFlags;
use gtk::{gio, glib, subclass::prelude::*};
use std::sync::Arc;

glib::wrapper! {
	pub struct Application(ObjectSubclass<app::Application>)
		@extends gio::Application, gtk::Application;
}

impl Application {
	pub fn new(files: Arc<Files>) -> Self {
		let app: Application = glib::Object::builder()
			.property("application-id", "uk.uuid.fiv")
			.property(
				"flags",
				ApplicationFlags::HANDLES_COMMAND_LINE | ApplicationFlags::NON_UNIQUE,
			)
			.build();

		app.imp().set_files(files);
		app
	}
}
