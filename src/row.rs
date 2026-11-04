//! Builds the list row for a search result.

use adw::gio;
use adw::gtk;
use adw::prelude::*;

use pop_launcher::{IconSource, SearchResult};

pub fn result_row(item: &SearchResult) -> adw::ActionRow {
    // Results contain arbitrary text, such as file names, so it must not be parsed as markup.
    let row = adw::ActionRow::builder()
        .title(item.name.as_str())
        .subtitle(item.description.as_str())
        .use_markup(false)
        .title_lines(1)
        .subtitle_lines(1)
        .activatable(true)
        // Keyboard focus stays in the search box, which drives the selection.
        .focusable(false)
        .build();

    if let Some(icon) = &item.icon {
        let image = icon_image(icon);
        image.set_pixel_size(32);
        row.add_prefix(&image);
    }
    if let Some(icon) = &item.category_icon {
        let image = icon_image(icon);
        image.add_css_class("dim-label");
        row.add_suffix(&image);
    }
    row
}

fn icon_image(icon: &IconSource) -> gtk::Image {
    match icon {
        IconSource::Name(path) if path.starts_with('/') => gtk::Image::from_file(&**path),
        IconSource::Name(name) => gtk::Image::from_icon_name(name),
        IconSource::Mime(mime) => {
            gtk::Image::from_gicon(&gio::functions::content_type_get_icon(mime))
        }
    }
}
