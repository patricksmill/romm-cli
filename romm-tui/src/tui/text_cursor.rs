//! UTF-8-safe byte cursor helpers for editable single-line text.

pub(crate) fn clamp_to_char_boundary(s: &str, pos: usize) -> usize {
    let mut pos = pos.min(s.len());
    while pos > 0 && !s.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

pub(crate) fn previous_char_boundary(s: &str, pos: usize) -> usize {
    let pos = clamp_to_char_boundary(s, pos);
    if pos == 0 {
        return 0;
    }
    s[..pos]
        .char_indices()
        .next_back()
        .map(|(idx, _)| idx)
        .unwrap_or(0)
}

pub(crate) fn next_char_boundary(s: &str, pos: usize) -> usize {
    let pos = clamp_to_char_boundary(s, pos);
    if pos >= s.len() {
        return s.len();
    }
    pos + s[pos..].chars().next().map(char::len_utf8).unwrap_or(0)
}

pub(crate) fn insert_char(s: &mut String, cursor: &mut usize, c: char) {
    let pos = clamp_to_char_boundary(s, *cursor);
    s.insert(pos, c);
    *cursor = pos + c.len_utf8();
}

pub(crate) fn insert_str(s: &mut String, cursor: &mut usize, text: &str) {
    let pos = clamp_to_char_boundary(s, *cursor);
    s.insert_str(pos, text);
    *cursor = pos + text.len();
}

pub(crate) fn delete_previous_char(s: &mut String, cursor: &mut usize) {
    let pos = clamp_to_char_boundary(s, *cursor);
    if pos == 0 {
        *cursor = 0;
        return;
    }
    let prev = previous_char_boundary(s, pos);
    s.replace_range(prev..pos, "");
    *cursor = prev;
}

pub(crate) fn move_left(s: &str, cursor: &mut usize) {
    *cursor = previous_char_boundary(s, *cursor);
}

pub(crate) fn move_right(s: &str, cursor: &mut usize) {
    *cursor = next_char_boundary(s, *cursor);
}

pub(crate) fn split_at_cursor(s: &str, cursor: usize) -> (&str, &str) {
    let pos = clamp_to_char_boundary(s, cursor);
    s.split_at(pos)
}

pub(crate) fn char_column(s: &str, cursor: usize) -> usize {
    let pos = clamp_to_char_boundary(s, cursor);
    s[..pos].chars().count()
}
