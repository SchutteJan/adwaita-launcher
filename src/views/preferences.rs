use std::cell::RefCell;
use std::rc::Rc;

use adw::gtk;
use adw::prelude::*;

use crate::config::{Config, Theme};

pub fn apply(panel: &impl IsA<gtk::Widget>, config: &Config) {
    if config.frosted_glass {
        panel.add_css_class("glass");
    } else {
        panel.remove_css_class("glass");
    }
    adw::StyleManager::default().set_color_scheme(match config.theme {
        Theme::System => adw::ColorScheme::Default,
        Theme::Light => adw::ColorScheme::ForceLight,
        Theme::Dark => adw::ColorScheme::ForceDark,
    });
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

    let labels: Vec<&str> = Theme::ALL.iter().map(|theme| theme.label()).collect();
    let theme = adw::ComboRow::builder()
        .title("Theme")
        .model(&gtk::StringList::new(&labels))
        .selected(
            Theme::ALL
                .iter()
                .position(|theme| *theme == config.borrow().theme)
                .unwrap_or(0) as u32,
        )
        .build();

    theme.connect_selected_notify({
        let panel = parent.clone();
        let config = config.clone();
        move |row| {
            let Some(selected) = Theme::ALL.get(row.selected() as usize) else {
                return;
            };
            let mut config = config.borrow_mut();
            config.theme = *selected;
            config.save();
            apply(&panel, &config);
        }
    });

    let group = adw::PreferencesGroup::builder().title("Appearance").build();
    group.add(&theme);
    group.add(&frosted_glass);

    let page = adw::PreferencesPage::new();
    page.add(&group);

    let dialog = adw::PreferencesDialog::builder()
        .title("Preferences")
        .build();
    dialog.add(&page);
    dialog.present(Some(parent));
}
