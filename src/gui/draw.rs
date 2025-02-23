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

use crate::fiv::{
	numeric::{DimensionsF64, PointF64, PointI32, Sf64, XYf64, Xf64, Yf64, Zero},
	Image, Orientation, Rotate,
};
use gtk::{cairo, gdk, glib, prelude::*};
use std::{
	rc::Rc,
	sync::{Arc, Mutex},
};

#[derive(Debug, Default)]
pub struct Draw {
	area: gtk::DrawingArea,
	image_draw: Rc<Mutex<ImageDraw>>,
}

#[derive(Debug, Default)]
struct ImageDraw {
	image: Option<Arc<Image>>,
	waiting: bool,
	zoom: Zoom,
	orientation: Orientation,
}

#[derive(Debug, Default)]
pub struct Zoom {
	scale: Option<Sf64>,
	position: PointF64,
	drag_offset: PointF64,
}

#[derive(Debug)]
struct Render {
	scale: Sf64,
	position: PointF64,
	dimensions: DimensionsF64,
}

impl Draw {
	pub fn new<F: FnOnce(&gtk::DrawingArea)>(f: F) -> Rc<Self> {
		let draw = Rc::new(Self::default());
		let draw_ref = Rc::downgrade(&draw);
		let draw_image_ref = Rc::downgrade(&draw.image_draw);

		draw.area
			.connect_draw(move |area, context| -> glib::Propagation {
				if let Some(draw_image) = draw_image_ref.upgrade() {
					draw_image.lock().unwrap().draw(&area.allocation(), context);
				}

				glib::Propagation::Proceed
			});

		draw.area
			.connect_scroll_event(move |_, event| -> glib::Propagation {
				if let Some(draw_copy) = draw_ref.upgrade() {
					draw_copy.scroll(event);
				}

				glib::Propagation::Proceed
			});
		draw.area.add_events(gdk::EventMask::SCROLL_MASK);

		f(&draw.area);
		draw
	}

	pub fn refresh(&self, image: Arc<Image>) {
		if self.image_draw.lock().unwrap().refresh(image) {
			self.redraw();
		}
	}

	pub fn scroll(&self, event: &gdk::EventScroll) {
		let zoom_factor: Sf64 = Sf64::try_from(1.10).unwrap();

		match event.direction() {
			gdk::ScrollDirection::Up => {
				self.zoom_adjust(zoom_factor);
			}

			gdk::ScrollDirection::Down => {
				self.zoom_adjust(1.0 / zoom_factor);
			}

			_ => (),
		}
	}

	pub fn zoom_actual(&self) {
		let mut image_draw = self.image_draw.lock().unwrap();

		if image_draw.zoom_actual(&self.area.allocation(), self.pointer()) {
			self.redraw();
		}
	}

	pub fn zoom_adjust(&self, scale: Sf64) {
		let mut image_draw = self.image_draw.lock().unwrap();

		if image_draw.zoom_adjust(&self.area.allocation(), self.pointer(), scale) {
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

	fn pointer(&self) -> PointI32 {
		let window = self.area.window().unwrap();
		let seat = self.area.display().default_seat().unwrap();
		let device_position = window.device_position(&seat.pointer().unwrap());
		let device_position = PointI32::new(device_position.1.into(), device_position.2.into());
		let window_position = device_position + window.position().into();

		self.area
			.toplevel()
			.unwrap()
			.translate_coordinates(
				&self.area,
				window_position.x.into(),
				window_position.y.into(),
			)
			.unwrap()
			.into()
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
			self.zoom = Zoom::default();
			self.image = Some(image);

			true
		} else {
			self.waiting && image.loaded() || self.orientation != image.orientation()
		}
	}

	pub fn zoom_actual(&mut self, allocation: &gtk::Allocation, pointer: PointI32) -> bool {
		self.zoom(allocation, pointer, None)
	}

	pub fn zoom_adjust(
		&mut self,
		allocation: &gtk::Allocation,
		pointer: PointI32,
		scale: Sf64,
	) -> bool {
		self.zoom(allocation, pointer, Some(scale))
	}

	pub fn zoom_fit(&mut self) -> bool {
		self.zoom = Zoom::default();
		self.image.is_some()
	}

	fn zoom(
		&mut self,
		allocation: &gtk::Allocation,
		pointer: PointI32,
		scale: Option<Sf64>,
	) -> bool {
		if let Some(image) = &self.image {
			let pointer: PointF64 = pointer.into();
			let render = self.calc_render(allocation, image);
			let scale = scale.map_or(Sf64::actual(), |value| render.scale * value);

			self.zoom.scale = Some(scale);
			self.zoom.position = pointer
				- ((pointer - render.position) / render.scale * scale)
				- self.zoom.drag_offset;

			true
		} else {
			false
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

			context.translate(render.position.x.into(), render.position.y.into());
			context.scale(render.scale.into(), render.scale.into());

			image.with_surface(|surface, loaded| {
				self.waiting = surface.is_none();

				if let Some(surface) = surface {
					match self.orientation.rotate {
						Rotate::Rotate0 => {}

						Rotate::Rotate90 => {
							context.translate(image.height().into(), 0.0);
							context.rotate(std::f64::consts::PI * 0.5);
						}

						Rotate::Rotate180 => {
							context.translate(image.width().into(), image.height().into());
							context.rotate(std::f64::consts::PI);
						}

						Rotate::Rotate270 => {
							context.translate(0.0, image.width().into());
							context.rotate(std::f64::consts::PI * 1.5);
						}
					};

					if self.orientation.horizontal_flip {
						context.translate(image.width().into(), 0.0);
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
					context.rectangle(
						0.0,
						0.0,
						render.dimensions.width.into(),
						render.dimensions.height.into(),
					);
					context.clip();
					context.paint().unwrap();
				}
			});
		}
	}

	fn calc_render(&self, allocation: &gtk::Rectangle, image: &Arc<Image>) -> Render {
		let output: DimensionsF64 = allocation.into();
		let input: DimensionsF64 = match self.orientation.rotate {
			Rotate::Rotate0 | Rotate::Rotate180 => image.metadata.dimensions.into(),
			Rotate::Rotate90 | Rotate::Rotate270 => image.metadata.dimensions.rotate90().into(),
		};

		let (scale, position) = if let Some(scale) = self.zoom.scale {
			fn constrain<T: XYf64<T>>(input_length: T, output_length: T, position: T) -> T {
				if input_length < output_length {
					// Image too small, centre
					(output_length - input_length) / 2.0
				} else if position > T::zero() {
					// Gap before the image, move to the start edge
					T::zero()
				} else if position + input_length < output_length {
					// Gap after the image, move to the end edge
					output_length - input_length
				} else {
					position
				}
			}

			let input = input * scale;
			let mut position = self.zoom.position + self.zoom.drag_offset;

			position.x = constrain(input.width, output.width, position.x);
			position.y = constrain(input.height, output.height, position.y);

			(scale, position)
		} else {
			let width_ratio = Sf64::ratio(output.width, input.width);
			let height_ratio = Sf64::ratio(output.height, input.height);
			let scale = Sf64::min(width_ratio, height_ratio);
			let input = input * scale;

			let position = if width_ratio >= height_ratio {
				PointF64::new((output.width - input.width) / 2.0, Yf64::zero())
			} else {
				PointF64::new(Xf64::zero(), (output.height - input.height) / 2.0)
			};

			(scale, position)
		};

		Render {
			scale,
			position,
			dimensions: input,
		}
	}
}
