# cargo-about 0.9.2 drops the license text of a `LicenseRef-*` license, with only a warning

## Symptom

`cargo-about generate --format json --frozen --fail` accepts a crate whose license expression names a project-specific
license such as `LicenseRef-Slint-Royalty-free-2.0`, exits 0, and prints one warning per crate:

```text
# cargo-about --color never generate --format json --frozen --fail --config about.toml --output-file about.json
WARN LicenseRef-Slint-Royalty-free-2.0 has no license file for crate 'slint 1.18.1'
```

The crate is then missing from `licenses[].used_by` in the output, although it is present in `crates[]`.
A notice generator that builds its list from `licenses[]` therefore leaves the crate and its license text out without an
error.
The 10 Slint crates of the music player desktop program (570 crates in all, 560 of them with a text) showed it.

## Cause

`src/generate.rs` of cargo-about 0.9.2 has one arm for SPDX identifiers and one for `LicenseRef-*`
(`LicenseItem::Other`).
The `Other` arm picks a crate's clarified license file with a filter that is the inverse of the `Spdx` arm's:

```rust
// src/generate.rs, LicenseItem::Other arm
if expr.requirements().any(|req| { /* the requirement is the same LicenseRef */ }) {
    return None;
}
```

A file whose clarified expression names the `LicenseRef` is dropped, which is the file it was meant to select.
There is also no canonical-text fallback for a `LicenseRef-*`, so the warning is the only sign.

## What did not work

A `[crate.clarify]` entry with `license = "LicenseRef-Slint-Royalty-free-2.0"`, the file
`LICENSES/LicenseRef-Slint-Royalty-free-2.0.md` and its SHA-256 produced the same ten warnings and the same output
(run on 2026-10-18).
A clarified license that does not name the `LicenseRef` would reach the arm, but it would record a wrong license for
the file, so it is not used.

## Workaround in this repository

`about.toml` of `package/music-player/desktop-app` accepts `LicenseRef-Slint-Royalty-free-2.0`, which keeps the gate:
without that entry (and without `GPL-3.0-only`) the same command exits 1 with
`failed to satisfy license requirements` for each Slint crate.
`bin/third-party-notices.mjs` then finds the crates that `crates[]` lists with a manifest license offering the
`LicenseRef` and that `licenses[]` lacks, requires each crate's own
`LICENSES/LicenseRef-Slint-Royalty-free-2.0.md` to match the SHA-256 of the vendored
`notices/slint-royalty-free-2.0.md`, counts them as covered, and writes the text in a Slint section of the notice file.
Any other crate without a text stops the script.

The 10 warnings still print on every run; they are expected.
No upstream report has been filed.
