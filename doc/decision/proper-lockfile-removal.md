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

   Superseded by the amendment of 2026-11-14:
    the lock is now a record file created by hard link,
    a dead holder is taken over under an arbitration record,
    and `onCompromised` is called.

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
  Superseded by the amendment of 2026-11-14:
   the lock is a record file created by hard link and released by `unlinkSync`,
   and a lock whose recorded holder is gone is taken over.
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
  Superseded by the amendment of 2026-11-14:
   the lock names its holder,
   a lock whose holder is gone is taken over,
   and no freshness write is needed.
- **No `onCompromised` callback.
  **
  The option is accepted but never invoked;
   pi-coding-agent's `auth-storage.js` reads the callback's effect via a `lockCompromised` flag that stays false,
   so its `throwIfCompromised()` checks are no-ops.
  Superseded by the amendment of 2026-11-14:
   the callback is called while the lock is held,
   through the ownership check that runs every `stale / 2` milliseconds.
- **No `retries.randomize`.
  **
  Backoff is strictly exponential.
  The workspace does not run concurrent pi instances,
   so deterministic backoff is acceptable.
  Superseded by the amendment of 2026-11-14:
   this machine runs sixteen to twenty-two pi agents at once,
   and a lock whose holder is gone is now taken over instead of retried forever.

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
   Superseded by the amendment of 2026-11-14:
    the next invocation takes the lock over and no manual removal is needed.
2. **Concurrent pi instances are not supported.
   **
   Upstream's locking provides cross-process safety with retry+jitter;
    the shim provides only the EEXIST atomic primitive.
   Multiple concurrent pi instances would serialise correctly (mkdirSync is atomic),
    but the deterministic backoff could synchronise their retries into a livelock.
   The workspace does not run multiple concurrent pi instances;
    if that changes,
    revisit the shim.
   Superseded by the amendment of 2026-11-14:
    this machine runs sixteen to twenty-two pi agents at once,
    so the condition for revisiting was met and the shim was revisited.
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

## Amendment, 2026-11-14T03:13:43-05:00: a lock whose holder is gone is taken over

The shim as first written never takes a lock over.
On 2026-11-14 an empty `<agentDir>/.models-store.json.lock` directory appeared at
 01:43:06 and stood until it was removed by hand at 02:02:59,
 and every pi run started in those twenty minutes died while loading an extension with
 `Lock file is already held (lock: /home/user/.pi/agent/.models-store.json.lock)`.
Sixteen pi processes that were already running went on working;
 a lock nobody was inside stopped everything that started.
This record's own condition for revisiting the shim was met,
 because this machine runs sixteen to twenty-two pi agents at once,
 started by a launcher with nobody watching,
 so risk callout 2 above no longer describes the workspace.

### What proves a holder gone

The lock is a file,
`.<basename>.lock`,
holding a record of the process that took it:
 that process's pid,
 the process start tick the kernel reports at field 22 of `/proc/<pid>/stat`,
 and this boot's id.
A caller that finds the lock reads the record and asks the kernel whether that exact process still exists.
A process that no longer exists,
 or one whose pid now belongs to a process started at a different tick,
 or a record from a different boot,
 is gone,
 and the caller takes the lock over.
A process that exists is alive,
 and its lock is never taken over,
 whatever the lock file's age.

The three cases:

- The holder is alive and blocked,
  as this machine's home drive did for up to 143 seconds on a small write
   in the night of 2026-11-14.
  Its process exists,
  so the answer is "alive" and the lock is left alone.
  This is what a freshness bound cannot promise and why upstream's `stale` was not adopted:
   a live process can be slow for longer than any bound,
   and time since a lock was made is not evidence about its holder on this machine.
- The holder was killed at any instruction.
  The lock is created by hard-linking a fully written record onto its path,
   and that link is the only step that takes the lock,
   so a killed holder leaves either no lock at all or a lock whose record names it.
  There is no instruction at which a kill leaves a lock that cannot be attributed.
- Two callers find the same dead lock in the same instant.
  Replacing a dead lock requires creating the arbitration record `<lock path>.takeover`,
   which is created the same atomic way,
   so exactly one caller replaces the dead lock and the others are refused with `ELOCKED`.
  A caller that has the record re-reads the lock before replacing it,
   so a lock that changed under it is left alone.
  A takeover record whose own process is gone is cleared by the next caller.

A lock nobody can be named for,
which is an empty lock directory from the shim before this amendment and a lock file
 whose record a machine crash emptied before its data reached the disk,
 is taken over once it is older than the grace:
 `options.stale` when the caller passes one,
ten seconds otherwise.
Neither shape is ever written by the amended shim,
 so the grace covers only states no live holder of this version can be in,
 and an empty directory from the previous shim is taken over rather than left forever.
Superseded by the amendment of 2026-11-14T10:45:08-05:00:
 age is no longer a reason to take a lock over,
 so a lock nobody can be named for is refused with the command that clears it and left for a person.

### A lock taken away from its holder

`release()` reads the record before removing the lock,
 so it never removes a lock that no longer names it.
Each holding carries a nonce of its own,
 so a release callback kept from an earlier holding of the same file by the same process removes nothing of the later
 holding either;
 a record of pid,
start tick and boot alone would have made the two holdings indistinguishable.
When the record is another holding's,
or no record is there at all,
 the caller's `onCompromised` is given an error with `code === 'ECOMPROMISED'` and the lock is left alone.
While the lock is held,
the same check runs every `stale / 2` milliseconds when the caller passed a callback,
 which is what makes `auth-storage.js`'s `throwIfCompromised()` guard meaningful:
 the caller learns its lock is gone before it writes,
not after.

Removing a lock this shim wrote is conditional on it still being the entry judged dead,
 because POSIX has no unlink-if-content:
 the entry is renamed aside under a private name,
 the entry actually moved is compared with the one inspected by inode and by record,
 and an entry that is not the judged one is hard-linked back.
So a lock another caller took in the microsecond between judgement and removal is put back rather than deleted.
A directory an earlier shim left has no record to compare and is removed on its age judgement alone.

### What the change costs and what it does not wait for

One lock and release with no contention now does a small write,
 a hard link,
 an unlink of the temporary record,
 one `lstat` and one read of the record,
 and reads of `/proc/<pid>/stat` and `/proc/sys/kernel/random/boot_id` (each cached for the process lifetime)
 instead of one `mkdir` and one `rmdir`.
Measured on this machine with both shims alternating inside each of thirty rounds of five hundred lock-and-release
 pairs,
in three rounds of that measurement,
 the middle cost of one pair is 41 to 57 microseconds before and 124 to 147 microseconds after,
 a ratio of 2.6 to 3.1 per round (1.4 to 4.7 over the individual rounds),
 against run-to-run spreads of 25 to 136 percent in the unchanged arm.
The section such a lock guards is a read and a write of a JSON file,
 so the added microseconds do not change what the caller waits for.
The shim adds no wait the caller did not ask for:
 a dead holder's lock is taken over inside the caller's first attempt,
 and the only retries are the caller's own loop or the `retries` the caller passed.

### What this amendment leaves out

- No freshness writes.
  `stale` is now the ownership check interval and the grace on a lock nobody can be named for;
   it is not a freshness bound,
  because age does not decide anything for a named holder.
- No `lockfilePath` option,
  so pi's trust store still locks the sibling of its directory
   rather than the path it names.
- No process identity outside `/proc`:
   on a platform without it the record carries the pid alone and a reused pid reads as alive,
   which refuses takeover rather than stealing a live holder's lock.
- No hard-link-free filesystem:
   taking a lock hard-links a written record onto its path,
  so a filesystem without hard links fails loudly instead of
   locking.
- No record to compare on a directory an earlier shim left:
   its removal is decided by age,
  so a caller that replaced that directory inside one removal can be affected in the way
   the file path no longer can.
  Superseded by the amendment of 2026-11-14T10:45:08-05:00:
   such a directory is no longer removed at all,
  and the caller is given the command that clears it.
- Nothing signals a holder that its lock is gone except `onCompromised`:
   `lockSync` callers that pass no callback learn of a taken lock at their next write,
  as they did before.

## Amendment, 2026-11-14T10:45:08-05:00: only a proven holder's lock is taken over

The amendment above took a lock over on two grounds besides a named holder that is gone:
 a lock nobody could be named for was aged out after a grace,
 and a record written in another PID namespace was looked up in the reader's own namespace.
A review of that version reproduced three ways for a live holder to lose its lock,
 each ending with two live processes that both believed they held one lock:

- a live holder's valid lock whose record could not be read,
  which is what the stalled home drive gives,
  was treated as naming nobody and aged out;
- a live holder inside a private PID namespace records the process number its namespace gives it,
  which in the reader's namespace names another process with another start tick,
  so the reader judged it gone;
- a live holder of the shim before this one,
  whose lock is an empty directory,
  was aged out and removed,
  which is every pi process started before the new shim was installed.

The reading this amendment implements is the coordinating session's,
adopted from this record and from two requirements
the project holds together:
nothing a person made is lost,
and nothing waits forever.
A lock is taken over only on proof that its holder is gone,
and nowhere else.
Where that cannot be judged,
this record already names what happens:
a person removes the lock by hand.
So a caller that meets such a lock ends within the bound its own retry loop allows,
 and says what it found,
 why it cannot tell,
 and the one command that clears it.
Age alone is never proof,
and an error reading the record is never proof.

### The two proofs

The record now also carries the inode of the PID namespace its holder was in,
 and both proofs are about the process the record names:

- the record was written in an earlier boot,
  so that process cannot exist now;
- the record names a process of this shim's own PID namespace that no longer exists,
  has exited,
  or whose number now belongs to a process started at another tick.

A record from another PID namespace is not judged:
 the reader cannot look the number up,
and nothing else in the record identifies the process across namespaces.
There is no boot-wide process identity on Linux to record instead,
 since a process's number in the initial namespace is not something a namespaced process can learn.

A lock with no record at all cannot be judged either.
The suggestion that a record-less lock's maker is gone when no process this reader can see started before the lock's
mtime was considered and is not used,
because it is not a proof:

- a maker in another PID namespace is invisible to this reader,
  so a live holder looks like no process at all;
- a wall clock stepped forward after the lock was made makes the live maker appear to have started after it,
   which is the direction that steals;
- a forward or backward step also moves the comparison between a file's mtime and a process start computed from
   uptime,
  and the two clocks are not the same clock;
- a directory's mtime can be moved by anything that writes in it,
  and `touch -d` can set it.

The same reasoning applies to a record whose bytes a machine crash emptied:
unreadable bytes name no process,
 so such a lock is refused and named rather than aged out.

### What a caller sees, end to end

`dist/core/models-store.js` and `dist/core/auth-storage.js` call
 `lockfile.lock(path, { retries: 0, stale: 30_000, onCompromised })` and loop for thirty seconds,
 so a lock this shim cannot judge is answered at once as `ELOCKED` and retried by the caller's own loop,
 whose deadline ends the wait and whose last error is this shim's.
The message travels to the person who reads the run:
 an installed pi 1.5.2 run against a disposable agent directory holding an empty lock directory prints,
after about
 thirty-eight seconds and its own retries,
the extension load failure followed by
 "The lock at … is an empty lock directory that an earlier version of this shim wrote.
… remove it and start again with:
 rmdir …".
A prompt run of the same ends with exit 1,
and the lock is still there afterwards.
A pi that meets a lock whose holder is proven gone starts clean.
A start that ends this way is then the driver's to start again,
which is a task another agent holds.

A read that fails is also not a lost lock for a holder:
 the ownership check says nothing,
and the release rethrows the read's error and stays un-released,
 so a stalled drive cannot make a live holder abandon a section it still holds.

### What the change costs and what it does not wait for

Nothing was added to the uncontended path:
the namespace inode is read once per process and cached,
 and the ownership check compares it in memory.
The measured cost recorded above therefore stands.
The shim adds no wait:
an unjudgeable lock is answered on the caller's first attempt,
 and a proven-gone holder's lock is taken over on it.

### What this amendment leaves out

- No proof for a record-less lock,
  so an empty directory made by the shim before this one is cleared by hand.
  Within one version this costs nothing,
  because every lock is a record file and a killed holder's record is proof;
   it costs the upgrade window only,
  where a pi started before this shim can still hold a directory.
- No proof across PID namespaces,
  so a lock a container took on a shared filesystem is cleared by hand,
   and the message names both namespaces so a person can tell what happened.
- No proof without `/proc`:
  a platform that publishes no process identity cannot take a lock over at all.
- No `lockfilePath` option,
  no freshness writes,
  and no signal to a holder except `onCompromised`,
   as recorded above.

## Amendment, 2026-11-14T15:13:55-05:00: the printed command is one shell word

The amendment above promises that a caller who cannot judge a lock is told the one command that clears it,
and a review
of that version found the promise broken for a path a shell splits:
the command was built as `rmdir <path>` with the path interpolated raw,
so for an agent directory whose path holds a space the printed `rmdir` failed and the printed `rm -f` ended with status
0 and removed nothing,
and a person who followed the words met the same refusal again with no sign of why.

The path is now encoded at the last interpolation,
where the text becomes a shell command and nowhere earlier:
single-quoted,
with each single quote inside it closed,
escaped outside the quotes and reopened.
That is the one quoting that holds every byte a path can carry,
a space,
a single quote,
a double quote,
a dollar sign,
a backtick and a line break among them,
and it is what the package's controls now run through `/bin/sh` for each of those five,
with the lock required gone
afterwards.

Nothing else changed:
the words are still the refusal,
the code is still `ELOCKED`,
`error.remedy` still carries the
command,
and the shim still never removes a lock it cannot judge.
