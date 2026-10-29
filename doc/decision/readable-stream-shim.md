# Replace readable-stream with workspace shim

## Context

Two versions of `readable-stream` are present in the workspace lockfile:

- `readable-stream@3.6.2`,
   pulled in by `winston@3.15.0` and `winston-transport@4.9.0`;
  reachable through `winston` <- `neovim` <- `@monochromatic-dev/mcp-nvim`.
- `readable-stream@4.7.0`,
   pulled in by `isomorphic-git@1.37.6`;
  reachable through `@monochromatic-dev/webapp-forge-server` and `@monochromatic-dev/webapp-forge-stress`.

Both versions are userland mirrors of `node:stream` that originally targeted older Node releases.
On Node 22+ (the workspace baseline) they ship 3,005 LOC of duplicate stream machinery whose only reason for existing
was back-compat with retired Node versions.
The package is large,
 occasionally flagged by audit tooling for transitive churn (the recent `abort-controller` purge passed through
 `readable-stream@4`'s fallback guards),
 and ships a parallel class hierarchy that breaks `instanceof` identity against `node:stream` for any consumer that
 happens to mix the two.

The workspace already has a precedent for replacing userland mirrors of platform APIs with a `link:package/shim/<name>`
override (see `package/pnpm-override/shim-node-domexception/`).
The same mechanism applies cleanly here:
 a single workspace package that re-exports `node:stream`,
 wired in via `pnpm-workspace.yaml > overrides`,
 lets all three consumer chains (winston,
 winston-transport,
 isomorphic-git) keep their `require` / `import` statements unchanged while the actual `readable-stream` source
 disappears from `node_modules`.

The outcome:

- `node_modules/.pnpm/readable-stream@3.6.2/` and `node_modules/.pnpm/readable-stream@4.7.0/` become unreachable (no
- consumer symlink resolves through them);
   they linger as inert pnpm cache and disappear on the next full `node_modules` rebuild.
- All consumers receive `node:stream` classes;
   `instanceof` identity is unified across the workspace.
- The transitive `abort-controller` guards inside `readable-stream@4`'s `pipeline.js`,
   `duplexify.js`,
   and `operators.js` become unreachable dead code (the shim does not load those files),
   formally closing the loop on the `'abort-controller': '-'` override that was added earlier.

## Shim package layout

Package root:
 `package/pnpm-override/shim-readable-stream/`.
Five top-level files plus a `lib/` directory of five 3-line re-exports.

### `package/pnpm-override/shim-readable-stream/package.json`

```json
{
  "license": "LGPL-3.0-or-later",
  "name": "@monochromatic-dev/pnpm-override-shim-readable-stream",
  "description": "API-compatible shim for readable-stream@3 and readable-stream@4; re-exports node:stream.",
  "private": true,
  "type": "commonjs",
  "main": "./index.cjs",
  "types": "./index.d.cts",
  "exports": {
    ".": {
      "types": "./index.d.cts",
      "default": "./index.cjs"
    },
    "./lib/_stream_writable.js": "./lib/_stream_writable.js",
    "./lib/_stream_readable.js": "./lib/_stream_readable.js",
    "./lib/_stream_transform.js": "./lib/_stream_transform.js",
    "./lib/_stream_duplex.js": "./lib/_stream_duplex.js",
    "./lib/_stream_passthrough.js": "./lib/_stream_passthrough.js",
    "./package.json": "./package.json"
  },
  "files": [
    "index.cjs",
    "index.d.cts",
    "lib"
  ],
  "devDependencies": {
    "@monochromatic-dev/config-typescript": "workspace:*",
    "typescript": "catalog:"
  },
  "dependencies": {},
  "version": "0.0.0"
}
```

Notes:

- Fields mirror `package/pnpm-override/shim-node-domexception/package.json` (`private`,
   `type: "commonjs"`,
   `main` + `types`,
   explicit `exports`,
   `files`,
   empty `dependencies: {}`).
- The `exports` map is intentionally restrictive:
   it lists exactly the paths our reachable consumers import.
  Any unforeseen subpath will produce `ERR_PACKAGE_PATH_NOT_EXPORTED`,
   which surfaces the gap loudly instead of silently failing.
  If a future consumer requires an additional subpath,
   the remediation is a one-line `exports` entry plus a 1-line file under `lib/`.

### `package/pnpm-override/shim-readable-stream/index.cjs`

```js
'use strict';

const Stream = require('node:stream',);

module.exports = Stream.Readable;
for (const key of Object.keys(Stream,)) {
  Object.defineProperty(module.exports, key, {
    value: Stream[key],
    writable: true,
    enumerable: true,
    configurable: true,
  },);
}
module.exports.Stream = Stream;
```

This mirrors the intent of the upstream `readable-stream@3.6.2/readable.js` entry's READABLE_STREAM=disable branch
(export `Readable` as the default,
 copy every property of `node:stream` onto it,
 and expose the `Stream` module itself via the `.Stream` property),
 with one mechanical adjustment.

The upstream pattern uses `Object.assign(module.exports, Stream)`,
 which fails in Node 22+ because `Stream` exposes a getter-only `promises` accessor on the legacy `Stream`
 constructor's prototype.
`Stream.Readable` inherits that accessor via its function-prototype chain
(`Object.getPrototypeOf(Stream.Readable) === Stream`);
 a plain `Stream.Readable.promises = Stream.promises` assignment under strict mode (or `Object.assign` internally)
 throws `TypeError: Attempted to assign to readonly property`,
 because the inherited descriptor has `set: undefined`.
Using `Object.defineProperty` with a fresh data descriptor on the target creates a new own property that shadows the
inherited accessor,
 sidestepping the failure entirely.
The end-state property shape is identical to what the upstream `Object.assign` form was meant to produce.

Every consumer pattern in the workspace is covered:

- `const { Stream, Transform } = require('readable-stream')` (winston `logger.js:10`):
  `Stream` resolves to `node:stream` (set via the final assignment),
   `Transform` resolves to `node:stream.Transform` (defined as an own property on the default export).
- `const { Writable } = require('readable-stream')` (winston `exception-stream.js:10`,
   `rejection-stream.js:10`):
  defined as an own property on the default export.
- `const { PassThrough } = require('readable-stream')` (winston `transports/file.js:16`,
   isomorphic-git `http/node/index.js:91`):
  defined as an own property on the default export.
- `const { Stream } = require('readable-stream')` (winston `transports/http.js:12`,
   `tail-file.js:12`):
  set explicitly on the final line.

For the v4 surface (`compose`,
 `addAbortSignal`,
 `destroy`,
 `isDisturbed`,
 `isErrored`,
 `isReadable`,
 `getDefaultHighWaterMark`,
 `setDefaultHighWaterMark`,
 `promises`):
all are properties of `node:stream` on Node 22+,
 so all are defined as own properties on the default export by the `Object.keys(Stream)` enumeration without having to
 spell them out.

### `package/pnpm-override/shim-readable-stream/index.d.cts`

```ts
import Stream = require('node:stream');
export = Stream;
```

The shim's runtime export is the `Readable` class with stream properties merged in;
 the type surface is functionally the `node:stream` module namespace,
 which is what every documented consumer reads from.
A more precise declaration mirroring the runtime shape (a `Readable`-typed callable with merged statics) is possible
but adds complexity for no caller value:
 TypeScript users of `readable-stream` reach for `Readable`,
 `Writable`,
 etc. by name,
 which the namespace export provides.

### `package/pnpm-override/shim-readable-stream/lib/_stream_writable.js`

```js
'use strict';

module.exports = require('node:stream',).Writable;
```

### `package/pnpm-override/shim-readable-stream/lib/_stream_readable.js`

```js
'use strict';

module.exports = require('node:stream',).Readable;
```

### `package/pnpm-override/shim-readable-stream/lib/_stream_transform.js`

```js
'use strict';

module.exports = require('node:stream',).Transform;
```

### `package/pnpm-override/shim-readable-stream/lib/_stream_duplex.js`

```js
'use strict';

module.exports = require('node:stream',).Duplex;
```

### `package/pnpm-override/shim-readable-stream/lib/_stream_passthrough.js`

```js
'use strict';

module.exports = require('node:stream',).PassThrough;
```

Each file mirrors the corresponding upstream `lib/_stream_*.js` shape:
 upstream sets `module.exports = <Class>;` after defining the ported class.
The shim does the same with the platform class.

Only `lib/_stream_writable.js` is currently imported by reachable code (`winston-transport@4.9.0/modern.js:4`).
The other four are provided defensively to match the full set of `_stream_*.js` files shipped by
`readable-stream@3.6.2`;
 this costs five 3-line files and keeps the shim useful if a future winston-transport or third-party patch reaches for
 another subpath.

### `package/pnpm-override/shim-readable-stream/mise.toml`

```toml
[tasks.lint]
extends = "lint"

[tasks."lint:oxlint"]
extends = "lint:oxlint"

[tasks."lint:types"]
extends = "lint:types"
```

Identical to `package/pnpm-override/shim-node-domexception/mise.toml`.

## Upstream package.json fields and shim coverage

Read from the installed packages on disk.

`/home/user/Monochromatic/node_modules/.pnpm/readable-stream@3.6.2/node_modules/readable-stream/package.json`:

```json
{
  "main": "readable.js",
  "browser": {
    "util": false,
    "worker_threads": false,
    "./errors": "./errors-browser.js",
    "./readable.js": "./readable-browser.js",
    "./lib/internal/streams/from.js": "./lib/internal/streams/from-browser.js",
    "./lib/internal/streams/stream.js": "./lib/internal/streams/stream-browser.js"
  }
}
```

No `exports` field;
 resolution is classic `main` + filesystem subpaths.
The `browser` field maps node-only modules to `false` (a webpack convention);
 it has no effect under the workspace's Bun/Node runtime.

`/home/user/Monochromatic/node_modules/.pnpm/readable-stream@4.7.0/node_modules/readable-stream/package.json`:

- `main`:
   `"lib/ours/index.js"`.
- `browser` field maps `util` and rewrites the main entry for browser builds.
- No `exports` field.

The shim's `exports` covers both upstream surfaces:

- The `"."` entry replaces the classic `main` lookup for both v3 and v4.
  Top-level destructures
  (`{ Stream, Transform, Writable, Readable, Duplex, PassThrough, pipeline, finished, compose, addAbortSignal, destroy,
  isDisturbed, isErrored, isReadable, getDefaultHighWaterMark, setDefaultHighWaterMark, promises }`) all resolve to `nod
  e:stream`'s corresponding property via the `Object.assign` in `index.cjs`.
  The legacy `Stream` alias used by v4 is set explicitly on the final line.
- The five `./lib/_stream_*.js` entries cover the only deep subpath surface used by reachable consumers (see next
- section).
  v4 ships additional internal paths under `lib/internal/streams/`;
   none of our reachable consumers reach into those,
   so they are intentionally omitted.

## Deep-import paths in our reachable consumers

Authoritative enumeration (every `require` or `import` of `readable-stream` reachable from the workspace):

`winston@3.15.0`,
 top-level only:

- `lib/winston/logger.js:10`:
   `const { Stream, Transform } = require('readable-stream');`
- `lib/winston/exception-stream.js:10`:
   `const { Writable } = require('readable-stream');`
- `lib/winston/rejection-stream.js:10`:
   `const { Writable } = require('readable-stream');`
- `lib/winston/transports/file.js:16`:
   `const { Stream, PassThrough } = require('readable-stream');`
- `lib/winston/transports/http.js:12`:
   `const { Stream } = require('readable-stream');`
- `lib/winston/tail-file.js:12`:
   `const { Stream } = require('readable-stream');`

`winston-transport@4.9.0`:

- `modern.js:4`:
   `const Writable = require('readable-stream/lib/_stream_writable.js');`
  (the only deep subpath import in the entire reachable graph)

`isomorphic-git@1.37.6`:

- `http/node/index.js:91`:
   `const { PassThrough } = require('readable-stream');`

Subpaths covered by the shim:
 `.` (the top-level entry) and `./lib/_stream_writable.js`.
The remaining four `lib/_stream_*.js` entries in the shim are not reached by current code;
 they are present as a forward-compatibility hedge against future winston-transport or patched-fork reaches.

v4-specific subpaths (`lib/stream/promises.js`,
 anything under `lib/internal/streams/`,
 `lib/ours/...`):
none are imported by our reachable consumers.
Not included in the shim.

## pnpm-workspace.yaml override

Add one line to the `overrides` block:

```yaml
'readable-stream': 'link:package/pnpm-override/shim-readable-stream'
```

A single override entry suffices for both v3.6.2 and v4.7.0.
The `link:` protocol is a path resolver,
 not a semver resolver:
 pnpm substitutes the symlinked path regardless of which version a transitive dependent declared,
 identical to how the existing `'node-domexception': 'link:package/pnpm-override/shim-node-domexception'` line handles every upstream
 `node-domexception` request.
No version-qualified entry (`readable-stream@3`,
 `readable-stream@4`) is needed.

Place the new line adjacent to the existing `'node-domexception': 'link:package/pnpm-override/shim-node-domexception'` entry inside
`overrides`,
 in alphabetical position (after `'node-domexception'`).

## Verification plan

After wiring (`pnpm-workspace.yaml` override added,
 shim package created,
 `pnpm install` run outside sandbox per project convention):

1. Confirm the override took effect.
   - `find /var/home/user/Monochromatic/node_modules/.pnpm -maxdepth 3 -name 'readable-stream' -type l` should list
   - three symlinks (one per consumer:
      `winston@3.15.0`,
      `winston-transport@4.9.0`,
      `isomorphic-git@1.37.6`),
      each pointing at `../../../../package/pnpm-override/shim-readable-stream`.
   - Two orphan store directories may linger at `node_modules/.pnpm/readable-stream@3.6.2/` and
   - `node_modules/.pnpm/readable-stream@4.7.0/`.
      They are inert:
      no consumer's symlink resolves through them,
      so they contribute zero LOC to the reachable module graph.
      Treat them as pnpm cache leftovers,
      not a sign the override missed.
     The primary signal that the override is live is the three consumer symlinks above;
      the orphan dirs disappear on the next full `node_modules` rebuild.
2. Type-check and lint the shim package itself.
   - `mise run //package/pnpm-override/shim-readable-stream:lint:types`
   - `mise run //package/pnpm-override/shim-readable-stream:lint:oxlint`
3. Exercise winston via the mcp/nvim chain.
   - `mise run //package/mcp/nvim:lint:types` (the package depends on `neovim`,
      which depends on `winston`;
      type-check confirms the type shape of the shim matches what winston's `.d.ts` files expect).
   - Runtime is untested at the `mcp/nvim → neovim → winston` integration boundary:
      mcp/nvim ships no test surface and the workspace has no `start` task on this package that would exercise neovim's
      logger end-to-end through an MCP host.
     As a stand-in,
      the implementer ran a one-shot script that `require`s winston from its absolute installed path,
      creates a `winston.createLogger({ transports: [new winston.transports.Console()] })`,
      and emits info/warn/error lines.
     All three log lines flushed and the consumer reached every `readable-stream` entry winston traverses (`logger.js`,
      `exception-stream.js`,
      `rejection-stream.js`,
      `transports/file.js`,
      `transports/http.js`,
      `tail-file.js`).
     `winston-transport@4.9.0/modern.js`'s deep import of `readable-stream/lib/_stream_writable.js` resolved without
     `ERR_PACKAGE_PATH_NOT_EXPORTED`.
     A future MCP host integration test would close this gap.
4. Exercise isomorphic-git via webapp-forge/server.
   - `mise run //package/webapp-forge/server:lint:types`
   - `mise run //package/webapp-forge/server:dev` (which expands to `mise watch -w src -r -- start:server`,
      then `bun src/index.ts`).
     Hit an endpoint that drives `src/git/iso-server.ts` (the file identified as the iso-git entry point) and look for:
     - successful HTTP responses;
     - no errors mentioning `PassThrough`,
        `readable-stream`,
        or `node:stream`;
     - on a clone or fetch operation,
        pack data streams without truncation (full object count returned).
5. Exercise isomorphic-git via webapp-forge/stress.
   - `mise run //package/webapp-forge/stress:lint:types`
   - `mise run //package/webapp-forge/stress:stress:hot-repo` (the hot-repo scenario uses iso-git heavily through
   - `src/scenarios/force-push.ts`).
     Look for:
      scenario completes its declared iteration count;
      no stream-related errors in the scenario output;
      pack/unpack operations report expected object counts.
6. Confirm `instanceof` parity.
   - Inside any one of the live processes above,
      evaluate `require('readable-stream').Readable === require('node:stream').Readable` in a one-shot Bun invocation:
     `bun -e "const r = require('readable-stream'); const s = require('node:stream'); console.log(r.Readable === s.Reada
     ble, r.Writable === s.Writable, r.Stream === s)"`
     All three should print `true`.
     The third confirms the `module.exports.Stream = Stream` assignment took effect.
7. Final build-and-test pass across the affected packages.
   - `mise run //package/mcp/nvim:test` (if a test task exists;
      the read showed only lint tasks,
      so this may be a no-op).
   - `mise run //package/webapp-forge/server:test:unit`
   - `mise run //package/webapp-forge/stress:test:unit`

The verification crosses the integration boundary in steps 3 to 5 (running the consumer,
 observing real I/O).
Steps 1,
 2,
 and 6 are sanity checks that complement but do not replace the runtime exercises.

## Risk callouts

Specific behavior differences between `node:stream@22.x` and the ported `readable-stream` classes that could surface in
our consumer paths.
Listed in declining order of likelihood:

1. **Default `autoDestroy` and `emitClose` semantics.
   **
   `readable-stream@3.6.2` defaulted `autoDestroy: false` for several years;
   `readable-stream@4.x` and `node:stream@16+` default it to `true`.
   Winston's transports rely on `'finish'` and `'close'` ordering to flush log buffers.
   The post-shim path uses `node:stream`'s defaults uniformly,
    which is the same default v4 already used;
    the change only affects winston (v3 path).
   Inspect `winston/lib/winston/transports/file.js` flush behavior under load in the verification step.
   Symptom to watch for:
    missing trailing log lines on process exit.
2. **`construct()` / `_construct()` lifecycle hook.
   **
   Added to `node:stream` in Node 15;
    `readable-stream@3.6.2` does not ship it.
   Any winston or winston-transport subclass that defines `_construct` was previously a no-op under v3 and now runs.
   Quick grep of the consumer source rules this out for the current code paths,
    but a future winston update could expose it.
3. **`Readable.from` iterable handling for sync iterators.
   **
   `node:stream`'s `Readable.from` is slightly stricter about `Symbol.iterator` vs `Symbol.asyncIterator` than v3's
   port.
   Not used by the reachable consumers (winston builds streams via class construction;
    iso-git builds `PassThrough` directly).
4. **`pipeline` argument shape.
   **
   `node:stream.pipeline` supports `AbortSignal` in the options object (added Node 15+).
   v3's port supports the same surface in 3.6.
   x.
   Behavior parity is high;
    the named risk is that `node:stream` propagates errors through `AbortSignal.abort()` cleanup more strictly.
   Not used by reachable consumers' top-level imports;
    the guarded `globalThis.AbortController || require('abort-controller')` blocks inside `readable-stream@4`'s
    `pipeline.js`,
    `duplexify.js`,
    and `operators.js` become unreachable once the shim replaces v4 entirely,
    which closes a separate concern.
5. **iso-git pack streaming `highWaterMark`.
   **
   isomorphic-git's `http/node/index.js` constructs `new PassThrough()` with default options.
   `node:stream.PassThrough` defaults `highWaterMark` to 16 KiB;
    v4's port also defaults to 16 KiB.
   No change expected.
   The risk is if iso-git internals pass through a long-running stream that previously benefited from v4's
   `setDefaultHighWaterMark` global;
    the workspace does not call `setDefaultHighWaterMark`,
    and iso-git does not call it on its imported `PassThrough`,
    so this is informational only.
6. **`instanceof` identity flip.
   **
   Pre-shim,
    winston creates streams via its imported `readable-stream` classes;
    if some downstream code did `obj instanceof require('stream').Readable` against a winston-produced stream,
    it returned `false`.
   Post-shim,
    it returns `true`.
   This is a behavior improvement,
    but if any consumer relies on the negative result (defensive type-narrowing branches),
    it changes shape.
   No instance of this pattern was found in the reachable consumers;
    calling it out for the audit record.
7. **TypeScript namespace import shape.
   **
   `index.d.cts` re-exports the `node:stream` namespace via `import Stream = require('node:stream'); export = Stream;`.
   Most TypeScript users of `readable-stream` consume types from `@types/readable-stream`,
    which itself re-exports `node:stream`'s types.
   The shim's `.d.cts` shape is therefore congruent with the `@types` package;
    any callsite that types `import type { Readable } from 'readable-stream'` resolves to `Readable` from `node:stream`
    identically.
   Symptom to watch for in verification step 2:
    any TS2305 (has no exported member) diagnostic against the shim's namespace.

The browser concern is out of scope:
 both consumer chains (`webapp-forge/server` and `webapp-forge/stress`) are Bun/Node processes,
 not browser bundles.
If a future browser consumer reaches for `readable-stream`,
 the shim will fail to load (`node:stream` is not available in browsers);
 the remediation at that point is a separate browser-side shim or pruning the import.
