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

#include <cstdlib>
#include <memory>

#include "Fiv.hpp"
#include "Image.hpp"

using namespace std;

int main(int argc, char *argv[]) {
	shared_ptr<Fiv> fiv(make_shared<Fiv>());
	if (!fiv->init(argc, argv))
		return EXIT_FAILURE;

	shared_ptr<Fiv::Images> images(make_shared<Fiv::Images>(fiv));
	shared_ptr<Image> image = images->current();
	cout << image << endl;
	cout << image->loadPrimary() << endl;
	if (image->loadThumbnail()) {
		image = image->getThumbnail();
		cout << image << endl;
		cout << image->loadPrimary() << endl;
	}

#if 0
	if (!Window::init())
		return EXIT_FAILURE;

	shared_ptr<Window> win(make_shared<MainWindow>(fiv));
	win->create();

	Window::mainLoop();
#endif
	return EXIT_SUCCESS;
}
