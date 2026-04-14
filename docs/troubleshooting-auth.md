# Troubleshooting authentication

This page explains common auth issues when using `romm-cli` on different machines (Windows, Linux, macOS, Docker, CI, SSH).

## How configuration is loaded

1. **Environment variables** (highest priority) — `API_BASE_URL`, `ROMM_DOWNLOAD_DIR`, `API_USE_HTTPS`, and auth-related vars.
2. **`config.json`** in the OS config directory — fills any field not set by the environment.
3. **OS keyring** — replaces placeholder strings in memory, including the on-disk sentinel `<stored-in-keyring>` written when secrets were stored in the keyring.

`romm-cli` does **not** read a `.env` file automatically. Use your shell, container env, or a tool that injects env vars before starting the process.

See [README.md](../README.md#configuration) for paths and variable names.

## Environment wins (CI, Docker, SSH)

For **automation**, **headless servers**, or when the keyring is unavailable, set credentials in the **environment**:

- **Bearer:** `API_TOKEN`
- **Basic:** `API_USERNAME` and `API_PASSWORD`
- **API key:** `API_KEY_HEADER` and `API_KEY`

Environment values override `config.json` for those fields, so you do not need a working keyring for that run.

## Token from a file at runtime (`ROMM_TOKEN_FILE`)

For **Docker** or **Kubernetes**, you can mount a secret as a file instead of passing the token in the environment:

- Set **`ROMM_TOKEN_FILE`** to the path of a UTF-8 file containing the bearer token (whitespace is trimmed).
- **`API_TOKEN_FILE`** is accepted as an alias.
- Precedence: **`API_TOKEN`** env var, then **`ROMM_TOKEN_FILE`** / **`API_TOKEN_FILE`**, then `config.json`.

If the variable is set but the file is missing, unreadable, empty after trim, or larger than **64 KiB**, `romm-cli` exits with an error when loading config.

## “Auth: None” but `config.json` shows a Bearer token

If `config.json` contains `"token": "<stored-in-keyring>"`, the real secret was stored in the **OS keyring** when you ran `init` or saved settings. At runtime, `romm-cli` must **read** that secret from the keyring.

If lookup fails (wrong OS user, headless Linux without a secret service, SSH session, etc.), loaded `auth` becomes **empty** and the TUI may show no credentials. **Fix:**

1. Set **`API_TOKEN`** in the environment for that session, or use **`ROMM_TOKEN_FILE`** as above, **or**
2. Run **`romm-cli init`** again on that machine/user so secrets are stored where the keyring can read them, **or**
3. Ensure you run as the **same Windows/macOS/Linux user** that created the credential.

## Windows Credential Manager

The Rust `keyring` crate stores Windows Generic credentials with a **target name** derived from service + key (see [keyring `windows` docs](https://docs.rs/keyring/latest/x86_64-pc-windows-msvc/keyring/windows/index.html)). For `romm-cli`, the bearer token entry is typically named like **`API_TOKEN.romm-cli`**, not necessarily starting with `romm-cli`. Search under **Generic credentials** or for **`API_TOKEN`**.

## Linux and macOS

- **Linux:** A desktop session often provides Secret Service / gnome-keyring; **SSH or minimal containers** may not — use env vars or **`ROMM_TOKEN_FILE`** there.
- **macOS:** Keychain usually works for GUI login sessions; remote or automated contexts may differ — same fallbacks as above.

## Security notes

- Prefer **`API_TOKEN`** or **`ROMM_TOKEN_FILE`** over storing long-lived plaintext tokens in `config.json`.
- On Unix, `romm-cli` sets restrictive permissions on `config.json` when it writes the file; still protect the file and your environment from other users.
