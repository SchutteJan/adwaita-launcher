mod launcher;
mod row;
mod service;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;
use adw::{glib, gtk};

use launcher::Launcher;
use pop_launcher::Request;
use service::Service;

const CSS: &str = include_str!("style.css");

const EXIT_AFTER_FIRST_FRAME: &str = "exit-after-first-frame";

fn main() {
    // Changing the icon theme's resource paths after it has loaded discards it, and reloading
    // a large theme is the slowest part of startup. These two lines keep it to a single load:
    // libadwaita adds its path before the load, and GTK has no app resource path to add after.
    adw::init().expect("failed to initialize libadwaita");

    let application = adw::Application::builder()
        .application_id("io.github.schuttejan.AdwaitaLauncher")
        .build();
    application.set_resource_base_path(None);

    // Registered so GApplication accepts the flag and lists it in --help.
    application.add_main_option(
        EXIT_AFTER_FIRST_FRAME,
        glib::Char::from(0),
        glib::OptionFlags::NONE,
        glib::OptionArg::None,
        "Quit once the first frame is painted, to benchmark startup time",
        None,
    );
    let exit_after_first_frame =
        std::env::args().any(|arg| arg == format!("--{EXIT_AFTER_FIRST_FRAME}"));

    application.connect_startup(|_| load_css());
    application.connect_activate(move |app| build_ui(app, exit_after_first_frame));
    application.run();
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(CSS);
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("no default display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &adw::Application, exit_after_first_frame: bool) {
    // add_css_class keeps GTK's default classes, which the css_classes builder method replaces.
    let entry = gtk::SearchEntry::builder()
        .placeholder_text("Search")
        .build();
    entry.add_css_class("launcher-entry");

    let list = gtk::ListBox::builder()
        .css_classes(["navigation-sidebar", "results"])
        .build();

    let scroller = gtk::ScrolledWindow::builder()
        .child(&list)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .build();

    let panel = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .css_classes(["glass"])
        .build();
    panel.append(&entry);
    panel.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    panel.append(&scroller);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Launcher")
        .default_width(640)
        .default_height(480)
        .content(&panel)
        .build();
    window.add_css_class("launcher-window");

    let launcher = Rc::new(Launcher {
        window: window.clone(),
        entry: entry.clone(),
        list,
        scroller,
        service: Service::spawn().expect("failed to start pop-launcher"),
        searching: Cell::new(false),
        queued: RefCell::new(None),
        activate_when_ready: Cell::new(false),
        ids: RefCell::new(Vec::new()),
    });
    launcher.clone().connect();
    app.connect_shutdown(move |_| launcher.service.send(&Request::Exit));

    window.present();
    entry.grab_focus();

    if exit_after_first_frame {
        let app = app.clone();
        window
            .frame_clock()
            .expect("a presented window is realized")
            .connect_after_paint(move |_| app.quit());
    }
}
