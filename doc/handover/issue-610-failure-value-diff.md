# Issue 610 fix in progress: module-test failure values as a difference

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/test-diagnostic` is now `package/test-harness/core-diagnostic`;
> `@monochromatic-dev/module-test` is now `@monochromatic-dev/test-harness-core`.

## Purpose

The owner asked what to do about
[Aquaticat/Monochromatic#610](https://github.com/Aquaticat/Monochromatic/issues/610):
 a failing `toEqual` prints `expected [ …(31) ] to deeply equal []`,
 hiding what differs.
 The owner chose the full difference-first diff for
 `package/module/test-diagnostic`,
 and chose to keep the chai-side upstream draft local.
 Halted mid-implementation for the 2026-10-21 reboot.

## Decisions already made and recorded

- `doc/decision/module-test-failure-value-diff.md`:
  accepted 2026-10-21.
  Whole-value renders rejected after a sizing correction
  (real findings are `174`-character sentences,
  so any display cap hides the differing entry),
  leaving a difference-first structural diff,
  caps only on the number of differences shown.
  chai's `truncateThreshold` stays at its default.
- `doc/troubleshooting/chai-failure-value-truncation.md`:
  chai 6.3.0 diagnosis,
  probe harness,
  prototyped upstream fix in `chai-failure-value-truncation.patch`,
  upstream draft kept local per the owner (2026-10-21).
- `doc/planning/module-test-failure-value-diff.md`:
  the accepted proposal with the three rejected options and their
  measurements.

## What exists so far

One uncommitted-at-reboot source file,
 now committed as WIP:

- `package/module/test-diagnostic/src/value-representation.ts`:
  `valueKind`,
  `valueRole`,
  `renderValue`,
  `enumerableKeys`,
  `describeSide`.
  **Known flaw to fix first:**
  `enumerableKeys`'s guard admits non-enumerable string keys
  (the `else if` branch is dead and the first clause ignores
  enumerability for strings).
  Correct rule from the chai probes:
  own enumerable string keys plus own enumerable symbol keys,
  in `Reflect.ownKeys` order.
  Also unverified against lint and types.

## What remains

1. Fix `enumerableKeys` as above;
   run `mise run //package/module/test-diagnostic:lint:types`
   and `lint:oxlint`.
2. `value-difference.ts`:
   the walk producing difference lines.
   Proven model is `entryDifference` and `differenceLines` in
   `package/translation-repair/test-fixture/src/scan-findings.test-fixture.ts`
   (copy-for-copy `foundNotListed`/`listedNotFound`/`reordered`).
   Lists copy-for-copy with order,
   records key by key recursing into changed values,
   strings line-level with context,
   leaves render both values,
   kind mismatches use `describeSide`.
   Caps on difference count with a visible `and N more differences`
   marker.
   Cycle-safe (pair-visited guard).
3. Wire into `package/module/test-diagnostic/src/format-error.ts`
   where an error carries `actual`/`expected`
   (chai sets both on `AssertionError`),
   without changing the one-line-per-error contract:
   difference lines follow the error's own line.
   Re-export from `src/index.ts`.
4. Unit tests in `src/value-difference.unit.test.ts`
   covering every branch,
   including the acceptance case from
   `doc/decision/module-test-failure-value-diff.md`:
   31 findings of 174 characters differing in one entry prints that
   one entry,
   not both lists.
   Import pattern matches `src/format-error.unit.test.ts`.
5. Consumer probe via `@monochromatic-dev/module-test`'s public API,
   then commit with `Closes #610`.

## Measured semantics the walk must match

From chai 6.3.0 probes (outputs in the session):
 `NaN` equals `NaN`,
 `+0` differs from `-0`,
 objects compare by own enumerable string and symbol keys in any order
 with deep values,
 non-enumerable keys ignored,
 getters read as values,
 arrays compare ordered and ignore extra non-index keys,
 holes equal `undefined`,
 `Map` and `Set` compare unordered with deep keys,
 `Date` by time value (`Invalid Date` equals `Invalid Date`),
 `RegExp` by source and flags,
 boxed primitives by inner value,
 errors by name and message (`TypeError('m')` differs from `Error('m')`),
 functions and promises and weak collections by identity,
 typed arrays and `ArrayBuffer` and `DataView` by contents and kind
 (`Uint8Array` differs from `Int8Array`),
 cycles by shape (`cycle vs flat` fails).

## How to resume

Run `git log --oneline -4` and read
`doc/decision/module-test-failure-value-diff.md` first,
 then follow "What remains".
 Scratch probes from the diagnosis live in `${HOME}/temp/agent/`
 (`chai-truncate-probe.mjs`,
 `upstream-prototype.ASOoaKUU/`),
 disposable;
 nothing depends on them.
