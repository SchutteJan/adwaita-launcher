use std::cell::RefCell;
use std::rc::Rc;

use adw::gtk;
use adw::prelude::*;

use crate::config::Config;

pub fn apply(panel: &impl IsA<gtk::Widget>, config: &Config) {
    if config.frosted_glass {
        panel.add_css_class("glass");
    } else {
        panel.remove_css_class("glass");
    }
}

/// Builds and opens the preferences dialog.
pub fn present(parent: &impl IsA<gtk::Widget>, config: &Rc<RefCell<Config>>) {
    let frosted_glass = adw::SwitchRow::builder()
        .title("Frosted Glass")
        .subtitle("Adds frosted glass effect to launcher window")
        .active(config.borrow().frosted_glass)
        .build();

    frosted_glass.connect_active_notify({
        let panel = parent.clone();
        let config = config.clone();
        move |row| {
            let mut config = config.borrow_mut();
            config.frosted_glass = row.is_active();
            config.save();
            apply(&panel, &config);
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
