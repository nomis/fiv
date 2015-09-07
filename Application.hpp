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

#ifndef fiv__APPLICATION_HPP_
#define fiv__APPLICATION_HPP_

#include <giomm-2.4/giomm/applicationcommandline.h>
#include <glibmm-2.4/glibmm/refptr.h>
#include <gtkmm-3.0/gtkmm/application.h>
#include <memory>

class MainWindow;

class Fiv;

class Application: public Gtk::Application, public std::enable_shared_from_this<Application> {
public:
	Application();
#if GLIBMM_MAJOR_VERSION < 2 || (GLIBMM_MAJOR_VERSION == 2 && GLIBMM_MINOR_VERSION < 46)
	virtual void on_shutdown();
#endif

protected:
	virtual void on_startup();
	virtual int on_command_line(const Glib::RefPtr<Gio::ApplicationCommandLine> &command_line);
	virtual void on_activate();
#if GLIBMM_MAJOR_VERSION > 2 || (GLIBMM_MAJOR_VERSION == 2 && GLIBMM_MINOR_VERSION >= 46)
	virtual void on_shutdown();
#endif

	void menu_file_exit(const Glib::VariantBase &parameter);

	std::shared_ptr<Fiv> fiv;
	std::shared_ptr<MainWindow> win;
};

#endif /* fiv__APPLICATION_HPP_ */
