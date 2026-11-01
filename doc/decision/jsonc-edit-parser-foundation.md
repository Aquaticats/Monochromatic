# Keep the hand-written JSONC parser for module-jsonc-edit

## Status

Accepted,
2026-06-30.

## Context

`package/jsonc-edit/core-ts` is the comment-preserving JSONC read,
 edit,
 and write
package,
 split out of the paused `package-paused/module/es` taxonomy and shaped after
its sibling `package/toml-edit/core`.
The goal is full edit parity with `toml-edit`:
 a free-function edit API over an immutable
state,
 with a serializer.

The open question was the parser foundation:
keep the hand-written JSONC parser that already lives in `module-es`,
 or adopt an
off-the-shelf parser the way `toml-edit` wraps `toml-eslint-parser`.
A `/choosing-technology` survey cloned,
 source-read,
 built,
 and functionally tested every
serious candidate.

The hand-written parser exists for reasons no surveyed library satisfies:

- A normalized,
   queryable comment model:
  every object key and every value carries at most one attached comment,
   and stacked
  `//` or `//region` lines merge into a single `mixed` comment.
- A hybrid fast-path that runs native `JSON.parse` over clean (comment-free) regions and
  only custom-parses where JSONC features appear.
  **Superseded:**
  the fast-path was removed on 2026-09-24;
  see the amendment section below.
- Zero runtime dependencies and house-style ownership of a stable syntax module (no regex
  where an index scan expresses the rule,
   immutable data,
   `max-lines`,
   rustdoc-equivalent
  TSDoc),
   per `doc/decision/stable-syntax-modules.md`.
- It must run in every modern environment,
   so a WebAssembly core is disqualified.
- Branded input types (`StringJsonc`) are non-optional.

## Decision

Keep the hand-written parser (as a clean rewrite,
 since the original is in disrepair) and
build the serializer and free-function edit API on top of it.
The write model is canonical rebuild from the structured tree with comments as first-class
data,
 not byte-identical splice;
this matches the parser's position-discarding design and exceeds `toml-edit` by making
comments queryable and editable rather than merely preserved.

## Consequences

- No source ranges are threaded through the parse,
   and no splice mode is offered.
  Unedited whitespace is normalized by canonical emit;
  raw scalar tokens and all comments are preserved.
- Every document takes the structured parse path,
  so clean and commented input share one
  number-spelling,
  comment-ownership and nesting-limit behavior.
  The `PlainJson` variant stays in the value model as a leaf a caller may build from native
  JSON,
  and the edit API normalizes it before editing.
- The branded `StringJsonc` type is redefined locally inside `jsonc-edit`,
   decoupled from
  the abandoned `module-es` taxonomy.
- Performance is a stated goal,
   so the package ships benchmarks against
  `jsonc-eslint-parser` and `jsonc-parser` to keep the structured parser honest.

## Rejected alternatives

Each library fails at least one reason-for-existence above.
Versions and behaviors below were confirmed by cloning and running each candidate,
 not from
documentation alone.

- `jsonc-eslint-parser` (ota-meshi,
   the direct `toml-eslint-parser` sibling):
  positioned AST plus a flat `ast.comments` array,
   but `traverseNodes` skips comment keys,
  so it never attaches a comment to its node;
  parse-only (no serializer);
  throws on the first syntax error (no recovery);
  pulls in `acorn` and statically imports `node:module` and `node:path`,
   so a browser bundle
  needs shims.
  Fails the comment model,
   the fast-path,
   zero-dep,
   and run-everywhere.
- `jsonc-parser` (microsoft):
  zero-dep,
   browser-safe,
   fault-tolerant,
   and a strong offset-splice editor
  (`modify` plus `applyEdits`),
   but comments are trivia with offsets,
   never attached to
  nodes;
  insert and delete drop neighboring comments (upstream issues 125 and 108);
  there is no value-tree serializer (`getNodeValue` discards comments).
  Cannot model comments as data.
- `comment-json` (kaelzhang):
  the closest match,
   with queryable per-key and per-value comments via Symbol positions and
  a stringifier,
   but stacked comments stay unmerged token arrays,
   duplicate keys silently
  collapse (data loss),
   and it is CJS-only with an `esprima` dependency.
- `@humanwhocodes/momoa`:
  well-maintained AST and tokenizer,
   but comments are flat tokens and `print()` discards
  comments and whitespace.
  A parser and pretty-printer,
   not a comment-preserving editor.
- `jju` (rlidwka):
  a working token-splice editor that preserves comments incidentally,
   but no queryable
  comment model,
   JSON5-scoped,
   CJS-only,
   and unreleased since 2018.
- `json5` and `@std/jsonc` (Deno std):
  both drop comments on parse.
- `jsonc-morph` (dsherret,
   Rust compiled to WebAssembly):
  the best byte-faithful round-trip in the field,
   with comments as CST nodes,
   but the
  comments are positional rather than attached or merged,
   it is pre-1.0,
   and its inlined
  WebAssembly cannot instantiate synchronously on a browser main thread.
  Fails run-everywhere.

## Related

- `doc/handover/jsonc-edit.md`:
  living implementation status.
- `doc/decision/stable-syntax-modules.md`:
  owning long-lived syntax modules in-repo.
- `doc/troubleshooting/toml.md`:
  why JSONC was chosen over TOML for workspace configuration.
- `doc/troubleshooting/c-like-comments.md`:
  the accepted block-comment-nesting limitation.

## Amendment, 2026-09-24: the native `JSON.parse` fast-path is removed

The Rust port work in `doc/planning/jsonc-edit-rust-port.md` confirmed two defects that the
fast-path caused,
 and the user's port requirements (Q6,
 Q8) make both unsupported behavior:

- An unedited number lost its source spelling on clean input,
  because `JSON.parse` keeps only
  the numeric value:
  `{"n":1e0}` emitted as `1`.
  Q6 requires an unedited literal to keep the spelling it was written with.
- A clean document bypassed the 512-container nesting limit,
  because the limit lives in the
  structured parser:
  a measured probe accepted clean nesting at depth 513 while the same depth with a trailing
  comment was rejected.
  Q8 requires the supported behavior to be shared by both implementations.

A direct measurement of the maintained bundle recorded both,
 and the fix removes the shortcut
rather than special-casing it:
 any spelling check cheap enough to keep the shortcut would cost as much as the structural
parse it avoids.

Measured cost,
 from `mise run //package/jsonc-edit/core-ts.bench:bench` (300 entries,
 3000
iterations,
 one run before and one after on the same machine):

- Clean input:
  11055 ops/s before,
  2794 ops/s after;
  microsoft `jsonc-parser` measured 2733
  ops/s in the same after-run and `jsonc-eslint-parser` 479 ops/s.
- Commented input:
  3893 ops/s before,
  3591 ops/s after,
  against 4193 and 876 ops/s for the same
  two comparisons.

The clean-input regression is accepted:
 correctness of unedited spelling and of the nesting
limit outranks the shortcut,
 and the structured parser stays level with microsoft
`jsonc-parser` on clean input while remaining several times faster than
`jsonc-eslint-parser`.
Single-run samples bound nothing smaller than their run-to-run spread;
 the commented-input
movement between the two runs (about 8 percent) is a rough indication of that spread,
 and the
clean-input change is far larger than it.

Guards added with the amendment,
 each shown failing before the fix and passing after:
 clean number spelling through `jsoncStringify`,
 clean depth 512 accepted and 513 rejected,
 a
later-line comma in arrays and objects,
 comment ownership around a later-line comma,
 CR and
CRLF line-comment termination,
 and a multi-line value comment keeping its owner across
emission and reparse.
The package unit suite (58 assertions groups),
 the conformance corpus and the fuzz property
suites pass after the change.
