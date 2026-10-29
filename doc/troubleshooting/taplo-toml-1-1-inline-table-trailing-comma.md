# taplo 0.14.0 rejects TOML 1.1 inline-table trailing commas with "expected value, trailing comma is not allowed"

`taplo` 0.14.0,
the latest `taplo` release on crates.io (published 2025-05-22),
reports a syntax error for `clap = { version = "3", features = [ "derive", ], }`,
a TOML 1.1 inline table that Cargo 1.98.1 accepts
and that `module-toml-edit` emits when it creates an inline table
(`package/toml-edit/core/src/toml-set.unit.test.ts:484` expects `x = { y = 1, }`).
It surfaced as research case `t20` in
[the Rust structured-edit research](../planning/monorepo-manager-route-research/rust-structured-edits.md)
("Probe results by candidate",
 "TOML").

This is a spec-version gap,
not a parser bug:
`taplo` 0.14.0 implements TOML 1.0,
which forbids the form,
and it was released before TOML 1.1.0 existed.
Support for trailing commas and newlines in inline tables landed on `tamasfe/taplo` master
in [tamasfe/taplo#849][pr-849] (merged 2026-03-11)
but is not in any crates.io release as of 2026-09-17.
Other TOML 1.1 syntax (`\e` and `\xHH` escapes,
 times without seconds) is still rejected on master.

## Symptom

`taplo::parser::parse` returns a non-empty `errors` vector.
The research probe printed the first error of case `t20` as:

```text
parse: Some(Error { range: 50..51, message: "expected value, trailing comma is not allowed" })
```

The diagnostic differs by surface form:

- Single-line inline table ending in a comma
   (`clap = { version = "3", features = [ "derive", ], }`,
   `x = { y = 1, }`):
   exactly one error,
   `expected value, trailing comma is not allowed`,
   at the closing `}`.
  The syntax tree still round-trips byte for byte,
   the DOM validates,
   and values read back correctly.
- Inline table spanning lines (`t = {` / `  a = 1,` / `  b = 2,` / `}`):
   `newline is not allowed in an inline table`,
   followed by `expected new line` and `expected identifier`;
   the DOM does not validate.
- Other TOML 1.1 syntax:
   `csi = "\e["` and `a = "\x61"` report `invalid escape sequence`;
   `t = 14:15` reports `unexpected token` and `expected new line`.

A trailing comma inside an array (`features = [ "derive", ]`) is valid TOML 1.0 and parses cleanly;
only the comma before the closing `}` triggers the error.

## Root cause

### taplo 0.14.0 parses inline tables with the TOML 1.0 grammar

Citations are `tamasfe/taplo` at tag `release-taplo-0.14.0` (commit `15963fc`),
identical to the published crate source
(`diff --recursive --brief crates/taplo/src` against the Cargo registry copy printed nothing).

`parse_inline_table` records whether the last token was a comma
and reports an error when `}` follows it,
and reports another error for any newline inside the braces
(`crates/taplo/src/parser/mod.rs:788-820`):

```rust
// crates/taplo/src/parser/mod.rs:788-820 (release-taplo-0.14.0)
fn parse_inline_table(&mut self) -> ParserResult<()> {
    self.must_token_or(BRACE_START, r#"expected "{""#)?;

    let mut first = true;
    let mut comma_last = false;
    let mut was_newline = false;

    loop {
        let t = match self.get_token() {
            Ok(t) => t,
            Err(_) => return self.report_error(r#"expected "}""#),
        };

        match t {
            BRACE_END => {
                if comma_last {
                    // it is still reported as a syntax error,
                    // but we can still analyze it as if it was a valid
                    // table.
                    let _ = self.report_error("expected value, trailing comma is not allowed");
                }
                break self.add_token()?;
            }
            NEWLINE => {
                // To avoid infinite loop in case
                // new lines are whitelisted.
                if was_newline {
                    break;
                }

                let _ = self.error("newline is not allowed in an inline table");
                was_newline = true;
            }
```

The two diagnostics go through different helpers.
`report_error` only records the error (`crates/taplo/src/parser/mod.rs:160-171`),
so the trailing-comma case keeps its normal tree shape;
`error` also re-emits the current token as an `ERROR` token unless it is whitelisted
(`crates/taplo/src/parser/mod.rs:123-158`),
and the harness case `multi-line-inline-table` then shows follow-on `expected new line`
and `expected identifier` errors and a DOM that fails validation:

```rust
// crates/taplo/src/parser/mod.rs:160-171 (release-taplo-0.14.0)
// report error without consuming the current the token
fn report_error(&mut self, message: &str) -> ParserResult<()> {
    let span = self.lexer.span();
    self.add_error(&Error {
        range: TextRange::new(
            TextSize::from(span.start as u32),
            TextSize::from(span.end as u32),
        ),
        message: message.into(),
    });
    Err(())
}
// crates/taplo/src/parser/mod.rs:148-152 (inside `error`)
if let Some(t) = self.current_token {
    if !self.whitelisted(t) {
        self.token_as(ERROR).ok();
    }
}
```

The rejection was tested as intended behavior:
the generated invalid-input suite at that tag asserts it
(`crates/taplo/src/tests/generated/invalid.rs:68-72`):

```rust
// crates/taplo/src/tests/generated/invalid.rs:68-72 (release-taplo-0.14.0)
fn inline_table_trailing_comma() {
    let src = "abc = { abc = 123, }\n";
    let p = crate::parser::parse(src);
    assert!(!p.errors.is_empty() || p.into_dom().validate().is_err());
}
```

The other TOML 1.1 gaps have the same shape.
The escape lexer lists only TOML 1.0 escapes,
with no `\e` or `\xHH` token (`crates/taplo/src/util/escape.rs:3-53`),
and the `TIME` token requires seconds (`crates/taplo/src/syntax.rs:76-77`):

```rust
// crates/taplo/src/syntax.rs:76-77 (release-taplo-0.14.0)
#[regex(r#"(?:[01]\d|2[0-3]):[0-5]\d:[0-5]\d(?:(?:\.|,)\d+)?"#)]
TIME,
```

### Spec-version gap, not a parser bug

TOML 1.0.0 forbids both forms (`toml-lang/toml` `toml.md:802-807` at tag `1.0.0`):

> Inline tables are intended to appear on a single line.
 A terminating comma (also
> called trailing comma) is not permitted after the last key/value pair in an
> inline table.
 No newlines are allowed between the curly braces unless they are
> valid within a value.

TOML 1.1.0 permits them (`toml.md:831-832` at tag `1.1.0`,
changelog entry "Allow newlines and trailing commas in inline tables ([#904])"):

> A terminating comma (also called trailing comma) is
> permitted after the last key/value pair.

TOML 1.1.0 is dated 2025-12-18 in its changelog and was published as a GitHub release on 2025-12-24.
`taplo` 0.14.0 was published on 2025-05-22,
its repository README at that tag calls it "a TOML v1.0.0 toolkit" (`README.md:11`),
and its CI ran `toml-test` v1.1.0 (`.github/workflows/ci.yaml:82-89`),
which "only supports TOML 1.0" according to the [tamasfe/taplo#774][pr-774] discussion (2025-04-12).
The diagnostic is correct for the spec version the release targets.

### Fixed on master, not released

Commit `d6e4a8f` ("support trailing commas and newlines for inline tables (toml 1.1.0)",
authored 2026-02-22,
merged through [tamasfe/taplo#849][pr-849] on 2026-03-11,
a port of [tamasfe/taplo#774][pr-774])
replaces the loop with a comma-or-end state machine
and deletes the `inline_table_trailing_comma` invalid test
(`crates/taplo/src/parser/mod.rs:789-830` at master `08f343b`):

```rust
// crates/taplo/src/parser/mod.rs:789-830 (master 08f343b, excerpt)
fn parse_inline_table(&mut self) -> ParserResult<()> {
    // https://github.com/toml-lang/toml/blob/78fcf9dd7eab7acfbaf147c684b649477e7bdd9c/toml.abnf#L238
    self.must_token_or(BRACE_START, r#"expected "{""#)?;

    let mut expect_comma_or_end = false;

    loop {
        // ...
        match t {
            BRACE_END => {
                self.add_token()?;
                break;
            }
            WHITESPACE | NEWLINE | COMMENT => {
                self.add_token()?;
            }
            COMMA => {
                if !expect_comma_or_end {
                    let _ = self.error(r#"unexpected ",""#);
                }
                self.add_token()?;
                expect_comma_or_end = false;
            }
```

Master CI now runs `toml-test` v2.1.0 with `-toml 1.1`
but skips the cases for the remaining gaps,
including `valid/string/escape-esc`,
`valid/string/hex-escape`,
and `valid/datetime/no-seconds` (`.github/workflows/ci.yaml:82-89` at `08f343b`).
No commit after the release tag touches `crates/taplo/src/util/escape.rs`,
and the only one touching `crates/taplo/src/syntax.rs` is `4c8ecf4` ("Allow leading zeros in dates").

crates.io still lists 0.14.0 as `max_stable_version`
(`https://crates.io/api/v1/crates/taplo`,
 fetched 2026-09-17),
and master still declares `version = "0.14.0"` in `crates/taplo/Cargo.toml`.
The project is in minimal maintenance:
the owner announced stepping down in [tamasfe/taplo#715][issue-715] (opened 2024-12-20)
and later wrote "I think I'll just pull the trigger and deprecate every extension and archive the repository"
(2025-11-10),
though the repository is not archived and a collaborator still merges changes.

### Corrected readings

- The task premise said the repository's Cargo manifests already use this form.
  Measured on 2026-09-17,
   none do:
   `git ls-files '*Cargo.toml' | xargs rg --line-number --multiline ',\s*\}'` printed nothing,
   while the same search matched `features` lines in those files,
   so the search ran.
  Across tracked `*.toml` files,
   every `,\s*\}` match is inside a string value
   (TypeScript in `mise.toml` task scripts,
   regex quantifiers such as `{250,}` in `package/forbidden-strings/cli/data/betterleaks-default-config.toml`)
   or in a TOML 1.1 test fixture under `package/test-fixture/toml-edit/src/valid/`.
  The form enters Cargo manifests only if `module-toml-edit` creates an inline table
   (`toml-set.unit.test.ts:484`)
   or someone writes it by hand.
- Calling this a `taplo` bug is wrong for 0.14.0:
   the release predates TOML 1.1.0 and states TOML 1.0.0 as its target.
  The actionable fact is that the fix exists on master but is unreleased.

### Same parser in this repository's dprint TOML plugin

`package/config/dprint/index.json:37` loads `https://plugins.dprint.dev/toml-0.7.0.wasm`.
`dprint/dprint-plugin-toml` at tag `0.7.0` depends on `taplo = { version = "0.12.1", default-features = false }`
(`Cargo.toml:39`),
and `taplo` at `release-taplo-0.12.1` carries the same two diagnostics
(`crates/taplo/src/parser/mod.rs:807`,
 `:818`).
Commit `3c1ec63e9` ("fix(config-dprint):
 exclude unsupported TOML fixtures",
2026-05-15) excludes the TOML 1.1 fixtures from dprint
(`package/config/dprint/index.json:16-30`),
including `toml10-invalid-toml11-valid-invalid28-inline-table-trailing-comma-input.toml`.
`dprint-plugin-toml` 0.8.0 (2026-08-18) lists "feat:
 use a custom TOML parser and support TOML 1.1 (#34)"
in its release notes and no longer depends on `taplo`;
whether 0.8.0 formats those fixtures was not probed here (Unverified).

### Frozen Cargo formatter failure in the October 17 Depot replay

The coherent desktop replay reproduced this compatibility gap with the actual authored manifest,
rather than a synthetic fixture.
At source `3434eaa455d569c18b52cf15100f77f9db251d21`,
`package/music-player/desktop-app/Cargo.toml` has 17,145 bytes
and SHA256 `263bc7bf0b0af05e32e22dac60b9b499249488332de48cf9bd93db8e915f537d`.
The original build,
Clippy,
and nextest tasks passed against that source.
The source-locked dprint `0.61.1` check failed with TOML plugin `0.7.0`:

```text
Line 96, column 105: expected value, trailing comma is not allowed

derer-software", ], }
```

Both recorded `mise --no-config` installation/version commands succeeded.
The actual exact-file format command exited one.
Source identities and selected Cargo bytes match before and after execution.
The native case is a successful control:
its actual changed Cargo manifest passed the same adapter and formatter version.
The [coherent native/Desktop evidence](../audit/depot-coherent-native-desktop-2026-10-17.evidence.json)
retains the complete formatter command records,
source identities,
provider joins,
and successful build/test outcomes.
The repaired missing-consumer issue `#1166` is separate from this executed format failure.

The freshly cloned plugin tag `0.7.0`,
commit `07383747e788015f7dbc224b7371b2bbe2f2cec5`,
confirms the parser dependency in `Cargo.toml:39`:

```toml
# Cargo.toml:39, dprint/dprint-plugin-toml tag 0.7.0
taplo = { version = "0.12.1", default-features = false }
```

Parsing occurs before Cargo conventions in `src/format_text.rs:42`.
`parse_taplo` calls the dependency parser and turns its first error into the format diagnostic
at `src/format_text.rs:52`:

```rust
// src/format_text.rs:42, dprint/dprint-plugin-toml tag 0.7.0
fn parse_and_process_node(file_path: &Path, text: &str, config: &Configuration) -> Result<SyntaxNode> {
  let node = parse_taplo(text)?;
```

```rust
// src/format_text.rs:52, dprint/dprint-plugin-toml tag 0.7.0
fn parse_taplo(text: &str) -> Result<SyntaxNode> {
  let parse_result = taplo::parser::parse(text);

  if let Some(err) = parse_result.errors.first() {
    bail!(
      "{}",
      dprint_core::formatting::utils::string_utils::format_diagnostic(Some((err.range.start().into(), err.range.end().into())), &err.message, text,)
```

This observation extends the September 17 fixture evidence to a frozen authored Cargo manifest.
It does not revise the dated September 17 repository scan.
No source edit or formatter upgrade was applied to the immutable workload.
The known grammar mismatch adds no new upstream defect or additive upstream report;
the existing upstream filing decision remains applicable.
Repository issue `#562` already records this formatter family and requests scoped follow-ups.

## Verification

Versions under test:

- `taplo` 0.14.0,
   crates.io checksum `c221a50eef1a5493074f11ca1ed62bef28c05a4d925002944cc686b2e783a5b3`
   (crates.io API and the harness `Cargo.lock` agree),
   tag `release-taplo-0.14.0`,
   commit `15963fcb555af6bb543e79caf664c3daa110f2ad` (from the crate's `.cargo_vcs_info.json`).
- `taplo` master `08f343be02ce1b20296470396a42f0fa47820449` (2026-07-28),
   which contains `d6e4a8f` (`git merge-base --is-ancestor` succeeded).
- Cargo `1.98.1 (797e8a9bc 2026-08-05)` for the manifest check.

Run on 2026-09-17 inside `docker.io/library/rust:slim` (`rustc 1.98.1`),
with only the harness directory mounted and no credentials.
The same `main.rs` builds against both `taplo` sources.

```toml
# ~/temp/agent/taplo-toml11/taplo-release/Cargo.toml
[package]
name = "taplo-release"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
taplo = { version = "=0.14.0", default-features = false }

[workspace]
```

```toml
# ~/temp/agent/taplo-toml11/taplo-master/Cargo.toml
[package]
name = "taplo-master"
version = "0.0.0"
edition = "2024"
publish = false

[[bin]]
name = "taplo-master"
path = "../taplo-release/src/main.rs"

[dependencies.taplo]
git = "https://github.com/tamasfe/taplo"
rev = "08f343be02ce1b20296470396a42f0fa47820449"
default-features = false

[workspace]
```

```toml
# ~/temp/agent/taplo-toml11/cargo-toml11/Cargo.toml
[package]
name = "cargo-toml11"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
clap = { version = "4", features = [ "derive", ], }

[workspace]
```

```rust
// ~/temp/agent/taplo-toml11/taplo-release/src/main.rs
#![forbid(unsafe_code)]
//! taplo TOML 1.1 acceptance probe:
 parse errors,
 lossless round trip,
 DOM validation,
 and value reads,
//! plus the consumer-side filter that ignores only the inline-table trailing-comma diagnostic.

const TRAILING_COMMA: &str = "expected value, trailing comma is not allowed";

/// (case name,
 TOML version that makes the source valid,
 source)
const CASES: &[(&str, &str, &str)] = &[
    ("cargo-inline-table-trailing-comma", "1.1", "clap = { version = \"3\", features = [ \"derive\", ], }  # c\n"),
    ("incumbent-created-inline-table", "1.1", "[foo]\nx = { y = 1, }\n"),
    ("multi-line-inline-table", "1.1", "t = {\n  a = 1,\n  b = 2,\n}\n"),
    ("string-escape-e", "1.1", "csi = \"\\e[\"\n"),
    ("string-escape-hex", "1.1", "a = \"\\x61\"\n"),
    ("time-without-seconds", "1.1", "t = 14:15\n"),
    ("cargo-inline-table-no-trailing-comma", "1.0", "clap = { version = \"3\", features = [ \"derive\", ] }  # c\n"),
    ("cargo-inline-table-compact", "1.0", "clap = { version = \"3\", features = [\"derive\"] }\n"),
    ("inline-table-double-comma", "invalid", "x = { y = 1,, }\n"),
    ("inline-table-leading-comma", "invalid", "x = { , y = 1 }\n"),
];

fn main() {
    for (name, spec, source) in CASES {
        let parse = taplo::parser::parse(source);
        let errors: Vec<String> = parse
            .errors
            .iter()
            .map(|e| format!("{}..{} {}", u32::from(e.range.start()), u32::from(e.range.end()), e.message))
            .collect();
        let only_trailing_comma = !parse.errors.is_empty() && parse.errors.iter().all(|e| e.message == TRAILING_COMMA);
        let lossless = parse.clone().into_syntax().to_string() == *source;
        let dom = parse.into_dom();
        let dom_valid = dom.validate().is_ok();
        let clap_version = dom.get("clap").get("version").as_str().map(|s| s.value().to_owned());
        let clap_features = dom.get("clap").get("features").as_array().map(|a| a.items().read().len());
        let accepted_by_filter = (errors.is_empty() || only_trailing_comma) && dom_valid && lossless;
        println!("{name} (valid in TOML {spec})");
        println!("  errors={errors:?}");
        println!("  lossless={lossless} dom_valid={dom_valid}");
        println!("  clap.version={clap_version:?} clap.features.len={clap_features:?}");
        println!("  accepted_by_trailing_comma_filter={accepted_by_filter}");
    }
}
```

```bash
# Build and run against taplo 0.14.0 and master, then ask Cargo to read the TOML 1.1 manifest.
podman run --rm --memory=2g --cpus=2 \
  --volume "${HOME}/temp/agent/taplo-toml11:/work:Z" --workdir /work \
  docker.io/library/rust:slim \
  sh -c 'mkdir -p cargo-toml11/src && touch cargo-toml11/src/lib.rs && cargo --version \
    && (cd taplo-release && cargo build --jobs 4 --quiet && target/debug/taplo-release) \
    && echo "--- taplo master 08f343b" \
    && (cd taplo-master && cargo build --jobs 4 --quiet && target/debug/taplo-master) \
    && (cd cargo-toml11 && cargo metadata --no-deps --format-version 1 --offline > /dev/null) \
    && echo "cargo metadata accepted cargo-toml11/Cargo.toml"'
```

The research corpus run agrees:
`taplo` 0.14.0 round-tripped 76 of the 91 valid `package/test-fixture/toml-edit/src/` fixtures,
failing `inline-table-sample01-1.1`,
`string-sample01-1.1-basic-strings`,
the five `toml10-invalid-toml11-valid-*` fixtures,
and several date and exponent fixtures
(`rust-structured-edits.md`,
 "Probe results by candidate";
the date and exponent failures were not re-examined here).

### Accepted cleanly by 0.14.0 and master (TOML 1.0 forms)

- `clap = { version = "3", features = [ "derive", ] }  # c`:
   `errors=[]`,
   `lossless=true dom_valid=true`,
   `clap.version=Some("3") clap.features.len=Some(1)`.
- `clap = { version = "3", features = ["derive"] }`:
   same result.

### Rejected by 0.14.0 with only the trailing-comma diagnostic; accepted by master

- `clap = { version = "3", features = [ "derive", ], }  # c`:
   0.14.0 `errors=["50..51 expected value, trailing comma is not allowed"]`,
   `lossless=true dom_valid=true`,
   `clap.version=Some("3") clap.features.len=Some(1)`;
   master `errors=[]` with the same values.
- `[foo]` / `x = { y = 1, }`:
   0.14.0 `errors=["19..20 expected value, trailing comma is not allowed"]`,
   `lossless=true dom_valid=true`;
   master `errors=[]`.

### Rejected by 0.14.0 with newline diagnostics; accepted by master

- `t = {` / `  a = 1,` / `  b = 2,` / `}`:
   0.14.0 prints `dom_valid=false` with these errors;
   master prints `errors=[]` and `dom_valid=true`.
    - `5..6 newline is not allowed in an inline table`
    - `13..14 expected new line`
    - `22..23 expected new line`
    - `24..25 expected identifier`

### Rejected by 0.14.0 and master (remaining TOML 1.1 gaps)

- `csi = "\e["`:
   both `errors=["7..7 invalid escape sequence"]`,
   `dom_valid=false`.
- `a = "\x61"`:
   both `errors=["5..5 invalid escape sequence"]`,
   `dom_valid=false`.
- `t = 14:15`:
   both `errors=["6..7 unexpected token", "7..9 expected new line"]`.

### Invalid in every TOML version, still rejected (positive control)

- `x = { y = 1,, }`:
   0.14.0 `errors=["12..13 unexpected \",\"", "14..15 expected value, trailing comma is not allowed"]`;
   master `errors=["12..13 unexpected \",\"", "16..16 expected \"}\""]`.
- `x = { , y = 1 }`:
   0.14.0 `errors=["6..7 unexpected \",\""]`;
   master `errors=["6..7 unexpected \",\"", "10..11 expected identifier", "14..15 expected \"=\""]`.

### Cargo accepts the TOML 1.1 manifest

- `cargo metadata --no-deps --format-version 1 --offline` with `cargo 1.98.1` on
   `clap = { version = "4", features = [ "derive", ], }` succeeded
   (`cargo metadata accepted cargo-toml11/Cargo.toml`).
  The research run also accepted `cfg-if = { version = "1", features = [ ], }`
   on nightly 1.100 and 1.98.1 (`rust-structured-edits.md`,
 "Probe results by candidate").

## Verified workarounds

### Parse TOML 1.1 input with `toml_edit` 0.25.15 instead of `taplo`

`toml_edit` 0.25.15+spec-1.1.0 round-trips
`clap = { version = "3", features = [ "derive", ], }` and a multi-line inline table byte for byte
(case `toml-1.1-inline-tables` in [toml-edit-comment-loss.md](toml-edit-comment-loss.md) "Verification"),
and the research harness scored its case `t20` as a pass.

Tradeoffs:

- Different API and tree:
   `toml_edit` is a document model with `Decor` trivia,
   not a `rowan` syntax tree with text ranges,
   so range-splice editors written against `taplo` do not port directly.
- Its editing calls have their own comment-loss behavior,
   documented with workarounds in [toml-edit-comment-loss.md](toml-edit-comment-loss.md).
- The research corpus run found two TOML 1.0 invalid fixtures that `toml_edit` accepts
   (`invalid22-key-newline-value-in-inline-table` and `invalid23-...`,
   `rust-structured-edits.md`,
 "Probe results by candidate").

The same research run found `tombi-parser` at commit `12c3b5b` round-trips all 91 valid fixtures including `t20`,
but it rejected only 78 of 108 invalid fixtures because semantic checks live in other `tombi` crates;
that alternative was not re-run here.

### Accept only the trailing-comma diagnostic from `taplo` 0.14.0

Because `report_error` records the trailing-comma error without consuming a token,
a caller can treat a parse whose only errors carry that exact message as valid,
after confirming the tree round-trips and the DOM validates.
This is the `accepted_by_filter` expression in the harness:

```rust
// Consumer-side filter for taplo 0.14.0 (excerpt of the harness main.rs).
const TRAILING_COMMA: &str = "expected value, trailing comma is not allowed";
let only_trailing_comma = !parse.errors.is_empty() && parse.errors.iter().all(|e| e.message == TRAILING_COMMA);
let lossless = parse.clone().into_syntax().to_string() == *source;
let dom = parse.into_dom();
let dom_valid = dom.validate().is_ok();
let accepted_by_filter = (errors.is_empty() || only_trailing_comma) && dom_valid && lossless;
```

Verified on 0.14.0:
 `cargo-inline-table-trailing-comma` and `incumbent-created-inline-table`
 print `accepted_by_trailing_comma_filter=true`,
 and the former reads `clap.version=Some("3")`;
 `inline-table-double-comma` still prints `false` because it also carries `unexpected ","`.

Tradeoffs:

- Covers single-line inline tables only;
   multi-line inline tables and the other TOML 1.1 syntax stay rejected.
- Matches on an English message string,
   which a future `taplo` release may reword or remove;
   pin the exact version while relying on it.
- The filter trusts that `report_error` leaves the tree intact,
   true at `crates/taplo/src/parser/mod.rs:160-171` for 0.14.0 only.
- `taplo`'s formatter and `Rewrite` helpers were not exercised on such input.

### Depend on `taplo` master by git revision

The `taplo-master` harness manifest pins `[dependencies.taplo]` to
`git = "https://github.com/tamasfe/taplo"` and `rev = "08f343be02ce1b20296470396a42f0fa47820449"`,
and parses every inline-table case cleanly,
including multi-line inline tables.

Tradeoffs:

- A crate with a git dependency cannot be published to crates.io
   ("[crates.io][] does not allow packages to be published
with dependencies on code published outside of [crates.io][]",
   `rust-lang/cargo` `doc/book/src/reference/specifying-dependencies.md:205-207`).
- `\e`,
   `\xHH`,
   and times without seconds are still rejected on master.
- The revision is unreleased,
   in a project whose owner has stepped down,
   so fixes arrive only through further git pins.

### Emit TOML 1.0-valid inline tables

Writing `clap = { version = "4", features = [ "derive", ] }`
(trailing comma inside the array only)
or `clap = { version = "4", features = ["derive"] }`
parses cleanly on 0.14.0 (cases `cargo-inline-table-no-trailing-comma` and `cargo-inline-table-compact`).

Tradeoffs:

- Diverges from the `module-toml-edit` creation style `{ y = 1, }`
   (research open question D13,
   `rust-structured-edits.md`,
 "Definition").
- Only controls files we write;
   third-party or hand-written TOML 1.1 input is still rejected.
- Rules out multi-line inline tables,
   which TOML 1.1 allows.

## What does not work

- **Upgrading `taplo` from crates.io.**
  No newer release exists:
   crates.io `max_stable_version` is 0.14.0 as of 2026-09-17,
   and master still declares version 0.14.0.
- **Widening the filter to the newline diagnostic.**
  `newline is not allowed in an inline table` goes through `error`,
   which re-emits the token as `ERROR`;
   the harness shows follow-on `expected new line` and `expected identifier` errors
   and `dom_valid=false` for `multi-line-inline-table` on 0.14.0.
- **Using `taplo` master for all TOML 1.1 syntax.**
  `string-escape-e`,
   `string-escape-hex`,
   and `time-without-seconds` fail identically on master and 0.14.0.

## Upstream filing artifact (nothing to file)

### Upstream filing decision

`.out-of-scope/` was checked first:
 `rg --ignore-case 'toml|taplo' .out-of-scope/` matched only `bun-install.md` and `cargo-workspace.md`,
 which mention `mise.toml` and `Cargo.toml` files,
 not `taplo`,
 so no exemption applies.

Candidate-fix applicability gate:
 the known fix `d6e4a8f` is absent from the installed 0.14.0 source
 (the rejecting branch is still at `crates/taplo/src/parser/mod.rs:802-808`),
 and its issue and test inputs match the observed failure
 ([tamasfe/taplo#333][issue-333]:
 "Line breaks and trailing comma in inline tables";
 added test `inline_table_with_linebreaks_and_trailing_comma`).
The harness shows master accepting the exact failing inputs,
so the incident is "fixed upstream,
 unreleased",
not a new defect.

Walking the six constraints:

1.  **Is it really upstream's fault?**
    No for the release in use.
    `taplo` 0.14.0 predates TOML 1.1.0,
    states TOML v1.0.0 as its target,
    and emits the correct TOML 1.0 diagnostic.
    The only upstream-side gap is that the merged fix has no release.
2.  **Can upstream fix it?**
    Yes,
    and it already did on master (#849).
3.  **Are they supporting this use case?**
    Mixed.
    [tamasfe/taplo#332][issue-332] (open,
    owner-authored tracking issue "TOML 1.1.0") lists #333 as done,
    and master CI runs `toml-test -toml 1.1`.
    But a collaborator answered [tamasfe/taplo#839][issue-839] ("TOML 1.1 is released",
    2026-01-15) with "No,
 please see #715",
    and on #332 (2026-04-29) wrote that the VS Code extension will not get TOML 1.1 support
    "as I do not have any way to publish updates for it".
4.  **Would the repo welcome our contribution?**
    Yes;
    no ban was found,
    and absence of a policy is not a fail.
    `CONTRIBUTING.md` at master asks contributors to "first discuss the change you wish to make via an issue";
    there is no AI policy file,
    no issue template,
    and no pull request template
    (`git ls-tree -r --name-only origin/HEAD` lists only `.github/FUNDING.yml` and workflow files under `.github/`).
5.  **Will they likely fix it?**
    The code fix is done;
    a crates.io release is what is missing,
    and the signals lean no:
    no crates.io release since 2025-05-22 although master gained commits dated 2026-01 through 2026-07,
    the owner announced deprecation and archiving (#715),
    and the collaborator lacks extension publishing access (#332,
 #849).
    No statement about crates.io publishing access was found.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    Not applicable.
    The minimal fix exists upstream as `d6e4a8f`,
    and the harness verified it against the failing inputs;
    there is no patch of ours to prototype.
    The remaining TOML 1.1 gaps (`\e`,
 `\xHH`,
 optional seconds) are outside this doc's cluster,
    already tracked by #332 and skipped deliberately in master CI.

Decision:
 file nothing,
 comment nothing.

### Duplicate search

Searched on 2026-09-17 with `gh search issues --repo tamasfe/taplo -- <term>`
and `gh search prs --repo tamasfe/taplo -- <term>`
(no state filter,
so open and closed both returned),
plus every issue and pull request created since 2026-01-01
(`gh search issues --repo tamasfe/taplo --created '>=2026-01-01'`,
 same for `prs`).
Issue terms:
 `toml 1.1`,
 `1.1.0`,
 `trailing comma`,
 `inline table newline`,
 `trailing comma is not allowed`,
 `newline is not allowed in an inline table`,
 `inline table`,
 `release`,
 `new release`,
 `publish crates.io`,
 `crates.io release`.
Pull request terms:
 `toml 1.1`,
 `1.1.0`,
 `trailing comma`,
 `inline table`,
 `release`,
 `spec 1.1`.
Threads read in full:

- [tamasfe/taplo#333][issue-333] (closed 2026-03-11 by #849,
   owner-authored 2022-10-08):
   exact duplicate of the parser behavior.
- [tamasfe/taplo#849][pr-849] (merged 2026-03-11) and [tamasfe/taplo#774][pr-774] (closed,
   superseded):
   the fix.
- [tamasfe/taplo#332][issue-332] (open):
   TOML 1.1.0 tracking issue with release-status comments.
- [tamasfe/taplo#839][issue-839] (closed 2026-01-15):
   TOML 1.1 support request,
   answered "No,
 please see #715".
- [tamasfe/taplo#715][issue-715] (closed 2025-11-10):
   project future and owner stepping down.
- [tamasfe/taplo#870][issue-870] (open):
   Even Better TOML errors on files Cargo accepts;
   commenters tie it to the Cargo schema ([tamasfe/taplo#863][issue-863]),
   not TOML 1.1 syntax,
   so not a duplicate.

Nothing to add:
 #333 is closed as fixed,
 #332 already carries user requests for a TOML 1.1 release and the collaborator's publishing constraint,
 and a request to publish a crates.io release would be a "+1" on that thread.
Post nothing.
No draft is kept,
 because no finding here is absent from those threads.

[issue-332]: https://github.com/tamasfe/taplo/issues/332
[issue-333]: https://github.com/tamasfe/taplo/issues/333
[issue-715]: https://github.com/tamasfe/taplo/issues/715
[issue-839]: https://github.com/tamasfe/taplo/issues/839
[issue-863]: https://github.com/tamasfe/taplo/issues/863
[issue-870]: https://github.com/tamasfe/taplo/issues/870
[pr-774]: https://github.com/tamasfe/taplo/pull/774
[pr-849]: https://github.com/tamasfe/taplo/pull/849
[crates.io]: https://crates.io
