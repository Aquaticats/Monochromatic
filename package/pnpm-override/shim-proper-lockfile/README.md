# @monochromatic-dev/pnpm-override-shim-proper-lockfile

API-compatible shim for [`proper-lockfile`](https://www.npmjs.com/package/proper-lockfile).
Implements `lockSync(file, options)` and `lock(file, options)` via `node:fs.mkdirSync`
 on a sibling `.<basename>.lock` directory.
`release()` calls `rmdirSync` on the same directory.
The release callback is synchronous from both entry points;
 `await release()` in async callers resolves immediately because `await undefined` is non-blocking.

`pnpm-workspace.yaml` substitutes the abandoned upstream
 with this shim via `overrides: { 'proper-lockfile': 'link:package/pnpm-override/shim-proper-lockfile' }`.
The upstream is hard-imported by `@earendil-works/pi-coding-agent@0.74.0`'s `dist/core/auth-storage.js` (line 12)
 and `dist/core/settings-manager.js` (line 4),
both statically re-exported from the package's `dist/index.js` barrel (lines 6 and 22).
The pi CLI host constructs `SettingsManager.create(...)` at startup (`dist/main.js:377`)
 and resolves API keys via `await this.authStorage.getApiKey(...)`
 during model resolution (`dist/core/model-registry.js:519`);
both paths invoke the shim's sync and async surfaces.

## Simplifications versus upstream

- **No realpath resolution.
  **
  The shim calls `path.resolve` on the target,
   never `fs.realpath`.
  Both pi-coding-agent callsites always pass `{ realpath: false }` (verified at `auth-storage.js:38`
   and `settings-manager.js:44`),
   so the shim's behavior matches the only contract exercised in this workspace.
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
  The option is accepted on the options record but never invoked.
  pi-coding-agent's `auth-storage.js` reads the callback's effect via a `lockCompromised` flag that stays false;
   `throwIfCompromised()` becomes a no-op.
- **No retry jitter.
  **
  Upstream's `retries.randomize` option is ignored;
   backoff is strictly exponential by `factor` from `minTimeout` to `maxTimeout`.
  The workspace does not run concurrent pi instances,
   so deterministic backoff is acceptable.

`lockSync` throws `ELOCKED` immediately on first conflict;
 callers handle retries themselves.
`auth-storage.js`'s `acquireLockSyncWithRetry` and `settings-manager.js`'s matching helper both wrap `lockSync`
 in a sync retry loop.
`lock` retries internally per `options.retries` (number or object form),
 matching upstream's contract for the async callsite at `auth-storage.js:88`.

See `TROUBLESHOOTING.dependencies.md` for the audit trail,
 `doc/decision/proper-lockfile-removal.md` for the decision rationale,
 and `doc/dependency-blocklist.md` for when to reach for a shim package versus a generic stub or removal.
