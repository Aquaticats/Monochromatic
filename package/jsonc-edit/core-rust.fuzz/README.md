# jsonc-edit.fuzz

Property fuzzing sidecar for `monochromatic-jsonc-edit`,
the Rust port of `package/jsonc-edit/core-ts`.
It sits beside the crate it fuzzes,
like the other `<package>.fuzz` sidecars in this repository.

## What it fuzzes

Four libFuzzer targets,
each attacking one published promise of the crate:

- `fuzz_parse_emit_roundtrip`:
   every generated document must parse,
   emit canonically,
   reparse to the same tree,
   and emit identically again.
   It also asserts that no comment body is lost by emission,
   and that the parsed tree respects the 512-container bound.
- `fuzz_reject_and_recover`:
   one deliberate byte mutation away from a valid document must either parse and satisfy every
   property above,
   or be refused with a non-empty message.
   No panic,
   no hang,
   no overflow.
- `fuzz_depth_envelope`:
   exact nesting depths from 1 to 900,
   well-formed and unterminated.
   At or below 512 the document must parse and measure at the depth it was built with;
   above it,
   the refusal must name the nesting limit;
   malformed input must be refused whatever its depth.
- `fuzz_edit_invariants`:
   a random resolved address plus a random action (set,
   delete,
   attach a value comment,
   attach a key comment,
   clear a comment).
   The source state must be unchanged afterwards,
   the edited document must emit and reparse stably,
   comments must survive,
   a set must be visible at its address,
   and a delete must remove it.

The roundtrip,
reject-and-recover and depth-envelope targets also call the span checks in `src/span_invariants.rs`:
every accepted document's `parse_jsonc_with_spans` result must equal the plain parse,
every span must slice back to its own token,
positions must agree with `jsonc_position_at`,
and a rejected document must be rejected identically by both entry points.

## Why a structured generator

Random bytes almost never form valid JSONC,
so a byte-only campaign would spend its budget failing at the first character.
`src/generators.rs` implements `arbitrary::Arbitrary` for a whole document:
container roots,
mixed line terminators (LF,
CRLF and bare CR),
line and block comments in leading,
trailing and stacked positions,
trailing commas,
and the difficult literals the contract cares about
(`1e0`,
`1.500`,
`-0`,
`9007199254740993`,
`1e999999999999999999999999`,
`\u0041`,
`\uD800`,
`\uDC00\uD800`).
`src/path_pick.rs` draws addresses that resolve,
so the edit campaign tests edits rather than not-found errors.

## Tasks

Run from this directory:

- `mise run test` runs the generator,
   invariant and address-picker unit tests through `cargo nextest run --lib --release`.
   Those tests include negative controls:
   each invariant is shown to panic on a deliberately broken input,
   so a quiet campaign is evidence rather than silence.
- `mise run lint:clippy` runs Clippy over every target with warnings denied.
- `mise run //:lint:monochromatic-lint -- package/jsonc-edit/core-rust.fuzz`
  runs the repository linter over this package,
  where `monochromatic-lint.config.jsonc` exempts fuzz sidecars
  from `rust/max-lines` and `rust/require-rustdoc`.
- `mise run list` lists the fuzz targets.
- `mise run build` builds every target with AddressSanitizer.
- `mise run smoke` runs every target for 30 seconds with the dictionary.
- `mise run run <target> [-- libFuzzer flags]` runs or replays one target;
   pass a crash artifact after `--` to minimize or replay it.

Campaigns belong in a resource-bounded container rather than on a workstation,
because libFuzzer otherwise runs until told to stop.
The house rule for that isolation is in `AGENTS.md` under hazardous commands.

## Layout

- `src/lib.rs` re-exports the generator,
   invariants and address picker.
- `src/generators.rs` builds documents,
   exact-depth documents,
   single mutations and replacement values.
- `src/invariants.rs` holds the property checks shared by every target.
- `src/span_invariants.rs` holds the span checks,
  with negative controls in `src/span_invariants_tests.rs`.
- `src/path_pick.rs` draws resolving addresses.
- `fuzz_targets/` holds the four harnesses.
   Each needs an explicit `[[bin]]` entry in `Cargo.toml`:
   `cargo-fuzz` discovers targets through `cargo metadata`,
   so a bare `fuzz_targets/` directory is invisible to it and `cargo fuzz list` then prints nothing
   while still exiting 0.
- `dictionary/jsonc-edit.dict` gives libFuzzer the JSONC tokens worth splicing.
- `seed/` holds committed starting inputs;
   `corpus/` and `artifacts/` are gitignored scratch that libFuzzer owns.
   Tasks copy seeds into `corpus/<target>` and pass only that one directory,
   because passing `seed/` as a second corpus directory let libFuzzer write newly discovered
   units into it,
   polluting tracked files with binary blobs.

## Relationship to the other suites

The shared contract lives in
`package/jsonc-edit/core-ts.conformance/fixtures/jsonc-conformance.json`,
which both the TypeScript conformance suite and the crate's own `fixture_tests.rs` read.
Fuzzing does not duplicate those cases;
it searches for inputs nobody wrote down,
and its invariants are the same properties the fixtures state:
canonical emission,
comment preservation and ownership,
number spelling,
the nesting bound,
and refusal instead of panic.
