# @monochromatic-dev/jsonc-edit-core-ts

Comment-preserving JSONC (JSON with comments) read,
 edit,
 and write.
A free-function edit API over an immutable `JsoncEditState`,
 with a canonical serializer
that treats comments as first-class,
 queryable data.

Sibling of `@monochromatic-dev/toml-edit-core`.
Where `toml-edit` wraps a third-party parser and offers byte-identical splice,
 `jsonc-edit`
keeps a hand-written parser whose distinctive value is a normalized comment model:
every object key and every value carries at most one comment,
 and stacked `//` or `//region`
lines merge into a single comment.
Ownership follows lines:
 a comment before a separator belongs to the key or value it precedes,
 and one after a separator on a later line belongs to the next key,
 while a line ends at LF or CRLF only,
 so a bare CR terminates a comment body without starting a new line.
`doc/decision/jsonc-comment-line-ownership.md` records that choice,
 and the Rust crate reads the same rule from the shared fixtures.
See `doc/decision/jsonc-edit-parser-foundation.md` for why no off-the-shelf parser fits.

## Status

Implemented:
 parser,
 canonical serializer,
 immutable edit API,
 and comment-as-data API,
with unit tests co-located in this package and property,
 conformance,
 benchmark,
 and
coverage-gate tooling in sidecar packages (see below).
See `doc/handover/jsonc-edit.md` for the build history.

## Tooling sidecars

This package's `src/` is kept pure runtime code so a whole-package mutation run stays scoped
to real runtime files.
 The non-runtime tooling lives in per-concern sidecar packages beside it:

- `@monochromatic-dev/jsonc-edit-core-ts.fuzz`:
   fast-check property tests (round-trip and the STB
  comment-safety guard),
   the run budget,
   and the deterministic V8 coverage-reachability gate.
- `@monochromatic-dev/jsonc-edit-core-ts.bench`:
   the parse benchmark against microsoft
  `jsonc-parser` and `jsonc-eslint-parser`.
- `@monochromatic-dev/jsonc-edit-core-ts.conformance`:
   the curated JSONC conformance corpus.

Each sidecar is private and imports this package through its `/ts` source subpath.

Mutation testing runs against this package via container-isolated Stryker:

```sh
mise run //package/jsonc-edit/core-ts:test:mutation -- --full-suite
```

`--full-suite` is required here because the unit tests are organized by API surface
(`parse`,
 `stringify`,
 `edit`,
 `comment`),
 not per source file,
 so every source file must run
against the whole unit suite rather than a filename-stem-matched subset.

The harness also includes the sidecar packages' `*.unit.test.ts` files as mutant killers (it
discovers sibling `jsonc-edit.<concern>` packages automatically),
 so the property and conformance
suites participate even though they live outside the runtime package.

The branch-level unit suite drives the whole-package score to 95.76% (474 killed,
 12 survived,
9 timeout,
 588 compile-error).
 Every remaining survivor is a verified equivalent mutant (no test
can distinguish it,
 for example a loop bound guarded by an `undefined` check or a log-only string)
and every timeout is an infinite-loop mutant that Stryker's default scoring counts as killed.
 So
every non-equivalent mutant is killed.
 The equivalents and timeouts are enumerated with their
proofs in `doc/handover/jsonc-edit.md`,
 and a survivor-triage pitfall (per-operand mutants that
look like a harness bug) is written up in `doc/troubleshooting/stryker-survivor-triage.md`.

## Why JSONC, and why canonical

- Comments survive read,
   edit,
   and write,
   and are addressable as data by path.
- Every document parses structurally,
   so clean and commented input keep the same number
  spellings,
   comment ownership and nesting limit.
- Runs in every modern environment:
  zero runtime dependencies,
   no WebAssembly.
- Canonical write model:
  unedited values keep their raw scalar text,
   all comments are preserved,
   and formatting is
  deterministic.
  This is not byte-identical splice;
  whitespace between tokens is normalized.

## API

```ts
import {
  parseJsoncEdit,
  jsoncStringify,
  jsoncGetValue,
  jsoncHas,
  jsoncKeys,
  jsoncSet,
  jsoncDelete,
  jsoncGetComment,
  jsoncSetComment,
  jsoncGetKeyComment,
  jsoncSetKeyComment,
  COMMENT_ABSENT,
} from '@monochromatic-dev/jsonc-edit-core-ts';

const state = parseJsoncEdit({ source: '{ "host": "localhost" } // cfg' as StringJsonc, },);
const next = jsoncSetComment({ state, path: ['host',], comment: { type: 'inline', text: ' default', }, },);
const text = jsoncStringify({ state: next, },);
```

The lower-level `parseJsonc` (returning a `JsoncValue`) and `emitJsoncValue` are also
exported for callers that do not need the edit state.

## Performance

The `mise run //package/jsonc-edit/core-ts.bench:bench` task compares parsing against
microsoft `jsonc-parser` and `jsonc-eslint-parser`.
On a representative run (300 entries,
 3000 iterations,
 jsonc-edit parsing both inputs structurally):

- Clean input:
   jsonc-edit runs at roughly 2794 ops/s,
  about level with microsoft `jsonc-parser` (2733 ops/s) and roughly six times faster than
  `jsonc-eslint-parser` (479 ops/s).
- Commented input:
   jsonc-edit runs at roughly 3591 ops/s,
  against microsoft `jsonc-parser` at 4193 ops/s and `jsonc-eslint-parser` at 876 ops/s,
  while also retaining comments as queryable data the other two do not model.

An earlier revision took a native `JSON.parse` shortcut for comment-free documents and
measured roughly 11055 ops/s on clean input.
That shortcut was removed because it dropped unedited number spellings (`1e0` emitted as
`1`) and bypassed the 512-container nesting limit,
   both of which the supported behavior
requires.
The roughly fourfold clean-input cost is the measured price of that correctness;
 run the task
to reproduce the numbers locally,
 since they are machine-dependent.

## Comment model

A parsed JSONC value is a discriminated union covering string,
 number,
 boolean,
 null,
 array,
record,
 and a `plainJson` leaf that a caller may build from native JSON.
The parser itself always produces structured nodes;
the edit API normalizes a `plainJson` leaf before editing it.
Every node may carry a `comment` of type `inline`,
 `block`,
 or `mixed`.
Record keys are themselves comment-bearing,
 so `{ /* a */ "k": /* b */ "v" }` attaches
`a` to the key and `b` to the value.
Duplicate keys are malformed input:
 the behavior is undefined and not a supported
contract.
 Correctly handling malformed JSON or JSONC is not a design goal,
 so which member a read,
 edit or delete resolves to when a key repeats is unspecified.
Every document takes the same structured parse path,
 so clean and commented input cannot
diverge.
