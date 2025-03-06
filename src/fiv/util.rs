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

pub mod exiv2_byte_order;
pub mod numeric;

use std::sync::{Condvar, Mutex};

#[derive(Debug)]
pub struct Waitable<T> {
	value: Mutex<T>,
	cv: Condvar,
}

impl<T: Copy + PartialEq<T>> Waitable<T> {
	pub fn new(value: T) -> Self {
		Self {
			value: Mutex::new(value),
			cv: Condvar::new(),
		}
	}

	pub fn get(&self) -> T {
		*self.value.lock().unwrap()
	}

	pub fn set(&self, value: T) {
		let mut value_mg = self.value.lock().unwrap();

		*value_mg = value;
		self.cv.notify_all();
	}

	pub fn wait(&self, expect: &T) {
		let mut actual = self.value.lock().unwrap();

		while &*actual != expect {
			actual = self.cv.wait(actual).unwrap();
		}
	}
}
