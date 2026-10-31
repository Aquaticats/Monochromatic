# xml-edit.fuzz

Property fuzzing sidecar for `monochromatic-xml-edit`.
It sits beside the crate it fuzzes,
like the other `<package>.fuzz` sidecars in this repository.

## What it fuzzes

Three libFuzzer targets,
each attacking one published promise of the crate:

- `fuzz_parse_roundtrip`:
   every generated document must parse,
   keep its source byte-for-byte,
   re-parse to an identical document,
   and each element's own source must parse as a document.
- `fuzz_reject_and_recover`:
   one deliberate mutation away from a valid document must either parse and satisfy every property above,
   or be refused with a non-empty message and a one-based position inside the source.
   No panic,
   no hang,
   no overflow.
- `fuzz_edit_invariants`:
   a random element plus a random edit (set an existing attribute,
   set a new attribute,
   append a child,
   remove an element).
   Bytes outside the targeted element must be unchanged,
   the new value must read back,
   the comment count must not change,
   and the result must re-parse.
   A refusal is allowed only for a value XML 1.0 cannot carry,
   a text-bearing parent,
   or the root.

## Why a structured generator

Random bytes almost never form well-formed XML,
so a byte-only campaign would spend its budget failing at the first character.
`src/generators.rs` implements `arbitrary::Arbitrary` for a whole document:
an optional byte order mark,
declaration and comments around a root,
mixed line terminators (LF,
CRLF and bare CR),
tabs or spaces or no indentation,
both quote styles,
spaced and unspaced `=`,
references,
comments that hold markup,
CDATA sections,
processing instructions and prefixed or non-ASCII names.
`mutated` then applies exactly one delete,
insert,
truncate or replace using tokens such as `<!DOCTYPE`,
`]]>`,
`&#` and a control character.

## Tasks

Run from this directory:

- `mise run test` runs the generator and invariant unit tests through `cargo nextest run --lib --release`.
   Those tests include a validity sweep (every generated document must parse)
   and negative controls:
   each invariant is shown to panic on a deliberately broken input,
   so a quiet campaign is evidence rather than silence.
- `mise run lint:clippy` runs Clippy over every target with warnings denied.
- `mise run list` lists the fuzz targets.
- `mise run build` builds every target with AddressSanitizer.
- `mise run smoke` runs every target for 30 seconds with the dictionary.
- `mise run run <target> [-- libFuzzer flags]` runs or replays one target.

Campaigns belong in a resource-bounded container rather than on a workstation,
because libFuzzer otherwise runs until told to stop.
The house rule for that isolation is in `AGENTS.md` under hazardous commands.

## Layout

- `src/lib.rs` re-exports the generator and invariants.
- `src/generators.rs` builds documents and single mutations.
- `src/invariants.rs` holds the property checks shared by every target.
- `fuzz_targets/` holds the three harnesses.
   Each needs an explicit `[[bin]]` entry in `Cargo.toml`,
   because `cargo-fuzz` discovers targets through `cargo metadata`.
- `dictionary/xml-edit.dict` gives libFuzzer the XML tokens worth splicing.
- `seed/` holds committed starting inputs;
   `corpus/` and `artifacts/` are gitignored scratch that libFuzzer owns.
   Tasks copy seeds into `corpus/<target>` and pass only that one directory,
   so libFuzzer cannot write discoveries into the tracked seeds.
