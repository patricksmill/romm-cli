# GitHub repo health (Option C) — design

**Date:** 2026-06-21  
**Status:** Implemented  
**Goal:** Green GitHub community profile and lower maintainer friction for a solo-maintainer Rust workspace, without duplicating existing docs.

---

## Context

The repository already has:

- Strong root `README.md` (quick start, doc index, workspace layout)
- `LICENSE` (MIT)
- CI + Release Please workflows
- Contributor-oriented docs: `docs/rust-guidelines.md`, `docs/architecture.md`, `docs/releases.md`

Missing for GitHub community health:

- `CONTRIBUTING.md`
- `CODE_OF_CONDUCT.md`
- `SECURITY.md`
- Issue template(s)
- Pull request template

**Constraints:** Solo maintainer, occasional drive-by PRs; keep maintenance low; reuse existing docs instead of copying content.

---

## Approach (Option C)

Add a lean **`.github/` bundle** plus three short root community files. Link out to existing docs for depth.

| Artifact | Purpose |
|----------|---------|
| `CONTRIBUTING.md` | Entry point: build, checks, commits, scopes, links |
| `CODE_OF_CONDUCT.md` | Contributor Covenant 2.1 (standard) |
| `SECURITY.md` | Private reporting path; no secrets in public issues |
| `.github/ISSUE_TEMPLATE/config.yml` | Template chooser + doc links |
| `.github/ISSUE_TEMPLATE/report.yml` | Single form: bug / feature / docs / question |
| `.github/PULL_REQUEST_TEMPLATE.md` | Short PR checklist |
| `README.md` (edit) | Point Contributing section at `CONTRIBUTING.md` |

**Out of scope (YAGNI):**

- `CODEOWNERS`, Discussions, FUNDING.yml, multiple issue YAML files
- Duplicating `rust-guidelines.md` or release runbook in CONTRIBUTING
- Good-first-issue automation or issue labels (can add later)

---

## Root community files

### `CONTRIBUTING.md` (~50 lines)

Sections:

1. **Thanks** — one line
2. **Before you open a PR** — clone, `cargo test --workspace`, required fmt/clippy sequence (match CI / `.cursor/rules/pre-commit-checks.mdc`)
3. **Commits** — Conventional Commits; scopes `api`, `cli`, `tui` (and crate names); link `docs/releases.md` changelog routing
4. **Which crate?** — table: change area → crate → doc
5. **What we won't merge** — secrets in issues/PRs; generated completion scripts; unrelated drive-by refactors
6. **Questions** — link `docs/troubleshooting-auth.md`, RomM upstream for server bugs

### `CODE_OF_CONDUCT.md`

- Contributor Covenant 2.1
- Enforcement contact: report via GitHub to `@patricksmill` or private email if listed in GitHub profile

### `SECURITY.md`

- Prefer [GitHub Security Advisories](https://github.com/patricksmill/romm-cli/security/advisories/new)
- Do not file public issues for: API tokens, keyring contents, full `config.json`, live server URLs with credentials
- Supported versions: latest release tags per crate (link `docs/compatibility.toml`)

---

## `.github/` templates

### `ISSUE_TEMPLATE/config.yml`

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

### `ISSUE_TEMPLATE/report.yml`

Single form with:

| Field | Notes |
|-------|--------|
| Type (dropdown) | Bug, Feature, Docs, Question |
| Crate (dropdown) | romm-api, romm-cli, romm-tui, not sure |
| Description | Required textarea |
| Version | Text (`romm-cli --version` / crate version) |
| Reproduce steps | Textarea; optional for features |
| Environment | OS, RomM version if known |

Labels: optional `labels: ["triage"]` if label exists; omit if not created yet.

### `PULL_REQUEST_TEMPLATE.md`

Checklist:

- [ ] Target crate(s): api / cli / tui
- [ ] `cargo fmt --check` and both clippy runs pass
- [ ] `cargo test --workspace` (or scoped crate tests if justified)
- [ ] Conventional commit / PR title scope matches crate changelog
- [ ] No secrets, tokens, or personal config in diff
- [ ] User-facing change documented in correct crate doc if needed

---

## README change

Replace the minimal Contributing block with a link to `CONTRIBUTING.md`; keep the three-line build snippet or move it entirely into CONTRIBUTING (prefer link only to avoid duplication).

---

## Verification

After merge:

1. GitHub **Settings → General → Community Standards** — all recommended items checked
2. Open **New issue** — see template chooser + contact links
3. Open **New PR** — template appears
4. `./tools/release-check.sh` still passes (no Rust changes expected)

---

## Success criteria

- Community profile shows: README, license, contributing, code of conduct, issue template, PR template
- New contributors find build/check/commit rules in under 2 clicks from README
- Maintainer gets structured issues (type + crate) without maintaining multiple templates
