# Module-test failure values as a difference, not two cut renders

Proposal for issue 610:
 a failing `expect(actual).toEqual(expected)` should name what differs,
 because the compared values are large enough that no whole-value render
 helps.
 Evidence and the chai-side diagnosis:
 [doc/troubleshooting/chai-failure-value-truncation.md](../troubleshooting/chai-failure-value-truncation.md).

## Problem as measured

- chai 6.3.0 cuts failure values at `config.truncateThreshold` (default `40`),
  so a failing compare prints `expected [ …(31) ] to deeply equal []`
  and nothing a reader can act on.
- Real compared values are not small.
  One source-scan finding entry is `174` characters
  (measured over the `promise-combinators-in-order` fixtures),
  and the issue's `31`-entry case is `5394` characters of near-duplicate
  sentences.
  Package-wide scan cases read the whole translation-repair family source.
- Real failures are also record-shaped:
  `expected { lines: [ …(2) ], …(2) } to deeply equal { lines: [ …(2) ], returned: 1, …(1) }`
  (`package/translation-repair/doc/audit-ledger-ambient-state-touched-by-library-code.md:44`).
- The repository already requires this shape of failure output:
  `package/translation-repair/source-scan/src/scan-direct-comparisons.unit.test.ts`
  fails any package-wide scan case that compares "through `toEqual` or
  `toStrictEqual` rather than a helper that names what differs".

## Options considered

### Capped whole-value render in the failure printer

Print `actual` and `expected` from the `AssertionError`,
 one item per line,
 behind a cap.

Pros:
 no chai config change,
 bounded output,
 simple.

Cons,
 measured:
 the cap hides the differing entry whenever it sits past the cap,
 which at `174`-character entries is the common case for a list compare;
 two `5394`-character renders still leave the reader scanning.
 Rejected after the sizing correction.

### Whole-value render plus `truncateThreshold = 0`

Pros:
 raw `error.message` also shows values.

Cons,
 measured:
 `1 488 921`-character messages on a `200 000`-number compare,
 and the failure render reads every getter and proxy trap in the values
 (`40` of `40` versus `2` of `40` at the default).
 Rejected.

### Difference-first structural diff in the failure printer

Read `AssertionError.actual` and `.expected`
 (chai sets both,
 installed bundle `index.js:1539`),
 compute the difference,
 and print only that,
 one difference per line.

Pros:
 output scales with the difference,
 not the value,
 so a `31`-entry list differing in one entry prints one line;
 covers list,
 record,
 and string compares,
 matching the measured failure shapes;
 no chai config change,
 so the flood and getter-walk costs cannot happen.

Cons:
 the most code,
 and the diff must handle cycles,
 mixed types,
 and getter-bearing values without derailing.

Chosen.

## Chosen design

- Owner:
  `package/test-harness/core-diagnostic`,
  beside `format-error.ts`,
  which today reads only `message`,
 `name`,
 `stack`,
 `cause`,
 and `errors`.
- Lists:
  copy-for-copy difference,
  generalizing `entryDifference` from
  `package/translation-repair/test-fixture/src/scan-findings.test-fixture.ts`
  (`foundNotListed`,
  `listedNotFound`,
  `reordered`).
  Whether that fixture then consumes the shared implementation is follow-up
  scope,
  proposed explicitly rather than assumed.
- Records:
  key-level difference
  (missing,
  extra,
  changed),
  recursing into changed values.
- Strings:
  a line-level difference with context.
- Scalars and type mismatches:
  both values,
  which are already small.
- Caps:
  on the number of differences shown,
  with a visible `and N more differences` marker,
  never on the compared input's size.
  Whether a single megabyte-scale entry (a whole page string) needs its own
  marker is an open limit,
  recorded rather than decided here.
- chai's `truncateThreshold` stays at its default;
  the headline may still read `[ …(31) ]` while the difference lines below it
  name the entry.

## Acceptance criterion

A failing `toEqual` over `31` findings of `174` characters each,
 differing in one entry,
 prints that one entry as the difference,
 not both lists.
 A failing record compare names the differing keys.

## Outcome

Shipped in `package/test-harness/core-diagnostic`,
 exported from `package/test-harness/core-diagnostic/src/index.ts`:

- `valueDifferenceLines` and `errorDifferenceLines` in
 `package/test-harness/core-diagnostic/src/value-difference.ts`,
 with the walks split across `value-walk.ts` and `value-walk-collections.ts`
 and shared plumbing in `difference-sink.ts`.
- Entry pairing in `value-entry-key.ts` and `list-difference.ts`,
 string lines in `string-difference.ts`,
 single-line renders in `value-representation.ts`.
- `formatNode` in `package/test-harness/core-diagnostic/src/format-error.ts`
 appends indented difference lines under an error carrying `actual` and
 `expected`.

Measured at the consumer boundary (`expect` from
`@monochromatic-dev/test-harness-core/ts` through `formatFailure`):
the `31`-finding acceptance case renders chai's truncated headline plus
two lines naming the one changed entry pair.
 A failing record compare names its keys.

Decided beyond the original sketch:

- The diff block stays silent when neither compared side can hold content
 the headline truncates,
 so matcher fields like `satisfy`'s predicate result never gain a
 misleading diff.
- Map values resolve through the paired expected key,
 since deep-equal map keys need not share identity.

Open limits recorded:

- A single megabyte-scale entry still renders whole on its difference
 line.
- The scan-findings fixture consuming `entryDifference` is follow-up
 scope.
- The chai-side upstream draft stays local,
 unposted.
