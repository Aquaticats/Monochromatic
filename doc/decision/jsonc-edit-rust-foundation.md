# Own the Rust JSONC parser and exact-number representation

## Status

Accepted,
 2026-09-24.
 Foundation adoption was authorized by the user in this session after both vet reports recorded
 saturated discovery,
 equal-depth validation,
 frozen-rubric scores,
 sensitivity reruns and full rankings.

## Context

`doc/planning/jsonc-edit-rust-port.md` defines the port:
 a native Rust crate named `monochromatic-jsonc-edit` that keeps the maintained TypeScript package's identity
 (JSONC container roots,
 canonical output,
 immutable edits,
 separately queryable comments on keys and values)
 while adding exact mathematical number comparison,
 retained unedited spelling,
 escaped unpaired UTF-16 surrogate support and mandatory memory safety.

Two selection runs under `.agents/skills/choosing-technology/SKILL.md` produced the evidence:

- [`tech-jsonc-edit-rust-parser-foundation-vet-2026-09-24-63342231.md`](../audit/tech-jsonc-edit-rust-parser-foundation-vet-2026-09-24-63342231.md)
   scored the repository-owned translation 23 of 25
   and the `biome_json_parser` 0.5.7 projection composition 10 of 25,
   with the owned candidate rated at least as high on every frozen criterion.
- [`tech-jsonc-edit-rust-exact-number-foundation-vet-2026-09-24-e8e0034a.md`](../audit/tech-jsonc-edit-rust-exact-number-foundation-vet-2026-09-24-e8e0034a.md)
   saturated with one survivor after 13303 registry records were screened
   and every published alternative exited on a source-verified hard gate.

Both survivors were validated at equal depth:
 the shared TypeScript conformance corpus,
 independent UTF-16 and comment-owner oracles,
 an external consumer lifecycle at depth 512,
 bounded malformed and over-depth inputs,
 and Windows GNU plus macOS ARM64 cross-target checks.

## Decision

The native crate implements its own JSONC foundation:

- Its own byte scanner,
   iterative container-frame parser,
   canonical emitter and comment-attachment model.
- Its own raw-token plus mathematical-identity number representation,
   keeping the source spelling separately from the compared value.
- No third-party parser,
   syntax-tree,
   decimal or bigint dependency;
   the standard library only.
- No `unsafe` block anywhere in the crate.

These behavior gates are mandatory in the product crate,
 not scratch-prototype conveniences:

- Accept 512 nested containers and reject the 513th **before** constructing deeper state.
- Retain escaped unpaired UTF-16 surrogates as code units,
   with a fallible conversion to Rust `String`.
- Keep one normalized comment queryable on every object key and every value,
   including multi-line value comments after emission and reparse.
- Compare numbers by exact mathematical value while retaining unedited spelling.
- Accept only JSONC container roots:
   comments and trailing commas,
   never JSON5-only forms.
- Terminate line comments at CR,
   LF or CRLF.

The maintained TypeScript package stays supported.
 Shared language-neutral fixtures and the confirmed defect fixes land in both implementations:
 the comma-after-trivia rejection,
 multi-line value-comment ownership,
 CR-only line comments,
 clean-input number spelling loss,
 and the clean `JSON.parse` fast-path depth bypass.

## Consequences

- This repository owns the parser,
   emitter,
   normalizer and their tests permanently.
   No upstream fixes,
   fuzzing corpus or external review arrive for free.
- There is no dependency graph to re-audit on version bumps,
   and the license notice burden is this crate's own LGPL-3.0-or-later text.
- Behavior changes are single-place edits,
   and TypeScript parity is enforced through shared fixtures rather than memory.
- Measured parse-to-editor-value cost was 3.5 to 7 times lower than the Biome projection across six workload
   shapes in scratch prototypes;
   product-level numbers must be re-measured before any performance claim is published.
- The Biome composition stays documented as the only other validated path,
   together with the obligations it would impose:
   a diagnostics gate before projection,
   a preparse depth preflight,
   and a source-length guard.

## Rejected alternatives

- `biome_json_parser` 0.5.7 with a projection adapter:
   87 selected registry packages,
   an unsafe `biome_rowan` pointer surface with release-disabled lexer assertions,
   a measured stack overflow while dropping an 8192-deep tree,
   a published helper that panics on an unterminated string
   ([`biome-json-syntax-unterminated-string-range.md`](../troubleshooting/biome-json-syntax-unterminated-string-range.md)),
   and a published crate line that predates upstream's own lexer fix.
- Published JSONC parsers and editors,
   each excluded as-is on a cited hard gate:
   `jsonc-parser` 0.33.2 and `jwc` (unpaired surrogate rejection),
   `nojson` (depth 128 plus surrogate rejection),
   `jsontape` (lone-surrogate grammar),
   `momoa` 3.2.6 (unchecked `char::from_u32_unchecked` on surrogates),
   `edikt-jsonc` and `json5format` (regex-defined production lexing),
   `json-five` 0.3.1 (block-comment span corruption
   in [`json-five-unterminated-block-comment.md`](../troubleshooting/json-five-unterminated-block-comment.md)),
   `fjson` (depth 128 and non-JSON whitespace),
   `serde_jsonc`,
   `serde_jsonc2` and `serde_json_lenient` (comments consumed as whitespace,
   trailing-comma rejection,
   and a CR-only line-comment swallow in
   [`serde-json-lenient-cr-line-comment.md`](../troubleshooting/serde-json-lenient-cr-line-comment.md)),
   plus `babbel_json`,
   `jqf-codec-json`,
   `subc-jsonc`,
   `fig`,
   `fracturedjson`,
   `dprint-plugin-jsonc`,
   `prim-fmt`,
   `jcfmt`,
   `zetch`,
   `patchloom`,
   `qubit-json`,
   `jsonrepair-rs` and `zoko-parser` on category,
   comment-model,
   regex or grammar gates.
- Published exact-number representations:
   `json-number` 0.4.10 (documented unsized-layout gap in
   [`json-number-unsized-layout.md`](../troubleshooting/json-number-unsized-layout.md)),
   `serde_json` `arbitrary_precision`,
   `jstrict` and `reliakit-json` (lexical equality),
   `bigdecimal`,
   `scientific`,
   `b10`,
   `ordecimal`,
   `decimal-bytes`,
   `fpdec`,
   `rust_decimal`,
   `hypercast`,
   `postgres-jsonb-canonical` and `puremp` (bounded exponent or scale),
   `dashu-ratio`,
   `bigfixed` and `fraction` (magnitude materialization on admitted input),
   `bignumber` and `number-general` (binary floating-point projection),
   and `qubit-json` (integer and float range rejection).
- Git-only leads without a published artifact:
   `richplastow/jsonc-lexer`,
   `hayas1/json-with-comments`
   (source-proven unchecked surrogate conversion in
   [`json-with-comments-surrogate-escape.md`](../troubleshooting/json-with-comments-surrogate-escape.md))
   and `jessevanassen/json5_parser`.
   Adopting any of them would mean vendoring or forking,
   which costs more than owning the parser and adds an unaudited surface.
