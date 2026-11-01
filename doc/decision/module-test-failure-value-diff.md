# Module-test failure values render as a difference, not whole values

## Status

Accepted 2026-10-21.
Proposal in `doc/planning/module-test-failure-value-diff.md`,
 accepted by the repository owner after a sizing correction that rejected
 whole-value renders.
 Closes issue 610's question of what a failing `toEqual` should print.
 chai-side diagnosis and the upstream draft kept local per the owner:
 `doc/troubleshooting/chai-failure-value-truncation.md`.

## Context

A failing `expect(actual).toEqual(expected)` printed both values cut to
chai's `config.truncateThreshold` (default `40` characters),
 so the failure said the values differ but not how
 (issue 610).
 Two whole-value render options were measured and rejected:

- capped renders hide the differing entry whenever it sits past the cap;
  real scan findings are `174`-character sentences
  (measured over the `promise-combinators-in-order` fixtures),
  so the issue's `31`-entry case is `5394` characters of near-duplicate
  text with one differing entry.
- uncapped renders (`truncateThreshold = 0`) flood:
  `1 488 921` characters on a `200 000`-number compare,
  and the render reads every getter and proxy trap in the values
  (`40` of `40` versus `2` of `40` at the default).

The repository already demanded the chosen shape:
 `package/translation-repair/source-scan/src/scan-direct-comparisons.unit.test.ts`
 fails any package-wide scan case comparing "through `toEqual` or
 `toStrictEqual` rather than a helper that names what differs".

## Decision

`package/test-harness/core-diagnostic` renders the difference between
 `AssertionError.actual` and `.expected`
 (chai sets both),
 one difference per line,
 and never renders whole values:

- lists:
  copy-for-copy difference
  (`foundNotListed`,
  `listedNotFound`,
  `reordered`),
  generalizing `entryDifference` in
  `package/translation-repair/test-fixture/src/scan-findings.test-fixture.ts`.
- records:
  key-level difference (missing,
  extra,
  changed),
  recursing into changed values.
- strings:
  line-level difference with context.
- scalars and kind mismatches:
  both values,
  already small.
- caps:
  on the number of differences shown,
  with a visible `and N more differences` marker,
  never on compared input size.

chai's `truncateThreshold` stays at its default,
 so the flood and getter-walk costs cannot occur.

## Alternatives rejected

- `config.truncateThreshold = 0` in `module-test`:
  measured flood (`1 488 921` characters) and getter walks
  (`40` of `40`).
- large finite threshold:
  measured non-monotonic;
  thresholds `34` and `40` collapse a `31`-item array to `[ Array(31) ]`
  while `35..39` and `41+` show partial listings,
  and tails still elide where differences sit.
- capped whole-value render in the failure printer:
  the cap hides the differing entry at measured real sizes,
  which is the failure mode being fixed.

## Costs accepted

- The diff is the most code of the options and must handle cycles,
  mixed kinds,
  and getter-bearing values without derailing.
- A single megabyte-scale entry
  (one whole page string) still renders whole beside its difference;
  recorded as an open limit in the proposal,
  not decided here.
- chai's headline may still read `expected [ …(31) ]`;
  the difference lines below it name the entry.

## Revisit triggers

- A failing compare whose difference no diff line names
  (record recursion gap,
  or a value kind the diff flattens to scalars).
- Whole-value rendering requested for a consumer that reads
  `error.message` alone,
  with the flood budget named.
- `scan-findings.test-fixture.ts` consuming the shared diff,
  dropping its private `entryDifference`.
