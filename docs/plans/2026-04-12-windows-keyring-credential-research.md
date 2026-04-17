# Windows credentials + `keyring` crate — research notes

## Context

`romm-cli` uses the Rust [`keyring`](https://crates.io/crates/keyring) crate (`3.6.x`) with:

```text
service = "romm-cli"
user    = "API_TOKEN" | "API_PASSWORD" | "API_KEY"   (per secret type)
```

via `keyring::Entry::new(KEYRING_SERVICE, key)` in [`src/config.rs`](../../src/config.rs).

Users may not see an obvious **“romm-cli”** entry in Windows Credential Manager, and `config.json` can contain `"token": "<stored-in-keyring>"` while the TUI still shows **auth: None** after reload.

## How Windows storage works in `keyring` 3.6

Per [official docs — `keyring::windows`](https://docs.rs/keyring/3.6.3/x86_64-pc-windows-msvc/keyring/windows/index.html):

1. **Backend:** Windows **Generic** credentials (not Web Passwords).
2. **Target name:** For `Entry::new(service, user)`, the credential’s **target name** is the concatenation **`user.service`** (note the order: **user first**, then **dot**, then **service**).
3. **Metadata:** Separate **username** and **comment** fields are filled by the crate for display/tooling.

### Mapped names for romm-cli

| Keyring key   | `Entry::new("romm-cli", key)` | Windows **target name** (Generic credential) |
|---------------|--------------------------------|-----------------------------------------------|
| `API_TOKEN`   | service=`romm-cli`, user=`API_TOKEN` | **`API_TOKEN.romm-cli`** |
| `API_PASSWORD`| user=`API_PASSWORD`            | **`API_PASSWORD.romm-cli`** |
| `API_KEY`     | user=`API_KEY`                 | **`API_KEY.romm-cli`** |

So searching Credential Manager for **`romm-cli` alone** may fail: the list is keyed by **target name**, which starts with **`API_TOKEN`**, not `romm-cli`.

Other CLIs (e.g. `jules-cli:default`, `Supabase CLI:supabase`) use different naming schemes; **romm-cli follows `keyring`’s default**, not a `service:key` style label.

## Why `config.json` can show `<stored-in-keyring>` but UI shows “no auth”

[`persist_user_config`](../../src/config.rs) only replaces the on-disk token with `<stored-in-keyring>` when `keyring_store` returns **`Ok`**. So at save time, **`set_password` succeeded** from the crate’s perspective.

[`load_config`](../../src/config.rs) resolves the sentinel by calling **`keyring_get("API_TOKEN")`**, which uses the **same** `Entry::new("romm-cli", "API_TOKEN")` mapping. If **`get_password` fails** (NoEntry, permission, or ambiguous match), the token string stays as the sentinel, and the final Bearer branch **drops** auth → **`auth: None`**.

So the failure mode is **not** “pairing didn’t save” but **round-trip**: **write appeared OK, read fails** (or read runs in a different user context than write).

## Likely causes (ranked)

1. **Looking for the wrong label in Credential Manager** — expect **`API_TOKEN.romm-cli`** under Generic credentials, not an entry whose primary label is `romm-cli`.
2. **Different Windows user / elevation** — credentials are per logon session; Admin vs normal, or different accounts, split the vault.
3. **Silent read failure** — less common if write succeeded in the same session, but antivirus/corporate policy can block CredRead while allowing CredWrite (rare).
4. **Placeholder without successful write** — inconsistent with code: failed write keeps **plaintext** in JSON and logs a **warning**. A file that is **only** sentinel implies **`Ok`** from `set_password`.

## Approaches to improve UX (product, not yet implemented)

| Approach | Pros | Cons |
|----------|------|------|
| **A. Document** the Windows target name `API_TOKEN.romm-cli` in user-facing docs | No code risk | Doesn’t fix read failures |
| **B. Info-level log** when `load_config` sees sentinel but `keyring_get` fails | Makes misconfiguration obvious | Log noise |
| **C. `Entry::new_with_target`** with a single friendly target (e.g. `romm-cli`) | Easier to find in UI | Must migrate existing entries or support both |
| **D. On failed read with sentinel, surface error** in TUI instead of showing `auth: None` | Matches user mental model | Requires UI strings |

**Recommendation:** **A + D** short term; consider **C** only if we want Windows-specific naming parity with other tools.

## Update: persist must not call `set_password` with the sentinel

If `persist_user_config` receives auth whose secret fields are already `<stored-in-keyring>` (e.g. TUI Settings merge when `load_config` could not resolve the keyring), it must **not** pass that literal string to `keyring_store`. Doing so overwrote the real vault entry with the placeholder. **`persist_user_config` now skips keyring updates when the value is the sentinel** and writes JSON unchanged—see `persist_user_config` in `src/config.rs`.

## References

- [keyring 3.6.3 — `Entry::new`](https://docs.rs/keyring/3.6.3/keyring/struct.Entry.html#method.new)
- [keyring 3.6.3 — `keyring::windows`](https://docs.rs/keyring/3.6.3/x86_64-pc-windows-msvc/keyring/windows/index.html) (target name = `user.service`)
