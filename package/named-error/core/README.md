# named-error-core

The base error class every named error class in this repository builds on.

An error class used to repeat its own identifier as a string:

```ts
export class TabbyMissingError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'TabbyMissingError';
  }
}
```

Nothing kept the two in step, so a renamed class left `error.name` stale, and
each new error class was another copy of the same constructor body. Extending
`NamedError` instead removes both:

```ts
import { NamedError, } from '@monochromatic-dev/named-error-core/ts';

export class TabbyMissingError extends NamedError {}
```

`new.target.name` names the leaf class, so a subclass of a subclass is still
named after the class that was constructed. A class that carries fields of its
own keeps its constructor and drops only the name assignment, because the base
constructor already ran that with the leaf class in hand.

```ts
export class QuotaExceededError extends NamedError {
  readonly usedBytes: number;

  constructor(usedBytes: number) {
    super(`quota exceeded by ${String(usedBytes)} bytes`);
    this.usedBytes = usedBytes;
  }
}
```

## When this base class does not fit

A class whose base type is fixed by something outside this repository cannot
extend `NamedError`: `package/upstream-fork/*` classes extend `TypeError`
because upstream `conf`, `p-map`, and their siblings throw `TypeError`, and a
caller's `instanceof TypeError` must keep its answer there. Those classes keep
their base and take their name from the class rather than from a repeated
literal, so the identifier is still stated once.

## Bundling

Every rolldown config in `package/config/rolldown` sets `mangle: false`, which
is what keeps a class's `name` intact in published artifacts. A config that
enables mangling would rename every error, so each package whose unit tests
assert an error's `name` runs those assertions against its built bundle.

## Tests

```shell
mise run //package/named-error/core:buildAndTest
```

The cases cover the name of a subclass that declares no constructor, the name
of a subclass of a subclass, a subclass carrying fields, message forwarding,
`options.cause` forwarding, and the prototype chain.
