# TUI Auth Updater Fix & Renaming

## Overview
When the user edits their authentication via the TUI Settings screen, the `SetupWizard` correctly updates the application's `Config` and `RommClient`. However, the newly rendered `SettingsScreen` still displays the old authentication status (e.g., "Basic") because the `auth_status` string is generated once during `SettingsScreen::new` and the UI was not properly re-initialized with the new config. Additionally, we need to rename "Bearer token" to "API Token" across the application to match the RomM web UI terminology.

## Approaches Considered

### 1. Re-initialize the Settings Screen properly (Selected)
When the `SetupWizard` completes successfully in `App::handle_setup_wizard`, we already call `SettingsScreen::new(&self.config, ...)`. The bug is that the `SettingsScreen::new` method itself calculates the `auth_status` string based on the *passed-in* config, which is correct, but we need to ensure the terminology is updated and the UI reflects the new state.

### 2. Make `auth_status` dynamic
Instead of calculating the `auth_status` string once when the `SettingsScreen` is created, we could change the `SettingsScreen` to hold a reference to the `Config` and calculate the string on-the-fly every time it renders. This is more complex due to Rust's lifetime rules and unnecessary since we already re-create the screen.

## Implementation Details

### 1. Rename "Bearer token" to "API Token"
Update the terminology in the following locations:
- `src/tui/screens/settings.rs`: Change `"Bearer token"` to `"API Token"`.
- `src/tui/screens/setup_wizard.rs`: Change `"Bearer token"` to `"API Token"` in `auth_labels()` and the `Step::Bearer` rendering logic.
- `src/commands/init.rs`: Change `"Bearer token"` to `"API Token"` in the interactive prompt choices.

### 2. Fix the UI State Update
The core issue is that when the user presses `s` to save settings in `App::handle_settings`, the `SettingsScreen`'s internal `auth_status` is *not* updated, even though `self.config` is. However, the auth changer uses `App::handle_setup_wizard`, which *does* call `SettingsScreen::new`. We need to ensure that the `auth_status` string generation in `SettingsScreen::new` correctly identifies the new auth type and uses the new "API Token" terminology.

**Changes:**
1. In `src/tui/screens/settings.rs`, update `SettingsScreen::new` to format the `auth_status` string correctly:
   ```rust
   Some(crate::config::AuthConfig::Bearer { .. }) => "API Token".to_string(),
   ```
2. In `src/tui/screens/setup_wizard.rs`, update `auth_labels`:
   ```rust
   "API Token (Bearer)",
   ```
3. In `src/commands/init.rs`, update the interactive prompt:
   ```rust
   "API Token (Bearer)",
   ```

## Testing
1. Run `cargo run -- tui`.
2. Go to Settings -> Auth.
3. Change from Basic to API Token.
4. Verify the Settings screen now says "Auth: API Token".
