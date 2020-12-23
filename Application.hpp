/*
 Copyright 2015,2020  Simon Arlott

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

#include <giomm/applicationcommandline.h>
#include <glibmm/refptr.h>
#include <gtkmm/application.h>
#include <glibmmconfig.h>
#include <memory>

class MainWindow;

class Fiv;

#ifndef GLIBMM_CHECK_VERSION
#define GLIBMM_CHECK_VERSION(major,minor,micro) \
	(GLIBMM_MAJOR_VERSION > (major) || \
	(GLIBMM_MAJOR_VERSION == (major) && GLIBMM_MINOR_VERSION > (minor)) || \
	(GLIBMM_MAJOR_VERSION == (major) && GLIBMM_MINOR_VERSION == (minor) && \
	 GLIBMM_MICRO_VERSION >= (micro)))
#endif

class Application: public Gtk::Application, public std::enable_shared_from_this<Application> {
public:
	Application();
#if !GLIBMM_CHECK_VERSION(2, 46, 0)
	void on_shutdown() override;
#endif

protected:
	void on_startup() override;
	int on_command_line(const Glib::RefPtr<Gio::ApplicationCommandLine> &command_line);
	void on_activate() override;
#if GLIBMM_CHECK_VERSION(2, 46, 0)
	void on_shutdown();
#endif

	void action_quit();

	std::shared_ptr<Fiv> fiv;
	std::shared_ptr<MainWindow> win;
};

#endif /* fiv__APPLICATION_HPP_ */
