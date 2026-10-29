# Node 26.10.0 type stripping rejects TypeScript decorator syntax
with `SyntaxError: Invalid or unexpected token` at the `@` token,
so test files cannot apply decorators syntactically

## Symptom

Running a `.ts` file through `node` directly
 (the repository's `test:unit` harness
runs test files as `node <file>`)
 fails as soon as the file contains decorator
syntax:

```text
file:///…/dec.ts:4
  @dec
  ^

SyntaxError: Invalid or unexpected token
    at compileSourceTextModule (node:internal/modules/esm/utils:355:16)
```

The same file compiles cleanly under `tsc`,
 and the same error appears under
the CommonJS loader path
 (`at wrapSafe (node:internal/modules/cjs/loader:1888:18)`),
so it is the type-stripping transform,
 not module resolution.

## Root cause

Node's built-in TypeScript support strips type annotations only;
 decorators
are not erasable syntax
 (they change runtime semantics),
 so the loader hands the
decorator token to the JavaScript parser unchanged and the parser rejects
`@`.

The probe that isolates it:

```ts
// ~/temp/agent/p-memoize-probe/dec.ts
function dec(value: unknown, context: unknown): void {}

class C {
  @dec
  async m(): Promise<number> { return 1; }
}
console.log('decorators-ok', typeof new C().m);
```

`node dec.ts` fails at the `@dec` line;
 `tsc --noEmit` on the same file passes.
`node --experimental-transform-types` is not available on this Node build
(`node: bad option: --experimental-transform-types`),
 so no flag opts in.

## Verification

- Node `v26.10.0`
 (`node --version`),
 TypeScript `7.0.2`
 via the workspace's `node_modules/.bin/tsc`.
- Harness: `node ${HOME}/temp/agent/p-memoize-probe/dec.ts` fails;
 the same
file with the decorator line removed runs.
- Works cleanly: ordinary annotations,
 interfaces,
 generics,
 `declare` fields,
 enums-free classes without decorators.
- Fails: `@decorator` on a method,
 `@decorator` on a getter,
 any decorator
expression position.

## Verified workarounds

Drive the decorator through its function interface with a synthetic
context object instead of decorator syntax:

```ts
// package/upstream-fork/p-memoize/src/p-memoize-decorator.unit.test.ts
const decoration = createFakeDecoration({ kind: 'method', isPrivate: false, });
pMemoizeDecorator()(counterMethod, decoration.context,);
decoration.initialize(alpha,);
```

Tradeoffs: the runtime's real `ClassMethodDecoratorContext` construction is
not exercised
 (the standard library provides no constructor for it),
 so
initializer registration and `context.name` handling are verified against a
shape double;
 TypeScript still type-checks real decorator application in a
separate `tsc`-only probe.

## What does not work

- `node --experimental-transform-types`: bad option on Node 26.10.0.
- Compiling tests to JS first: the repository's `test:unit` task runs raw
`.ts` files through `node`,
 so a build step before every test run is a
workflow change,
 not a local fix.
- `experimentalDecorators` in tsconfig: changes the decorator protocol
shape entirely
 (legacy decorators),
 so the code under test would differ from what
consumers run.

## Upstream filing artifact

Nothing to add.
 Non-erasable syntax rejection is documented Node behavior
(the loader's remit is erasable annotations only),
 and no architectural
prototype exists here for a strip-types decorator transform.
 Revisit only
if Node ships decorator support in its transform pipeline.
