use adw::gio;
use adw::gtk;
use adw::prelude::*;

use pop_launcher::{IconSource, SearchResult};

/// The size of a result's own icon, in pixels. The category icon keeps GTK's default size.
const ICON_SIZE: i32 = 32;

/// Room kept above and below a row that is scrolled into view. It covers the list's padding,
/// so the highlight isn't cut off at the panel edge.
const SCROLL_MARGIN: f64 = 12.0;

#[derive(Clone)]
pub struct ResultList {
    list: gtk::ListBox,
    scroller: gtk::ScrolledWindow,
}

impl ResultList {
    pub fn new() -> Self {
        let list = gtk::ListBox::builder()
            .css_classes(["navigation-sidebar", "results"])
            .build();

        let scroller = gtk::ScrolledWindow::builder()
            .child(&list)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .build();

        Self { list, scroller }
    }

    pub fn widget(&self) -> &gtk::ScrolledWindow {
        &self.scroller
    }

    pub fn show(&self, items: &[SearchResult]) {
        while let Some(row) = self.list.first_child() {
            self.list.remove(&row);
        }
        for item in items {
            self.list.append(&result_row(item));
        }
        self.list.select_row(self.list.row_at_index(0).as_ref());
        self.scroller.vadjustment().set_value(0.0);
    }

    pub fn selected(&self) -> Option<usize> {
        row_index(&self.list.selected_row()?)
    }

    pub fn move_selection(&self, delta: i32) {
        let Some(selected) = self.list.selected_row() else {
            return;
        };
        if let Some(next) = self.list.row_at_index(selected.index() + delta) {
            self.list.select_row(Some(&next));
            self.scroll_to(&next);
        }
    }

    pub fn connect_activated(&self, on_activated: impl Fn(usize) + 'static) {
        self.list.connect_row_activated(move |_, row| {
            if let Some(index) = row_index(row) {
                on_activated(index);
            }
        });
    }

    fn scroll_to(&self, row: &gtk::ListBoxRow) {
        let Some(bounds) = row.compute_bounds(&self.list) else {
            return;
        };
        let top = f64::from(bounds.y()) - SCROLL_MARGIN;
        let bottom = f64::from(bounds.y() + bounds.height()) + SCROLL_MARGIN;
        let view = self.scroller.vadjustment();
        if top < view.value() {
            view.set_value(top);
        } else if bottom > view.value() + view.page_size() {
            view.set_value(bottom - view.page_size());
        }
    }
}

/// The position of `row` in its list, or `None` if it isn't in one.
fn row_index(row: &gtk::ListBoxRow) -> Option<usize> {
    usize::try_from(row.index()).ok()
}

fn result_row(item: &SearchResult) -> adw::ActionRow {
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
        image.set_pixel_size(ICON_SIZE);
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
