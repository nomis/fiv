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
#include <glibmm/optioncontext.h>
#include <glibmm/optiongroup.h>
#include <glibmm/refptr.h>
#include <gtk/gtkmain.h>
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
		auto mnuImage = Gio::Menu::create();
		mnuImage->append("_Quit", "app.quit");
		set_accels_for_action("app.quit", {"<Primary>q", "q", "<Alt>F4"});
		menubar->append_submenu("_Image", mnuImage);
	}

	{
		auto mnuEdit = Gio::Menu::create();
		mnuEdit->append("Rotate _Left", "win.edit.rotateLeft");
		set_accels_for_action("win.edit.rotateLeft", {"l"});

		mnuEdit->append("Rotate _Right", "win.edit.rotateRight");
		set_accels_for_action("win.edit.rotateRight", {"r"});

		mnuEdit->append("Flip _Horizontal", "win.edit.flipHorizontal");
		set_accels_for_action("win.edit.flipHorizontal", {"h"});

		mnuEdit->append("Flip _Vertical", "win.edit.flipVertical");
		set_accels_for_action("win.edit.flipVertical", {"v"});
		menubar->append_submenu("_Edit", mnuEdit);
	}

	{
		auto mnuView = Gio::Menu::create();
		mnuView->append("_Previous", "win.view.previous");
		set_accels_for_action("win.view.previous", {"Left"});

		mnuView->append("_Next", "win.view.next");
		set_accels_for_action("win.view.next", {"Right"});

		mnuView->append("_First", "win.view.first");
		set_accels_for_action("win.view.first", {"Home"});

		mnuView->append("_Last", "win.view.last");
		set_accels_for_action("win.view.last", {"End"});
		menubar->append_submenu("_View", mnuView);
	}

	set_menubar(menubar);
	add_action("quit", sigc::mem_fun(this, &Application::action_quit));
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
	fiv->addListener(static_pointer_cast<Events>(win));
	add_window(*win);
	win->show();
}

void Application::on_shutdown() {
	if (fiv)
		fiv->exit();
}

void Application::action_quit() {
	for (auto window : get_windows())
		window->hide();
}
