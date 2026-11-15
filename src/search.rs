use pop_launcher::{Request, SearchResult};

#[derive(Debug, Default)]
pub struct SearchState {
    searching: bool,
    /// The newest text typed while a search was running.
    queued: Option<String>,
    /// The result ID of each row, by row index.
    ids: Vec<u32>,
}

#[derive(Debug)]
pub enum Update {
    /// Newer text was typed meanwhile
    Stale(Request),
    /// Show the results.
    Show,
}

impl SearchState {
    /// The text changed. Returns the search to send, unless one is already running.
    #[must_use]
    pub fn text_changed(&mut self, text: String) -> Option<Request> {
        if self.searching {
            self.queued = Some(text);
            None
        } else {
            self.searching = true;
            Some(Request::Search(text))
        }
    }

    /// Returns the request that activates the row at `index`.
    #[must_use]
    pub fn activate(&self, index: usize) -> Option<Request> {
        // During a search, the IDs on the displayed rows no longer mean anything to the service.
        if self.searching {
            return None;
        }
        self.ids.get(index).copied().map(Request::Activate)
    }

    /// The results of the running search arrived.
    #[must_use]
    pub fn results_arrived(&mut self, items: &[SearchResult]) -> Update {
        // These results are already out of date, so skip straight to the newest text.
        if let Some(text) = self.queued.take() {
            return Update::Stale(Request::Search(text));
        }
        self.searching = false;
        self.ids = items.iter().map(|item| item.id).collect();
        Update::Show
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn results(ids: &[u32]) -> Vec<SearchResult> {
        ids.iter()
            .map(|&id| SearchResult {
                id,
                name: String::new(),
                description: String::new(),
                icon: None,
                category_icon: None,
                window: None,
            })
            .collect()
    }

    /// A state that has finished a search and shows a row for each of `ids`.
    fn showing(ids: &[u32]) -> SearchState {
        let mut state = SearchState::default();
        let _ = state.text_changed("fire".into());
        let _ = state.results_arrived(&results(ids));
        state
    }

    #[test]
    fn the_first_text_is_searched_immediately() {
        let mut state = SearchState::default();
        assert!(matches!(
            state.text_changed("f".into()),
            Some(Request::Search(text)) if text == "f"
        ));
    }

    #[test]
    fn text_typed_during_a_search_is_not_sent() {
        let mut state = SearchState::default();
        let _ = state.text_changed("f".into());
        assert!(state.text_changed("fi".into()).is_none());
        assert!(state.text_changed("fir".into()).is_none());
    }

    #[test]
    fn stale_results_are_replaced_by_a_search_for_the_newest_text() {
        let mut state = SearchState::default();
        let _ = state.text_changed("f".into());
        let _ = state.text_changed("fi".into());
        let _ = state.text_changed("fir".into());

        assert!(matches!(
            state.results_arrived(&results(&[1])),
            Update::Stale(Request::Search(text)) if text == "fir"
        ));
        // That search is running now, so typing queues again.
        assert!(state.text_changed("fire".into()).is_none());
    }

    #[test]
    fn results_are_shown_when_nothing_newer_was_typed() {
        let mut state = SearchState::default();
        let _ = state.text_changed("f".into());
        assert!(matches!(
            state.results_arrived(&results(&[1])),
            Update::Show
        ));
        // The search is over, so the next text is sent straight away.
        assert!(state.text_changed("fi".into()).is_some());
    }

    #[test]
    fn activating_a_row_sends_its_id() {
        let state = showing(&[7, 8, 9]);
        assert!(matches!(state.activate(1), Some(Request::Activate(8))));
    }

    #[test]
    fn activating_during_a_search_is_ignored() {
        let mut state = showing(&[7]);
        let _ = state.text_changed("fi".into());
        assert!(state.activate(0).is_none());

        // The new results bring new IDs, which can be activated again.
        let _ = state.results_arrived(&results(&[3]));
        assert!(matches!(state.activate(0), Some(Request::Activate(3))));
    }

    #[test]
    fn a_missing_row_activates_nothing() {
        assert!(showing(&[7]).activate(1).is_none());
        assert!(showing(&[]).activate(0).is_none());
    }
}
