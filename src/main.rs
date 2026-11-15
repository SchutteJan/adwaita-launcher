mod launcher;
mod search;
mod service;
mod views;

use adw::prelude::*;
use adw::{glib, gtk};

use launcher::Launcher;
use service::Service;
use views::window::Window;

const APP_ID: &str = "io.github.schuttejan.AdwaitaLauncher";
const EXIT_AFTER_FIRST_FRAME: &str = "exit-after-first-frame";

fn main() -> glib::ExitCode {
    // Handled before GTK starts, so it works without a display.
    if std::env::args().any(|arg| arg == "--version") {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return glib::ExitCode::SUCCESS;
    }

    // Changing the icon theme's resource paths after it has loaded discards it, and reloading
    // a large theme is the slowest part of startup. These two lines keep it to a single load:
    // libadwaita adds its path before the load, and GTK has no app resource path to add after.
    adw::init().expect("failed to initialize libadwaita");

    let application = adw::Application::builder().application_id(APP_ID).build();
    application.set_resource_base_path(None);

    // Registered so GApplication accepts the flags and lists them in --help.
    application.add_main_option(
        "version",
        glib::Char::from(0),
        glib::OptionFlags::NONE,
        glib::OptionArg::None,
        "Print the version and exit",
        None,
    );
    application.add_main_option(
        EXIT_AFTER_FIRST_FRAME,
        glib::Char::from(0),
        glib::OptionFlags::NONE,
        glib::OptionArg::None,
        "Quit once the first frame is painted, for benchmarking startup time",
        None,
    );
    let exit_after_first_frame =
        std::env::args().any(|arg| arg == format!("--{EXIT_AFTER_FIRST_FRAME}"));

    application.connect_startup(|_| load_css());
    application.connect_activate(move |app| activate(app, exit_after_first_frame));

    application.run()
}

fn load_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_str!("style.css"));

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("no default display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn activate(app: &adw::Application, exit_after_first_frame: bool) {
    let window = Window::new(app);
    let service = Service::spawn().expect("failed to start pop-launcher");
    let launcher = Launcher::new(&window, service);
    app.connect_shutdown(move |_| launcher.shutdown());

    window.present();

    if exit_after_first_frame {
        let app = app.clone();
        window.connect_after_paint(move || app.quit());
    }
}
