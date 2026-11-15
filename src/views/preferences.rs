use adw::gtk;
use adw::prelude::*;

/// Builds and opens the preferences dialog.
pub fn present(parent: &impl IsA<gtk::Widget>) {
    let on_class = "glass";
    let frosted_glass = adw::SwitchRow::builder()
        .title("Frosted Glass")
        .subtitle("Adds frosted glass effect to launcher window")
        .active(parent.has_css_class(on_class))
        .build();

    frosted_glass.connect_active_notify({
        let panel = parent.clone();
        move |row| {
            if row.is_active() {
                panel.add_css_class(on_class);
            } else {
                panel.remove_css_class(on_class);
            }
        }
    });

    let group = adw::PreferencesGroup::builder().title("Appearance").build();
    group.add(&frosted_glass);

    let page = adw::PreferencesPage::new();
    page.add(&group);

    let dialog = adw::PreferencesDialog::builder()
        .title("Preferences")
        .build();
    dialog.add(&page);
    dialog.present(Some(parent));
}
