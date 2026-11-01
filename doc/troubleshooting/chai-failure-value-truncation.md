# chai 6.3.0 `config.truncateThreshold` cuts failing compare values at 40 chars; finite thresholds are non-monotonic

Scope note (issue 610):
 this records the diagnosis behind
[`Aquaticat/Monochromatic#610`](https://github.com/Aquaticat/Monochromatic/issues/610),
 where `@monochromatic-dev/test-harness-core`'s failing `toEqual` printed
`expected [ …(31) ] to deeply equal []` and hid the compared values.
 The consumer-side fix choice for that issue is a separate decision;
 this file holds the chai-side mechanism,
 measurements,
 workarounds,
 and the upstream prototype.

## Symptom

A failing deep comparison built on chai 6.3.0 prints both sides cut to
`config.truncateThreshold` characters (default `40`),
 so the failure line says the values differ but not how.
 Measured message shapes at the default,
all on `expect(actual).to.deep.equal([])`
 or an unequal partner:

- a 31-item list of records:
  `expected [ …(31) ] to deeply equal []`
  (message length `37`),
  and two unequal 31-item lists:
  `expected [ …(31) ] to deeply equal [ …(31) ]`
  (length `44`)
- a 31-item number list:
  `expected [ Array(31) ] to deeply equal []`
  (length `41`),
  the loupe-elided render already exceeding `40` characters
- long strings:
  `expected 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx…' to deeply equal 'yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy…'`
  (length `106`)
- a 12-key record:
  `expected { key0: +0, key1: 1, key2: 2, …(9) } to deeply equal {}`
- a depth-5 nested object,
  where even the keys are dropped:
  `expected { Object (a) } to deeply equal { Object (a) }`

The trigger is any compared value whose loupe render reaches the threshold
(40 characters by default),
 independent of assertion kind:
 `toEqual`,
 `toStrictEqual`,
 `toContain`,
 `deep.include`,
 and `deep.equal` all share the display path.

## Root cause

The display chain runs in three stages,
 and the cut is intentional in stage 2 but mis-decided in stage 3.

### Stage 1: the threshold is the render budget

chai's `config.truncateThreshold` defaults to `40`
 (`node_modules/.pnpm/chai@6.3.0/node_modules/chai/index.js:860`,
 same in upstream `lib/chai/config.js:49` at commit `78d9637`):

```js
  truncateThreshold: 40,
```

`utils.inspect` passes that threshold to loupe as its character budget
 (`node_modules/.pnpm/chai@6.3.0/node_modules/chai/index.js:928`,
 upstream `lib/chai/utils/inspect.js:28`):

```js
    truncate: config.truncateThreshold ? config.truncateThreshold : Infinity
```

A falsy threshold is the documented disable
 (`lib/chai/config.js:38`:
 "Set it to zero if you want to disable truncating altogether").
 loupe then elides list entries,
string tails,
and object values to fit
 the budget,
marking drops as `…(N)`.

### Stage 2: objDisplay replaces the render with a shape summary

After the budgeted render,
 `objDisplay` compares the render's own length to the same budget and,
 on a hit,
replaces it with a type summary
 (`node_modules/.pnpm/chai@6.3.0/node_modules/chai/index.js:937`,
 `:949`,
 `:956`):

```js
  if (config.truncateThreshold && str.length >= config.truncateThreshold) {
    ...
      return "[ Array(" + objAsArray.length + ") ]";
    ...
      return "{ Object (" + kstr + ") }";
```

This is the `expected [ …(31) ]` and `expected [ Array(31) ]` behaviour:
 `str` is the already-elided render,
 so the decision reads the elided length,
 not the value's.

### Stage 3: the decision is non-monotonic in the threshold

Because both stages share one number,
 raising the threshold moves the render budget and the replacement
 boundary together and they cross unpredictably.
 The same 31-item number list (full render `145` characters) measured:

- threshold `30`:
  `expected [ +0, 1, 2, 3, 4, 5, …(25) ] to deeply equal []`
  (partial content,
  length `56`)
- threshold `34`:
  `expected [ Array(31) ] to deeply equal []`
  (collapsed,
  length `41`)
- threshold `35` to `39`:
  partial `…(n)` listings again (length `62` to `65`)
- threshold `40`:
  `expected [ Array(31) ] to deeply equal []`
  (collapsed again)
- threshold `41` and up to `144`:
  partial listings of growing size

So a finite "large" threshold does not buy reliable output:
 the collapse still lands on specific budgets,
 and just below the full render the listing still elides the tail
 where the difference often lives.

### Why a consumer cannot fix this through module-test

`@monochromatic-dev/test-harness-core-expect` builds `expect` on chai
 (`package/test-harness/core-expect/src/expect-matchers.ts:13`,
 plugin registration `:28`),
 and neither `module-test` nor `module-test-expect` re-exports chai's `config`
 (`package/test-harness/core/src/index.ts:33`,
 `package/test-harness/core-expect/src/index.ts:1`).
 chai's `expect` object carries no `config` property either
 (measured:
`typeof expect.config === 'undefined'`),
 and overriding `chai.util.objDisplay` is rejected
 (measured:
`TypeError: Cannot set property objDisplay of #<Object> which has only a getter`).
 The harness diagnostic printer
 (`package/test-harness/core-diagnostic/src/format-error.ts:149`)
 reads `message`,
 `name`,
 `stack`,
 `cause`,
 and `errors` only;
 the `AssertionError` also carries raw `actual` and `expected`
 (`node_modules/.pnpm/chai@6.3.0/node_modules/chai/index.js:1539`),
 which no workspace printer reads.

## Verification

Version under test:
 chai `6.3.0` from the pnpm store,
 tarball integrity
 `sha512-XWAtwJ6OHO+tj0EKCs0Y2UamnyOxseZWltU4x2U2wh8g4AigdjwvtUjvLP2tqkA/avxHEtzxNaqGq/YGNwckKg==`
 (from `npm view chai@6.3.0 dist.integrity`),
 matching the upstream `lib/` sources cloned at commit
 `78d9637c79e211ed4f2455d9ba9b8a47be22dc6c`.

Harness,
 saved during the investigation as `${HOME}/temp/agent/chai-truncate-probe.mjs`
 and run from the repository root:

```bash
# node chai-truncate-probe.mjs
cd -- /var/home/user/Monochromatic && node "${HOME}/temp/agent/chai-truncate-probe.mjs"
```

```js
// chai-truncate-probe.mjs
const chai = await import(
  `${process.cwd()}/node_modules/.pnpm/chai@6.3.0/node_modules/chai/index.js`
);
const { expect, config } = chai;

function failMessage(run) {
  try { run(); return '<passed>'; } catch (error) { return error.message; }
}
function withThreshold(threshold, run) {
  const previous = config.truncateThreshold;
  config.truncateThreshold = threshold;
  try { return run(); } finally { config.truncateThreshold = previous; }
}

const findings31 = Array.from({ length: 31 }, (_, i) => ({
  file: `src/file-${i}.ts`, line: i + 1, message: `unused identifier number ${i}`,
}));
const nums31 = Array.from({ length: 31 }, (_, i) => i);
const record12 = Object.fromEntries(
  Array.from({ length: 12 }, (_, i) => [`key${i}`, i]));

// catalog A: failing shapes at the default threshold
for (const [label, run] of [
  ['A1 findings31 vs []', () => expect(findings31).to.deep.equal([])],
  ['A2 findings31 vs findings31 with item 15 changed', () => expect(findings31)
    .to.deep.equal(findings31.map((f, i) => (i === 15 ? { ...f, line: 999 } : f)))],
  ['A3 nums31 vs []', () => expect(nums31).to.deep.equal([])],
  ['A4 long strings', () => expect('x'.repeat(120)).to.deep.equal('y'.repeat(120))],
  ['A5 record12 vs {}', () => expect(record12).to.deep.equal({})],
  ['A6 nested depth 5', () => expect({ a: { b: { c: { d: { e: 'deep-value' } } } } })
    .to.deep.equal({ a: { b: { c: { d: { e: 'other' } } } } })],
]) {
  const message = failMessage(run);
  console.log(`${label} | len=${message.length} | ${message}`);
}

// catalog B: truncation disabled three ways
for (const threshold of [0, Infinity, Number.MAX_SAFE_INTEGER]) {
  const message = withThreshold(threshold, () =>
    failMessage(() => expect(findings31).to.deep.equal([])));
  console.log(`B threshold=${threshold} | len=${message.length}`);
  console.log(`  head: ${message.slice(0, 80)}...`);
}

// catalog C: the finite-threshold cliff
for (const threshold of [30, 34, 35, 39, 40, 41]) {
  const message = withThreshold(threshold, () =>
    failMessage(() => expect(nums31).to.deep.equal([])));
  console.log(`C threshold=${threshold} | len=${message.length} | ${message}`);
}

// catalog D: getter walk depth (40 children, one getter each)
globalThis.__probeGetterCount = 0;
function buildWide() {
  return Object.fromEntries(Array.from({ length: 40 }, (_, i) => [
    `k${i}`, { get trap() { globalThis.__probeGetterCount += 1; return i; } },
  ]));
}
for (const threshold of [40, 0]) {
  globalThis.__probeGetterCount = 0;
  withThreshold(threshold, () => failMessage(() => expect(buildWide()).to.deep.equal({})));
  console.log(`D threshold=${threshold} | getters read=${globalThis.__probeGetterCount}`);
}

// catalog E: output size at scale
const big = Array.from({ length: 200_000 }, (_, i) => i);
for (const threshold of [40, 0]) {
  const startedAt = Date.now();
  const message = withThreshold(threshold, () =>
    failMessage(() => expect(big).to.deep.equal([])));
  console.log(`E threshold=${threshold} | len=${message.length} | elapsedMs=${Date.now() - startedAt}`);
}
```

### Patterns that fail at the default

The `Symptom` section's shapes are the failing catalog,
 reproduced by the harness's `A` cases.
 Every one is a failure message that omits the difference.

### Patterns that work cleanly

Disabling truncation (any falsy threshold,
 `Infinity`,
 or `Number.MAX_SAFE_INTEGER`,
 all measured identical) renders both sides whole
 (the 31-finding list reaches message length `2419`,
 every finding readable):

```text
B threshold=0 | len=2419
  head: expected [ { file: 'src/file-0.ts', line: 1, message: 'unused identifier number ...
B threshold=Infinity | len=2419
  head: expected [ { file: 'src/file-0.ts', line: 1, message: 'unused identifier number ...
B threshold=9007199254740991 | len=2419
  head: expected [ { file: 'src/file-0.ts', line: 1, message: 'unused identifier number ...
```

The monotone part of the budget path also works as documented:
 thresholds below the full render length grow the visible prefix smoothly
 (harness `C` case at threshold `30`,
 `35`,
 `39`,
 `41`).

### Side effects and scale, measured

- Getter and proxy walks deepen with the budget:
  40 child objects each holding one getter read `2` getters at threshold `40`
  and `40` at threshold `0`.
  The failure message is built only when the assertion fails,
  so passing tests pay nothing.
- Message size is unbounded when truncation is off:
  `200 000` numbers against `[]` produce a `1 488 921` character message
  (vs `66` characters at threshold `40`),
  rendered in `48` to `173 ms` across five runs on this host.
  The harness's `E` case records this.

## Verified workarounds

### Set the threshold to 0 at expect construction

`package/test-harness/core-expect` owns the single chai instance,
 so setting `chai.config.truncateThreshold = 0` beside the `use(...)` calls
 in `package/test-harness/core-expect/src/expect-matchers.ts:28`
 fixes every chai-backed matcher at once
 (harness `B` cases).
 Tradeoffs:
 failure messages grow without bound,
 so a pathological compare floods the log
 (measured `1.4 MB` for `200 000` numbers),
 and the render reads every getter and proxy trap in the compared values
 (measured `40` of `40` reads instead of `2` of `40`).
 There is still no diff,
 only two whole renders.

### Print the difference before comparing

`package/translation-repair/test-fixture/src/scan-findings.test-fixture.ts`
 (`expectNoFindings`) formats every finding one per line and then runs
 the original comparison;
 the translation-repair scans already moved onto it
 (issue 610's workaround section).
 Tradeoffs:
 it covers only the call sites that adopt it,
 duplicates value rendering outside the assertion library,
 and turns one comparison into two steps.

## What does not work

- Raising the threshold to a large finite value
 (probed `2 000`,
 `10 000`,
 `100`):
  output stays non-monotonic near the render length
 (root cause stage 3) and the tail,
  where differences usually sit,
 still elides (`…(N)`).
- `Number.MAX_SAFE_INTEGER` as a special sentinel:
  it works,
  but only because it is large;
  `0` is chai's documented disable and behaves identically
 (measured,
 all three produce the same `2419` character message).
- Overriding `chai.util.objDisplay` from a `chai.use` plugin:
  the property is getter-only
 (measured `TypeError: Cannot set property objDisplay of #<Object> which has only a getter`).
  Upstream issue 1766 asked for a replacement hook and was closed
  pointing at `Symbol('chai/inspect')` on the compared objects,
  which requires touching every value type under test.
- Reading `AssertionError.actual` / `.expected` in the harness printer:
  they exist but nothing in `test-harness/core-diagnostic` reads or prints them,
  so a consumer cannot recover the values from the log as shipped.

## Upstream filing decision

Out-of-scope check:
 `.out-of-scope/` holds no chai or test-display exemption
 (searched for `chai` and `truncate`).

Duplicate search on `chaijs/chai` (`gh search issues`,
 terms `truncateThreshold`,
 `objDisplay`,
 `"[ Array("`,
 "sanity limit" ):

- [#203](https://github.com/chaijs/chai/issues/203)
 "Sanity limit on string length of actual & expected in the error message?"
 is where the threshold behaviour was introduced,
 closed as completed in `1.9.1`.
- [#1766](https://github.com/chaijs/chai/issues/1766)
 "Request:
  allow custom `objDisplay` implementation" is closed,
 directing reporters to per-object custom inspectors.
- [#1728](https://github.com/chaijs/chai/issues/1728)
 "`truncateThreshold: 0` prints `process` and triggers DeprecationWarning"
 is open and adjacent (the disable path),
 but reports a deprecation warning from deep property reads,
 not the replacement decision.
- [#1352](https://github.com/chaijs/chai/issues/1352)
 "Allow configuring inspection depth" is open and orthogonal.

No issue reports the non-monotonic replacement boundary;
 this would be a new issue,
 not a comment.

### The 6-constraint check

1. Is it really upstream's fault?
   Yes.
   The decision reads the elided render's length against the render budget
 (`lib/chai/utils/objDisplay.js:27` quoted in root cause stage 2),
 which is upstream's code and config contract.
2. Can upstream fix it?
   Yes.
   The prototype below restores the documented contract in two files
 and keeps the whole suite green.
3. Are they supporting this use case?
   Yes.
   `config.truncateThreshold` is documented public config
 (`lib/chai/config.js:31`),
 `test/configuration.js:660` pins its display contract,
 and issue 203 shows the feature exists for readable failure messages.
4. Would the repo welcome our contribution?
   Likely.
   `CONTRIBUTING.md` invites issues and pull requests and states no AI
 restriction (searched `CONTRIBUTING.md` for AI,
 LLM,
 generated;
 no ban found,
 no issue or PR template in `.github/`),
 and maintainers answered issues 1728 and 1766 directly in 2025.
   The kept draft discloses agent assistance as this repository requires.
5. Will they likely fix it?
   Plausible.
   No documented won't-fix,
   recent commits in the path
 (clone HEAD `78d9637`,
 2026-10-19),
   and the failing expectations are test-pinned artifacts of the bug.
6. Have we prototyped a minimal fix compatible with their architecture?
   Yes.
   Disposable clone at commit `78d9637`,
   origin `https://github.com/chaijs/chai.git` verified,
   push disabled.
   `npm install`,
   `npm run build`,
   baseline `512 passing`,
   patch applied,
   `npm run build`,
   reproduction and suite re-run.

Verification output,
 pre-patch and post-patch,
same script
 (`node repro.mjs` in the prototype directory):

```text
# pre-patch (chai at 78d9637)
threshold=34 | expected [ Array(31) ] to deeply equal []
threshold=35 | expected [ +0, 1, 2, 3, 4, 5, 6, 7, …(23) ] to deeply equal []
threshold=40 | expected [ Array(31) ] to deeply equal []
threshold=41 | expected [ +0, 1, 2, 3, 4, 5, 6, 7, 8, 9, …(21) ] to deeply equal []
# documented contract (test/configuration.js cases) both still pass
threshold=20 | expected { Object (v) } to deeply equal { v: 'x' }
threshold=0  | expected { v: 'something longer than 20' } to deeply equal { v: 'x' }

# post-patch
threshold=20 | expected [ Array(31) ] to deeply equal []
threshold=34 | expected [ Array(31) ] to deeply equal []
threshold=35 | expected [ Array(31) ] to deeply equal []
threshold=40 | expected [ Array(31) ] to deeply equal []
threshold=41 | expected [ Array(31) ] to deeply equal []
threshold=145 | expected [ +0, 1, 2, ..., 30 ] to deeply equal []
threshold=20 | expected { Object (v) } to deeply equal { v: 'x' }
threshold=0  | expected { v: 'something longer than 20' } to deeply equal { v: 'x' }

# suite after patch
512 passing
```

The fix makes the replacement decision read the untruncated length
 and leaves the budgeted render as the output,
 so collapse happens exactly when the value does not fit,
 monotonically.
 It keeps `test/configuration.js:660`'s threshold-20 contract.
 Nine pinned expectations in `test/assert.js`,
 `test/expect.js`,
and `test/globalErr.js` that asserted the leaky partial
 renders were updated to the documented shape summaries
 (for example `expected [ Array(4) ] to equal [ Array(4) ]` where they
 pinned `expected [ 'string1', 'string2', …(2) ] to equal [ 'string5', 'string6', …(2) ]`).
 The complete diff:
 [chai-failure-value-truncation.patch](chai-failure-value-truncation.patch).

### Upstream filing artifact

Draft issue for `chaijs/chai`.
 All six constraints hold,
 so the draft below is technically fileable;
 the repository owner decided on 2026-10-20 to keep it local and not post
 to `chaijs/chai`.
 Treat posting as closed unless the owner reopens it.

~~~md
Title: objDisplay's shape-summary replacement is non-monotonic in truncateThreshold

`config.truncateThreshold` serves two roles in `lib/chai/utils/objDisplay.js`:
the loupe render budget (`lib/chai/utils/inspect.js:28`) and the
replacement boundary (`objDisplay.js:27`). The boundary reads the length of
the already-budgeted render, so loupe's `…(n)` elision arithmetic decides
whether a value collapses to `[ Array(n) ]` / `{ Object (keys) }`.

Measured on 6.3.0 and on 78d9637, one 31-number array (full render 145 chars):

    threshold=30 | expected [ +0, 1, 2, 3, 4, 5, …(25) ] to deeply equal []
    threshold=34 | expected [ Array(31) ] to deeply equal []
    threshold=35 | expected [ +0, 1, 2, 3, 4, 5, 6, 7, …(23) ] to deeply equal []
    threshold=40 | expected [ Array(31) ] to deeply equal []
    threshold=41 | expected [ +0, 1, 2, 3, 4, 5, 6, 7, 8, 9, …(21) ] to deeply equal []

Raising the threshold can remove content that a lower threshold showed, and
users tuning the knob for "more detail" cannot predict which side of the
collapse they land on. `test/configuration.js`'s threshold-20 case passes only
because `str.length >= 20` happens to hold there.

Suggested fix: decide replacement on the untruncated render length
(compute `inspect(obj, undefined, undefined, undefined, Infinity)` and compare
that against `config.truncateThreshold`), keeping the budgeted render as the
return value. A prototype against 78d9637 does this in
`lib/chai/utils/inspect.js` (optional `truncate` argument) and
`lib/chai/utils/objDisplay.js`; it keeps `test/configuration.js:660` green and
makes the sweep above collapse at 20..41 and show the full listing at 145+.
Nine expectations in `test/assert.js`, `test/expect.js`, and
`test/globalErr.js` pinned the leaky partial renders and were updated to the
documented shape summaries.

Reproduction and full diff are recorded in our
doc/troubleshooting/chai-failure-value-truncation.md; I can send the diff as a
PR. This report and the prototype were produced with agent assistance and
verified by a human-run reproduction script.
~~~

## Next action

The module-test-side fix choice for issue 610 is recorded as a proposal:
 [doc/planning/module-test-failure-value-diff.md](../planning/module-test-failure-value-diff.md).
 The whole-value render options were rejected after a sizing correction
 (real findings are `174`-character sentences,
 so any cap hides the differing entry),
 leaving a difference-first structural diff in `package/test-harness/core-diagnostic`.
 The upstream draft stays local per the 2026-10-20 owner decision.
