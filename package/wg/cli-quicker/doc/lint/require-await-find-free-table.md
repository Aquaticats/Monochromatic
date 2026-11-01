# `require-await` suppression on `findFreeTable`

`src/tunnel-route.ts` `findFreeTable` returns the promise of an inner async IIFE
(`scan`) without using `await` itself.
 The `eslint/require-await` rule reports the
outer function because it contains no `await` expression.

## Why the shape exists

`no-restricted-syntax/no-function-root-let` forbids `let candidate = BASE_TABLE`
at the function-body root.
 The sanctioned remediation (named-function IIFE or
extracted helper) puts the loop cursor inside `scan`,
 so `findFreeTable` only
delegates.
 Adding a redundant `await` to satisfy `require-await` would re-wrap the
same promise for no behavioral reason.

## Why config cannot resolve it

`require-await` is semantic,
 not configurable per-function;
 the only toggle is the
rule itself,
 and `no-restricted-syntax` forbids disabling whole rules.
 A scoped
block disable around just this delegation is the tightest possible suppression and
is the pattern LN6 requires (disable -> TSDoc -> declaration -> enable on the next
line after the declaration).

## Sources

- Rule behavior:
   `eslint/require-await` flags async functions lacking `await`.
- Forcing constraint:
   `package/oxlint-plugin/no-restricted-syntax/src/rule/` (`no-function-root-let`).
