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

#ifndef fiv__IMAGEDRAWABLE_H_
#define fiv__IMAGEDRAWABLE_H_

#include <cairomm/context.h>
#include <cairomm/refptr.h>
#include <gtkmm/drawingarea.h>
#include <memory>

#include "Fiv.hpp"
#include "Image.hpp"

class ImageDrawable: public Gtk::DrawingArea {
public:
	ImageDrawable();
	void setImages(std::shared_ptr<Fiv::Images> images);
	void update();

private:
	virtual bool on_draw(const Cairo::RefPtr<Cairo::Context> &cr);
	void drawImage(const Cairo::RefPtr<Cairo::Context> &cr, const int width, const int height);

	std::shared_ptr<Fiv::Images> images;
};

#endif /* fiv__IMAGEDRAWABLE_H_ */
