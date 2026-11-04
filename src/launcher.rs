//! Connects the widgets to pop-launcher: searching, selecting and activating results.

use std::cell::{Cell, RefCell};
use std::path::Path;
use std::rc::Rc;

use adw::gtk;
use adw::prelude::*;

use crate::row::result_row;
use crate::service::Service;
use pop_launcher::{Request, Response, SearchResult};

pub struct Launcher {
    pub window: adw::ApplicationWindow,
    pub entry: gtk::SearchEntry,
    pub list: gtk::ListBox,
    pub scroller: gtk::ScrolledWindow,
    pub service: Service,
    /// A search was sent and its `Update` hasn't arrived. Updates carry no query ID, and
    /// result IDs are only valid for the latest search, so one search runs at a time.
    pub searching: Cell<bool>,
    /// The newest text typed while a search was running.
    pub queued: RefCell<Option<String>>,
    /// Enter was pressed during a search, so activate once its results arrive.
    pub activate_when_ready: Cell<bool>,
    /// The result ID of each row, by row index.
    pub ids: RefCell<Vec<u32>>,
}

impl Launcher {
    /// Hooks up all signals. Everything lives until the application exits.
    pub fn connect(self: Rc<Self>) {
        self.service.listen({
            let this = self.clone();
            move |response| this.handle(response)
        });
        self.entry.connect_changed({
            let this = self.clone();
            move |entry| this.search(entry.text().into())
        });
        self.entry.connect_activate({
            let this = self.clone();
            move |_| {
                if this.searching.get() {
                    this.activate_when_ready.set(true);
                } else {
                    this.activate_selected();
                }
            }
        });
        self.entry.connect_stop_search({
            let window = self.window.clone();
            move |_| window.close()
        });
        self.list.connect_row_activated({
            let this = self.clone();
            move |_, row| this.activate_row(row)
        });

        // Capture the arrow keys before the text field sees them.
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        keys.connect_key_pressed({
            let this = self.clone();
            move |_, key, _, _| match key {
                gtk::gdk::Key::Up => {
                    this.move_selection(-1);
                    gtk::glib::Propagation::Stop
                }
                gtk::gdk::Key::Down => {
                    this.move_selection(1);
                    gtk::glib::Propagation::Stop
                }
                _ => gtk::glib::Propagation::Proceed,
            }
        });
        self.entry.add_controller(keys);
    }

    fn search(&self, text: String) {
        if self.searching.get() {
            self.queued.replace(Some(text));
        } else {
            self.searching.set(true);
            self.service.send(&Request::Search(text));
        }
    }

    fn handle(&self, response: Response) {
        match response {
            Response::Update(items) => self.handle_update(&items),
            Response::Fill(text) => {
                self.entry.set_text(&text);
                self.entry.set_position(-1);
            }
            Response::DesktopEntry { path, .. } => self.launch(&path),
            Response::Close => self.window.close(),
            other => eprintln!("ignoring response: {other:?}"),
        }
    }

    fn handle_update(&self, items: &[SearchResult]) {
        self.searching.set(false);
        // These results are already out of date, so skip straight to the newest text.
        if let Some(text) = self.queued.take() {
            self.search(text);
            return;
        }
        self.show_results(items);
        if self.activate_when_ready.take() {
            self.activate_selected();
        }
    }

    fn show_results(&self, items: &[SearchResult]) {
        while let Some(row) = self.list.first_child() {
            self.list.remove(&row);
        }
        for item in items {
            self.list.append(&result_row(item));
        }
        self.ids.replace(items.iter().map(|item| item.id).collect());
        self.list.select_row(self.list.row_at_index(0).as_ref());
        self.scroller.vadjustment().set_value(0.0);
    }

    fn move_selection(&self, delta: i32) {
        let Some(selected) = self.list.selected_row() else {
            return;
        };
        if let Some(next) = self.list.row_at_index(selected.index() + delta) {
            self.list.select_row(Some(&next));
            self.scroll_to(&next);
        }
    }

    fn scroll_to(&self, row: &gtk::ListBoxRow) {
        let Some(bounds) = row.compute_bounds(&self.list) else {
            return;
        };
        // Include the list's padding, so the highlight isn't cut off at the panel edge.
        let top = f64::from(bounds.y()) - 12.0;
        let bottom = f64::from(bounds.y() + bounds.height()) + 12.0;
        let view = self.scroller.vadjustment();
        if top < view.value() {
            view.set_value(top);
        } else if bottom > view.value() + view.page_size() {
            view.set_value(bottom - view.page_size());
        }
    }

    fn activate_selected(&self) {
        if let Some(row) = self.list.selected_row() {
            self.activate_row(&row);
        }
    }

    fn activate_row(&self, row: &gtk::ListBoxRow) {
        // During a search, the IDs on the displayed rows no longer mean anything to the service.
        if self.searching.get() {
            return;
        }
        if let Some(id) = self.ids.borrow().get(row.index() as usize) {
            self.service.send(&Request::Activate(*id));
        }
    }

    fn launch(&self, path: &Path) {
        let Some(app) = gio_unix::DesktopAppInfo::from_filename(path) else {
            eprintln!("not a valid desktop entry: {}", path.display());
            return;
        };
        let context = WidgetExt::display(&self.window).app_launch_context();
        match app.launch(&[], Some(&context)) {
            Ok(()) => self.window.close(),
            Err(error) => eprintln!("failed to launch {}: {error}", path.display()),
        }
    }
}
