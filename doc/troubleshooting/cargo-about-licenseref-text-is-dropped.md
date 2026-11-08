# cargo-about 0.9.2 drops the license text of a `LicenseRef-*` license, with only a warning

## Symptom

`cargo-about generate --format json --frozen --fail` accepts a crate whose license expression names a project-specific
license such as `LicenseRef-Slint-Royalty-free-2.0`,
exits 0,
and prints one warning per crate:

```text
# cargo-about --color never generate --format json --frozen --fail --config about.toml --output-file about.json
WARN LicenseRef-Slint-Royalty-free-2.0 has no license file for crate 'slint 1.18.1'
```

The crate is then missing from `licenses[].used_by` in the output,
although it is present in `crates[]`.
A notice generator that builds its list from `licenses[]` therefore leaves the crate and its license text out without an
error.
The 10 Slint crates of the music player desktop program (570 crates in all,
560 of them with a text) showed it.

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

A file whose clarified expression names the `LicenseRef` is dropped,
which is the file it was meant to select.
There is also no canonical-text fallback for a `LicenseRef-*`,
so the warning is the only sign.

## What did not work

A `[crate.clarify]` entry with `license = "LicenseRef-Slint-Royalty-free-2.0"`,
the file
`LICENSES/LicenseRef-Slint-Royalty-free-2.0.md` and its SHA-256 produced the same ten warnings and the same output
(run on 2026-10-18).
A clarified license that does not name the `LicenseRef` would reach the arm,
but it would record a wrong license for
the file,
so it is not used.

## Workaround in this repository

`about.toml` of `package/music-player/desktop-app` accepts `LicenseRef-Slint-Royalty-free-2.0`,
which keeps the gate:
without that entry (and without `GPL-3.0-only`) the same command exits 1 with
`failed to satisfy license requirements` for each Slint crate.
`bin/third-party-notices.mjs` then finds the crates that `crates[]` lists with a manifest license offering the
`LicenseRef` and that `licenses[]` lacks,
requires each crate's own
`LICENSES/LicenseRef-Slint-Royalty-free-2.0.md` to match the SHA-256 of the vendored
`notices/slint-royalty-free-2.0.md`,
counts them as covered,
and writes the text in a Slint section of the notice file.
Any other crate without a text stops the script.

The 10 warnings still print on every run;
they are expected.
No upstream report has been filed.

## The vendored copy is verbatim, and a rewrap breaks its checksum

`bin/third-party-notices.mjs` then fails with
`notices/slint-royalty-free-2.0.md does not match the checksum recorded in this script; review it first`
whenever the vendored file's bytes differ from the SHA-256 it pins,
and the neighbouring check fails with `<crate> does not ship <file> with the vendored license text` when a Slint
crate's own copy differs from the vendored one.
The message naming a crate can only report an upstream change;
the message naming the vendored file can also report a change made here.

The Markdown whitespace fixer did exactly that twice,
through the `markdown/semantic-line-breaks` rule of
`mise run format:monochromatic-lint`.
It reflowed the file in 5513294572 and again in 2f369455ca on 2026-11-07;
5b9e9be057 had restored the bytes in between,
and the `notices` job of the release dry run 42448767356 failed on the second reflow.
The license itself did not change:
every Slint 1.18.1 crate still ships the file with
SHA-256 `5167f5056e850419106ab6265efbdca7cba4d99c849d1445ca0bbf6a1e2315fe`.
Pinning the constant to a reflowed copy would therefore fail the check that compares each crate's file with it.

The root `monochromatic-lint.config.jsonc` therefore exempts
`package/music-player/desktop-app/notices/` from `markdown/semantic-line-breaks` alone.
Every other Markdown rule still judges those files,
and the checksum stays the check that protects them.
A reflowed copy is restored from the revision that had the plain bytes
(`git checkout 5b9e9be057 -- package/music-player/desktop-app/notices/slint-royalty-free-2.0.md`);
changing the constant is not the fix.
