# Declaring an absence marker

A module that reports "the value is absent" declares one marker beside the
contract that returns it:

```ts
// package/fs-path/core/src/root-filesystem-contract.ts
export const ABSENT: unique symbol = Symbol(
  'root discovery path content absent on filesystem',
);
```

The marker is a `unique symbol`,
never `null`,
`undefined` or `''`,
so an absent value stays distinct from a legitimate empty one,
and `content !== ABSENT` narrows the union.

## The declaration stays in its module

No helper can own this idiom,
and `@monochromatic-dev/type-guard-core` therefore exports no factory for it:

- TypeScript rejects a function whose return type is `unique symbol`:
  `TS1335: 'unique symbol' types are not allowed here`.
- It rejects assigning a call result to a `unique symbol` declaration:
  `TS2322: Type 'symbol' is not assignable to type 'unique symbol'`.

A factory could only hand back a plain `symbol`,
which erases the difference between one module's absent marker and another's,
so the declaration stays where its contract lives.

## Inference and annotation agree

`const NOT_FOUND = Symbol('module specifier resolution candidate not found')`
is typed `unique symbol` too:
seeding it from a second `Symbol()` call fails to compile,
exactly as it does against the annotated form.
The annotation is still written out,
because it states the shape the file promises,
rather than leaving it to inference.

## Name what is absent

The description is the only text a stack trace or a debugger shows for the
marker,
so it names what is missing and why in plain words.
The `no-restricted-syntax(no-low-information-symbol-description)` lint rule
rejects a description of fewer than three distinct words.
