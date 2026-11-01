# Enforcing Rust test sidecars

Status: accepted by the user on 2026-10-14 ("Lets enforce rust test sidecars"), option A.
Implementation: branch `feat/lint-tests-in-sidecars`; the record of what shipped is at the end of this document.

## Purpose

Rust unit tests in this repository should always live in a sidecar file,
declared from the source file with `#[cfg(test)] #[path = "<name>_tests.rs"] mod tests;`.
The layout is a recorded pattern in `doc/philosophy/agents.md`, not an enforced rule.
This proposal decides how to enforce it and what the rule must allow.

## Evidence

Measured on 2026-10-13 with `git grep` over tracked `*.rs` files.

- 540 files contain `#[cfg(test)]`.
- 526 files contain a `#[path = ...]` attribute.
  Not every one is necessarily a test sidecar; the count is an upper bound on the sidecar convention.
- Four files (455 lines in total, tests and source together) hold an inline `mod tests { ... }` block:
  `package/nested-wayland-session/cli/src/notification_mode.rs`,
  `package/file-manager/variant-qt/src/cxxqt_object.rs`,
  `package/ide/app/src/selection_paint.rs`
  and `package/linter/monochromatic-lint/src/markdown_block_start.rs`.
- Five files outside any `cfg(test)` module carry `#[test]` functions:
  the `package/linter/monochromatic-lint.fuzz/src` files `explicit_types.rs`, `lib.rs`, `markdown.rs`,
  `orchestration.rs` and `rust_style.rs`,
  plus the probe `doc/troubleshooting/slint-listview-random-seek-probe.rs`.
- 101 tracked files sit under a `tests/` directory (integration test targets),
  75 of them in `package/ide/app`.

## What native Clippy can do

Clippy 0.1.101 (`clippy-driver -W help`, probed on a scratch crate at `~/temp/agent/clippy-probe-20261013`).

- `clippy::inline_modules` (restriction, off by default) flagged a plain inline module,
  an inline `mod tests` without `cfg(test)`, and a feature-gated inline module.
  It did not flag an inline `#[cfg(test)]` module, with or without `#[test]` functions inside it.
  The exemption is documented in the lint's source, `clippy_lints/src/module_style.rs`:
  inline test modules and modules nested in them are not linted.
  So the lint cannot enforce this proposal.
- `clippy::tests_outside_test_module` requires tests to sit in a `#[cfg(test)]` module.
  A sidecar passes it, so it neither helps nor conflicts.
- The `disallowed_*` family covers methods, macros, types, names and fields, not items or attributes,
  and `clippy.toml` has no setting for module layout.

Conclusion: native Clippy cannot enforce the sidecar rule.

## Options

### A. A `rust/tests-in-sidecars` rule in `monochromatic-lint`

The linter already ships `rust/max-lines`, `rust/require-rustdoc`, `rust/no-anonymous-functions`
and `rust/require-explicit-types`, with access to the parsed source.
Note of 2026-10-18:
`rust/no-anonymous-functions` was removed afterwards,
see `doc/decision/rust-closure-lint-dylint.md`.
The rule reports two shapes:

- a `mod` item with an inline body that carries `#[cfg(test)]` (including `cfg(all(test, ...))`);
- a `#[test]` function (any `...::test` attribute) in a file that no `#[cfg(test)] mod name;` declaration loads.

Pros: exact semantics, runs at commit time through the wrapper, independent of Clippy.
Cons: a new rule to maintain, with its own tests, mutation run and linter release.

### B. Clippy lints only

Not viable, see the section on native Clippy.

### C. A Clippy fork or an upstream option

Pros: native.
Cons: the maintainers exempted `cfg(test)` on purpose, and nothing is available today.

### D. A written rule in `AGENTS.md` only

Pros: cheap, immediate.
Cons: nothing enforces it.

### E. A `ripgrep` check in a mise task

Pros: fast to write.
Cons: textual matching cannot tell an inline `mod x {` from a declaration reliably.

Ranking: A > E > D > C > B.
A beats E because it works on the parse tree, which the linter already has.
E beats D because a check can fail a commit and a written rule cannot.
D beats C because it exists today while C needs upstream agreement.
C beats B because B cannot detect the case at all.

## Decisions adopted

The two open questions were settled by what a rule can see, and the carve-outs follow the measured repository.

- A rule receives one parsed file and cannot see which file declares it,
  so a sidecar is recognized by its name: `*_tests.rs`, `tests.rs`, or any file under a `tests` directory.
  Measured over the tracked tree, every sidecar already follows this name, and the rule reported no false positive
  among the more than five hundred sidecar files.
- Exemptions use the existing configuration, not crate-name logic:
  a later block sets `rust/tests-in-sidecars` to `off` for the `*.fuzz` crates and `doc/troubleshooting`,
  in the same style as the existing `rust-exemptions` block, which leaves their other rules selected.
- Doctests live in doc comments and are out of scope, as are virtual files of embedded sources.
- Integration tests under `tests/` are allowed.
- Test-only helper functions without `#[test]` are not reported.
- Severity is `error` from the start, enabled only after the four inline modules were migrated.

## Migration

Done on the branch, one commit per package: the inline test modules of
`package/nested-wayland-session/cli/src/notification_mode.rs`,
`package/file-manager/variant-qt/src/cxxqt_object.rs`,
`package/ide/app/src/selection_paint.rs`
and `package/linter/monochromatic-lint/src/markdown_block_start.rs`
moved to `<name>_tests.rs` sidecars declared with `#[cfg(test)] #[path = "<name>_tests.rs"] mod tests;`.
Not compiled here: `file-manager-qt` (needs the Qt libraries) and, until the container run, `ide` and
`nested-wayland-session`; the moved bodies are byte-for-byte the old ones with one level of indentation removed.

## Measurement

The rule, built from the branch, was run over the whole repository with a scratch configuration.
Before the migration it reported 18 findings: the 4 inline modules, 9 test functions in the `monochromatic-lint.fuzz`
crate and 5 in the `doc/troubleshooting` probe, and nothing else.
After the migration, without exemptions, 15 findings remained (the fuzz crate gained one control test);
with the two exemptions, none, and the run exited 0.

## Record of what shipped

The rule shipped as `rust/tests-in-sidecars` in `monochromatic-lint` 0.1.3 (rule in `3124762a4e`, version bump in
`de9bd3a71a`, merge of `main` in `c954f74798`); crates.io lists 0.1.3.
The repository's own configuration turns the rule on at `error` in `5a85f0c8d1` (`monochromatic-lint.config.jsonc`),
with one exemption block that switches it off for the fuzz crates' smoke tests under `**/fuzz/**` and `**/*.fuzz/**`
and for the recorded reproductions under `doc/troubleshooting/`.
The four inline test modules the evidence section counted were migrated to sidecars before the rule was enabled.
The release age rule exempts the repository's own published crates by exact name (`4059fc9925`), so a freshly
published linter or wrapper installs through `mise upgrade <tool>` without waiting the 24 hours other tools wait.
