/*
 * fiv - Fast Image Viewer
 * Copyright 2015,2018,2025  Simon Arlott
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

use super::Files;
use gtk::glib::once_cell::unsync::OnceCell;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct Application {
	app_name: OnceCell<String>,
	files: OnceCell<Arc<Files>>,
	window: OnceCell<gtk::ApplicationWindow>,
}

#[glib::object_subclass]
impl ObjectSubclass for Application {
	const NAME: &'static str = "Application";
	type Type = super::Application;
	type ParentType = gtk::Application;
}

impl ObjectImpl for Application {}

impl Application {
	pub fn set_files(&self, files: Arc<Files>) {
		self.files.set(files).unwrap();
	}

	pub fn update_title(&self) {
		let window = self.window.get().unwrap();
		let files = self.files.get().unwrap();
		let current = files.current();

		window.set_title(&format!(
			"{}: {} ({}/{}{})",
			self.app_name.get().unwrap(),
			current.filename.display(),
			current.position,
			current.total,
			if files.is_loading() { "+" } else { "" }
		));
	}
}

impl ApplicationImpl for Application {
	fn startup(&self) {
		self.parent_startup();

		self.app_name
			.set(String::from(glib::application_name().unwrap()))
			.unwrap();

		self.window
			.set(gtk::ApplicationWindow::new(&*self.obj()))
			.unwrap();
		let window = self.window.get().unwrap();

		window.set_default_size(1920 / 2, 1080 / 2);
	}

	/// The command line is ignored here, see CommandLineArgs::parse()
	fn command_line(&self, _cmd: &gio::ApplicationCommandLine) -> glib::ExitCode {
		self.activate();
		glib::ExitCode::SUCCESS
	}

	fn activate(&self) {
		self.parent_activate();

		let window = self.window.get().unwrap();

		self.update_title();

		window.maximize();
		window.show_all();
		window.present();
	}
}

impl GtkApplicationImpl for Application {}
