mod launcher;
mod row;
mod service;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::gtk;
use adw::prelude::*;

use launcher::Launcher;
use pop_launcher::Request;
use service::Service;

const CSS: &str = include_str!("style.css");

fn main() {
    let application = adw::Application::builder()
        .application_id("io.github.schuttejan.AdwaitaLauncher")
        .build();

    application.connect_startup(|_| load_css());
    application.connect_activate(build_ui);
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

fn build_ui(app: &adw::Application) {
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
}
