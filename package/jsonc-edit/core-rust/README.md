# monochromatic-jsonc-edit

Comment-preserving JSONC parse,
 immutable edit and canonical emit for Rust.

This crate is the native sibling of the TypeScript package
`@monochromatic-dev/jsonc-edit-core-ts`.
 Both implement the same supported behavior:
 a JSONC document is an object or array root,
 `//` and `/* */` comments are data rather than discarded trivia,
 trailing commas are accepted,
 JSON5-only forms are rejected,
 and output is canonical rather than byte-preserving.

## Model

- Every object key and every value carries at most one attached,
   normalized comment.
   Stacked comments merge in source order into a single `mixed` comment.
- Edits are immutable:
   each operation returns a new state and leaves the previous state usable.
- Numbers compare by exact mathematical value,
   so `1`,
   `1.0` and `1e0` are equal,
   while an unedited literal keeps the spelling it was written with.
- Escaped unpaired UTF-16 surrogates survive as code units,
   with a fallible conversion to Rust `String` for values that UTF-8 cannot express.
- Container nesting is bounded:
   512 open containers are accepted and a 513th is rejected before deeper state is built.
- Line comments terminate at CR,
   LF or CRLF.
- A comment belongs to the key or value it precedes,
   or to the value before a separator when it follows that separator on the same line.
   For ownership a line ends at LF or CRLF only:
   a bare CR terminates a comment body but does not start a new line,
   so a comment reached after a bare CR stays with the preceding value.
   `doc/decision/jsonc-comment-line-ownership.md` records that choice.

## Source spans

`parse_jsonc_with_spans` returns the same tree as `parse_jsonc` plus the position of every value and every object key:
byte offset,
line and column of the start and of the end (exclusive).
Spans travel beside the tree,
never inside it,
so the tree structs,
their equality and every caller that builds them by hand are unchanged.

- `JsoncSpans::values` lists the spans of all values in document (pre-)order,
   the root first;
   `JsoncSpans::keys` lists the object member keys in source order.
   A caller that walks the tree depth-first with one counter per list reads each node's span with no lookup.
- Lines and columns count from 1.
   The column counts Unicode scalar values,
   only LF ends a line,
   and a bare CR is one column.
- Spans are recorded in the same single forward pass as the tree:
   position counters only move forward,
   so the cost stays linear in the text and the 512-container bound is unchanged.
   `parse_jsonc` switches the recorder off and pays nothing.
- `jsonc_position_at(source, offset)` gives the same line and column for a bare byte offset,
   such as a parse error's `offset`.

## Dependencies

None.
 The standard library is the only dependency,
 and the crate contains no `unsafe` block.
 `doc/decision/jsonc-edit-rust-foundation.md` records why the parser,
 emitter and exact-number identity are owned here rather than delegated to a published crate,
 and
 `doc/audit/tech-jsonc-edit-rust-parser-foundation-vet-2026-09-24-63342231.md` plus
 `doc/audit/tech-jsonc-edit-rust-exact-number-foundation-vet-2026-09-24-e8e0034a.md`
 hold the selection evidence.

## Status

Version 0.1.1 is being built out in this package.
 It differs from 0.1.0 in two fixes found by fuzzing:
 a comment body carrying a bare CR is emitted in block form instead of as a `//` line it would end early,
 and replacing a scalar document root is refused.
 The public API and its usage examples land with the ported modules;
 this README grows as each surface is added rather than documenting items that do not exist yet.

## License

LGPL-3.0-or-later.
 The license texts are in `LICENSES/`.
