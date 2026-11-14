# @monochromatic-dev/pnpm-override-shim-proper-lockfile

API-compatible shim for [`proper-lockfile`](https://www.npmjs.com/package/proper-lockfile).
Implements `lockSync(file, options)` and `lock(file, options)` by creating a sibling `.<basename>.lock` file atomically,
holding in it a record of the process that took it.
`release()` removes the lock only while that record still names the releasing process.

`pnpm-workspace.yaml` substitutes the abandoned upstream
 with this shim via `overrides: { 'proper-lockfile': 'link:package/pnpm-override/shim-proper-lockfile' }`.
The upstream is hard-imported by `@earendil-works/pi-coding-agent@1.5.2`'s `dist/core/auth-storage.js` (line 7)
 and `dist/core/settings-manager.js` (line 4),
both statically re-exported from the package's `dist/index.js` barrel.
The pi CLI host constructs `SettingsManager.create(...)` at startup (`dist/main.js:377`)
 and resolves API keys via `await this.authStorage.getApiKey(...)`
 during model resolution (`dist/core/model-registry.js:519`);
both paths invoke the shim's sync and async surfaces.
The `openai-fast` extension of this workspace restores a provider model catalog through that async path
 (`FileModelsStore`,
`dist/core/models-store.js:30`),
 which is how one dead holder stopped every pi run on 2026-11-14,
described in
 `doc/decision/proper-lockfile-removal.md`.
The release callback is synchronous from both entry points,
 so `await release()` in async callers resolves immediately because `await undefined` is non-blocking.

## Pi dependency trust

PZG:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

## A lock whose holder is gone

The lock file records the process that took it:
its pid,
 the process start tick the kernel reports at field 22 of `/proc/<pid>/stat`,
 the inode of the PID namespace it was in,
 and the boot id.
A later caller reads that record and asks the kernel whether the named process still exists at that start tick.
A lock whose recorded process is gone is taken over by the caller that finds it;
 a lock whose recorded process is alive is never taken over,
 however old the lock file looks.
That is the proof,
and it is why a holder blocked on this machine's stalled home drive for minutes keeps its lock:
 its process still exists,
so nothing about the lock's age matters.

There are exactly two proofs,
and both are about the process the record names:

- the record was written in an earlier boot,
  so that process cannot exist now;
- the record names a process of this PID namespace that no longer exists,
  has exited,
  or whose number now belongs to a process started at another tick.

## A lock this shim cannot judge

A lock is taken over on proof and alone,
so every other lock is refused with an error whose message says what was found,
 why it cannot be told whether a run is still inside the lock,
and the one command that clears it.
That covers:

- an empty lock directory,
  which is the lock the shim before this one wrote,
  and which names no process;
- a lock file whose record could not be read,
  which may still be a live holder's lock on a drive that stalled;
- a lock file whose bytes are not a record,
  which is what a machine that lost power can leave;
- a record written in another PID namespace,
  whose process number names nothing this process can look up.

The code is `ELOCKED`,
so a caller's own retry loop is what bounds the wait and what ends it,
and the caller's own last
error carries the words.
The shim never waits by itself and never guesses:
an unjudgeable lock stays until a person removes it with the command
the message names,
which is the recovery `doc/decision/proper-lockfile-removal.md` records for what the shim cannot
settle.

The takeover record `.<basename>.lock.takeover` arbitrates between callers that find the same dead lock in the same
instant:
creating it is atomic,
so exactly one caller replaces the dead lock and the others are refused with
`ELOCKED`.
It is removed when the lock is settled,
and a takeover record whose own process is proven gone is cleared by the caller
that finds it.

## A lock taken away from its holder

`release()` reads the record in the lock before removing it.
When the record is another holding's,
the lock belongs to someone else and is left alone;
 the caller's `onCompromised` callback is given an error with `code === 'ECOMPROMISED'`.
Each holding carries a nonce of its own,
so a release callback kept from an earlier holding of the same file by the same
 process removes nothing either.
While the lock is held,
the same check runs every `stale / 2` milliseconds when the caller passed a callback,
 so a holder learns that its lock is no longer its own before it writes.
A read that fails is not a lost lock:
the watch says nothing,
and the release rethrows the read's error and stays
 un-released,
so a later call retries rather than a live holder being told its lock is gone.

## Simplifications versus upstream

- **No realpath resolution.
  **
  The shim calls `path.resolve` on the target,
   never `fs.realpath`.
  Both pi-coding-agent callsites pass `{ realpath: false }` (verified at `auth-storage.js:51`
   and `settings-manager.js:132`),
   so the shim's behavior matches the only contract exercised in this workspace.
- **No freshness writes.
  **
  Upstream rewrites the lock's mtime on a `stale` interval and treats a lock older than `stale` as abandoned.
  The shim never writes while it holds,
  because its takeover decision does not depend on age at all:
   the record names the holder,
  and the kernel answers whether that holder exists.
  `stale` bounds the ownership check interval of a held lock and nothing else.
- **No retry jitter.
  **
  Upstream's `retries.randomize` option is ignored;
   backoff is strictly exponential by `factor` from `minTimeout` to `maxTimeout`.
- **No `lockfilePath` option.
  **
  The lock path is always the `.<basename>.lock` sibling.
  pi-coding-agent's trust store passes `lockfilePath`,
  so its lock lands at the sibling of the trust directory
   rather than at the path it names;
  both pi runs of one workspace still share that one lock.

`lockSync` throws `ELOCKED` on first conflict after taking over a dead holder's lock;
 callers handle retries themselves.
`auth-storage.js`'s `acquireLockSyncWithRetry` and `settings-manager.js`'s matching helper both wrap `lockSync`
 in a sync retry loop.
`lock` retries internally per `options.retries` (number or object form),
 matching upstream's contract for the async callsite at `auth-storage.js:93`,
 which passes `retries: 0` and keeps its own thirty-second loop as the only retry.

See `TROUBLESHOOTING.dependencies.md` for the audit trail,
 `doc/decision/proper-lockfile-removal.md` for the decision rationale and its amendment,
 and `doc/dependency-blocklist.md` for when to reach for a shim package versus a generic stub or removal.
