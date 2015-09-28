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

#include <glibmm/miscutils.h>
#include <glibmmconfig.h>
#include <iomanip>
#include <iostream>
#include <memory>

#include "Application.hpp"
#include "Fiv.hpp"
#include "ThreadLocalStream.hpp"

using namespace std;

int main(int argc, char *argv[]) {
	cout.rdbuf(&ThreadLocalOStream::instance);
	cerr.rdbuf(&ThreadLocalEStream::instance);
	clog.rdbuf(&ThreadLocalEStream::instance);

	Glib::set_application_name(Fiv::appName);
	auto app = make_shared<Application>();
	int ret = app->run(argc, argv);
#if !GLIBMM_CHECK_VERSION(2, 46, 0)
	app->on_shutdown();
#endif
	return ret;
}
