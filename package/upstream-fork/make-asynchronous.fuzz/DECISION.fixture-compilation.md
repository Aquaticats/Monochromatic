# Decision: fixture compilation in the make-asynchronous-fork fuzz sidecar

## Context

The `make-asynchronous-fork.fuzz` differential oracle must feed both
 implementations structurally identical wrapped functions.
 Both sides serialize the wrapped function with
 `Function.prototype.toString` and run the source in a worker,
 so any fixture closing over spec state (payload objects,
 kind flags,
 value arrays)
 serializes into a worker-side `ReferenceError` instead of the specified
 outcome.
 Baking payloads in as JSON literals keeps fixtures self-contained,
 but building those literals needs `new Function` compilation in the
 fixture builders.

The repository's lint stack bans every obvious compilation mechanism:

- oxlint's built-in `eslint/no-new-func` (correctness category,
 blocking through `denyWarnings`) bans the `Function` constructor.
- oxlint's built-in `typescript/no-implied-eval` (correctness category,
 blocking through `denyWarnings`) bans implied eval through the same
 constructor.
- `typescript/no-unsafe-type-assertion` flags the factory's cast from the
 untyped constructor to the fixture signature.
- `unicorn/consistent-function-scoping` would hoist the compiled fixtures,
 which would reintroduce shared state between the two oracle sides.

## Decision

Compile fixture bodies through `new Function` inside
 `compileFixture` and `compileIterableFixture`
 (`package/upstream-fork/make-asynchronous.fuzz/src/spec-fixtures.ts`),
 carrying scoped
`// oxlint-disable-next-line eslint/no-new-func, typescript/no-implied-eval, typescript/no-unsafe-type-assertion -- ...`
suppresssions that name the worker-serialization boundary.

## Why the other paths lose

- Plain closures over the spec object serialize cleanly on the main thread
 but arrive in the worker without their bindings,
 so the first fuzz run fails with `ReferenceError: payloadJson is not
 defined` instead of the specified outcome.
- A `Proxy` `apply` trap avoids compilation but replaces a plain function
 with a per-call proxy whose `toString` is the trap source, not the
 fixture behavior,
 so both implementations would run identical trap text instead of the
 generated outcome.
- String-free fixture shapes (fixed outcome tables keyed by argument count)
 cannot express the generated payload, value-list, and failure-flag space
 the arbitraries produce.

## Why the scoped suppression fits

The suppressed rules are third-party heuristics whose own messages
 ("define the function directly",
 "do not use the Function constructor")
 are unsatisfiable at a worker-serialization boundary:
 the fixture source must be constructed as text to stay closed-over
 nothing.
 The `no-disable-*` ban list covers repo-authored rules only,
 so these scoped suppressions stay available,
 following the
 `package/upstream-fork/pify/DECISION.callback-capture.md`
 precedent for external-boundary mirrors.
 Each suppression names the rule,
 the boundary,
 and why the text construction is the only remaining capture.

## Consequences

- Fixture builders stay per-spec pure functions returning fresh,
 self-contained functions for each oracle side.
- The differential oracle in
 `package/upstream-fork/make-asynchronous.fuzz` compares settlements
 against upstream `make-asynchronous` 2.1.0 with this compilation in
 place.
- The runtime package itself never compiles source:
 `makeCallWorkerBody` and `makeIterableWorkerBody` only concatenate the
 caller-supplied function text.
