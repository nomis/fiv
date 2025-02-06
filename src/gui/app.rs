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
use gio::Menu;
use gtk::gio::SimpleAction;
use gtk::glib::once_cell::unsync::OnceCell;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct Application {
	app_name: OnceCell<String>,
	files: OnceCell<Arc<Files>>,
	window: OnceCell<gtk::ApplicationWindow>,
	state: Arc<Mutex<State>>,
}

#[derive(Debug, Default)]
struct State {
	full_screen: bool,
}

#[glib::object_subclass]
impl ObjectSubclass for Application {
	const NAME: &'static str = "Application";
	type Type = super::Application;
	type ParentType = gtk::Application;
}

impl ObjectImpl for Application {}

#[derive(strum::AsRefStr)]
#[strum(prefix = "app.")]
enum AppAction {
	Quit,
}

#[derive(strum::AsRefStr)]
#[strum(prefix = "win.")]
enum WinAction {
	ViewFullScreen,
}

trait ApplicationExtActionEnum {
	fn add_action_ext<F: Fn(&SimpleAction, Option<&glib::Variant>) + 'static>(
		&self,
		name: AppAction,
		f: F,
	);
}

impl ApplicationExtActionEnum for gtk::Application {
	fn add_action_ext<F: Fn(&SimpleAction, Option<&glib::Variant>) + 'static>(
		&self,
		name: AppAction,
		f: F,
	) {
		let action = SimpleAction::new(
			name.as_ref()
				.split_once('.')
				.expect("Enum str values are prefixed with \"app.\"")
				.1,
			None,
		);
		action.connect_activate(f);
		self.add_action(&action);
	}
}

trait ApplicationWindowExtActionEnum {
	fn add_action_ext<F: Fn(&SimpleAction, Option<&glib::Variant>) + 'static>(
		&self,
		name: WinAction,
		f: F,
	);
}

impl ApplicationWindowExtActionEnum for gtk::ApplicationWindow {
	fn add_action_ext<F: Fn(&SimpleAction, Option<&glib::Variant>) + 'static>(
		&self,
		name: WinAction,
		f: F,
	) {
		let action = SimpleAction::new(
			name.as_ref()
				.split_once('.')
				.expect("Enum str values are prefixed with \"win.\"")
				.1,
			None,
		);
		action.connect_activate(f);
		self.add_action(&action);
	}
}

impl Application {
	pub fn init(&self, files: Arc<Files>) {
		self.files.set(files).unwrap();

		let self_ref = self.downgrade();

		glib::MainContext::default().spawn_local(async move {
			if let Some(app) = self_ref.upgrade() {
				let files = app.files.get().unwrap();

				loop {
					files.ui_wait().await;
					app.refresh();
				}
			}
		});
	}

	fn build_menu(&self) {
		let obj = self.obj();
		let app = obj.dynamic_cast_ref::<gtk::Application>().unwrap();
		let window = self.window.get().unwrap();
		let menu_bar = Menu::new();
		let image_menu = Menu::new();

		image_menu.append(
			Some("F_ull Screen"),
			Some(WinAction::ViewFullScreen.as_ref()),
		);
		app.set_accels_for_action(WinAction::ViewFullScreen.as_ref(), &["F11"]);

		image_menu.append(Some("_Quit"), Some(AppAction::Quit.as_ref()));
		app.set_accels_for_action(AppAction::Quit.as_ref(), &["<Primary>q", "q", "<Alt>F4"]);

		menu_bar.append_submenu(Some("_Image"), &image_menu);

		app.set_menubar(Some(&menu_bar));

		let state_copy = self.state.clone();

		window.add_action_ext(
			WinAction::ViewFullScreen,
			glib::clone!(@weak window => move |_, _| {
				let mut state = state_copy.lock().unwrap();

				if state.full_screen {
					window.set_show_menubar(true);
					window.unfullscreen();
					state.full_screen = false;
				} else {
					window.fullscreen();
					window.set_show_menubar(false);
					state.full_screen = true;
				}
			}),
		);

		app.add_action_ext(
			AppAction::Quit,
			glib::clone!(@weak app => move |_, _| {
				app.quit();
			}),
		);
	}

	pub fn refresh(&self) {
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
			.set(
				gtk::ApplicationWindow::builder()
					.application(&*self.obj())
					.default_width(1920 / 2)
					.default_height(1080 / 2)
					.build(),
			)
			.unwrap();

		self.build_menu();
	}

	/// The command line is ignored here, see CommandLineArgs::parse()
	fn command_line(&self, _cmd: &gio::ApplicationCommandLine) -> glib::ExitCode {
		self.activate();
		glib::ExitCode::SUCCESS
	}

	fn activate(&self) {
		self.parent_activate();

		let window = self.window.get().unwrap();

		self.refresh();

		window.maximize();
		window.show_all();
		window.present();
	}
}

impl GtkApplicationImpl for Application {}
