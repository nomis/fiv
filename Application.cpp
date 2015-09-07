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

#include "Application.hpp"

#include <giomm/application.h>
#include <giomm/applicationcommandline.h>
#include <giomm/menu.h>
#include <giomm/simpleaction.h>
#include <glibmm/optioncontext.h>
#include <glibmm/optiongroup.h>
#include <glibmm/refptr.h>
#include <glibmm/signalproxy.h>
#include <glibmm/ustring.h>
#include <glibmm/varianttype.h>
#include <gtk/gtkmain.h>
#include <sigc++/connection.h>
#include <sigc++/functors/mem_fun.h>
#include <cstdlib>
#include <memory>

#include "Fiv.hpp"
#include "MainWindow.hpp"

using namespace std;

Application::Application() : Gtk::Application(Fiv::appId, Gio::APPLICATION_HANDLES_COMMAND_LINE) {

}

void Application::on_startup() {
	Gtk::Application::on_startup();

	auto menubar = Gio::Menu::create();
	{
		auto mnuFile = Gio::Menu::create();
		{
			auto mnuFileExit = Gio::SimpleAction::create("file.exit", Glib::VARIANT_TYPE_STRING);
			{
				mnuFile->append("E_xit", mnuFileExit->get_name());
				set_accel_for_action(mnuFileExit->get_name(), "<Primary>q");

				mnuFileExit->signal_activate().connect(sigc::mem_fun(this, &Application::menu_file_exit));
				add_action(mnuFileExit);
			}
		}
		menubar->append_submenu("_File", mnuFile);
	}

	set_menubar(menubar);
}

int Application::on_command_line(const Glib::RefPtr<Gio::ApplicationCommandLine> &cmd) {
	Glib::OptionContext ctx;

	Glib::OptionGroup group("options", "Application Options");
	ctx.set_main_group(group);

	Glib::OptionGroup gtkgroup(gtk_get_option_group(true));
	ctx.add_group(gtkgroup);

	int argc;
	char **argv = cmd->get_arguments(argc);
	if (!ctx.parse(argc, argv))
		return EXIT_FAILURE;

	fiv = make_shared<Fiv>();
	if (!fiv->init(argc, argv))
		return EXIT_FAILURE;

	activate();
	return EXIT_SUCCESS;
}

void Application::on_activate() {
	win = make_shared<MainWindow>(fiv);
	add_window(*win);
	win->show();
}

void Application::on_shutdown() {
	if (fiv)
		fiv->exit();
}

void Application::menu_file_exit(const Glib::VariantBase &parameter __attribute__((unused))) {
	quit();
}
