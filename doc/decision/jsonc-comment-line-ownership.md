# JSONC comment ownership ends a line at LF or CRLF only

## Context

The maintained TypeScript package and the ported Rust crate
`monochromatic-jsonc-edit` attach every comment to exactly one owner:
a key,
a value,
or the document root.
Ownership is decided by lines:
a comment before a separator belongs to the key or value it precedes,
a comment after a separator on the same line belongs to the value before that separator,
and a comment after a separator on a later line belongs to the next key.

While verifying the packaged crate from a disposable consumer on 2026-09-25,
both implementations were probed on six separator and comment shapes and agreed exactly,
including one asymmetry:
a bare CR ends a line comment body but does not end a line for ownership.
Measured on identical input:

- `{"a":1,\n// x\n"b":2}` owns the comment on key `b`.
- `{"a":1,\r\n// x\r\n"b":2}` owns the comment on key `b`.
- `{"a":1,\r// x\r"b":2}` owns the comment on value `a`.
- `{"a":1, // x\n"b":2}` owns the comment on value `a`.

The mechanism is that both scanners decide "same line" from `\n` alone
(`package/jsonc-edit/core-ts/src/scan.ts:11` and
`package/jsonc-edit/core-rust/src/scan.rs:216`),
while the comment body scanner ends at CR,
LF or CRLF.
A document written entirely with classic-Mac CR line endings therefore attaches every
comment to the preceding value.

## Options

- Keep `\n`-only line boundaries for ownership.
   No code change,
   both implementations already agree,
   and LF-only line counting matches neighboring JSONC tooling such as microsoft
   `jsonc-parser`.
   Cost:
   internally asymmetric,
   since CR ends a comment but not a line,
   and whole-CR documents own comments differently from their LF and CRLF equivalents.
- Treat a bare CR as a line break for ownership in both implementations.
   Consistent with CR already being a comment terminator,
   and CR-only documents then behave like LF and CRLF ones.
   Cost:
   a design change to the released TypeScript package plus a matching Rust change,
   re-verification of the shared whitespace and comma-lookahead logic that the fuzz and
   conformance suites exercise,
   for an input shape no requirement names.

## Decision

The user chose to keep `\n`-only line boundaries for ownership on 2026-09-25.
A bare CR terminates a comment body and does not start a new line for ownership purposes.

## Consequences

- The rule is now part of the shared contract:
   `contract.lineOwnership` in `fixtures/jsonc-conformance.json` states it,
   and four `commentOwnership` cases lock it in for LF,
   CRLF,
   a bare CR after a separator,
   and a whole-CR document.
   Both maintained implementations read that file,
   and a `test:shared-fixtures` task in each package fails when the copies drift.
- The ownership cases were shown to be live evidence:
   mutating the bare-CR expectation in the Rust copy alone made
   `fixture_tests::comment_ownership_matches_the_fixture` fail,
   and restoring the file made it pass again.
- Both READMEs state the rule,
   `package/jsonc-edit/core-rust/README.md` under "Model" and
   `package/jsonc-edit/core-ts/README.md` in its comment-model paragraph.
- If bare-CR documents later become inputs that matter,
   this decision is the place to reverse:
   the change would touch the same-line notion in both scanners at once,
   and the four fixture cases would flip with it.
