/*
 Copyright 2015  Simon Arlott

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

#include "ImageDrawable.hpp"

#include <cairo.h>
#include <cairomm/context.h>
#include <cairomm/enums.h>
#include <cairomm/pattern.h>
#include <cairomm/refptr.h>
#include <cairomm/surface.h>
#include <cairomm/types.h>
#include <gdk/gdk.h>
#include <gdkmm/device.h>
#include <gdkmm/rectangle.h>
#include <gdkmm/window.h>
#include <glib.h>
#include <glibmm/refptr.h>
#include <gtkmm/widget.h>
#include <algorithm>
#include <chrono>
#include <cmath>
#include <map>
#include <memory>
#include <set>
#include <utility>
#include <vector>

#include "Fiv.hpp"
#include "Image.hpp"

using namespace std;

using std::chrono::duration_cast;
using std::chrono::nanoseconds;
using std::chrono::steady_clock;

extern std::chrono::steady_clock::time_point startup;

ImageDrawable::ImageDrawable() {
	waiting = false;
	afPoints = false;
	zoom = NAN;
	x = 0;
	y = 0;
	dragOffsetX = 0;
	dragOffsetY = 0;

	add_events(Gdk::SCROLL_MASK);
}

void ImageDrawable::setImages(shared_ptr<Fiv> images_) {
	images = images_;
}

void ImageDrawable::update() {
	zoom = NAN;
	redraw();
}

void ImageDrawable::redraw() {
	if (is_visible()) {
		auto win = get_window();
		if (win) {
			auto allocation = get_allocation();
			Gdk::Rectangle rect(0, 0, allocation.get_width(), allocation.get_height());
			win->invalidate_rect(rect, false);
		}
	} else {
		show();
	}
}

void ImageDrawable::loaded() {
	if (waiting)
		redraw();
}

void inline ImageDrawable::calcRenderedImage(shared_ptr<Image> image, const Gtk::Allocation &allocation,
		int &rwidth, int &rheight, double &rscale, double &rx, double &ry) {
	const int awidth = allocation.get_width();
	const int aheight = allocation.get_height();

	switch (image->getOrientation().first) {
	case Image::Rotate::ROTATE_NONE:
	case Image::Rotate::ROTATE_180:
	default:
		rwidth = image->width();
		rheight = image->height();
		break;

	case Image::Rotate::ROTATE_90:
	case Image::Rotate::ROTATE_270:
		rwidth = image->height();
		rheight = image->width();
		break;
	}

	if (std::isnan(zoom)) {
		rscale = min((double)awidth / rwidth, (double)aheight / rheight);

		if ((double)awidth / rwidth >= (double)aheight / rheight) {
			rx = ((double)awidth - rscale * rwidth) / 2;
			ry = 0;
		} else {
			rx = 0;
			ry = ((double)aheight - rscale * rheight) / 2;
		}
	} else {
		rscale = zoom;
		rx = x + dragOffsetX;
		ry = y + dragOffsetY;

		if (rwidth * rscale < awidth) {
			// Image width too small, centre horizontally
			rx = ((double)awidth - rscale * rwidth) / 2;
		} else if (rx > 0) {
			// Gap at the left of the image, move to the left edge
			rx = 0;
		} else if (rx + rwidth * rscale < awidth) {
			// Gap at the right of the image, move to the right edge
			rx = awidth - rwidth * rscale;
		}

		if (rheight * rscale < aheight) {
			// Image height too small, centre vertically
			ry = ((double)aheight - rscale * rheight) / 2;
		} else if (ry > 0) {
			// Gap at the top of the image, move to the top edge
			ry = 0;
		} else if (ry + rheight * rscale < aheight) {
			// Gap at the bottom of the image, move to the bottom edge
			ry = aheight - rheight * rscale;
		}
	}
}

void ImageDrawable::finalisePosition() {
	int rheight, rwidth;
	double rscale, rx, ry;

	calcRenderedImage(images->current(), get_allocation(), rwidth, rheight, rscale, rx, ry);

	x = rx;
	y = ry;
	dragOffsetX = 0;
	dragOffsetY = 0;
}

void ImageDrawable::zoomActual() {
	applyZoom(NAN);
}

void ImageDrawable::zoomFit() {
	zoom = NAN;
	redraw();
}

void ImageDrawable::toggleAfPoints() {
	afPoints = !afPoints;
	redraw();
}

void ImageDrawable::dragBegin(double startX __attribute__((unused)), double startY __attribute__((unused))) {
	auto win = get_window();
	if (win)
		win->set_cursor(Gdk::Cursor::create(Gdk::CursorType::FLEUR));
	finalisePosition();
}

void ImageDrawable::dragUpdate(double offsetX, double offsetY) {
	dragOffsetX = offsetX;
	dragOffsetY = offsetY;
	redraw();
}

void ImageDrawable::dragEnd(double offsetX, double offsetY) {
	dragOffsetX = offsetX;
	dragOffsetY = offsetY;
	finalisePosition();
	redraw();

	auto win = get_window();
	if (win)
		win->set_cursor();
}

void ImageDrawable::applyZoom(double scale) {
	int px, py;
	int rwidth, rheight;
	double rscale;
	double rx, ry;

	get_pointer(px, py);

	calcRenderedImage(images->current(), get_allocation(), rwidth, rheight, rscale, rx, ry);

	if (std::isnan(scale)) {
		zoom = 1;
	} else {
		zoom = rscale * scale;
	}
	x = px - ((px - rx) / rscale * zoom) - dragOffsetX;
	y = py - ((py - ry) / rscale * zoom) - dragOffsetY;
	redraw();
}

static void copyCairoClip(const Cairo::RefPtr<Cairo::Context> &src, const Cairo::RefPtr<Cairo::Context> &dst) {
	try {
		vector<Cairo::Rectangle> rects;
		src->copy_clip_rectangle_list(rects);
		for (auto& rect : rects) {
			//cout << "clip " << rect.x << "x" << rect.y << "+" << rect.width << "+" << rect.height << endl;
			dst->rectangle(rect.x, rect.y, rect.width, rect.height);
		}
		dst->clip();
	} catch (...) {
		Cairo::Rectangle rect;
		src->get_clip_extents(rect.x, rect.y, rect.width, rect.height);
		rect.width -= rect.x;
		rect.height -= rect.y;
		//cout << "clip " << rect.x << "x" << rect.y << "+" << rect.width << "+" << rect.height << endl;
		dst->rectangle(rect.x, rect.y, rect.width, rect.height);
		dst->clip();
	}
}

bool ImageDrawable::on_draw(const Cairo::RefPtr<Cairo::Context> &cr) {
	static bool first_draw = true;
	auto started = steady_clock::now();

	Gtk::Allocation allocation = get_allocation();

	//cout << "draw " << awidth << "x" << aheight << endl;

	auto surface = Cairo::ImageSurface::create(Cairo::Format::FORMAT_RGB24, allocation.get_width(), allocation.get_height());
	auto cr2 = Cairo::Context::create(surface);
	copyCairoClip(cr, cr2);
	drawImage(cr2, allocation);

	//auto start = chrono::steady_clock::now();
	cr->set_source(surface, 0, 0);
	cr->paint();
	//auto stop = chrono::steady_clock::now();
	//cout << "copy " << chrono::duration_cast<chrono::milliseconds>(stop - start).count() << "ms" << endl;

	if (first_draw && !waiting) {
		first_draw = false;

		std::cout << "First image draw started at "
			<< duration_cast<nanoseconds>(started - startup).count() / 1000000.0
			<< "ms" << endl;

		auto end = steady_clock::now();
		std::cout << "First image draw finished at "
			<< duration_cast<nanoseconds>(end - startup).count() / 1000000.0
			<< "ms" << endl;
	}

	return true;
}

void ImageDrawable::drawImage(const Cairo::RefPtr<Cairo::Context> &cr, const Gtk::Allocation &allocation) {
	auto image = images->current();
	auto surface = image->getPrimary();
	int rwidth, rheight;
	double rscale;
	double rx, ry;

	//cout << "image " << iwidth << "x" << iheight << " " << iorientation.first << "," << iorientation.second << endl;

	calcRenderedImage(image, allocation, rwidth, rheight, rscale, rx, ry);

	cr->translate(rx, ry);
	cr->scale(rscale, rscale);

	waiting = !surface;

	if (image->isPrimaryFailed()) {
		// TODO display fancy failed indicator
		cr->set_source_rgb(0.75, 0.5, 0.5);
		cr->rectangle(0, 0, rwidth, rheight);
		cr->clip();
		cr->paint();
		return;
	} else if (!surface) {
		// TODO display fancy loading animation
		cr->set_source_rgb(0.5, 0.75, 0.5);
		cr->rectangle(0, 0, rwidth, rheight);
		cr->clip();
		cr->paint();
		return;
	}

	switch (image->getOrientation().first) {
	case Image::Rotate::ROTATE_NONE:
		break;

	case Image::Rotate::ROTATE_90:
		cr->translate(image->height(), 0);
		cr->rotate_degrees(90);
		break;

	case Image::Rotate::ROTATE_180:
		cr->translate(image->width(), image->height());
		cr->rotate_degrees(180);
		break;

	case Image::Rotate::ROTATE_270:
		cr->translate(0, image->width());
		cr->rotate_degrees(270);
		break;
	}

	if (image->getOrientation().second) {
		cr->translate(image->width(), 0);
		cr->scale(-1, 1);
	}

	auto pattern = Cairo::SurfacePattern::create(surface);
	pattern->set_filter(Cairo::Filter::FILTER_FAST);
	cr->set_source(pattern);

	//auto start = chrono::steady_clock::now();
	cr->paint();
	//auto stop = chrono::steady_clock::now();
	//cout << "paint " << chrono::duration_cast<chrono::milliseconds>(stop - start).count() << "ms" << endl;

	if (afPoints) {
		//start = chrono::steady_clock::now();
		auto properties = image->getProperties();
		valarray<double> dashes(5.0 / rscale, 5.0 / rscale);

		cr->save();
		cr->set_operator(static_cast<Cairo::Operator>(CAIRO_OPERATOR_DIFFERENCE));

		for (auto &rect : properties.focusPoints) {
			if (properties.focusPointsActive.find(rect) != properties.focusPointsActive.cend()) {
				cr->set_source_rgb(1, 0, 1);
				cr->set_line_width(4.0 / rscale);
				cr->unset_dash();
			} else if (properties.focusPointsSelected.find(rect) != properties.focusPointsSelected.cend()) {
				cr->set_source_rgb(1, 0, 0);
				cr->set_line_width(2.0 / rscale);
				cr->unset_dash();
			} else {
				cr->set_source_rgb(1, 1, 1);
				cr->set_line_width(1.0 / rscale);
				cr->set_dash(dashes, 0);
			}
			cr->rectangle(rect.x, rect.y, rect.width, rect.height);
			cr->stroke();
		}

		cr->restore();

		//stop = chrono::steady_clock::now();
		//cout << "afpaint " << chrono::duration_cast<chrono::milliseconds>(stop - start).count() << "ms" << endl;
	}
}

bool ImageDrawable::on_scroll_event(GdkEventScroll *scroll) {
	static const double zoomFactor = 1.10;

	if (scroll->direction == GDK_SCROLL_UP) {
		applyZoom(zoomFactor);
	} else if (scroll->direction == GDK_SCROLL_DOWN) {
		applyZoom(1.0/zoomFactor);
	}

	return true;
}
