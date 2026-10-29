# module-throws

Ready to publish.

Expression-position throwing helper for code that needs to throw a prebuilt `Error`
where JavaScript syntax requires an expression.

## Export

Defined in `src/throws.ts`:

- `throws(error: Error): never`:
   throws the same `Error` instance and never returns.

## Usage

```ts
import { throws, } from '@monochromatic-dev/throws-core';

const token = maybeToken ?? throws(new MissingTokenError(),);
```

The return type is `never`,
 so TypeScript keeps the non-nullish branch type:

```ts
declare const maybeToken: string | undefined;

const token: string = maybeToken ?? throws(new MissingTokenError(),);
```

Use this helper when statement `throw` cannot preserve the surrounding expression semantics:

- Nullish fallback expressions where the fallback must throw a domain-specific error.
- Default parameter initializers where `function.length`,
   parameter scope,
   or evaluation order matters.
- Destructuring defaults that intentionally trigger only on `undefined`.
- Class field initializers where declaration order or private-field initialization matters.
- Declarative object literals whose contextual typing should remain inline.

## Relationship to module-or-throw

`@monochromatic-dev/or-throw-core` validates a value and returns that same value when it passes:

```ts
const token = nonNullishOrThrow(maybeToken,);
```

That package is right when the failure can use the package's generic error policy.
This package is right when the failure must throw a caller-owned domain `Error`:

```ts
const token = maybeToken ?? throws(new MissingTokenError(context,),);
```

## Footguns avoided

`throws` exposes only an `Error` parameter.
JavaScript can still call any function with any value at runtime,
and TypeScript's structural `Error` type means a plain `{ name, message }` object can satisfy the type.
Callers should still construct real `Error` instances.
The TypeScript API deliberately omits string and descriptor overloads:

- `Error` preserves stack,
   message,
   cause,
   and subclass identity for catch sites.
- Prebuilt errors let callers decide the concrete domain subclass.
- String overloads would force this helper to construct `new Error`,
   moving stack capture into the helper.
- Descriptor objects would recreate ad hoc mapping from strings to constructors.
- `unknown` would make non-`Error` throws easy and catch-side handling worse.

`throws` does not call `Error.captureStackTrace` because that API is V8-specific.
It does not log the error because error messages can contain secrets and callers own logging policy.

## Prefer statement throw when possible

Do not use this helper as a blanket replacement for statement `throw`.
When a block can express the control flow clearly,
 use direct statements:

```ts
if (token === undefined)
  throw new MissingTokenError(context,);
```

Use `throws` only when extracting the expression would change observable semantics
or make the type relationship less clear.
