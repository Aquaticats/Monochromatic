# Oxlint 1.65 native readonly rule misclassifies nested branded primitives

## Symptom

Oxlint's former native `typescript/prefer-readonly-parameter-types` rule reported an options object containing a
branded primitive:

```typescript
import { cssRem, type CssValue, } from '@monochromatic-dev/hyperscript-core/ts';

export function focusOutline(
  { offset = cssRem(OUTLINE_WIDTH,), }: { readonly offset?: CssValue; } = {},
): CssDeclarations {
  // Implementation omitted.
}
```

`CssValue` is a branded primitive:

```typescript
export type CssValue = string & { readonly __cssValue: unique symbol; };
```

A bare `CssValue` parameter passed,
but the same type nested in an otherwise readonly object failed.
Adding `CssValue` to the native rule's `allow` option did not change the result.

## Root cause

The native tsgolint implementation called its branded-literal exemption only for the complete parameter type.
It did not call that exemption during recursive property classification.

In tsgolint commit `78f9a83`,
`internal/rules/prefer_readonly_parameter_types/prefer_readonly_parameter_types.go` first called
`isTypeReadonly` for the parameter and then called `isTypeBrandedLiteralLike` for that same top-level type.
For `focusOutline`,
the top-level type is the options object,
not `CssValue`.

The recursive checker then reached the structural intersection behind `CssValue`.
Its `allow` matcher required an alias or symbol name before checking package or file provenance,
but the nested structural intersection no longer carried the `CssValue` alias.
The `allow` entry therefore could not match.

The corresponding typescript-eslint implementation at commit `f891c29` had the same division:

- `package/eslint-plugin/src/rules/prefer-readonly-parameter-types.ts` applied the brand exemption to the parameter;
- `package/type-utils/src/isTypeReadonly.ts` recursively classified properties without a branded-primitive check.

This behavior belongs to the retired native rule.
The repository-owned semantic rule does not use its type-name allowlist or its top-level-only branded exemption.

## Verified resolution

`prefer-readonly-parameter-type/prefer-readonly-parameter-types` classifies unions and intersections by their semantic
constituents.
Primitive constituents are readonly.
A type-only brand object is readonly when its brand property is readonly and reaches no mutable runtime state.
The enclosing options object is then checked normally,
including its own property modifiers and every other reachable constituent.

No site-specific exception is needed.
The authored API remains:

```typescript
export function focusOutline(
  { offset = cssRem(OUTLINE_WIDTH,), }: { readonly offset?: CssValue; } = {},
): CssDeclarations {
  // Implementation omitted.
}
```

The replacement rule also keeps the important negative case:
an object with any writable or mutable reachable property still reports even when another property is a branded
primitive.

## Verification

The original native-rule probe established:

- bare `CssValue` passed;
- optional,
  required,
  and mapped-readonly nested `CssValue` properties reported;
- a plain readonly `string` property passed;
- adding a name-only or package-qualified `CssValue` allow entry did not help.

On 2026-07-13,
the current package command was rerun:

```sh
mise run //package/done/app:lint:oxlint
```

The command returned status `1` with 35 replacement-rule findings elsewhere in the package.
There was no finding containing `focusOutline` or `CssValue`.
This verifies that the nested branded options parameter no longer needs special handling while unrelated mutable and
uncertain inputs remain enforced.

Focused semantic-rule verification uses:

```sh
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:lint:types
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:lint:oxlint
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:test:unit
```

The type task,
Oxlint task,
and complete unit suite pass after the callback-effect assertion update.

## What does not work

### Add `CssValue` to a type-name allowlist

The native recursive type lost the alias before matching.
More importantly,
the replacement rule intentionally has no type-name escape hatch because a safe use depends on the complete reachable
shape and effect path.

### Enable `treatMethodsAsReadonly`

That broad policy can hide real state changes through maps,
sets,
and capability objects.
It does not express why this specific branded primitive is safe.

### Repeat an ownership marker on the nested property

`ForeignBorrowed` records foreign ownership,
not branded immutability.
Applying it here would misstate provenance and could hide an owned mutable sibling.

### Change the public function to a positional parameter

A positional `CssValue` happened to avoid the native bug,
but changing an established options-object API for a linter implementation detail is not justified.
The semantic replacement handles the existing API directly.

## Upstream filing artifact

### Upstream filing decision

1. **Is it really upstream's fault?**
    The native behavior followed its documented deep-readonly model and a deliberately
   top-level branded-literal exemption.
2. **Can upstream fix it?**
    A recursive branded exemption would change the old rule's semantic core and intersection
   handling.
3. **Are they supporting this use case?**
    The native rule did not document recursive branded-primitive treatment.
4. **Would the repo welcome our contribution?**
    Maintainer discussion on typescript-eslint issue `#1790` declined the
   general recursive intersection case while accepting a narrower common-case implementation.
5. **Will they likely fix it?**
    The top-level-only tests and later alias-preservation work did not extend the exemption
   to nested property values.
6. **Have we prototyped a minimal fix compatible with their architecture?**
    The repository-owned semantic classifier is
   the implemented and verified replacement.

Nothing should be filed upstream.
The repository no longer consumes the native rule behavior,
and the replacement resolves the user-facing case without weakening unrelated checks.

## Source audit boundary

The investigation read the deciding tsgolint and typescript-eslint rule and readonly-classifier sources,
not only CLI output.
A repository search found the related authoring-identity and ESTree troubleshooting documents;
neither duplicates the nested branded-primitive mechanism.
A root `.out-of-scope` search for readonly,
Oxlint,
TSDoc,
and foreign-ownership topics found no applicable entry.
