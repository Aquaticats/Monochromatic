# Test harness all-equal matcher

## Status

Accepted,
 2026-06-11.
 Tracking issue:
 [#251][issue].

## Context

The `find*Root` finders in `package/fs-path/core`
(`findMiseMonorepoRoot`,
 `findGitRepoRoot`,
 `findPnpmWorkspaceRoot`,
their `*Cached` variants,
 and the sibling `find*PackageRoot` family)
have tests asserting several values resolve to one directory.
These were written as transitive `toBe` chains:

```ts
expect(miseRoot,).toBe(gitRoot,);
expect(gitRoot,).toBe(pnpmRoot,);
```

The chain is correct (`toBe` is `===`,
 which is transitive) but noisy to read
and fiddly to extend.
A sweep of all 466 `*.test.ts` files found the pattern in three files,
concentrated in the two `fs-path` root-finder tests plus one incidental case in
`package/logger/core/src/sinks/file.unit.test.ts`.

Neither JavaScript nor the harness had an "all equal" primitive.
`===`/`Object.is` and `node:assert` are binary only.
The harness `MatcherSet` (`@monochromatic-dev/test-harness-core`) was entirely binary.

## Decision

Add three array-actual matchers to `@monochromatic-dev/test-harness-core`,
implemented as a chai plugin in `src/expect-matchers-collection.ts`:

- `toAllBe()`:
   every element strictly equals the first (mirrors `toBe`).
- `toAllEqual()`:
   every element deep-equals the first (mirrors `toEqual`).
- `toSatisfyAll(predicate)`:
   predicate holds for every element.

Each anchors on the first element;
 since strict and deep equality are both
transitive,
 "every element equals the first" is exactly "all mutually equal".
The plugin uses chai's `this.assert`,
 so `.not` negates correctly and failures
carry chai's value diff.
`toAllBe`/`toAllEqual` throw on a non-array actual or fewer than two values;
`toSatisfyAll` passes vacuously on an empty array,
 mirroring `Array.prototype.every`.

This was the `A + B`,
 strict-and-deep option from the issue's flavor question.

## Alternatives considered

No third-party dependency was adoptable:

- `chai-each` (`jamesthomasonjr/chai-each`):
   rejected.
  Stuck at npm v0.0.1,
   last commit 2018,
   two open bugs untouched for 5-6 years
  (one a core correctness defect,
   "only the last array element is being checked").
  It monkeypatches `chai.Assertion.prototype` globally,
   and our `expect` returns a
  fixed `MatcherSet` facade that never exposes chai's Assertion,
   so its `.each`
  would be unreachable without wrapper plumbing anyway.
- `jest-extended` (`toSatisfyAll`):
   rejected as a dependency.
  Actively maintained,
   but registers matchers through Jest's `expect.extend`,
  which our chai-based hand-rolled `expect` has no equivalent of.
  Its `toSatisfyAll` semantics (predicate over every element,
   vacuous-true on
  empty via `every`) were adopted by reimplementation,
   not by dependency.
- RSpec `all`,
   Java Hamcrest `everyItem(equalTo)`,
   PyHamcrest `only_contains`,
  python-precisely `all_elements`,
   AssertJ `containsOnly`/`allSatisfy`:
  cross-language,
   usable only as reference flavors.
  All express "all equal" the same way (apply an `equals(anchor)` matcher to every
  element),
   which is the shape adopted here.

Flavor and placement alternatives:

- Free helper `expectAllToBe([...])` in the `fs-path` tests:
   lighter,
   but the
  decision chose harness-wide matchers for discoverability across packages.
- Composable `.each.toBe(first)` (RSpec/chai-each flavor):
   rejected.
  The `MatcherSet` is flat,
   not composable;
   exposing `.each` is a facade rewrite.
- Inline `expect(new Set([...]).size).toBe(1)` or the anchor loop:
   zero new API,
  but the `Set` form has worse diagnostics ("expected 2 to equal 1") and is
  strict-only.

## Consequences

- `find-monorepo-root` and `find-package-root` agreement,
   sequential,
   and
  concurrent tests now use `expect([...]).toAllBe()`.
- New matcher names are registered in `MATCHER_KEYS`,
   so `.not`,
   `.rejects`,
  `.resolves`,
   and the scoped assertion counter wrap them like any other matcher.
- The TypeScript augmentation reopens chai's ambient `Chai.Assertion` interface
  (declaration merging requires `namespace` + `interface`,
   scoped lint disables
  document why).

[issue]: https://github.com/Aquaticat/Monochromatic/issues/251
