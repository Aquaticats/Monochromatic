# A bundled chunk called a function without importing it

Building `package/pi-shared/subagent-driver` with rolldown 1.3.1 produced a bundle that
threw `ReferenceError: roomIn is not defined` as soon as `summaryTextOf` ran.
The same source passed every type check,
so nothing before a test run of the built output could have shown it.
This record states the mechanism,
the shape that avoids it,
and what is still unknown.

## What was seen

The source exported `roomIn` from `src/pool-limits.ts`,
and `src/pool-tallies.ts` imported it.
The build's chunk `dist/final/node/pool-gate-*.mjs` held `pool-tallies`' functions,
and its first line imported only two bindings from the `pool-limits` chunk:

```js
import { m as poolNamed, u as PROVIDER_POOLS } from './pool-limits-DABd0Oyj.mjs';
```

The chunk `pool-limits-DABd0Oyj.mjs` did export it (`export{roomIn as _,...}`),
and the same chunk body called `roomIn(pool)` where `summaryTextOf` and `poolHasRoom` needed it.
Every test that reached those two threw
`ReferenceError: roomIn is not defined`,
with the stack inside the bundle rather than in any source file.

Moving `roomIn` into a module of its own did not help:
the function was then inlined into `dist/final/node/index.mjs`,
and the `pool-gate` chunk still called it without importing it.

## The shape that works

Put the helper in the module whose other functions land in the same chunk as its callers.
Here that is `src/pool-tallies.ts`:
`summaryTextOf` and `poolHasRoom` call it,
and the function is defined beside them,
so no chunk boundary is crossed.
The bundle then defines `function roomIn(pool){...}` inside `pool-gate-*.mjs` itself.

A first fix would be to disable chunk splitting for this package,
which costs the shared chunks every command benefits from;
the placement above keeps the build as it is.

## What is not known

- Whether every cross-module function reference is affected, or only some.
  One function in one package was observed.
  `PROVIDER_POOLS` and `poolNamed` crossed the same chunk boundary and were imported correctly,
  so the failure is not "any binding from another module".
- Whether the alias role played.
  The missing binding was exported as `_`,
  a single-character alias,
  and no other export of that chunk had one.
- Whether an older or newer rolldown behaves the same.
  Only 1.3.1 was tried.

## How a change here is checked

Every unit test of this package imports the built `dist/final/node/index.mjs`,
and the command tests run the built bins through their `mise` tasks,
so a dropped import shows up as a `ReferenceError` in a test run.
`lint:types` alone cannot see it:
the type declarations are correct,
and only the emitted chunk is wrong.
Run `mise run //package/pi-shared/subagent-driver:test:unit` after moving a function between modules,
not just `lint:types`.
