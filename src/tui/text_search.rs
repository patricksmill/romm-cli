//! Shared filter/jump search state and string matching for library list and ROM panes.

use unicode_normalization::UnicodeNormalization;

/// Filter vs jump mode (same semantics for consoles/collections list and games table).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibrarySearchMode {
    Filter,
    Jump,
}

/// Typing/filter state for one pane (list or ROMs).
#[derive(Debug, Clone)]
pub struct SearchState {
    pub mode: Option<LibrarySearchMode>,
    pub query: String,
    pub normalized_query: String,
    pub filter_browsing: bool,
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            mode: None,
            query: String::new(),
            normalized_query: String::new(),
            filter_browsing: false,
        }
    }

    pub fn clear(&mut self) {
        self.mode = None;
        self.filter_browsing = false;
        self.query.clear();
        self.normalized_query.clear();
    }

    pub fn enter(&mut self, mode: LibrarySearchMode) {
        self.mode = Some(mode);
        self.filter_browsing = false;
        self.query.clear();
        self.normalized_query.clear();
    }

    pub fn add_char(&mut self, c: char) {
        self.query.push(c);
        self.normalized_query = normalize_label(&self.query);
    }

    pub fn delete_char(&mut self) {
        self.query.pop();
        self.normalized_query = normalize_label(&self.query);
    }

    /// True when the ROM/list table should show only rows matching the query.
    pub fn filter_active(&self) -> bool {
        (self.mode == Some(LibrarySearchMode::Filter) || self.filter_browsing)
            && !self.normalized_query.is_empty()
    }

    /// Enter pressed while in filter typing mode: close bar; optionally enter filter-browse.
    pub fn commit_filter_bar(&mut self) {
        if self.mode == Some(LibrarySearchMode::Filter) {
            self.mode = None;
            self.filter_browsing = !self.query.trim().is_empty();
        } else {
            self.mode = None;
        }
    }
}

/// Strip diacritics and lowercase for substring search.
pub fn normalize_label(s: &str) -> String {
    s.nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .collect::<String>()
        .to_lowercase()
}

/// Source indices into `labels` where the normalized label contains `normalized_query`.
pub fn filter_source_indices(labels: &[String], normalized_query: &str) -> Vec<usize> {
    if normalized_query.is_empty() {
        return (0..labels.len()).collect();
    }
    labels
        .iter()
        .enumerate()
        .filter(|(_, lab)| normalize_label(lab).contains(normalized_query))
        .map(|(i, _)| i)
        .collect()
}

/// Next matching row index in the full `labels` slice (jump search), wrapping.
pub fn jump_next_index(
    labels: &[String],
    selected_source: usize,
    normalized_query: &str,
    next: bool,
) -> Option<usize> {
    if normalized_query.is_empty() || labels.is_empty() {
        return None;
    }
    let len = labels.len();
    let start = if next {
        (selected_source + 1) % len
    } else {
        selected_source
    };
    for i in 0..len {
        let idx = (start + i) % len;
        if normalize_label(&labels[idx]).contains(normalized_query) {
            return Some(idx);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_source_indices_finds_substrings() {
        let labels = vec![
            "Alpha".to_string(),
            "Beta".to_string(),
            "Alphabet".to_string(),
        ];
        let q = normalize_label("alp");
        let idx = filter_source_indices(&labels, &q);
        assert_eq!(idx, vec![0, 2]);
    }

    #[test]
    fn jump_next_index_wraps() {
        let labels = vec!["a".into(), "bb".into(), "ab".into()];
        let q = normalize_label("b");
        assert_eq!(jump_next_index(&labels, 0, &q, false), Some(1));
        assert_eq!(jump_next_index(&labels, 1, &q, true), Some(2));
        assert_eq!(jump_next_index(&labels, 2, &q, true), Some(1));
    }
}
