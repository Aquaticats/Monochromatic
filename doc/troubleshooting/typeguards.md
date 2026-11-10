# TypeScript typeguards: `value: unknown` parameter loses caller's type information through narrowing; generic-extends pattern preserves it

## Symptom

A typeguard declared as `function isSchema(value: unknown):
value is Schema` compiles cleanly and runs correctly at
runtime,
 but inside the narrowed branch,
 the original
caller's type information is gone:

```ts
const richObject = {
  parse: x => x,
  weight: 100,
  metadata: { version: '1.0', },
  validator: customValidator,
  cache: new Map(),
};

if (isSchema(richObject as unknown,)) {
  richObject.weight; // Error: 'weight' does not exist on type 'Schema'
  richObject.metadata; // Error
  richObject.validator; // Error
  richObject.cache; // Error
}
```

The compiler narrows `richObject` to the predicate type
(`Schema`) and discards everything else.
 Callers must either
re-cast back to the original type,
 write a more specific
typeguard,
 or restructure the predicate to preserve the
input.

Additional surprises:

- The runtime body of `isSchema(value: unknown)` is
  permitted to accept any value (`null`,
   `undefined`,
  primitives,
   plain objects).
   Each of those compiles
  successfully but explodes at runtime if the body assumes
  object semantics.
   The signature does not catch this.
- `any` inputs are still narrowed to the predicate type:
  the "escape hatch" does not bypass the typeguard's effect.
- Intersection types (`A & B & C`) can lose parts during
  narrowing depending on the guard pattern.
- Casting (`value as unknown as T & typeof value`) can
  produce worse narrowing than direct usage.

## Root cause

TypeScript's narrowing rule for a typeguard `(x: P): x is
T` is:
 inside the `if (guard(x))` block,
 the type of `x`
becomes `T` (or `T & inputType` when the input type has
useful structure).
 When the input type is `unknown`,
 the
narrowed result is `T` alone,
 with no intersection.
 The
caller's richer type is discarded.

Concretely:

- `(x: SchemaWithWeight): x is Schema`:
   narrowing produces
  `SchemaWithWeight & Schema`,
   which preserves `weight`.
- `(x: unknown): x is Schema`:
   narrowing produces `Schema`
  alone;
   the caller's properties are gone.
- `(x: T): x is T` with `T extends Schema`:
   narrowing
  produces `T`,
   which preserves the caller's properties when
  `T` was the rich type.

The behaviour is correct by spec;
 the rule is "narrow to the
predicate,
 intersected with the parameter type".
 `unknown`
contributes nothing to the intersection.

## Verification

Version under test:
 TypeScript pinned in workspace
`tsconfig.json`.
 Reproduce with the 84 test scenarios in
`typeguard.behaviorTest.ts` (workspace path);
 the table below
summarises the dominant findings.

Behaviour by guard signature and caller's input type:

- `isSchema(value: unknown): value is Schema` applied to a
  typed value cast to `unknown`:
   loses caller's properties.
- `isSchema(value: Schema): value is Schema` applied to a
  typed value:
   preserves caller's properties through
  intersection.
- `isSchema<const T extends Schema = Schema>(value: T):
  value is T` applied to a typed value:
   preserves caller's
  properties exactly.
- All three guards accept runtime-unsafe inputs (null,
  undefined,
   primitives) at compile time when the parameter
  is `unknown`.

## Verified workaround: prefer the generic-extends pattern

```ts
function isSchema<const T extends Schema = Schema,>(
  value: T,
): value is T {
  // runtime validation
}

function isString<const T extends string = string,>(
  value: T,
): value is T {
  return typeof value === 'string';
}
```

Tradeoffs:

- Caller must pass an already-typed value.
   For untrusted
  data (e.g. `JSON.parse(...)`),
   the caller must perform an
  explicit cast:
   `isSchema(untrusted as unknown as Schema &
  typeof untrusted)`.
   The cast is louder than the implicit
  `unknown` form;
   that loudness is the point because it
  exposes the trust boundary in the source.
- The `const` generic parameter is required to preserve the
  caller's narrow type literal where applicable.
- The generic default (`= Schema`) makes the call site
  ergonomic when the caller does not have a more specific
  type available.

## Alternative workaround: simple typed parameter

When generics feel heavy:

```ts
function isSchema(value: Schema,): value is Schema {
  // runtime validation
}
```

Tradeoff:
 preserves properties for typed inputs through
intersection but still requires casting for `unknown` data.
Simpler than the generic;
 less precise for callers with
narrower-than-Schema types.

## What does not work

- `function isSchema(value: unknown): value is Schema` as
  the only guard for general use:
   loses caller's properties
  on every typed call site,
   forcing widespread casts to
  recover the type.
- `value as unknown as Schema & typeof value` as the
  universal recovery cast:
   visually noisy and easy to apply
  incorrectly.
   Use only at the trust boundary.
- Relying on `any` to bypass narrowing:
   `any` is still
  narrowed by typeguards.
   The "do anything" type does not
  protect against the narrowing rule.
- Intersection types as protection:
   `A & B & C` can be
  reduced to `A` during narrowing if the guard pattern
  discards the rest.
   The `&` operator does not guarantee
  preservation through narrowing.
- Casting to "help" TypeScript:
   `isGoodType(typed as unknown
  as GoodType & typeof typed)` sometimes produces worse
  narrowing than direct usage.
   The cast is for the trust
  boundary;
   do not sprinkle it speculatively.

## Practical guidelines

### Validating unknown data (trust boundary)

```ts
const untrusted: unknown = JSON.parse(data,);

if (isSchema(untrusted as unknown as Schema & typeof untrusted,))
  untrusted.parse('test',);
```

The cast is explicit at the boundary;
 downstream code sees
the original type.

### Narrowing typed values (no boundary crossed)

```ts
const typed: SchemaWithWeight | string = getValue();

if (isSchema(typed,))
  typed.weight; // preserved when `typed` was `SchemaWithWeight`
```

### Objects with additional properties

```ts
const enriched = { parse: x => x, weight: 100, metadata: {...} };

// DON'T cast to unknown; loses all properties
if (isSchema(enriched as unknown)) {
  enriched.weight;  // lost
}

// DO use direct validation; preserves properties
if (isSchema(enriched)) {
  enriched.weight;     // preserved
  enriched.metadata;   // preserved
}
```

## Migration strategy

Refactor all typeguards in the workspace to the
generic-extends pattern,
 in this order:

1.  Basic guards:
     `string`,
     `number`,
     `boolean`,
     `error`.
2.  Collections:
     `array`,
     `map`,
     `set`,
     `promise`.
3.  Complex objects:
     `schema`,
     `jsonl`.
4.  Update tests and callers;
     in particular,
     replace `as
    unknown` casts at call sites with the explicit
    trust-boundary form.

Evidence:
 84 test scenarios in
`typeguard.behaviorTest.ts` exercise the patterns above.

## Why we do not file this upstream

The behaviour is correct per the TypeScript specification.
Walking the 5 constraints:

1.  **Is it really upstream's fault?
    ** No;
     the narrowing rule
    is documented and consistent.
2.  **Can upstream fix it?
    ** Changing the narrowing rule to
    "intersect with the original input type" would break
    existing valid code and undermine the safety contract
    `unknown` provides.
3.  **Are they supporting this use case?
    ** Yes;
     both the
    `unknown` pattern and the generic-extends pattern are
    documented.
4.  **Will they likely fix it?
    ** N/A.
5.  **Have we prototyped a minimal fix?
    ** N/A.

Decision:
 no upstream report.
 The fix lives at our boundary
(generic-extends typeguard pattern + explicit cast at the
trust boundary).
