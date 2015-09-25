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

#include <cairomm/context.h>
#include <cairomm/enums.h>
#include <cairomm/pattern.h>
#include <cairomm/refptr.h>
#include <cairomm/surface.h>
#include <cairomm/types.h>
#include <gdk/gdk.h>
#include <gdkmm/rectangle.h>
#include <gdkmm/window.h>
#include <glib.h>
#include <glibmm/refptr.h>
#include <gtkmm/widget.h>
#include <algorithm>
#include <cmath>
#include <iostream>
#include <memory>
#include <mutex>
#include <utility>
#include <vector>

#include "Fiv.hpp"
#include "Image.hpp"

using namespace std;

ImageDrawable::ImageDrawable() {
	waiting = false;
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
	lock_guard<mutex> lckDrawing(mtxDrawing);

	if (waiting)
		redraw();
}

bool inline ImageDrawable::calcRenderedImage(shared_ptr<Image> image, const int &awidth, const int &aheight, Image::Orientation iorientation, int &iwidth, int &iheight,
		int &rwidth, int &rheight, double &rscale, double &rx, double &ry) {
	iorientation = image->getOrientation();
	iwidth = image->width();
	iheight = image->height();

	switch (iorientation.first) {
	case Image::Rotate::ROTATE_NONE:
	case Image::Rotate::ROTATE_180:
		rwidth = iwidth;
		rheight = iheight;
		break;

	case Image::Rotate::ROTATE_90:
	case Image::Rotate::ROTATE_270:
		rwidth = iheight;
		rheight = iwidth;
		break;

	default:
		return false;
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

	return true;
}

void ImageDrawable::finaliseRenderedImage() {
	Gtk::Allocation allocation = get_allocation();
	const int awidth = allocation.get_width();
	const int aheight = allocation.get_height();

	auto current = images->current();
	auto image = current->getPrimary();
	Image::Orientation iorientation;
	int iwidth, iheight;
	int rwidth, rheight;
	double rscale;
	double rx, ry;

	if (!calcRenderedImage(current, awidth, aheight, iorientation, iwidth, iheight, rwidth, rheight, rscale, rx, ry))
		return;

	x = rx;
	y = ry;
	dragOffsetX = 0;
	dragOffsetY = 0;
}

void ImageDrawable::zoomActual() {
	applyZoom(NAN);
}

void ImageDrawable::zoomFit() {
	lock_guard<mutex> lckDrawing(mtxDrawing);

	zoom = NAN;
	redraw();
}

void ImageDrawable::dragBegin(double startX __attribute__((unused)), double startY __attribute__((unused))) {
	finaliseRenderedImage();
}

void ImageDrawable::dragUpdate(double offsetX, double offsetY) {
	unique_lock<mutex> lckDrawing(mtxDrawing);
	dragOffsetX = offsetX;
	dragOffsetY = offsetY;
	redraw();
}

void ImageDrawable::dragEnd(double offsetX, double offsetY) {
	unique_lock<mutex> lckDrawing(mtxDrawing);
	dragOffsetX = offsetX;
	dragOffsetY = offsetY;
	finaliseRenderedImage();
	redraw();
}

void ImageDrawable::applyZoom(double scale) {
	Gtk::Allocation allocation = get_allocation();
	const int awidth = allocation.get_width();
	const int aheight = allocation.get_height();
	int px, py;

	get_pointer(px, py);

	lock_guard<mutex> lckDrawing(mtxDrawing);

	auto current = images->current();
	auto image = current->getPrimary();
	Image::Orientation iorientation;
	int iwidth, iheight;
	int rwidth, rheight;
	double rscale;
	double rx, ry;

	if (!calcRenderedImage(current, awidth, aheight, iorientation, iwidth, iheight, rwidth, rheight, rscale, rx, ry))
		return;

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
	Gtk::Allocation allocation = get_allocation();
	const int awidth = allocation.get_width();
	const int aheight = allocation.get_height();

	//cout << "draw " << awidth << "x" << aheight << endl;

	auto surface = Cairo::ImageSurface::create(Cairo::Format::FORMAT_RGB24, awidth, aheight);
	auto cr2 = Cairo::Context::create(surface);
	copyCairoClip(cr, cr2);
	drawImage(cr2, awidth, aheight);

	//auto start = chrono::steady_clock::now();
	cr->set_source(surface, 0, 0);
	cr->paint();
	//auto stop = chrono::steady_clock::now();
	//cout << "copy " << chrono::duration_cast<chrono::milliseconds>(stop - start).count() << "ms" << endl;

	return true;
}

void ImageDrawable::drawImage(const Cairo::RefPtr<Cairo::Context> &cr, const int awidth, const int aheight) {
	unique_lock<mutex> lckDrawing(mtxDrawing);

	auto current = images->current();
	auto image = current->getPrimary();
	Image::Orientation iorientation;
	int iwidth, iheight;
	int rwidth, rheight;
	double rscale;
	double rx, ry;

	//cout << "image " << iwidth << "x" << iheight << " " << iorientation.first << "," << iorientation.second << endl;

	if (!calcRenderedImage(current, awidth, aheight, iorientation, iwidth, iheight, rwidth, rheight, rscale, rx, ry))
		return;

	cr->translate(rx, ry);
	cr->scale(rscale, rscale);

	waiting = !image;
	lckDrawing.unlock();

	if (!image) {
		// TODO display fancy loading animation
		// TODO handle failed images
		cr->set_source_rgb(0, 0, 0);
		cr->paint();
		return;
	}

	switch (iorientation.first) {
	case Image::Rotate::ROTATE_NONE:
		break;

	case Image::Rotate::ROTATE_90:
		cr->translate(iheight, 0);
		cr->rotate_degrees(90);
		break;

	case Image::Rotate::ROTATE_180:
		cr->translate(iwidth, iheight);
		cr->rotate_degrees(180);
		break;

	case Image::Rotate::ROTATE_270:
		cr->translate(0, iwidth);
		cr->rotate_degrees(270);
		break;
	}

	if (iorientation.second) {
		cr->translate(iwidth, 0);
		cr->scale(-1, 1);
	}

	auto pattern = Cairo::SurfacePattern::create(image);
	pattern->set_filter(Cairo::Filter::FILTER_FAST);
	cr->set_source(pattern);

	//auto start = chrono::steady_clock::now();
	cr->paint();
	//auto stop = chrono::steady_clock::now();
	//cout << "paint " << chrono::duration_cast<chrono::milliseconds>(stop - start).count() << "ms" << endl;
}

bool ImageDrawable::on_scroll_event(GdkEventScroll *event) {
	static const double zoomFactor = 1.10;

	if (event->direction == GDK_SCROLL_UP) {
		applyZoom(zoomFactor);
	} else if (event->direction == GDK_SCROLL_DOWN) {
		applyZoom(1.0/zoomFactor);
	}

	return true;
}
