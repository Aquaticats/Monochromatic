# @monochromatic-dev/pnpm-override-stub-silent

Ready to publish.

A workspace stub package used by `.pnpmfile.mjs` to substitute for blocked dependencies whose policy is `silent`.

When a package is blocklisted with `action: 'silent'`
in the policy table at the repo root's `.pnpmfile.policies.json`
(read by the hook in `.pnpmfile.mjs`),
 the pnpm install hook rewrites every transitive dependency entry pointing at the blocked package to point here instead.
The stub exports a callable Proxy whose property access,
 function call,
 and `new` invocation return the Proxy itself,
 and whose `in` checks return `false`.
Three property keys are answered differently,
 because returning the Proxy for them crashes or hangs a consumer instead of handing it a wrong value:

- `then`,
   `catch`,
   and `finally` read as `undefined`,
   so awaiting the stub,
   a property chain of it,
   or a call result of it settles instead of deadlocking on a `then` whose resolve and reject callbacks the stub
   never invokes.
   Code that calls one of them as a method throws `TypeError` at that call,
   a shape the package declaration cannot express,
   because one `PropertyKey` index signature types them as the stub itself.
- `Symbol.toPrimitive` reads as a function returning `[stub-silent]`,
   so `String(stub)` and template interpolation produce that text instead of throwing
   `TypeError: Cannot convert object to primitive value`.
- `'prototype' in stub` answers `true`,
   because JavaScript forbids a Proxy from reporting a non-configurable own property of its target as absent,
   and the wrapped function's `prototype` is the only such property.

This lets shape-probing code (`typeof X === 'function'`,
 `X.someMethod`,
 `new X()`,
 `X()`) run without throwing,
 at the cost of incorrect downstream behavior whenever the stub's value is actually used.

Use this only for soft migrations where you want the build green
and you accept incorrect-but-not-crashing runtime behavior.
For a loud failure,
 use `@monochromatic-dev/pnpm-override-stub-throwing` instead.
For optional dependencies that consumers handle via `try { require } catch`,
 use global removal in `pnpm-workspace.yaml`.

See `doc/dependency-blocklist.md` for the full mechanism and decision rule.
