# File-enforcer Rust rewrite

## Status

Being grilled with the user, started 2026-10-29.
Nothing here is accepted yet;
decisions land in this file as the user answers them.

## Request

"Migrate to Rust + clap + config with jsonc for file-enforcer.
Ground up rewrite.
No need to preserve anything." (user, 2026-10-29)

## Constraints stated by the user

- "Consider the original file-enforcer bad in every way and don't treat it as Jesus." (user, 2026-10-29)
  The TypeScript package at `package/file-enforcer/core` is evidence only of which files the repo needs managed.
  Its builder API,
  subsystem boundaries,
  CLI shape,
  staleness manifest,
  locking,
  and config-as-TypeScript model are not requirements and are not a design to port.

## Decisions

### Standalone tool that meow can embed

Asked 2026-10-29 how the rewrite relates to meow,
whose accepted decision (`doc/decision/monorepo-manager-all-rust.md`, 2026-09-16)
puts file enforcement inside the single meow binary with an HCL config and daemon-owned watch mode.
The user chose "Standalone, meow can embed".

- The engine is a library crate and the CLI is a clap binary over it;
  meow may depend on the library later instead of rewriting it.
- The tool has its own watch mode until a meow daemon exists.
- The config is JSONC,
  following the cli-git and linter rewrites (`doc/decision/cli-git-rust-rewrite.md`),
  which also moved off HCL.
- Consequence to record when this is accepted:
  amend `doc/decision/monorepo-manager-all-rust.md` so meow links this crate and its file-enforcement config is JSONC.

Evidence at the time:
meow is plan-only with no code
(`doc/handover/monorepo-manager.md`, "Design research; no product code"),
last designed 2026-09-21,
and issue #1799 prefers an existing tool over building a monorepo manager.

### Adopted from the request and repository rules

- Rust is approved for this tool by the request itself;
  no existing approval scope (meow, cli-git, the linter) covers a standalone file-enforcer.
- The CLI uses clap,
  as requested;
  repository precedent is clap derive in `package/forbidden-strings/cli` and others.
- Every Rust dependency comes from `rust-catalog.toml` through `workspace = true`
  (`doc/decision/rust-dependency-catalog.md`).
- Rust tests run through nextest (`RNQ` in `AGENTS.md`).
- Every line the tool writes is one JSON object (`schemaVersion`, `sequence`, `type`, `code`, `message`, `hint`),
  matching meow's accepted "Emit every line as a json, error or warning or not." (user, 2026-09-17)
  and the live cli-git event stream (`package/git-policy/cli/src/native/diagnostics.rs:223`).
  Adopted unasked because the library is meant for meow to embed;
  open to the user's veto.
- The TypeScript file-enforcer keeps running until the Rust tool replaces it;
  concurrent edits in `package/file-enforcer/core` belong to other sessions.

## Open questions

Tracked in the grilling rounds;
each answered question moves into a decisions section with the user's words and date.
