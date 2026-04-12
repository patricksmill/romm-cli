# TUI Global Error Handling and Testing Design

## Overview
Currently, the TUI crashes (exit code 1) when the RomM API returns a 500 Internal Server Error because it uses the `?` operator to bubble up API errors from the event loop. This design introduces a global error toast to gracefully handle API failures and uses `wiremock` to test the TUI state machine.

## Architecture & Error Handling
- **State Changes**: Add `pub global_error: Option<String>` to `App` in `src/tui/app.rs`. Add a helper method `fn set_error(&mut self, err: anyhow::Error)` that formats the error and sets `global_error`.
- **Event Loop Changes**: In `App::run`, before dispatching keys to screens, check if `global_error` is set. If it is, intercept all keys. If the user presses `Esc` or `Enter`, clear the error. Other keys are ignored.
- **API Call Changes**: Replace `?` on API calls (e.g., `let platforms = self.client.call(&ListPlatforms).await?;`) with a `match` statement. If `Ok`, proceed. If `Err`, call `self.set_error(e)` and return `Ok(false)` to keep the app running.
- **Rendering**: In `App::render`, after drawing the current screen, if `global_error` is `Some(msg)`, draw a red `Clear` block (a popup) in the center of the screen with the error text and a "Press Esc to dismiss" footer.

## Testing Strategy
- **Testing Setup**: Add `wiremock` to `dev-dependencies` in `Cargo.toml`. Create a new integration test file `tests/tui_app.rs`.
- **The Error Test Case**: Spin up a `MockServer` using `wiremock` and configure it to return a `500 Internal Server Error` for `GET /api/platforms`. Create a `Config` pointing to the `MockServer`'s URL. Instantiate `RommClient` and `App` with this config. Simulate a user pressing `Enter` on the Main Menu. Assert that `app.global_error` is `Some("ROMM API error: 500 Internal Server Error...")` and that the app did not crash.
- **The Success Test Case**: Configure the `MockServer` to return `200 OK` with valid JSON for `GET /api/platforms` and `GET /api/collections`. Simulate a user pressing `Enter` on the Main Menu. Assert that the app successfully transitions to the `LibraryBrowse` screen.