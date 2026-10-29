# Dependency blocklist

This repo bans third-party packages through two complementary mechanisms.
Both ban globally (across every workspace package and every level of the dependency graph).
They differ in what lands in `node_modules` after the install.

## Two homes, four outcomes

1. **Generic substitution stub on disk**:
    edit the policy table in `.pnpmfile.policies.json` at the repo root.
   The `readPackage` hook in `.pnpmfile.mjs` reads that table
   and replaces every dependency entry pointing at the blocked package with a workspace stub.
   Two stub kinds exist:
    throwing and silent.

2. **Nothing on disk,
    or an API-compatible shim on disk**:
    edit the `overrides` block in `pnpm-workspace.yaml`.
   pnpm's native `"name": "-"` primitive removes the package from the resolved graph entirely.
   The same block also accepts `link:package/shim/<name>` to substitute the package with a workspace shim that
   re-exports the real API.

The split is intentional.
pnpm has a first-class removal primitive;
 reimplementing it would add code without adding capability.
Generic substitution needs a workspace-aware hook,
 since pnpm's overrides cannot point at a workspace package by aliased name in every version.
Shim substitution uses pnpm's overrides directly because `link:` is a path-based protocol that sidesteps workspace-name
resolution.

## Scope and what is not covered

The `.pnpmfile.mjs` hook walks `dependencies` and `optionalDependencies` on every manifest pnpm parses.
It does not walk `devDependencies` or `peerDependencies`:

- `devDependencies` on a transitive package are not installed by pnpm,
   so iterating them would produce noisy substitution warnings for entries that never reach `node_modules`.
- `devDependencies` on a direct workspace package (`package/<x>/package.json`) are read from disk by pnpm,
   not via the `readPackage` hook,
   so the policy does not see them either.
- `peerDependencies` are not auto-installed in this repo (`pnpm-workspace.yaml` sets `autoInstallPeers: false`);
   substituting them is a no-op.

Practical consequence:
 a workspace package that adds the blocked dependency to its own `devDependencies` slips past the hook entirely.
The policy ban applies to runtime deps and transitive trees,
 not to dev-time-only deps on workspace packages.
To police workspace devDeps,
 add a lint rule (e.g. `eslint-plugin-import/no-restricted-paths`,
 or a custom check that greps each `package.json` against the same name set) and run it in CI.
The substitutions in `pnpm-workspace.yaml`'s `overrides` block (removal,
 parent-scoped removal,
 `link:` shim) are not affected by this scope;
 they apply globally regardless of which dep field a package declares.

## Decision rule

Pick the lightest action that surfaces the problem at the right place.

- **Removal** (edit `pnpm-workspace.yaml`):
   all importers handle a missing module gracefully.
  The common pattern is `try { require('optional-thing') } catch {}` for plugin-style integrations (winston transports,
   passport strategies,
   optional native bindings).
  The catch fires,
   the fallback runs,
   the build stays green.
  For hard `require` or static `import`,
   removal yields `MODULE_NOT_FOUND` at the call site,
   which is loud but uninformative.

- **Throwing stub** (edit `.pnpmfile.mjs`,
   action `throw`):
   at least one importer hard-imports the package,
   or you want a custom error message instead of `MODULE_NOT_FOUND`.
  Loading the stub evaluates a `throw new Error(...)` that names the policy file.
  Optional importers (try/catch) still see the catch fire,
   with your message inside the error.

- **Silent stub** (edit `.pnpmfile.mjs`,
   action `silent`):
   soft migration where you want builds green and accept incorrect-but-not-crashing runtime behavior.
  The stub is a callable Proxy whose property accesses,
   function calls,
   and `new` invocations all return the stub itself.
  `in` checks return `false`.

- **API-compatible shim** (create `package/shim/<name>/`,
   wire via `pnpm-workspace.yaml` `overrides`):
   an importer hard-imports the package and relies on its constructor or function shape,
   the package is deprecated or unwanted as a direct source,
   and the real API is trivially reproducible (e.g. re-exporting a `globalThis.*` value,
   delegating to a native primitive,
   or wrapping a different package).
  Generic stubs break the runtime contract;
   removal leaves the static import dangling.
  A shim package that re-exports the real API keeps consumers working
  while removing the unwanted source from `node_modules`.

If you are unsure,
 prefer the throwing stub.
It collapses to the same observable as removal when importers wrap their `require` in `try/catch`,
 and beats removal when they do not.
Silent is a deliberate trade-off;
 reach for it only when a loud failure would block work you cannot fix today.
The shim is the right call when the real API is trivial and the importer cannot tolerate stub semantics.

## How to add a substitution

Edit the policy table in `.pnpmfile.policies.json`.
The hook in `.pnpmfile.mjs` imports this file and indexes it by package name;
 you do not touch the hook.

```json
{
  "//": "Companion to overrides in pnpm-workspace.yaml. See doc/dependency-blocklist.md.",
  "moment": {
    "action": "throw",
    "reason": "use date-fns or native Intl.DateTimeFormat (see doc/decision/2026-05-no-moment.md)"
  },
  "is-array": {
    "action": "silent",
    "reason": "native Array.isArray; silent substitution while migrating consumers"
  },
  "lodash": {
    "action": "throw",
    "reason": "use native ES + es-toolkit",
    "allowed": ["@monochromatic-dev/webapp-legacy"]
  }
}
```

Keys beginning with `//` are comments (an npm package name can never start with `/`);
 the hook drops them when building its lookup table,
 so use them for rationale JSON cannot express as real comments.
`reason` is surfaced in the install-time warning to stderr.
`allowed` is an optional array of consumer workspace-package names;
 matching consumers keep resolving to the real dependency.

After editing,
 run a clean install (`mise run //:install` or the equivalent) and read stderr for the warnings.
Each `(dependent, blocked, action)` tuple warns once per install.
Because pnpm keys re-resolution off the `.pnpmfile.mjs` checksum and not the imported JSON,
 a data-only edit may not re-trigger resolution on its own;
 see "Verification after adding an entry" for how to force re-resolution.

## How to add a global removal

Edit `pnpm-workspace.yaml`'s `overrides` block:

```yaml
overrides:
  request: '-' # remove globally, no warning
  'consumer-x>request': '8.x' # but consumer-x keeps the real one (parent-scoped allowlist)
```

Removal is silent.
If you want a warning printed during install,
 prefer the throwing stub instead.

## Parent-scoped removal

For dropping a specific transitive child only when imported by a specific parent (e.g. `jspdf>canvg`,
 `@earendil-works/pi-ai>@google/genai`),
 keep using the existing `pnpm-workspace.yaml` pattern.
The 14 existing parent-scoped entries cover surgical cases where the global mechanisms are too broad.
See `TROUBLESHOOTING.dependencies.md` for the audit trail behind each one.

## How to add an API-compatible shim

Create a workspace package under `package/shim/<name>/` with five files (mirror `package/pnpm-override/shim-node-domexception/`):

- `package.json`:
   private,
   `"type": "commonjs"`,
   name `@monochromatic-dev/shim-<name>`,
   `main: "./index.cjs"`,
   `types: "./index.d.cts"`.
- `index.cjs`:
   the API-compatible replacement source.
- `index.d.cts`:
   a minimal type declaration matching the exported shape (use `export = _;` to mirror CJS default export).
- `mise.toml`:
   `extends` the standard `lint`,
   `lint:oxlint`,
   `lint:types` tasks.
- `README.md`:
   one paragraph stating what the shim replaces,
   the API contract,
   and a cross-reference to `TROUBLESHOOTING.dependencies.md`.

Wire the substitution by adding one line to `pnpm-workspace.yaml`'s `overrides` block:

```yaml
overrides:
  '<blocked-name>': 'link:package/shim/<name>'
```

pnpm rewrites every transitive edge that resolved `<blocked-name>` to point at the workspace path;
 the real npm package is no longer installed under `node_modules/.pnpm/`.
No install-time warning is emitted;
 the substitution is silent.
Document the rationale in `TROUBLESHOOTING.dependencies.md` so future readers understand why the shim exists.

## Worked examples

### Throw on a package that ships unwanted polyfills

```json
{
  "array.prototype.flat": {
    "action": "throw",
    "reason": "Node 17+ has Array.prototype.flat natively; replace the import"
  }
}
```

On the next install,
 every package whose manifest declares `array.prototype.flat` produces one stderr line:

```text
[blocked-dep] eslint-plugin-import@2.32.1 -> array.prototype.flat [throw]:
  substituting with stub-throw. Node 17+ has Array.prototype.flat natively;
  replace the import (previous spec: ^1.3.2)
```

If any consumer evaluates `require('array.prototype.flat')`,
 the stub's `index.cjs` throws an error pointing at this doc.

### Silent for a soft migration

```json
{
  "old-feature-flag-client": {
    "action": "silent",
    "reason": "feature-flag client v2 incoming; v1 stubbed during migration"
  }
}
```

Code that does `flagClient.isEnabled('something')` runs without throwing.
`isEnabled` reads back as the stub Proxy,
 which is callable and returns the stub.
Returns are not truthy/falsy in a useful way,
 so feature checks default to "missing";
 verify each call site behaves acceptably before shipping.

### Allowlist a single legacy consumer

```json
{
  "moment": {
    "action": "throw",
    "reason": "use date-fns",
    "allowed": ["@monochromatic-dev/webapp-edu-legacy"]
  }
}
```

The legacy webapp keeps the real `moment`;
 every other workspace package and every transitive dep resolves to the throwing stub.

### Global removal of an optional dep

In `pnpm-workspace.yaml`:

```yaml
overrides:
  fsevents: '-'
```

Every package whose manifest depends on `fsevents` (chokidar and friends) gets it removed.
Linux and Windows installs were already skipping it via `os` constraints;
 the override makes the removal explicit and disk-saving.
Consumers wrapping `require('fsevents')` in try/catch continue working.

### Shim for a deprecated package hard-imported by a transitive consumer

`fetch-blob@3.2.0/from.js:3` does `import DOMException from 'node-domexception'`
and uses it as a constructor at `from.js:86`.
The upstream package is deprecated but its runtime behaviour on Node 17+
and Bun is `module.exports = globalThis.DOMException`,
 which is trivially reproducible.

The shim package lives at `package/pnpm-override/shim-node-domexception/`.
Its `index.cjs` is a single line:

```js
module.exports = globalThis.DOMException;
```

`pnpm-workspace.yaml` substitutes the upstream package globally:

```yaml
overrides:
  'node-domexception': 'link:package/pnpm-override/shim-node-domexception'
```

After install,
 every transitive `node-domexception` edge resolves to the workspace shim.
The deprecation warning is gone.
Consumers (`fetch-blob`,
 `node-fetch`,
 the `@libsql/hrana-client` and `gaxios` subtrees) see the native `DOMException` constructor exactly as before,
 so the `throw new DOMException(...)` paths in `fetch-blob` keep producing real `Error`-derived exceptions.

### Shim for a multi-class library across two upstream versions

`readable-stream` is a userland mirror of `node:stream` that two consumer chains pull in at different major versions:
`winston@3.15.0` and `winston-transport@4.9.0` depend on `readable-stream@3.6.2`,
 while `isomorphic-git@1.37.6` depends on `readable-stream@4.7.0`.
Both versions exist purely for back-compat with retired Node releases;
 on Node 22+ the platform's `node:stream` module covers every API both versions expose.

The shim at `package/pnpm-override/shim-readable-stream/` re-exports `node:stream`:

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

The pattern uses `Object.defineProperty` per key instead of the more idiomatic `Object.assign(module.exports, Stream)`
because `Stream.Readable` inherits a getter-only `promises` accessor via its function-prototype chain
(`Object.getPrototypeOf(Stream.Readable) === Stream`);
 under strict mode (which CommonJS modules run in by default in Node 22+),
 `Object.assign` throws `TypeError: Attempted to assign to readonly property` when it reaches the inherited accessor.
`Object.defineProperty` creates a new own data property that shadows the inherited accessor,
 sidestepping the failure.

The shim also ships five 3-line files under `lib/_stream_*.js` (`_stream_readable.js`,
 `_stream_writable.js`,
 `_stream_transform.js`,
 `_stream_duplex.js`,
 `_stream_passthrough.js`),
 each re-exporting the matching `node:stream` class.
`winston-transport@4.9.0/modern.js:4` deep-imports `require('readable-stream/lib/_stream_writable.js')`;
 the corresponding `exports` entry in `package.json` routes that subpath to the shim's file.

A single override entry covers both upstream versions because `link:` is a path resolver,
 not a semver resolver:

```yaml
overrides:
  'readable-stream': 'link:package/pnpm-override/shim-readable-stream'
```

After install,
 three transitive `readable-stream` edges (`winston@3.15.0/node_modules/readable-stream`,
 `winston-transport@4.9.0/node_modules/readable-stream`,
 `isomorphic-git@1.37.6/node_modules/readable-stream`) symlink to the workspace shim.
The substitution unifies `instanceof` identity across the workspace (every consumer of `readable-stream` now operates
on `node:stream`'s classes directly) and makes the transitive `abort-controller` fallback guards inside
`readable-stream@4`'s `pipeline.js`,
 `duplexify.js`,
 and `operators.js` unreachable,
 formally closing the loop on the earlier `'abort-controller': '-'` removal.

`doc/decision/readable-stream-shim.md` records the full audit,
 including the enumerated consumer surface,
 the verification plan,
 and the behavior risk callouts (default `autoDestroy` flip,
 `_construct` lifecycle,
 `Readable.from` strictness,
 etc.).

## Verification after adding an entry

1. Run install (`mise run prepare:pnpm:install`).
   pnpm v11 stores a `pnpmfileChecksum` line in `pnpm-lock.yaml` computed from the `.pnpmfile.mjs` source;
    editing the hook file changes the checksum and pnpm re-runs resolution automatically.
   The checksum does not follow the imported `.pnpmfile.policies.json`,
    so a data-only edit (the common case now) may leave the checksum unchanged and pnpm may reuse the cached resolution.
   After editing only the JSON,
    change `.pnpmfile.mjs`'s bytes to force re-resolution (the checksum hashes the hook source,
    so a byte change is what re-triggers it).
   `pnpm install --force` alone was observed not to re-run the hook on pnpm 11.6.0 while the checksum was unchanged,
    so the byte change,
    not `--force`,
    is the reliable trigger.
   See `doc/troubleshooting/pnpm-minimum-release-age-exclude-first-match.md` ("What does not work") for the session
   that established this.
2. Read stderr for the `[blocked-dep]` lines.
   Confirm the dependent names you expected appear;
    one warning per `(dependent, blocked, action)` tuple.
3. For a throwing stub,
    exercise the import in a probe script.
   From the repo root:
    `node -e "require('<blocked-pkg>')"`.
   Expect the stub's error message naming this doc.
4. For a silent stub,
    probe with `node -e "console.log(require('<blocked-pkg>'))"`.
   Expect the printed value to look like `Proxy([Function: silentStub])`;
    no throw.
5. For removal,
    probe with `node -e "try { require('<blocked-pkg>') } catch (e) { console.log(e.code) }"`.
   Expect `MODULE_NOT_FOUND`.
6. For a shim,
    probe the API surface that the original importer exercises.
   For a constructor shim:
    `node -e "const X = require('<blocked-pkg>'); const e = new X('msg','Name'); console.log(e.name, e.message, e instanceof Error)"`.
   Also check that the install no longer lists the package as a deprecated subdependency in stderr
   and that `pnpm why <blocked-pkg>` resolves through the workspace link.

## Cross-references

- `.pnpmfile.policies.json` at the repo root:
   the policy data table (sentinel `//` keys carry comments).
- `.pnpmfile.mjs` at the repo root:
   the `readPackage` hook that reads the table and applies substitutions.
- `pnpm-workspace.yaml`'s `overrides` block:
   global removals,
   parent-scoped removals,
   and `link:` substitutions to workspace shims.
- `package/pnpm-override/stub-throwing/`:
   the workspace stub used by `action: 'throw'`.
- `package/pnpm-override/stub-silent/`:
   the workspace stub used by `action: 'silent'`.
- `package/shim/<name>/`:
   workspace shims for API-compatible substitution;
   current entries are `package/pnpm-override/shim-node-domexception/` and `package/pnpm-override/shim-readable-stream/`.
- `TROUBLESHOOTING.dependencies.md`:
   the audit trail for the existing parent-scoped overrides;
   cross-link entries here when a substitution replaces or augments one of those overrides.
- `TROUBLESHOOTING.pnpmfile.md`:
   why the policy implementation is `.pnpmfile.mjs` with JSDoc types rather than `.pnpmfile.ts`;
   source trace,
   maintainer rationale,
   and the rejected pre-strip workaround.
