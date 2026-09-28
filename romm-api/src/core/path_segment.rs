//! A single, sanitized path component for joining untrusted names onto local directories.

use std::fmt;
use std::path::{Component, Path, PathBuf};

/// A single path component guaranteed to be `Component::Normal`:
/// no separators, not `.`/`..`, not empty.
///
/// The only constructor is [`PathSegment::sanitize`], so joining one onto a
/// directory can never escape that directory.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathSegment(String);

impl PathSegment {
    /// Sanitize `raw`; use `fallback` (also sanitized) if nothing usable remains.
    pub fn sanitize(raw: &str, fallback: &str) -> Self {
        let s = clean(raw)
            .or_else(|| clean(fallback))
            .unwrap_or_else(|| "_".to_string());
        debug_assert!(
            matches!(
                Path::new(&s).components().collect::<Vec<_>>().as_slice(),
                [Component::Normal(_)]
            ),
            "PathSegment {s:?} is not a single normal component"
        );
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<Path> for PathSegment {
    fn as_ref(&self) -> &Path {
        Path::new(&self.0)
    }
}

impl PartialEq<&str> for PathSegment {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl fmt::Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

pub trait JoinSegment {
    fn join_segment(&self, seg: &PathSegment) -> PathBuf;
}

impl JoinSegment for Path {
    fn join_segment(&self, seg: &PathSegment) -> PathBuf {
        self.join(&seg.0)
    }
}

fn clean(raw: &str) -> Option<String> {
    let mapped: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ' ') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = mapped.trim();
    if matches!(trimmed, "." | "..") {
        return Some("_".to_string());
    }
    // Windows silently drops trailing dots and spaces.
    let trimmed = trimmed.trim_end_matches(['.', ' ']);
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_single_normal(seg: &PathSegment) {
        let parent = Path::new("/base/dir");
        let joined = parent.join_segment(seg);
        assert_eq!(joined.parent(), Some(parent), "{seg:?} escaped parent");
        assert!(matches!(
            Path::new(seg.as_str())
                .components()
                .collect::<Vec<_>>()
                .as_slice(),
            [Component::Normal(_)]
        ));
    }

    #[test]
    fn hostile_inputs_stay_inside_parent() {
        for raw in [
            "..",
            ".",
            "",
            "a/b",
            "C:\\x",
            "/etc/passwd",
            "  ..  ",
            "...",
            "name. . ",
            "../../etc",
            "\\\\?\\C:\\windows",
        ] {
            assert_single_normal(&PathSegment::sanitize(raw, "fallback"));
        }
    }

    #[test]
    fn preserves_ordinary_names() {
        assert_eq!(
            PathSegment::sanitize("Mario Kart", "x").as_str(),
            "Mario Kart"
        );
        assert_eq!(
            PathSegment::sanitize("Zelda (USA).xci", "x").as_str(),
            "Zelda _USA_.xci"
        );
    }

    #[test]
    fn dot_components_become_underscore() {
        assert_eq!(PathSegment::sanitize("..", "x").as_str(), "_");
        assert_eq!(PathSegment::sanitize(" . ", "x").as_str(), "_");
    }

    #[test]
    fn trailing_dots_and_spaces_are_trimmed() {
        assert_eq!(PathSegment::sanitize("Game. ", "x").as_str(), "Game");
    }

    #[test]
    fn empty_uses_fallback() {
        assert_eq!(PathSegment::sanitize("", "rom-7").as_str(), "rom-7");
        assert_eq!(PathSegment::sanitize("...", "rom-7").as_str(), "rom-7");
        assert_eq!(PathSegment::sanitize("", "").as_str(), "_");
    }
}
