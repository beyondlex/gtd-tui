# Repository Guidelines

## Project Structure & Module Organization

This repo is currently a planning skeleton. The only committed artifacts are the high‑level spec and local skill bundles.

- `SPEC.md`: Product and architecture plan (source of truth for intended structure).
- `gtd-core/`: Placeholder for the Rust core library (currently empty).
- `skills/` and `.agents/`: Agent skill definitions used by this workspace.
- `skills-lock.json`: Skill manifest/lock.

As implementation starts, follow the planned layout in `SPEC.md` (e.g., `gtd-core/src/models`, `gtd-tui/src/ui`, `gtd-tui/src/commands`).

## Build, Test, and Development Commands

There is no build or test system wired up yet. Once Rust crates are added, standard Cargo commands are expected, for example:

- `cargo build` — build the current crate/workspace.
- `cargo test` — run unit and integration tests.
- `cargo fmt` / `cargo clippy` — format and lint.

If you introduce new tooling (e.g., `just`, `make`), document the exact commands here.

## Coding Style & Naming Conventions

Rust style should follow `rustfmt` defaults and Clippy recommendations. Use clear, domain‑driven names that mirror `SPEC.md` entities (e.g., `Task`, `Project`, `Area`, `HotkeyConfig`). Prefer `snake_case` for functions/modules and `PascalCase` for types. Add a `rustfmt.toml` only if deviations are needed.

## Testing Guidelines

No tests exist yet. When adding tests, prefer:

- Unit tests co‑located in `src/` modules.
- Integration tests under `gtd-core/tests/` or `gtd-tui/tests/`.
- Test names that describe behavior, e.g., `creates_task_with_due_date`.

State coverage expectations in this section once a baseline exists.

## Commit & Pull Request Guidelines

Commit history is minimal and uses short, imperative messages (e.g., `init`, `ignore`). There is no formal convention yet; keep messages concise and action‑oriented.

For PRs, include:

- A brief summary of changes and rationale.
- Any new commands or config required to run locally.
- Testing performed (or a note if not applicable).

## Security & Configuration Notes

Planned data locations are documented in `SPEC.md` (e.g., `~/.local/share/gtd-tui/` for local storage). If you add config files (hotkeys, settings), document paths and formats here and avoid committing user‑specific data.
