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
	Image, Orientation, Rotate,
	numeric::{DimensionsF64, PointF64, PointI32, Sf64, XYf64, Xf64, Yf64, Zero},
};
use gtk::{cairo, gdk, glib, prelude::*};
use std::{
	rc::Rc,
	sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct Draw {
	area: gtk::DrawingArea,
	drag_gesture: gtk::GestureDrag,
	zoom_gesture: gtk::GestureZoom,
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
	tolerance: PointF64,
}

#[derive(Debug)]
struct RenderAt {
	scale: Sf64,
	position: PointF64,
	dimensions: DimensionsF64,
}

impl Draw {
	pub fn new<F: FnOnce(&gtk::DrawingArea)>(f: F) -> Rc<Self> {
		let draw = {
			let area = gtk::DrawingArea::default();
			let drag_gesture = gtk::GestureDrag::new(&area);
			let zoom_gesture = gtk::GestureZoom::new(&area);

			Rc::new(Self {
				area,
				drag_gesture,
				zoom_gesture,
				image_draw: Rc::new(Mutex::new(ImageDraw::default())),
			})
		};

		{
			let draw_image_ref = Rc::downgrade(&draw.image_draw);
			draw.area
				.connect_draw(move |area, context| -> glib::Propagation {
					if let Some(draw_image) = draw_image_ref.upgrade() {
						draw_image.lock().unwrap().draw(&area.allocation(), context);
					}

					glib::Propagation::Proceed
				});
		}

		{
			let draw_ref = Rc::downgrade(&draw);
			draw.area
				.connect_scroll_event(move |_, event| -> glib::Propagation {
					if let Some(draw_copy) = draw_ref.upgrade() {
						draw_copy.scroll(event);
					}

					glib::Propagation::Proceed
				});
			draw.area.add_events(gdk::EventMask::SCROLL_MASK);
		}

		{
			let draw_ref = Rc::downgrade(&draw);
			draw.zoom_gesture.connect_scale_changed(move |_, scale| {
				if let Some(draw_copy) = draw_ref.upgrade() {
					draw_copy.zoom_adjust(Sf64::try_from(scale).unwrap());
				}
			});
		}

		{
			let draw_ref = Rc::downgrade(&draw);
			draw.drag_gesture
				.connect_drag_begin(move |_, start_x, start_y| {
					if let Some(draw_copy) = draw_ref.upgrade() {
						draw_copy.drag_begin((start_x, start_y).into());
					}
				});
		}

		{
			let draw_ref = Rc::downgrade(&draw);
			draw.drag_gesture
				.connect_drag_update(move |_, offset_x, offset_y| {
					if let Some(draw_copy) = draw_ref.upgrade() {
						draw_copy.drag_update((offset_x, offset_y).into());
					}
				});
		}

		{
			let draw_ref = Rc::downgrade(&draw);
			draw.drag_gesture
				.connect_drag_end(move |_, offset_x, offset_y| {
					if let Some(draw_copy) = draw_ref.upgrade() {
						draw_copy.drag_end((offset_x, offset_y).into());
					}
				});
		}

		f(&draw.area);
		draw
	}

	pub fn refresh(&self, image: Arc<Image>) {
		if self.image_draw.lock().unwrap().refresh(image) {
			self.redraw();
		}
	}

	pub fn drag_begin(&self, start: PointF64) {
		let mut image_draw = self.image_draw.lock().unwrap();
		let window = self.area.window().unwrap();
		let display = window.display();

		window.set_cursor(gdk::Cursor::for_display(&display, gdk::CursorType::Fleur).as_ref());
		image_draw.drag_begin(&self.area.allocation(), start);
	}

	pub fn drag_update(&self, offset: PointF64) {
		let mut image_draw = self.image_draw.lock().unwrap();

		if image_draw.drag_update(&self.area.allocation(), offset) {
			self.redraw();
		}
	}

	pub fn drag_end(&self, offset: PointF64) {
		let mut image_draw = self.image_draw.lock().unwrap();
		let window = self.area.window().unwrap();

		window.set_cursor(None);
		if image_draw.drag_end(&self.area.allocation(), offset) {
			self.redraw();
		}
	}

	fn scroll(&self, event: &gdk::EventScroll) {
		let zoom_factor = Sf64::try_from(1.10).unwrap();

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

	pub fn drag_begin(&mut self, allocation: &gtk::Allocation, _start: PointF64) -> bool {
		self.zoom.drag_offset = PointF64::default();

		if let Some(render) = self.calc_render(allocation, true) {
			self.zoom.position = render.position;
		}

		false
	}

	pub fn drag_update(&mut self, allocation: &gtk::Allocation, offset: PointF64) -> bool {
		self.zoom.drag_offset = offset;

		self.calc_render(allocation, true);
		self.image.is_some()
	}

	pub fn drag_end(&mut self, allocation: &gtk::Allocation, offset: PointF64) -> bool {
		self.zoom.drag_offset = offset;

		if let Some(render) = self.calc_render(allocation, true) {
			self.zoom.position = render.position;
		}

		self.zoom.drag_offset = PointF64::default();
		self.image.is_some()
	}

	pub fn zoom_actual(&mut self, allocation: &gtk::Allocation, pointer: PointI32) -> bool {
		self.zoom(allocation, pointer, None);
		self.image.is_some()
	}

	pub fn zoom_adjust(
		&mut self,
		allocation: &gtk::Allocation,
		pointer: PointI32,
		scale: Sf64,
	) -> bool {
		self.zoom(allocation, pointer, Some(scale));
		self.image.is_some()
	}

	pub fn zoom_fit(&mut self) -> bool {
		self.zoom = Zoom::default();
		self.image.is_some()
	}

	fn zoom(&mut self, allocation: &gtk::Allocation, pointer: PointI32, scale: Option<Sf64>) {
		if let Some(render) = self.calc_render(allocation, true) {
			let pointer: PointF64 = pointer.into();
			let scale = scale.map_or(Sf64::actual(), |value| render.scale * value);

			self.zoom.scale = Some(scale);
			self.zoom.position = pointer
				- ((pointer - render.position) / render.scale * scale)
				- self.zoom.drag_offset;

			self.calc_render(allocation, false);
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
		let Some(render) = self.calc_render(allocation, true) else {
			return;
		};

		let Some(image) = &self.image else {
			return;
		};

		self.orientation = image.orientation();

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
				// returns otherwise `context` will still have a reference to it
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

	fn calc_render(&mut self, allocation: &gtk::Rectangle, restricted: bool) -> Option<RenderAt> {
		let Some(image) = &self.image else {
			return None;
		};

		let output: DimensionsF64 = allocation.into();
		let input: DimensionsF64 = match self.orientation.rotate {
			Rotate::Rotate0 | Rotate::Rotate180 => image.metadata.dimensions.into(),
			Rotate::Rotate90 | Rotate::Rotate270 => image.metadata.dimensions.rotate90().into(),
		};

		let (scale, position, target_position) = if let Some(scale) = self.zoom.scale {
			/// Allow zooming in without changing position, but constrain drag
			/// operations so that they can't move further away from the centre
			fn constrain_value<T: XYf64<T>>(
				input_length: T,
				output_length: T,
				position: T,
				tolerance: T,
				restricted: bool,
				too_small: bool,
			) -> (T, T) {
				let target_position = if input_length <= output_length {
					// Image too small, try to centre
					(output_length - input_length) / 2.0
				} else if position > T::zero() {
					// Gap before the image, move to the start edge
					T::zero()
				} else if position + input_length < output_length {
					// Gap after the image, move to the end edge
					output_length - input_length
				} else {
					position
				};

				let position = if too_small {
					// Image too small in both directions, always centre
					(output_length - input_length) / 2.0
				} else if restricted {
					let lower = if tolerance.into().lt(&0.0) {
						target_position + tolerance
					} else {
						target_position
					};
					let upper = if tolerance.into().gt(&0.0) {
						target_position + tolerance
					} else {
						target_position
					};

					position.clamp(lower, upper)
				} else {
					position
				};

				(position, target_position)
			}

			fn constrain_point(
				input: DimensionsF64,
				output: DimensionsF64,
				position: PointF64,
				tolerance: PointF64,
				restricted: bool,
			) -> (PointF64, PointF64) {
				let too_small = input <= output;
				let x = constrain_value(
					input.width,
					output.width,
					position.x,
					tolerance.x,
					restricted,
					too_small,
				);
				let y = constrain_value(
					input.height,
					output.height,
					position.y,
					tolerance.y,
					restricted,
					too_small,
				);

				(PointF64::new(x.0, y.0), PointF64::new(x.1, y.1))
			}

			let (position, target_position) = constrain_point(
				input * scale,
				output,
				self.zoom.position + self.zoom.drag_offset,
				self.zoom.tolerance,
				restricted,
			);

			(scale, position, target_position)
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

			(scale, position, position)
		};

		self.zoom.tolerance = position - target_position;

		Some(RenderAt {
			scale,
			position,
			dimensions: input,
		})
	}
}
