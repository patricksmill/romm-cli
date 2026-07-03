# Tabbed game detail — implementation plan

> **For Claude:** implement this plan task-by-task.

**Goal:** Split the game detail metadata panel into 3 tabs (Info / Saves / Achievements) to reduce clutter and add scrolling.

**Architecture:** Add `DetailTab` enum to `game_detail/types.rs`, split `render_metadata_panel` into tab bar + per-tab renderers, gate tab-specific keys in the handler.

**Reference:** [2026-07-03-game-detail-tabs-design.md](2026-07-03-game-detail-tabs-design.md)

---

## Task 0: Branch

```bash
git checkout -b feat/game-detail-tabs
```

Baseline checks:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings
cargo test
```

---

## Task 1: Add `DetailTab` enum and state fields

**Files:**
- Modify: `romm-tui/src/tui/screens/game_detail/types.rs`

**Step 1:** Add `DetailTab` enum mirroring `SettingsTab` pattern:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailTab {
    Info,
    Saves,
    Achievements,
}

impl DetailTab {
    pub const ALL: [DetailTab; 3] = [DetailTab::Info, DetailTab::Saves, DetailTab::Achievements];

    pub fn index(self) -> usize {
        match self {
            DetailTab::Info => 0,
            DetailTab::Saves => 1,
            DetailTab::Achievements => 2,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            DetailTab::Info => "Info",
            DetailTab::Saves => "Saves",
            DetailTab::Achievements => "Achievements",
        }
    }
}
```

**Step 2:** Add fields to `GameDetailScreen`:

```rust
pub active_tab: DetailTab,
pub achievement_scroll_offset: usize,
```

**Step 3:** Initialize in `state.rs` `GameDetailScreen::new()`:

```rust
active_tab: DetailTab::Info,
achievement_scroll_offset: 0,
```

**Step 4:** Add tab navigation helpers in `state.rs`:

```rust
pub fn select_tab(&mut self, tab: DetailTab) {
    self.active_tab = tab;
}
```

**Step 5:** Export `DetailTab` from `mod.rs`.

**Step 6: Commit**

```
feat(romm-tui): add DetailTab enum and state fields
```

---

## Task 2: Split render into tab bar + per-tab dispatch

**Files:**
- Modify: `romm-tui/src/tui/screens/game_detail/render.rs`

**Step 1:** In `render_metadata_panel`, split the area into two vertical chunks: tab bar (height 3) + content area.

```rust
let meta_chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(3), Constraint::Min(1)])
    .split(area);
```

**Step 2:** Render tab bar using ratatui `Tabs` widget (same pattern as `settings/render.rs` lines 208-218):

```rust
let titles = DetailTab::ALL
    .iter()
    .map(|tab| Line::from(Span::raw(tab.title())))
    .collect::<Vec<_>>();
let tabs = Tabs::new(titles)
    .select(self.active_tab.index())
    .block(styles.panel_block_untitled())
    .style(styles.muted())
    .highlight_style(styles.selection());
f.render_widget(tabs, meta_chunks[0]);
```

**Step 3:** Dispatch to per-tab render methods:

```rust
match self.active_tab {
    DetailTab::Info => self.render_info_tab(f, meta_chunks[1], styles),
    DetailTab::Saves => self.render_saves_tab(f, meta_chunks[1], styles),
    DetailTab::Achievements => self.render_achievements_tab(f, meta_chunks[1], styles),
}
```

**Step 4:** Extract `render_info_tab` — move the current metadata lines (title through technical + DLC) into this method. Wrap in a `Paragraph` with the `"Info"` block title.

**Step 5:** Create `render_saves_tab` — render the save list with scroll windowing:

```rust
fn render_saves_tab(&self, f: &mut Frame, area: Rect, styles: &RommStyles) {
    let block = styles.panel_block("Saves");
    let inner = block.inner(area);
    let visible_height = inner.height as usize;
    let lines = save_lines(&self.saves_state, self.selected_save_index);
    // Window based on selected_save_index
    let start = self.selected_save_index.saturating_sub(visible_height.saturating_sub(1));
    let windowed: Vec<_> = lines.into_iter().skip(start).take(visible_height).collect();
    let p = Paragraph::new(windowed).block(block).style(styles.text());
    f.render_widget(p, area);
}
```

**Step 6:** Create `render_achievements_tab` — same pattern with `achievement_scroll_offset`:

```rust
fn render_achievements_tab(&self, f: &mut Frame, area: Rect, styles: &RommStyles) {
    let block = styles.panel_block("Achievements");
    let inner = block.inner(area);
    let visible_height = inner.height as usize;
    let lines = achievement_lines(&self.achievements_state);
    let start = self.achievement_scroll_offset;
    let windowed: Vec<_> = lines.into_iter().skip(start).take(visible_height).collect();
    let p = Paragraph::new(windowed).block(block).style(styles.text());
    f.render_widget(p, area);
}
```

**Step 7: Commit**

```
feat(romm-tui): split game detail into tabbed layout
```

---

## Task 3: Remove truncation from save_lines and achievement_lines

**Files:**
- Modify: `romm-tui/src/tui/screens/game_detail/saves.rs`
- Modify: `romm-tui/src/tui/screens/game_detail/achievements.rs`

**Step 1:** In `saves.rs`, remove `.take(8)` so the full save list is returned.

**Step 2:** In `achievements.rs`, remove `.take(8)` and the `"… and N more"` overflow line.

**Step 3:** Update the `achievement_lines_shows_earned_marker` test assertion if the index changes (line `[1]` → line `[1]` should still work since the header is still first).

**Step 4: Commit**

```
refactor(romm-tui): remove hardcoded truncation from saves and achievements
```

---

## Task 4: Gate key handlers by active tab

**Files:**
- Modify: `romm-tui/src/tui/app/handlers/game_detail.rs`

**Step 1:** Add `1`/`2`/`3` key handlers to switch tabs:

```rust
KeyCode::Char('1') => detail.select_tab(DetailTab::Info),
KeyCode::Char('2') => detail.select_tab(DetailTab::Saves),
KeyCode::Char('3') => detail.select_tab(DetailTab::Achievements),
```

**Step 2:** Gate save-specific keys (`j`/`k` for save nav, `u` upload, `D` download) behind `detail.active_tab == DetailTab::Saves`.

**Step 3:** Add `j`/`k` scrolling for the Achievements tab:

```rust
KeyCode::Up | KeyCode::Char('k') if detail.active_tab == DetailTab::Achievements => {
    detail.achievement_scroll_offset = detail.achievement_scroll_offset.saturating_sub(1);
}
KeyCode::Down | KeyCode::Char('j') if detail.active_tab == DetailTab::Achievements => {
    // Clamp to max scroll
    detail.achievement_scroll_offset += 1;
}
```

**Step 4:** Global keys (`Esc`, `Enter` for download, `e`, `m`, `t`, `o`, `q`, `Shift+U`, `Ctrl+←/→`) remain ungated — they work from any tab.

**Step 5: Commit**

```
feat(romm-tui): gate detail keys by active tab, add tab switching
```

---

## Task 5: Context-sensitive footer hints

**Files:**
- Modify: `romm-tui/src/tui/screens/game_detail/state.rs`

**Step 1:** Replace the current `footer_help_entries` implementation. Return different hint arrays based on `self.active_tab`:

- **Info:** `e Extras`, `m Match`/`t Technical`, `Shift+U Unmatch`, `Ctrl+←/→ Resize`, `1/2/3 Tabs`
- **Saves:** `u Upload`, `D Download`, `j/k Navigate`, `1/2/3 Tabs`
- **Achievements:** `j/k Scroll`, `1/2/3 Tabs`

Keep the technical-mode toggle in the Info tab footer.

**Step 2:** Update `footer_help_entries_mention_extras_shortcut` and `footer_help_entries_track_technical_mode` tests — these now only apply when `active_tab == DetailTab::Info`.

**Step 3: Commit**

```
feat(romm-tui): context-sensitive footer hints per tab
```

---

## Task 6: Update existing tests

**Files:**
- Modify: `romm-tui/src/tui/screens/game_detail/tests.rs`
- Modify: `romm-tui/tests/tui_app.rs` (if any tests assume single-pane detail)

**Step 1:** Update all `GameDetailScreen` construction in tests to account for the new `active_tab` and `achievement_scroll_offset` fields (these have defaults set in `new()` so should be fine, but verify).

**Step 2:** Add a basic test: switching tabs changes `active_tab`.

**Step 3:** Add a test: `j`/`k` on Info tab does nothing (no save nav on wrong tab).

**Step 4: Commit**

```
test(romm-tui): update tests for tabbed game detail
```

---

## Task 7: Final verification

```bash
cargo fmt
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings
cargo test
```

Manual checklist:

- [ ] Game detail opens on Info tab by default
- [ ] `1`/`2`/`3` switch tabs, tab bar highlights correctly
- [ ] Saves tab shows full list, `j`/`k` navigate, `u`/`D` work
- [ ] Achievements tab shows full list with scroll
- [ ] `u`/`D` do nothing on Info and Achievements tabs
- [ ] Cover panel stays visible on all tabs
- [ ] Footer hints change per tab
- [ ] `Esc` returns to library from any tab
