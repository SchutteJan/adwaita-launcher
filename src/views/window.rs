use std::cell::RefCell;
use std::rc::Rc;

use adw::gio;
use adw::gtk;
use adw::prelude::*;

use pop_launcher::SearchResult;

use crate::config::Config;
use crate::views::results::{ROW_SHORTCUTS, ResultList};
use crate::views::{about, preferences};

#[derive(Debug)]
pub enum Event {
    QueryChanged(String),
    ResultActivated(usize),
}

#[derive(Clone)]
pub struct Window {
    window: adw::ApplicationWindow,
    // Keep private please
    widgets: WindowWidgets,
}

#[derive(Clone)]
struct WindowWidgets {
    /// Top-level container for the window view
    container: gtk::Box,
    /// Text entry for commands
    search_entry: gtk::SearchEntry,
    results: ResultList,
}

impl Window {
    pub fn new(app: &adw::Application, config: Config) -> Self {
        let widgets = Window::build_ui();
        preferences::apply(&widgets.container, &config);
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Launcher")
            .default_width(640)
            .default_height(480)
            .content(&widgets.container)
            .build();
        window.add_css_class("launcher-window");

        let action_preferences = gio::ActionEntry::builder("preferences")
            .activate({
                let container = widgets.container.clone();
                let config = Rc::new(RefCell::new(config));
                move |_: &adw::ApplicationWindow, _, _| preferences::present(&container, &config)
            })
            .build();

        let action_about = gio::ActionEntry::builder("about")
            .activate(|window, _, _| about::present(window))
            .build();

        let action_close = gio::ActionEntry::builder("close")
            .activate(|window: &adw::ApplicationWindow, _, _| {
                window.close();
            })
            .build();

        window.add_action_entries([action_preferences, action_about, action_close]);

        let this = Self { window, widgets };
        this.connect_keys();
        this
    }

    pub fn present(&self) {
        self.window.present();
        self.widgets.search_entry.grab_focus();
    }

    pub fn close(&self) {
        self.window.close();
    }

    pub fn set_query(&self, text: &str) {
        self.widgets.search_entry.set_text(text);
        self.widgets.search_entry.set_position(-1);
    }

    pub fn show_results(&self, items: &[SearchResult]) {
        self.widgets.results.show(items);
    }

    pub fn launch_context(&self) -> gio::AppLaunchContext {
        WidgetExt::display(&self.window)
            .app_launch_context()
            .upcast()
    }

    pub fn listen(&self, on_event: impl Fn(Event) + 'static) {
        let on_event = Rc::new(on_event);
        let WindowWidgets {
            search_entry,
            results,
            ..
        } = &self.widgets;

        search_entry.connect_changed({
            let on_event = on_event.clone();
            move |entry| on_event(Event::QueryChanged(entry.text().into()))
        });
        search_entry.connect_activate({
            let on_event = on_event.clone();
            let results = results.clone();
            move |_| {
                if let Some(index) = results.selected() {
                    on_event(Event::ResultActivated(index));
                }
            }
        });
        let shortcuts = gtk::ShortcutController::new();
        for (index, accelerator) in ROW_SHORTCUTS.into_iter().enumerate() {
            let on_event = on_event.clone();
            let activate = gtk::CallbackAction::new(move |_, _| {
                on_event(Event::ResultActivated(index));
                gtk::glib::Propagation::Stop
            });
            shortcuts.add_shortcut(gtk::Shortcut::new(
                gtk::ShortcutTrigger::parse_string(accelerator),
                Some(activate),
            ));
        }
        search_entry.add_controller(shortcuts);

        results.connect_activated(move |index| on_event(Event::ResultActivated(index)));
    }

    pub fn connect_after_paint(&self, on_paint: impl Fn() + 'static) {
        self.window
            .frame_clock()
            .expect("a presented window is realized")
            .connect_after_paint(move |_| on_paint());
    }

    fn connect_keys(&self) {
        let entry = &self.widgets.search_entry;

        entry.connect_stop_search({
            let window = self.window.clone();
            move |_| window.close()
        });

        // Capture the arrow keys, so they don't update cursor position
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        keys.connect_key_pressed({
            let results = self.widgets.results.clone();
            move |_, key, _, _| match key {
                gtk::gdk::Key::Up => {
                    results.move_selection(-1);
                    gtk::glib::Propagation::Stop
                }
                gtk::gdk::Key::Down => {
                    results.move_selection(1);
                    gtk::glib::Propagation::Stop
                }
                _ => gtk::glib::Propagation::Proceed,
            }
        });
        entry.add_controller(keys);
    }

    fn build_ui() -> WindowWidgets {
        let search_entry = gtk::SearchEntry::builder()
            .placeholder_text("Type '?' for help")
            .build();
        search_entry.add_css_class("launcher-entry");

        let results = ResultList::new();

        let menu = gio::Menu::new();
        menu.append(Some("Preferences"), Some("win.preferences"));
        menu.append(Some("About"), Some("win.about"));
        menu.append(Some("Close"), Some("win.close"));

        let menu_button = gtk::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .tooltip_text("Main Menu")
            .menu_model(&menu)
            .direction(gtk::ArrowType::Up)
            .halign(gtk::Align::End)
            .focus_on_click(false)
            .margin_start(8)
            .margin_end(8)
            .margin_bottom(8)
            .css_classes(["flat", "square"])
            .build();

        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .css_classes(["launcher-panel"])
            .build();
        container.append(&search_entry);
        container.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        container.append(results.widget());
        container.append(&menu_button);
        WindowWidgets {
            container,
            search_entry,
            results,
        }
    }
}
