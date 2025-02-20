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

	fn redraw(&self) {
		if self.area.is_visible() {
			self.area.queue_draw();
		} else {
			self.area.show();
		}
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

	pub fn draw(&mut self, allocation: &gtk::Rectangle, context: &cairo::Context) {
		let surface = cairo::ImageSurface::create(
			cairo::Format::Rgb24,
			allocation.width(),
			allocation.height(),
		);
		let context2 = cairo::Context::new(surface.as_ref().unwrap()).unwrap();

		copy_cairo_clip(context, &context2);

		self.draw_image(allocation, &context2);

		context
			.set_source_surface(surface.as_ref().unwrap(), 0.0, 0.0)
			.unwrap();
		context.paint().unwrap();
	}

	fn draw_image(&mut self, allocation: &gtk::Rectangle, context: &cairo::Context) {
		if let Some(image) = &self.image {
			self.orientation = image.orientation();

			let position = self.calc_position(allocation, image);

			context.translate(position.x, position.y);
			context.scale(position.scale, position.scale);

			image.with_surface(|surface, loaded| {
				self.waiting = surface.is_none();

				if let Some(surface) = surface {
					match self.orientation.rotate {
						Rotate::Rotate0 => {}

						Rotate::Rotate90 => {
							context.translate(f64::from(image.height), 0.0);
							context.rotate(90.0);
						}

						Rotate::Rotate180 => {
							context.translate(f64::from(image.width), f64::from(image.height));
							context.rotate(180.0);
						}

						Rotate::Rotate270 => {
							context.translate(0.0, f64::from(image.width));
							context.rotate(270.0);
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

					context.set_source_rgb(0.0, 0.0, 0.0);
				} else {
					if loaded {
						context.set_source_rgb(0.75, 0.5, 0.5);
					} else {
						context.set_source_rgb(0.5, 0.75, 0.5);
					}
					context.rectangle(0.0, 0.0, position.width, position.height);
					context.clip();
					context.paint().unwrap();
				}
			});
		}
	}

	fn calc_position(&self, allocation: &gtk::Rectangle, image: &Arc<Image>) -> Render {
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

			if output_width / width >= output_height / height {
				((output_width - scale * width) / 2.0, 0.0, scale)
			} else {
				(0.0, (output_height - scale * height) / 2.0, scale)
			}
		} else {
			let scale = self.position.zoom;
			let mut rx = self.position.x + self.position.drag_offset_x;
			let mut ry = self.position.y + self.position.drag_offset_y;

			if width * scale < output_width {
				// Image width too small, centre horizontally
				rx = (output_width - scale * width) / 2.0;
			} else if rx > 0.0 {
				// Gap at the left of the image, move to the left edge
				rx = 0.0;
			} else if rx + width * scale < output_width {
				// Gap at the right of the image, move to the right edge
				rx = output_width - width * scale;
			}

			if height * scale < output_height {
				// Image height too small, centre vertically
				ry = (output_height - scale * height) / 2.0;
			} else if ry > 0.0 {
				// Gap at the top of the image, move to the top edge
				ry = 0.0;
			} else if ry + height * scale < output_height {
				// Gap at the bottom of the image, move to the bottom edge
				ry = output_height - height * scale;
			}

			(rx, ry, scale)
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
