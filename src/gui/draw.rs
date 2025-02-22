/*
 * fiv - Fast Image Viewer
 * Copyright 2015,2020,2025  Simon Arlott
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

use crate::fiv::{Image, Orientation, Rotate};
use gtk::{cairo, glib, prelude::*};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct Draw {
	area: gtk::DrawingArea,
	image_draw: Arc<Mutex<ImageDraw>>,
}

#[derive(Debug, Default)]
struct ImageDraw {
	image: Option<Arc<Image>>,
	waiting: bool,
	position: Position,
	orientation: Orientation,
}

#[derive(Debug)]
pub struct Position {
	x: f64,
	y: f64,
	zoom: f64,
	drag_offset_x: f64,
	drag_offset_y: f64,
}

#[derive(Debug)]
struct Render {
	x: f64,
	y: f64,
	scale: f64,
	width: f64,
	height: f64,
}

impl Default for Position {
	fn default() -> Self {
		Self {
			x: 0.0,
			y: 0.0,
			zoom: f64::NAN,
			drag_offset_x: 0.0,
			drag_offset_y: 0.0,
		}
	}
}

impl Draw {
	pub fn new<F: FnOnce(&gtk::DrawingArea)>(f: F) -> Self {
		let draw = Self::default();
		let draw_image_ref = Arc::downgrade(&draw.image_draw);

		draw.area
			.connect_draw(move |area, context| -> glib::Propagation {
				if let Some(draw_image) = draw_image_ref.upgrade() {
					draw_image.lock().unwrap().draw(&area.allocation(), context);
				}

				glib::Propagation::Proceed
			});
		f(&draw.area);
		draw
	}

	pub fn refresh(&self, image: Arc<Image>) {
		if self.image_draw.lock().unwrap().refresh(image) {
			self.redraw();
		}
	}

	pub fn zoom_actual(&self) {
		let mut image_draw = self.image_draw.lock().unwrap();

		if image_draw.zoom_actual(&self.area.allocation(), self.pointer()) {
			self.redraw();
		}
	}

	pub fn zoom_fit(&self) {
		if self.image_draw.lock().unwrap().zoom_fit() {
			self.redraw();
		}
	}

	fn redraw(&self) {
		if self.area.is_visible() {
			self.area.queue_draw();
		} else {
			self.area.show();
		}
	}

	fn pointer(&self) -> (i32, i32) {
		let window = self.area.window().unwrap();
		let seat = self.area.display().default_seat().unwrap();
		let position = window.device_position(&seat.pointer().unwrap());

		// Co-ordinates are relative to the `gdk::Window` (excluding the
		// menu bar) and should be adjusted for the `gtk::DrawingArea`
		// position but that includes the menu bar!
		//
		// Do nothing because the drawing area is currently at the top
		// left of the window.
		(position.1, position.2)
	}
}

impl ImageDraw {
	pub fn refresh(&mut self, image: Arc<Image>) -> bool {
		let changed = match &self.image {
			Some(other) => !Arc::ptr_eq(&image, other),
			None => true,
		};

		if changed {
			self.orientation = image.orientation();
			self.position = Position::default();
			self.image = Some(image);

			true
		} else {
			self.waiting && image.loaded() || self.orientation != image.orientation()
		}
	}

	pub fn zoom_actual(&mut self, allocation: &gtk::Allocation, pointer: (i32, i32)) -> bool {
		self.zoom(allocation, pointer, f64::NAN);
		self.image.is_some()
	}

	pub fn zoom_fit(&mut self) -> bool {
		self.position.zoom = f64::NAN;
		self.image.is_some()
	}

	fn zoom(&mut self, allocation: &gtk::Allocation, pointer: (i32, i32), scale: f64) {
		if let Some(image) = &self.image {
			let (pointer_x, pointer_y) = (f64::from(pointer.0), f64::from(pointer.1));
			let render = self.calc_render(allocation, image);

			self.position.zoom = if f64::is_nan(scale) {
				1.0
			} else {
				render.scale * scale
			};

			self.position.x = pointer_x
				- ((pointer_x - render.x) / render.scale * self.position.zoom)
				- self.position.drag_offset_x;
			self.position.y = pointer_y
				- ((pointer_y - render.y) / render.scale * self.position.zoom)
				- self.position.drag_offset_y;
		}
	}

	pub fn draw(&mut self, allocation: &gtk::Rectangle, context: &cairo::Context) {
		let surface = cairo::ImageSurface::create(
			cairo::Format::Rgb24,
			allocation.width(),
			allocation.height(),
		);
		let context2 = cairo::Context::new(surface.as_ref().unwrap()).unwrap();

		Self::copy_cairo_clip(context, &context2);

		self.draw_image(allocation, &context2);

		context
			.set_source_surface(surface.as_ref().unwrap(), 0.0, 0.0)
			.unwrap();
		context.paint().unwrap();
	}

	fn copy_cairo_clip(src: &cairo::Context, dst: &cairo::Context) {
		if let Ok(rects) = src.copy_clip_rectangle_list() {
			for rect in rects.iter() {
				dst.rectangle(rect.x(), rect.y(), rect.width(), rect.height());
			}
			dst.clip();
		} else if let Ok((x, y, width, height)) = src.clip_extents() {
			dst.rectangle(x, y, width - x, height - y);
			dst.clip();
		}
	}

	fn draw_image(&mut self, allocation: &gtk::Rectangle, context: &cairo::Context) {
		if let Some(image) = &self.image {
			self.orientation = image.orientation();

			let render = self.calc_render(allocation, image);

			context.translate(render.x, render.y);
			context.scale(render.scale, render.scale);

			image.with_surface(|surface, loaded| {
				self.waiting = surface.is_none();

				if let Some(surface) = surface {
					match self.orientation.rotate {
						Rotate::Rotate0 => {}

						Rotate::Rotate90 => {
							context.translate(f64::from(image.height), 0.0);
							context.rotate(std::f64::consts::PI * 0.5);
						}

						Rotate::Rotate180 => {
							context.translate(f64::from(image.width), f64::from(image.height));
							context.rotate(std::f64::consts::PI);
						}

						Rotate::Rotate270 => {
							context.translate(0.0, f64::from(image.width));
							context.rotate(std::f64::consts::PI * 1.5);
						}
					};

					if self.orientation.horizontal_flip {
						context.translate(f64::from(image.width), 0.0);
						context.scale(-1.0, 1.0);
					}

					let pattern = cairo::SurfacePattern::create(surface);
					pattern.set_filter(cairo::Filter::Fast);
					context.set_source(pattern).unwrap();
					context.paint().unwrap();

					// Release the `surface` after using it, before this closure
					// returns otherwise `context` will still have a reference
					// to it
					context.set_source_rgb(0.0, 0.0, 0.0);
				} else {
					if loaded {
						context.set_source_rgb(0.75, 0.5, 0.5);
					} else {
						context.set_source_rgb(0.5, 0.75, 0.5);
					}
					context.rectangle(0.0, 0.0, render.width, render.height);
					context.clip();
					context.paint().unwrap();
				}
			});
		}
	}

	fn calc_render(&self, allocation: &gtk::Rectangle, image: &Arc<Image>) -> Render {
		let output_width = f64::from(allocation.width());
		let output_height = f64::from(allocation.height());

		let (width, height) = match self.orientation.rotate {
			Rotate::Rotate0 | Rotate::Rotate180 => {
				(f64::from(image.width), f64::from(image.height))
			}
			Rotate::Rotate90 | Rotate::Rotate270 => {
				(f64::from(image.height), f64::from(image.width))
			}
		};

		let (x, y, scale) = if self.position.zoom.is_nan() {
			let scale = f64::min(output_width / width, output_height / height);
			let x;
			let y;

			if output_width / width >= output_height / height {
				x = (output_width - scale * width) / 2.0;
				y = 0.0;
			} else {
				x = 0.0;
				y = (output_height - scale * height) / 2.0;
			}

			(x, y, scale)
		} else {
			let scale = self.position.zoom;
			let constrain = |length, output_length, position| {
				if length * scale < output_length {
					// Image too small, centre
					(output_length - scale * length) / 2.0
				} else if position > 0.0 {
					// Gap before the image, move to the start edge
					0.0
				} else if position + length * scale < output_length {
					// Gap after the image, move to the end edge
					output_length - length * scale
				} else {
					position
				}
			};

			let x = constrain(
				width,
				output_width,
				self.position.x + self.position.drag_offset_x,
			);
			let y = constrain(
				width,
				output_width,
				self.position.y + self.position.drag_offset_y,
			);

			(x, y, scale)
		};

		Render {
			x,
			y,
			scale,
			width,
			height,
		}
	}
}
