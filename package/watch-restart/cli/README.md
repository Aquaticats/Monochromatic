# @monochromatic-dev/watch-restart-cli

Ready to publish.

Watch source files and restart a long-running child process on content change.

Replaces `watchexec` in editord's dev loop and is reusable as a library for any
workspace dev server.
 Two failures the previous loop paid for are excluded by
construction:
 signal propagation through deep process trees (the watcher owns
one child,
 not a tree),
 and the SIGINT hang from watchexec's `-j` filter (no
filter DSL,
 no embedded interpreter).

## Why a package

editord originally shelled out to `watchexec -w src/server --no-meta -r --
node src/server/index.ts`.
 Two failure modes documented in
the repo-root `TROUBLESHOOTING.mise-watch.md` motivated the
replacement:
 a Tokio reference-cycle SIGINT hang in watchexec's jaq filter
mode,
 and SIGTERM not reaching grandchildren through the `watchexec → mise → sh
→ node` chain.
 A workspace-local TypeScript implementation pins the runtime we
can audit (chokidar plus `child_process.spawn`) and exposes a library API that
other dev servers in the workspace can reuse without learning a new tool.

The save-side content-equality skip in
`package/desktop-daemon/editord/src/server/operations/save.ts` is unchanged
and complementary:
 editord's own writes never reach the watcher,
 and the
watcher's content-hash filter catches byte-identical writes from external
editors (vim,
 vscode) that bypass `save.ts` entirely.

## CLI

```text
watch-restart [-w <dir>...]
              [-i <glob>...] [-e <glob>...]
              [--include-regex <pat>...] [--exclude-regex <pat>...]
              [--ext <ext>...] [--type <kind>...]
              [--events <list>]
              [--hidden | --no-hidden]
              [--follow-symlinks | --no-follow-symlinks]
              [--gitignore | --no-gitignore] [--ignore-file <path>...]
              [--depth <n>] [--poll <ms>]
              [--no-content-changed] [--max-hash-size <bytes>]
              [--debounce <ms>] [--no-initial]
              [--clear | --no-clear]
              [--signal <name>] [--stop-timeout <ms>]
              [--process-group | --no-process-group]
              -- <cmd> [<args>...]
```

editord's invocation (defaults cover this case):

```bash
watch-restart -w src/server -- node src/server/index.ts
```

### Filter flags

These flags shape which events fire a restart.
 Filters compose AND across
categories,
 OR within a category.

- `-w`,
   `--watch <dir>`:
   directory to watch recursively.
   Repeatable.
   Required.
- `-i`,
   `--include <glob>`:
   include glob (picomatch),
   OR'd across repeats.
   Default:
   everything.
- `-e`,
   `--exclude <glob>`:
   exclude glob (picomatch);
   any-match short-circuits to skip.
   Default:
   none.
- `--include-regex <pat>`:
   include regex against `relativePath`,
   OR'd across repeats.
   Use for anchored alternation and lookarounds picomatch cannot express.
- `--exclude-regex <pat>`:
   exclude regex;
   any-match short-circuits to skip.
- `--ext <ext>`:
   file-extension shorthand (with or without dot).
   Repeatable;
   comma list accepted.
   ANDs with `--include` if both supplied.
- `--type <kind>`:
   entity kinds admitted (`file`,
   `dir`).
   Repeatable.
   Default:
   `file` (dev-server case ignores directory create/delete).
- `--events <list>`:
   comma-separated subset of `add,change,unlink,addDir,unlinkDir`.
   Default:
   all kinds reaching the filter.
- `--hidden` / `--no-hidden`:
   include hidden files and directories (path segments starting with `.`).
   Default:
   `--no-hidden`.
- `--gitignore` / `--no-gitignore`:
   respect each watch root's `.gitignore`.
   Default:
   `--gitignore`.
- `--ignore-file <path>`:
   extra gitignore-format file.
   Repeatable;
   patterns AND with `.gitignore`.
- `--depth <n>`:
   subdirectory recursion cap from each watch root.
   Default:
   unlimited.
- `--no-content-changed`:
   pass byte-identical writes through.
   Default:
   byte-identical writes are skipped.
- `--max-hash-size <bytes>`:
   files above this size bypass the hash compare and always fire.
   Default:
   `16777216` (16 MiB).

### Watcher config flags

These flags affect which events chokidar emits in the first place,
 not
which events fire a restart.

- `--follow-symlinks` / `--no-follow-symlinks`:
   follow symbolic links during traversal.
   Default:
   `--no-follow-symlinks` (safer default;
   opt in for vendored / symlinked source trees).
- `--poll <ms>`:
   switch chokidar to polling mode with this interval (filesystems without inotify,
   e.g. NFS,
   WSL1 on Windows-FS).
   Default:
   native fs events.

### Restart-driver flags

These flags shape how the child is killed and relaunched once a match fires.

- `--debounce <ms>`:
   coalesce events within this window into one restart.
   Default:
   `100`.
- `--no-initial`:
   skip the initial run;
   only restart on events.
   Default:
   run immediately.
- `--clear` / `--no-clear`:
   write `\x1b[2J\x1b[H` to stdout before every spawn (initial and restart).
   Default:
   `--no-clear`.
- `--signal <name>`:
   first signal sent on stop/restart.
   Default:
   `SIGTERM`.
   Accepts `SIGTERM`,
   `SIGINT`,
   `SIGHUP`,
   `SIGUSR1`,
   `SIGUSR2`.
   `--signal SIGHUP` drives soft-reload servers that re-read config without exiting.
- `--stop-timeout <ms>`:
   SIGTERM-then-SIGKILL grace period (escalation is always SIGKILL regardless of `--signal`).
   Default:
   `5000`.
- `--process-group` / `--no-process-group`:
   spawn detached so the child leads its own process group and signal `-pid` so the whole subtree receives the signal.
   Default:
   `--process-group`.
   Turn off when the dev command does not spawn its own subprocesses.

### AND-of-OR-of cap rule

The CLI compiles every flag dimension into a single filter that ANDs across
categories with OR within each.
 Concretely,
 an event fires iff:

```text
(any -w matches event.path) AND
(event.kind ∈ --events list) AND
(event.entity ∈ --type list) AND
(--hidden OR no segment of event.path starts with '.') AND
(--no-gitignore OR no loaded gitignore pattern matches event.path) AND
(no -e glob matches OR no -e glob given) AND
(any -i glob matches OR no -i glob given) AND
(no --exclude-regex matches OR no --exclude-regex given) AND
(any --include-regex matches OR no --include-regex given) AND
(event.ext ∈ --ext list OR no --ext given) AND
(NOT --content-changed OR hash differs from cached)
```

Compositions beyond this shape (e.g. `(A AND B) OR (C AND D)`) fall back
to the library API's `filter?` option,
 which accepts a TypeScript
predicate function.
 This is a deliberate design choice;
 see
`HANDOVER.custom-dev-server-watcher.md` for the analysis.

## Library

```ts
import {
  anyFilter,
  composeFilters,
  contentHashFilter,
  extFilter,
  gitignoreFilter,
  globFilter,
  hiddenFilter,
  regexFilter,
  startWatchRestart,
  typeFilter,
  type WatchCtx,
  type WatchEntityType,
  type WatchEvent,
  type WatchEventKind,
  type WatchFilter,
} from '@monochromatic-dev/watch-restart-cli';

const handle = await startWatchRestart({
  paths: ['src/server',],
  command: 'node',
  args: ['src/server/index.ts',],
  extensions: ['.ts',],
  contentChanged: true,
  debounce: 100,
  stopTimeout: 5000,
  killSignal: 'SIGTERM',
  processGroup: true,
},);

// later:
await handle.stop();
```

`filter` accepts a custom predicate that is AND'd with the flag-derived filter.
Use this for boolean compositions the CLI flags cannot express.

Built-in helpers (composable via `composeFilters` for all-of and `anyFilter`
for any-of):

- `contentHashFilter()`:
   byte-equality skip using `ctx.hashCache`;
   configure size cap via `startWatchRestart({ maxHashSize })`.
- `extFilter([...])`:
   extension allowlist (case-insensitive,
   leading dot optional).
- `globFilter({ include?, exclude? })`:
   picomatch globs against `relativePath`.
- `regexFilter({ include?, exclude? })`:
   regex against `relativePath`.
- `typeFilter([...])`:
   entity-kind allowlist (`'file'`,
   `'dir'`).
- `hiddenFilter({ allowHidden? })`:
   drop dotfile path segments unless opted in.
- `gitignoreFilter({ roots, extraFiles? })`:
   async;
   loads `.gitignore` from each root plus extras and rejects matched paths.

## Examples

Watch CSS source for a build task:

```bash
watch-restart -w src/client --ext .css --ext .scss -- mise run build:css
```

Vendored `src/` with symlinked workspace deps:

```bash
watch-restart -w src --follow-symlinks --depth 5 -- node src/index.ts
```

Anchored alternation picomatch cannot express:

```bash
watch-restart -w src --exclude-regex '\.(test|spec|fixture)\.[jt]sx?$' -- node src/index.ts
```

Network-mounted source tree:

```bash
watch-restart -w /mnt/nfs/src --poll 500 -- node src/index.ts
```

Soft-reload server that re-reads config on SIGHUP:

```bash
watch-restart -w src --signal SIGHUP -- node src/index.ts
```

Clear the terminal on each restart:

```bash
watch-restart -w src --clear -- node src/index.ts
```

## Choices

- **chokidar 5** for file watching,
   not `@parcel/watcher` (native install
  surface),
   `fabiospampinato/watcher` (smaller production track record),
  native `fs.watch` (atomic-save and chunked-write handling is on us),
   or
  watchman (daemon designed to amortise across many tools;
   one project is the
  wrong scale).
- **`ignore` (kaelzhang/node-ignore) for `.gitignore` parsing**,
   not a
  hand-rolled matcher.
   Gitignore semantics are subtle (negation `!`,
  anchored leading `/`,
   directory-only trailing `/`,
   double-star `**`)
  and delegating to a battle-tested zero-dep library keeps the
  behaviour predictable and matched to git's own rules.
- **Custom `child_process.spawn` wrapper** for restart,
   not `nodemon`
  (Node-only restart semantics),
   `pm2` (production process manager),
   or
  `node --watch` (watches the import graph,
   no content-hash filter,
  HMR-like semantics wrong for a server with sockets/tokens).
- **Process-group ownership on by default**.
   The default child is spawned
  detached (POSIX `setsid`) and signaled via `-pid` so a dev command that
  itself spawns workers (node `--watch`,
   vite) gets killed together with
  its subtree.
   `--no-process-group` reverts to direct-child signalling.
- **No DSL filter language**.
   Structured CLI flags compile down to a
  TypeScript predicate internally;
   the library API's `filter?` covers cases
  the flag set does not.
   The handover document records the option matrix
  (CEL via `@marcbachmann/cel-js`,
   JSONLogic,
   jq via WASM,
   custom mini-DSL).
- **In-memory content-hash cache**.
   Pre-populated during chokidar's initial
  walk (events before the `ready` event record without restarting);
   after
  `ready` an unknown path is treated as a genuinely new file and fires.
  Persisting the cache to disk is intentionally out of scope.

## Tests

`mise run //package/watch-restart/cli:test:unit` covers the baseline
cases plus the Q6 expansion:

Baseline:

1. Byte-identical write produces no restart.
2. Atomic save (rename `_tmp` → file) has a skipped unit-test placeholder because
   chokidar `atomic` + `awaitWriteFinish` timing is flaky in isolation;
    coverage is
   expected through editord dev-loop integration verification.
3. Two writes inside the debounce window coalesce to one restart.
4. Deletion fires once and clears the file's cache entry.
5. SIGTERM exits the watcher and child cleanly within `--stop-timeout`.
6. `--ext .ts` filters change events on non-`.ts` files.
7. `--exclude '**/*.test.ts'` suppresses change events on excluded files.
8. `--no-content-changed` lets a byte-identical write through.
9. Pre-populate-on-start:
    pre-`ready` events record without restarting;
   subsequent same-hash events skip;
    different-hash events restart.
10. `--no-initial` does not run the child at startup;
     the first qualifying
    event still triggers a start.

Q6 (watchexec parity):

11. `regexFilter` admits anchored alternation (`\.(test|spec|fixture)\.[jt]sx?$`)
    against `relativePath`;
     exclude beats include when both match.
12. `typeFilter` admits / rejects based on `entity` (`'file'` vs `'dir'`);
    default `['file']` ignores `addDir` / `unlinkDir` events.
13. `hiddenFilter` rejects path segments starting with `.`;
     `allowHidden`
    passes them through.
     Extension-only dots (e.g. `index.ts`) are not hidden.
14. `gitignoreFilter` reads each watch root's `.gitignore`,
     parses with
    the `ignore` package,
     and rejects matched paths.
     ENOENT on a configured
    `extraFile` collapses to no-op;
     multi-root patterns AND together.
15. `--depth 1` admits root files and one level of subdirs;
     rejects deeper.
16. `--poll <ms>` switches chokidar to polling mode;
     live events still fire.
17. `--follow-symlinks` walks into symlinked directories;
     default does not.
18. `killSignal: 'SIGHUP'` sends SIGHUP first,
     then SIGKILL escalates.
19. `processGroup: true` (default) routes through the configured
    `processSignal` sink with negative pid;
     the direct-handle path is not taken.
20. `clear: true` runs the configured `writeClear` sink before every spawn
    (initial and restart);
     `clear: false` (default) never runs it.
