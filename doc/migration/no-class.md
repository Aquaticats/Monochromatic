# Migrate classes to factory functions

Tracks the migration to `no-restricted-syntax/no-class`.
 The rule is shipped
at severity `'warn'` so the current footprint shows in lint output without
blocking CI.
 After every file in this document has been migrated or
justified,
 flip the severity to `'error'` in
`package/config/oxlint/src/rule/restriction.ts`.

## Rule summary

A `class` declaration or expression passes when **either**:

- The direct superclass identifier ends with a configured suffix,
   **or**
- The class's own name ends with a configured suffix.

Default suffixes:
 `Error`,
 `Element`.
 They cover:

- Direct `extends Error` (the global ends in `Error`).
- Direct `extends HTMLElement` / `extends LitElement` (both end in `Element`).
- Transitive chains like `class TomlPathNotFoundError extends TomlEditError`
  where the child class is named `*Error`.
- `declare class` in ambient `.d.ts` files (skipped entirely;
   no runtime).

Anything else is forbidden.
 Replace with a factory function returning a
frozen object.

## Migration recipe: stateful coordinator

Before:

```ts
export class HashCache {
  readonly #map: Map<string, string> = new Map();
  readonly #maxHashSize: number;

  constructor({ maxHashSize, }: HashCacheOptions = {},) {
    this.#maxHashSize = maxHashSize ?? DEFAULT_MAX_HASH_SIZE_BYTES;
  }

  async hashFile(absolutePath: string,): Promise<string | null> {
    const { size, } = await stat(absolutePath,);
    if (size > this.#maxHashSize)
      return null;
    /* ... */
  }

  get(absolutePath: string,): string | undefined {
    return this.#map.get(absolutePath,);
  }
}
```

After:

```ts
export type HashCache = Readonly<{
  hashFile(absolutePath: string,): Promise<string | null>;
  get(absolutePath: string,): string | undefined;
}>;

export function createHashCache(
  { maxHashSize, }: HashCacheOptions = {},
): HashCache {
  const map = new Map<string, string>();
  const cap = maxHashSize ?? DEFAULT_MAX_HASH_SIZE_BYTES;

  async function hashFile(absolutePath: string,): Promise<string | null> {
    const { size, } = await stat(absolutePath,);
    if (size > cap)
      return null;
    /* ... */
  }

  function get(absolutePath: string,): string | undefined {
    return map.get(absolutePath,);
  }

  return Object.freeze({
    hashFile,
    get,
  },);
}
```

Notes:

- The explicit `Readonly<{ ... }>` return type closes the leak path the
  user's correction flagged:
   `map` is captured but not referenced in the
  return shape,
   so no consumer can reach it.
- `delete` (reserved word) becomes `function deleteFn() {}` aliased as
  `delete: deleteFn` in the returned literal when that name is needed.
- Getters are written `get size() { return map.size }` directly inside
  the returned object literal.
- Method-to-method calls go through the closure binding,
   not `this`:
  `set(...)` instead of `this.set(...)`.

## Migration recipe: Disposable

`Symbol.dispose` and `Symbol.asyncDispose` belong as keys on the returned
literal.
 The `using` syntax does not require class membership.

Before:

```ts
class TcpServerDisposable implements Disposable {
  readonly #server: net.Server;
  constructor(server: net.Server,) { this.#server = server; }
  [Symbol.dispose](): void { this.#server.close(); }
}

using server = new TcpServerDisposable(makeServer(),);
```

After:

```ts
function disposableServer(server: net.Server,): Disposable {
  return {
    [Symbol.dispose](): void { server.close(); },
  };
}

using server = disposableServer(makeServer(),);
```

The same shape works for `AsyncDisposable` with
`async [Symbol.asyncDispose]() { ... }`.

## File inventory

Each file below contains at least one class flagged by the rule under the
default suffix list.
 Migrate one file per commit,
 following the per-file
recipe in the section above.
 Group A is production code;
 group B is test
fixtures.

### Group A: stateful coordinators (production)

1. `package/watch-restart/cli/src/watcher.ts` -- `Watcher`
1. `package/watch-restart/cli/src/child.ts` -- `Child`
1. `package/watch-restart/cli/src/hash-cache.ts` -- `HashCache`
1. `package/pi-plugin/morph-compact/src/morph-client.ts` -- `MorphCompactClient`
1. `package/zip-writer/core/src/index.ts` -- `ZipWriter`
1. `package/figma/kiwi/src/index.ts` -- `BinaryReader` (see "Lint blind spot" below)

### Completed in editord

Migrated on 2026-05-20:

1. `package/desktop-daemon/editord/src/server/lsp/lsp-pool.ts` -- `LspPool`
1. `package/desktop-daemon/editord/src/server/lsp/lsp-client.ts` -- `LspClient`
1. `package/desktop-daemon/editord/src/server/lsp/diagnostic-store.ts` -- `DiagnosticStore`
1. `package/desktop-daemon/editord/src/server/operations/watch-filesystem.ts` -- `DirWatcher`
1. `package/desktop-daemon/editord/src/client/ws/client.ts` -- `EditorWsClient`
1. `package/desktop-daemon/editord/src/client/context-menu/context-menu.ts` -- `ContextMenu`
1. `package/done/app/src/client/components/task-detail-autofill.ts` -- `AutofillManager`
1. `package/done/variant-postcss/src/client/components/task-detail-autofill.ts` -- `AutofillController`

### Group B: test-fixture Disposables

1. `package/watch-restart/cli/src/start.unit.test.ts` -- `FakeChild`
1. `package/watch-restart/cli/src/child.unit.test.ts` -- `FakeChild`
1. `package/webapp-forge/server/src/server/routes/git.cli.unit.test.ts` -- `DisposableServer`
1. `package/pi-plugin/morph-compact/src/ipc-socket-tcp.unit.test.ts` -- `TcpServerDisposable`
1. `package/pi-plugin/morph-compact/src/api-key.unit.test.ts` -- `EnvRestore`
1. `package/pi-plugin/morph-compact/src/ipc-file.unit.test.ts` -- `FileDisposable`
1. `package/pi-plugin/morph-compact/src/ipc-socket-unix.unit.test.ts` -- `SocketServerDisposable`
1. `package/test-harness/core/src/sinon.unit.test.ts` -- `Greeter`

## Lint blind spot

`package/figma/kiwi` does not declare a `lint:oxlint` mise task;
 its
`mise.toml` only wires up `build`,
 `watch:build`,
 `lint`,
 and `lint:types`.
The class in `src/index.ts` (`BinaryReader`) is therefore not caught by the
no-class rule today.
 Two options:

- Migrate `BinaryReader` to a factory manually and verify by reading the
  source,
   since lint won't flag it.
- Add a `[tasks."lint:oxlint"] extends = "lint:oxlint"` entry to the
  package's `mise.toml` and let lint catch it.

Choose during migration;
 either way the class needs to go.

## Per-file workflow

For each file:

1. Read the class top to bottom;
    note every field,
    every method,
    every
   place `this` is captured by a callback.
1. Draft the factory shape (`type Foo = Readonly<{ ... }>` then
   `function createFoo(): Foo`).
1. Move fields into closure-scoped `const`s.
1. Rewrite each method as a named function inside the factory body.
1. Replace `this.foo(...)` with `foo(...)` and `this.#bar` with `bar`.
1. Return `Object.freeze({ ... })`.
1. Update all call sites:
    `new Foo(...)` becomes `createFoo(...)`.
1. Run the package's tests:
    `mise run //package/<path>:test:unit`.
1. Run the package's lint:
    `mise run //package/<path>:lint`.
1. Commit (one logical unit per `AGENTS.md`).

## Escape hatch

When a class genuinely cannot be migrated (e.g. a third-party API mandates
a class shape via decorator metadata),
 wrap the declaration with a scoped
disable plus justification:

```ts
/* oxlint-disable-next-line no-restricted-syntax/no-class -- <constraint>;
   factory shape rejected by <library> because <reason> */
export class Surface extends ThirdPartyBase {
  /* ... */
}
```

The disable applies to the literal next physical line only;
 place it
immediately above the `class` keyword,
 not above the TSDoc block.

## Verified scope

Counts produced by running `mise run //<pkg>:lint:oxlint` against each
package below,
 sum = 15 remaining caught warnings plus 1 unlinted class (BinaryReader):

- `package/watch-restart/cli`:
   5 (`Watcher`,
   `Child`,
   `HashCache`,
   `FakeChild` x2)
- `package/pi-plugin/morph-compact`:
   5 (`MorphCompactClient`,
   `EnvRestore`,
   `FileDisposable`,
   `TcpServerDisposable`,
   `SocketServerDisposable`)
- `package/done/app`:
   1 (`AutofillManager`)
- `package/done/variant-postcss`:
   1 (`AutofillController`)
- `package/zip-writer/core`:
   1 (`ZipWriter`)
- `package/test-harness/core`:
   1 (`Greeter`)
- `package/webapp-forge/server`:
   1 (`DisposableServer`)
- `package/figma/kiwi`:
   1 (`BinaryReader`,
   lint-blind)

Existing `*Error` classes in `package/toml-edit/core/src/errors.ts` pass
via the suffix allowlist and are not in scope.
 The 32 web-component
classes across `editord` and `webapp-productivity` pass via `Element`
on the superclass name.

## Cutover

After every file in the inventory above is migrated or has a justified
disable comment:

1. Run `mise run lint` from the repo root and confirm zero warnings from
   `no-restricted-syntax/no-class`.
1. Edit `package/config/oxlint/src/rule/restriction.ts`:
    change
   `'no-restricted-syntax/no-class': 'warn'` to `'error'`.
    Remove the
   `MIGRATION.no-class.md` reference in the comment.
1. Run `mise run lint; mise run buildAndTest` to confirm no regression.
1. Delete this file in the same commit.
1. Update `AGENTS.md` to encode the rule alongside the other syntax
   bans in the "TypeScript" section.

## Suffix configuration

If a future case needs an additional allowlist suffix (e.g. a project
adopts `EventTarget` subclassing as a pattern),
 override the rule's
options in `package/config/oxlint/src/rule/restriction.ts`:

```ts
'no-restricted-syntax/no-class': [
  'error',
  { suffixes: ['Error', 'Element', 'EventTarget', ], },
],
```

Per-package overrides go in `package/config/oxlint/src/overrides.ts`
using the same shape.
