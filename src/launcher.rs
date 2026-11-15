use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use adw::gio::prelude::*;

use crate::search::{SearchState, Update};
use crate::service::{self, Service};
use crate::views::window::{self, Window};

use pop_launcher::{Request, Response, SearchResult};

pub struct Launcher {
    window: Window,
    service: Service,
    state: RefCell<SearchState>,
}

impl Launcher {
    pub fn new(window: &Window, service: Service) -> Rc<Self> {
        let this = Rc::new(Self {
            window: window.clone(),
            service,
            state: RefCell::default(),
        });
        this.window.listen({
            let this = this.clone();
            move |event| this.handle_window(event)
        });
        this.service.listen({
            let this = this.clone();
            move |event| this.handle_service(event)
        });
        this
    }

    pub fn shutdown(&self) {
        self.service.send(&Request::Exit);
    }

    fn handle_window(&self, event: window::Event) {
        let request = match event {
            window::Event::QueryChanged(text) => self.state.borrow_mut().text_changed(text),
            window::Event::ResultActivated(index) => self.state.borrow().activate(index),
        };
        if let Some(request) = request {
            self.service.send(&request);
        }
    }

    fn handle_service(&self, event: service::Event) {
        match event {
            service::Event::Response(response) => self.handle_response(response),
            service::Event::Closed => self.window.close(),
        }
    }

    fn handle_response(&self, response: Response) {
        match response {
            Response::Update(items) => self.handle_update(&items),
            Response::Fill(text) => self.window.set_query(&text),
            Response::DesktopEntry { path, .. } => self.launch(&path),
            Response::Close => self.window.close(),
            other @ Response::Context { .. } => eprintln!("ignoring response: {other:?}"),
        }
    }

    fn handle_update(&self, items: &[SearchResult]) {
        let update = self.state.borrow_mut().results_arrived(items);
        match update {
            Update::Stale(search) => self.service.send(&search),
            Update::Show => self.window.show_results(items),
        }
    }

    fn launch(&self, path: &Path) {
        let Some(app) = gio_unix::DesktopAppInfo::from_filename(path) else {
            eprintln!("not a valid desktop entry: {}", path.display());
            return;
        };
        match app.launch(&[], Some(&self.window.launch_context())) {
            Ok(()) => self.window.close(),
            Err(error) => eprintln!("failed to launch {}: {error}", path.display()),
        }
    }
}
