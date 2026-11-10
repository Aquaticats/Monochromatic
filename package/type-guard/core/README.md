# type-guard-core

Ready to publish.

Runtime type guards for values that arrive as `unknown`.

`@sindresorhus/is` supplies the primitive predicates;
this package owns the guards the library does not hold,
so a shape the repository narrows on has one body
instead of one per module.
`isRecord` was copied into more than fifty files
and `isErrnoException` into six,
with the copies disagreeing about what they accept.

## `isRecord`

Returns whether a value is a record:
an object that is neither `null`
nor an array
nor a function.
The narrowed type is `Readonly<Record<string, unknown>>`,
because a value that only passed "it is a record" has not been validated
past its shape,
so a caller that wants to change a field builds a new record.

Accepted: plain objects,
objects with a `null` prototype,
class instances,
`Map`,
`Date`,
and errors.
Rejected: `null`,
`undefined`,
arrays,
functions,
and every other primitive.

```ts
// package/type-guard/core/src/is-record.ts
isRecord({ field: 'value', }); // true
isRecord([]); // false
```

The function exclusion is deliberate:
`@sindresorhus/is`'s `object` predicate counts functions as objects,
while the copies this package replaces tested `typeof value === 'object'`,
which never matches a function.

## `isErrnoException`

Returns whether a thrown value is an error carrying an errno `code`,
narrowing to `NodeJS.ErrnoException`.

```ts
// package/type-guard/core/src/is-errno-exception.ts
const error: NodeJS.ErrnoException = new Error('missing');
error.code = 'ENOENT';
isErrnoException(error); // true
isErrnoException({ code: 'ENOENT' }); // false
```

The body requires a real error as well as a code.
Node's filesystem and process functions throw genuine errors,
so a bare object shaped like an errno came from some other thrower,
and reading its `code` as a missing path would swallow that failure.
`package/wg/cli-quicker` settled this body and its test suite;
this package takes both over,
so the strict spelling is the one every caller reaches.

## Absence sentinels are declared, not built

A module that reports "the value is absent" declares its own marker:

```ts
// package/fs-path/core/src/root-filesystem-contract.ts
export const ABSENT: unique symbol = Symbol(
  'root discovery path content absent on filesystem',
);
```

No factory can own that idiom.
TypeScript rejects a function whose return type is `unique symbol`
(`TS1335: 'unique symbol' types are not allowed here`),
and rejects assigning a call result to a `unique symbol` declaration
(`TS2322`),
so a helper could only hand back a plain `symbol`,
which erases the distinction between one module's absent marker
and another's.
The declaration therefore stays beside the contract that returns it,
with the annotation written out:
`const NAME: unique symbol = Symbol('what is absent and why',)`.
A description names what is missing in plain words,
because it is the only thing a stack trace or a debugger shows.

## Usage

```ts
import {
  isErrnoException,
  isRecord,
} from '@monochromatic-dev/type-guard-core/ts';

declare const payload: unknown;

if (!isRecord(payload,)) {
  throw new Error('response body is not a record',);
}

const { result, } = payload;
```

Source imports use the package's `/ts` subpath,
per `doc/decision/workspace-ts-source-imports.md`.
