use adw::gtk;
use adw::prelude::*;

pub fn present(panel: &gtk::Box) {
    let dialog = adw::AboutDialog::builder()
        .application_name("Adwaita Launcher")
        .application_icon("system-search-symbolic")
        .version(env!("CARGO_PKG_VERSION"))
        .developer_name("Jan Schutte")
        .developers(["Jan Schutte"])
        .comments(env!("CARGO_PKG_DESCRIPTION"))
        .website(env!("CARGO_PKG_REPOSITORY"))
        .issue_url(concat!(env!("CARGO_PKG_REPOSITORY"), "/issues"))
        .license_type(gtk::License::Gpl30Only)
        .build();
    dialog.present(Some(panel));
}
