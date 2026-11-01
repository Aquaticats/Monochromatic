# Node 26.12.0 typed error fields bypass inherited name setters

## Symptom

A proposed TypeScript port adds `override readonly name: string` to an existing error class.
Ordinary message and cause checks still pass.
A JavaScript consumer's inherited `name` setter stops running,
and construction creates an own `name` property instead.

The real proposal produces no syntax or runtime exception.
The regression assertion reports `AssertionError [ERR_ASSERTION]`:
the observed setter count is zero instead of one.
The defect is the unnecessary field added by the consumer port.

## Root cause

In Node release `v26.12.0`,
commit `26e72d85a611e222e3f06025de8a3c496923bacd`,
`lib/internal/modules/typescript.js:168` selects the type-stripping mode:

```js
// Node checkout: lib/internal/modules/typescript.js
const options = {
  mode: 'strip-only',
  filename: sourceURL ?? filename,
};
```

The parser loaded at `lib/internal/modules/typescript.js:36` is `amaro.transformSync`:

```js
// Node checkout: lib/internal/modules/typescript.js
const amaro = require('internal/deps/amaro/dist/index');
return amaro.transformSync;
```

The actual proposal's stripped output,
retained as `policy-error-field-tC0CTZ/stripped.mjs:14` in the private verification directory,
still contains `name;`.
The annotation and modifiers disappear;
the class field remains executable JavaScript.

V8 emits an own-property definition for that field.
The same Node checkout's `deps/v8/src/interpreter/bytecode-generator.cc:3909` contains:

```cpp
// Node checkout: deps/v8/src/interpreter/bytecode-generator.cc
builder()->DefineNamedOwnProperty(
    builder()->Receiver(),
    property->key()->AsLiteral()->AsRawPropertyName(),
    feedback_index(slot));
```

That own field exists before the constructor's `this.name = this.constructor.name` assignment.
It shadows the consumer's prototype accessor.
The original class has no field declaration,
so the assignment reaches the inherited setter.
Calling the whole change type-only was the incorrect assumption.

## Verification

The standalone program passes without an argument and fails when passed `field`:

```ts
// error-field-probe.ts
import assert from 'node:assert/strict';

class AssignmentError extends Error {
  constructor(message: string) {
    super(message);
    this.name = this.constructor.name;
  }
}

class FieldError extends Error {
  override readonly name: string;

  constructor(message: string) {
    super(message);
    this.name = this.constructor.name;
  }
}

function observe(Type: new(message: string) => Error) {
  const received: string[] = [];
  class ConsumerError extends Type {}
  Object.defineProperty(ConsumerError.prototype, 'name', {
    configurable: true,
    get() { return 'consumer-name'; },
    set(value: string) { received.push(value); },
  });
  const error = new ConsumerError('probe');
  return {
    setterCalls: received.length,
    ownName: Object.hasOwn(error, 'name'),
    name: error.name,
  };
}

const Type = process.argv[2] === 'field' ? FieldError : AssignmentError;
assert.deepEqual(observe(Type), {
  setterCalls: 1,
  ownName: false,
  name: 'consumer-name',
});
```

Passing catalog:

- The assignment-only class invokes the setter once and leaves the accessor inherited.
- The corrected typed class without a `name` member matches that behavior.
- Both proposed and corrected classes retain ordinary message and cause identity.

Failing catalog:

- The proposed annotated field invokes no inherited setter and creates an own name.
- The standalone `field` variant fails its preserved-behavior assertion.

Private receipts are `policy-error-field-receipt.json`,
`corrected-error-field-receipt.json`,
`error-field-repro-assignment.log` and `error-field-repro-field.log` under
`${HOME}/temp/agent/pi-mimo-preparation-failure-20261018`.
The maintained [built error regression](../../package/pi-plugin/auto-mode/src/native-policy-slot-error.unit.test.ts)
also checks constructor arity,
native prototypes,
optional cause identity and facade/implementation identity.

## Verified workaround

Remove the unnecessary member declaration and preserve the constructor assignment.
The inherited `Error.name` declaration already supplies its string type.
This preserves the existing setter behavior without adding another runtime member.
The tradeoff is retaining a dynamically derived error name;
renaming or minifying a constructing subclass can change that name.

## What does not work

- Adding `readonly` does not erase the field.
- Checking only message and cause misses this difference;
  those positive controls pass for the rejected proposal too.
- Treating every added TypeScript declaration as runtime-neutral skips the emitted field boundary.

## Upstream filing decision

- Upstream fault: not established.
  The consumer introduced the executable field.
- Fixability: the engine implementation is changeable,
  but changing JavaScript field definition is not the proposed consumer repair.
- Supported use case: Node documents erasable TypeScript syntax,
  and its V8 source implements class fields through own-property definitions.
- Contribution policy: not inspected because no upstream change is proposed;
  no restriction or welcome is inferred.
- Maintainer disposition: not investigated;
  no claim about a future upstream fix is made.
- Prototype: the consumer correction and its positive controls are verified.
  No engine patch is proposed.

Nothing to add upstream:
the unnecessary declaration belongs to this repository's proposed port.
