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
use super::draw::DrawingArea;
use crate::fiv::{Mark, Navigate, Rotate};
use gtk::gio::{Menu, SimpleAction};
use gtk::glib::Variant;
use gtk::glib::once_cell::unsync::OnceCell;
use gtk::{gdk, gio, glib, prelude::*, subclass::prelude::*};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct Application {
	app_name: OnceCell<String>,
	files: OnceCell<Arc<Files>>,
	window: OnceCell<gtk::ApplicationWindow>,
	state: Mutex<State>,
	drawing_area: OnceCell<Rc<DrawingArea>>,
	view_full_screen_action: OnceCell<SimpleAction>,
}

#[derive(Debug, Default)]
struct State {
	full_screen: bool,
	af_points: bool,
}

#[glib::object_subclass]
impl ObjectSubclass for Application {
	const NAME: &'static str = "Application";
	type Type = super::Application;
	type ParentType = gtk::Application;
}

impl ObjectImpl for Application {}

#[derive(Debug, Copy, Clone, strum::AsRefStr)]
#[strum(prefix = "app.")]
enum AppAction {
	Quit,
}

#[derive(Debug, Copy, Clone, strum::AsRefStr)]
#[strum(prefix = "win.")]
enum WinAction {
	ImageRotateLeft,
	ImageRotateRight,
	ImageFlipHorizontal,
	ImageFlipVertical,
	EditMark,
	EditToggleMark,
	EditUnmark,
	ViewFirst,
	ViewPrevious,
	ViewNext,
	ViewLast,
	ViewZoomActual,
	ViewZoomFit,
	ViewFullScreen,
	ViewAFPoints,
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

	fn add_stateful_action(
		&self,
		name: T,
		func: fn(&Self, &SimpleAction, Option<&Variant>),
		accels: &[&str],
		state: bool,
	) -> SimpleAction;
}

impl ApplicationAction<AppAction> for Application {
	fn add_action(&self, name: AppAction, func: fn(&Self, AppAction), accels: &[&str]) {
		let self_ref = self.downgrade();
		let action = self.new_action(name.as_ref(), accels, None, move |_, _| {
			if let Some(app) = self_ref.upgrade() {
				func(&app, name);
			}
		});

		let obj = self.obj();
		let app = obj.dynamic_cast_ref::<gtk::Application>().unwrap();
		app.add_action(&action);
	}

	fn add_stateful_action(
		&self,
		name: AppAction,
		func: fn(&Self, &SimpleAction, Option<&Variant>),
		accels: &[&str],
		state: bool,
	) -> SimpleAction {
		let self_ref = self.downgrade();
		let action = self.new_action(name.as_ref(), accels, Some(state), move |action, value| {
			if let Some(app) = self_ref.upgrade() {
				func(&app, action, value);
			}
		});

		let obj = self.obj();
		let app = obj.dynamic_cast_ref::<gtk::Application>().unwrap();
		app.add_action(&action);

		action
	}
}

impl ApplicationAction<WinAction> for Application {
	fn add_action(&self, name: WinAction, func: fn(&Self, WinAction), accels: &[&str]) {
		let self_ref = self.downgrade();
		let action = self.new_action(name.as_ref(), accels, None, move |_, _| {
			if let Some(app) = self_ref.upgrade() {
				func(&app, name);
			}
		});

		let window = self.window.get().unwrap();
		window.add_action(&action);
	}

	fn add_stateful_action(
		&self,
		name: WinAction,
		func: fn(&Self, &SimpleAction, Option<&Variant>),
		accels: &[&str],
		state: bool,
	) -> SimpleAction {
		let self_ref = self.downgrade();
		let action = self.new_action(name.as_ref(), accels, Some(state), move |action, value| {
			if let Some(app) = self_ref.upgrade() {
				func(&app, action, value);
			}
		});

		let window = self.window.get().unwrap();
		window.add_action(&action);

		action
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

		while files.ui_wait().await {
			self.refresh();
		}
	}

	fn new_action<F: Fn(&SimpleAction, Option<&glib::Variant>) + 'static>(
		&self,
		name: &str,
		accels: &[&str],
		state: Option<bool>,
		func: F,
	) -> SimpleAction {
		let short_name = name
			.split_once('.')
			.expect("Enum str values are prefixed with \"app.\" or \"win.\"")
			.1;
		let action = match state {
			Some(value) => SimpleAction::new_stateful(short_name, None, &value.to_variant()),
			None => SimpleAction::new(short_name, None),
		};

		match state {
			Some(_) => action.connect_change_state(func),
			None => action.connect_activate(func),
		};

		let obj = self.obj();
		let app = obj.dynamic_cast_ref::<gtk::Application>().unwrap();
		app.set_accels_for_action(name, accels);

		action
	}

	fn build_menu_bar(&self) {
		let obj = self.obj();
		let app = obj.dynamic_cast_ref::<gtk::Application>().unwrap();
		let files = self.files.get().unwrap();
		let menu_bar = Menu::new();

		menu_bar.append_submenu(Some("_Image"), &self.build_image_menu());
		if files.mark_supported() {
			menu_bar.append_submenu(Some("_Edit"), &self.build_edit_menu());
		}
		menu_bar.append_submenu(Some("_View"), &self.build_view_menu());

		app.set_menubar(Some(&menu_bar));
	}

	fn build_image_menu(&self) -> Menu {
		let menu = Menu::new();
		let rotate_section = Menu::new();
		let flip_section = Menu::new();
		let app_section = Menu::new();

		rotate_section.append_ext("Rotate _Left", WinAction::ImageRotateLeft);
		self.add_action(WinAction::ImageRotateLeft, Self::files_action, &["l"]);
		rotate_section.append_ext("Rotate _Right", WinAction::ImageRotateRight);
		self.add_action(WinAction::ImageRotateRight, Self::files_action, &["r"]);
		menu.append_section(None, &rotate_section);

		flip_section.append_ext("Flip _Horizontal", WinAction::ImageFlipHorizontal);
		self.add_action(WinAction::ImageFlipHorizontal, Self::files_action, &["h"]);
		flip_section.append_ext("Flip _Vertical", WinAction::ImageFlipVertical);
		self.add_action(WinAction::ImageFlipVertical, Self::files_action, &["v"]);
		menu.append_section(None, &flip_section);

		app_section.append_ext("_Quit", AppAction::Quit);
		self.add_action(AppAction::Quit, Self::quit, &["<Primary>q", "q", "<Alt>F4"]);
		menu.append_section(None, &app_section);

		menu
	}

	fn build_edit_menu(&self) -> Menu {
		let menu = Menu::new();
		let mark_section = Menu::new();

		mark_section.append_ext("_Mark", WinAction::EditMark);
		self.add_action(WinAction::EditMark, Self::files_action, &["Insert"]);
		mark_section.append_ext("_Toggle mark", WinAction::EditToggleMark);
		self.add_action(WinAction::EditToggleMark, Self::files_action, &["Tab"]);
		mark_section.append_ext("_Unmark", WinAction::EditUnmark);
		self.add_action(WinAction::EditUnmark, Self::files_action, &["Delete"]);
		menu.append_section(None, &mark_section);

		menu
	}

	fn build_view_menu(&self) -> Menu {
		let menu = Menu::new();
		let zoom_section = Menu::new();
		let nav_section = Menu::new();
		let win_section = Menu::new();
		let overlay_section = Menu::new();

		nav_section.append_ext("_Previous", WinAction::ViewPrevious);
		self.add_action(WinAction::ViewPrevious, Self::files_action, &["Left"]);
		nav_section.append_ext("_Next", WinAction::ViewNext);
		self.add_action(
			WinAction::ViewNext,
			Self::files_action,
			&["Right", "Return"],
		);
		nav_section.append_ext("_First", WinAction::ViewFirst);
		self.add_action(WinAction::ViewFirst, Self::files_action, &["Home"]);
		nav_section.append_ext("_Last", WinAction::ViewLast);
		self.add_action(WinAction::ViewLast, Self::files_action, &["End"]);
		menu.append_section(None, &nav_section);

		zoom_section.append_ext("Norm_al Size", WinAction::ViewZoomActual);
		self.add_action(WinAction::ViewZoomActual, Self::zoom_action, &["a", "1"]);
		zoom_section.append_ext("Best _Fit", WinAction::ViewZoomFit);
		self.add_action(WinAction::ViewZoomFit, Self::zoom_action, &["f"]);
		menu.append_section(None, &zoom_section);

		win_section.append_ext("F_ull Screen", WinAction::ViewFullScreen);
		self.view_full_screen_action
			.set(self.add_stateful_action(
				WinAction::ViewFullScreen,
				Self::view_fullscreen,
				&["F11"],
				false,
			))
			.unwrap();
		menu.append_section(None, &win_section);

		overlay_section.append_ext("AF P_oints", WinAction::ViewAFPoints);
		self.add_stateful_action(WinAction::ViewAFPoints, Self::view_af_points, &["p"], false);
		menu.append_section(None, &overlay_section);

		menu
	}

	pub fn refresh(&self) {
		let window = self.window.get().unwrap();
		let drawing_area = self.drawing_area.get().unwrap();
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
			if files.starting() { "+" } else { "" }
		));

		if let Some(image) = current.image {
			drawing_area.refresh(image);
		}
	}

	fn files_action(&self, action: WinAction) {
		let files = self.files.get().unwrap();

		match action {
			WinAction::ImageRotateLeft => files.orientation(Rotate::Rotate270, false),
			WinAction::ImageRotateRight => files.orientation(Rotate::Rotate90, false),
			WinAction::ImageFlipHorizontal => files.orientation(Rotate::Rotate0, true),
			WinAction::ImageFlipVertical => files.orientation(Rotate::Rotate180, true),
			WinAction::EditMark => files.mark(Mark::Set),
			WinAction::EditToggleMark => files.mark(Mark::Toggle),
			WinAction::EditUnmark => files.mark(Mark::Unset),
			WinAction::ViewFirst => files.navigate(Navigate::First),
			WinAction::ViewPrevious => files.navigate(Navigate::Previous),
			WinAction::ViewNext => files.navigate(Navigate::Next),
			WinAction::ViewLast => files.navigate(Navigate::Last),
			_ => (),
		}
	}

	fn zoom_action(&self, action: WinAction) {
		let drawing_area = self.drawing_area.get().unwrap();

		match action {
			WinAction::ViewZoomActual => drawing_area.zoom_actual(),
			WinAction::ViewZoomFit => drawing_area.zoom_fit(),
			_ => (),
		}
	}

	fn view_fullscreen(&self, _action: &SimpleAction, value: Option<&Variant>) {
		let window = self.window.get().unwrap();

		if let Some(value) = value {
			if value.get().unwrap() {
				window.fullscreen();
			} else {
				window.unfullscreen();
			}
		}
	}

	fn view_af_points(&self, action: &SimpleAction, value: Option<&Variant>) {
		let drawing_area = self.drawing_area.get().unwrap();
		let mut state = self.state.lock().unwrap();

		if let Some(value) = value {
			state.af_points = value.get().unwrap();
			action.set_state(value);
			drawing_area.af_points(state.af_points);
		}
	}

	fn window_state_changed(&self, full_screen: bool) {
		let mut state = self.state.lock().unwrap();

		if state.full_screen != full_screen {
			let window = self.window.get().unwrap();

			state.full_screen = full_screen;
			self.view_full_screen_action
				.get()
				.unwrap()
				.set_state(&state.full_screen.to_variant());
			window.set_show_menubar(!state.full_screen);
		}
	}

	fn quit(&self, _action: AppAction) {
		let obj = self.obj();
		let app = obj.dynamic_cast_ref::<gtk::Application>().unwrap();
		let window = self.window.get().unwrap();

		window.hide();
		app.remove_window(window);
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

		self.build_menu_bar();

		let window = self.window.get().unwrap();

		let self_ref = self.downgrade();

		window.connect_window_state_event(move |_, event| -> glib::Propagation {
			if let Some(app) = self_ref.upgrade() {
				let full_screen = event
					.new_window_state()
					.contains(gdk::WindowState::FULLSCREEN);

				app.window_state_changed(full_screen);
			}

			glib::Propagation::Proceed
		});

		self.drawing_area
			.set(DrawingArea::new(
				self.files.get().unwrap().begin(),
				|widget| window.add(widget),
			))
			.unwrap();
	}

	/// The command line is ignored here, see `CommandLineArgs::parse()`
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

	fn shutdown(&self) {
		self.parent_shutdown();
		self.files.get().unwrap().shutdown();
	}
}

impl GtkApplicationImpl for Application {}
