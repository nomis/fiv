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

#![warn(clippy::pedantic)]
#![warn(clippy::shadow_unrelated)]
mod fiv;
mod gui;

use crate::fiv::{CommandLineArgs, Files};
use clap::Parser;
use gtk::{glib, prelude::*};
use std::{sync::Arc, time::Instant};

fn main() -> glib::ExitCode {
	let startup = Instant::now();
	let args = Arc::new(CommandLineArgs::parse());

	stderrlog::new()
		.module(module_path!())
		.verbosity(usize::from(args.verbose) + 2)
		.init()
		.unwrap();

	let files = Files::new(&args, startup);

	if files.start() {
		let exit_code = gui::Application::new(files.clone()).run_with_args::<&str>(&[]);
		files.join();
		exit_code
	} else {
		glib::ExitCode::FAILURE
	}
}
