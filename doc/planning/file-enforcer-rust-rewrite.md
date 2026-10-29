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

## Known precedent to reconcile

- `doc/decision/monorepo-manager-all-rust.md` (accepted 2026-09-16) puts a Rust rewrite of file-enforcer inside the
  single-file monorepo manager placeholder-named meow,
  configured in OpenTofu-shaped HCL
  (`doc/decision/monorepo-manager-hcl-front-end.md`).
  How this request relates to that decision is an open question for the user.

## Open questions

Tracked in the grilling rounds;
each answered question moves into a decisions section with the user's words and date.
