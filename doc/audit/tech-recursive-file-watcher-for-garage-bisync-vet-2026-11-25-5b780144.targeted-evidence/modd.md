# Targeted evidence for modd 0.8

Status:
 agent-drafted 2026-11-25T18:32:33-05:00;
 incomplete,
 not reviewed by anyone else,
 not approved by the owner.
It records source and tracker evidence for one candidate of the vet report
`doc/audit/tech-recursive-file-watcher-for-garage-bisync-vet-2026-11-25-5b780144.md`.
It does not score, rank or recommend,
and it changes nothing in the vet report.
Everything below was read or queried;
nothing from the candidate was installed, built, tested, scripted, run or put in a container,
so no record here is a measured runtime result.

## Subject

The candidate is modd,
repository `cortesi/modd`,
release tag `v0.8`,
described in the vet report's ledger entry "modd 0.8".
The owner's request that starts the vet,
quoted from `doc/planning/garage-bisync-run-on-file-change.md`:

> Make my Garage setup trigger a manual sync whenever something changes on relevant dirs.

The governing skill is `.claude/skills/choosing-technology/SKILL.md`;
the sections applied here are "Lifecycle",
"Screening hard gates",
"Evidence records",
"Open-source technology gates",
"Native, Wasm, and prebuilt overlay",
and "Maintenance audit".

## Clone and revision

- Clone path: `/home/user/temp/agent/modd-source-for-garage-watcher-vet-2026-11-25`.
- Clone command, from the parent directory, exit status 0:

  ```bash
  gh repo clone cortesi/modd \
    "/home/user/temp/agent/modd-source-for-garage-watcher-vet-2026-11-25" \
    -- --depth 1 --branch v0.8
  ```

- Commit the tag points to: `b0c08baa4ed03a83ba48050774df55f00bef00cb`,
  committer date `2019-01-21T08:10:17+13:00`,
  subject "Update changelog, bump version"
  (`git log -1`, `git tag --points-at HEAD` printed `v0.8`, `git describe --tags` printed `v0.8`).
- The clone was deleted when this run ended, under its own scratch-cleanup rule,
  so a reader who wants to check a line re-clones the tag with the command above;
  every path below is relative to the root of that tree.
- Repository metadata at access date:
  MIT in the `license.spdx_id` field,
  2967 stars,
  not archived,
  `pushed_at` `2026-06-21T09:26:43Z`,
  42 open issues and 9 open pull requests
  (`gh api repos/cortesi/modd`, `gh api -X GET search/issues ...`).
- Every `path:line` below is relative to that clone path.
  The vendored dependency copies inside it are part of the released tree.
- Access date for every URL and API record below: 2026-11-25,
  on CachyOS,
  Linux 7.3.5-2-cachyos,
  x86_64,
  in `/home/user/Monochromatic` on branch `main`.

## Repository-wide search for this candidate's name

Commands, run from `/home/user/Monochromatic`:

```bash
rg -c -i '\bmodd\b' \
  --glob '!doc/audit/tech-recursive-file-watcher-for-garage-bisync-vet-2026-11-25-5b780144.md' \
  --glob '!doc/planning/garage-bisync-run-on-file-change.md' .
```

That printed nothing and exited 1: no match anywhere in the checkout,
which covers `doc/`, `package/`, `package-paused/`, `.agents/`, `.claude/`,
`toolchain.lock.json`, `mise.toml` files and every README.

- `rg -l -i 'cortesi' .` exit status 0, one hit:
  `doc/audit/tech-recursive-file-watcher-for-garage-bisync-vet-2026-11-25-5b780144.md`.
- `find . -path ./.git -prune -o -iname '*modd*' -print` printed nothing:
  no file or folder in the repository is named after this candidate.

Verdicts found for this candidate, as counterevidence:

- `doc/planning/garage-bisync-run-on-file-change.md:173`,
  owner answer, round 5,
  keeps modd in the vet:
  "the third-party vet goes on with Watchman, onchange, modd and directory-watcher".
  No verdict against modd is recorded there or anywhere else in the repository.
- In `doc/audit/pitchfork-jdx.md` the word does not appear at all,
  so the earlier watchexec judgement has no modd counterpart.

So the repository records no prior verdict on modd,
and none of the recorded verdicts on other tools names it.

## Hard constraint: open source with inspectable source

- Candidate and version: modd `v0.8`, commit `b0c08baa`.
- Claim: the release is open source and its source is inspectable.
- Gate: vet report "Hard constraints", first bullet.
- Status: hard gate, pass.
- Primary source: `LICENSE:1-3`,
  "The MIT License (MIT)" and "Copyright (c) 2015 Aldo Cortesi <aldo@corte.si>",
  and the full MIT text at `LICENSE:5-21`.
- Source for the released version: `git tag --points-at HEAD` at the clone reports `v0.8`,
  so the tag exists and resolves to that commit,
  and `modd.go:19` carries `const Version = "0.8"`.
- Access date: 2026-11-25.
- Corroboration: `gh api repos/cortesi/modd` reports `"license":"MIT"`.
- Outcome: pass.
  The release tag is public,
  the tree it names is readable in full,
  and MIT settles reuse.

## Hard constraint: runs an arbitrary command

- Candidate and version: modd `v0.8`.
- Claim: modd runs `systemctl --user start rclone-bisync-garage.service`
  or any other command the config names.
- Gate: vet report "Hard constraints", second bullet.
- Status: hard gate, pass, on the reading that a shell string is an arbitrary command.
- Primary source: `prep.go:44-89`, `RunPreps`,
  which for each `b.Preps` entry calls `RunProc(cmd, sh, b.InDir, ...)` at `prep.go:71`,
  and `prep.go:24-41`, `RunProc`,
  whose doc comment at `prep.go:24` reads
  "runs a process to completion, sending output to log".
  Commands come from `modd.conf` blocks parsed by `conf.Parse` (`modd.go:79-84`).
- Access date: 2026-11-25.
- Corroboration: `README.md` section "Blocks" and "Prep commands".
- Outcome: pass,
  with the execution shape recorded under "Capability: how the command runs".

## Hard constraint: Linux x86-64, macOS arm64 and Windows x64

- Candidate and version: modd `v0.8`.
- Claim: the same watcher binary runs on all three targets.
- Gate: vet report "Hard constraints", third bullet,
  and the multi-platform overlay in "Base components and overlays".
- Status: hard gate,
  open.
  The watching backend exists in source for all three systems;
  the released artifacts cover Linux x86-64 and Windows x64,
  and macOS only as a 64-bit Intel build.
- Access date: 2026-11-25.
- Primary sources, backend per system:
  - Linux: `vendor/github.com/rjeczalik/notify/watcher_inotify.go:5` build tag `// +build linux`,
    `newWatcher` at `:54`, inotify through `unix.InotifyAddWatch` at `:92`.
  - macOS: `vendor/github.com/rjeczalik/notify/watcher_fsevents.go:5`
    build tag `// +build darwin,!kqueue`,
    `newWatcher` at `:182`,
    and it implements the recursive watcher interface
    (`RecursiveWatch` at `:249`, `RecursiveRewatch` at `:271`,
    interface declared at `vendor/github.com/rjeczalik/notify/watcher.go:49-56`).
    The FSEvents path is cgo:
    `vendor/github.com/rjeczalik/notify/watcher_fsevents_cgo.go` exists in the tree.
  - Windows: `vendor/github.com/rjeczalik/notify/watcher_readdcw.go:5`
    build tag `// +build windows`,
    `newWatcher` at `:268`,
    recursive support through `RecursiveWatch` at `:288`.
  - kqueue, for the BSDs and for a kqueue build on macOS:
    `vendor/github.com/rjeczalik/notify/watcher_kqueue.go:5`,
    `// +build darwin,kqueue dragonfly freebsd netbsd openbsd`.
  - Any other system gets a stub that fails:
    `vendor/github.com/rjeczalik/notify/watcher_notimplemented.go:5-15`,
    `errors.New("notify: not implemented")`.
- Counterevidence on macOS arm64:
  the only macOS asset is `modd-0.8-osx64.tgz`,
  and the build script that produced it maps `osx64` to `darwin`/`amd64`
  (`godist` `gobuild:15`, see "Installed-artifact provenance" below),
  so it is an Intel build.
  `README.md:25-26` states release coverage as
  "released for OSX, Windows, Linux, FreeBSD, NetBSD and OpenBSD" with no architecture list.
  Open issue #132, "modd immediately killed on macOS ARM64 (Apple Silicon) with no useful error message",
  created `2025-05-16T15:38:13Z`, updated `2025-05-18T02:43:12Z`,
  reports `modd` built with
  `go install github.com/cortesi/modd/cmd/modd@latest` being killed on Sonoma arm64.
  The reporter's follow-up, `2025-05-18T02:43:11Z`, quotes the crash report:
  "SIGKILL (Code Signature Invalid) / Termination Reason: Namespace CODESIGNING, Code 2 Invalid Page",
  and says the fix was an ad-hoc signature,
  `codesign --force --deep --sign - /Users/can/Development/go/bin/modd`.
  So the kill was macOS rejecting an unsigned or invalidly signed binary,
  not a defect in modd's arm64 code;
  the release assets are equally unsigned, since the release carries no signature of any kind.
  Issue #119, comment `2022-12-28T13:55:12Z`, records that a macOS source build needs
  `CGO_ENABLED=1` because of the FSEvents binding.
- Outcome: open.
  Source supports all three systems,
  Linux x86-64 and Windows x64 have matching release artifacts,
  and macOS arm64 has neither an artifact nor a recorded successful run in the tracker.

## Hard constraint: recursion, including folders created after start

- Candidate and version: modd `v0.8`.
- Claim: every folder under the watched root is watched,
  including folders created after start,
  on Linux.
- Gate: vet report "Hard constraints", fourth bullet,
  and the owner's round 1 answer "every folder and every folder created later".
- Status: hard gate,
  pass on source,
  with an open tracker item that asserts the opposite for an older tree.
- Primary source, the four links in the chain:
  - `modd.go:182-193`:
    include patterns from the config are handed to the watcher,
    `watcher, err := moddwatch.Watch(currentDir, ipatts, []string{}, lullTime, modchan)`.
  - `vendor/github.com/cortesi/moddwatch/watch.go:411-418`:
    for each base directory the package calls
    `notify.Watch(filepath.Join(p, "..."), evtch, notify.All)`,
    where the trailing `...` is the recursive watch form
    documented at `watch.go:376-402`.
  - `vendor/github.com/rjeczalik/notify/tree.go:15-21`:
    a backend that is not natively recursive is wrapped in a nonrecursive tree.
  - `vendor/github.com/rjeczalik/notify/tree_nonrecursive.go:206-233`, `watchrec`,
    walks the tree and adds one watch per directory at startup.
    The walk is `node.AddDir` at `vendor/github.com/rjeczalik/notify/node.go:63-90`,
    which descends real directories and skips symlinks at `:86`.
- The code that adds a watch for a folder created after start,
  Linux path, is `vendor/github.com/rjeczalik/notify/tree_nonrecursive.go:80-101`:

  ```go
  func (t *nonrecursiveTree) internal(rec <-chan EventInfo) {
      ...
      err := nd.Add(ei.Path()).AddDir(t.recFunc(eset))
      ...
  ```

  It is fed by `dispatch` at `:66-75`,
  which forwards every `Create` event that is a directory on a recursive watchpoint:

  ```go
  if !isrec || ei.Event() != Create {
      return
  }
  if ok, err := ei.(isDirer).isDir(); !ok || err != nil {
      return
  }
  t.rec <- ei
  ```

  `isDir` for inotify is `vendor/github.com/rjeczalik/notify/event_inotify.go:75`,
  `e.sys.Mask&unix.IN_ISDIR != 0`,
  and `Create` is encoded to `IN_CREATE|IN_MOVED_TO` at `:298-301` of `watcher_inotify.go`.
- Counterevidence: issue #44, "Improve Linux recursive watch",
  opened `2017-11-17T19:55:02Z`, updated `2017-11-17T19:55:02Z`, still open.
  Its body states, of the notify package:

  > The rjeczalik/notify package half-hides this from us by walking the tree on startup
  > and adding a watch for every directory.
  > However, it doesn't automatically add watches fo directories created after startup,
  > so we don't get notified of events within that directory.

  The issue was written against the tree as vendored in November 2017;
  the vendored copy at `v0.8` pins
  `github.com/rjeczalik/notify` revision `629144ba06a1c6af28c1e42c228e3d42594ce081`
  (`Gopkg.lock:74-80`) and does contain the `internal` watch-adder above,
  so the statement is stale for this revision,
  but the issue was never closed and no record confirms it.
  The same body names the remaining risk:
  "it's not clear to me that we can do this entirely without races",
  which the `internal` loop does not close:
  a directory created and written into before its watch is added loses those events.
- Two further limits on the recursion:
  only base directories derived from include patterns are watched,
  so a block pattern narrower than the tree watches only that subtree
  (`vendor/github.com/cortesi/moddwatch/watch.go:334-370`, `baseDirs`);
  and symlinked directories are not traversed
  (`README.md:184-200`, "Symlinks"; `node.go:86`).
- Outcome: pass on source for Linux,
  with the race window and the stale open issue recorded,
  and with no runtime confirmation,
  since this run may not run the candidate.

## Hard constraint: never interrupts a running command

- Candidate and version: modd `v0.8`.
- Claim: a run of the command finishes;
  a change never kills it.
- Gate: vet report "Hard constraints", sixth bullet,
  owner answer, round 4.
- Status: hard gate,
  pass for a `prep:` command,
  fail for a `daemon:` command.
- Primary source:
  - prep commands run in the block goroutine and the next block waits:
    `modd.go:109-146`, `runBlock` calls `RunPreps` at `:132-138` and only then `dpen.Restart()` at `:145`;
    `prep.go:24-41` runs each command through `ex.Run(log, true)` at `:32`,
    which waits for the readers and then `cmd.Wait()` (`shell/shell.go:127-148`).
    No path in that chain sends a signal to a prep process.
  - daemon commands are signalled on every triggered block:
    `daemon.go:76-92`, `Restart`,
    `d.log.Notice(">> sending signal %s", d.conf.RestartSignal)` at `:87` then `d.ex.Signal(...)`,
    with SIGHUP the default (`README.md:70-72`,
    "Daemons are sent a SIGHUP (by default) when their block is triggered").
    On POSIX the signal goes to the whole process group
    (`shell/proc_posix.go:15-17`, `syscall.Kill(-e.cmd.Process.Pid, ...)`);
    on Windows it is `taskkill /f /t /pid <pid>` (`shell/proc_windows.go:19-21`).
- Adjacent behaviour on modd's own exit:
  `modd.go:175-180` handles SIGINT and SIGKILL by shutting down daemons and calling `os.Exit(0)`.
  A prep command runs in its own process group (`shell/proc_posix.go:11-13`, `Setpgid: true`),
  so that exit path sends it nothing and does not wait for it;
  only the pipes modd reads from are closed by the exit.
- Outcome: pass for the design's shape,
  which runs `systemctl --user start` as a `prep:` line.
  A config that put the same command on a `daemon:` line would signal the client on every change.

## Hard constraint: queues one more run natively

- Candidate and version: modd `v0.8`.
- Claim: a change that lands while the command runs produces another run after it finishes,
  from the watcher itself.
- Gate: vet report "Hard constraints", seventh bullet,
  owner answer, round 4 ("Native queueing becomes a hard requirement").
- Status: hard gate, pass.
- Primary source:
  - `modd.go:225`, `modchan := make(chan *moddwatch.Mod, 1024)`.
  - `modd.go:202-218`, the single consumer:
    `for mod := range modchan { ... mr.trigger(currentDir, mod, dworld) }`,
    so a batch that arrives during a run waits in the buffered channel.
  - `vendor/github.com/cortesi/moddwatch/watch.go:296-302`, `send`,
    does a blocking send `w.modch <- m` under a mutex, so it queues rather than drops.
- Granularity:
  the queue holds completed batches, not a single "run again" flag.
  `batch` (`vendor/github.com/cortesi/moddwatch/watch.go:249-283`)
  closes a batch on a 100 ms lull or on the 8 s `MaxLullWait`,
  and each closed batch is one queued entry,
  so a change spread over a long run can leave several queued runs, not one.
  This matches the vet report's reading of `modd.go:202-225`.
- Related open bug, for the same mechanism:
  issue #130, "no de-duplication of triggered items", `2023-11-30T09:29:27Z`, open,
  reports the same file listed three times in `@mods` for three created files.
- Outcome: pass.
  The queued runs happen after the current block returns and nothing in this path kills a prep run.

## Capability: how the command runs, and what happens to changed paths

- Claim recorded: whether the command is argv or a shell string,
  and whether changed paths go into the command line, the environment or stdin,
  and how they are quoted.
- Gate: vet report "Capabilities recorded for every candidate",
  and the skill's "Production source" parser and escaping boundary.
- Status: capability record, not a gate.
- Primary source:
  - The command is a shell string.
    The default shell is modd's own interpreter:
    `shell/shell.go:24`, `var Default = "modd"`,
    and `shell/shell.go:201-218` builds argv as
    `exec.Command(shcmd, "--exec", command)` for that shell,
    `exec.Command(shcmd, "-c", command)` for `bash` and `sh`,
    `exec.Command(shcmd, "-Command", command)` for `powershell`.
  - For the default shell, `shcmd` is modd's own executable
    (`shell/shell.go:189-195`, `return os.Executable()`),
    so modd re-executes itself with `--exec`,
    which parses the string with `mvdan.cc/sh/syntax` and interprets it in process
    (`cmd/modd/main.go:60-85`).
    `cmd/modd/main.go:69-72` sets `r.KillTimeout = -1`;
    the interpreter documents that negative value at
    `vendor/mvdan.cc/sh/interp/interp.go:390-401`
    as "a kill signal will be sent immediately" when the interpreter's context is cancelled,
    and nothing cancels that context during a command.
  - Changed paths are substituted into the command string and nowhere else.
    `varcmd/varcmd.go:73-88` builds `@mods` and `@dirmods` from the batch
    (`mkArgs`, `:52-58`) and substitutes them by textual replacement (`:94-124`).
    No environment variable is set and nothing is written to stdin.
    `moddwatch.Mod.All()` (`vendor/github.com/cortesi/moddwatch/watch.go:84-93`)
    includes changed and added paths, not deleted ones.
  - Quoting: `varcmd/varcmd.go:28-34`:

    ```go
    // FIXME: This is actually dependent on the shell used.
    func quotePath(path string) string {
        path = strings.Replace(path, "\"", "\\\"", -1)
        return "\"" + path + "\""
    }
    ```

    Only `"` is escaped, then the path is wrapped in double quotes,
    and the string is parsed again by the shell.
    Inside double quotes the shell still expands `$`, backticks and `$(...)`,
    so a file name containing those is not passed literally,
    and a name ending in `\` escapes the closing quote.
    Names with spaces, parentheses, brackets, quotes,
    and Japanese or Chinese characters pass through this path unchanged,
    because `"` is the only character that needs and gets escaping for them.
- Consequence for this design:
  the trigger command names no path at all,
  so it does not touch this quoting path;
  a config that interpolated `@mods` would.
- Outcome: recorded.
  The command is a shell string run by an embedded interpreter,
  paths are interpolated into that string with a quoting routine its own author marks as shell-dependent.

## Capability: busy behaviour, and whether any mode stops a run

- Claim recorded: exactly one of queue, drop, kill or run concurrently by default,
  and with the option that queues,
  from source.
- Gate: vet report "Capabilities recorded for every candidate".
- Status: capability record, not a gate.
- Primary source: there is only one mode, and it is queue.
  - Queue: `modd.go:202` and `modd.go:225`, as recorded above;
    `vendor/github.com/cortesi/moddwatch/watch.go:296-302`.
  - No drop of the batch that matters:
    the sending side blocks instead of dropping.
  - No kill of a prep run: no signal path from an event to a prep process
    (see "Hard constraint: never interrupts a running command").
  - No concurrency: one goroutine consumes `modchan`, so blocks run one after another
    (`modd.go:202`, `modd.go:109-146`).
  - There is no flag or config key that selects a busy mode:
    `cmd/modd/main.go:19-53` declares every flag,
    `--file`, `--noconf`, `--bell`, `--ignores`, `--notify`, `--prep`, `--debug`, `--exec`,
    and none of them concerns busy behaviour.
- Where events can still be lost, recorded because it bounds the queue:
  the batch reader sends with a blocking send into a channel of 1024 (`modd.go:225`);
  when that channel is full, the reader stops draining the 4096-slot event channel
  (`vendor/github.com/cortesi/moddwatch/watch.go:410`),
  and the notify library then drops events at `vendor/github.com/rjeczalik/notify/watchpoint.go:86-93`:

  ```go
  select {
  case ch <- ei:
  default: // Drop event if receiver is too slow
      dbgprintf("dropped %s on %q: receiver too slow", ei.Event(), ei.Path())
  }
  ```

  `dbgprintf` is a no-op unless the `debug` build tag or `NOTIFY_DEBUG` is set
  (`vendor/github.com/rjeczalik/notify/debug.go:16-21`,
  `vendor/github.com/rjeczalik/notify/debug_nodebug.go:9`),
  so that loss is silent in a normal build.
  The kernel side is equally silent:
  `vendor/github.com/rjeczalik/notify/watcher_inotify.go:272-275` discards
  `IN_IGNORED` and `IN_Q_OVERFLOW` events.
- Outcome: recorded.
  Queue, always, with silent event loss possible only after roughly a thousand queued batches.

## Capability: quiet period or debounce, and its default

- Claim recorded: the quiet-period option and its default.
- Gate: vet report "Capabilities recorded for every candidate"
  ("merges a burst of events into one run").
- Status: capability record, not a gate.
- Primary source:
  - `modd.go:21`, `const lullTime = time.Millisecond * 100`.
  - `vendor/github.com/cortesi/moddwatch/watch.go:18`,
    `const MaxLullWait = time.Second * 8`,
    documented at `:16-18` as the ceiling for a continuous stream of modifications.
  - `batch` at `vendor/github.com/cortesi/moddwatch/watch.go:249-283`
    returns a batch after a `lullTime` gap or after `maxTime`.
  - The values are constants, not options:
    `lullTime` is passed as a literal at `modd.go:193`,
    and no flag at `cmd/modd/main.go:19-53` or config key in `conf/conf.go:23-31` sets it.
- Corroboration that no user setting exists:
  pull request #101, "add silence period a.k.a. debounce",
  opened `2021-05-05T17:44:13Z`, updated `2026-10-13T15:52:20Z`, still open,
  proposes the feature;
  its only comment, `2026-10-13T15:52:20Z`, asks for it.
- Outcome: recorded.
  A fixed 100 ms lull, bounded at 8 s, not configurable.

## Capability: path exclusion

- Claim recorded: excludes paths by pattern.
- Gate: vet report "Capabilities recorded for every candidate".
- Status: capability record, not a gate.
- Primary source:
  - Block patterns carry negations:
    `README.md:144-157`, "Negation",
    where a pattern prefixed with `!` removes matches,
    with the example `** !**/*.html !"docs/**"`.
  - Matching is applied by
    `vendor/github.com/cortesi/moddwatch/filter/filter.go:23-41`, `File`,
    "At least one include pattern and no exclude patterns must match",
    over the batch's changed, deleted and added lists
    (`vendor/github.com/cortesi/moddwatch/watch.go:142-157`, `Filter`).
  - A default ignore list applies to patterns unless the block opts out:
    `modd.go:25-51`, `CommonExcludes`, covering VCS folders,
    `**.tmp`, `**~`, `**#`, `**.bak`, `**.swp`, `**.py[cod]` and `**/node_modules/**`;
    applied at `modd.go:93`, `newcnf.CommonExcludes(CommonExcludes)`,
    skipped by the `+noignore` block flag (`README.md:159-171`).
  - Excludes are directory-aware for walking, but watches are not narrowed:
    `moddwatch.Watch` is called with `[]string{}` as excludes
    (`modd.go:193`), so an excluded file is filtered out after its event arrives.
- Outcome: recorded.
  Pattern exclusion works at the event-filter stage;
  it does not reduce the number of inotify watches.

## Capability: what happens when the inotify watch limit runs out

- Claim recorded: error, silent loss or fallback, from source.
- Gate: vet report "Capabilities recorded for every candidate".
- Status: capability record, not a gate.
- Primary source, startup:
  - `vendor/github.com/rjeczalik/notify/watcher_inotify.go:92-95`,
    `iwd, err := unix.InotifyAddWatch(...)`, then `if err != nil { return }`,
    so `ENOSPC` from the kernel comes back as an error.
  - It propagates through `watchrec` at
    `vendor/github.com/rjeczalik/notify/tree_nonrecursive.go:228-230`
    ("account every path that failed to be (re)watched and retry" is a `TODO` at `:226-227`),
    through `moddwatch.Watch` at `vendor/github.com/cortesi/moddwatch/watch.go:413-417`
    ("could not watch path '%s'"),
    and through `modd.go:193-197` ("Error watching: %s").
  - `cmd/modd/main.go:124-129` logs that error with `log.Shout` and returns from `main`,
    so the process ends with exit status 0 after printing the error.
- Primary source, folders created after start:
  the watch is added in `internal` at
  `vendor/github.com/rjeczalik/notify/tree_nonrecursive.go:96-100`,
  and the only reporting is `dbgprintf("internal(%p) error: %v", rec, err)`.
  That is a no-op in a normal build,
  so a limit hit there costs that folder's events with no message.
- Primary source, kernel queue overflow:
  dropped silently at `vendor/github.com/rjeczalik/notify/watcher_inotify.go:272-275`.
- Fallback: none.
  No polling mode, no degraded path;
  the only retry logic is the `TODO` quoted above.
- Scale on the target host, from the design record:
  `fs.inotify.max_user_watches = 524288` against 1454 folders,
  so a start is far below the limit;
  the risk is the per-folder watch-adder failing later, silently.
- Outcome: recorded.
  Startup exhaustion is an error message plus exit status 0;
  later exhaustion and queue overflow are silent event loss.

## Capability: security-relevant surface

- Claim recorded: listeners or sockets, privileged operations, network access, update checks, telemetry.
- Gate: vet report "Base components and overlays"
  (the high-trust overlay is not applicable;
  the sensitive-data overlay is not applicable),
  and the skill's "Production source" security boundary.
- Status: capability record, not a gate.
- Primary source and probes:
  - No network code in the linked packages:
    `grep -rn 'net/http\|net\.Dial\|net\.Listen\|"net"' --include='*.go' --exclude-dir=vendor .`
    printed no match in modd's own packages;
    `grep -rn '"net' --include='*.go' vendor/mvdan.cc/sh/interp vendor/mvdan.cc/sh/expand vendor/mvdan.cc/sh/syntax`
    printed no match.
    `vendor/golang.org/x/net/context` is the only `x/net` package imported by linked code
    (`vendor/github.com/cortesi/termlog/context.go:3`);
    the tree also vendors unimported `x/net` packages such as `http2`, which do not link.
  - No listener or socket anywhere:
    the same `net.Listen` probe matched nothing in linked code.
  - No update check and no telemetry:
    no flag or code path fetches anything;
    `cmd/modd/main.go:19-53` lists every flag, and none is about updates or reporting.
  - Desktop notifications are off unless `-n` is passed
    (`cmd/modd/main.go:40-42` and `:100-108`),
    and then modd spawns `growlnotify` or `notify-send`
    (`notify/notify.go:36-53`), picked by `PlatformNotifier` at `:56-63`
    through `exec.LookPath` at `:10-16`.
  - No privileged operation:
    modd runs as the invoking user,
    changes directory for an `indir` block and changes back
    (`modd.go:110-131`),
    and its only `taskkill` use is for daemons on Windows (`shell/proc_windows.go:19-21`).
  - It reads exactly one file of its own, the config, and re-reads it on change
    (`modd.go:182-185` adds the config's directory to the watched patterns unless `-c`).
  - The process-execution boundary is the command string,
    run through the embedded interpreter or a chosen external shell
    (`shell/shell.go:201-218`),
    with no timeout (`cmd/modd/main.go:69-72`).
  - The one injection-shaped path is the `@mods`/`@dirmods` substitution recorded above:
    a file name containing `$(...)` or a backtick becomes shell syntax
    when a config interpolates those variables.
- Outcome: recorded.
  No network, socket, update or telemetry surface;
  the risks are the command string itself and the path-quoting routine.

## Installed-artifact provenance

- Candidate and version: modd `v0.8`.
- Claim: what a user installs on each target can be traced to the tagged source,
  with a workflow or registry mapping plus a checksum or signature.
- Gate: skill "Screening hard gates"
  ("native, Wasm, or prebuilt provenance cannot be mapped to source";
  "build provenance is unknown"),
  and "Native, Wasm, and prebuilt overlay".
- Status: hard gate,
  fail for the prebuilt release assets,
  because no build definition, workflow, checksum or signature ties them to `b0c08baa`.
- What exists, per system:

### Linux x86-64

- Release asset `modd-0.8-linux64.tgz`, 3,324,376 bytes,
  created `2019-01-20T19:24:16Z`, 337,114 downloads
  (`gh api repos/cortesi/modd/releases/tags/v0.8`).
- The asset carries no checksum and no signature from its publisher:
  every asset in that release reports `"digest":null`,
  and the release has no checksum file
  (the ten assets are the platform archives only).
- The build definition is outside the repository:
  `README.md:458-460` says
  "The scripts used to build this package for distribution can be found"
  at `https://github.com/cortesi/godist`,
  and "External packages are vendored using dep".
  In `cortesi/godist`, `gobuild:13-24` maps `linux`/`amd64` to the `linux64` suffix,
  and `gobuild:58-69` runs `go build -o ... ./cmd/modd` with `GOOS` and `GOARCH` taken from the environment.
  That repository is unpinned by modd (no version, tag or lock reference),
  has no release workflow, and its own last push is `2016-12-11T19:09:48Z`.
- The repository at the tag has no release workflow:
  `.travis.yml:1-19` runs `go get`, `go install ./cmd/modd` and `go test -v -race ./...` on
  linux, osx and windows with Go 1.11,
  and there is no `.github/` directory in the `v0.8` tree.
  CI was added only on `master` at `2026-06-21`
  (`.github/workflows/ci.yml`, merged as PR #136),
  and that workflow has one job, `test`, with no release or upload step.
- One third-party checksum exists:
  the AUR package `modd` 0.8-1 pins the asset with
  `sha256sums=('65d5b5410d0b11dec3427d9363426add67bde60ddbd8277f5794c981af4b8ad7')`
  for `modd-0.8-linux64.tgz` (`PKGBUILD:10-11`, `.SRCINFO:9-10`).
  It is a checksum of the artifact, not evidence of how it was built,
  and the package is orphaned (`Maintainer` null), flagged out of date since
  `1688831953` (2023-07-08), `x86_64` only, and installs the prebuilt binary
  (`PKGBUILD:13-17`).
- Other install routes on this system:
  `mise registry` has no `modd` row
  (`mise registry | grep -iE '\bmodd\b'` printed nothing),
  aqua's registry package is restricted to `darwin`/`amd64` (below),
  and the AUR snapshot above is the only packaged route found.
  A source build via
  `go install github.com/cortesi/modd/cmd/modd@v0.8` would be inspectable
  but is not what any package manager installs.

### macOS arm64

- There is no arm64 asset.
  The macOS asset is `modd-0.8-osx64.tgz`, 3,787,377 bytes,
  created `2019-01-20T19:24:41Z`,
  and `gobuild:15` maps the `osx64` suffix to `darwin`/`amd64`.
  An arm64 Mac would run it under Rosetta 2 or not at all.
- Aqua encodes exactly that:
  `pkgs/cortesi/modd/registry.yaml` sets `rosetta2: true`
  and limits the package with `supported_envs: [darwin, amd64]`,
  while `pkgs/cortesi/modd/pkg.yaml` pins `cortesi/modd@v0.8`.
- Homebrew no longer offers it:
  `https://formulae.brew.sh/api/formula.json` fetched 2026-11-25 holds 8,740 formulae and
  no formula named `modd`, and `https://formulae.brew.sh/api/formula/modd.json` returns a 404 page,
  so the `brew install modd` line at `README.md:34-36` is stale.
  Issue #119, "Error: modd has been disabled because it is not maintained upstream!",
  created `2022-10-07T15:15:03Z`,
  records the removal, and a comment at `2022-12-31T01:41:24Z` dates the disable
  to 2022-09-14 in `homebrew-core`.
- A source build is possible but needs cgo for FSEvents
  (issue #119, comment `2022-12-28T13:55:12Z`: `CGO_ENABLED=1 go install ...`),
  and the unsigned build a user installs is what macOS killed in issue #132.
- Outcome for this system: no traceable artifact at all,
  and no provenance question to answer,
  since none was published.

### Windows x64

- Release asset `modd-0.8-windows64.zip`, 3,132,767 bytes,
  created `2019-01-20T19:24:51Z`,
  built by the same `godist` script (`gobuild:16` maps `windows`/`amd64` to `windows64`).
- No checksum, no signature and no Authenticode evidence:
  the release API reports `"digest":null` for this asset,
  and nothing in the repository signs it.
- winget: the vet report's Q18 enumerated every manifest directory and recorded none for `modd`;
  this run did not re-run that enumeration, so that negative rests on the vet report alone.
- Outcome: same as Linux, an unattested prebuilt archive.

### Why this is a fail rather than an open item

- The tag-to-source direction is proven (`v0.8` is `b0c08baa`).
- The source-to-artifact direction is not:
  the build is a hand-run, unpinned script in another repository,
  the publishing repository has no release workflow at the tag,
  the assets carry no checksum or signature,
  and the release page records no build statement.
- The skill's overlay section is explicit:
  "Unknown build provenance is a hard failure" and
  "A prebuilt artifact without verifiable source mapping is ineligible."
- This run could not close the gap without executing a candidate artifact,
  which it may not do,
  and even a byte comparison would prove only identity with that build,
  not that the build came from the tag.

## Tests and CI, as recorded

- At the tag: `.travis.yml:1-19`, `go test -v -race ./...` on linux, osx and windows,
  Go 1.11, plus `go get -t -v ./...` and `go install ./cmd/modd`.
- Test files in the released tree:
  `modd_test.go`, `proc_test.go`, `varcmd/varcmd_test.go`, `shell/shell_test.go`,
  `conf/conf_test.go`, `conf/lex_test.go`, `conf/parse_test.go`, `conf/parse_posix_test.go`,
  `vendor/github.com/cortesi/moddwatch/watch_test.go`,
  `vendor/github.com/cortesi/moddwatch/filter/filter_test.go`.
- No fuzzing or mutation harness is present in the tree
  (`find . -path ./.git -prune -o -name '*fuzz*' -print` printed nothing).
- On `master`, since `2026-06-21`, the same tests run on ubuntu, macos and windows
  with Go 1.26 (`.github/workflows/ci.yml:13-42`),
  with `gofmt`/`go vet` on ubuntu only.
- None of this was run in this session.

## Maintenance audit

Method: the count-and-sample procedure in the skill's "Maintenance audit",
run through `gh api` and `gh api -X GET search/issues` on 2026-11-25.
Window: 12 months back from that date, `updated:>=2025-11-25`.

Issues created or updated in the window: one, so all of it was inspected.

- #135, "Example usage of `{alt1,...}`",
  created `2025-10-03T07:32:14Z`, updated `2026-02-20T09:19:07Z`, open, one comment.
  The comment, `2026-02-20T09:19:06Z`, is from `wader` and answers the question with
  `"**/*.{c,h}" { prep: make test }`.
  No maintainer label, assignment, milestone or closure accompanies it,
  and the issue stays open.

Pull requests created or updated in the window: four, so all were inspected.

- #137, "go get -u && go mod tidy", opened and merged `2026-06-21T09:23:46Z`/`09:26:43Z`
  by `wader`, `merged_by` `wader`, 0 reviews, 0 comments, 22 additions, 45 deletions.
- #136, "Add CI via github actions", opened and merged `2026-06-21T09:16:42Z`/`09:19:27Z`
  by `wader`, `merged_by` `wader`, 0 reviews, 0 comments, 42 additions, 1 file.
  Both branch names are inside the repository (`cortesi/ci`, `cortesi/mod-update-tidy`),
  so `wader` holds write access there.
- #115, "adds support for lines which are longer than the read buffer",
  opened `2022-04-22T15:17:28Z`, closed unmerged `2026-05-13T21:38:15Z`,
  no comment and no review recorded.
- #101, "add silence period a.k.a. debounce ",
  opened `2021-05-05T17:44:13Z`, updated `2026-10-13T15:52:20Z`, open,
  one comment from `MagicRB`, no maintainer comment, `mergeable_state` "clean".

Release cadence:

- Releases: `v0.1` 2016-02-02, `v0.2` 2016-02-10, `v0.3` 2016-04-07, `v0.4` 2016-09-28,
  `v0.5` 2017-11-16, `v0.6` 2018-04-12, `v0.7` 2018-07-25, `v0.8` 2019-01-20.
  Nothing since.
- Tag `v0.8.1` exists at commit `41d9ef6a250494eb2139c0b1331369d78eff1ec3`
  (`gh api repos/cortesi/modd/tags`),
  dated `2023-05-03T00:30:26Z`, with the message
  "Updated the version of moddwatch to one that uses a newer version of github.com/rjeczalik/notify.
  This should fix #126, but I havent tested that since I am not on the proper OS version."
  There is no GitHub release and no artifact for it,
  so a binary user stays on the 2019 build while the source fix sits in a tag.
- `master` is 38 commits ahead of `v0.8`
  (`gh api repos/cortesi/modd/compare/v0.8...master`), including a conversion to Go modules
  (`093e0006`, 2019-08-09) and the 2026 CI and dependency work.
  None of it has reached a release.

Maintainer concentration:

- 16 contributors in total.
  `cortesi` 258 contributions, `kardianos` 6, `wader` 4, `joncalhoun` 3,
  `jesselang` 2, then single contributions.
  That is about 91 per cent of the 284 recorded contributions from one account.
- Last commit by `cortesi` in the history: `2025-05-23T08:30:36Z`.
  Every commit in the 12-month window is `wader`'s, four of them on one day, `2026-06-21`.

Stale and unanswered bugs, and abandonment signals, from the open list of 42:

- #44, "Improve Linux recursive watch", opened and last updated `2017-11-17T19:55:02Z`,
  no comment, never closed;
  quoted as counterevidence under the recursion record.
- #119, "Error: modd has been disabled because it is not maintained upstream!",
  `2022-10-07`, 13 comments, latest `2024-05-22T10:32:53Z`,
  which recommends closing the issue.
- #132, "modd immediately killed on macOS ARM64 (Apple Silicon) with no useful error message",
  `2025-05-16`, updated `2025-05-18`, open.
- #130, "no de-duplication of triggered items", `2023-11-30`, no comment,
  and its body opens with "I can see that this repo is not being maintained,
  but I'll leave it here nonetheless".
- #124, "BUG: modd does not seem to shutdown deamon processes correctly when getting stopped",
  updated `2023-04-03T00:38:51Z`.
- #102, "modd on macOS never triggers events in ~/documents/x but works in ~/Documents/x",
  open.

Reading, in the skill's terms:
an active-but-narrow maintenance posture,
with source and CI touching the tree in 2026 and no release since 2019,
and a tracker where one of the ten most recently updated open issues got a reply
(#135, from `wader`) and the rest of that sample shows no maintainer action.
It is not fully abandoned:
`master` moves and CI runs on three platforms.
It is also not one of the skill's healthy shapes,
because the work does not reach a release
and the tracker's oldest functional gaps stay open.
Zero tracker activity is not the case here,
so low-signal activity is not the reading either.

## Summary: each hard constraint, pass, fail or open

- Open source with inspectable source: pass.
  Decided by `LICENSE:1-21` (MIT) and the tag `v0.8` at commit `b0c08baa`.
- Runs an arbitrary command: pass.
  Decided by `prep.go:24-41`, `prep.go:71` and `shell/shell.go:201-218`
  (an arbitrary shell string, run to completion).
- Runs on Linux x86-64, macOS arm64 and Windows x64: open.
  Decided by the three backends in source
  (`watcher_inotify.go:5`, `watcher_fsevents.go:5`, `watcher_readdcw.go:5`)
  against artifacts that cover Linux x86-64 and Windows x64 only,
  with `osx64` built as `darwin`/`amd64` (`gobuild:15`) and issue #132 still open.
- Watches every folder recursively, including folders created after start: pass on source.
  Decided by `modd.go:193`,
  `vendor/github.com/cortesi/moddwatch/watch.go:413`,
  `vendor/github.com/rjeczalik/notify/tree_nonrecursive.go:206-233` for the start walk
  and `:66-101` for folders created later,
  with issue #44 recorded as stale counterevidence and the add-watch race recorded as a gap.
- After a change during a run, one more run starts after it ends: pass.
  Decided by `modd.go:202` and `modd.go:225` with
  `vendor/github.com/cortesi/moddwatch/watch.go:296-302`,
  with the qualification that one queued entry exists per lull-closed batch.
- Never interrupts a running command: pass for a `prep:` command.
  Decided by `modd.go:109-146` and `prep.go:32`
  (no signal path to a prep process),
  against `daemon.go:76-92` and `shell/proc_posix.go:15-17`,
  which signal a `daemon:` command on every change.
- Queues one more run natively: pass.
  Decided by `modd.go:202` and `modd.go:225`,
  with silent event loss possible only when the queue and the event buffers are both full
  (`watchpoint.go:86-93`).
- Installed-artifact provenance, as the skill's screening gate requires:
  fail for the prebuilt release assets.
  Decided by `README.md:458-460` (build script in `cortesi/godist`, unpinned),
  `.travis.yml:1-19` and the absence of any release workflow in the `v0.8` tree,
  and `"digest":null` on all ten assets of `https://github.com/cortesi/modd/releases/tag/v0.8`.
  The tag to source direction is proven;
  the artifact to tag direction is not.

## What this record could not establish

- Any runtime behaviour:
  this run may not install, build or run the candidate,
  so the queue,
  the new-folder watch,
  the quoting of unusual names and the limit behaviour are source readings only.
- Whether the published archives were built from `b0c08baa`:
  no workflow, checksum or signature records it,
  and only a byte comparison against a rebuild would test it,
  which this run may not do.
- macOS arm64 behaviour on real hardware:
  the only evidence is the artifact mapping, issue #132 and the cgo note in #119.
- Whether the vendored `notify` revision's new-folder watch-adding works under load on this host,
  including the race issue #44 names.
