//! Shared filesystem path browser for TUI (directory or file pick).

use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

/// Whether the user must pick a directory or a regular file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathPickerMode {
    Directory,
    File,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathPickerFocus {
    PathBar,
    List,
}

#[derive(Debug, Clone)]
struct ListEntry {
    label: String,
    /// `None` means the synthetic "use this folder" row (directory mode only).
    path: Option<PathBuf>,
    is_dir: bool,
    is_use_here: bool,
}

/// Result of handling a key press.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathPickerEvent {
    /// No completion; keep browsing.
    None,
    /// User chose a path (caller should validate with app rules if needed).
    Confirmed(PathBuf),
}

#[derive(Debug, Clone)]
pub struct PathPicker {
    mode: PathPickerMode,
    pub focus: PathPickerFocus,
    /// Editable path string (may be relative or absolute).
    pub path_text: String,
    pub path_cursor: usize,
    /// Directory whose contents are listed.
    browse_dir: PathBuf,
    entries: Vec<ListEntry>,
    list_state: ListState,
    io_error: Option<String>,
}

impl PathPicker {
    pub fn new(mode: PathPickerMode, initial_path: &str) -> Self {
        let path_text = initial_path.to_string();
        let path_cursor = path_text.len();
        let browse_dir = resolve_browse_directory(&path_text);
        let mut s = Self {
            mode,
            focus: PathPickerFocus::PathBar,
            path_text,
            path_cursor,
            browse_dir,
            entries: Vec::new(),
            list_state: ListState::default(),
            io_error: None,
        };
        s.refresh_entries();
        s
    }

    /// Current path string shown in the bar (trimmed for persistence).
    pub fn path_trimmed(&self) -> String {
        self.path_text.trim().to_string()
    }

    pub fn set_path_text(&mut self, s: String) {
        self.path_text = s;
        self.path_cursor = self.path_text.len();
        self.sync_browse_from_path_text();
        self.refresh_entries();
    }

    fn sync_browse_from_path_text(&mut self) {
        self.browse_dir = resolve_browse_directory(&self.path_text);
        self.io_error = None;
    }

    fn refresh_entries(&mut self) {
        self.entries.clear();
        let name_filter = entry_name_prefix_filter(&self.path_text, &self.browse_dir);
        if self.mode == PathPickerMode::Directory {
            self.entries.push(ListEntry {
                label: "< Use this folder >".to_string(),
                path: None,
                is_dir: true,
                is_use_here: true,
            });
        }
        match std::fs::read_dir(&self.browse_dir) {
            Ok(rd) => {
                self.io_error = None;
                let mut dirs: Vec<PathBuf> = Vec::new();
                let mut files: Vec<PathBuf> = Vec::new();
                for e in rd.flatten() {
                    let p = e.path();
                    let Ok(ft) = e.file_type() else {
                        continue;
                    };
                    let fname = file_name_display(&p);
                    if let Some(ref pref) = name_filter {
                        if !fname.to_lowercase().starts_with(pref) {
                            continue;
                        }
                    }
                    if ft.is_dir() {
                        dirs.push(p);
                    } else if ft.is_file() {
                        files.push(p);
                    }
                }
                dirs.sort_by(|a, b| cmp_path_names(a, b));
                files.sort_by(|a, b| cmp_path_names(a, b));
                if self.browse_dir.parent().is_some() {
                    self.entries.push(ListEntry {
                        label: "..".to_string(),
                        path: Some(self.browse_dir.parent().unwrap().to_path_buf()),
                        is_dir: true,
                        is_use_here: false,
                    });
                }
                for p in dirs {
                    let name = file_name_display(&p);
                    self.entries.push(ListEntry {
                        label: format!("[{name}]"),
                        path: Some(p),
                        is_dir: true,
                        is_use_here: false,
                    });
                }
                if self.mode == PathPickerMode::File {
                    for p in files {
                        let name = file_name_display(&p);
                        self.entries.push(ListEntry {
                            label: name,
                            path: Some(p),
                            is_dir: false,
                            is_use_here: false,
                        });
                    }
                } else {
                    for p in files {
                        let name = file_name_display(&p);
                        self.entries.push(ListEntry {
                            label: format!("{name}  (file)"),
                            path: Some(p),
                            is_dir: false,
                            is_use_here: false,
                        });
                    }
                }
            }
            Err(e) => {
                self.io_error = Some(format!("{}", e));
            }
        }
        let n = self.entries.len();
        let sel = self
            .list_state
            .selected()
            .unwrap_or(0)
            .min(n.saturating_sub(1));
        self.list_state.select(Some(sel));
    }

    fn confirm_browse_dir(&self) -> PathPickerEvent {
        PathPickerEvent::Confirmed(self.browse_dir.clone())
    }

    /// Directory mode: confirm the path typed in the bar (may not exist yet; caller runs
    /// `create_dir_all` via validation). Empty bar falls back to the listing directory.
    fn confirm_typed_directory_path(&self) -> PathPickerEvent {
        let t = self.path_text.trim();
        if t.is_empty() {
            return self.confirm_browse_dir();
        }
        PathPickerEvent::Confirmed(resolve_path_for_confirm(t))
    }

    fn try_confirm_path_text(&self) -> Option<PathPickerEvent> {
        let t = self.path_text.trim();
        if t.is_empty() {
            return None;
        }
        let p = resolve_path_for_confirm(t);
        match self.mode {
            PathPickerMode::Directory => {
                if p.exists() && !p.is_dir() {
                    return None;
                }
                Some(PathPickerEvent::Confirmed(p))
            }
            PathPickerMode::File => {
                let meta = std::fs::metadata(&p).ok()?;
                if meta.is_file() {
                    Some(PathPickerEvent::Confirmed(p))
                } else {
                    None
                }
            }
        }
    }

    fn enter_list_selection(&mut self) -> PathPickerEvent {
        let idx = self.list_state.selected().unwrap_or(0);
        let Some(entry) = self.entries.get(idx) else {
            return PathPickerEvent::None;
        };
        if entry.is_use_here {
            return self.confirm_browse_dir();
        }
        let Some(ref target) = entry.path else {
            return PathPickerEvent::None;
        };
        if entry.is_dir {
            self.browse_dir = target.clone();
            self.path_text = target.display().to_string();
            self.path_cursor = self.path_text.len();
            self.refresh_entries();
            self.list_state.select(Some(0));
            PathPickerEvent::None
        } else if self.mode == PathPickerMode::File {
            PathPickerEvent::Confirmed(target.clone())
        } else {
            PathPickerEvent::None
        }
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> PathPickerEvent {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        // Ctrl+Enter: confirm typed directory path (dir mode) or existing file path (file mode).
        if ctrl && key.code == KeyCode::Enter {
            if self.mode == PathPickerMode::Directory {
                return self.confirm_typed_directory_path();
            }
            if let Some(ev) = self.try_confirm_path_text() {
                return ev;
            }
            return PathPickerEvent::None;
        }

        match key.code {
            KeyCode::Tab => {
                self.focus = match self.focus {
                    PathPickerFocus::PathBar => PathPickerFocus::List,
                    PathPickerFocus::List => PathPickerFocus::PathBar,
                };
                PathPickerEvent::None
            }
            KeyCode::Esc => PathPickerEvent::None,
            _ => match self.focus {
                PathPickerFocus::PathBar => self.handle_path_bar_key(key),
                PathPickerFocus::List => self.handle_list_key(key),
            },
        }
    }

    fn handle_path_bar_key(&mut self, key: &KeyEvent) -> PathPickerEvent {
        match key.code {
            KeyCode::Enter => {
                if let Some(ev) = self.try_confirm_path_text() {
                    return ev;
                }
                PathPickerEvent::None
            }
            KeyCode::Char(c) => {
                let pos = self.path_cursor.min(self.path_text.len());
                self.path_text.insert(pos, c);
                self.path_cursor = pos + c.len_utf8();
                self.sync_browse_from_path_text();
                self.refresh_entries();
                PathPickerEvent::None
            }
            KeyCode::Backspace => {
                if self.path_cursor > 0 && self.path_cursor <= self.path_text.len() {
                    let prev = self.path_text[..self.path_cursor]
                        .chars()
                        .next_back()
                        .map(|c| c.len_utf8())
                        .unwrap_or(1);
                    let start = self.path_cursor - prev;
                    self.path_text.replace_range(start..self.path_cursor, "");
                    self.path_cursor = start;
                    self.sync_browse_from_path_text();
                    self.refresh_entries();
                }
                PathPickerEvent::None
            }
            KeyCode::Left => {
                if self.path_cursor > 0 {
                    let prev = self.path_text[..self.path_cursor]
                        .chars()
                        .next_back()
                        .map(|c| c.len_utf8())
                        .unwrap_or(1);
                    self.path_cursor -= prev;
                }
                PathPickerEvent::None
            }
            KeyCode::Right => {
                if self.path_cursor < self.path_text.len() {
                    let next = self.path_text[self.path_cursor..]
                        .chars()
                        .next()
                        .map(|c| c.len_utf8())
                        .unwrap_or(1);
                    self.path_cursor += next;
                }
                PathPickerEvent::None
            }
            KeyCode::Up | KeyCode::Down => {
                // Move focus to the list without consuming the arrow as a list move — otherwise
                // PathBar+Up lands on row 0 and immediately "moves up" nowhere.
                self.focus = PathPickerFocus::List;
                PathPickerEvent::None
            }
            _ => PathPickerEvent::None,
        }
    }

    fn handle_list_key(&mut self, key: &KeyEvent) -> PathPickerEvent {
        let n = self.entries.len();
        if n == 0 {
            return PathPickerEvent::None;
        }
        let sel = self.list_state.selected().unwrap_or(0);
        match key.code {
            KeyCode::Enter => self.enter_list_selection(),
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
                if sel == 0 {
                    self.focus = PathPickerFocus::PathBar;
                } else {
                    self.list_state.select(Some(sel - 1));
                }
                PathPickerEvent::None
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                let i = (sel + 1).min(n - 1);
                self.list_state.select(Some(i));
                PathPickerEvent::None
            }
            KeyCode::Home => {
                self.list_state.select(Some(0));
                PathPickerEvent::None
            }
            KeyCode::End => {
                self.list_state.select(Some(n - 1));
                PathPickerEvent::None
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.focus = PathPickerFocus::PathBar;
                PathPickerEvent::None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.focus = PathPickerFocus::PathBar;
                PathPickerEvent::None
            }
            KeyCode::Char(c) if !matches!(c, 'j' | 'k' | 'h' | 'l' | 'J' | 'K' | 'H' | 'L') => {
                self.focus = PathPickerFocus::PathBar;
                self.handle_path_bar_key(key)
            }
            _ => PathPickerEvent::None,
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, title: &str, footer_hint: &str) {
        let block = Block::default().title(title).borders(Borders::ALL);
        let inner = block.inner(area);
        f.render_widget(block, area);

        let path_h = if self.io_error.is_some() { 2u16 } else { 1u16 };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(path_h),
                Constraint::Min(3),
                Constraint::Length(2),
            ])
            .split(inner);

        let path_prefix = match self.focus {
            PathPickerFocus::PathBar => "▶ ",
            PathPickerFocus::List => "  ",
        };
        let before: String = self.path_text.chars().take(self.path_cursor).collect();
        let after: String = self.path_text.chars().skip(self.path_cursor).collect();
        let path_style = if self.focus == PathPickerFocus::PathBar {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let path_line = format!("{path_prefix}{before}▏{after}");
        let path_row = Rect {
            x: chunks[0].x,
            y: chunks[0].y,
            width: chunks[0].width,
            height: 1,
        };
        f.render_widget(Paragraph::new(path_line).style(path_style), path_row);

        if let Some(ref err) = self.io_error {
            f.render_widget(
                Paragraph::new(format!("⚠ {err}")).style(Style::default().fg(Color::Red)),
                Rect {
                    x: chunks[0].x,
                    y: chunks[0].y + 1,
                    width: chunks[0].width,
                    height: 1,
                },
            );
        }

        let list_block = Block::default().borders(Borders::ALL).border_style(
            if self.focus == PathPickerFocus::List {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            },
        );
        let list_inner = list_block.inner(chunks[1]);
        f.render_widget(list_block, chunks[1]);

        let items: Vec<ListItem> = self
            .entries
            .iter()
            .map(|e| {
                let style = if e.is_use_here {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else if !e.is_dir {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };
                ListItem::new(e.label.clone()).style(style)
            })
            .collect();
        let list = List::new(items).highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
        f.render_stateful_widget(list, list_inner, &mut self.list_state);

        f.render_widget(
            Paragraph::new(footer_hint).style(Style::default().fg(Color::Cyan)),
            chunks[2],
        );
    }

    /// Terminal cursor inside path bar (relative to `area` passed to `render`).
    pub fn cursor_position(&self, area: Rect, title: &str) -> Option<(u16, u16)> {
        if self.focus != PathPickerFocus::PathBar {
            return None;
        }
        let block = Block::default().title(title).borders(Borders::ALL);
        let inner = block.inner(area);
        let path_h = if self.io_error.is_some() { 2u16 } else { 1u16 };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(path_h),
                Constraint::Min(3),
                Constraint::Length(2),
            ])
            .split(inner);
        let path_prefix_chars = 2u16; // "▶ "
        let byte_before = self.path_cursor.min(self.path_text.len());
        let path_before: String = self.path_text.chars().take(byte_before).collect();
        let x = chunks[0].x + path_prefix_chars + path_before.chars().count() as u16;
        let y = chunks[0].y;
        Some((x.min(chunks[0].x + chunks[0].width.saturating_sub(1)), y))
    }
}

/// If the user is typing past `browse_dir` into a new path segment, return that segment's
/// prefix so the listing can narrow to matching names (case-insensitive).
fn entry_name_prefix_filter(path_text: &str, browse_dir: &Path) -> Option<String> {
    let trimmed = path_text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let abs = typed_absolute_path(trimmed);
    if abs.as_os_str().is_empty() {
        return None;
    }
    let rel = strip_prefix_path(&abs, browse_dir)?;
    let mut it = rel.components();
    let first = it.next()?;
    let s = first.as_os_str().to_string_lossy();
    if s.is_empty() {
        None
    } else {
        Some(s.to_lowercase())
    }
}

fn typed_absolute_path(trimmed: &str) -> PathBuf {
    let p = PathBuf::from(trimmed);
    if p.is_relative() {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(p)
    } else {
        p
    }
}

fn strip_prefix_path(full: &Path, prefix: &Path) -> Option<PathBuf> {
    let mut full_c = full.components();
    let prefix_c: Vec<_> = prefix.components().collect();
    if prefix_c.is_empty() {
        return Some(PathBuf::from_iter(full_c));
    }
    for c in &prefix_c {
        if full_c.next()? != *c {
            return None;
        }
    }
    Some(PathBuf::from_iter(full_c))
}

fn file_name_display(p: &Path) -> String {
    p.file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn cmp_path_names(a: &Path, b: &Path) -> std::cmp::Ordering {
    let sa = a.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let sb = b.file_name().and_then(|s| s.to_str()).unwrap_or("");
    sa.to_lowercase().cmp(&sb.to_lowercase())
}

/// Resolve a directory to list for the given user input.
pub fn resolve_browse_directory(path_input: &str) -> PathBuf {
    let trimmed = path_input.trim();
    if trimmed.is_empty() {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    }
    let p = PathBuf::from(trimmed);
    let p = if p.is_relative() {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(&p)
    } else {
        p
    };

    if let Ok(meta) = std::fs::metadata(&p) {
        if meta.is_dir() {
            return p;
        }
        if meta.is_file() {
            return p.parent().map(Path::to_path_buf).unwrap_or(p);
        }
    }

    let mut cur = p.clone();
    loop {
        if let Ok(m) = std::fs::metadata(&cur) {
            if m.is_dir() {
                return cur;
            }
        }
        if let Some(parent) = cur.parent() {
            if parent == cur {
                break;
            }
            cur = parent.to_path_buf();
        } else {
            break;
        }
    }

    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn resolve_path_for_confirm(trimmed: &str) -> PathBuf {
    let p = PathBuf::from(trimmed);
    if p.is_relative() {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(p)
    } else {
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_browse_empty_uses_home_or_dot() {
        let r = resolve_browse_directory("");
        assert!(r.exists() || r == std::path::Path::new("."));
    }

    #[test]
    fn resolve_browse_existing_dir() {
        let tmp = std::env::temp_dir();
        let r = resolve_browse_directory(tmp.to_str().unwrap());
        assert_eq!(r, tmp);
    }

    #[test]
    fn resolve_browse_nonexistent_child_lists_parent() {
        let tmp = std::env::temp_dir();
        let bogus = tmp.join("romm_path_picker_nonexistent_child_xyz");
        let r = resolve_browse_directory(bogus.to_str().unwrap());
        assert_eq!(r, tmp);
    }

    #[test]
    fn entry_name_prefix_filter_incomplete_segment() {
        let tmp = std::env::temp_dir();
        let browse = resolve_browse_directory(tmp.to_str().unwrap());
        let tail = tmp.join("romm_filter_test_nonexistent_abc");
        let typed = tail.to_string_lossy();
        let f = entry_name_prefix_filter(&typed, &browse);
        assert_eq!(f.as_deref(), Some("romm_filter_test_nonexistent_abc"));
    }
}
