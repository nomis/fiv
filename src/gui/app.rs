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
use crate::fiv::{Mark, Navigate};
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

#[derive(Copy, Clone, Debug, strum::AsRefStr)]
#[strum(prefix = "app.")]
enum AppAction {
	Quit,
}

#[derive(Copy, Clone, Debug, strum::AsRefStr)]
#[strum(prefix = "win.")]
enum WinAction {
	EditMark,
	EditToggleMark,
	EditUnmark,
	ViewFirst,
	ViewPrevious,
	ViewNext,
	ViewLast,
	ViewFullScreen,
}

trait MenuExtActionEnum<T> {
	fn append_ext(&self, label: &str, action: T);
}

impl MenuExtActionEnum<AppAction> for Menu {
	fn append_ext(&self, label: &str, action: AppAction) {
		self.append(Some(label), Some(action.as_ref()));
	}
}

impl MenuExtActionEnum<WinAction> for Menu {
	fn append_ext(&self, label: &str, action: WinAction) {
		self.append(Some(label), Some(action.as_ref()));
	}
}

trait ApplicationAction<T> {
	fn add_action(&self, name: T, func: fn(&Self, T), accels: &[&str]);
}

impl ApplicationAction<AppAction> for Application {
	fn add_action(&self, name: AppAction, func: fn(&Self, AppAction), accels: &[&str]) {
		let action = SimpleAction::new(
			name.as_ref()
				.split_once('.')
				.expect("Enum str values are prefixed with \"app.\"")
				.1,
			None,
		);

		let self_ref = self.downgrade();
		action.connect_activate(move |_, _| {
			if let Some(app) = self_ref.upgrade() {
				func(&app, name);
			}
		});

		let obj = self.obj();
		let app = obj.dynamic_cast_ref::<gtk::Application>().unwrap();
		app.set_accels_for_action(name.as_ref(), accels);
		app.add_action(&action);
	}
}

impl ApplicationAction<WinAction> for Application {
	fn add_action(&self, name: WinAction, func: fn(&Self, WinAction), accels: &[&str]) {
		let action = SimpleAction::new(
			name.as_ref()
				.split_once('.')
				.expect("Enum str values are prefixed with \"win.\"")
				.1,
			None,
		);

		let self_ref = self.downgrade();
		action.connect_activate(move |_, _| {
			if let Some(app) = self_ref.upgrade() {
				func(&app, name);
			}
		});

		let obj = self.obj();
		let app = obj.dynamic_cast_ref::<gtk::Application>().unwrap();
		app.set_accels_for_action(name.as_ref(), accels);

		let window = self.window.get().unwrap();
		window.add_action(&action);
	}
}

impl Application {
	pub fn init(&self, files: Arc<Files>) {
		self.files.set(files).unwrap();

		let self_ref = self.downgrade();

		glib::MainContext::default().spawn_local(async move {
			if let Some(app) = self_ref.upgrade() {
				app.process_events().await;
			}
		});
	}

	async fn process_events(&self) {
		let files = self.files.get().unwrap();

		loop {
			files.ui_wait().await;
			self.refresh();
		}
	}

	fn build_menu(&self) {
		let obj = self.obj();
		let app = obj.dynamic_cast_ref::<gtk::Application>().unwrap();
		let files = self.files.get().unwrap();
		let menu_bar = Menu::new();
		let image_menu = Menu::new();
		let image_menu_app = Menu::new();
		let edit_menu = Menu::new();
		let edit_menu_mark = Menu::new();
		let view_menu = Menu::new();
		let view_menu_nav = Menu::new();
		let view_menu_win = Menu::new();

		image_menu_app.append_ext("_Quit", AppAction::Quit);
		self.add_action(AppAction::Quit, Self::quit, &["<Primary>q", "q", "<Alt>F4"]);
		image_menu.append_section(None, &image_menu_app);
		menu_bar.append_submenu(Some("_Image"), &image_menu);

		if files.mark_supported() {
			edit_menu_mark.append_ext("_Mark", WinAction::EditMark);
			self.add_action(WinAction::EditMark, Self::files_action, &["Insert"]);
			edit_menu_mark.append_ext("_Toggle mark", WinAction::EditToggleMark);
			self.add_action(WinAction::EditToggleMark, Self::files_action, &["Tab"]);
			edit_menu_mark.append_ext("_Unmark", WinAction::EditUnmark);
			self.add_action(WinAction::EditUnmark, Self::files_action, &["Delete"]);
			edit_menu.append_section(None, &edit_menu_mark);
			menu_bar.append_submenu(Some("_Edit"), &edit_menu);
		}

		view_menu_nav.append_ext("_Previous", WinAction::ViewPrevious);
		self.add_action(WinAction::ViewPrevious, Self::files_action, &["Left"]);
		view_menu_nav.append_ext("_Next", WinAction::ViewNext);
		self.add_action(
			WinAction::ViewNext,
			Self::files_action,
			&["Right", "Return"],
		);
		view_menu_nav.append_ext("_First", WinAction::ViewFirst);
		self.add_action(WinAction::ViewFirst, Self::files_action, &["Home"]);
		view_menu_nav.append_ext("_Last", WinAction::ViewLast);
		self.add_action(WinAction::ViewLast, Self::files_action, &["End"]);
		view_menu.append_section(None, &view_menu_nav);

		view_menu_win.append_ext("F_ull Screen", WinAction::ViewFullScreen);
		self.add_action(WinAction::ViewFullScreen, Self::view_fullscreen, &["F11"]);
		view_menu.append_section(None, &view_menu_win);
		menu_bar.append_submenu(Some("_View"), &view_menu);

		app.set_menubar(Some(&menu_bar));
	}

	pub fn refresh(&self) {
		let window = self.window.get().unwrap();
		let files = self.files.get().unwrap();
		let current = files.current();

		window.set_title(&format!(
			"{}: {}{} ({}/{}{})",
			self.app_name.get().unwrap(),
			current.filename.display(),
			if files.mark_supported() {
				match current.mark {
					Some(true) => " ☑",
					Some(false) => " ☐",
					None => " ◌",
				}
			} else {
				""
			},
			current.position,
			current.total,
			if files.is_loading() { "+" } else { "" }
		));
	}

	fn files_action(&self, action: WinAction) {
		let files = self.files.get().unwrap();

		match action {
			WinAction::EditMark => files.mark(Mark::Set),
			WinAction::EditToggleMark => files.mark(Mark::Toggle),
			WinAction::EditUnmark => files.mark(Mark::Unset),
			WinAction::ViewFirst => files.navigate(Navigate::First),
			WinAction::ViewPrevious => files.navigate(Navigate::Previous),
			WinAction::ViewNext => files.navigate(Navigate::Next),
			WinAction::ViewLast => files.navigate(Navigate::Last),
			_ => {}
		}
	}

	fn view_fullscreen(&self, _action: WinAction) {
		let window = self.window.get().unwrap();
		let mut state = self.state.lock().unwrap();

		if state.full_screen {
			window.set_show_menubar(true);
			window.unfullscreen();
			state.full_screen = false;
		} else {
			window.fullscreen();
			window.set_show_menubar(false);
			state.full_screen = true;
		}
	}

	fn quit(&self, _action: AppAction) {
		let obj = self.obj();
		let app = obj.dynamic_cast_ref::<gtk::Application>().unwrap();
		app.quit();
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
