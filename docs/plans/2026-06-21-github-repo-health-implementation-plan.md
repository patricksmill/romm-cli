# GitHub Repo Health (Option C) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Status:** Completed (2026-06-21)

**Goal:** Add lean community files and `.github/` templates so GitHub repo health is complete for a solo-maintainer workspace.

**Architecture:** Three short root markdown files (CONTRIBUTING, CoC, SECURITY) link to existing `docs/`; one YAML issue form + config + PR template live under `.github/`. README points at CONTRIBUTING. No Rust or CI changes.

**Tech Stack:** GitHub community files, YAML issue forms, Markdown templates.

**Design:** [2026-06-21-github-repo-health-design.md](./2026-06-21-github-repo-health-design.md)

---

### Task 1: Add CONTRIBUTING.md

**Files:**
- Create: `CONTRIBUTING.md`
- Reference: `docs/rust-guidelines.md`, `docs/releases.md`, `docs/architecture.md`, `.cursor/rules/pre-commit-checks.mdc`

**Step 1: Create the file**

Write `CONTRIBUTING.md` with these sections (full prose, ~50 lines):

```markdown
# Contributing

Thanks for helping improve romm-cli.

## Before you open a PR

```bash
git clone https://github.com/patricksmill/romm-cli
cd romm-cli
cargo test --workspace
```

Required checks (match CI):

```bash
cargo fmt
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings
```

## Commits and changelogs

Use [Conventional Commits](https://www.conventionalcommits.org/). Scope by crate so Release Please routes changelog entries:

| Scope | Crate |
|-------|-------|
| `api`, `config`, `download`, `client` | `romm-api` |
| `cli`, `completions`, `auth` | `romm-cli` |
| `tui`, `settings` | `romm-tui` |

See [docs/releases.md](docs/releases.md) for versioning and publish order (`romm-api` first).

## Where to change code

| Area | Start here |
|------|------------|
| HTTP client, downloads, config | [docs/api.md](docs/api.md), [docs/architecture.md](docs/architecture.md) |
| CLI commands, `--json` | [docs/cli.md](docs/cli.md) |
| TUI screens, keybindings | [docs/tui.md](docs/tui.md) |
| Rust conventions, gaps | [docs/rust-guidelines.md](docs/rust-guidelines.md) |

## Please don't

- Commit generated shell completion scripts (use `romm-cli completions <shell>` at install time).
- Include API tokens, keyring dumps, or full `config.json` in issues or PRs.
- Open drive-by refactors unrelated to the issue you are fixing.

## Questions

- Auth / keyring / Docker: [docs/troubleshooting-auth.md](docs/troubleshooting-auth.md)
- RomM server bugs: [rommapp/romm](https://github.com/rommapp/romm/issues)
```

**Step 2: Verify links**

Open each relative link path; confirm files exist.

**Step 3: Commit**

```bash
git add CONTRIBUTING.md
git commit -m "docs: add CONTRIBUTING.md for community profile"
```

---

### Task 2: Add CODE_OF_CONDUCT.md

**Files:**
- Create: `CODE_OF_CONDUCT.md`

**Step 1: Add Contributor Covenant 2.1**

Use the standard [Contributor Covenant 2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/) text. Set:

- Enforcement contact: `https://github.com/patricksmill` (GitHub profile) with note to use Security Advisories for sensitive reports.

**Step 2: Commit**

```bash
git add CODE_OF_CONDUCT.md
git commit -m "docs: add Contributor Covenant code of conduct"
```

---

### Task 3: Add SECURITY.md

**Files:**
- Create: `SECURITY.md`

**Step 1: Create the file**

```markdown
# Security policy

## Reporting a vulnerability

Please report security issues **privately**:

- [GitHub Security Advisories](https://github.com/patricksmill/romm-cli/security/advisories/new) (preferred)

Do **not** open public issues for vulnerabilities.

## What not to paste publicly

- RomM API tokens or passwords
- OS keyring exports or `config.json` with real credentials
- Live server URLs that include credentials

## Supported versions

We address security issues on the latest released versions tracked in [docs/compatibility.toml](docs/compatibility.toml). Upgrade to the newest `romm-api`, `romm-cli`, and `romm-tui` tags before reporting when possible.
```

**Step 2: Commit**

```bash
git add SECURITY.md
git commit -m "docs: add security policy for private reporting"
```

---

### Task 4: Add issue template config

**Files:**
- Create: `.github/ISSUE_TEMPLATE/config.yml`

**Step 1: Create config.yml**

```yaml
blank_issues: false
contact_links:
  - name: Documentation
    url: https://github.com/patricksmill/romm-cli/tree/main/docs
  - name: Auth troubleshooting
    url: https://github.com/patricksmill/romm-cli/blob/main/docs/troubleshooting-auth.md
  - name: RomM server (upstream)
    url: https://github.com/rommapp/romm/issues
```

**Step 2: Commit**

```bash
git add .github/ISSUE_TEMPLATE/config.yml
git commit -m "docs: add issue template chooser config"
```

---

### Task 5: Add unified issue form

**Files:**
- Create: `.github/ISSUE_TEMPLATE/report.yml`

**Step 1: Create report.yml**

```yaml
name: Report
description: Bug, feature, docs, or question
title: "[type]: "
body:
  - type: dropdown
    id: issue_type
    attributes:
      label: Type
      options:
        - Bug
        - Feature
        - Docs
        - Question
    validations:
      required: true
  - type: dropdown
    id: crate
    attributes:
      label: Crate
      options:
        - romm-cli
        - romm-tui
        - romm-api
        - Not sure
    validations:
      required: true
  - type: textarea
    id: description
    attributes:
      label: Description
      description: What happened or what you want?
    validations:
      required: true
  - type: input
    id: version
    attributes:
      label: Version
      description: Output of `romm-cli --version`, `romm-tui --version`, or crate version
  - type: textarea
    id: reproduce
    attributes:
      label: Steps to reproduce
      description: For bugs; command, config (redact secrets), expected vs actual
  - type: textarea
    id: environment
    attributes:
      label: Environment
      description: OS, RomM server version if known
```

**Step 2: Commit**

```bash
git add .github/ISSUE_TEMPLATE/report.yml
git commit -m "docs: add unified GitHub issue form"
```

---

### Task 6: Add pull request template

**Files:**
- Create: `.github/PULL_REQUEST_TEMPLATE.md`

**Step 1: Create template**

```markdown
## Summary

<!-- One or two sentences: what and why -->

## Crate(s)

- [ ] romm-api
- [ ] romm-cli
- [ ] romm-tui
- [ ] docs / CI only

## Checklist

- [ ] `cargo fmt --check` passes
- [ ] Both `cargo clippy` runs pass (`--all-features` and `--no-default-features`, `-D warnings`)
- [ ] `cargo test --workspace` passes (or scoped tests with reason)
- [ ] PR title uses Conventional Commits with the right scope for the changelog
- [ ] No secrets, tokens, or personal config in the diff
- [ ] User-facing changes documented in the relevant `docs/*.md` if needed
```

**Step 2: Commit**

```bash
git add .github/PULL_REQUEST_TEMPLATE.md
git commit -m "docs: add pull request template"
```

---

### Task 7: Update README Contributing section

**Files:**
- Modify: `README.md:89-99`

**Step 1: Replace Contributing block**

Change to:

```markdown
## Contributing

Issues and pull requests are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for build steps, required checks, and commit conventions.

Contributor deep-dive: [rust-guidelines.md](docs/rust-guidelines.md).
```

Remove duplicate `git clone` / `cargo build` block (now in CONTRIBUTING).

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: link README contributing section to CONTRIBUTING.md"
```

---

### Task 8: Verify (no Rust changes)

**Step 1: Local sanity**

```bash
cargo fmt --check
./tools/release-check.sh
```

Expected: same pass/fail as before (no crate version changes in this work).

**Step 2: GitHub UI (after push)**

- Repo → Community Standards: all items green
- New Issue: "Report" template + contact links
- New PR: checklist template visible

**Step 3: Optional single squash commit**

If preferred, squash Tasks 1–7 into one commit:

```bash
git commit -m "docs: add GitHub community health files and templates"
```

---

## Execution handoff

Plan complete and saved to `docs/plans/2026-06-21-github-repo-health-implementation-plan.md`.

**1. Subagent-Driven (this session)** — implement task-by-task here with review between tasks.

**2. Parallel Session** — open a new session with `@executing-plans` in a worktree.

Which approach?
