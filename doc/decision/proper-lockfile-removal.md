# Replace proper-lockfile with workspace shim

## Context

`proper-lockfile@4.1.2` (moxystudio/node-proper-lockfile,
 last commit 2021-01) enters the resolved graph as a transitive of `@earendil-works/pi-coding-agent@0.74.0`.
The package is abandoned and the workspace's dependency policy (`doc/dependency-blocklist.md`) targets such transitives
for removal through one of four mechanisms:
 throwing stub,
 silent stub,
 pure removal,
 or API-compatible shim.

The decision below selects the API-compatible shim path.
The two alternatives (silent stub and pure removal) both fail under at least one code path
that pi-coding-agent exercises during normal startup,
 and the workspace silent stub additionally has a thenable trap that hangs the async lock path.

## First-party callers

`rg -n -e "AuthStorage|SettingsManager|FileAuthStorageBackend|InMemoryAuthStorageBackend|proper-lockfile|lockfile" /var/home/user/Monochromatic/package/` returns zero matches.
Every `@earendil-works/pi-coding-agent` import across the workspace is `import type { ... }`:

- `package/pi-plugin/auto-mode/src/index.ts:12-17`:
   `import type { ExtensionAPI, ExtensionContext, ToolCallEvent, ToolCallEventResult }`
- `package/pi-plugin/auto-mode/src/{ask-user.ts:14, budget-model.ts:15, budget-model-auth.ts:13, context.ts:18, evaluate.ts:16, index.unit.test.ts:8, signals.ts:15, tool-helpers.ts:18}`:
   type-only
- `package/pi-plugin/morph-compact/src/{compaction.ts, compaction-handler.ts, compress-branch.ts, file-tracking.ts, formatting.ts, index.ts, ipc-launch.ts, types.ts}`:
   type-only
- `package/pi-plugin/morph-compact/src/{compress-branch.unit.test.ts, file-tracking.unit.test.ts, formatting.unit.test.ts}`:
   type-only
- `package/pi-plugin/terminal-title/src/{index.ts:27, index.unit.test.ts:13}`:
   type-only

All three workspace pi packages declare `@earendil-works/pi-coding-agent` only under `peerDependencies` +
`devDependencies`,
 never `dependencies` (see `package/pi-plugin/auto-mode/package.json:26-37`).

No first-party source constructs `AuthStorage`,
 `FileAuthStorageBackend`,
 `InMemoryAuthStorageBackend`,
 or `SettingsManager`,
 and no first-party source calls `lockfile.lockSync` or `lockfile.lock`.

## pi-coding-agent runtime callers

The package is consumed at runtime because `package/pi-plugin/auto-mode` ships at `dist/final/node/index.mjs`
and is loaded by the pi CLI binary (the host).
The relevant call sites inside the installed `@earendil-works/pi-coding-agent@0.74.0` dist:

- `dist/main.js:377`:
   `const startupSettingsManager = SettingsManager.create(cwd, agentDir);`
- `dist/main.js:408`:
   `const authStorage = AuthStorage.create();`
- `dist/core/sdk.js:90,92`:
   defaulted construction via `?? AuthStorage.create(authPath)` and `?? SettingsManager.create(cwd, agentDir)`
- `dist/core/agent-session-services.js:56,57`:
   same defaulting pattern
- `dist/core/resource-loader.js:121`:
   `this.settingsManager = options.settingsManager ?? SettingsManager.create(this.cwd, this.agentDir);`
- `dist/package-manager-cli.js:304,358`:
   `SettingsManager.create(cwd, agentDir)`
- `dist/core/model-registry.js:519`:
   `const apiKeyFromAuthStorage = await this.authStorage.getApiKey(model.provider, { includeFallback: false });`

`SettingsManager.create` reaches `FileSettingsStorage` -> `withLock` (sync) -> `lockfile.lockSync(...)` at
`settings-manager.js:44`,
 but only when the settings file already exists (`settings-manager.js:69`).
`AuthStorage.getApiKey` reaches `FileAuthStorageBackend` -> `withLockAsync` -> `lockfile.lock(...)` at
`auth-storage.js:88` and `await release()` at `auth-storage.js:116`.

Both the sync and async lock paths are exercised during normal pi startup,
 not only during OAuth refresh.

## Silent-stub semantics

`package/pnpm-override/stub-silent/index.cjs` is a `Proxy`
over a no-op function whose `get` trap returns `module.exports` for every property;
 including `then`.
That makes the stub a thenable:
 `await stub` enters the Promise resolution machinery,
 calls `stub.then(resolve, reject)`,
 the `apply` trap returns the stub (neither callback is invoked),
 and the await never settles.
The Proxy is callable,
 `apply`,
 `construct`,
 and `get` are all wired,
 so `lockfile.lockSync(path, opts)` and `lockfile.lock(path, opts)` both return the stub itself.

Consequence for the pi-coding-agent chain:

- Sync path (`SettingsManager` startup load):
   `release = lockfile.lockSync(...)` assigns the Proxy;
   `release()` in finally returns the Proxy;
   no crash.
- Async path (`AuthStorage.getApiKey` during model resolution):
   `release = await lockfile.lock(...)` hangs on the thenable trap.
  pi hangs on startup model lookup.

A `then` carve-out in `package/pnpm-override/stub-silent/` would fix this,
 but the current implementation does not have one.
Editing the silent stub to add the carve-out for one consumer weakens the stub contract for every other policy entry,
 and is out of scope here.

## Ranked decision

1. **API-compatible shim (recommended).
   **
   Real implementation of `lockSync(path, options)`
   and `lock(path, options)` via `node:fs.mkdirSync({ recursive: false })` on a sibling `.<basename>.lock` directory.
   Returns a release function (sync from `lockSync`,
    the same sync function from `lock`).
   Both sync and async startup paths execute correctly,
    including `await release()`.
   Decoupled from the silent stub's thenable trap.
   Matches the precedent at `package/pnpm-override/shim-node-domexception/` and `package/pnpm-override/shim-readable-stream/`.

   - Pros:
      every existing call site keeps working;
      no latent hang;
      future pi versions that exercise additional `AuthStorage`/`SettingsManager` methods continue to work.
   - Cons:
      five new files plus a `tsconfig.json`;
      the shim's `mkdirSync`-based locking is simpler than upstream (no stale-file detection,
      no `onCompromised` callback,
      no retry-with-jitter),
      so it does not protect against crashed pi processes leaving stale lock dirs.

2. **Silent stub.
   **
   Add `proper-lockfile` to `.pnpmfile.mjs` `POLICY` with `action: 'silent'`.
   Works for the sync path but hangs the async path under `await release` and under `await lockfile.lock(...)`
   because the workspace stub's Proxy treats every property access (including `.then`) as a self-return.

   - Pros:
      one-line change,
      no new package.
   - Cons:
      hangs pi on every startup that resolves a model API key (`model-registry.js:519` is on the hot path,
      not just OAuth refresh).

3. **Removal**
   (`pnpm-workspace.yaml` `overrides: { 'proper-lockfile': '-' }`
or parent-scoped `'@earendil-works/pi-coding-agent>proper-lockfile': '-'`).
   Causes `auth-storage.js:12` and `settings-manager.js:4` to throw `MODULE_NOT_FOUND` at module load.
   Since `dist/index.js:6,22` statically re-exports both,
    pi-coding-agent itself fails to load.
   The runtime extension load chain (`package/pi-plugin/auto-mode` -> host pi -> `pi-coding-agent/dist/index.js`)
   breaks.

   - Pros:
      smallest possible install footprint.
   - Cons:
      not viable;
      crashes pi.

Ranking:
 1 > 2 > 3.

- 1 > 2 because the silent stub hangs the async lock path on a startup-hot code path (`model-registry.js:519`),
   and the workspace silent stub does not carve out `.then`.
- 2 > 3 because the silent stub at least allows module-load to succeed;
   pure removal crashes pi-coding-agent before any first-party code runs.

## Shim package layout

Package root:
 `package/pnpm-override/shim-proper-lockfile/`.

- `package.json`:
   private,
   `"type": "commonjs"`,
   name `@monochromatic-dev/pnpm-override-shim-proper-lockfile`,
   `main: "./index.cjs"`,
   `types: "./index.d.cts"`,
   LGPL-3.0-or-later (matches workspace shim precedent,
   not upstream MIT).
- `index.cjs`:
   the API-compatible replacement source.
  Acquires the lock by atomic `mkdirSync`;
   releases by `rmdirSync`.
  `lockSync` throws `ELOCKED` on first conflict (the upstream contract).
  `lock` retries internally per `options.retries` (number or object form) with exponential backoff
  from `minTimeout` to `maxTimeout`,
   no jitter.
- `index.d.cts`:
   declares the upstream's exported shape with `export = lock` and a merged namespace.
- `mise.toml`:
   inherits the standard `lint`,
   `lint:oxlint`,
   `lint:types` tasks.
- `tsconfig.json`:
   extends `@monochromatic-dev/config-typescript`;
   `include: ["index.d.cts"]`.
- `README.md`:
   one paragraph stating what the shim replaces,
   the API contract,
   the upstream features intentionally omitted,
   and the cross-references.

The `module.exports` shape mirrors the upstream pattern:

```js
module.exports = lock;
module.exports.lock = lock;
module.exports.lockSync = lockSync;
```

The function-form export (rather than `module.exports = { lockSync, lock }`) survives a future caller doing
`lockfile(...)` directly.
Upstream uses this pattern and we follow it for maximum compatibility.

## Simplifications versus upstream

- **No `fs.realpath`.
  **
  Both pi-coding-agent callsites always pass `{ realpath: false }`
  (verified at `auth-storage.js:38` and `settings-manager.js:44`);
   the shim treats every target as already-resolved through `path.resolve`.
- **No stale-lock detection.
  **
  Upstream rewrites a sentinel file inside the lock dir on a `stale` interval
  and removes the lock if no holder is alive.
  The shim has no equivalent.
  If pi crashes while holding the lock,
   the next pi invocation throws ELOCKED until the user removes `<agentDir>/.auth.json.lock`
   or `<agentDir>/.settings.json.lock` manually.
- **No `onCompromised` callback.
  **
  The option is accepted but never invoked;
   pi-coding-agent's `auth-storage.js` reads the callback's effect via a `lockCompromised` flag that stays false,
   so its `throwIfCompromised()` checks are no-ops.
- **No `retries.randomize`.
  **
  Backoff is strictly exponential.
  The workspace does not run concurrent pi instances,
   so deterministic backoff is acceptable.

The sync/async retry asymmetry mirrors the upstream contract:

- `lockSync` throws `ELOCKED` immediately on first conflict;
   callers handle retries themselves
   (`auth-storage.js:32-54` and `settings-manager.js:38-60` both wrap `lockSync` in a 10-attempt sync retry loop
with a 20ms busy-wait).
- `lock` retries internally per `options.retries` per the async callsite at `auth-storage.js:88`.

## pnpm-workspace.yaml override

A single line in the `overrides` block:

```yaml
overrides:
  'proper-lockfile': 'link:package/pnpm-override/shim-proper-lockfile'
```

The `link:` protocol is path-based,
 so pnpm substitutes the symlinked path regardless of which version a transitive dependent declared,
 identical to how the existing `'node-domexception': 'link:package/pnpm-override/shim-node-domexception'` line handles every upstream
 `node-domexception` request.
No version-qualified entry is needed.

The entry is placed adjacent to the existing `node-domexception`
and `readable-stream` shim entries inside `overrides` (between them,
 in alphabetical position).

No `.pnpmfile.mjs` change.
The `POLICY` object in `.pnpmfile.mjs` covers `throw` and `silent` only and is the wrong home for shim substitution.

## Verification plan

After wiring (`pnpm-workspace.yaml` override added,
 shim package created,
 `pnpm install` run outside sandbox per project convention):

1. **Install reshapes the graph.
   **
   `mise run prepare:pnpm:install` outside sandbox.
   `pnpm why proper-lockfile` should show resolution through `link:package/pnpm-override/shim-proper-lockfile`,
    not `proper-lockfile@4.1.2`.
   The `node_modules/.pnpm/proper-lockfile@*` directory should no longer exist.
2. **Shim loads and exports the right shape.
   **
   `node -e "const l = require('@monochromatic-dev/pnpm-override-shim-proper-lockfile'); console.log(typeof l.lockSync, typeof l.lock);"` -> `function function`.
3. **Sync lock works and `release()` cleans up.
   **
   Probe:
    write a target file,
    call `lockSync(p)`,
    check the `.<basename>.lock` directory exists,
    call `release()`,
    check the directory is gone.
4. **Async lock works under `await release()`.
   **
   Probe:
    write a target file,
    `release = await lock(p, { retries: 0 })`,
    `await release()`.
   Expect `typeof release === 'function'` and clean release.
5. **ELOCKED is thrown when the lock is held.
   **
   Probe:
    call `lockSync(p)` twice;
    second call should throw with `err.code === 'ELOCKED'`.
6. **pi-coding-agent loads end-to-end.
   **
   `node -e "import('@earendil-works/pi-coding-agent').then(m => console.log('AuthStorage:', typeof m.AuthStorage, 'SettingsManager:', typeof m.SettingsManager));"`.
   Expect `AuthStorage: function SettingsManager: function`.
   This proves the shim's resolution unblocks pi-coding-agent's module load,
    but does not exercise `withLockAsync`.
7. **End-to-end via the pi CLI.
   **
   `pi --help` should print help text and exit 0.
   A hang indicates the async lock path is still failing.

Caveat:
 `settings-manager.js:69` only acquires the lock when the settings file already exists,
 so on a host that has never run pi before,
 the lock acquisition path is not exercised by `pi --help`.
To force the lock path on a fresh machine:
 run `pi` once interactively or seed `<agentDir>/settings.json` with `{}`.

## Risk callouts

1. **Crashed pi leaves stale lock dir.
   **
   Without stale detection,
    the next pi invocation throws ELOCKED until the user `rmdir`s the lock directory.
   Manual recovery is acceptable in single-instance workflows;
    document in the README so the operator knows where to look.
2. **Concurrent pi instances are not supported.
   **
   Upstream's locking provides cross-process safety with retry+jitter;
    the shim provides only the EEXIST atomic primitive.
   Multiple concurrent pi instances would serialise correctly (mkdirSync is atomic),
    but the deterministic backoff could synchronise their retries into a livelock.
   The workspace does not run multiple concurrent pi instances;
    if that changes,
    revisit the shim.
3. **NFS-mounted home directories.
   **
   The upstream's `realpath` option exists partly to normalize symlinks before locking on NFS.
   The shim never calls `realpath`,
    so on an NFS mount where `~/.pi` is a symlink,
    the lock directory is created at the symlink's target path.
   Same effective behaviour as upstream-with-`realpath:false`,
    which is what pi-coding-agent always passes.
   Informational;
    no action required.
4. **Future pi-coding-agent versions may use `unlock`/`unlockSync`/`check`/`checkSync`.
   **
   The shim does not export these.
   If a future version adds calls to them,
    module-load succeeds (since `import lockfile from "proper-lockfile"` only fails if the package is missing,
    not if specific named functions are absent) but the call site throws `TypeError: lockfile.unlock is not a function`.
   When that happens,
    add the missing functions to the shim and re-pin the audit entry.
