# ROM metadata editing

Edit **game metadata** (provider match, name, summary, cover) for a ROM on the RomM server. This is separate from per-user **`roms props`** — those remain user-specific notes and are not merged into metadata commands.

**Server requirement:** RomM **4.8+** with OpenAPI-exposed endpoints:

- `GET /api/search/roms`
- `PUT /api/roms/{id}` (multipart)
- `GET /api/roms/{id}`

The CLI and TUI detect support at runtime via the cached OpenAPI registry.

---

## CLI (`roms metadata`)

Alias: `roms meta`.

```bash
# Search provider candidates
romm-cli roms metadata search <rom-id> --query "zelda"

# Apply match by provider ID
romm-cli roms metadata match <rom-id> --igdb-id 1234

# Interactive picker (default on TTY when no ID flags)
romm-cli roms metadata match <rom-id> --query "zelda" --pick

# Edit text fields or cover
romm-cli roms metadata edit <rom-id> --name "My title"
romm-cli roms metadata edit <rom-id> --summary "Notes"
romm-cli roms metadata edit <rom-id> --url-cover "https://…"
romm-cli roms metadata edit <rom-id> --artwork ./cover.png

# Clear provider links or cover art
romm-cli roms metadata unmatch <rom-id> --yes
romm-cli roms metadata remove-cover <rom-id> --yes
```

All subcommands accept `--json` for machine-readable output.

Applying a provider ID triggers RomM’s server-side scrape (no client-side IGDB/Moby keys). After writes, the local ROM list cache for that platform is invalidated.

---

## TUI (game detail)

From **game detail**:

| Key | Action |
|-----|--------|
| `m` | Open metadata match picker (searches providers, pick row, Enter to apply) |
| `Shift+U` | Unmatch metadata (confirm with `y`) |
| `t` | Toggle technical details (was `m` before metadata editing) |

When the server lacks metadata edit endpoints, the TUI shows a short notice instead of opening the picker.

---

## Related

- [cli.md](cli.md) — full command reference
- [tui.md](tui.md) — TUI overview
- [save-sync.md](save-sync.md) — separate feature with its own compatibility gate
