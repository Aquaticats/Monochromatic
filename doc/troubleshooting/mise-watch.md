# mise 2026.3.8 `mise watch` drops `--no-meta` / `-J` flags; watchexec 2.5.1 `-j` filter hangs forever on SIGINT due to an Arc cycle keeping `FilterProgs`' channel sender alive; plus saved-token / EADDRINUSE / unnecessary-restart quirks of the editord watch loop

This file groups five independent mise + watchexec issues
that bit the editord dev-server watch loop.
 Each gets its own
canonical section.

---

## Bug 1: `mise watch` drops `--no-meta` and `-J` (filter-prog) flags

### Symptom

`mise watch` advertises `--no-meta` and `-J`/`--filter-prog` in its `--help` output
but silently drops them when constructing the underlying `watchexec` command.

```sh
mise watch -w src/server --no-meta -J @src/server/content-changed.jaq -r -- start:server
```

Debug output shows the actual command:

```toml
DEBUG $ watchexec --restart --watch src/server -- /home/user/.local/bin/mise run start:server
```

`--no-meta` and `-J @src/server/content-changed.jaq` are both missing.

### Root cause

mise 2026.3.8 parses these flags on the CLI side (they appear
in `--help`) but does not forward them to watchexec when it
spawns the child process.
 The `-J` flag is particularly
confusing because mise remaps watchexec's lowercase `-j`
(filter-prog) to uppercase `-J`,
 while taking `-j` for its own
`--jobs`.

### Verified workaround

Call `watchexec` directly instead of `mise watch`,
and run the server command directly (not through `mise run`):

```toml
# mise.toml
[tasks."dev:server"]
# Calls watchexec directly because mise drops --no-meta and -j flags.
# Runs bun directly (not `mise run start:server`) to keep a flat process tree
# so SIGTERM reaches the server process -- see "EADDRINUSE from deep process trees" below.
run = "watchexec -w src/server --no-meta -j @src/server/content-changed.jaq -r -- bun src/server/index.ts"
```

Note the flag difference:
 use lowercase `-j` (watchexec native) not uppercase `-J` (mise alias).

### What does not work

- `mise watch --no-meta`:
   flag parsed but not forwarded to watchexec
- `mise watch -J @file.jaq`:
   same,
   parsed but dropped
- `mise watch --fs-events create,remove,rename,modify`:
   untested but likely same issue

### Verification

Versions under test:

- mise 2026.3.8 linux-x64 (original report)
- mise HEAD `70c2f0ba06bca99417d3f0e416ef0363ec91bf2e` (`v2026.5.11-1-g70c2f0b`),
  re-verified 2026-05-17.
   Same bug shape;
   the forwarding gap is unchanged.
- watchexec 2.5.0 (original) and 2.5.1 (HEAD).

### Why we would file this upstream

1. **Is it really upstream's fault?
   ** Yes;
    advertised but
   silently dropped flags.
2. **Can upstream fix it?
   ** Yes;
    forward the parsed flags to
   the watchexec invocation.
3. **Are they supporting this use case?
   ** Implicitly;
    the
   flags appear in `--help`.
4. **Will they likely fix it?
   ** Plausible.
5. **Have we prototyped a minimal fix?
   ** Yes;
    patch + DEBUG output
   comparison below.
    Diff lives in
   [mise-watch.patch](mise-watch.patch).

Decision:
 worth raising as an issue;
 the calling-watchexec-
directly workaround removes the urgency.

#### Verified DEBUG output comparison

Reproducer scaffolding (used by both runs):

```sh
FRESH=$(mktemp -d)
echo "true" > "$FRESH/minimal.jaq"
cat > "$FRESH/mise.toml" <<'EOF'
[tasks.echo-hi]
run = "echo hi"
EOF
mise trust "$FRESH/mise.toml"
```

Pre-patch (HEAD `70c2f0b`,
 `cargo build --release -p mise`):

```text
$ mise -vv watch -w "$FRESH" --fs-events create,modify -J @"$FRESH/minimal.jaq" echo-hi
DEBUG [src/cli/watch.rs:225] $ watchexec --watch /tmp/tmp.MF9OvLfNoe -- /tmp/.../mise run echo-hi

$ mise -vv watch -w "$FRESH" --no-meta -J @"$FRESH/minimal.jaq" echo-hi
DEBUG [src/cli/watch.rs:225] $ watchexec --watch /tmp/tmp.MF9OvLfNoe -- /tmp/.../mise run echo-hi

$ mise -vv watch -w "$FRESH" -J @"$FRESH/minimal.jaq" echo-hi
DEBUG [src/cli/watch.rs:225] $ watchexec --watch /tmp/tmp.MF9OvLfNoe -- /tmp/.../mise run echo-hi
```

Every filter flag is silently dropped:
 the spawned `watchexec` argv has only `--watch`
and the trailing task command.

Post-patch (same HEAD plus the diff in `mise-watch.patch`):

```text
$ mise -vv watch -w "$FRESH" --fs-events create,modify -J @"$FRESH/minimal.jaq" echo-hi
DEBUG [src/cli/watch.rs:258] $ watchexec --watch /tmp/tmp.MF9OvLfNoe --fs-events create,modify --filter-prog @/tmp/tmp.MF9OvLfNoe/minimal.jaq -- /tmp/.../mise run echo-hi

$ mise -vv watch -w "$FRESH" --no-meta -J @"$FRESH/minimal.jaq" echo-hi
DEBUG [src/cli/watch.rs:258] $ watchexec --watch /tmp/tmp.MF9OvLfNoe --no-meta --filter-prog @/tmp/tmp.MF9OvLfNoe/minimal.jaq -- /tmp/.../mise run echo-hi

$ mise -vv watch -w "$FRESH" -J @"$FRESH/minimal.jaq" echo-hi
DEBUG [src/cli/watch.rs:258] $ watchexec --watch /tmp/tmp.MF9OvLfNoe --filter-prog @/tmp/tmp.MF9OvLfNoe/minimal.jaq -- /tmp/.../mise run echo-hi
```

Both `--no-meta` and `--fs-events` reach watchexec,
 and the jaq filter program is forwarded
as `--filter-prog @file` (watchexec's long form for `-j`;
 mise reserves `-j` for `--jobs`,
so it advertises `-J` to its own users and the patch translates by emitting the unambiguous
long flag).

`--fs-events` is forwarded only when the user-provided set differs from the clap default
(`create,remove,rename,modify,metadata`).
 Forwarding the default unconditionally would
collide with `--no-meta` because watchexec also rejects both being given together.

Tests passing post-patch (`cargo test --release -p mise`,
 filtered to `cli::watch`):

```text
running 4 tests
test cli::watch::tests::merge_dedupes_across_tasks ... ok
test cli::watch::tests::merge_does_not_let_one_task_exclude_anothers_include ... ok
test cli::watch::tests::merge_single_task_splits_pos_and_neg ... ok
test cli::watch::tests::merge_unescapes_literal_bang ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 952 filtered out
```

Broader sanity check (`cargo test --release -p mise --bins`):
 956 / 956 pass.

### Draft upstream issue (kept as reference; revise before filing)

To file against `jdx/mise`:

````md
Title: `mise watch` silently drops `--no-meta`, `--filter-prog`/`-J`, and `--fs-events` when spawning watchexec

Labels: bug

#### Summary

`mise watch` parses several watchexec passthrough flags on its own CLI surface
(they appear in `mise watch --help` under the Filtering option set) but does not
forward them when constructing the underlying `watchexec` argv. Affected flags
observed at HEAD `70c2f0ba06bca99417d3f0e416ef0363ec91bf2e`
(`v2026.5.11-1-g70c2f0b`):

- `--no-meta`
- `--filter-prog` / `-J` (jaq filter program; mise's `-J` short alias for watchexec's `-j` because mise reserves `-j` for `--jobs`)
- `--fs-events` (kernel-level event-type filter)

`src/cli/watch.rs` parses every flag into `WatchexecArgs` via clap derive but the
section that builds the spawned `args: Vec<String>` (around the
`debug!("$ watchexec ...")` site) skips `filter_fs_meta`, `filter_programs`, and
`filter_fs_events` entirely. The end result is that the user-supplied flags are
silently no-ops.

#### Reproducer (no editord plumbing, scratch dir only)

```sh
FRESH=$(mktemp -d)
echo "true" > "$FRESH/minimal.jaq"
cat > "$FRESH/mise.toml" <<'EOF'
[tasks.echo-hi]
run = "echo hi"
EOF
mise trust "$FRESH/mise.toml"
cd "$FRESH"

mise -vv watch -w "$FRESH" --fs-events create,modify -J "@$FRESH/minimal.jaq" echo-hi
# DEBUG $ watchexec --watch /tmp/<scratch> -- /path/to/mise run echo-hi
# (expected: --fs-events create,modify --filter-prog @... are present)

mise -vv watch -w "$FRESH" --no-meta -J "@$FRESH/minimal.jaq" echo-hi
# DEBUG $ watchexec --watch /tmp/<scratch> -- /path/to/mise run echo-hi
# (expected: --no-meta --filter-prog @... are present)
```

#### Suggested fix

Extend the args-building block in `src/cli/watch.rs::Watch::run` to push the
three missing flags after `filter_patterns`:

```rust
if self.watchexec.filter_fs_meta {
    args.push("--no-meta".to_string());
}
if !self.watchexec.filter_fs_events.is_empty()
    && self.watchexec.filter_fs_events.as_slice() != DEFAULT_FS_EVENTS
{
    args.push("--fs-events".to_string());
    args.push(
        self.watchexec
            .filter_fs_events
            .iter()
            .map(|e| e.to_possible_value().expect("FsEvent has ValueEnum names").get_name().to_string())
            .collect::<Vec<_>>()
            .join(","),
    );
}
if !self.watchexec.filter_programs.is_empty() {
    for prog in &self.watchexec.filter_programs {
        args.push("--filter-prog".to_string());
        args.push(prog.clone());
    }
}
```

Where `DEFAULT_FS_EVENTS` mirrors the clap `default_value` so a user who has not
touched `--fs-events` does not get a redundant flag forwarded (and avoids the
`--no-meta` vs `--fs-events` clap conflict that watchexec also enforces).

Emitting `--filter-prog` (the long form) rather than `-j` keeps the subprocess
argv unambiguous; the short form `-j` works equally on watchexec's side but the
long form is self-documenting.

#### Verified in

- mise HEAD `70c2f0ba06bca99417d3f0e416ef0363ec91bf2e` (`v2026.5.11-1-g70c2f0b`),
  release build with `cargo build --release -p mise`.
- Pre-patch: filter flags missing from the spawned `watchexec` argv per the
  `DEBUG $ watchexec ...` log line at `src/cli/watch.rs:225`.
- Post-patch: same DEBUG line shows `--no-meta`, `--fs-events <set>`, and
  `--filter-prog @...` forwarded as expected.
- `cargo test --release -p mise` (filtered to `cli::watch`): 4 / 4 pass.
- Broader mise binary unit suite (`cargo test --release -p mise --bins`):
  956 / 956 pass.
````

---

## Bug 2: Unnecessary restarts on metadata-only or same-content writes

### Symptom

`mise watch -w src/server -r -- start:server` restarts the
server process when file metadata changes (mtime from
`touch`,
 `chmod`) or when a file is written with identical
content (e.g. format-on-save producing the same output,
 or
Ctrl+S without changes).

This causes problems when editing the watched project's own source files
in a tool served by that project (editord editing its own source).
Every save restarts the server,
 killing WebSocket connections and LSP servers.

### Root cause

By default,
 watchexec triggers on all filesystem event types
including `Modify(Metadata(Any))`.
 A bare `touch` or `chmod`
fires this event even though file content is unchanged.

For content-preserving writes (editor saves identical bytes),
the OS reports `Modify(Data(Any))`,
 a real data write event,
because it cannot distinguish "same bytes" from "different
bytes" at the kernel level.

### Verified workaround

Two layers,
 in different places:

**Layer 1:
 `--no-meta` on watchexec**:
 suppresses `Modify(Metadata(Any))`
events at the kernel level.
 Handles `touch`,
 `chmod`,
 and similar
metadata-only changes.

**Layer 2:
 skip-on-identical-content in editord's save handler**:
`saveFile` reads the existing file before writing and returns early when
the new content matches.
 No write happens,
 no fs event fires,
 no restart.
This covers the dogfooding case (editing editord's own source from within
editord),
 which is the practical reason the filter existed.

An earlier version used `-j @content-changed.jaq` on watchexec for the
content-hash check.
 That layer was removed because watchexec hangs on
SIGINT when `-j` is used;
 see "watchexec `-j` filter program hangs on
SIGINT" below.

**Behavior,
 by event:
**

- `touch` (mtime only):
   suppressed by `--no-meta`.
   No restart.
- `chmod` (perms only):
   suppressed by `--no-meta`.
   No restart.
- editord saves identical content:
   `saveFile` returns early without
  writing.
   No fs event,
   no restart.
- editord saves different content:
   `saveFile` writes through,
   watchexec
  fires.
   Restart.
- External editor writes identical bytes (rare):
   watchexec fires.
   Restart.
- External editor writes different bytes:
   watchexec fires.
   Restart.
- `git checkout` (revert):
   watchexec fires.
   Restart.
- New file created:
   watchexec fires.
   Restart.

### Constraints

- The save-side skip only covers writes routed through editord's own
  `saveFile`.
   External editors (vim,
   vscode) doing format-on-save with
  byte-identical output still trigger a restart.
   This is rare in practice;
  the common no-op-save case in this codebase is editord-on-editord.
- Skipping the write also skips the mtime touch.
   Any consumer that keys
  off mtime sees the file as unchanged,
   which matches the semantics.

### Update (2026-05): external-editor case is also covered now

editord's `dev:server` runs through `@monochromatic-dev/watch-restart-cli`,
which maintains an in-process content-hash cache and re-hashes every
file on each filesystem event.
 A write with identical bytes (whoever
issued it:
 editord's `saveFile`,
 vim's `formatprg`,
 vscode's
`editor.formatOnSave`,
 `cp -p` from a sibling shell) produces a cache
hit and no restart.
 The save-side skip in `saveFile` stays;
 it is
strictly cheaper than the watch-side compare (no write,
 no event,
 no
hash) for the editord-on-editord path it already covered.

### Why we do not file this upstream

1. **Is it really upstream's fault?
   ** No;
    the OS reports
   `Modify(Data(Any))` for "same bytes" writes because the
   kernel cannot tell them apart from "different bytes".
    The
   watcher behaves correctly.
2. **Can upstream fix it?
   ** Not in watchexec;
    the data is
   not there to filter on.
3. **Are they supporting this use case?
   ** Yes;
    the `-j`
   filter program exists for exactly this.
4. **Will they likely fix it?
   ** N/A.
5. **Have we prototyped a minimal fix?
   ** N/A.

Decision:
 no upstream report.
 Fix lives at our boundary
(saveFile early-return + watch-restart's content-hash cache).

---

## Bug 3: Server restart generates fresh auth token (client loses connection)

### Symptom

When the dev server restarts (via watchexec or manual
kill+restart),
 the new instance generates a new
`crypto.randomUUID()` auth token.
 The browser client has the
old token embedded in its WebSocket URL and cannot reconnect;
every reconnect attempt fails with "unauthorized".

### Root cause

The auth token was generated in-memory on every startup with
no persistence.
 The client's `EditorWsClient` reconnects with
exponential backoff but always uses the original token from
`#wsUrl` (set once in the constructor).

### Verified workaround

A token file at `$TMPDIR/editord-<port>.token` persists the token across restarts:

1. **On startup**:
    check if the token file exists and its mtime is within 3 seconds.
   If so,
    reuse the token (auto-restart detected).
    Otherwise generate a fresh UUID.
2. **While running**:
    re-write the token file every 1 second to keep its mtime fresh.
3. **On SIGTERM** (auto-restart from watchexec):
    stop the mtime touch interval
   but **keep the token file** so the next instance finds it fresh.
4. **On SIGINT** (user Ctrl+C):
    delete the token file since no restart is expected.

The 3-second freshness window is wide enough to cover the gap between
watchexec killing the old process and spawning the new one (typically < 500ms)
but narrow enough that a stale file from a crashed process won't be reused
on the next manual start.

### What does not work

- Deleting the token file on both SIGTERM and SIGINT;
   the new process starts
  after the old one exits,
   and if the file is deleted,
   it has no token to reuse.
  This was the initial implementation bug:
   `handleShutdown` ran `cleanupToken()`
  which deleted the file before the new process could read it.
- Using the same cleanup function for both signals;
   SIGTERM must preserve the
  file,
   SIGINT must delete it.
   Split into `handleSigterm` and `handleSigint`.

### Why we do not file this upstream

Internal to editord;
 no external upstream.
 5 constraints
walked:
 not applicable.
 Decision:
 no upstream report.

---

## Bug 4: EADDRINUSE from deep process trees on restart

### Symptom

After switching to `watchexec` directly,
 restarts fail with
`EADDRINUSE` because the previous server process still holds
the port.

### Root cause

When the inner command is `mise run start:server`,
 the
process tree is:

```text
watchexec → mise → sh → bun src/server/index.ts
```

watchexec sends SIGTERM to its direct child (`mise`) on
restart.
 mise exits,
 but the signal does not propagate
through the task shell to `bun`.
 The bun process orphans and keeps
the port bound.

### Verified workaround

Run `bun` directly as the inner command so watchexec's SIGTERM reaches it:

```text
watchexec -w src/server -r -- bun src/server/index.ts
```

Process tree:
 `watchexec → bun`.
 SIGTERM goes directly to bun,
the signal handler runs,
 the port is released before the new instance starts.

### Update (2026-05)

editord's `dev:server` migrated off watchexec to
`@monochromatic-dev/watch-restart-cli`.
 The new tree is even
shallower (`watch-restart → bun`,
 no intermediate wrapper at all) because
the watcher spawns the bun child directly via `node:child_process.spawn`
with `stdio: 'inherit'`.
 SIGTERM still reaches bun by construction;
 the
mitigation principle (flat process tree) is preserved by tool choice
rather than by careful argv composition.

### What does not work

- `watchexec -r -- mise run start:server`:
  SIGTERM does not propagate
  through the `mise → sh → bun` chain,
   leaving orphaned bun processes

### Why we do not file this upstream

1. **Is it really upstream's fault?
   ** Borderline.
    The task shell
   does not propagate signals to child processes by default;
   mise inherits that behaviour.
    Both are documented.
2. **Can upstream fix it?
   ** mise could opt to spawn child
   processes that forward signals;
    the task shell could change its
   default.
    Both are large behaviour changes.
3. **Are they supporting this use case?
   ** Yes for direct
   `bun` invocations;
    the deep-tree case is a side effect.
4. **Will they likely fix it?
   ** No targeted fix expected.
5. **Have we prototyped a minimal fix?
   ** Bypass at our
   boundary:
    flat process tree via direct watchexec/bun
   invocation.

Decision:
 no upstream report.
 Mitigation lives in the task
definition.

---

## Bug 5: watchexec `-j` filter program hangs on SIGINT (Ctrl+C does not return)

### Symptom

`mise run //package/desktop-daemon/editord:dev` does not return to the
terminal on Ctrl+C.
 Three processes survive the signal and block the
terminal indefinitely:

```text
mise run //package/desktop-daemon/editord:dev
└── sh -c watchexec -w src/server --no-meta -j @src/server/content-changed.jaq -r -- bun src/server/index.ts
    └── watchexec ...
```

`bun` and the other two parallel tasks (`watch:build:js:client`,
`watch:build:css`) exit cleanly.
 `watchexec` does not.
 The wrapping shell and
the parent `mise` wait on it,
 so the terminal cannot be released.

Subsequent SIGINT and SIGTERM signals to the hung `watchexec` are ignored.
Only SIGKILL kills it.

### Root cause

`watchexec` 2.5.1 enters a permanent hang on SIGINT when the
`-j` (filter program) flag is used.
 The bug is reproducible
in isolation,
 with no editord plumbing:

```sh
mkdir -p /tmp/quietdir && echo "true" > /tmp/minimal.jaq
watchexec -w /tmp/quietdir -j @/tmp/minimal.jaq -- sleep 9999
# Press Ctrl+C
# `[Waiting 10s for processes to exit before stopping...]` is printed
# `sleep` exits within a second
# `watchexec` never exits; further Ctrl+C does nothing
```

Bisection of flags confirms `-j` is the trigger.
 First SIGINT exits the
following invocations cleanly:

- `watchexec -w DIR -r -- CMD`
- `watchexec -w DIR --no-meta -r -- CMD`

First SIGINT hangs the following invocations:

- `watchexec -w DIR -j @file.jaq -- CMD`
- `watchexec -w DIR -j @file.jaq -r -- CMD`
- `watchexec -w DIR --no-meta -j @file.jaq -r -- CMD` (the production form)
- `watchexec -w DIR -j 'true' -- CMD` (**inline jaq form,
   same hang**)

watchexec's `--help` documents `-j` accepts either an inline jaq expression or,
 if the argument starts with `@`,
 a path to a file containing a jaq program.
 The hang affects **both forms** because the cycle lives in
`FilterProgs::new` (`crates/cli/src/filterer/progs.rs:66-115`),
 which is called whenever `-j` is present regardless of where the program comes from.
 Reproduced 2026-05-11 with watchexec 2.5.1:
 a process running `watchexec -w /tmp/x -j 'true' -- sleep 9999` survives the first SIGINT and only dies on SIGKILL.

The watchexec docs (CLI `--help` and the discussions link at
<https://github.com/watchexec/watchexec/discussions/592>)
advertise the inline form on equal footing with `@file`.
 There is no documented warning that either form hangs on SIGINT.
 The draft upstream issue at the end of this section should call out that inline jaq is affected too,
 so an upstream fix covers both code paths.

#### Source-level trace

`FilterProgs::new` in `crates/cli/src/filterer/progs.rs:66-115` (v2.5.1)
spawns a blocking task whose only exit path is the channel closing:

```rust
let task = spawn_blocking(move || {
    'chan: while let Some((event, sender)) = receiver.blocking_recv() {
        // run jaq programs
    }
    Ok(()) as miette::Result<()>
});

tokio::spawn(async {
    match task.await { /* log */ }
});
```

`receiver.blocking_recv()` returns `None` only when every `Sender` is
dropped.
 The matching `Sender` lives in `FilterProgs.channel`,
 held by
`Arc<WatchexecFilterer>` in `Config.filterer`
(`ChangeableFilterer = Arc<RwLock<Arc<dyn Filterer>>>`).

A strong reference cycle keeps `Arc<Config>` alive past shutdown:

```text
Arc<Config>
  .action_handler  (ChangeableFn = Arc<RwLock<Arc<dyn Fn>>>)
    closure        registered at crates/cli/src/config.rs:206-209
      captures     state = state.clone()  (Arc<InnerState>)
        Arc<InnerState>
          .watchexec  OnceLock<Arc<Watchexec>>   (state.rs:45)
            Arc<Watchexec>
              .config  Arc<Config>               (watchexec.rs:138-203)
                ^─────── cycles back ────────────┘
```

`InnerState.watchexec` is set inside `run_watchexec` (`crates/cli/src/lib.rs:42-45`)
after the closure has already captured `state`.
 The cycle is established
before the first event arrives.

When SIGINT fires:

1. The action handler calls `quit(action)` which schedules
   `action.quit_gracefully(stop_signal, stop_timeout)`
   (`crates/cli/src/config.rs:298-322`).
2. `bun` receives SIGTERM,
    runs its shutdown,
    exits.
3. The action worker breaks out of its loop
   (`crates/lib/src/action/worker.rs:63-86`).
4. The main task ends and runs `tasks.shutdown().await`
   (`crates/lib/src/watchexec.rs:191-193`),
    aborting the remaining async
   workers.
5. `wx.main().await` resolves,
    `run_watchexec` returns,
    `run` returns,
   the async block in `main()` completes,
    `block_on` returns.
6. The `tokio::runtime::Runtime` drops.
    Drop waits for `spawn_blocking`
   tasks to finish naturally (they cannot be aborted).
7. The `FilterProgs` blocking thread is still parked in
   `receiver.blocking_recv()` because the cycle is keeping `Arc<Config>`
   alive,
    which keeps `Arc<WatchexecFilterer>` alive,
    which keeps the
   `Sender` alive,
    which keeps the channel open.
8. The runtime never releases its threads;
    the process never exits.

Without `-j`,
 no `FilterProgs` and no `spawn_blocking` task is created,
so the runtime aborts the async tasks and exits despite the same cycle
existing on the closure side.

### Verification

Versions under test:

- watchexec 2.5.1 linux-x64 (from mise install)
- watchexec/watchexec HEAD at commit `9d8e3443` (post-2.5.1;
  both cited files are byte-identical to v2.5.1)
- bun 1.3.13 linux-x64
- mise 2026.5.0 linux-x64

### Verified workaround

Resolved in two stages.

Stage 1 (2026-04,
 commit `27051b66`):
 remove `-j` from `dev:server`;
 move
the content-equality check into editord's `saveFile`.
 The `--no-meta`
flag stays.
 Covered the editord-on-editord dogfooding case but external
editors doing byte-identical format-on-save still triggered a restart.

Stage 2 (2026-05):
 editord's `dev:server` migrates off watchexec
entirely to `@monochromatic-dev/watch-restart-cli`.
 The new
package owns an in-process content-hash cache (suppressing byte-identical
writes regardless of writer,
 including external editors' format-on-save),
spawns the bun child directly via `node:child_process` (one process
layer;
 SIGTERM reaches the child without traversing a wrapper),
 and has
no jaq filter program at all;
 so neither the SIGINT hang nor the deep-
tree EADDRINUSE failure mode can recur for this task.
 The watchexec bug
remains real;
 the editord loop simply no longer touches the code path.

The "mise watch drops --no-meta and -J" section above still stands as a
general mise/watchexec interaction bug;
 the editord task's escape from
it was to call watchexec directly (stage 1) and then to leave watchexec
behind (stage 2).

Earlier alternatives considered:

1. Keep `-j` and recover the terminal manually after Ctrl+C:
   `pkill -9 -f 'watchexec -w src/server'`.
    Rejected because every dev
   session needs manual cleanup.
2. Drop `-j` with no replacement filter.
    Rejected because the dogfooding
   case (editord editing editord's source) restarts on every Ctrl+S,
    even
   when no content changed,
    which kills WebSocket connections and LSP
   servers.
3. Custom TypeScript watcher replacing watchexec entirely.
    Initially
   rejected as over-scoped (stage 1);
    revisited and adopted in stage 2
   once the external-editor format-on-save case and the architectural
   failure modes (SIGINT hang,
    deep-tree signal propagation) were
   weighed against the implementation cost.
    Lives at
   `package/watch-restart/cli/`;
    see its README for the chokidar
   - custom `child_process.spawn` rationale.

### What does not work

- Sending a second or third SIGINT:
   the action handler's
  `quit_again.fetch_add` escalation (`config.rs`:
  298-322) is gated on the
  action worker receiving a fresh event,
   but the worker is already past
  the loop,
   blocked on graceful-shutdown await.
- Sending SIGTERM to the hung watchexec process:
   ignored for the same
  reason.
   SIGKILL is the only signal that works.
- `--stop-timeout 0`:
   addresses how long watchexec waits for the child
  to exit,
   not the runtime shutdown wait on the blocking thread.
- Upgrading watchexec:
   the relevant files (`crates/cli/src/filterer/progs.rs`,
  `crates/cli/src/config.rs:206-209`,
   `crates/cli/src/state.rs:45`) are
  byte-identical between v2.5.1 and the current HEAD.

### Why we would file this upstream

All 5 constraints hold:

1. **Is it really upstream's fault?
   ** Yes;
    the Arc cycle
   keeping `FilterProgs`' channel sender alive past shutdown
   is the root cause.
2. **Can upstream fix it?
   ** Yes;
    two concrete patches sketched
   in the draft below (Weak capture in the action handler,
    or
   explicit Drop/Notify shutdown for FilterProgs).
3. **Are they supporting this use case?
   ** Yes;
    `-j` is a
   first-class CLI feature.
4. **Will they likely fix it?
   ** Plausible;
    watchexec is
   actively maintained.
5. **Have we prototyped a minimal fix?
   ** Yes;
    the Weak-capture
   variant from the draft.
    See "Verified prototype" below.

Decision:
 worth filing.

### Verified prototype

Fresh clone of `https://github.com/watchexec/watchexec.git` at HEAD
`9d8e3443ee5fbbf07baa0e0bff2c2c63d40f1a4f` (the post-2.5.1 commit cited in
"Source-level trace";
 cited files byte-identical to v2.5.1).
 Two pre/post
binaries built at `target/release/watchexec`,
 scripted SIGINT-hang
reproducer at `repro/sigint-check.sh` inside the prototype workspace.

Pre/post timings (deadline +6s after SIGINT,
 SIGKILL on overrun):

```text
[prepatch/inline] HANG: alive 5.67s after SIGINT; killing with SIGKILL
[prepatch/file]   HANG: alive 5.87s after SIGINT; killing with SIGKILL
[postpatch/inline] OK: exited within 0.21s after SIGINT (rc=0)
[postpatch/file]   OK: exited within 0.00s after SIGINT (rc=0)
```

Control runs without `-j` exit in 0.00s both pre- and post-patch,
 confirming
the patch is targeted (no regression on the non-`-j` shutdown path).

`cargo test --workspace --release` post-patch:
 all suites pass
(`watchexec_signals` 5/5,
 `watchexec_supervisor` 8/8,
 doc-tests across
`watchexec_cli`,
 `watchexec_events`,
 `watchexec_filterer_globset`,
`watchexec_filterer_ignore` clean).

Full patch text in `mise-watch.patch` (sibling file),
 section "Bug 5".
 Two hunks in `crates/cli/src/config.rs`:
 change
`make_config`'s `let state = state.clone();` to `let state =
Arc::downgrade(state);`,
 and rewrite the matching `let state =
state.clone();` inside `on_action_async` to `let Some(state) =
state.upgrade() else { debug!(...); return Box::new(async move { action })
};`.
 Comments inline name the cycle being broken and the upgrade-fails
path.

### Draft upstream issue (kept as reference; revise before filing)

To file against `watchexec/watchexec`:

````markdown
Title: `-j` (filter program) causes process to hang forever on SIGINT after the command exits

Labels: bug

#### Reproduction

The hang affects both forms of `-j` (file and inline). The CLI `--help`
documents both on equal footing; neither warns about the SIGINT issue.

File form:

```sh
mkdir -p /tmp/quietdir && echo "true" > /tmp/minimal.jaq
watchexec -w /tmp/quietdir -j @/tmp/minimal.jaq -- sleep 9999
```

Inline form (also hangs):

```sh
mkdir -p /tmp/quietdir
watchexec -w /tmp/quietdir -j 'true' -- sleep 9999
```

In either case, press Ctrl+C: `[Waiting 10s for processes to exit before
stopping...]` is printed, `sleep` exits within a second, but `watchexec`
never exits. Further SIGINT/SIGTERM signals are ignored; only SIGKILL
terminates the process.

Without `-j`, the same invocation (`watchexec -w /tmp/quietdir -- sleep
9999`) exits cleanly on the first SIGINT.

#### Diagnosis

`FilterProgs::new` in `crates/cli/src/filterer/progs.rs:66-115` spawns a
blocking task that exits only when the channel closes:

```rust
let task = spawn_blocking(move || {
    while let Some((event, sender)) = receiver.blocking_recv() { ... }
    Ok(())
});
```

The matching `Sender` lives through `Arc<WatchexecFilterer>` ->
`Config.filterer` -> `Arc<Config>`. A reference cycle between
`Arc<Config>`, the action-handler closure, and the captured
`Arc<InnerState>` (which holds `OnceLock<Arc<Watchexec>>` in
`crates/cli/src/state.rs:45` and is filled inside `run_watchexec` in
`crates/cli/src/lib.rs:42-45`) keeps `Arc<Config>` alive past shutdown.
The `Sender` never drops, the receiver never returns `None`, the blocking
thread never exits, and `Runtime::drop` waits on it indefinitely.

Without `-j` no blocking task is created, so the runtime exits despite
the same cycle.

#### Verified in

- watchexec 2.5.1 linux-x64
- HEAD at `9d8e3443`; the cited files are byte-identical to v2.5.1.

#### Suggested fix

Either break the cycle or give `FilterProgs` an explicit shutdown path:

- In `crates/cli/src/config.rs:206-209`, capture `Arc::downgrade(&state)`
  instead of `state.clone()` and upgrade inside the closure. The action
  handler runs only while the runtime is live, so a `Weak` upgrade will
  always succeed during normal operation, and the closure will not pin
  `Arc<InnerState>` past shutdown.
- Or, in `FilterProgs`, hold the `Sender` in a structure with a `Drop`
  impl that calls `mpsc::Sender::downgrade` or sends a sentinel to break
  out of the blocking loop. A `tokio::sync::Notify` wired into the loop
  would also work.

#### Verified patch (Weak capture)

Applies cleanly to HEAD `9d8e3443`. Two hunks in `crates/cli/src/config.rs`;
the second adds a `state.upgrade()` early return that runs only during
shutdown:

```diff
--- a/crates/cli/src/config.rs
+++ b/crates/cli/src/config.rs
@@ -98,7 +98,12 @@ pub fn make_config(args: &Args, state: &State) -> Result<Config> {
        let clear = args.output.screen_clear;

        let emit_events_to = args.events.emit_events_to;
-       let state = state.clone();
+       let state = Arc::downgrade(state);

        if args.only_emit_events {
                config.on_action(move |mut action| {
@@ -206,7 +211,14 @@ pub fn make_config(args: &Args, state: &State) -> Result<Config> {
        config.on_action_async(move |mut action| {
                let add_envs = add_envs.clone();
                let command = command.clone();
-               let state = state.clone();
+               let Some(state) = state.upgrade() else {
+                       debug!("InnerState dropped before action; skipping handler");
+                       return Box::new(async move { action });
+               };
                let queued = queued.clone();
                let quit_again = quit_again.clone();
                let paused = paused.clone();
```

(Source comments naming the cycle being broken are kept in the full
sibling patch at `mise-watch.patch`; trimmed here for
issue-readability.)

#### Verification

Scripted SIGINT-hang reproducer; deadline +6s after SIGINT, SIGKILL on
overrun:

```text
prepatch  inline: HANG  (alive 5.67s after SIGINT; SIGKILL fired)
prepatch  file:   HANG  (alive 5.87s after SIGINT; SIGKILL fired)
postpatch inline: OK    (exited within 0.21s of SIGINT, rc=0)
postpatch file:   OK    (exited within 0.00s of SIGINT, rc=0)
```

`cargo test --workspace --release`: all suites pass post-patch
(`watchexec_signals` 5/5, `watchexec_supervisor` 8/8, doc-tests clean).
Control runs without `-j` exit in 0.00s pre- and post-patch.
````
