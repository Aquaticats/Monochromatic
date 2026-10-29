# Handover: extract editord's dev-server watcher into a standalone dev-script package

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`.

## What you are picking up

editord's dev loop currently shells out to `watchexec` for file-watching and bun-process restart.
 The relevant task is in `packages/desktop-daemon/editord/mise.toml`:

```toml
run = "watchexec -w src/server --no-meta -r -- bun src/server/index.ts"
```

Two earlier rounds of work landed pieces that you should treat as the starting state,
 not the destination:

1. `--no-meta` suppresses `Modify(Metadata(Any))` events.
    Documented in `TROUBLESHOOTING.mise-watch.md`.
2. `saveFile` in `src/server/operations/save.ts` short-circuits
   when on-disk content already matches the requested write.
    Commit `27051b66`.
    The earlier `-j @content-changed.jaq` filter on watchexec was dropped because watchexec hangs on SIGINT
    when `-j` is used (`TROUBLESHOOTING.mise-watch.md`,
    "watchexec `-j` filter program hangs on SIGINT").

What is still on watchexec and what this handover is about:

- External editors (vim,
   vscode) doing format-on-save with byte-identical output still trigger a restart.
   The save-side skip cannot see those writes because they do not pass through editord.
   Rare but real.
- `watchexec` is an external native binary for a single dev task in a TypeScript-first monorepo,
   with documented architectural failure modes (SIGINT hang,
   signal propagation through process trees) the project has paid for once already.
- No other workspace package has a clean way to reuse the watch-and-restart pattern;
   `watch:build:css` is the obvious near-term candidate to also benefit from a content-hash filter.

## The decision

Create a new dev-script package at `packages/dev-script/watch-restart/`
(npm name `@monochromatic-dev/dev-script-watch-restart`).
 It exposes:

- A library API (`startWatchRestart(...)`) for programmatic use.
- A CLI bin entry `watch-restart` mirroring the watchexec invocation shape used today.

editord's `dev:server` task becomes a thin consumer of that package's CLI.
 The package is workspace-internal (`private: true`),
 built and consumed via pnpm catalog the same way `dev-script-file-enforcer` is.

For the watching layer use **chokidar 5** (the current major;
 verified by cloning `paulmillr/chokidar`,
 `package.json` shows `5.0.0`).
 For the restart driver use a small `node:child_process.spawn` wrapper.
 The previous "stay editord-local" recommendation is overridden:
 the watcher stands on its own as a package.

The save-side content-equality skip in `save.ts` stays.
 It is independently good and complements the watcher
 (cheaper than a watcher firing and re-hashing for an editord-internal save).

## Alternatives and prior art

Two orthogonal axes;
 conflating them muddles the comparison.
 Pick one from each.

### Axis 1: file watching

#### chokidar 5 (recommended)

- Cost:
   new workspace dep,
   chokidar plus its single transitive `readdirp`,
   both maintained by paulmillr.
- Fit:
   strong.
- Notes:
   `atomic: true` handles rename+create atomic saves out of the box;
   `awaitWriteFinish` handles chunked writes;
   recursive watching cross-platform.
   Production user list spans webpack,
   rollup,
   vite,
   gulp.
   Listed in `AUDIT.md` as "to evaluate";
   adopting it here clears that item.

#### `@parcel/watcher`

- Cost:
   new dep with native binding (prebuilt binaries per OS/arch).
- Fit:
   viable.
- Notes:
   Native C++ via N-API;
   very fast;
   recursive native everywhere.
   Adds compile/install surface (prebuilds usually work but the failure mode is uglier than a JS lib).
   Used in Parcel and Lit.

#### `watcher` (`fabiospampinato/watcher`)

- Cost:
   new dep,
   no transitive deps.
- Fit:
   acceptable.
- Notes:
   Smaller alternative to chokidar;
   listed in `AUDIT.md` separately from chokidar.
   Less production track record.

#### native `node:fs/promises.watch`

- Cost:
   zero deps.
- Fit:
   viable but more code.
- Notes:
   The path `packages/dev-script/file-enforcer/src/watch/` already takes;
   ~150 LOC plus tests for atomic-save,
   debounce,
   hash-compare.
   Not a great fit for a package deliverable that should minimize surprise.

#### `watchman`

- Cost:
   persistent background daemon plus `fb-watchman` npm client (callback-style,
   version 2.0.2 on npm but actively maintained;
   most recent commits at `facebook/watchman` `watchman/node/` from 2024-12 modernisation pass;
   2025-07 cleanup).
   Daemon installable via mise/brew like watchexec.
- Fit:
   would work,
   but not justified for this case.
- Notes:
   see "Can we just use watchman?
  " below for the full reasoning.
   Short version:
   watchman is architecturally a long-lived daemon designed to amortise its cost across many tools and many directories.
   We have one tool watching one directory.
   The watchman DSL is metadata-only (no content access),
   so the byte-identical filter still has to live in a TS hash cache.
   Cross-platform recursion and atomic-save handling are first-class in chokidar 5 too,
   so we do not gain those by switching.

### Aside: programmable filter DSL support across watchers

watchexec's `-j` flag (the flag whose SIGINT hang motivated this work) is uncommon.
 Only two widely-used watchers expose a programmable filter DSL:

- **watchexec**:
   `-j` runs a jaq program (a jq-compatible language) per event.
   State can be threaded through a kv-store (`kv_store`,
   `kv_fetch`,
   `kv_clear`),
   which is what made `content-changed.jaq` possible.
   Custom filter definitions extend the stdlib with `file_meta`,
   `file_size`,
   `file_read(bytes)`,
   `file_hash`,
   `string | hash`,
   and logging primitives.
   The hang affects both forms (inline `-j 'program'` and file `-j @program.jaq`);
   see `TROUBLESHOOTING.mise-watch.md`.
- **watchman**:
   the S-expression operators listed under the watchman entry above.
   Stateless per query;
   operates on the file index,
   not on content.

Every other watcher uses one of:
 glob patterns (`@parcel/watcher`'s `ignore`,
 `nodemon`'s `watch`/`ignore`,
 rollup-watch's `watch.exclude`),
 regex/string (chokidar's `ignored` accepts these),
 or a JS function callback (chokidar's `ignored` also accepts these;
 `fabiospampinato/watcher` similar).
 No JavaScript-side watcher ships a jq-language filter.

### Filter API for the new package: this is a design decision, not a one-liner

An earlier draft of this handover said "the hash cache becomes ~20 lines of TypeScript.
" That claim is unsafe:
 it replaces a general programmable filter with one hardcoded predicate,
 and the second consumer that wants a different filter shape
 (e.g. `watch:build:css` rebuilding only on style-token changes) would have to fork the package or extend it ad hoc.
 The planning agent should design the filter API deliberately,
 treating it as the package's main extension point.

Three options,
 ranked from cheapest to most expensive:

#### Option F1: function-predicate API (recommended starting point)

Library accepts `filter: (event, ctx) => boolean | Promise<boolean>`.
 Callers write TypeScript.
 The content-hash filter ships as a built-in helper alongside the watcher:

- `contentHashFilter()` returns a predicate closed over a private `Map<absolutePath, sha256hex>`;
   first-seen and same-hash events return `false` (skip),
   hash-different events return `true` (fire).
- Other built-in helpers as needed:
   `extFilter(['.ts', '.tsx'])`,
   `globFilter(...)`,
   `composeFilters(...)`.

For in-process TypeScript callers,
 function predicates are strictly more expressive than a DSL (you have the full language).
 The original jaq DSL existed because watchexec is a Rust binary
 and users cannot extend it without a foreign-function boundary;
 that boundary does not exist for us.
 The CLI form accepts `--filter-script <path>`:
 the path is loaded as a TS module and its default export is the predicate.

This is the recommended starting point because it covers every use case we have evidence for (content-hash,
 glob,
 extension) without inventing language design.

#### Option F2: small declarative format (compromise)

Ship the predicate API plus a small declarative form for CLI use without writing a `.ts` file.
 Example shape (illustrative,
 not prescriptive):

```jsonc
{ "all": [{ "extension": [".ts", ".tsx"] }, { "contentChanged": true }] }
```

This is purpose-built for our event shape and avoids reinventing jq.
 The CLI flag is something like `--filter-config <path>`.
 The format is a `Record` lookup keyed on operator name;
 adding an operator is one entry in a switch-equivalent.
 Worth the cost only if the planning agent identifies a second consumer
 that genuinely wants a config-driven filter (today only `content-hash + globs` is on the table).

#### Option F3: jaq subset or full port (last resort)

A subset of jaq in TypeScript.
 The reference Rust implementation (`01mf02/jaq`) is on the order of 10k,
20k LOC across the parser,
 optimizer,
 and evaluator;
 a TypeScript port that matches watchexec's `--help`-documented surface (the jaq stdlib plus `file_meta`,
 `file_size`,
 `file_read`,
 `file_hash`,
 `kv_store`,
 `kv_fetch`,
 `kv_clear`,
 logging primitives) is a multi-week project,
 not a side concern.
 We would also inherit jaq's semantic edge cases
 (e.g. the "stops after outputting the first value" gotcha that watchexec's own help calls out).

Reject unless and until the project has a documented need to consume *existing* jaq programs from another tool.
 We do not.

### Recommendation

**F1 (function-predicate API) + ship `contentHashFilter()` as a built-in helper.
** Keep the package's extensibility surface open and let TypeScript itself be the filter language.
 Revisit F2 only after a second consumer materialises with a real config-file need;
 F3 only after that consumer specifically needs jq syntax.

### Should this package be Rust instead of TypeScript?

`packages/cli/forbidden-strings/` is Rust,
 so the question is fair:
 why is this one not?
 The honest comparison,
 dimension by dimension.

#### Where forbidden-strings justifies Rust

- **Hot path.
  ** Runs in the pre-commit hook with a ~5 ms budget;
   Node startup alone is 50 to 100 ms and Bun startup is 30 to 80 ms.
- **CPU-bound work.
  ** Scans up to 2,700 files (21 MiB) in 15.5 ms;
   the per-file regex+aho-corasick cost dominates.
   Rust's `regex`,
   `aho-corasick`,
   and `resharp` crates win measurably here.
- **No library API surface.
  ** Consumed via CLI from a pre-commit hook and from CI.
   Nothing in the workspace wants to `import` it.
- **Native ecosystem advantage.
  ** `resharp` exists in Rust only;
   the set-algebra operators (`A&B`,
   `~A`) require it.

#### How `watch-restart` differs

- **Not a hot path.
  ** The watcher runs for the duration of a dev session and reacts to a few filesystem events per minute.
   Startup pays once and is invisible against the bun server's own startup.
- **Not CPU-bound.
  ** Per-event work is one stat plus one SHA-256 of a few hundred KB.
   Both Rust and Bun complete this in well under a millisecond of wall time;
   on this workload they are indistinguishable.
- **Library API is the point.
  ** The "second consumer can import the library" item is in the acceptance criteria.
   Workspace consumers expect
   `import { startWatchRestart, contentHashFilter } from '@monochromatic-dev/dev-script-watch-restart'`.
   A Rust binary cannot serve that without a NAPI-N layer,
   and the workspace currently has no NAPI build infrastructure (forbidden-strings ships as a standalone CLI binary;
   nothing imports it).
- **The filter API is TypeScript functions (F1 above).
  ** With a TypeScript watcher,
   a filter is `(event, ctx) => boolean | Promise<boolean>`:
   one cross-module call.
   With a Rust watcher,
   the filter is one of:
   a jaq DSL (rejected in F3),
   a subprocess-per-event boundary (slow,
   fragile),
   or a NAPI callback bridge (build complexity).
   The cross-language boundary costs more than the Rust speed buys back,
   by a wide margin.
- **The watchexec lesson cuts the other way.
  ** watchexec is Rust;
   the SIGINT hang we paid for is a Tokio reference cycle in `FilterProgs::new`.
   Rust does not automatically deliver correct signal+async semantics;
   a complex Rust async runtime hides exactly this class of bug.
   A single-threaded TypeScript watcher with one child process and direct `child.kill('SIGTERM')` is easier to audit
   and harder to construct a reference-cycle bug in.
- **`AGENTS.md` default.
  ** The workspace rule is TypeScript for dev scripts;
   forbidden-strings is the documented exception,
   granted because of the pre-commit hot path.
   Our workload does not have an analogous justification.

#### When this should be revisited

- If the watcher ever runs in a pre-commit or git-hook hot path with a sub-100ms budget.
   None of the proposed second consumers (e.g. `watch:build:css`) is hot in that sense.
- If the workspace gains NAPI build infrastructure (because something else in the repo needs it),
   enabling a Rust core with TS bindings without inventing the pipeline for this package alone.
- If a workload appears that genuinely is CPU-bound at fs-event rate (a hashing benchmark on hundreds of MB per event,
   for instance).

None of these is true today.
 The recommendation stays **TypeScript**,
 for the same shape of reason watchman is rejected:
 the language that would buy us speed on a hot path is the wrong choice for a package whose contract is "expose a TS
 function and a CLI to a TS-first workspace.
"

### Can we just use watchman?

This question came up while drafting the handover.
 The honest answer:
 yes,
 watchman would technically work,
 and the package shape (watcher + hash-cache + child-process + filter) is unchanged whether the watcher dep is chokidar
 or `fb-watchman`.
 The reason chokidar wins for our case is concrete,
 not a hand-wave about "overkill for very large repos.
" Spelling it out so the planning agent does not have to re-derive it.

#### What watchman offers that's real

- Cross-platform recursive watching,
   with all the platform-event-coalescing tricks Meta has fixed over a decade.
   Chokidar gives us this too (via `awaitWriteFinish` and `atomic`),
   but watchman's are the platform-native code paths.
- `watchman-make` ships in the watchman distribution for "watch and run a command" workflows.
   Looks like a candidate replacement for both the watcher *and* the restart driver in this handover.
- Multiple tools can share a single daemon's watches.
   If the system already runs several dev servers under watchman,
   adding ours is near-zero marginal cost.
- A programmable filter DSL (S-expression operators) that runs inside the daemon,
   before events cross the IPC boundary.
   Cheaper per-event for high firehose rates than a JS filter.

#### What watchman costs

- **Persistent daemon.
  ** `idle_reap_age_seconds` defaults to 5 days (`watchman/website/docs/config.md`).
   Watches are stored in a statefile and reinstated when the daemon restarts.
   Starting `dev:server` once leaves a watch and the daemon running across reboots until a 5-day idle window
   or a `watchman shutdown-server` call.
   For a single developer running one project,
   this is process state to reason about and clean up;
   for a system with ten projects,
   this is amortised infrastructure.
- **`fb-watchman` is callback-based** (`client.command(['watch-project', dir], (err, resp) => { ... })`).
   Wrapping in a Promise API is straightforward but is more glue we maintain.
- **`watchman-make` requires Python and `pywatchman`.
  ** Our repo's `mise.toml` does not currently install Python tooling.
   Adding it for one dev task crosses an architectural line.
- **The DSL is metadata-only.
  ** The byte-identical filter that motivated this whole work cannot live in watchman;
   it has to live in TS.
   So we adopt watchman *and* keep the hash cache.
   The DSL benefit is reduced to glob-style pre-filtering,
   which chokidar's `ignored` option does for free.
- **Cross-platform recursive is not a differentiator here.
  ** Chokidar already abstracts FSEvents/inotify/ReadDirectoryChangesW.

#### When this should be revisited

The watchman calculus flips if any of these become true:

- This monorepo grows to a size where chokidar's startup cost (the per-directory recursive scan via `readdirp`) becomes
   noticeable.
   Today the watched directory is `src/server/`,
   tens of files;
   the scan cost is microseconds.
- A second long-lived dev server is added to the workspace,
   then a third.
   Sharing a watchman daemon across them genuinely reduces the moving parts.
- The repo grows to need scm-aware querying
   (watchman's `scm-query` integrates with hg/git and gives "what changed since this commit" semantics for free).
   We do not need this today.
- The team adopts a Python toolchain that already brings `pywatchman` in.
   Then `watchman-make` becomes free.

None of these is true today.
 The recommendation stays **chokidar 5 + custom `child_process.spawn`**,
 but the rejection is "watchman's wins don't apply to our use case,
" not "watchman is bad.
"

### Axis 2: restart driver

#### custom `child_process.spawn` wrapper (recommended)

- Cost:
   ~80 LOC.
- Fit:
   strong.
- Notes:
   Direct control over signal propagation.
   Disqualifies the watchexec-style "SIGTERM does not reach the grandchild" failure mode by construction
   (one level of process tree).
   Tested pattern:
   `packages/desktop-daemon/editord/src/server/operations/spawn-detached.ts`.

#### `nodemon`

- Cost:
   new dep.
- Fit:
   poor.
- Notes:
   Designed to wrap Node,
   not Bun;
   assumes JS modules;
   no first-class content-hash filter;
   configuration is JSON-file-driven;
   restart semantics rely on Node module-cache invalidation that does not apply to Bun.

#### `pm2`

- Cost:
   new dep,
   heavy.
- Fit:
   poor.
- Notes:
   Production process manager,
   not a dev tool.
   Out of scope.

#### `bun --watch` / `bun --hot`

- Cost:
   zero.
- Fit:
   does not fit.
- Notes:
   (a) watches the **import graph**,
   not the source tree,
   so files not imported at startup do not trigger;
   (b) no content-hash filter,
   byte-identical writes still restart;
   (c) `--hot` is HMR-like (preserves state),
   which is the wrong semantics for a server that holds sockets/tokens;
   (d) `--watch` restart semantics under stdio inherit are undocumented for our config.
   Save the planning agent the detour.

#### `watchexec` (current)

- Cost:
   already installed via mise.
- Fit:
   rejected.
- Notes:
   The whole reason for this work.
   SIGINT hang with `-j` (analyzed in `TROUBLESHOOTING.mise-watch.md`);
   process-tree signal propagation problems also documented.

### Recommendation

**chokidar 5 + custom `child_process.spawn` wrapper.
**

The two failures we already paid for (signal propagation in deep process trees,
 SIGINT hang) both stem from wrapping a process with a tool whose internal async model we cannot inspect.
 A direct `spawn` keeps the tree one level deep,
 which makes signals trivially reliable.
 The watching layer is the part where reaching for a battle-tested library is worth the dependency:
 chokidar handles atomic-save rename+create natively,
 debouncing,
 cross-platform recursion,
 and the edge cases the prior-art `file-enforcer` watcher had to reimplement.

If the planning agent disagrees and prefers native `fs.watch` (matching `file-enforcer`'s pattern),
 the package shape and API are unchanged;
 only the watcher implementation differs.
 State the choice in the package README so future readers see the trade-off without re-deriving it.

## The hash cache stays regardless of library choice

Chokidar's `atomic: true` detects `mv _tmp file` atomic saves and emits a single `change` event for them.
 `awaitWriteFinish` waits for chunked writes to settle before firing.
 **Neither suppresses byte-identical writes.
** An external editor with format-on-save that produces identical output emits `change` under chokidar,
 under `@parcel/watcher`,
 under native `fs.watch`,
 under Bun's `path_watcher`,
 and under esbuild's polling watcher (modKey on Linux is `(inode, size, mtime_sec, mtime_nsec, mode, uid)`;
 mtime updates on any write,
 so a same-content save flips modKey and triggers re-detection;
 `internal/fs/fs_real.go:33-40` and the `stateFileHasModKey` branch at `internal/fs/fs_real.go:522-528`).

### Why no general-purpose watcher library does this

The pattern is established but lives at the *writer*,
 not the watcher.
 Several reference points:

- **Webpack `output.compareBeforeEmit`** (default `true`).
   `lib/Compiler.js` reads the destination,
   compares the new content to existing bytes,
   and skips the write when they match.
   The inline comment names the motivation:
   "to keep mtime and don't trigger watchers".
- **`file-enforcer/src/write.ts:overwrite()`** in this repo:
   same pattern,
   same reason.
- **`packages/desktop-daemon/editord/src/server/operations/save.ts`** in this repo:
   same pattern again,
   added in commit `27051b66`.

The structural reason for the universal write-side / watch-side split:

1. **Cost vs benefit.
   ** Hashing every event at watch-firehose rate costs disk I/O per event.
    Hashing at write-time is amortised:
    you were going to read or buffer the bytes anyway.
2. **The wrong layer.
   ** When the writer is under your control,
    write-side dedup is strictly better than watch-side dedup:
    it saves the disk write AND the mtime touch AND prevents the event firing AND avoids the consumer-side re-hash.
3. **State boundary.
   ** Watch libraries are designed stateless:
    they emit events,
    the consumer maintains state.
    A content-hash cache is consumer state.
    Watchexec's `-j` flag was exactly the experiment of pushing that state into the watcher,
    and the reference cycle that hangs SIGINT (`TROUBLESHOOTING.mise-watch.md`) is the kind of failure a stateless
    watcher does not have.

Some watchers do narrow content checks at the margin (esbuild falls back to byte compare when modKey is unusable,
 `internal/fs/fs_real.go:530-536`;
 `fabiospampinato/watcher` skips the rare "same-inode-both-zero-bytes" case in `watcher_poller.ts:115`),
 but no widely-used watcher library suppresses byte-identical writes as a primary feature,
 because the writer is the better place.

### Why we need it anyway

editord cannot control vim or vscode.
 Their save handlers do not consult our `save.ts` skip.
 They write the same bytes;
 the OS reports `Modify(Data(Any))`;
 the kernel cannot tell "same bytes" from "different bytes".
 The only places where byte-equality is knowable on the event side are (a) inside a stateful watcher (re-read and hash)
 or (b) inside the writer (compare before write).

Since (b) does not apply to external editors,
 the watcher must do (a).
 The replacement is a `Map<absolutePath, sha256hex>` content-hash cache:
 on every event,
 re-hash the file and compare.
 Different hash,
 restart and store.
 Same hash,
 skip silently.
 Deletion or first-seen,
 store without restart (first-seen) or trigger (deletion).

Do not remove the hash cache when adopting chokidar.
 The settle-detection is orthogonal.
 The hash cache is the part of `watchexec -j` that we still need.

## Why standalone package

The previous handover argued for keeping the watcher editord-local under "Three similar lines is better than a
premature abstraction" (`AGENTS.md`).
 That decision is overridden because:

- The watcher needs its own README,
   tests,
   and stable API to "stand on its own";
   a package is the existing workspace shape for that.
- Watchexec's failure modes are not editord-specific.
   Any package that wraps a long-running process for development would hit them.
   Centralizing the fix avoids the rule's other failure mode:
   hard-coding a pattern into one consumer makes the second consumer reimplement it.
- `AUDIT.md` already lists `chokidar` and `watcher` as "to evaluate".
   Adopting one here gives a concrete reason for the eval and a single integration point.
- The package can later absorb `watch:build:css`
   (which currently runs `mise watch -w src/client -g '...' -r -- build:css`)
   when content-hash filtering becomes worth the move.
   The library API supports that without a second extraction.

## Package shape

Follow `packages/dev-script/file-enforcer` as the closest existing model (library + CLI;
 build to `dist`;
 published surface via `exports`).
 `packages/dev-script/task-util` is the next closest (CLI-only with multiple `bin` entries)
 but the watcher needs a library export,
 so file-enforcer is the better template.

- `packages/dev-script/watch-restart/package.json`:
   name `@monochromatic-dev/dev-script-watch-restart`,
   `private: true`,
   `main`/`exports` pointing at the built dist plus a `./ts` export for direct source consumption
   (file-enforcer pattern),
   `bin.watch-restart` pointing at the CLI entry.
- `packages/dev-script/watch-restart/mise.toml`:
   `extends`-only tasks for `build`,
   `build:js:node`,
   `watch:build:js:node`,
   `lint`,
   `lint:types`,
   `lint:oxlint`,
   `test:unit`.
- `packages/dev-script/watch-restart/tsdown.node.config.ts`:
   same pattern as file-enforcer.
- `packages/dev-script/watch-restart/tsconfig.json`:
   workspace standard.
- `packages/dev-script/watch-restart/README.md`:
   motivation,
   CLI usage,
   library usage,
   the chokidar-vs-fs.
  watch choice with reason.
- `packages/dev-script/watch-restart/src/index.ts`:
   CLI entry,
   `#!/usr/bin/env bun` shebang.
   Library export goes through a separate `mod.ts` or named exports in `index.ts`;
   pick whichever matches file-enforcer's current layout when you read it.
- `packages/dev-script/watch-restart/src/`:
   split files to stay under the max-lines limit.
   Suggested decomposition (the planning agent finalizes):
  - watcher (chokidar adapter,
     event normalization)
  - hash-cache (Map of path -> sha256 hex;
     uses the helper at `packages/module/es/src/types/t string/f/t string/hash/`)
  - child-process (spawn/restart/stop with SIGTERM-then-SIGKILL timeout)
  - cli (argument parsing via `@optique/run` consistent with task-util)
  - logger (use `@monochromatic-dev/module-logger` per `AGENTS.md`)
- Unit tests at `*.unit.test.ts` covering:
   byte-identical save produces no restart,
   atomic rename with new content fires once,
   two writes inside the debounce window coalesce to one restart,
   deletion fires once,
   SIGTERM exits cleanly within ~1s.

### CLI surface

Mirror the subset of watchexec flags the project uses,
 so the editord `mise.toml` change is mechanical:

```
watch-restart -w <dir> [-w <dir>...] [--debounce <ms>] [--stop-timeout <ms>] -- <cmd> [<args>...]
```

No `-r` flag (restart-on-change is the only mode).
 No `-j`/filter DSL (the hash cache subsumes it).
 No `--no-meta` (metadata-only changes produce identical hashes and are skipped by construction).

The exact library API signature (`startWatchRestart(...)` and any auxiliary exports) is left to the planning agent.
 The CLI surface above is the only consumer contract this handover commits to.

### editord consumer side

- `packages/desktop-daemon/editord/package.json`:
   add `@monochromatic-dev/dev-script-watch-restart` to `dependencies` (workspace dep).
- `packages/desktop-daemon/editord/mise.toml`:
   `dev:server.run` becomes `watch-restart -w src/server -- bun src/server/index.ts`.
   Remove the watchexec comment block;
   replace with a one-liner pointing at this package.

## Signal handling constraint (carries over)

This is the constraint that disqualifies several "easy" restart options and the planning agent needs it written down:

- watchexec hung on SIGINT because of an internal reference cycle.
   We cannot adopt a tool whose async runtime we cannot inspect.
- The earlier `watchexec → mise → sh → bun` chain dropped SIGTERM and orphaned bun.
   Anything that introduces a process layer between the watcher and the server is suspect.
- Direct `proc.kill('SIGTERM')` against the spawned bun child,
   with a 5-second timer and SIGKILL fallback,
   matches the documented `--stop-timeout 0` discussion in `TROUBLESHOOTING.mise-watch.md`
   and keeps the tree one level deep.

## Scope

In scope:

- New package `packages/dev-script/watch-restart/` as described.
- Add chokidar (and its single dep `readdirp`) to the pnpm catalog and the new package's `dependencies`.
- Update editord's `package.json` and `mise.toml` to consume it.
- Amend `TROUBLESHOOTING.mise-watch.md`:
   the watchexec sections stay (they are upstream-bug documentation).
   Update the "Workaround" paragraph of the SIGINT-hang section to note the loop has migrated off watchexec entirely.
   The save-side skip section stays accurate.

Out of scope:

- Any change to `watch:build:js:client`,
   `watch:build:js:node`,
   `watch:build:css`.
   These do not use `-j` and are not in the failure mode we are addressing.
   Migrating them is a separate decision once the library API has settled.
- Any change to the editord runtime `DirWatcher` at `src/server/operations/watch-filesystem.ts`.
   That watches user files served to the browser,
   not server source files.
- Any change to `save.ts` content-equality skip.
   Keep it.
- Filing the upstream watchexec patch.
   The draft is in `TROUBLESHOOTING.mise-watch.md` and is a separate task.
- Cross-platform recursive logic for macOS or Windows beyond what chokidar handles for free.
   Chokidar already abstracts FSEvents and ReadDirectoryChangesW;
   the planning agent verifies,
   the package documents.
- Persisting the hash cache to disk.
   In-memory is correct:
   a process restart re-scans and treats files as first-seen,
   no spurious restart.

## Acceptance criteria

1. `packages/dev-script/watch-restart/` exists with `README.md`,
    `package.json`,
    `mise.toml`,
    `tsconfig.json`,
    `tsdown.node.config.ts`,
    and `src/`.
2. `mise run //packages/dev-script/watch-restart:build` succeeds and produces a runnable `dist/`.
3. `mise run //packages/dev-script/watch-restart:lint` exits zero.
4. `mise run //packages/dev-script/watch-restart:lint:types` exits zero.
5. `mise run //packages/dev-script/watch-restart:test:unit` passes;
    tests cover at minimum the five cases listed under "Unit tests" above.
6. `mise run //packages/desktop-daemon/editord:dev:server` starts the bun server through `watch-restart`,
    prints startup logs,
    and responds to HTTP on the configured port.
7. Editing any file under `src/server/` with new content triggers exactly one restart;
    previous bun process exits,
    new bun process starts on the same port (no `EADDRINUSE`).
8. Saving a file under `src/server/` with byte-identical content
   from any source (editord's own save handler or an external editor) produces no restart.
9. `touch src/server/index.ts` produces no restart.
    The hash compare short-circuits a metadata-only event.
10. Ctrl+C in the dev terminal exits the watcher within ~1 second.
     The bun child exits.
     The terminal prompt returns.
     A subsequent `mise run //packages/desktop-daemon/editord:dev:server` succeeds.
11. `kill -TERM <pid>` against the watcher process produces the same clean shutdown.
12. The bun child's stdout and stderr appear in the dev terminal unchanged (stdio inherit;
     do not buffer or transform).
13. `rg 'watchexec' packages/desktop-daemon/editord/` returns no matches
    (the binary may still resolve via `mise install`;
     the editord package no longer references it).
14. A second consumer can import the library:
     `import { startWatchRestart } from '@monochromatic-dev/dev-script-watch-restart'` resolves through the workspace,
     types check,
     and a smoke test in the package exercises the API surface that consumers would use.

## Files to touch

- `packages/dev-script/watch-restart/` (new package,
   structure above)
- `pnpm-workspace.yaml`:
   add `chokidar` and `readdirp` to the catalog
   (or confirm chokidar's transitive `readdirp` does not need its own catalog entry,
   depending on workspace conventions;
   check sibling additions like `nano-spawn`).
- `packages/desktop-daemon/editord/package.json`:
   add workspace dep on `@monochromatic-dev/dev-script-watch-restart`.
- `packages/desktop-daemon/editord/mise.toml`:
   rewrite `dev:server.run`;
   replace the watchexec comment block with a one-liner pointing at this package.
- `TROUBLESHOOTING.mise-watch.md`:
   amend the SIGINT-hang section's "Workaround" paragraph;
   preserve the upstream-bug analysis and draft issue.
- `AUDIT.md`:
   tick `chokidar` (or `watcher`,
   depending on what the planning agent picks) as evaluated,
   with a one-line reason and the package path that uses it.

## Files to leave alone

- `packages/desktop-daemon/editord/src/server/`:
   server runtime;
   the watcher is dev-loop-side.
- `packages/desktop-daemon/editord/src/server/operations/save.ts`:
   keep the content-equality skip.
- `packages/desktop-daemon/editord/src/server/operations/watch-filesystem.ts`:
   editord runtime watcher for browser-served files.
   Different concern.
- `watch:build:js:*` and `watch:build:css` tasks:
   out of scope.
- `packages/dev-script/file-enforcer/src/watch/`:
   reference implementation;
   do not refactor.
   The new package borrows the shape but uses chokidar instead of `fs.watch`.

## Reference material

- Watchexec failure-mode analysis (the constraint that disqualifies wrapper-tool restart options):
   `TROUBLESHOOTING.mise-watch.md`,
   sections "EADDRINUSE from deep process trees on restart" and "watchexec `-j` filter program hangs on SIGINT".
- Prior-art directory watcher (native `fs.watch`):
   `packages/dev-script/file-enforcer/src/watch/watch-dir.ts:37-79`.
- Prior-art watcher orchestration with debounce and abort:
   `packages/dev-script/file-enforcer/src/watch/watch.ts:34-168`.
- Prior-art `DirWatcher` class with debounce
   and per-path suppression (heavier than needed but shows the per-path classify pattern):
   `packages/desktop-daemon/editord/src/server/operations/watch-filesystem.ts:45-271`.
- Save-side content-equality skip (already shipped,
   do not modify):
   `packages/desktop-daemon/editord/src/server/operations/save.ts:31-77`.
- SHA-256 hex helper in `@monochromatic-dev/module-es`:
   `packages/module/es/src/types/t string/f/t string/hash/`;
   exported through the package's standard surface.
- Package shape template:
   `packages/dev-script/file-enforcer/{package.json,mise.toml,tsdown.node.config.ts,tsconfig.json}`.
- CLI shape template (for the bin entry and argument parser choice):
   `packages/dev-script/task-util/`.
- Chokidar 5 verification:
   cloned from `paulmillr/chokidar`;
   `package.json` reports `5.0.0` with single dep `readdirp ^5.0.0`;
   `awaitWriteFinish` and `atomic` are first-class options;
   `engines.node >= 20.19.0`.
- `AGENTS.md` rules:
   TS-only dev scripts (no bash),
   tagged loggers from `@monochromatic-dev/module-logger`,
   package-completeness (README + zero-error lint + tests on every code path),
   max-lines remediation by splitting.
