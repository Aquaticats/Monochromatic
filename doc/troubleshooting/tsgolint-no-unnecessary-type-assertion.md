# tsgolint `no-unnecessary-type-assertion` false positive on generic-function calls whose type parameter appears in a union with `null`/`undefined`

oxlint 1.55.0 + oxlint-tsgolint 0.16.0 (typescript-go commit
`4a59cd78390d`).

## Status: fixed upstream in tsgolint v0.17.2 (PR #824) and refined in v0.17.3 (PR #826)

The workspace catalog floor is `oxlint-tsgolint >= 0.16.0` (`pnpm-workspace.yaml`),
but pnpm currently resolves it to `oxlint-tsgolint@0.19.0`,
 which already includes
both upstream commits.
 Existing installs do not exhibit the bug;
 the catalog
floor should be raised to `>= 0.17.3` opportunistically (separate change;
 not
part of this troubleshooting entry).

The remainder of this document preserves the root-cause analysis of the v0.16.0
behaviour for historical record and to keep the audit trail of the 5-constraint
upstream-filing check.

Empirical verification (this entry's prototype step):

- Fresh clone:
   `https://github.com/oxc-project/tsgolint.git` at
  `1dcd2a6f4138f4b9a273a231def838995ac589e5` (origin and HEAD captured before
  the build).
- Reproducer file (`repro.ts`):

  ```ts
  interface Element {
    tagName: string;
  }
  interface HTMLFormElement extends Element {
    action: string;
    method: string;
    elements: unknown;
  }
  declare const document: {
    querySelector<E extends Element = Element,>(selectors: string,): E | null;
  };
  function notNullishOrThrow<T,>(value: T | null | undefined,): T {
    if (value === null || value === undefined)
      throw new Error('nullish',);
    return value;
  }
  export const form = notNullishOrThrow(
    document.querySelector('.myForm',),
  ) as HTMLFormElement;
  ```

- Built `tsgolint` binary at `v0.16.0` and at HEAD with the standard
  `git submodule update --init`,
   `git am --3way --no-gpg-sign patches/*.patch`,
  collections copy,
   `go build ./cmd/tsgolint` sequence.
- `tsgolint --tsconfig tsconfig.json` at v0.16.0 reports
  `no-unnecessary-type-assertion`:
   "This assertion is unnecessary since it does
  not change the type of the expression.
  " on `as HTMLFormElement` (the
  false positive under audit).
- `tsgolint --tsconfig tsconfig.json` at HEAD does **not** report
  `no-unnecessary-type-assertion` on the same reproducer.
   The fix lands.

### Side effect at HEAD: `no-unsafe-type-assertion` fires instead

At HEAD the same reproducer produces a different diagnostic from a sibling rule:

```text
no-unsafe-type-assertion - Unsafe assertion from `any` detected: consider using
type guards or a safer assertion.
```

The `no-unsafe-type-assertion` rule uses `GetTypeAtLocation` on the inner
expression (`internal/rules/no_unsafe_type_assertion/no_unsafe_type_assertion.go:60`),
not the new context-free path.
 The rule existed in v0.16.0 and did **not** fire
on this reproducer there.
 The behaviour change between the two builds is not in
`no-unsafe-type-assertion` itself;
 the typescript-go submodule pinned by each
tsgolint version differs,
 and the inferred type of
`notNullishOrThrow(document.querySelector('.myForm'))` (without the contextual
hint that the AsExpression supplied) is reported as `any` at HEAD.
 The original
`no-unnecessary-type-assertion` bug is gone,
 but the wrapper-around-generic-call
shape is not in PR #824's regression test set.
 This side effect is out of scope
for this troubleshooting entry;
 documenting here so the next investigator does
not retread the same path.

## Symptom

tsgolint (the type-aware backend behind oxlint
`--type-aware`) flags `as T` assertions as unnecessary when
the asserted expression is a call to a generic function whose
parameter type includes a union with `null` or `undefined`
(e.g. `<T>(value: T | null | undefined): T`).

The diagnostic claims "This expression already has the type
'X'" where X is the asserted type,
 even though removing the
assertion changes the inferred type.

Minimal observation:

```ts
// notNullishOrThrow.ts
export function notNullishOrThrow<T,>(value: T | null | undefined,): T {
  if (value === null || value === undefined)
    throw new Error('nullish',);
  return value;
}

// repro.ts
import { notNullishOrThrow, } from './notNullishOrThrow.ts';

// oxlint reports: "This assertion is unnecessary since it does not change the type of the expression."
// oxlint claims: "This expression already has the type 'HTMLFormElement'"
// Actual type without assertion: Element
const form = notNullishOrThrow(
  document.querySelector('.myForm',),
) as HTMLFormElement;
```

Removing `as HTMLFormElement` changes the inferred type to
`Element`,
 causing TS2339 errors on `HTMLFormElement`-specific
properties (`.action`,
 `.method`,
 `.elements`).

## Root cause

The bug is in tsgolint's `no-unnecessary-type-assertion` rule
interacting with typescript-go's contextual typing for
`AsExpression` nodes.

### Call path

1.  The rule calls `getUncastType(node)` to get the
    expression's type **without** the assertion.
2.  `getUncastType` calls
    `ctx.TypeChecker.GetTypeAtLocation(expression)` on the
    inner call expression.
3.  `GetTypeAtLocation` -> `getTypeOfNode` ->
    `getRegularTypeOfExpression` -> `getTypeOfExpression` ->
    `checkExpressionEx`.
4.  `checkExpressionEx` -> `checkCallExpression` ->
    `getResolvedSignature`,
     which resolves generic type
    parameters.
5.  During resolution,
     TypeScript calls `getContextualType` on
    the call expression.
6.  `getContextualType` for the child of an `AsExpression`
    returns the **asserted type**
    (`internal/checker/checker.go:28318-28322`):

    ```go
    case ast.KindTypeAssertionExpression, ast.KindAsExpression:
        return c.getTypeFromTypeNode(parent.Type())
    ```

7.  This contextual type (`HTMLFormElement`) feeds into
    generic inference for `T`.
8.  With parameter type `T | null | undefined` and argument
    `Element | null`,
     the contextual return type
    `HTMLFormElement` causes `T` to resolve as `HTMLFormElement`
    instead of `Element`.
9.  The resolved signature is cached
    (`signatureLinks.resolvedSignature`,
    `internal/checker/checker.go:8162-8164`),
     so subsequent
    queries return the same `HTMLFormElement` result.
10. The rule sees `uncastType == castType` and flags the
    assertion as unnecessary.

### Why this is a paradox

The assertion provides the contextual type that changes the
expression's inferred type to match the asserted type.
Removing the assertion removes the contextual type,
 causing
the expression's type to revert.
 The assertion makes itself
appear unnecessary.

### Why it only triggers with union parameter types

- `wrap<T>(value: T): T` with arg `Element`:
   argument
  inference strongly fixes `T = Element`.
   Contextual
  `HTMLFormElement` cannot override it.
  `as HTMLFormElement` is correctly **not** flagged.
- `stripNull<T>(value: T | null): T` with arg `Element |
  null`:
   the union in the parameter type allows contextual
  typing to have more influence during inference.
  `as HTMLFormElement` **is** incorrectly flagged.

### Source locations

- **tsgolint rule**:
  `internal/rules/no_unnecessary_type_assertion/no_unnecessary_type_assertion.go`
  - `getUncastType` (line 232-249):
     uses
    `GetTypeAtLocation`,
     which includes contextual typing.
  - The IIFE special case (line 235-246) already works around
    a similar issue by using `getResolvedSignature` +
    `GetReturnTypeOfSignature` instead.
- **typescript-go contextual type**:
  `internal/checker/checker.go:28318-28322`:
   returns the
  asserted type as contextual type for children of
  `AsExpression`.
- **typescript-go signature caching**:
  `internal/checker/checker.go:8162-8164`:
  `getResolvedSignature` caches the resolved signature,
  preserving the contextually-influenced inference.

## Verification

Version under test:

- oxlint 1.55.0
- oxlint-tsgolint 0.16.0
- typescript-go commit `4a59cd78390d`

Reproduce with the snippet above against the workspace's
oxlint configuration.
 The diagnostic surfaces only when the
generic parameter is unioned with `null`/`undefined` and the
argument types resolve such that the contextual type
overrides the inferred type parameter.

## Verified workaround

> Applies only when pinned to oxlint-tsgolint v0.16.0 to v0.17.1.
>  Versions
> v0.17.2 and later include the upstream fix.

Use the generic type parameter on `querySelector` instead of
a post-hoc `as` assertion:

```ts
// Instead of:
const form = notNullishOrThrow(
  document.querySelector('.myForm',),
) as HTMLFormElement;

// Use:
const form = notNullishOrThrow(
  document.querySelector<HTMLFormElement>('.myForm',),
);
```

`querySelector<HTMLFormElement>` returns `HTMLFormElement |
null`,
 which makes `notNullishOrThrow` infer `T =
HTMLFormElement` and return `HTMLFormElement`.
 No assertion
is needed,
 so both tsgo and oxlint are satisfied.

Tradeoff:
 requires the wrapped helper to accept the generic
hint.
 For helpers like `notNullishOrThrow` this is already the
shape;
 for less-generic helpers this approach does not apply
and the assertion must remain (with an
`oxlint-disable-next-line` annotation explaining why).

## What does not work

- Removing the assertion:
   changes the type to `Element`
  (the actual inferred type),
   breaking downstream code that
  relies on `HTMLFormElement` methods.
- Adding `oxlint-disable-next-line` without explanation:
   the
  workspace requires justification on lint disables;
   the
  justification "tsgolint false positive on generic + nullable
  union" is acceptable.
- Trying to coerce inference via `<T = HTMLFormElement>`:
  TypeScript still uses the contextual type from the
  assertion;
   the explicit default does not bypass the
  contextual inference.

## Why we would have filed this upstream (audit trail; already fixed)

The 5-constraint check below is preserved for audit purposes.
 The fix is
already shipped upstream;
 do not file.

1.  **Is it really upstream's fault?
    ** Yes;
     `getUncastType`'s
    use of `GetTypeAtLocation` was the defect.
2.  **Can upstream fix it?
    ** Yes;
     the change ended up as a tightly-scoped
    conditional in `getUncastType` (single-hunk in PR #824,
     refined by one
    hunk in PR #826).
3.  **Are they supporting this use case?
    ** Yes;
     the rule is
    shipped and the case is in scope.
4.  **Will they likely fix it?
    ** Already fixed:
     PR #824 (commit `599e150`,
    first tagged in v0.17.2) introduced the context-free type path for
    call-like expressions;
     PR #826 (commit `f8a6ae2`,
     first tagged in v0.17.3)
    extended it to await-wrapped calls.
5.  **Have we prototyped a minimal fix?
    ** Verified upstream patch via
    approach 3 (see below).
     No local PR needed;
     the change was already merged
    and released by the time the prototype step ran.

Decision:
 do **not** file.
 The fix is already in the version the workspace is
actually consuming (v0.19.0 via the `>=0.16.0` catalog floor).

### Suggested fix for tsgolint: approach 3 verified upstream

The `getUncastType` function should compute the expression's
type **without** contextual typing from the parent
`AsExpression`.
 Three approaches were considered when the doc was first
written:

1.  **Extend the IIFE pattern to all generic calls**:
     use
    `getResolvedSignature` + `GetReturnTypeOfSignature` for any
    call expression,
     not just IIFEs.
     Avoids `GetTypeAtLocation`
    entirely for calls.
     Not pursued upstream.
2.  **Check for generic function calls**:
     if the callee has
    generic type parameters,
     skip the rule (the same approach
    typescript-eslint acknowledged in
    [typescript-eslint#528](https://github.com/typescript-eslint/typescript-eslint/issues/528)).
    Not pursued upstream.
3.  **Strip contextual type**:
     temporarily remove the
    `AsExpression` parent before calling `GetTypeAtLocation`,
    or use a checker API that excludes contextual typing.
     **Verified
    upstream choice.
    ** PR #824 introduced a call-like branch that consults
    `Checker_getContextFreeTypeOfExpression` before falling back to
    `GetTypeAtLocation`;
     PR #826 widened the branch to cover `await`-wrapped
    calls.
     The upstream PR titles framed the fix against
    [oxc-project/oxc#20656](https://github.com/oxc-project/oxc/issues/20656)
    (`querySelector` with a defaulted type parameter),
     not against this doc's
    union-with-null shape,
     but the same `getContextFreeTypeOfExpression` code
    path resolves both classes of false positive.

Combined upstream diff (PR #824 + PR #826) on
`internal/rules/no_unnecessary_type_assertion/no_unnecessary_type_assertion.go`:

```diff
@@ -228,6 +228,18 @@ var NoUnnecessaryTypeAssertionRule = rule.Rule{
 			callee := ast.SkipParentheses(expression.AsCallExpression().Expression)
 			return ast.IsArrowFunction(callee) || ast.IsFunctionExpression(callee)
 		}
+		var isContextSensitiveCallLikeExpression func(expression *ast.Node) bool
+		isContextSensitiveCallLikeExpression = func(expression *ast.Node) bool {
+			if ast.IsCallExpression(expression) || ast.IsNewExpression(expression) || ast.IsTaggedTemplateExpression(expression) {
+				return true
+			}
+
+			if ast.IsAwaitExpression(expression) {
+				return isContextSensitiveCallLikeExpression(ast.SkipParentheses(expression.Expression()))
+			}
+
+			return false
+		}

 		getUncastType := func(node *ast.Node) *checker.Type {
 			expression := ast.SkipParentheses(node.Expression())
@@ -245,6 +257,15 @@ var NoUnnecessaryTypeAssertionRule = rule.Rule{
 				}
 			}

+			// For call-like expressions, use the context-free expression type so
+			// contextual typing from the assertion itself doesn't leak into generic
+			// inference for the original expression.
+			if isContextSensitiveCallLikeExpression(expression) {
+				if t := checker.Checker_getContextFreeTypeOfExpression(ctx.TypeChecker, expression); t != nil {
+					return t
+				}
+			}
+
 			return ctx.TypeChecker.GetTypeAtLocation(expression)
 		}
```

Verification command (run against the fresh clone in `/tmp/tmp.zGzIFb5u6J/`):

```sh
# Pre-fix at v0.16.0
cd "$(mktemp -d)" && gh repo clone oxc-project/tsgolint && cd tsgolint
git checkout v0.16.0
git submodule update --init --recursive
(cd typescript-go && git am --3way --no-gpg-sign ../patches/*.patch)
mkdir -p internal/collections && find ./typescript-go/internal/collections -type f ! -name '*_test.go' -exec cp {} internal/collections/ \;
go build -o /tmp/tsgolint-v0.16.0 ./cmd/tsgolint
/tmp/tsgolint-v0.16.0 --tsconfig <repro-dir>/tsconfig.json   # reports no-unnecessary-type-assertion

# Post-fix at HEAD (1dcd2a6f4138f4b9a273a231def838995ac589e5 at time of writing)
git checkout main
# repeat submodule init, am, collections copy, then:
go build -o /tmp/tsgolint-HEAD ./cmd/tsgolint
/tmp/tsgolint-HEAD --tsconfig <repro-dir>/tsconfig.json      # no no-unnecessary-type-assertion
```

## Draft upstream issue (kept for audit; **do not file**)

````md
**STATUS: ALREADY FIXED UPSTREAM. DO NOT FILE.** The diagnosis below was
written against tsgolint v0.16.0. PR
[oxc-project/tsgolint#824](https://github.com/oxc-project/tsgolint/pull/824)
(merged 2026-03-23, released in v0.17.2) and PR
[oxc-project/tsgolint#826](https://github.com/oxc-project/tsgolint/pull/826)
(merged 2026-03-24, released in v0.17.3) implement approach 3 from the
"Suggested fix" subsection above. The draft is preserved so the audit trail
of the 5-constraint check remains reproducible.

**Title**: `no-unnecessary-type-assertion` false positive on generic function calls with nullable parameter types

**Labels**: bug, rule:no-unnecessary-type-assertion

**Description**:

`no-unnecessary-type-assertion` incorrectly flags `as` assertions on calls to generic functions whose generic parameter appears in a union with `null` or `undefined` in the function signature.

**Reproduction**:

```ts
function notNullishOrThrow<T,>(value: T | null | undefined,): T {
  if (value === null || value === undefined)
    throw new Error('nullish',);
  return value;
}

// Flagged: "This assertion is unnecessary since it does not change the type of the expression."
// Claims: "This expression already has the type 'HTMLFormElement'"
const form = notNullishOrThrow(
  document.querySelector('.myForm',),
) as HTMLFormElement;
```

**Expected**: no warning. Without the assertion, the expression type is `Element` (inferred from `querySelector`).

**Actual**: warns that the assertion is unnecessary and claims the expression already has type `HTMLFormElement`.

**Root cause**: `getUncastType` calls `GetTypeAtLocation(expression)`, which triggers `checkExpressionEx`. The contextual type for the child of an `AsExpression` is the asserted type itself (`internal/checker/checker.go:28318-28322`). This contextual type feeds into generic inference and changes the inferred type parameter, making `uncastType` equal to `castType`. The assertion provides the contextual type that makes itself appear unnecessary.

This is the same fundamental issue as
[typescript-eslint/typescript-eslint#528](https://github.com/typescript-eslint/typescript-eslint/issues/528).

**Suggested fix**: extend the IIFE pattern at `internal/rules/no_unnecessary_type_assertion/no_unnecessary_type_assertion.go:235-246` to all generic call expressions, or skip generic function calls entirely (per typescript-eslint's acknowledged limitation).

**Environment**: oxlint 1.55.0, tsgolint 0.16.0, typescript-go `4a59cd78390d`
````
