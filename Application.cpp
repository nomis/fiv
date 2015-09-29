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

Application::Application() : Gtk::Application(Glib::ustring(), Gio::APPLICATION_HANDLES_COMMAND_LINE) {

}

void Application::on_startup() {
	Gtk::Application::on_startup();

	auto menubar = Gio::Menu::create();

	{
		auto mnuImage = Gio::Menu::create();

		mnuImage->append("Rotate _Left", "win.image.rotateLeft");
		set_accels_for_action("win.edit.rotateLeft", {"l"});

		mnuImage->append("Rotate _Right", "win.image.rotateRight");
		set_accels_for_action("win.image.rotateRight", {"r"});

		// TODO separator

		mnuImage->append("Flip _Horizontal", "win.image.flipHorizontal");
		set_accels_for_action("win.image.flipHorizontal", {"h"});

		mnuImage->append("Flip _Vertical", "win.image.flipVertical");
		set_accels_for_action("win.image.flipVertical", {"v"});

		// TODO separator

		mnuImage->append("_Quit", "app.quit");
		set_accels_for_action("app.quit", {"<Primary>q", "q", "<Alt>F4"});

		menubar->append_submenu("_Image", mnuImage);
	}

	{
		auto mnuEdit = Gio::Menu::create();

		mnuEdit->append("&Mark", "win.edit.mark");
		set_accels_for_action("win.edit.mark", {"Insert"});

		mnuEdit->append("&Toggle mark", "win.edit.toggleMark");
		set_accels_for_action("win.edit.toggleMark", {"Tab"});

		mnuEdit->append("&Unmark", "win.edit.unmark");
		set_accels_for_action("win.edit.unmark", {"Delete"});

		menubar->append_submenu("_Edit", mnuEdit);
	}

	{
		auto mnuView = Gio::Menu::create();

		mnuView->append("_Previous", "win.view.previous");
		set_accels_for_action("win.view.previous", {"Left"});

		mnuView->append("_Next", "win.view.next");
		set_accels_for_action("win.view.next", {"Right", "Return"});

		mnuView->append("_First", "win.view.first");
		set_accels_for_action("win.view.first", {"Home"});

		mnuView->append("_Last", "win.view.last");
		set_accels_for_action("win.view.last", {"End"});

		// TODO separator

		mnuView->append("Norm_al Size", "win.view.zoomActual");
		set_accels_for_action("win.view.zoomActual", {"a", "1"});

		mnuView->append("Best _Fit", "win.view.zoomFit");
		set_accels_for_action("win.view.zoomFit", {"f"});

		// TODO separator

		mnuView->append("F_ull Screen", "win.view.fullScreen");
		set_accels_for_action("win.view.fullScreen", {"F11"});

		// TODO separator

		mnuView->append("AF P_oints", "win.view.afPoints");
		set_accels_for_action("win.view.afPoints", {"p"});

		menubar->append_submenu("_View", mnuView);
	}

	set_menubar(menubar);
	add_action("quit", sigc::mem_fun(this, &Application::action_quit));
}

int Application::on_command_line(const Glib::RefPtr<Gio::ApplicationCommandLine> &cmd) {
	Glib::OptionContext ctx;


	Glib::OptionGroup group("options", "Application Options");

	Glib::OptionEntry maxPreload;
	int optMaxPreload = 100;
	maxPreload.set_short_name('p');
	maxPreload.set_long_name("preload");
	maxPreload.set_arg_description("N");
	stringstream maxPreloadDesc;
	maxPreloadDesc << "Number of images to preload (default=" << optMaxPreload << ")";
	maxPreload.set_description(maxPreloadDesc.str());
	group.add_entry(maxPreload, optMaxPreload);

	Glib::OptionEntry markDirectory;
	Glib::ustring optMarkDirectory = "";
	markDirectory.set_short_name('m');
	markDirectory.set_long_name("markDirectory");
	markDirectory.set_arg_description("path");
	markDirectory.set_description("Location to use to mark images using symlinks");
	group.add_entry(markDirectory, optMarkDirectory);

	ctx.set_main_group(group);

	Glib::OptionGroup gtkgroup(gtk_get_option_group(true));
	ctx.add_group(gtkgroup);

	int argc;
	char **argv = cmd->get_arguments(argc);
	if (!ctx.parse(argc, argv))
		return EXIT_FAILURE;

	fiv = make_shared<Fiv>();
	fiv->markDirectory = optMarkDirectory;
	fiv->maxPreload = optMaxPreload;
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
