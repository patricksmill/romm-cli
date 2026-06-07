#!/usr/bin/env python3
"""Filter a monolithic release-please changelog into per-crate changelogs by commit scope.

One-time migration tool: point SOURCE at a full monolith export before filtering.
Do not re-run against an already-filtered romm-cli/CHANGELOG.md.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "romm-cli" / "CHANGELOG.md"

SKIP_SCOPES = {
    "tests",
    "docs",
    "ci",
    "formatting",
    "main",
    "cargo",
    "setup",
    "openapi_sync",
}

API_SCOPES = {
    "api",
    "config",
    "download",
    "sync",
    "save-sync",
    "paths",
    "hash",
    "search",
    "collections",
    "interrupt",
    "core",
    "client",
    "openapi",
    "romm-api",
}

CLI_SCOPES = {"cli", "completions", "auth", "romm-cli", "init", "roms"}

TUI_SCOPES = {
    "tui",
    "settings",
    "setup-wizard",
    "setup_wizard",
    "rom-load",
    "app",
    "cover",
    "upload",
}

CRATE_HEADERS = {
    "api": """# Changelog

All notable changes to **romm-api** are documented in this file.

Entries before the workspace split (1.0.0) are filtered from the unified monolith history by conventional-commit scope. Frontend-only changes appear in [romm-cli/CHANGELOG.md](../romm-cli/CHANGELOG.md) or [romm-tui/CHANGELOG.md](../romm-tui/CHANGELOG.md).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
""",
    "cli": """# Changelog

All notable changes to **romm-cli** are documented in this file.

Entries before the workspace split (1.0.0) are filtered from the unified monolith history by conventional-commit scope. Shared library changes appear in [romm-api/CHANGELOG.md](../romm-api/CHANGELOG.md). Terminal UI changes appear in [romm-tui/CHANGELOG.md](../romm-tui/CHANGELOG.md).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
""",
    "tui": """# Changelog

All notable changes to **romm-tui** are documented in this file.

Entries before the workspace split (1.0.0) are filtered from the unified monolith history by conventional-commit scope. Shared library changes appear in [romm-api/CHANGELOG.md](../romm-api/CHANGELOG.md). CLI and scripting changes appear in [romm-cli/CHANGELOG.md](../romm-cli/CHANGELOG.md).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
""",
}

CRATE_PATHS = {
    "api": ROOT / "romm-api" / "CHANGELOG.md",
    "cli": ROOT / "romm-cli" / "CHANGELOG.md",
    "tui": ROOT / "romm-tui" / "CHANGELOG.md",
}

VERSION_RE = re.compile(r"^## \[([^\]]+)\]")
SCOPE_RE = re.compile(r"^[\*-]\s+\*\*([^:*]+)")


def normalize_scope(raw: str) -> str:
    return raw.split("/")[0].lower().replace("_", "-")


def classify_line(line: str, crate: str) -> bool:
    stripped = line.strip()
    if not stripped or stripped.startswith("#"):
        return False

    lower = stripped.lower()
    scope_match = SCOPE_RE.match(stripped)
    scope = normalize_scope(scope_match.group(1)) if scope_match else None

    if scope in SKIP_SCOPES:
        return False

    if scope == "errors":
        if "exit code" in lower:
            return crate == "cli"
        return crate == "api"

    if scope == "update":
        if "romm-cli tui" in lower:
            return crate == "cli"
        return crate in {"api", "cli"}

    if scope == "cache":
        if "command" in lower:
            return crate == "cli"
        return crate == "api"

    if scope == "scan":
        return crate in {"api", "cli"}

    if scope == "roms":
        return crate == "cli"

    if scope == "upload":
        return crate == "tui"

    if scope == "auth":
        return crate == "cli"

    if scope == "config":
        if stripped.startswith("- **TUI:**") or stripped.startswith("- **tui:**"):
            return crate == "tui"
        if "config/tui" in lower:
            return crate in {"api", "tui"}
        return crate == "api"

    if stripped.startswith("- **TUI:**") or stripped.startswith("- **tui:**"):
        return crate == "tui"

    if scope in API_SCOPES:
        return crate == "api"
    if scope in CLI_SCOPES:
        return crate == "cli"
    if scope in TUI_SCOPES:
        return crate == "tui"

    return classify_unscoped(stripped, crate)


def classify_unscoped(line: str, crate: str) -> bool:
    lower = line.lower()

    if "implement tui and cli frontends" in lower:
        return crate in {"cli", "tui"}

    if "cli application structure with tui" in lower:
        return crate == "cli"

    if "romm-tui" in lower and "binary" in lower:
        return crate == "tui"

    if "romm-cli init" in lower or "interactive user configuration" in lower:
        return crate == "cli"

    if "self-update command" in lower or "resource-action subcommands" in lower:
        return crate == "cli"

    if any(
        token in lower
        for token in (
            "openapi",
            "endpoint",
            "device management",
            "sync endpoint",
            "compatibility check",
            "hash computation",
            "download management",
            "download extras command",
            "library scan functionality",
            "virtual and smart collections",
            "keyring integration",
            "unauthenticated json",
        )
    ):
        return crate == "api"

    if any(
        token in lower
        for token in (
            "setup wizard",
            "settings screen",
            "library screen",
            "game detail",
            "terminal ui",
            " lazyloading",
            "keyboard navigation",
            "help overlay",
            "path picker",
            "startup steps to tui",
            "pairing authentication",
            "appearance tab",
            "global shortcut",
            "startup splash",
        )
    ):
        return crate == "tui"

    if "save sync functionality to tui and cli" in lower:
        return crate == "api"

    if "cli and tui" in lower and "update prompt" in lower:
        return crate == "api"

    return False


def filter_changelog(text: str, crate: str) -> str:
    lines = text.splitlines()
    # Skip generated header; rebuild per crate.
    start = 0
    for i, line in enumerate(lines):
        if line.startswith("## ["):
            start = i
            break

    out: list[str] = [CRATE_HEADERS[crate].rstrip(), ""]
    current_section: list[str] = []
    section_title: str | None = None
    version_header: str | None = None
    pending_version: str | None = None

    def flush_version() -> None:
        nonlocal current_section, section_title, version_header, pending_version
        if version_header is None:
            return
        body_lines: list[str] = []
        current_sub: list[str] = []
        sub_title: str | None = None

        def flush_sub() -> None:
            nonlocal current_sub, sub_title
            if sub_title and current_sub:
                body_lines.append(sub_title)
                body_lines.extend(current_sub)
                body_lines.append("")
            current_sub = []
            sub_title = None

        for entry in current_section:
            if entry.startswith("### "):
                flush_sub()
                sub_title = entry
            elif classify_line(entry, crate):
                current_sub.append(entry)

        flush_sub()

        while body_lines and body_lines[-1] == "":
            body_lines.pop()

        if body_lines:
            out.append(version_header)
            out.append("")
            out.extend(body_lines)
            out.append("")

        current_section = []
        section_title = None
        version_header = None
        pending_version = None

    for line in lines[start:]:
        if line.startswith("[Unreleased]:") or line.startswith("[0."):
            break

        version_match = VERSION_RE.match(line)
        if version_match:
            flush_version()
            pending_version = version_match.group(1)
            version_header = line
            continue

        if line.startswith("### "):
            current_section.append(line)
            continue

        if line.strip().startswith(("*", "-")):
            current_section.append(line)
            continue

        if line.strip() == "" and not current_section:
            continue

    flush_version()
    return "\n".join(out).rstrip() + "\n"


def main() -> int:
    source_text = SOURCE.read_text(encoding="utf-8")
    for crate, path in CRATE_PATHS.items():
        filtered = filter_changelog(source_text, crate)
        path.write_text(filtered, encoding="utf-8", newline="\n")
        print(f"wrote {path.relative_to(ROOT)} ({len(filtered.splitlines())} lines)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
