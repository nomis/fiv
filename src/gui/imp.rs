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

use gtk::{gio, glib, prelude::*, subclass::prelude::*};

#[derive(Debug, Default)]
pub struct Application {}

#[glib::object_subclass]
impl ObjectSubclass for Application {
	const NAME: &'static str = "Application";
	type Type = super::Application;
	type ParentType = gtk::Application;
}

impl ObjectImpl for Application {}

impl ApplicationImpl for Application {
	#[allow(clippy::needless_return)]
	fn command_line(&self, _cmd: &gio::ApplicationCommandLine) -> glib::ExitCode {
		self.activate();
		return glib::ExitCode::SUCCESS;
	}

	fn activate(&self) {
		self.parent_activate();

		let window = gtk::ApplicationWindow::new(&*self.obj());

		window.set_default_size(1920 / 2, 1080 / 2);
		window.maximize();
		window.show_all();
		window.present();
	}
}

impl GtkApplicationImpl for Application {}
