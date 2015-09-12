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
#include <cairomm/matrix.h>
#include <cairomm/pattern.h>
#include <cairomm/refptr.h>
#include <gtkmm/widget.h>
#include <algorithm>
#include <chrono>
#include <cmath>
#include <functional>
#include <iostream>
#include <memory>
#include <sstream>
#include <string>
#include <thread>
#include <vector>

#include "Fiv.hpp"
#include "Image.hpp"

using namespace std;

ImageDrawable::ImageDrawable() {

}

void ImageDrawable::setImages(shared_ptr<Fiv::Images> images_) {
	images = images_;
}

bool ImageDrawable::on_draw(const Cairo::RefPtr<Cairo::Context> &cr) {
	Gtk::Allocation allocation = get_allocation();
	const int awidth = allocation.get_width();
	const int aheight = allocation.get_height();

	//cout << "draw " << awidth << "x" << aheight << endl;

	auto surface = Cairo::ImageSurface::create(Cairo::Format::FORMAT_RGB24, awidth, aheight);
	drawImage(surface);

	//auto start = chrono::steady_clock::now();
	cr->set_source(surface, 0, 0);
	cr->paint();
	//auto stop = chrono::steady_clock::now();
	//cout << "copy " << chrono::duration_cast<chrono::milliseconds>(stop - start).count() << "ms" << endl;

	return true;
}

void ImageDrawable::drawImage(const Cairo::RefPtr<Cairo::ImageSurface> &surface) {
	auto cr = Cairo::Context::create(surface);

	const int awidth = surface->get_width();
	const int aheight = surface->get_height();

	auto current = images->current();
	current->loadPrimary();
	auto image = current->getPrimary();
	auto orientation = current->getOrientation();
	const int iwidth = current->width();
	const int iheight = current->height();
	double scale = 1;

	//cout << "image " << iwidth << "x" << iheight << " " << orientation << endl;

	switch (orientation) {
	case Image::Orientation::NORMAL:
	case Image::Orientation::MIRROR_HORIZONTAL:
	case Image::Orientation::ROTATE_180:
	case Image::Orientation::MIRROR_VERTICAL:
	default:
		scale = min((double)awidth/iwidth, (double)aheight/iheight);
		break;

	case Image::Orientation::MIRROR_HORIZONTAL_ROTATE_270:
	case Image::Orientation::ROTATE_90:
	case Image::Orientation::MIRROR_HORIZONTAL_ROTATE_90:
	case Image::Orientation::ROTATE_270:
		scale = min((double)awidth/iheight, (double)aheight/iwidth);
		break;
	}

	cr->scale(scale, scale);

	switch (orientation) {
	case Image::Orientation::NORMAL:
	default:
		break;

	case Image::Orientation::MIRROR_HORIZONTAL:
		cr->translate(iwidth, 0);
		cr->scale(-1, 1);
		break;

	case Image::Orientation::ROTATE_180:
		cr->translate(iwidth, iheight);
		cr->rotate_degrees(180);
		break;

	case Image::Orientation::MIRROR_VERTICAL:
		cr->translate(0, iheight);
		cr->scale(1, -1);
		break;

	case Image::Orientation::MIRROR_HORIZONTAL_ROTATE_270:
		cr->translate(0, iwidth);
		cr->rotate_degrees(270);
		cr->translate(iwidth, 0);
		cr->scale(-1, 1);
		break;

	case Image::Orientation::ROTATE_90:
		cr->translate(iheight, 0);
		cr->rotate_degrees(90);
		break;

	case Image::Orientation::MIRROR_HORIZONTAL_ROTATE_90:
		cr->translate(iheight, 0);
		cr->rotate_degrees(90);
		cr->translate(iwidth, 0);
		cr->scale(-1, 1);
		break;

	case Image::Orientation::ROTATE_270:
		cr->translate(0, iwidth);
		cr->rotate_degrees(270);
		break;
	}

	auto pattern = Cairo::SurfacePattern::create(image);
	pattern->set_filter(Cairo::Filter::FILTER_FAST);
	cr->set_source(pattern);

	//auto start = chrono::steady_clock::now();
	cr->paint();
	//auto stop = chrono::steady_clock::now();
	//cout << "paint " << chrono::duration_cast<chrono::milliseconds>(stop - start).count() << "ms" << endl;
}
