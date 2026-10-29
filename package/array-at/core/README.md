# module-array-at

Private pending publication clearance.

`@monochromatic-dev/array-at-core` is a throwing array accessor with:

- exact tuple-element return types;
- positive and negative indices;
- safe-integer proof helpers for dynamic indices;
- matching static and runtime diagnostics;
- runtime distinction between sparse slots and explicitly stored `undefined`.

## Publication gate

This package is intentionally configured with `"private": true`.
Do not remove that field or publish the package until the repository owner explicitly clears publication.
The version,
 export map,
 license files,
 repository metadata,
 and `publishConfig` are prepared for a future public release.

## Usage

```ts
import {
  arrayAt,
  ArrayAtError,
  asSafeInteger,
} from '@monochromatic-dev/array-at-core';

const last = arrayAt({
  array: [10, 20, 30],
  index: -1,
});
// `last` has type 30.

declare const indexFromJson: number;
const selected = arrayAt({
  array: [10, 20, 30],
  index: asSafeInteger(indexFromJson),
});
// `selected` has type 10 | 20 | 30.

try {
  arrayAt({ array: [10], index: asSafeInteger(2), });
}
catch (error) {
  if (error instanceof ArrayAtError) {
    const codes = error.diagnostics.map(({ code, }) => code);
    console.error(codes);
  }
}
```

The correlated `{ array, index }` parameter preserves valid relationships in union inputs.
A separate array parameter and index parameter would lose that correlation during generic inference.

## Exports

### `arrayAt`

```ts
function arrayAt<const Argument extends ArrayAtArgument>(
  argument: Argument & ValidateArrayAtArgument<Argument>,
): ArrayAtResult<Argument>;
```

`arrayAt` returns an assigned in-range element.
It never returns `undefined` to represent failed access.
Failure throws `ArrayAtError`.
An explicitly stored `undefined` remains a real value and can be returned at runtime.

Positive indices start at zero.
Negative indices count backward from array end:
`-1` selects the last element and `-array.length` selects the first.

### `SafeInteger` and proof helpers

A literal such as `2` proves its own safe-integer status to TypeScript.
A value typed as plain `number` might be fractional,
 infinite,
 `NaN`,
 or outside exact IEEE-754 integer representation.
Plain numbers must pass through one of these helpers:

- `isSafeInteger(value)` narrows within a branch.
- `assertSafeInteger(value)` narrows an existing binding or throws.
- `asSafeInteger(value)` returns the same number with the `SafeInteger` brand or throws.

The proof covers `Number.isSafeInteger` only.
It does not promise that the number is in range for a particular array.
`arrayAt` performs that dependent check.

### `ArrayAtError`

`ArrayAtError` extends `Error` and exposes:

- `diagnostics`,
   a frozen readonly array;
- `index`,
   the requested numeric index;
- optional `length`,
   present when validation included array context.

Diagnostic order is not part of the interface.
`Error.message` joins every diagnostic message with newlines,
 but line order is likewise unspecified.
Callers should branch on diagnostic `code`,
 not array position or message text.

### Diagnostic types

`ArrayAtDiagnostic` is the union of individual diagnostic shapes.
`ArrayAtDiagnostics<ArrayValue, Index>` computes the readonly unordered diagnostic collection visible to TypeScript.

Runtime and static diagnostics use these codes:

- `non-safe-integer`:
   runtime number or numeric literal is not a safe integer.
- `empty-array`:
   array has no element to return.
- `out-of-range`:
   safe integer lies before start or past end.
- `unassigned-slot`:
   resolved in-range slot has never been assigned.
- `unproven-safe-integer`:
   static-only diagnostic for plain `number`.

A range diagnostic includes:

- `direction`,
   either `before-start` or `past-end`;
- exact `distance` beyond nearest valid bound;
- positive and negative valid bounds;
- requested index and array length.

An unassigned-slot diagnostic also includes `resolvedIndex`.

## Aggregated diagnostics

Validation reports every independently actionable diagnostic whose prerequisites hold.
Safe-integer status and array emptiness are independent.
For example,
 an empty array combined with index `1.5` reports both `non-safe-integer` and `empty-array`.

Dependent checks do not create cascaded noise:

- range validation requires a safe integer and non-empty array;
- slot-assignment validation requires a safe,
   in-range index.

Every failing operation throws one `ArrayAtError` carrying the resulting diagnostic collection.

## Static and runtime seam

Static validation catches facts encoded by literal and tuple types.
Runtime validation remains authoritative for:

- arrays whose lengths are unknown to TypeScript;
- values arriving from JSON,
   storage,
   or arithmetic;
- tuples mutated after inference;
- actual sparse-array ownership.

TypeScript models both a sparse tuple slot and an explicitly written `undefined` as `undefined`.
Static validation therefore rejects both conservatively.
Runtime validation uses `Object.hasOwn` and rejects only a genuinely unassigned slot.
This is a deliberate static/runtime difference in the cautious direction.

## Type-level arithmetic

TypeScript does not provide subtraction over numeric literal types.
The implementation converts safe integer spellings into bounded decimal sequences and performs long subtraction.
This supports large literal indices without constructing a tuple whose length equals the index.

Recursion is bounded by safe-integer decimal width.
Single-digit subtraction uses tuples no longer than decimal base size.
Runtime subtraction occurs only after `Number.isSafeInteger` succeeds,
 keeping directional distances exact.

Diagnostic factory return annotations are the wording seam:
runtime validation calls those factories,
 while type-level validation reads their return types.
The compiler therefore prevents static and runtime message templates from drifting independently.

## Development

Run package verification through mise:

```sh
mise run //package/array-at/core:lint:types
mise run //package/array-at/core:lint:oxlint
mise run //package/array-at/core:buildAndTest
```

Unit tests import the built artifact from `dist/final/neutral/index.mjs`.
Type tests cover successful inference,
 invalid-call rejection,
 union correlation,
 aggregate diagnostics,
 and large-index arithmetic.
