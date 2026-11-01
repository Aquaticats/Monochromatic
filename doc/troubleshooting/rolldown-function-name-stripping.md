# Rolldown 1.2.9 minified builds strip named function expression names, so property-name inference renames `fn.name` and wrapper `toString` markers change

## Symptom

A named function expression used as an object property value keeps its name
in source execution
 (`node src.ts`)
 but loses it in the bundled output
 (`rolldown
--configLoader native`):

```ts
// source: the wrapper is explicitly named 'memoized'
const memoized = mimicFunction({
  to: function memoized(this: unknown, call) { /* … */ },
  from: fn,
  ignoreNonConfigurable: true,
},);
```

Observable symptom from the affected package's test run:

```text
AssertionError: expected '/* Wrapped with to() */…' to equal '/* Wrapped with memoized() */…'
```

The marker shows `to`,
 the property key the function expression was assigned
to,
 instead of the declared name.

## Root cause

An anonymous function expression assigned as an object property value gets
its `name` from the property key
 (ECMAScript `SetFunctionName`),
 and a named
function expression keeps its own name.
 Rolldown's minified bundle drops the
inner name binding
 (it is unreferenced),
 which turns the named expression
into an anonymous one,
 so property-key inference runs:

```js
// package/upstream-fork/p-memoize/dist/final/neutral/index.mjs
let memoized=mimicFunction({to:function({args}){/* … */},from:fn,ignoreNonConfigurable:!0});
```

`mimicFunction` reads `to.name` before copying the source identity
(`src/mimic-function.ts`: `const { name, } = to;`),
 so the wrapper marker and
any name-dependent output silently change with the build mode.

Earlier reading was wrong: blaming the source shape
 (`to: function memoized`)
is not the cause;
 the same source keeps the name under `node src.ts`.
 Only the
bundled artifact loses it.

## Verification

- rolldown `v1.2.9`
 (build banner `✔ rolldown v1.2.9 Finished`),
 Node
`v26.10.0`.
- Harness: `node probe.mjs` printing `o.to.name` for
`const o = { to: function memoized() { return 1; } }` reports `memoized`;
 the
bundled build of the same pattern reports `to`
 (visible in the bundle excerpt
above and in the failing assertion).
- Works cleanly: function declarations referenced by name in source
execution;
 named function expressions in unbundled runs;
 explicit
`Object.defineProperty` name assignment in bundled runs.
- Fails: named function expressions as property values after minification;
 anonymous ones always take the property key
 (`to`),
 bundled or not.

## Verified workarounds

Set the name explicitly before any code reads it:

```ts
// package/upstream-fork/p-memoize/src/p-memoize.ts
to: Object.defineProperty(
  function memoized(/* … */) { /* … */ },
  'name',
  {
    value: 'memoized',
    configurable: true,
  },
),
```

Tradeoffs: the name is now data instead of syntax,
 so renaming the wrapper
requires editing the string too;
 the comment above the call site names the
reason.
 The marker output is stable across build modes and matches the
unbundled source exactly.

## What does not work

- Relying on named function expressions: minification strips the name.
- Relying on `const memoized = function (…) {…}` name inference: same
stripping,
 and the repository's `no-variable-function-expression` rule bans
that shape anyway.
- Comparing against a bundled `fn.name` in tests without pinning it: the
value differs between source runs and artifact runs,
 so tests pass locally
and fail against `dist`.

## Upstream filing artifact

Nothing to add.
 Dropping unreferenced name bindings in minified output is
standard minifier behavior
 (source maps and `keepNames`-style options are the
documented escape hatches),
 and this fork's fix is a runtime-owned name
assignment.
 Revisit only if a `keepNames` guarantee is verified for the
repository's rolldown config and preferred over the explicit assignment.
