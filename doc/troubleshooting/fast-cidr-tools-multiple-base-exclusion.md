# `fast-cidr-tools` 0.3.4 `exclude()` skips a covered base after mutating multiple base intervals

## Symptom

`fast-cidr-tools` 0.3.4 returns an allowed CIDR that is completely covered by the exclusion when one exclusion
covers multiple disjoint allowed intervals.

The published-package probe supplied:

```text
allowed:    10.0.0.0/30, 10.0.0.8/30
excluded:   10.0.0.0/28
expected:   []
actual:     ["10.0.0.8/30"]
```

Node's assertion rendered:

```text
AssertionError [ERR_ASSERTION]: fast-cidr-tools: one exclusion covers multiple disjoint allowed intervals
+ actual - expected

+ [
+   '10.0.0.8/30'
+ ]
- []
```

Single-base subtraction works.
The defect requires a changed base followed by another original base that the same exclusion should process.
The `sort` argument does not affect it.

## Root cause

The source under test is release commit
[`d37506e5fcacc7a04760bd9c1b8c924d877bbc39`][release-commit].
The clone used for source inspection was `~/temp/agent/fast-cidr-tools-0.3.4`.

### A covered base becomes an empty remainder list

`src/exclude.ts:31-40` returns no remainder when the exclusion contains the current base:

```ts
if (a_start === b_start && a_end === b_end) {
  return [];
}

if (a_start > b_start && a_end < b_end) {
  return [];
}
```

Both `/30` bases in the reproduction lie inside the excluded `/28`.
The first call to `excludeNets` therefore changes the first base to an empty remainder list.

### Mutation shifts the next base into the current index

`src/exclude.ts:103-124` first merges the base list,
 then mutates it while walking one exclusion:

```ts
const basenets = _basenets.length === 1
  ? [parse(_basenets[0])]
  : mergeToTuples(_basenets);

for (let i = 0, len = exclnets.length; i < len; i++) {
  const excl = exclnets[i];

  let index = 0;
  while (index < basenets.length) {
    const base = basenets[index];
    const remainders = excludeNets(base, excl);
    if (remainders.length !== 1 || remainders[0][0] !== base[0] || remainders[0][1] !== base[1]) {
      for (let j = 0, len = remainders.length; j < len; j++) {
        basenets.push(remainders[j]);
      }
      basenets.splice(index, 1);
    }
```

For the first base,
 `splice(0, 1)` removes that base.
The second original base moves from index one to index zero.

### The unconditional increment skips the shifted base

`src/exclude.ts:126` increments after both changed and unchanged cases:

```ts
index++;
```

After the splice,
 `index` becomes one while the unprocessed second base is now at zero.
The loop ends because the array length is one.
The second base reaches output unchanged.

An earlier possibility was that opt-in sorting caused the wrong result.
That reading is false.
Sorting runs at `src/exclude.ts:130-132`,
 after subtraction,
 and cannot restore an interval that the loop should
have removed.

## Verification

### Version and artifact

Verified on 2026-07-28 against:

- npm `fast-cidr-tools` 0.3.4;
- integrity
  `sha512-WQNW+ynysAsI+O3YX2269Ff1wx6+xTyKrtLPN0TaZOf5ZZfFNPS59J0vmCrJbpno5z3vJ5sX4wUHpJL7avuHLg==`;
- release commit `d37506e5fcacc7a04760bd9c1b8c924d877bbc39`;
- Node 24.18.0 on Linux x86-64 in a network-disabled container.

The complete failing transcript is
`~/temp/agent/wg-cidr-validation/logs/proc_11-stderr.log`.
The upstream suite and byte-identical local build passed before the consumer probe.

### Runnable published-package harness

```js
// reproduce.mjs
import assert from 'node:assert/strict';
import { exclude } from 'fast-cidr-tools';

const actual = exclude(
  ['10.0.0.0/30', '10.0.0.8/30'],
  ['10.0.0.0/28'],
  true,
);

assert.deepEqual(actual, []);
```

```console
$ npm install --ignore-scripts --save-exact fast-cidr-tools@0.3.4
$ node reproduce.mjs
AssertionError [ERR_ASSERTION]
```

### Patterns that work cleanly

The full upstream suite passed these one-base shapes:

- `exclude(['1.0.0.0/23'], ['1.0.1.0/24'])` returns `['1.0.0.0/24']`;
- `exclude(['::/127'], ['::1/128'])` returns `['::/128']`;
- `exclude(['1.0.0.0/24'], ['1.0.0.0/16'])` returns `[]`.

These are recorded at `test/index.test.ts:47-58` and in
`~/temp/agent/wg-cidr-validation/logs/proc_15-stdout.log`.

### Pattern that fails

The confirmed failing shape is:

- at least two disjoint base intervals;
- one exclusion containing the current base and a later base;
- the current base changes,
   so `splice` shifts the later base left;
- the unconditional increment skips that shifted base.

The reproduction uses two IPv4 `/30` bases covered by one IPv4 `/28` exclusion.

### Prototype result

The minimal prototype moves `index++` into the unchanged branch.
Changed entries are removed without advancing,
 so the shifted original base is processed next.
The prototype also adds the missing regression vector.

The patch is [`fast-cidr-tools-multiple-base-exclusion.patch`](fast-cidr-tools-multiple-base-exclusion.patch).
It was applied in a fresh disposable clone at release commit
`d37506e5fcacc7a04760bd9c1b8c924d877bbc39`.

A first sandbox attempt built successfully but mounted `node_modules` at `/deps`.
Mocha then failed to resolve `@swc-node/register` from its real package path.
Mounting the inspected dependency tree at `/work/node_modules` fixed that harness-only resolution error.

The corrected network-disabled run:

- rebuilt ESM,
   CommonJS,
   and declarations;
- passed all seven upstream test groups,
   including the new regression assertion;
- passed a separate built-artifact assertion for the failing vector;
- exited zero in one second with `prototype consumer passed`.

The transcript is `~/temp/agent/wg-cidr-validation/logs/proc_24-stdout.log`.

## Verified workarounds

### Keep `cidr-tools`

The current `wg-allowedips` plan uses `cidr-tools` 12.1.3.
The same five-vector consumer harness,
 including this reproduction,
 passed against its published artifact.

Tradeoff:
 `cidr-tools` has one runtime dependency and deliberately rudimentary parsing,
 so the planned `node:net` address
check and prefix bounds remain necessary.
This workaround avoids project-owned set arithmetic and the defect entirely.

### Carry the minimal patch in a fork

Apply
[`fast-cidr-tools-multiple-base-exclusion.patch`](fast-cidr-tools-multiple-base-exclusion.patch)
and consume a forked release.
The prototype proves the patch against the upstream suite and failing built-artifact assertion.

Tradeoff:
 the project becomes responsible for publishing,
 provenance,
 dependency updates,
 and rebasing until upstream
ships the fix.
The package still has a larger runtime graph and four upstream lint warnings.

## What does not work

- Passing `sort: true` does not work.
  Sorting runs after subtraction and only reorders the wrong set.
- Pre-merging the base list does not work.
  `exclude()` already calls `mergeToTuples(_basenets)` before the faulty loop,
   and the reproduction still fails.
- Reordering the two bases is not a correctness fix.
  It changes which original interval is skipped but preserves the mutation-plus-increment failure mode.
- Stricter address or prefix validation does not work.
  Every reproduction input is a valid aligned CIDR;
   the defect occurs after parsing.
- Relying on the upstream suite alone does not work.
  Version 0.3.4's seven test groups pass,
   but its exclusion catalog has no exclusion spanning multiple disjoint bases.

## Upstream filing artifact

### Duplicate and policy checks

No matching open or closed issue or pull request was found with these literal repository searches:

- `exclude multiple base networks subtraction`;
- `exclude bug`;
- `subtraction bug`;
- `multiple CIDR exclude`;
- `basenets splice`.

The repository has no `CONTRIBUTING.md`,
 issue template,
 pull request template,
 code of conduct,
 security policy,
or AI-assistance policy.
No matching exemption exists under this repository's `.out-of-scope/` directory.
External pull requests [7][pr-7] and [8][pr-8] received maintainer review,
 were merged,
 and reached release 0.3.4.

### Upstream filing decision

1.  **Is it really upstream's fault?
    ** Yes.
    The published `exclude()` loop skips an unprocessed base after its own splice.
    The reproduction uses the documented array interface and valid CIDRs.
2.  **Can upstream fix it?
    ** Yes.
    Moving the increment to the unchanged branch fixes the mutation invariant without changing the API.
3.  **Are they supporting this use case?
    ** Yes.
    The README advertises `exclude()` at `README.md:21`,
     and `test/index.test.ts:47-58` treats array subtraction as a
    supported operation.
4.  **Would the repo welcome our contribution?
    ** Yes.
    No contribution or AI policy bars it,
     and comparable external correctness pull requests 7 and 8 were reviewed
    and merged.
5.  **Will they likely fix it?
    ** Yes with moderate confidence.
    No duplicate or won't-fix signal exists,
     and the maintainer released the prior external correctness fixes.
6.  **Have we prototyped a minimal compatible fix?
    ** Yes.
    The linked two-hunk patch passed the upstream suite and a separate built-artifact regression assertion.

All filing constraints pass.
This report keeps a fileable draft but does not send it without separate authorization for external communication.

### Draft issue

~~~md
Title: `exclude()` skips the next base interval after removing a changed interval

Labels: `bug`

## Description

`fast-cidr-tools` 0.3.4 can leave an excluded CIDR in its result when one
exclusion covers multiple disjoint base intervals.

## Reproduction

```js
import assert from 'node:assert/strict';
import { exclude } from 'fast-cidr-tools';

const actual = exclude(
  ['10.0.0.0/30', '10.0.0.8/30'],
  ['10.0.0.0/28'],
  true,
);

assert.deepEqual(actual, []);
```

Actual output:

```text
['10.0.0.8/30']
```

Expected output:

```text
[]
```

## Root cause

In `src/exclude.ts:111-127`, a changed base appends its remainders and is
removed with `basenets.splice(index, 1)`. The next original base shifts into
that index. The unconditional `index++` then skips it.

## Suggested fix

Advance `index` only when `excludeNets` leaves the current base unchanged.
When the current base changes, keep the index so the shifted original base is
processed next.

Add the reproduction to the existing `exclude` test group. This two-hunk
change passes all seven upstream test groups and a separate assertion against
the rebuilt ESM artifact.
~~~

[pr-7]: https://github.com/SukkaW/fast-cidr-tools/pull/7
[pr-8]: https://github.com/SukkaW/fast-cidr-tools/pull/8
[release-commit]: https://github.com/SukkaW/fast-cidr-tools/commit/d37506e5fcacc7a04760bd9c1b8c924d877bbc39
