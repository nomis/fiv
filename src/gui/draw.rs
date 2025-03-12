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

use crate::{
	fiv::{
		Image, Orientation, Rotate,
		numeric::{DimensionsF64, PointF64, PointI32, Sf64, XYf64, Xf64, Yf64, Zero},
	},
	nutype_const,
};
use gtk::{cairo, gdk, glib, prelude::*};
use log::trace;
use std::{
	rc::Rc,
	sync::{Arc, Mutex},
	time::Instant,
};

nutype_const!(SCROLL_ZOOM_FACTOR, Sf64, 1.10);

// Don't allow zooming too far in/out, it'll cause errors in cairo, and
// subnormal numbers are considered non-finite
nutype_const!(MIN_ZOOM, Sf64, 1.0 / u32::MAX as f64);
nutype_const!(MAX_ZOOM, Sf64, u32::MAX as f64);

#[derive(Debug)]
pub struct DrawingArea {
	widget: gtk::DrawingArea,
	drag_gesture: gtk::GestureDrag,
	zoom_gesture: gtk::GestureZoom,
	image_draw: Rc<Mutex<ImageDraw>>,
}

#[derive(Debug)]
struct ImageDraw {
	startup: Startup,
	image: Option<Arc<Image>>,
	waiting: bool,
	zoom: Zoom,
	orientation: Orientation,
	af_points: bool,
}

#[derive(Debug)]
struct Startup {
	begin: Instant,
	draw: bool,
}

#[derive(Debug, Default)]
pub struct Zoom {
	scale: Option<Sf64>,
	position: PointF64,
	drag_offset: PointF64,
	tolerance: PointF64,
}

#[derive(Debug)]
struct DrawAt {
	dimensions: DimensionsF64,
	position: PointF64,
	scale: Sf64,
}

impl ImageDraw {
	pub fn new(startup: Instant) -> Self {
		Self {
			startup: Startup::new(startup),
			image: None,
			waiting: false,
			zoom: Zoom::default(),
			orientation: Orientation::default(),
			af_points: false,
		}
	}
}

impl Startup {
	pub fn new(begin: Instant) -> Self {
		Self { begin, draw: false }
	}
}

impl DrawingArea {
	pub fn new<F: FnOnce(&gtk::DrawingArea)>(startup: Instant, f: F) -> Rc<Self> {
		let drawing_area = {
			let widget = gtk::DrawingArea::default();
			let drag_gesture = gtk::GestureDrag::new(&widget);
			let zoom_gesture = gtk::GestureZoom::new(&widget);

			Rc::new(Self {
				widget,
				drag_gesture,
				zoom_gesture,
				image_draw: Rc::new(Mutex::new(ImageDraw::new(startup))),
			})
		};

		{
			let draw_image_ref = Rc::downgrade(&drawing_area.image_draw);
			drawing_area
				.widget
				.connect_draw(move |area, context| -> glib::Propagation {
					if let Some(draw_image) = draw_image_ref.upgrade() {
						draw_image.lock().unwrap().draw(
							&area.allocation(),
							area.scale_factor(),
							context,
						);
					}

					glib::Propagation::Proceed
				});
		}

		{
			let draw_ref = Rc::downgrade(&drawing_area);
			drawing_area
				.widget
				.connect_scroll_event(move |_, event| -> glib::Propagation {
					if let Some(draw_copy) = draw_ref.upgrade() {
						draw_copy.scroll(event);
					}

					glib::Propagation::Proceed
				});

			drawing_area.widget.add_events(gdk::EventMask::SCROLL_MASK);
		}

		{
			let draw_ref = Rc::downgrade(&drawing_area);
			drawing_area
				.zoom_gesture
				.connect_scale_changed(move |_, scale| {
					if let Some(draw_copy) = draw_ref.upgrade() {
						draw_copy.zoom_adjust(Sf64::try_from(scale).unwrap());
					}
				});
		}

		{
			let draw_ref = Rc::downgrade(&drawing_area);
			drawing_area
				.drag_gesture
				.connect_drag_begin(move |_, start_x, start_y| {
					if let Some(draw_copy) = draw_ref.upgrade() {
						draw_copy.drag_begin((start_x, start_y).into());
					}
				});
		}

		{
			let draw_ref = Rc::downgrade(&drawing_area);
			drawing_area
				.drag_gesture
				.connect_drag_update(move |_, offset_x, offset_y| {
					if let Some(draw_copy) = draw_ref.upgrade() {
						draw_copy.drag_update((offset_x, offset_y).into());
					}
				});
		}

		{
			let draw_ref = Rc::downgrade(&drawing_area);
			drawing_area
				.drag_gesture
				.connect_drag_end(move |_, offset_x, offset_y| {
					if let Some(draw_copy) = draw_ref.upgrade() {
						draw_copy.drag_end((offset_x, offset_y).into());
					}
				});
		}

		f(&drawing_area.widget);
		drawing_area
	}

	pub fn refresh(&self, image: Arc<Image>) {
		if self.image_draw.lock().unwrap().refresh(image) {
			self.redraw();
		}
	}

	pub fn drag_begin(&self, start: PointF64) {
		let mut image_draw = self.image_draw.lock().unwrap();
		let window = self.widget.window().unwrap();
		let display = window.display();

		window.set_cursor(gdk::Cursor::for_display(&display, gdk::CursorType::Fleur).as_ref());
		image_draw.drag_begin(&self.widget.allocation(), start);
	}

	pub fn drag_update(&self, offset: PointF64) {
		let mut image_draw = self.image_draw.lock().unwrap();

		if image_draw.drag_update(&self.widget.allocation(), offset) {
			self.redraw();
		}
	}

	pub fn drag_end(&self, offset: PointF64) {
		let mut image_draw = self.image_draw.lock().unwrap();
		let window = self.widget.window().unwrap();

		window.set_cursor(None);
		if image_draw.drag_end(&self.widget.allocation(), offset) {
			self.redraw();
		}
	}

	fn scroll(&self, event: &gdk::EventScroll) {
		match event.direction() {
			gdk::ScrollDirection::Up => {
				self.zoom_adjust(SCROLL_ZOOM_FACTOR);
			}

			gdk::ScrollDirection::Down => {
				self.zoom_adjust(1.0 / SCROLL_ZOOM_FACTOR);
			}

			_ => (),
		}
	}

	pub fn zoom_actual(&self) {
		let mut image_draw = self.image_draw.lock().unwrap();

		if image_draw.zoom_actual(&self.widget.allocation(), self.pointer()) {
			self.redraw();
		}
	}

	pub fn zoom_adjust(&self, scale: Sf64) {
		let mut image_draw = self.image_draw.lock().unwrap();

		if image_draw.zoom_adjust(&self.widget.allocation(), self.pointer(), scale) {
			self.redraw();
		}
	}

	pub fn zoom_fit(&self) {
		if self.image_draw.lock().unwrap().zoom_fit() {
			self.redraw();
		}
	}

	pub fn af_points(&self, enable: bool) {
		if self.image_draw.lock().unwrap().af_points(enable) {
			self.redraw();
		}
	}

	fn redraw(&self) {
		if self.widget.is_visible() {
			self.widget.queue_draw();
		} else {
			self.widget.show();
		}
	}

	fn pointer(&self) -> PointI32 {
		let window = self.widget.window().unwrap();
		let seat = self.widget.display().default_seat().unwrap();
		let device_position = window.device_position(&seat.pointer().unwrap());
		let device_position = PointI32::new(device_position.1.into(), device_position.2.into());
		let window_position = device_position + window.position().into();

		self.widget
			.toplevel()
			.unwrap()
			.translate_coordinates(
				&self.widget,
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
			Some(other) => !Arc::ptr_eq(&image, other) || self.orientation != image.orientation(),
			None => true,
		};

		if changed {
			self.orientation = image.orientation();
			self.zoom = Zoom::default();
			self.image = Some(image);

			true
		} else {
			self.waiting && image.loaded()
		}
	}

	pub fn drag_begin(&mut self, allocation: &gtk::Allocation, _start: PointF64) -> bool {
		self.zoom.drag_offset = PointF64::default();

		if let Some(draw_at) = self.calc_draw_position(allocation, true) {
			self.zoom.position = draw_at.position;
		}

		false
	}

	pub fn drag_update(&mut self, allocation: &gtk::Allocation, offset: PointF64) -> bool {
		self.zoom.drag_offset = offset;

		self.calc_draw_position(allocation, true);
		self.image.is_some()
	}

	pub fn drag_end(&mut self, allocation: &gtk::Allocation, offset: PointF64) -> bool {
		self.zoom.drag_offset = offset;

		if let Some(draw_at) = self.calc_draw_position(allocation, true) {
			self.zoom.position = draw_at.position;
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
		if let Some(draw_at) = self.calc_draw_position(allocation, true) {
			let pointer: PointF64 = pointer.into();
			let scale = scale.map_or(Sf64::actual(), |value| {
				let value = value.clamp(MIN_ZOOM, MAX_ZOOM);

				(draw_at.scale * value).clamp(MIN_ZOOM, MAX_ZOOM)
			});

			self.zoom.scale = Some(scale);
			self.zoom.position = pointer
				- ((pointer - draw_at.position) / draw_at.scale * scale)
				- self.zoom.drag_offset;

			self.calc_draw_position(allocation, false);
		}
	}

	pub fn af_points(&mut self, enable: bool) -> bool {
		self.af_points = enable;
		self.image
			.as_ref()
			.is_some_and(|image| image.metadata.af_points.is_some())
	}

	pub fn draw(
		&mut self,
		allocation: &gtk::Rectangle,
		scale_factor: i32,
		context: &cairo::Context,
	) {
		let started = self.startup.begin.elapsed();
		let surface = cairo::ImageSurface::create(
			cairo::Format::Rgb24,
			allocation.width() * scale_factor,
			allocation.height() * scale_factor,
		)
		.unwrap();
		let scale_factor = f64::from(scale_factor);
		surface.set_device_scale(scale_factor, scale_factor);
		let context2 = cairo::Context::new(&surface).unwrap();

		Self::copy_cairo_clip(context, &context2);

		self.draw_image(allocation, &context2);

		context.set_source_surface(&surface, 0.0, 0.0).unwrap();
		context.paint().unwrap();

		if !self.startup.draw && !self.waiting {
			self.startup.draw = true;

			trace!("First image draw started at {:?}", started);
			trace!(
				"First image draw finished at {:?}",
				self.startup.begin.elapsed()
			);
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

	fn draw_image(&mut self, allocation: &gtk::Rectangle, context: &cairo::Context) {
		let Some(draw_at) = self.calc_draw_position(allocation, true) else {
			return;
		};

		let Some(image) = &self.image else {
			return;
		};

		self.orientation = image.orientation();

		context.translate(draw_at.position.x.into(), draw_at.position.y.into());
		context.scale(draw_at.scale.into(), draw_at.scale.into());

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

				if self.af_points {
					if let Some(af_points) = &image.metadata.af_points {
						let dashes = [(5.0 / draw_at.scale).into(); 2];
						let dots = [(2.0 / draw_at.scale).into(); 2];

						context.save().unwrap();
						context.set_operator(cairo::Operator::Difference);

						for af_point in af_points {
							if af_point.active {
								context.set_source_rgb(1.0, 0.0, 1.0);
								context.set_line_width((4.0 / draw_at.scale).into());
								context.set_dash(&[], 0.0);
							} else if af_point.selected {
								context.set_source_rgb(1.0, 0.0, 0.0);
								context.set_line_width((2.0 / draw_at.scale).into());
								context.set_dash(&dots, 0.0);
							} else {
								context.set_source_rgb(1.0, 1.0, 1.0);
								context.set_line_width((1.0 / draw_at.scale).into());
								context.set_dash(&dashes, 0.0);
							}
							context.rectangle(
								af_point.position.x.into(),
								af_point.position.y.into(),
								af_point.dimensions.width.into(),
								af_point.dimensions.height.into(),
							);
							context.stroke().unwrap();
						}

						context.restore().unwrap();
					}
				}
			} else {
				if loaded {
					context.set_source_rgb(0.75, 0.5, 0.5);
				} else {
					context.set_source_rgb(0.5, 0.75, 0.5);
				}
				context.rectangle(
					0.0,
					0.0,
					draw_at.dimensions.width.into(),
					draw_at.dimensions.height.into(),
				);
				context.clip();
				context.paint().unwrap();
			}
		});
	}

	fn calc_draw_position(
		&mut self,
		allocation: &gtk::Rectangle,
		restricted: bool,
	) -> Option<DrawAt> {
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

		Some(DrawAt {
			dimensions: input,
			position,
			scale,
		})
	}
}
