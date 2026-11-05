//! Builds the list row for a search result.

use adw::gio;
use adw::gtk;
use adw::prelude::*;

use pop_launcher::{IconSource, SearchResult};

pub fn result_row(item: &SearchResult) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title_lines(1)
        .subtitle_lines(1)
        .activatable(true)
        // Keyboard focus stays in the search box, which drives the selection.
        .focusable(false)
        .build();
    // Results contain arbitrary text, such as file names, so it must not be parsed as markup.
    // This only works after construction: passed to the builder, the labels still parse the text.
    row.set_use_markup(false);
    row.set_title(&item.name);
    row.set_subtitle(&item.description);

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
