# Handover: splitting JSONC out of module-es into module-jsonc-edit

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/jsonc-edit.conformance` is now `package/jsonc-edit/core-ts.conformance`;
> `package/module/jsonc-edit.bench` is now `package/jsonc-edit/core-ts.bench`;
> `package/module/jsonc-edit.fuzz` is now `package/jsonc-edit/core-ts.fuzz`;
> `package/module/jsonc-edit` is now `package/jsonc-edit/core-ts`;
> `package/module/toml-edit` is now `package/toml-edit/core`.

Living status doc.
Updated as the package is scaffolded and implemented.

## Goal

Extract the hand-written comment-preserving JSONC parser from
`package-paused/module/es` into a new active package `package/module/jsonc-edit`
(`private: true`),
 and build a full read,
 edit,
 and write API on top,
 shaped after
`package/module/toml-edit`.
Performance is an explicit goal (the native `JSON.parse` fast-path).

## Where this work lives

- Worktree:
  `/var/home/user/worktrees/jsonc-edit` on branch `feat/jsonc-edit`.
- The whole feature lives on that branch;
  merge to `main` when ready.

## Decisions (locked)

See `doc/decision/jsonc-edit-parser-foundation.md` for the full rationale.
Summary:

- Keep the hand-written parser (clean rewrite),
   do not adopt a library.
  Every surveyed library fails a reason-for-existence:
  merged per-key and per-value comment model,
   `JSON.parse` fast-path,
   zero-dep,
  run-everywhere (no WebAssembly),
   branded types.
- Write model:
  canonical rebuild from the structured tree,
   comments as first-class data.
  No source ranges,
   no splice mode.
  Raw scalar tokens preserved for unedited values;
  duplicate keys preserved losslessly.
- API:
  immutable `JsoncEditState`,
   named free-functions with a `jsonc` prefix.
  The `$` / namespace export style is dropped.
- Comment API:
  attached model (`jsoncGetComment` / `jsoncSetComment` for a value's comment,
  `jsoncGetKeyComment` / `jsoncSetKeyComment` for a key's comment).
- Tests:
  full `toml-edit` parity (unit,
   fast-check fuzz,
   STB adversarial boundaries,
   real-world
  corpus,
   V8 coverage-baseline gate,
   a curated JSONC conformance harness).
- es cleanup:
  delete the `t object/t jsonc` subtree plus the t-string and t-boolean jsonc type bits
  from `module-es`,
   and drop the jsonc namespaces from its barrels.

## Public API (target)

- `parseJsoncEdit`,
   `jsoncStringify`
- `jsoncGet`,
   `jsoncGetValue`,
   `jsoncHas`,
   `jsoncKeys`,
   `jsoncSet`,
   `jsoncDelete`
- `jsoncGetComment`,
   `jsoncSetComment`,
   `jsoncGetKeyComment`,
   `jsoncSetKeyComment`

## How to run (in the worktree)

```bash
# from /var/home/user/worktrees/jsonc-edit
mise i                                                    # install pinned toolchain (done once)
mise run //package/module/jsonc-edit:build
mise run //package/module/jsonc-edit:lint:oxlint
mise run //package/module/jsonc-edit:lint:types
mise run //package/module/jsonc-edit:test:unit          # co-located unit tests
mise run //package/module/jsonc-edit:test:mutation -- --full-suite

# Non-runtime tooling lives in per-concern sidecar packages:
mise run //package/module/jsonc-edit.fuzz:test:unit      # property tests (default budget)
mise run //package/module/jsonc-edit.fuzz:fuzz           # longer fast-check campaign
mise run //package/module/jsonc-edit.fuzz:fuzz:coverage  # V8 reachability gate
mise run //package/module/jsonc-edit.bench:bench         # parse benchmark
mise run //package/module/jsonc-edit.conformance:test:conformance
```

## Status

Complete and merged to `main`.
 The runtime package builds,
 type-checks,
 and lints clean
(zero errors,
 zero warnings);
 its co-located unit tests pass;
 the fuzz,
 bench,
 and
conformance sidecars pass;
 the coverage gate reports all 17 runtime source files reachable.
Both original follow-ups are done (logger OPFS fix picked up via merge;
 the troubleshooting
docs repointed to `package/module/jsonc-edit`).

The non-runtime tooling now lives in per-concern sidecar packages so the runtime package's
`src/` is pure production code,
 which scopes a whole-package mutation run to real runtime
files:

- `module-jsonc-edit.fuzz`:
   fast-check property tests,
   run budget,
   V8 coverage-reachability gate.
- `module-jsonc-edit.bench`:
   parse benchmark.
- `module-jsonc-edit.conformance`:
   curated JSONC conformance corpus.

Mutation testing is wired via `//package/module/jsonc-edit:test:mutation` (container-isolated
Stryker,
 mirroring file-enforcer).
 `--full-suite` is required because the unit tests are
organized by API surface (`parse`,
 `stringify`,
 `edit`,
 `comment`),
 not per source file,
 so the
harness's filename-stem test selection would otherwise pick no tests for files like `scan.ts`.

Progress (newest first):

- 2026-06-30 (mutation hardening to the equivalent-mutant floor):
  Drove the whole-package score from 68.69% to 95.76% (474 killed / 12 survived / 9 timeout
  / 588 compile-error) by adding branch-level unit tests for every reachable path:
   direct
  tests for the internal `scan`,
   `parse-trivia`,
   `parse-close`,
   emit,
   and `parse-scalar`
  helpers,
   error-name assertions,
   and deeper `edit`,
   `comment`,
   `parse`,
   and `stringify`
  coverage (array-element set versus append,
   out-of-range and missing-segment errors,
  duplicate-key last-wins,
   numeric-segment-into-record and string-segment-into-array kind
  errors,
   dangling-comment folding,
   depth-guard limits,
   exact canonical layout,
   lone
  slash/star inputs that are not comments).
   Every remaining survivor was verified to be a
  genuine equivalent mutant or an infinite-loop mutant the harness detects by timeout.
  Counting the 9 timeouts as detected (Stryker's default) and excluding the 12 equivalents,
  every non-equivalent mutant is killed.

  A diagnostic detour is written up in `doc/troubleshooting/stryker-survivor-triage.md`:
  Stryker emits one `ConditionalExpression` mutant per `&&`/`||` operand plus the whole
  test,
   told apart only by column,
   so a survivor read by line alone and reproduced as the
  whole-condition variant looked like a harness bug but was a per-operand equivalent.
   The
  harness itself was verified correct.

  Equivalent mutants (Survived,
   no test can distinguish them):
  - `edit-navigate.ts:57` `(typeof segment === 'string') -> true`:
     a record reached with a
    numeric segment gives `findLast` no match,
     so the result is `NODE_ABSENT` either way
    (record keys are strings,
     and `string === number` is always false).
  - `parse-close.ts:33` and `:120` `(dangling.length === 0) -> false`:
     folding an empty
    dangling list is a no-op,
     since `appendComments` returns the node unchanged for it.
  - `parse-jsonc.ts:16` and `:50` (logger tag and trace strings to `""`):
     log-only,
     no parse
    output depends on them.
  - `parse-jsonc.ts:76` `(parsed === FASTPATH_MISS) -> false`:
     `FASTPATH_MISS` is a symbol
    already excluded by the following `typeof parsed !== 'object'` arm.
  - `parse-trivia.ts:47` and `:160`,
     `scan.ts:70` and `:181`
    `cursor < source.length -> cursor <= source.length`:
     the loop returns on an `undefined`
    character,
     so the extra boundary iteration produces the same result.
  - `parse-trivia.ts:165` `char === '\n' -> false` and `'\n' -> ""`:
     the newline branch
    returns the same `{ comments, commaSeen, end }` the generic fall-through already returns.

  Timeout-detected mutants (infinite loops;
   Stryker's default scores these as killed):
  - `parse.ts:127` and `:230`,
     `parse-trivia.ts:47` and `:160` (loop body to `{}`):
     removing
    the cursor advance makes the scan loop never terminate.
  - `scan.ts:78` `cursor += 1 -> cursor -= 1`:
     the string-escape skip runs backward.
  - `scan.ts:351` (line-comment newline literal to `""`):
     `indexOf('')` returns the start
    offset,
     so the comment scanner never advances.
  - `scan.ts:401` cluster:
     breaking the unterminated-block-comment close check makes
    `scanBlockComment` return a non-advancing end,
     so the caller loops forever.
- 2026-06-30 (sidecars + mutation testing):
  Extracted the non-runtime tooling into three per-concern sidecar packages
  (`jsonc-edit.fuzz`,
   `jsonc-edit.bench`,
   `jsonc-edit.conformance`) so the runtime
  package src is pure production code;
   wired container-isolated Stryker mutation testing
  (`test:mutation`,
   `--full-suite`).
   Fixed a pre-existing bootstrap failure in the shared
  mutation runtime image:
   the baked `npm.package_manager = "pnpm"` made a fresh container
  try to install pnpm with pnpm;
   the Containerfile now forces `MISE_NPM_PACKAGE_MANAGER=npm`
  for that install step.
   Runtime package and all three sidecars verified green.
  Taught the mutation harness (`selectTestsForSource`) to also include sibling sidecar
  `*.unit.test.ts` files as mutant killers;
   they resolve to the mutated `/work` source through
  the workspace `/ts` relative symlink,
   so the fuzz and conformance suites participate.
   That
  lifted the whole-package score from 59.60% (295 killed / 191 survived,
   unit tests only)
  to 68.69% (340 killed / 146 survived / 9 timeout / 588 compile-error).
   Remaining survivors
  cluster in the edit and comment write API (`edit-set` 63%,
   `edit-comment` 52%) and the
  close/trivia parse paths (`parse-close` 7%),
   which the property and conformance suites do not
  exercise;
   hardening those unit tests is the natural follow-up.
- 2026-06-30 (complete):
  Lint-remediated the parser to the functional rules,
   then built the canonical
  serializer,
   the immutable edit API,
   the comment-as-data API,
   the unit/property/
  conformance test suites,
   the coverage-baseline gate,
   and the parse benchmark;
   removed
  the JSONC code from the paused `module-es`.
   All verified end to end against the built
  bundle.
- 2026-06-30 (later):
  Package scaffolded and building.
   Value model (`comment.ts`,
   `brand.ts`,
  `value.ts`) done,
   type-checks and lints clean.
   Parser core written:
  `errors.ts`,
   `merge-comments.ts`,
   `scan.ts`,
   `parse-trivia.ts`,
   `parse.ts`,
  `parse-jsonc.ts`.
   The parser type-checks but does NOT yet pass oxlint:
  it is written in an imperative cursor style that violates the repo's
  functional rules.
   Committed as WIP.
   Lint remediation is the next step,
  before the serializer.
- 2026-06-30:
  worktree created,
   toolchain installed,
   decision doc written,
   task list set up.

## Lint remediation plan for the parser core

The parser logic is correct and type-checks;
 only its style needs reshaping to
satisfy these rules (learned from the rule sources and fixtures):

- `no-restricted-syntax/no-function-root-let`:
  a root-level `let` is allowed only in the "helper shape" where the function
  ends with `return <thatIdentifier>`.
   Otherwise move the cursor into a
  `for (let cursor = start; cursor < source.length; )` init (empty increment,
  advance inside),
   which is not a function-root `let`.
   Accumulators (`comment`,
  `commaSeen`) should become a `const` array pushed to in the loop,
   then reduced
  or derived at the single return.
- `no-restricted-syntax/no-nullish-union` (shortcode `l`):
  no `T | undefined` or `T | null` annotations.
   Use `?:` optionals,
   plain `T`,
  or a `Symbol` sentinel.
   So `TriviaScan`,
   `TrailingScan`,
   `ScalarScan`,
   and the
  helper params that read `comment?: JsoncComment | undefined` drop the
  `| undefined`;
   locals typed `X | undefined` get restructured away (collect into
  a `const` array,
   conditionally include the property at return so no `undefined`
  is ever assigned to an optional).
- `eslint(init-declarations)`:
  every binding initialized at declaration.
- `exactOptionalPropertyTypes`:
  never assign `undefined` to an optional property;
   build the return object with
  the property present only when defined (`length === 0 ? { end } : { comment, end }`).

Concretely,
 restructure `scan.ts` (`scanString`,
 `scanNumber`),
`parse-trivia.ts` (`skipTrivia`,
 `captureTrailing`),
 and the loops in `parse.ts`
to the for-init-cursor plus const-accumulator shape.
 Also one file trips
`max-lines` (300 code lines):
 split `parse.ts` (move `closeArray`/`closeRecord`
and the entry helpers to a sibling,
 keep the mutually-recursive
`parseValue`/`parseArray`/`parseRecord`/`parseEntry` together).

## Task map

Tracked in the session task list.
Phases:
decision doc (done),
this handover (living),
scaffold,
types,
parser rewrite,
serializer,
edit API,
comment API,
tests,
fuzz,
conformance,
coverage gate,
benchmarks,
es cleanup,
final verify.

## Source of truth for behavior

The original parser and its tests under
`package-paused/module/es/src/types/t object/t jsonc/` are the behavioral spec.
The rewrite preserves their invariants;
it does not preserve their structure or style.

## Open implementation notes

- The value model needs raw scalar text on nodes so canonical emit can preserve
  author formatting (`1.0` stays `1.0`) for unedited values.
- `fastPath.ts` and a few siblings exceed the 300-line `max-lines` budget;
  split on the way in,
   do not carry the violation over.
- Benchmarks compare against `jsonc-eslint-parser` and `jsonc-parser`
  (devDependencies of this package).
