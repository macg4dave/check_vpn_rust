# Project — Copilot / AI Agent Instructions (Template)
## Enter project info here
Purpose: Provide concise, project-wide context and expectations for an AI coding agent working in this Rust repository.

## Project Overview

* Rust application: `<brief project summary here>`
* Architecture: `<CLI / TUI / web service / library / whatever>`
* Goals: `<performance / correctness / portability / simplicity>`

## Folder Structure (fill in as appropriate)

* `/src/` — main source code
* `/tests/` — integration tests
* `/crates/<name>/` — workspace members (if any)
* `/docs/` — documentation
* `/scripts/` — helper scripts
* `.github/instructions/` — prompt templates or path-specific instructions

## Tools & Environment

* Language: Rust (stable recommended).
* Commands:

  * Build: `cargo build`
  * Test: `cargo test`
  * Run: `cargo run`
* Recommended: `rust-analyzer`, `cargo fmt`, `clippy`.

## Coding Standards

* Follow `rustfmt` style.
* Prefer clear, idiomatic Rust.
* Use `Result` for error handling; no sloppy `unwrap()` outside tests/examples.
* Keep changes minimal and modular.
* Document public APIs with Rustdoc.
* Add comments for complex logic.
* Avoid `unsafe` unless justified and tested.
* Avoid dead code; remove any unused items unless intentionally left for future modules.

## Testing & PR Expectations

* All changes must pass `cargo test` before submission.
* Any behavioral change must include new or updated tests.
* Maintain existing public APIs unless explicitly allowed to change.
* Keep patches focused and avoid large refactors unless requested.
* Update documentation and CHANGELOG for user-facing changes.

## Security & External Resources

* Do not fetch private or non-public resources.
* Ask for missing data rather than guessing critically important details.

## Response Preferences for AI

* Provide a short plan before writing code.
* Submit minimal code patches that compile.
* Use idiomatic Rust patterns (iterators, pattern matching, etc.).
* Tests should accompany functionality changes.

## Hard Requirements (include in every instruction file)

* Run the test suite locally and include full output.
* Apply the smallest-possible change that solves the problem.
* Add or update tests for any altered behavior.
* Preserve CLI and API semantics unless explicitly allowed to modify.
* No removal of features/tests without migration notes and replacement tests.
