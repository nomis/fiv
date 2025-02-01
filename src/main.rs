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

mod fiv;
mod gui;

use crate::fiv::{CommandLineArgs, Files};
use clap::Parser;
use gtk::{glib, prelude::*};
use std::sync::Arc;

fn main() -> glib::ExitCode {
	let args = Arc::new(CommandLineArgs::parse());
	let pool = Arc::new(rayon::ThreadPoolBuilder::new().build().unwrap());
	let files = Files::new(pool, args);

	if files.start() {
		gui::Application::default().run()
	} else {
		glib::ExitCode::FAILURE
	}
}
