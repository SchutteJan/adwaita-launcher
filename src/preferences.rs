use adw::gtk;
use adw::prelude::*;

// Opens the dialog over the launcher. It's built on demand to keep it out of startup time.
pub fn present(panel: &gtk::Box) {
    let frosted_glass = switch_row(
        panel,
        "Frosted Glass",
        "Adds frosted glass effect to launcher window",
        "glass",
    );

    let group = adw::PreferencesGroup::builder().title("Appearance").build();
    group.add(&frosted_glass);

    let page = adw::PreferencesPage::new();
    page.add(&group);

    let dialog = adw::PreferencesDialog::builder()
        .title("Preferences")
        .build();
    dialog.add(&page);
    dialog.present(Some(panel));
}

fn switch_row(
    panel: &gtk::Box,
    title: &str,
    subtitle: &str,
    on_class: &'static str,
) -> adw::SwitchRow {
    let row = adw::SwitchRow::builder()
        .title(title)
        .subtitle(subtitle)
        .active(panel.has_css_class(on_class))
        .build();
    row.connect_active_notify({
        let panel = panel.clone();
        move |row| {
            if row.is_active() {
                panel.add_css_class(on_class);
            } else {
                panel.remove_css_class(on_class);
            }
        }
    });
    row
}
