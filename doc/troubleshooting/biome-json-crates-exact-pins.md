# Biome JSON crates 0.5.7 from crates.io do not compile because later patch releases of their Biome dependencies break the API

## Symptom

A new crate that depends on [`biome_json_parser`][biome] 0.5.7 and `biome_json_syntax` 0.5.7 from crates.io,
the latest published versions,
fails `cargo build` with a fresh lockfile.
Cargo resolves the caret requirements to `biome_rowan` 0.5.8,
`biome_parser` 0.5.8,
and `biome_unicode_table` 0.5.9,
and the build fails in stages as each is pinned back.

With default resolution,
`biome_json_syntax` fails first:

```text
error[E0046]: not all trait items implemented, missing: `is_trivia`
  --> .../biome_json_syntax-0.5.7/src/lib.rs:46:1
   |
46 | impl biome_rowan::SyntaxKind for JsonSyntaxKind {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `is_trivia` in implementation

error[E0618]: expected function, found `FileSourceError`
  --> .../biome_json_syntax-0.5.7/src/file_source.rs:62:28
   |
62 |             .ok_or_else(|| FileSourceError::MissingFileName(path.into()))?
error: could not compile `biome_json_syntax` (lib) due to 6 previous errors
```

The `E0618` error repeats for `file_source.rs:64`,
`:68`,
`:70`,
and `:88`.

After `cargo update --package biome_rowan@0.5.8 --precise 0.5.7`,
`biome_parser` 0.5.8 fails instead:

```text
error[E0599]: no method named `is_trivia` found for type parameter `K` in the current scope
   --> .../biome_parser-0.5.8/src/lexer.rs:418:37
error[E0599]: no method named `is_trivia` found for type parameter `K` in the current scope
   --> .../biome_parser-0.5.8/src/lexer.rs:429:40
error[E0599]: no method named `is_trivia` found for associated type `<Lex as Lexer<'l>>::Kind` in the current scope
   --> .../biome_parser-0.5.8/src/lexer.rs:635:29
error[E0277]: `SendNode` doesn't implement `std::fmt::Debug`
   --> .../biome_parser-0.5.8/src/lib.rs:656:5
error: could not compile `biome_parser` (lib) due to 4 previous errors
```

After also pinning `biome_parser` to 0.5.7,
`biome_json_parser` fails:

```text
error[E0004]: non-exhaustive patterns: `biome_unicode_table::Dispatch::DOL` not covered
   --> .../biome_json_parser-0.5.7/src/lexer/mod.rs:307:15
    |
307 |         match dispatched {
    |               ^^^^^^^^^^ pattern `biome_unicode_table::Dispatch::DOL` not covered
error: could not compile `biome_json_parser` (lib) due to 1 previous error
```

Pinning `biome_unicode_table` to 0.5.7 as well makes the build succeed.

The structured-edit research for `meow`
(`doc/planning/monorepo-manager-route-research/rust-structured-edits.md`,
sections "Probe results by candidate" and "J3")
hit the same failures and pinned seven crates.
Three of those pins are required;
see "Readings this investigation corrected".

## Root cause

### The crates.io release history

Measured from the crates.io API on 2026-09-17:

- `biome_json_parser`,
  `biome_json_syntax`,
  and `biome_json_factory`:
  latest 0.5.7,
  published 2024-03-12.
- `biome_rowan`,
  `biome_parser`,
  `biome_console`,
  `biome_diagnostics`,
  `biome_text_edit`,
  `biome_text_size`,
  and `biome_string_case`:
  latest 0.5.8,
  published 2024-12-18.
- `biome_unicode_table`:
  latest 0.5.9,
  published 2024-12-18.
- `biome_css_parser`,
  `biome_css_syntax`,
  and `biome_css_factory`:
  latest 0.5.8,
  published 2024-12-18.

The December 2024 release refreshed the CSS crates and their shared dependencies but not the JSON crates.
No Biome crate has a crates.io release after 2024-12-18.

`biome_json_parser` 0.5.7 declares its Biome dependencies with plain version strings,
which Cargo reads as caret requirements
(`Cargo.toml` of the published crate):

```toml
[dependencies.biome_parser]
version = "0.5.7"

[dependencies.biome_rowan]
version = "0.5.7"

[dependencies.biome_unicode_table]
version = "0.5.7"
```

Cargo's SemVer guide treats a `0.y.z` patch bump as compatible:
"Initial development releases starting with '0.y.z' can treat changes in 'y' as a major release,
and 'z' as a minor release"
([Cargo SemVer compatibility][cargo-semver]).
So `^0.5.7` selects 0.5.8 and 0.5.9.

### `biome_rowan` 0.5.8 adds a required trait method

`biome_rowan` 0.5.8,
`src/syntax.rs:47-48`,
inside `pub trait SyntaxKind` (`:25`):

```rust
    /// Returns `true` if this kind is a trivia.
    fn is_trivia(self) -> bool;
```

The method has no default body.
`biome_json_syntax` 0.5.7 implements the trait without it (`src/lib.rs:46`),
which is the `E0046` error.
The Cargo SemVer guide lists this as
"Major:
 adding a non-defaulted trait item".

### `biome_rowan` 0.5.8 removes variant fields from `FileSourceError`

`biome_rowan` 0.5.7 declared `MissingFileName(PathBuf)`,
`MissingFileExtension(PathBuf)`,
and `UnknownExtension(String, String)`.
`biome_rowan` 0.5.8,
`src/file_source.rs:3-16`:

```rust
/// Errors around the construct of the source type
#[derive(Debug)]
pub enum FileSourceError {
    /// The path has no file name
    MissingFileName,
    /// The path has no file extension
    MissingFileExtension,
    /// The source type is unknown
    UnknownExtension,
    /// The file name is unknown (not a well-known file name)
    UnknownFileName,
    /// The language id is unknown
    UnknownLanguageId,
}
```

`biome_json_syntax` 0.5.7 still calls the tuple forms
(`src/file_source.rs:62`):

```rust
            .ok_or_else(|| FileSourceError::MissingFileName(path.into()))?
```

which is the `E0618` error.

### `biome_unicode_table` 0.5.9 adds a variant to an exhaustive enum

`biome_unicode_table` 0.5.9,
`src/bytes.rs:3-23`:

```rust
/// Every handler a byte coming in could be mapped to
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Dispatch {
    /// Error token
    ERR,

    /// Whitespace
    WHS,

    /// Exclamation
    EXL,

    /// Single `'` or Double quote `"`
    QOT,

    /// ASCII letter or `_`
    IDT,

    /// Dollar sign `$`
    DOL,
```

`Dispatch` has no `#[non_exhaustive]`.
`biome_json_parser` 0.5.7 matches it exhaustively without a wildcard
(`src/lexer/mod.rs:307-337`,
abridged):

```rust
        match dispatched {
            WHS => self.consume_newline_or_whitespaces(),
            QOT => self.lex_string_literal(current),
            IDT => self.lex_identifier(current),
            // ...
            ERR | EXL | HAS | TLD | PIP | TPL | CRT | BSL | AT_ | QST | MOR | LSS | SEM | MUL
            | PNO | PNC | PRD | PRC | AMP | EQL | PLS => self.eat_unexpected_character(),
        }
```

which is the `E0004` error.
The Cargo SemVer guide lists this as
"Major:
 adding new enum variants (without `non_exhaustive`)".

### The 0.5.8 crates under-declare their own minimums

`biome_parser` 0.5.8 still declares `biome_rowan` as `version = "0.5.7"` in its published `Cargo.toml`,
yet calls the 0.5.8-only method (`src/lexer.rs:418`):

```rust
        if !checkpoint.current_kind.is_trivia() {
```

So the resolution `biome_parser` 0.5.8 with `biome_rowan` 0.5.7
satisfies every declared requirement and still does not compile,
which is why pinning `biome_rowan` alone fails.
`biome_css_syntax` 0.5.8 has the same gap:
with `biome_rowan` held at 0.5.7 it fails with
"method `is_trivia` is not a member of trait `biome_rowan::SyntaxKind`" (`src/lib.rs:151`)
and "no variant ... named `UnknownFileName` found for enum `FileSourceError`" (`src/file_source.rs:38`).

### The upstream source has moved on under the same version number

At Biome `main` (`2b5cd1ed014f938f6d165990dcb9a765b0231d0e`,
2026-09-17):

- `crates/biome_json_parser/Cargo.toml:3` still says `version = "0.5.7"` and `:13` says `publish = true`;
  the root `Cargo.toml:76`,
  `:92`,
  `:97`,
  and `:111` pin `biome_json_parser`,
  `biome_parser`,
  `biome_rowan`,
  and `biome_unicode_table` at `version = "0.5.7"`.
- The source already matches the newer API:
  `crates/biome_json_syntax/src/lib.rs:79` implements `fn is_trivia(self) -> bool`,
  and `crates/biome_json_parser/src/lexer/mod.rs:360` matches `IDT | DOL => self.lex_identifier(current)`.
- `crates/biome_json_parser/Cargo.toml` depends on `biome_languages` 0.1.0 (root `Cargo.toml:116`),
  which returns HTTP 404 from `https://crates.io/api/v1/crates/biome_languages`.

So the fix is a publishing action,
not a source change:
a coherent release of the JSON crates against the 0.5.8 set,
a new version of the whole set,
or yanking the December 2024 versions,
which would also strand the CSS 0.5.8 crates.

### Does Biome still support crates.io consumers

Evidence,
in the order it bears on the question:

- The maintainer's reply on [biomejs/biome#5151][biome-5151],
  which reports this exact failure for `biome_json_syntax`:
  "We don't guarantee the usage of our crates around these cases,
  but if this happens,
  you're free to send a PR to fix those cases."
  The issue is open,
  labeled `S-Good first issue` and `S-Help-wanted`,
  with no later maintainer activity (timeline read 2026-09-17).
- Publishing infrastructure exists and has been maintained.
  [#7769][biome-7769] (merged 2025-10-17) added
  `.github/workflows/publish-crates.yml` and `scripts/update-crates-versions.mjs`,
  stating "We have one single version for all crates.
  New crates get published with the version we already have."
  [#7801][biome-7801] (merged 2025-10-20) switched the workflow to token publishing.
  [#11123][biome-11123] (merged 2026-07-29) trimmed "published crate sizes" for 35 crates.
- The workflow has never published anything.
  `gh api repos/biomejs/biome/actions/workflows/publish-crates.yml/runs` lists 17 runs,
  all `failure`,
  all `push` events between 2025-10-20 and 2025-10-22;
  the current file triggers only on `workflow_dispatch`
  (`.github/workflows/publish-crates.yml:2-3`),
  and no run exists after `4c5ea0682 ci: fix workflow` (2025-10-22).
  crates.io shows no release after 2024-12-18.
- At `main`,
  `git grep` finds `publish = true` in 70 crate manifests,
  and `crates/biome_json_parser/README.md:8` still shows a crates.io version badge.
- All 39 crates.io crates named `biome_*` were last updated on or before 2024-12-18
  (crates.io search API,
  every result page).

Conclusion:
 Biome intends crates.io publishing and keeps the tooling current,
 but it does not guarantee crates.io consumers,
 has not released since 2024-12-18,
 and has left this known breakage open since 2025-02-18.
Treat the crates.io Biome crates as a frozen,
 unsupported snapshot.

### Readings this investigation corrected

- The research recorded that the build "compiled only after `cargo update ... --precise 0.5.7`" for seven crates.
  Only `biome_rowan`,
  `biome_parser`,
  and `biome_unicode_table` must be pinned for `biome_json_parser` and `biome_json_syntax`;
  `biome_console`,
  `biome_diagnostics`,
  `biome_text_edit`,
  and `biome_text_size` compiled at 0.5.8 ("Three lockfile pins").
  Pinning all seven also builds ("Seven lockfile pins").

## Verification

Versions under test:

- `biome_json_parser` 0.5.7
  (checksum `9c6d23fb9b683e6356c094b4a0cb38f8aa0acee60ce9c3ef24628d21a204de4d`)
  and `biome_json_syntax` 0.5.7
  (checksum `f2645ca57f75680d3d390b2482c35db5850b1d849e1f96151a12f15f4abdb097`),
  both published 2024-03-12.
- `biome_rowan` 0.5.8,
  `biome_parser` 0.5.8,
  and `biome_unicode_table` 0.5.9,
  published 2024-12-18,
  against their 0.5.7 releases
  (checksums `d3c2dc25a7ba6ae89526340034abed6c89fac35b79060786771e32ed4aac77e7`,
  `955dd999f32c086371d5c0e64b4ea1a50f50c98f1f31a3b9fe17ef47198de19b`,
  and `87e8604d34b02180a58af1dbdaac166f1805f27f5370934142a3246f83870952`).
- `rustc 1.97.1 (8bab26f4f 2026-07-14)` in `docker.io/library/rust:1.97-bookworm`
  (`sha256:389c1ae98c20fbcadca68a685482749267cec3c90893ae4671c5a37cc894c416`),
  run on 2026-09-17.

### Harness

```toml
# biome-json-crates-repro/Cargo.toml
[package]
name = "biome_json_crates_repro"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
biome_json_parser = "0.5.7"
biome_json_syntax = "0.5.7"
```

```rust
// biome-json-crates-repro/src/main.rs
use biome_json_parser::{parse_json, JsonParserOptions};

fn main() {
    let options = JsonParserOptions::default().with_allow_comments().with_allow_trailing_commas();
    let cases = [
        "{ \"a\": 1, }",
        "{ /* block */ \"a\": 1 }",
        "{\n  \"compilerOptions\": {\n    /* strictness */\n    \"strict\": false, // todo\n  },\n}\n",
        "{ a: 1 }",
    ];
    for src in cases {
        let parse = parse_json(src, options);
        println!(
            "errors={} lossless={} {src:?}",
            parse.has_errors(),
            parse.syntax().to_string() == src
        );
    }
}
```

Every command ran in a container with no credentials and no repository mount,
with a throwaway `CARGO_HOME` inside the harness directory:

```sh
podman run --rm --memory=2g --cpus=2 \
  --volume "${HARNESS}:/work:Z" --workdir /work --env CARGO_HOME=/work/cargo-home \
  docker.io/library/rust:1.97-bookworm cargo build --jobs 2
```

`cargo update` steps used the same `podman run` prefix.

### Resolutions that fail

- Fresh lockfile,
  caret requirements:
  `E0046` and five `E0618` errors in `biome_json_syntax` 0.5.7 ("Symptom").
- `cargo update --package biome_rowan@0.5.8 --precise 0.5.7`
  (Cargo reported `Downgrading biome_rowan v0.5.8 -> v0.5.7`):
  three `E0599` errors and one `E0277` error in `biome_parser` 0.5.8.
- Then `cargo update --package biome_parser@0.5.8 --precise 0.5.7`:
  `E0004` for `Dispatch::DOL` in `biome_json_parser` 0.5.7.
- `biome_parser`,
  `biome_rowan`,
  and `biome_unicode_table` held at `=0.5.7` in `Cargo.toml` plus `biome_css_parser = "0.5.8"`:
  six errors in `biome_css_syntax` 0.5.8,
  including:

```text
error[E0407]: method `is_trivia` is not a member of trait `biome_rowan::SyntaxKind`
   --> .../biome_css_syntax-0.5.8/src/lib.rs:151:5
error[E0599]: no variant, associated function, or constant named `UnknownFileName` found for enum `FileSourceError` in the current scope
  --> .../biome_css_syntax-0.5.8/src/file_source.rs:38:30
error: could not compile `biome_css_syntax` (lib) due to 6 previous errors
```

### Resolutions that build

Each printed the same result for the harness cases:

```text
errors=false lossless=true "{ \"a\": 1, }"
errors=false lossless=true "{ /* block */ \"a\": 1 }"
errors=false lossless=true "{\n  \"compilerOptions\": {\n    /* strictness */\n    \"strict\": false, // todo\n  },\n}\n"
errors=true lossless=true "{ a: 1 }"
```

- Three lockfile pins:
  after `cargo update --package biome_unicode_table@0.5.9 --precise 0.5.7`,
  `Cargo.lock` held `biome_console`,
  `biome_diagnostics`,
  `biome_text_edit`,
  and `biome_text_size` at 0.5.8 and every other Biome crate at 0.5.7.
- Seven lockfile pins:
  after also pinning `biome_diagnostics`,
  `biome_console`,
  `biome_text_edit`,
  and `biome_text_size` to 0.5.7,
  every Biome crate in `Cargo.lock` was 0.5.7.
- Manifest pins from no lockfile
  (see "Hold the three broken crates at `=0.5.7` in `Cargo.toml`");
  Cargo printed:

```text
     Locking 89 packages to latest compatible versions
      Adding biome_parser v0.5.7 (available: v0.5.8)
      Adding biome_rowan v0.5.7 (available: v0.5.8)
      Adding biome_unicode_table v0.5.7 (available: v0.5.9)
```

## Verified workarounds

### Hold the three broken crates at `=0.5.7` in `Cargo.toml`

```toml
# consumer Cargo.toml
[dependencies]
biome_json_parser = "0.5.7"
biome_json_syntax = "0.5.7"
# The 2024-12-18 releases of these three break biome_json_* 0.5.7; exact requirements keep them on 0.5.7.
biome_parser = "=0.5.7"
biome_rowan = "=0.5.7"
biome_unicode_table = "=0.5.7"
```

Verified from a deleted `Cargo.lock` and target directory ("Resolutions that build").

Tradeoffs:

- Frozen at the 2024-03-12 source;
  no Biome JSON parser change after that date reaches the build.
- Mixes 0.5.7 and 0.5.8 support crates (`biome_diagnostics`,
  `biome_console`,
  `biome_text_edit`,
  `biome_text_size`),
  a combination Biome never released together;
  it compiled and parsed the harness cases,
  but nothing else was exercised.
- Cannot coexist with `biome_css_parser` 0.5.8 or any other crate that needs the 0.5.8 API
  (last failing resolution in "Resolutions that fail").
- The pins must be direct dependencies even though the code never names these crates.

### Pin all seven support crates

Pinning `biome_console`,
`biome_diagnostics`,
`biome_text_edit`,
and `biome_text_size` to 0.5.7 as well keeps the whole dependency set at one release.
Verified as lockfile pins ("Seven lockfile pins").
Brioche took this route in its `Cargo.toml`
([brioche-dev/brioche#184][brioche-184]:
"I ultimately decided to pin all (transitive) dependencies to v0.5.7").

Tradeoffs:

- Every tradeoff of the three-crate pin.
- More pins to carry and remove later.
- Lockfile-only pins (`cargo update --precise`) are undone by the next plain `cargo update`;
  put them in `Cargo.toml` as `=0.5.7` to make them durable.

## What does not work

- **Default caret requirements.**
  They select the December 2024 releases,
  which break `biome_json_syntax` 0.5.7.
- **Pinning only `biome_rowan`.**
  `biome_parser` 0.5.8 declares `biome_rowan` `0.5.7` but needs 0.5.8's `is_trivia`.
- **Pinning `biome_rowan` and `biome_parser` but not `biome_unicode_table`.**
  `biome_json_parser` 0.5.7 then fails on `Dispatch::DOL`.
- **Mixing Biome's CSS 0.5.8 crates with the JSON 0.5.7 pins.**
  `biome_css_syntax` 0.5.8 needs the 0.5.8 `biome_rowan` API.

## Upstream filing decision

Decision:
 do not open an issue;
 [biomejs/biome#5151][biome-5151] is a duplicate.
The additive comment in "Draft comment (do not post as-is)" is kept as a record,
 but constraint 4 fails for an agent-written comment,
 so any comment must be written by a human in their own words.
Nothing was filed or commented upstream from this session.

`.out-of-scope/` was checked:
 it has no entry for Biome,
 its crates,
 or crates.io packaging.

### Duplicate search

Searched 2026-09-17 with `gh search issues --repo biomejs/biome --include-prs` and `gh search prs`,
 open and closed,
 for `crates.io`,
 `is_trivia`,
 `biome_rowan 0.5.8`,
 `publish crates`,
 `Dispatch DOL`,
 `crates.io version`,
 and `biome_json_parser`.
Threads read in full:

- [#5151][biome-5151] (open,
   2025-02-18):
   "Broken build when pulling some Biome dependencies from crates.io".
  Reproduction:
   `cargo add biome_json_syntax` then `cargo check`,
   with the same `E0046` and `E0618` errors,
   attributed to `biome_rowan` 0.5.8.
  One maintainer comment,
   quoted in "Does Biome still support crates.io consumers".
  Cross-referenced by [brioche-dev/brioche#184][brioche-184].
- [#7769][biome-7769],
   [#7801][biome-7801],
   and [#11123][biome-11123]:
   publishing infrastructure,
   not the breakage.

What the thread lacks,
 and a human comment could add:

- `biome_json_parser` 0.5.7 hits a second break (`Dispatch::DOL`,
   from `biome_unicode_table` 0.5.9) after `biome_rowan` and `biome_parser` are pinned.
- `biome_parser` 0.5.8 and `biome_css_syntax` 0.5.8 declare `biome_rowan` `0.5.7` but need 0.5.8,
   so no single-crate pin works.
- The minimal workaround:
   `=0.5.7` for `biome_parser`,
   `biome_rowan`,
   and `biome_unicode_table`.
- `main` already implements `is_trivia` and `DOL` for JSON,
   so publishing a coherent set fixes new consumers without code changes.

### Six constraints

1.  **Upstream's fault:**
    yes.
    Packaging behavior:
     patch releases added a required trait method,
     removed enum variant fields,
     and added a variant to an exhaustive public enum,
     each a major change under Cargo's SemVer guide,
     and the 0.5.8 crates under-declare their `biome_rowan` minimum.
2.  **Upstream can fix it:**
    yes.
    Publishing the JSON crates against the 0.5.8 set,
     or a new version of every crate through the existing `publish-crates.yml`,
     fixes resolution;
     no architectural limit is involved.
3.  **Supported use case:**
    equivocal.
    The crates are `publish = true`,
     have crates.io badges,
     and have maintained publishing tooling,
     but the maintainer wrote "We don't guarantee the usage of our crates around these cases".
4.  **Contribution welcome:**
    no,
     for anything this session could write.
    `CONTRIBUTING.md:71` says
     "Please do not use AI to write pull request descriptions or contributor communication for this project",
     and `:73` says such pull requests may be closed.
    A comment drafted here is AI-written contributor communication,
     and a fully human-authored comment is not something this session can truthfully provide.
    Biome does accept disclosed AI assistance for code (`CONTRIBUTING.md:47-65`),
     and `.github/agentscan.yml` configures agent detection.
5.  **Likely to fix:**
    soft yes.
    The maintainer invited a pull request and labeled the issue help-wanted,
     and publishing tooling was updated through 2026-07-29;
     but the issue has had no maintainer activity since 2025-02-18,
     and nothing was published.
6.  **Prototype:**
    not triggered,
     because constraint 4 fails.
    The minimal fix is also a release action by maintainers with the `crate-publish` environment
     ([#7769][biome-7769]:
     "only @biomejs/core-contributors can publish and approve the release"),
     not a source patch.

### Draft comment (do not post as-is)

A human who wants to comment must write it themselves;
 these are the facts to draw on.

~~~md
Additional data for this issue (crates.io, fresh lockfile, rustc 1.97.1):

- `biome_json_parser` 0.5.7 has a second break. After
  `cargo update --package biome_rowan@0.5.8 --precise 0.5.7` and the same for `biome_parser`,
  it fails with `error[E0004]: non-exhaustive patterns: biome_unicode_table::Dispatch::DOL not covered`
  at `biome_json_parser-0.5.7/src/lexer/mod.rs:307`, because `biome_unicode_table` 0.5.9 added `DOL`.
- Pinning only `biome_rowan` does not work: `biome_parser` 0.5.8 declares `biome_rowan = "0.5.7"` but calls
  `is_trivia` (`biome_parser-0.5.8/src/lexer.rs:418`, `:429`, `:635`). `biome_css_syntax` 0.5.8 has the same gap.
- Workaround that builds from no lockfile and parses JSONC losslessly:

  ```toml
  biome_json_parser = "0.5.7"
  biome_json_syntax = "0.5.7"
  biome_parser = "=0.5.7"
  biome_rowan = "=0.5.7"
  biome_unicode_table = "=0.5.7"
  ```

- `main` already implements `is_trivia` for `JsonSyntaxKind` and handles `DOL` in the JSON lexer, so publishing
  the JSON crates against the 0.5.8 set (or a new version of the whole set) would fix new consumers without code
  changes.
~~~

[biome]: https://github.com/biomejs/biome
[biome-5151]: https://github.com/biomejs/biome/issues/5151
[biome-7769]: https://github.com/biomejs/biome/pull/7769
[biome-7801]: https://github.com/biomejs/biome/pull/7801
[biome-11123]: https://github.com/biomejs/biome/pull/11123
[brioche-184]: https://github.com/brioche-dev/brioche/pull/184
[cargo-semver]: https://doc.rust-lang.org/cargo/reference/semver.html
