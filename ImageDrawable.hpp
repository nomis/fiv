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
#include <glib.h>
#include <gdk/gdk.h>
#include <gtkmm/drawingarea.h>
#include <gtkmm/gesture.h>
#include <memory>
#include <mutex>

#include "Image.hpp"

class Fiv;

class ImageDrawable: public Gtk::DrawingArea {
public:
	ImageDrawable();
	void setImages(std::shared_ptr<Fiv> images);
	void update();
	void redraw();
	void loaded();
	void zoomActual();
	void zoomFit();
	void toggleAfPoints();
	void dragBegin(double startX, double startY);
	void dragUpdate(double offsetX, double offsetY);
	void dragEnd(double offsetX, double offsetY);
	void applyZoom(double scale);

private:
	bool inline calcRenderedImage(std::shared_ptr<Image> image, const Gtk::Allocation &allocation,
			int &rwidth, int &rheight, double &rscale, double &rx, double &ry);
	void finalisePosition();
	void drawImage(const Cairo::RefPtr<Cairo::Context> &cr, const Gtk::Allocation &allocation);
	virtual bool on_draw(const Cairo::RefPtr<Cairo::Context> &cr);
	virtual bool on_scroll_event(GdkEventScroll *event);

	std::shared_ptr<Fiv> images;

	std::mutex mtxDrawing;
	bool waiting;
	bool afPoints;
	double zoom;
	double x, y;

	double dragOffsetX, dragOffsetY;
};

#endif /* fiv__IMAGEDRAWABLE_H_ */
