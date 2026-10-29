# module-or-throw

Ready to publish.

Runtime assertions that pass the value through or throw.

Each helper narrows the static type when the runtime check passes,
and throws an `Error` describing the offending value when the check fails.
Use these wherever the non-null assertion operator (`!`) would otherwise be reached for:
the runtime check turns a silent type lie into a loud,
 debuggable failure.

## Helpers

### Boolean shape

<table>
<thead>
<tr>
<th>Function</th>
<th>Throws when</th>
</tr>
</thead>
<tbody>
<tr>
<td>`nonNullishOrThrow`</td>
<td>value is `null` or `undefined`</td>
</tr>
<tr>
<td>`truthyOrThrow`</td>
<td>value is falsy (`false`, `0`, `0n`, `''`, `null`, `undefined`, or `NaN`)</td>
</tr>
<tr>
<td>`falsyOrThrow`</td>
<td>value is truthy</td>
</tr>
</tbody>
</table>

### Container size

<table>
<thead>
<tr>
<th>Function</th>
<th>Throws when</th>
</tr>
</thead>
<tbody>
<tr>
<td>`emptyOrThrow`</td>
<td>value lacks a recognized size shape, or has nonzero size</td>
</tr>
<tr>
<td>`nonemptyOrThrow`</td>
<td>value lacks a recognized size shape, or has zero size</td>
</tr>
</tbody>
</table>

Recognized size shapes:
 strings,
 arrays,
 `Set`,
 `Map`,
 plain objects.
`WeakSet` and `WeakMap` are intentionally rejected (no enumerable size).
Bare iterables and async iterables are rejected (sizing requires consumption).

### Iterable protocols

<table>
<thead>
<tr>
<th>Function</th>
<th>Throws when</th>
</tr>
</thead>
<tbody>
<tr>
<td>`iterableOrThrow`</td>
<td>value does not implement the sync-iterable protocol</td>
</tr>
<tr>
<td>`asyncIterableOrThrow`</td>
<td>value does not implement the async-iterable protocol</td>
</tr>
<tr>
<td>`maybeAsyncIterableOrThrow`</td>
<td>value implements neither protocol</td>
</tr>
</tbody>
</table>

Strings count as sync-iterable.
Objects must have `Symbol.iterator` or `Symbol.asyncIterator` to pass.

### Container instances

<table>
<thead>
<tr>
<th>Function</th>
<th>Throws when</th>
</tr>
</thead>
<tbody>
<tr>
<td>`arrayOrThrow`</td>
<td>`Array.isArray(value)` is `false`</td>
</tr>
<tr>
<td>`setOrThrow`</td>
<td>value is not `instanceof Set`</td>
</tr>
<tr>
<td>`mapOrThrow`</td>
<td>value is not `instanceof Map`</td>
</tr>
<tr>
<td>`weakSetOrThrow`</td>
<td>value is not `instanceof WeakSet`</td>
</tr>
<tr>
<td>`weakMapOrThrow`</td>
<td>value is not `instanceof WeakMap`</td>
</tr>
</tbody>
</table>

### Standard built-ins

<table>
<thead>
<tr>
<th>Function</th>
<th>Throws when</th>
</tr>
</thead>
<tbody>
<tr>
<td>`promiseOrThrow`</td>
<td>value is not `instanceof Promise`</td>
</tr>
<tr>
<td>`dateOrThrow`</td>
<td>value is not `instanceof Date`</td>
</tr>
<tr>
<td>`regExpOrThrow`</td>
<td>value is not `instanceof RegExp`</td>
</tr>
<tr>
<td>`errorOrThrow`</td>
<td>`Error.isError(value)` is `false`</td>
</tr>
</tbody>
</table>

`dateOrThrow` does not check date validity (invalid dates still pass).
`promiseOrThrow` rejects thenables.
`errorOrThrow` accepts genuine errors from any realm and rejects error-shaped plain objects.

### typeof primitives

<table>
<thead>
<tr>
<th>Function</th>
<th>Throws when</th>
</tr>
</thead>
<tbody>
<tr>
<td>`stringOrThrow`</td>
<td>`typeof value !== 'string'`</td>
</tr>
<tr>
<td>`numberOrThrow`</td>
<td>`typeof value !== 'number'` (accepts `NaN` and `Infinity`)</td>
</tr>
<tr>
<td>`bigintOrThrow`</td>
<td>`typeof value !== 'bigint'`</td>
</tr>
<tr>
<td>`booleanOrThrow`</td>
<td>`typeof value !== 'boolean'`</td>
</tr>
<tr>
<td>`symbolOrThrow`</td>
<td>`typeof value !== 'symbol'`</td>
</tr>
<tr>
<td>`functionOrThrow`</td>
<td>`typeof value !== 'function'`</td>
</tr>
<tr>
<td>`objectOrThrow`</td>
<td>`typeof value !== 'object'` or `value === null`</td>
</tr>
</tbody>
</table>

Boxed-primitive wrappers (`new String(...)`,
 `new Number(...)`,
 etc.) are intentionally rejected.

### Numeric union

<table>
<thead>
<tr>
<th>Function</th>
<th>Throws when</th>
</tr>
</thead>
<tbody>
<tr>
<td>`numericOrThrow`</td>
<td>value is neither `number` nor `bigint`</td>
</tr>
</tbody>
</table>

`numericOrThrow` corresponds to the `t numeric/` category in `module-es`.

### Custom predicates

- `satisfiesOrThrow({ value, predicate? })(candidate)` throws when the synchronous
  satisfaction check fails.
- `satisfiesOrThrowAsync({ value, predicate? })(candidate)` throws when the async-capable
  satisfaction check fails.

When `predicate` is omitted,
 both helpers use `Object.is(candidate, value)`.
Default equality narrows the successful return type to the intersection of the candidate type
and the configured value type.

When `predicate` is present,
 it receives one object parameter:
`{ candidate, value }`.
A `true` result returns `candidate` unchanged.
A `false` result throws an `Error`.
Predicates may also throw their own error.
That is a supported use case:
 the helper does not wrap or replace predicate-thrown errors,
so predicates can provide domain-specific diagnostics.
Predicates must return literal booleans.
The synchronous helper rejects Promise-returning predicates at runtime and points callers to
`satisfiesOrThrowAsync`.
The async helper accepts `boolean` or `Promise<boolean>`,
 then rejects resolved non-boolean values.
Async predicate rejections also propagate unchanged.
Custom predicates deliberately return the candidate type as-is because predicates can be fuzzy:
for example,
 a case-insensitive predicate can accept `'READY'` for configured value `'ready'`,
so typing the result as literal `'ready'` would lie about the runtime value.

## Usage

```ts
import {
  arrayOrThrow,
  nonNullishOrThrow,
  stringOrThrow,
} from '@monochromatic-dev/or-throw-core';

const el = nonNullishOrThrow(document.querySelector('.target',),);
// el is Element (was Element | null)

const text = stringOrThrow(el.textContent,);
// text is string (was string | null)

const tokens = arrayOrThrow(text.match(/\w+/g,),);
// tokens is readonly unknown[] (was RegExpMatchArray | null)
```

### Custom predicate usage

```ts
import {
  satisfiesOrThrow,
  satisfiesOrThrowAsync,
} from '@monochromatic-dev/or-throw-core';

declare const rawStatus: unknown;
declare const expectedChecksum: string;
declare const filePath: string;
declare function checksumMatches(parameters: {
  readonly candidate: unknown;
  readonly value: string;
}): Promise<boolean>;

const exactReady = satisfiesOrThrow({ value: 'ready' as const, })(rawStatus,);
// exactReady is typed as 'ready' when rawStatus was unknown.

const readyish = satisfiesOrThrow({
  value: 'ready',
  predicate: ({ candidate, value, }) =>
    ((typeof candidate) === 'string')
    && (candidate.toLowerCase() === value),
})('READY',);
// readyish is the original candidate, 'READY'.

const stored = await satisfiesOrThrowAsync({
  value: expectedChecksum,
  predicate: async ({ candidate, value, }) => await checksumMatches({
    candidate,
    value,
  },),
})(filePath,);
// stored is the original candidate, filePath.
```

## Types

```ts
import type {
  ExtractOrUnknown,
  Falsy,
} from '@monochromatic-dev/or-throw-core';
```

- `Falsy`:
   union of `false | 0 | 0n | '' | null | undefined`.
  `NaN` is falsy at runtime but cannot be represented as a literal type.
- `ExtractOrUnknown<T, U>`:
   variant of `Extract<T, U>` that returns `U` when `T` is `unknown`,
  instead of collapsing to `never`.
  Used internally by every `Extract`-based narrowing helper so `unknown` inputs
  (e.g. parsed JSON) narrow correctly.
- `SatisfiesOrThrowPredicateParameters<Candidate, Value>`:
   object passed into custom predicates.
- `SatisfiesOrThrowPredicate<Value, Candidate = unknown>`:
   synchronous predicate type returning `boolean`.
- `SatisfiesOrThrowAsyncPredicate<Value, Candidate = unknown>`:
   async-capable predicate type returning `boolean`
  or `Promise<boolean>`.

## NaN caveat

`NaN` is falsy at runtime but cannot be represented as a TypeScript literal type.
`truthyOrThrow(NaN,)` throws at runtime,
but TypeScript cannot statically exclude `NaN` from a `number` parameter,
so the returned type is the same `number`.
`numberOrThrow` accepts `NaN`:
 it is a number-typed value.
Callers that need a NaN-narrowed type must use `Number.isNaN` separately.

## Design decisions

- **Source-only.
  ** The package has no build step;
   consumers import directly from `src/index.ts`.
- **Direct named exports.
  ** No `$` aliasing.
   Consumers write `import { nonNullishOrThrow }` and call the function by its real name.
- **One file per helper.
  ** Each helper owns a sibling file;
   `index.ts` is a pure re-export barrel.
- **Family convention.
  ** Every helper in this package uses the `<predicate>OrThrow` suffix
  so the throw semantics are explicit at every call site,
  even when the import line is folded or out of view.
- **`ExtractOrUnknown` over `Extract`.
  ** Plain `Extract<unknown, X>` evaluates to `never`,
  silently breaking the most common call shape (a value typed `unknown` from parsed JSON or
  a fetched payload).
   Every narrowing helper uses `ExtractOrUnknown` instead.
- **Skipped categories.
  ** Value-range refinements (`positiveOrThrow`,
   `finiteOrThrow`,
  `integerOrThrow`) are intentionally out of scope;
   add them when a call site needs them.
