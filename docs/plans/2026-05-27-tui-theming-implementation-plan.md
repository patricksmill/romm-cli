# TUI Theming Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add preset TUI themes via ratatui-themekit, persisted in config, selectable from a new Settings Appearance tab.

**Architecture:** `Config.theme` stores a theme ID string; `App` resolves it to `Box<dyn Theme>` at startup; `RommStyles` maps app roles to theme semantic slots; all TUI render paths take `&RommStyles` instead of hardcoded `Color::`.

**Tech Stack:** Rust, ratatui 0.30, ratatui-themekit 0.6 (serde feature), existing Settings tab framework.

**Design reference:** [2026-05-27-tui-theming-design.md](./2026-05-27-tui-theming-design.md)

---

### Task 1: Add dependency and config field

**Files:**

- Modify: `Cargo.toml`
- Modify: `src/config.rs`
- Test: extend existing config tests or add `src/config.rs` `#[cfg(test)]` block

**Step 1: Write failing test**

```rust
#[test]
fn config_theme_defaults_to_terminal() {
    let cfg: Config = serde_json::from_str(r#"{"base_url":"http://x","download_dir":"/tmp"}"#).unwrap();
    assert_eq!(cfg.theme, "terminal");
}

#[test]
fn config_theme_round_trip() {
    let json = r#"{"base_url":"http://x","download_dir":"/tmp","theme":"dracula"}"#;
    let cfg: Config = serde_json::from_str(json).unwrap();
    assert_eq!(cfg.theme, "dracula");
}
```

**Step 2: Run test**

Run: `cargo test config_theme --features tui`  
Expected: FAIL — `theme` field missing on `Config`

**Step 3: Implement**

- Add `ratatui-themekit = { version = "0.6", optional = true, features = ["serde"] }` under `[dependencies]`
- Add to `tui` feature: `"dep:ratatui-themekit"`
- Add to `Config`:

```rust
/// TUI color theme ID (see ratatui-themekit `available_theme_ids`).
#[serde(default = "default_theme_id")]
pub theme: String,

fn default_theme_id() -> String {
    "terminal".to_string()
}
```

- In `load_config`, resolve theme from `ROMM_THEME` env then JSON (same pattern as `use_https`).
- Include `theme` in `persist_user_config` saves from Settings.

**Step 4: Run test**

Run: `cargo test config_theme --features tui`  
Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml src/config.rs
git commit -m "feat(config): add persisted TUI theme field"
```

---

### Task 2: RommStyles module and App wiring

**Files:**

- Create: `src/tui/theme.rs`
- Modify: `src/tui/mod.rs`
- Modify: `src/tui/app/mod.rs`
- Modify: `src/tui/app/render.rs` (pass styles into screens — signature changes start here)

**Step 1: Write failing test**

```rust
// src/tui/theme.rs
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui_themekit::{resolve_theme, TerminalNative};

    #[test]
    fn resolve_unknown_falls_back_to_terminal() {
        let theme = resolve_theme_or_default("not-a-theme");
        assert_eq!(theme.id(), "terminal");
    }
}
```

**Step 2: Run test**

Run: `cargo test tui::theme --features tui`  
Expected: FAIL — module missing

**Step 3: Implement**

`src/tui/theme.rs`:

```rust
use ratatui::style::{Modifier, Style};
use ratatui_themekit::{resolve_theme, Theme, TerminalNative};

pub fn resolve_theme_or_default(id: &str) -> Box<dyn Theme> {
    let t = resolve_theme(id);
    if t.id() == "terminal" && id != "terminal" && id != "no-color" {
        // resolve_theme already falls back; log if input was wrong
        tracing::warn!(theme = id, "unknown theme ID, using terminal");
    }
    Box::new(/* store resolved theme — use a wrapper or match on id */)
}
```

Implement `RommStyles<'a>` with methods from design doc (`selection`, `label`, `success`, `error`, `warning`, `muted`, `primary_text`, `border_focus`, `footer_hint`).

On `App`:

```rust
pub struct App {
    // ...
    theme: Box<dyn Theme>,
}

impl App {
    pub fn styles(&self) -> RommStyles<'_> {
        RommStyles::new(self.theme.as_ref())
    }
}
```

Initialize `theme` in `App::new` from `config.theme`.

**Step 4: Run test**

Run: `cargo test tui::theme --features tui`  
Expected: PASS

**Step 5: Commit**

```bash
git add src/tui/theme.rs src/tui/mod.rs src/tui/app/mod.rs
git commit -m "feat(tui): add RommStyles theme wrapper on App"
```

---

### Task 3: Settings Appearance tab

**Files:**

- Modify: `src/tui/screens/settings/types.rs`
- Modify: `src/tui/screens/settings/state.rs`
- Modify: `src/tui/screens/settings/render.rs`
- Modify: `src/tui/app/handlers/settings.rs`
- Modify: `src/tui/screens/settings/tests.rs`

**Step 1: Write failing test**

```rust
#[test]
fn appearance_tab_has_theme_row() {
    let screen = settings_screen_with_defaults();
    screen.selected_tab = SettingsTab::Appearance;
    let rows = screen.visible_rows();
    assert_eq!(rows, vec![SettingsRow::Theme]);
}
```

**Step 2: Run test**

Run: `cargo test appearance_tab --features tui`  
Expected: FAIL

**Step 3: Implement**

- Add `SettingsTab::Appearance` to enum; update `ALL`, `COUNT`, `index`, `title`
- Add `SettingsRow::Theme`
- Add `pub theme_id: String` to `SettingsScreen`; init from `config.theme`
- Add `APPEARANCE_ROWS: [SettingsRow; 1] = [SettingsRow::Theme]`
- Render row: `Theme: {human name}` using resolved theme's `name()`
- Handler: `KeyCode::Left` / `KeyCode::Right` on Theme row cycles `available_theme_ids()`; update `theme_id` and call callback to refresh `App.theme`
- On save (`S`): include `theme: settings.theme_id` in `Config` passed to `persist_user_config`
- Tab bar: insert "Appearance" (suggest after Extras, before Auth/Maint)

**App handler:** pass `&mut App` theme when settings cycles theme for live preview:

```rust
self.theme = resolve_theme_or_default(&settings.theme_id);
```

**Step 4: Run test**

Run: `cargo test tui::screens::settings --features tui`  
Expected: PASS

**Step 5: Commit**

```bash
git add src/tui/screens/settings/ src/tui/app/handlers/settings.rs
git commit -m "feat(tui): add Appearance settings tab with theme picker"
```

---

### Task 4: Migrate global overlays and path picker

**Files:**

- Modify: `src/tui/app/render.rs`
- Modify: `src/tui/path_picker.rs`

**Step 1: Update signatures**

Add `styles: &RommStyles` parameter to `PathPicker::render` and use in `App::render` global overlays (update prompt, errors, keyboard help if styled).

**Step 2: Replace colors**

| Old | New |
|-----|-----|
| `Color::Yellow` (focus) | `styles.selection()` |
| `Color::Green` | `styles.success()` |
| `Color::Red` | `styles.error()` |
| `Color::Cyan` | `styles.label()` |
| `Color::DarkGray` | `styles.muted()` |

**Step 3: Verify**

Run: `cargo build --features tui`  
Expected: compiles (other screens may still use old signatures — fix in Task 5)

**Step 4: Commit**

```bash
git add src/tui/app/render.rs src/tui/path_picker.rs
git commit -m "refactor(tui): theme path picker and global overlays"
```

---

### Task 5: Migrate all screen renderers

**Files:**

- Modify: `src/tui/screens/main_menu.rs`
- Modify: `src/tui/screens/library_browse/render.rs`
- Modify: `src/tui/screens/search.rs`
- Modify: `src/tui/screens/game_detail/render.rs`
- Modify: `src/tui/screens/download.rs`
- Modify: `src/tui/screens/extras_picker.rs`
- Modify: `src/tui/screens/connected_splash.rs`
- Modify: `src/tui/screens/setup_wizard/render.rs`, `layout.rs`
- Modify: `src/tui/screens/settings/render.rs`, `console.rs`
- Modify: `src/tui/app/handlers/setup_wizard.rs`
- Modify: `src/tui/app/background/tasks.rs`
- Modify: `src/tui/app/render.rs` (dispatch with `self.styles()`)

**Step 1:** Add `styles: &RommStyles` to each `render` method; update `App::render` dispatch.

**Step 2:** Replace every TUI `Color::` with semantic `styles.*()`. For `settings.message: Option<(String, Color)>`, change to store a message kind enum (`Success`, `Error`, `Warning`, `Info`) and resolve color at render time from `styles`.

**Step 3: Verify**

Run: `cargo clippy --all-targets --all-features -- -D warnings`  
Run: `cargo test --features tui`  
Expected: PASS, no remaining `Color::` in `src/tui/` except inside `theme.rs`

**Step 4: Commit**

```bash
git add src/tui/
git commit -m "refactor(tui): migrate all screens to RommStyles"
```

---

### Task 6: Documentation

**Files:**

- Modify: `docs/tui.md` (theming section)
- Modify: `README.md` (env table: `ROMM_THEME`)

**Step 1:** Document Appearance tab, available theme IDs, default `terminal`, `NO_COLOR` behavior.

**Step 2: Commit**

```bash
git add docs/tui.md README.md
git commit -m "docs: document TUI theme settings"
```

---

### Task 7: Final verification

**Run:**

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --features tui
```

**Manual smoke checklist:**

- [ ] Main menu list highlight uses theme accent
- [ ] Library browse, search, game detail readable in Dracula and Nord
- [ ] Settings Appearance tab cycles themes with live preview
- [ ] Save persists theme; restart loads saved theme
- [ ] `NO_COLOR=1` yields uncolored output

**Commit if any fixups needed.**

---

**Plan complete and saved to `docs/plans/2026-05-27-tui-theming-implementation-plan.md`.**

**Execution options:**

1. **Subagent-Driven (this session)** — dispatch fresh subagent per task, review between tasks  
2. **Parallel Session (separate)** — open new session with executing-plans and checkpoints  

**Which approach?**
