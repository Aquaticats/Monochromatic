# The ZFS route's root modes as modes of the one wrapper executable

## Status

A proposal,
put to the owner.
It is not a decision,
and nothing here is approved or final.
It records what the project decided in
[the one-program record](../decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md),
what the owner then answered about the ZED hook's name in
[the start-name record](../decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md),
and designs how the two are carried out.
Every claim about a program this repository does not own is read from that program's source,
and every claim about this repository's code is re-read on `main` at `68c8dcaa21`,
the commit this round merged into its branch.
The decision records and the handover are untouched by this round.

This is the second revision,
the round of 2026-11-24T02:29:00-05:00.
It replaces the text of the first revision
(`1e5f417430`),
which settled choice 1 on the ZED hook's name and the words,
and which a same-vendor second pass then read again.
That pass is not an independent review:
it is a second reader of the same vendor,
and its file is
`~/temp/agent/gate-review-zfs-root-modes-revision-2026-11-24/keep/re-read-of-the-revised-one-program-design.md`.
It found two High, four Medium and nine Low items,
and this revision answers every one of them in
"What this revision changed, and why".
The probes that answer them are in this round's `keep/`,
named at each claim and listed in the appendix.

## What this revision changed, and why

### The two High findings

The first is that choice 3's recommended layout made the name, not the words, select the modes for units and adoption,
which is not what the answer's second clause says,
and that the first revision's summary sentence therefore did not hold.
Section "Choice 1: what selects a root mode" now compares three dispatch rules on twenty-eight starts,
ranks them,
and states for every install layout exactly which spelling selects a mode there.
The rule this revision recommends checks the words first on any file,
so the words select a mode whatever file carries them,
and gives the installed name only ZED's no argument plus `--help` and `--version`.

The second is that the first revision's guard change pinned a call site rather than a name:
its `ALLOWED` row was keyed by file and rule,
so it excused the four old needles in `main.rs` as well,
and every variant the second pass wrote passed the proposed hook guard.
Section "The guard change, exactly" now
gives the argument-vector read its own rule id and its own `ALLOWED` row,
forbids every way of reaching the program name in `main.rs`,
pins the three lines that may name the vector,
and adds a guard on the names module itself.
`keep/guard-replay-revision2.log` shows the proposed `main` passing all three guards,
each of the second pass's six variants failing,
two more edits failing,
and each planted names module failing a rule of its own.

### The four Medium findings

The cold-start numbers are relabelled and extended.
The old arm measured the wrapper's own main-level start with one file's pages evicted,
which is page-cache-cold and ARC-warm on this host's ZFS:
not a start after boot.
This round measures again with the command each arm runs named,
reports the single disk-cold sample of the old log apart,
and measures a disk-cold start where one can be made:
an Ubuntu 24.04 guest created through the repository's own guest driver,
with the whole page cache dropped before every run,
destroyed through `mvm` when the round ended.
Section "What else the modes keep, and what they lose" gives the numbers as ratios and pairs,
which is what a comparison between two arms can support.

Choice 4's option C is placed exactly:
it sits before the runtime probe's check at `src/native/invocation/entry.rs:344`,
so it costs what option B costs,
and the ranking now compares the two by where the code lives and by the parameter the entry would need.

The change list is re-derived with `rg` on `main` now,
with the line numbers it finds,
and it gains the files the second pass named:
`bin/native-install.unit.test.ts`,
`bin/stand-in-install-programs-for-tests.ts`,
`health/doctor_zfs_route/identity_checks.rs`,
`health/doctor_zfs_route/handler_checks.rs`'s two placeholder names,
and the wrapper's own management help,
which has no line for the root modes today.
`mode_awaiting_its_step.rs` is struck from the list: it names no program.

The words with no mode word after them now refuse with the usage,
as `service` needs `run`.
`zfs-root` alone used to start a full sweep,
and only `start-probe` and `adopt-worktree-parent` check the effective user
(`handler_mode_dispatch.rs:174` and `adopt_worktree_parent_mode.rs:315`),
so a person who forgot the mode started real work.

### The nine Low findings

- The `LiveGate` rebuttal keeps its verdict and loses its wrong account.
  The wrapper does run the gate before its first raw request,
  and the gate's call arrived after the commit the first review read:
  `worktree_clone_add_route/add_zfs.rs` was added in `b01af8c3bd` (2026-11-23T20:54:55-05:00),
  which is not an ancestor of `c2701d58a5`,
  the commit that review read.
  The first review's `rg` was right for its commit;
  it did not miss a caller.
- The trailing slash and the dot forms of `argv[0]` are probed (cases 10 to 13).
  `Path::file_name` reads `.../name/` as `name`,
  so a trailing slash still names the zedlet,
  and only a caller that writes `argv[0]` itself can produce one,
  because `execve` of a regular file with a trailing slash fails.
- "Five names that a suffix test would have taken" becomes what the probe shows:
  of the five near-miss names, a suffix test takes one (`xhistory_event-...`) and a contains test two
  (`xhistory_event-...` and `...datasets.bak`), where this rule takes none.
- Every printed piece now names the log and the case it came from,
  the stray-copy sentence is said the right way round,
  and the lock-holder record a start writes is this round's own measurement
  (`keep/probe-install-layouts.log`),
  not a pair of rows spliced from another log.
- No choice claims a per-option run it does not have:
  choices 4 and 7 say which probe covers them,
  and every option from choice 2 on shows a command and what it prints or decides.
- The bytes root installs come from the lock row's `artifacts.linux-x64.url` and `sha256`,
  and section "Steps 18 and 19" says who runs the step and what root trusts,
  which is the reviewed lock file in this repository,
  not the account's copy.
- `doctor` prints the root-owned path,
  never `sudo git monochromatic-git-policy-cli ...`,
  whose `git` is the account's file.
- The kernel inserts the empty `argv[0]` when an argument vector is empty
  (`fs/exec.c`, the `bprm->argc == 0` block),
  measured here through `execv(path, {NULL})`;
  and the root-mode arm is gated `#[cfg(target_os = "linux")]`,
  because `worktree_clone_mount_handler` is.
- Choice 2's separability list names every place the word sits,
  the rule's own text,
  the `native:install` check,
  the wrapper's management help,
  `SPEC.md` and the guest commands.

## What the decision requires

The one-program record answers "One program only,
ever",
and its "What follows" is the requirement this design starts from:

- the release holds one executable,
  and the root modes
  (`boot-reconcile`,
  `shutdown-unmount`,
  `start-probe`,
  `holder-scan`,
  `adopt-worktree-parent`
  and the no-argument sweep the ZED hook makes)
  become modes of `monochromatic-git-policy-cli`;
- step 5's release changes are undone
  (the second `[[bin]]`,
  the release table rows,
  the binstall row,
  the lock's `bins` row
  and the archive's second program);
- the wrapper still links no OpenZFS library and starts where OpenZFS is absent;
- how root reaches the modes,
  what the ZED hook and the units name,
  and how a root mode keeps the wrapper's Git,
  LFS and network paths from running,
  are this design's to propose,
  and the start-name record has since answered the first part of that.

The start-name record's answer has two clauses,
and everything in choice 1 follows from the pair:

- only the zedlet's file name selects a root mode,
  compared as the whole last path component,
  and it is the only exception to the ban on reading a start name;
- the boot and shutdown units,
  adoption and a person reach the root modes by words in the wrapper's own namespace,
  as `service run` does.

What makes the problem hard is unchanged:
the wrapper is the `git` a person runs,
so its arguments are Git's arguments,
and an argument alone cannot mean "root mode" without taking a word away from Git.
And ZED hands a zedlet no arguments at all.

## What the code holds today

### The two entries

`package/git-policy/cli/src/native/main.rs` is the wrapper's entry.
It collects the arguments once, skipping the program name (`main.rs:52`),
answers the service mode when the leading words are
`monochromatic-git-policy-cli service run` (`main.rs:56`,
`invocation::service_main::is_service_start`),
installs the payload-free panic hook,
collects the environment,
and hands everything to `invocation::entry::run_process` (`main.rs:72`),
then waits for the log lines it handed to their own threads
(`end_after_the_log`).

`package/git-policy/cli/src/native/mount_git_worktree_clone_datasets/main.rs` is the second program's entry,
deleted by this design:
`args_os().skip(1)`,
`worktree_clone_mount_handler::run`,
the same two log waits,
and the exit.
Its grammar (`worktree_clone_mount_handler/handler_mode_arguments.rs:319`) takes
one leading word out of
`boot-reconcile`,
`start-probe`,
`shutdown-unmount`,
`holder-scan`,
`adopt-worktree-parent`,
`--help` and `--version`;
an empty argument list is `AttachingSweep`;
anything else is a refusal that names the word and the modes.
`handler_mode_dispatch.rs:317` to `:338` gives every mode the same two answers,
a line and status 0,
or a refusal line and status 2,
and reads the compatibility gate before a mode that will make a request
(`run_behind_at` at `:317`).
`handler_install_paths.rs:28` to `:42` holds the program's name,
the two installed paths,
and the configuration path.

The service mode is the precedent this design follows,
and the answer says so:
a mode chosen inside `main`,
before the panic hook,
before the environment is collected,
and before anything of the wrapper's own start runs.
It differs from the words path in three ways this design must state,
because they are what a reader will compare:

- `service run` also answers the earlier command word `cli-git`
  (`service_main.rs:74`, `EARLIER_SERVICE_WORDS`);
  the root words answer only `monochromatic-git-policy-cli`,
  because no earlier release has a root-mode unit to keep working;
- `service` needs a third word, `run`;
  so does the words path need a mode word,
  which is why `zfs-root` alone refuses (choice 1);
- `service` has no no-argument form at all,
  while the installed name's no argument is ZED's own start.

### The release's lists, and what reads them

`package/git-policy/cli/Cargo.toml:35` to `:48` declares two `[[bin]]` tables,
and its `[package.metadata.binstall]` table (`:56` to `:58`) names `{ bin }`,
so one archive unpacks both programs.
Every list that counts them must agree with the manifest,
and `bin/native-release-bins.unit.test.ts` is the guard that says so:
the release job's `cargo build --bin` flags (`.github/workflows/cargo-publish.yml:2666`)
and its `bin=` copy lines (`:2695` to `:2698`),
`package/crate-release/cli/releasable-crates.json:30`'s `checkBins` for the crate,
`toolchain.lock.json`'s `bins` row for `cargo:monochromatic-git-policy-cli`,
and `src/native/source_guard/release_bins_tests.rs:31`.
`toolchain.plan.jsonc` holds the same row as intent,
and `toolchain.lock.json` is written from the plan:
`mpm-shadow`'s update keeps every other row as it is and gives it "the plan's current bins",
so a changed plan row moves the lock row through the tool,
offline (`toolchain:lock:check` runs
`install --lockfile-only --frozen-lockfile`).
The crate's `pinFiles` row holds only the fuzz crate,
so the decision's "pinFiles row" has nothing left to undo.

`bin/native-install-steps.ts:104` holds `handlerProgram` and `:121` `handlerVersionLine`.
`bin/native-install.ts:447` runs the handler by bare name through `PATH`
and `:459` refuses unless its `--version` line names the release.
`bin/stand-in-install-programs-for-tests.ts:169` to `:186` writes the stand-in handler those tests install,
and `:335` lists it beside the stand-in wrapper.

Six Rust test files start the second program by
`env!("CARGO_BIN_EXE_mount-git-worktree-clone-datasets")`,
at ten sites:

- `binary/binary_mount_git_worktree_clone_datasets_tests.rs:106`;
- `binary/binary_zfs_adoption_guest_tests.rs:118`;
- `binary/binary_zfs_holder_scan_guest_tests.rs:290` and `:892`;
- `binary/binary_zfs_step9_m_arms_tests.rs:200` and `:561`;
- `binary/binary_zfs_sweep_guest_tests.rs:230`, `:243` and `:875`;
- `binary/binary_zfs_unmount_guest_tests.rs:238`.

Each is a binary-level control under `src/native/binary/binary_tests.rs`.
`e2e/start-probe-guest.ts:42` names the built binary in its `HANDLER` constant
and its guest commands run `${HANDLER} start-probe`;
`e2e/mount-git-worktree-clone-datasets-guest.ts` drives the second program with the same built path.
`mise.toml:268` is the e2e task named for it.

### The ZFS code already in the wrapper, and the gate it runs

The wrapper already carries the ZFS code.
`Cargo.toml` depends on
`monochromatic-zfs-request-to-openzfs-kernel-device` for the typed `/dev/zfs` requests,
and the wrapper runs the same compatibility gate before its first request,
which plan step 7 added:
`worktree_clone_add_route/add_zfs.rs:67` calls `LiveGate.verdict()` inside `OpenedDevice::open()`,
reached from `add_route_run.rs:206`,
both release code.
The call arrived in `b01af8c3bd` (2026-11-23T20:54:55-05:00),
after the commit the first gate review read (`c2701d58a5`),
which is why that review found no gate and why its `rg` was right for its commit.
So a root mode inside the wrapper adds no dependency and no library:
the decision's "links no OpenZFS library and starts where OpenZFS is absent" check stays as it is,
and `worktree_clone_mount_handler` is already `#[cfg(target_os = "linux")]` (`lib.rs:414`).

### The sentence and the two guards

Three places say the wrapper never reads its start name,
and the answer changes all three for exactly one name:

- `SPEC.md:19101`, in "#### External hooks":
  "the executable does not read the name it was started under".
- `src/native/source_guard/hook_self_invocation_tests.rs:74`,
  `main_does_not_dispatch_on_its_start_name`,
  which forbids `args_os().next()`, `start_name`, `current_exe`, `arg0`, `file_name`,
  `hook_dispatch` and `hook_entry` in `main.rs`,
  requires `args_os()` exactly once,
  requires `args_os().skip(1)`
  and requires `entry::run_process(`.
- `src/native/source_guard/no_self_invocation_rules_tests.rs:26`,
  the `argv0-dispatch` scan rule,
  whose needles are `args_os().next()`, `args().next()`, `.nth(0)`, `argv[0]` and `arg0(`
  (`:29`).

Beside them,
`src/native/main.rs:53` explains the service mode as chosen
"never by the name this program was started as",
and `binary/binary_external_hook_tests.rs:170` proves that a start under each Git hook event's name behaves as `git`.
Both stay true for every name but the one,
which is what the guard change has to express.

The scan's `ALLOWED` rows are keyed by file and rule
(`no_self_invocation_tests.rs`, `unexplained` and `stale`),
so a row excuses every needle of its rule in its file.
That is why the argument-vector read needs a rule of its own (section "The guard change, exactly").

### The wrapper's management help, which has no line for the words

`src/native/invocation/management_arguments.rs:47` holds `MANAGEMENT_USAGE` and `:61` `MANAGEMENT_HELP`,
and neither names a root mode.
A person who types `git monochromatic-git-policy-cli --help` today reads:

```text
Usage: git monochromatic-git-policy-cli <command> [options]

Run repository policies directly.

Commands:
  check    Check policies over an explicit scope.
  fix      Apply policy fixes over an explicit scope.
  service  status: whether the commit service answers (it sets itself up when a commit needs it).
  ...
```

and a person who types the words is answered by the management arm's refusal,
because the root-mode arm does not exist yet:

```text
$ ./target/release/monochromatic-git-policy-cli monochromatic-git-policy-cli zfs-root --version
Usage: git monochromatic-git-policy-cli check (--all | -- <pathspec>...) [--policy <id>]...
       git monochromatic-git-policy-cli fix (--all | -- <pathspec>...) [--policy <id>]...
       git monochromatic-git-policy-cli doctor
       git monochromatic-git-policy-cli service (status | settle-earlier-batches | stop-units-of-removed-worktrees)
       ...
exit=2
```

So the change needs one line in each of those two constants,
and the tests that assert them
(`invocation/management_arguments_tests.rs` and the two that compare the usage line)
move with it.
Without that line the words work and nothing teaches them.

## What a launcher hands a start

### ZED runs a zedlet by path, with no arguments

From OpenZFS `zfs-2.4.4`
(`keep/zed-source/` in the first revision round's folder):

- `cmd/zed/zed_exec.c:141` builds `path` as `"<zedlet_dir>/<prog>"`;
- `:161` to `:166` point descriptors 0,
  1 and 2 at `/dev/null`
  when ZED runs in the foreground,
  which the host's unit does (`ExecStart=/usr/bin/zed -F`);
- `:167` puts the event descriptor on 3;
- `:168` is `execle(path, prog, NULL, env)`:
  the argument vector is the program name alone,
  and `argv[0]` is `prog`,
  the bare file name with no directory;
- `:421` matches a zedlet against the event class and requires a non-letter straight after it;
- `cmd/zed/zed_conf.c:373` to `:405` accept a file only when `stat` succeeds,
  it is a regular file,
  root owns it (unless `-f`),
  its owner execute bit is set,
  and it is not group- or other-writable;
- `cmd/zed/zed_conf.c:51` caps ZED at 16 concurrent zedlet jobs (`zcp->max_jobs = 16`);
- `cmd/zed/zed.c:238` to `:241` set `umask(0)` and then `chdir("/")`,
  so no zedlet ever starts in a repository.

`cmd/zed/zed_event.c:1007` to `:1022` build the child's whole environment,
and it is not inherited:
the event's nvlist,
a restricted list (`IFS`,
`PATH` from `_PATH_STDPATH`,
`ZDB`,
`ZED`,
`ZFS`,
`ZINJECT`,
`ZPOOL`,
`ZFS_ALIAS`,
`ZFS_VERSION`,
`ZFS_RELEASE`),
`TZ` when the parent has it,
`ZED_PID`,
`ZED_ZEDLET_DIR`,
`ZEVENT_SUBCLASS`
and the `ZEVENT_TIME_*` values.

This round's replay of that child step,
with the fixture writing down what it was handed
(`keep/probe-zed-replay.log`,
from `keep/zed_replay.c` and `keep/replay_fixture.rs`):

```text
PROBE argv0=Some("history_event-mount_git_worktree_clone_datasets")
PROBE last_component=Some("history_event-mount_git_worktree_clone_datasets")
PROBE argc=0
PROBE cwd=Ok("/")
PROBE env PATH=Some("/usr/bin:/bin:/usr/sbin:/sbin")
PROBE env HOME=None
PROBE env TZ=Some("UTC")
PROBE env IFS=Some(" \t\n")
PROBE env ZED_PID=Some("4242")
PROBE env ZED_ZEDLET_DIR=Some("/etc/zfs/zed.d")
PROBE env ZEVENT_SUBCLASS=Some("\"history_command\"")
PROBE fd0=/dev/null
PROBE fd1=/dev/null
PROBE fd2=/dev/null
PROBE fd3=~round/scratch/zedprobe/zed.d/zevent.nvlist
PROBE decided=root-mode: the attaching sweep (no word)
```

The fixture's `argc=` counts the arguments after the program name,
so `0` is the empty vector `execle(path, prog, NULL, env)` builds;
the program still has one element,
its `argv[0]`,
the bare name with no directory.

Consequences a design cannot talk its way out of:

- a mode selector that needs an argument cannot be reached by the ZED hook,
  because ZED passes none;
- `argv[0]` is the bare file name,
  so a name rule must read the last path component and must work with no slash at all;
- ZED offers no way to add a variable of this repository's own to that environment,
  so a variable cannot select a mode;
- ZED starts a zedlet from `/`,
  so nothing a zedlet start does can be about a repository unless a mode goes and finds one.

`current_exe()` is not the name either:
under a symlink it answers the link's target,
so it cannot carry the installed name ZED passes.

### systemd runs what `ExecStart` spells

A transient unit started with
`ExecStart=<path> boot-reconcile`
gave the program `argv[0]` as the full path as written,
and `boot-reconcile` as its only argument,
which the first revision round's transcript shows.
So a unit may pass words,
and its `argv[0]` is a path,
which the one name rule reads.

### A caller can choose `argv[0]`, and not when ZED or systemd starts it

`exec -a` sets anything,
and ZED's `execle` and systemd's `ExecStart` fix the value.
A start whose `argv[0]` a lower-privileged caller chose as the zedlet's name is a root-mode start as that caller,
which is harmless for an account and is why none of this design's launchers passes a caller's choice through
(`keep/probe-dispatch-candidates.log`, case 26).

### An empty `argv[0]` and a NULL vector are one empty argument

`execve` with an empty vector,
and `execve` with one empty element,
both reach the program as a single empty argument.
Measured on this kernel
(`keep/probe-dispatch-candidates.log`, case 28,
through `keep/exec_empty_argv.c` and `keep/argv_view.c`):

```text
    start: exec_empty_argv argc0 ./argv_view   (execv with argv = {NULL})
PROBE argc=1
PROBE argv[0]=
    start: exec_empty_argv empty0 ./argv_view  (execv with argv = {"", NULL})
PROBE argc=1
PROBE argv[0]=
```

The kernel inserts the empty element
(`fs/exec.c`, the `bprm->argc == 0` block),
so the design's rule decides both without indexing anything:
the last component of `""` is none,
which is not the zedlet's name,
so both are the wrapper (case 27).
ZED never does either:
its `execle` passes one element.

### The bytes must fit the launcher's libc

A root-owned file that a guest cannot start is no evidence,
so this round read the floor each program links against:
the wrapper needs at most `GLIBC_2.39` and the handler `GLIBC_2.34`
(`readelf --version-info`),
and the Ubuntu 24.04 guest this round measured in has 2.39,
so both host-built binaries start there.
That is what made a guest measurement possible at all this round,
where the first revision round's Fedora guest refused every route in
(the four refusals are in that round's `keep/state-so-far.md`).

## Choice 1: what selects a root mode

### The three rules probed

Every rule below carries the same answer to the review's Medium finding:
the words with no mode word after them refuse with the usage,
as `service` needs `run`,
so `zfs-root` alone never starts a sweep.
They differ in what a word does under the installed name.

Option A,
the rule this design recommends:

- the two words `monochromatic-git-policy-cli zfs-root`, followed by a mode word,
  select a root mode on **any** file;
- under the installed ZED history event's name,
  ZED's no argument is the sweep,
  and `--help` and `--version` are the modes' own answers;
- under that name,
  any other word is refused before the modes' grammar sees it,
  with the usage and status 2;
- every other start is the wrapper.

Option B:

- as A for the words;
- under the installed name only ZED's no argument selects a root mode;
- every word there,
  `--help` and `--version` included,
  is refused before the grammar.

Option C,
the first revision's rule:

- the installed name selects a root mode and the modes' grammar reads whatever follows,
  with the words pair dropped when it leads them;
- the words select a root mode on any file.

All three read the name as the whole last path component,
which `Path::file_name` answers,
and refuse with status 2.

### What each decides

`keep/probe-dispatch-candidates.log`,
from `keep/dispatch_probe.rs`,
which is the three rules and nothing else,
run over twenty-eight starts.
The cases that decide the question:

- ZED's own child step,
  the bare name and no argument (case 1):
  A, B and C all decide the attaching sweep.
- the installed path,
  the words and `boot-reconcile`,
  which is a unit's `ExecStart` (case 3):
  all three decide `boot-reconcile`.
- the installed path,
  a bare `boot-reconcile`,
  which is step 19's earlier line and a person's guess (case 4):
  A and B refuse before the grammar;
  C decides `boot-reconcile`.
- the installed path,
  `--help` and `--version` (cases 5 and 6):
  A decides the grammar's help and version;
  B refuses both;
  C decides both.
- the installed path,
  the words and no mode word (case 7):
  all three refuse.
- the installed path,
  the namespace word misspelled (case 8):
  A and B refuse the first word;
  C hands the grammar `monochromatic-git-policy-cli`, which the grammar refuses as no mode.
- the installed path,
  the mode word misspelled (case 9):
  all three refuse.
- the installed path with a trailing slash (case 10),
  doubled slashes (11),
  `./` before it (12)
  and `sub/../` before it (13):
  all three decide the sweep,
  because `Path::file_name` reads the last component the same way.
- the bare name through a `PATH` lookup (case 14):
  all three decide the sweep.
- five near-miss names (cases 15 to 19):
  all three decide the wrapper,
  and a contains test would have taken three of them.
- a stray copy under another name with a bare word (case 20):
  all three decide the wrapper.
- a stray copy under the zedlet's own name in another folder,
  no argument (case 21):
  all three decide the sweep;
  with a bare word (case 22):
  A and B refuse,
  C decides that mode.
- the wrapper's own name with the words (case 23):
  all three decide the mode;
  with the words and no mode word (case 24):
  all three refuse.
- an adoption's three options through the words (case 25):
  all three decide `adopt-worktree-parent` with them.
- a caller-chosen `argv[0]` through `exec -a` (case 26):
  A and B refuse the bare word,
  C decides the mode.
- an empty `argv[0]` (case 27):
  all three decide the wrapper.

### Ranking of choice 1

Ranking:
A > B > C.

A over B:
both fail closed for the installed file,
and both let the words select a mode on any file,
which is the answer's second clause.
A answers `--help` and `--version` under the installed name,
which is the one spelling `doctor`'s installed-file check and the release smoke read today,
so a check on the installed bytes stays one command with no words in it.
B's price is that same check spelled with the words,
and its gain is a rule with one fewer arm;
that is a smaller rule, not a stronger one.

B over C:
C accepts a third spelling the answer never names,
a bare mode word under the installed name,
and it drops the words pair there,
so under the recommended layout the words would not be what selects a mode at all
(the second pass's High finding).
C is also the only one of the three that decides a bare word under a stray copy of the zedlet's name (case 22),
where A and B refuse.

The three agree on everything else the probe covers,
including the two cases a shell cannot spell.

### The rule, in the entry

The whole argument vector goes to one module and one function,
so `main` reads no element and compares no name:

```rust
// package/git-policy/cli/src/native/main.rs, the first statements of main
let arguments: Vec<OsString> = std::env::args_os().collect();
// The handler is the crate's Linux-only module, like the mount table it reads, so the root
// modes exist only in a Linux build and every other target keeps the wrapper's one mode.
#[cfg(target_os = "linux")]
{
    match root_mode_names::root_mode_start(arguments.as_slice()) {
        RootModeStart::Wrapper => {}
        RootModeStart::Arguments(mode_arguments) => {
            git_policy_cli::panic_notice::install_payload_free_panic_hook();
            let code: i32 = git_policy_cli::worktree_clone_mount_handler::run(mode_arguments);
            end_after_the_log(code);
        }
        RootModeStart::Refused(misuse) => {
            git_policy_cli::panic_notice::install_payload_free_panic_hook();
            let code: i32 = git_policy_cli::worktree_clone_mount_handler::refuse_root_mode_misuse(misuse);
            end_after_the_log(code);
        }
    }
}
// The service check, the panic hook, the environment and entry::run_process
// are the code that stands today, with `handed` taken from `arguments`.
```

`src/native/worktree_clone_mount_handler/root_mode_names.rs` is the one place the name,
the words and the grammar's argument list meet.
It holds three things:

- the constant `ZEDLET_START_NAME`, the installed ZED history event's own name;
- the constant `ROOT_GROUP_WORD`, choice 2's word;
- the function `root_mode_start`, which answers one of three things:
  the arguments the grammar reads,
  a misuse it refuses before the grammar,
  or the wrapper.

The group word compares against `foundation/wrapper_names_old_and_new.rs`'s `COMMAND_WORD.current`
and never against a literal,
so the program's own word stays in one constant.
The folder already links `foundation/` in its `uses/` set,
so no link is added,
and the folder's own top doc gains the file first,
before `handler_install_paths.rs`,
because it uses only `foundation` and the standard library.

The module, whole:

```rust
// package/git-policy/cli/src/native/worktree_clone_mount_handler/root_mode_names.rs
/// The installed ZED history event's own file name: the one start name that selects a root
/// mode, compared as the whole last path component.
pub const ZEDLET_START_NAME: &str = "history_event-mount_git_worktree_clone_datasets";

/// The namespace word the root modes sit under, beside `service`.
pub const ROOT_GROUP_WORD: &str = "zfs-root";

/// What a start under the root modes' own spellings asked for and cannot have.
pub enum RootModeMisuse<'arguments> {
    /// The root-mode words with no mode word after them.
    NoModeWordAfterTheGroupWord,
    /// A word under the installed name that is neither ZED's none nor `--help` nor `--version`.
    WordUnderTheZedletName(&'arguments OsStr),
}

/// What one start is.
pub enum RootModeStart<'arguments> {
    /// A root-mode start, with the words the modes' grammar reads.
    Arguments(&'arguments [OsString]),
    /// A start refused before the grammar, so no mode runs and Git is never reached.
    Refused(RootModeMisuse<'arguments>),
    /// The wrapper, unchanged.
    Wrapper,
}

pub fn root_mode_start(arguments: &[OsString]) -> RootModeStart<'_> {
    let (start_name, handed): (Option<&OsStr>, &[OsString]) = split_the_name(arguments);
    if starts_with_the_group_words(handed) {
        let mode_arguments: &[OsString] = &handed[2..];
        if mode_arguments.is_empty() {
            return RootModeStart::Refused(RootModeMisuse::NoModeWordAfterTheGroupWord);
        }
        return RootModeStart::Arguments(mode_arguments);
    }
    if !names_the_zedlet(start_name) {
        return RootModeStart::Wrapper;
    }
    if handed.is_empty() || (handed.len() == 1 && (handed[0] == OsStr::new("--help") || handed[0] == OsStr::new("--version"))) {
        return RootModeStart::Arguments(handed);
    }
    return RootModeStart::Refused(RootModeMisuse::WordUnderTheZedletName(handed[0].as_os_str()));
}

fn starts_with_the_group_words(handed: &[OsString]) -> bool {
    return handed.len() >= 2 && handed[0] == OsStr::new(COMMAND_WORD.current) && handed[1] == OsStr::new(ROOT_GROUP_WORD);
}

fn names_the_zedlet(start_name: Option<&OsStr>) -> bool {
    return start_name.and_then(last_component) == Some(ZEDLET_START_NAME);
}

fn last_component(start_name: &OsStr) -> Option<&str> {
    return Path::new(start_name).file_name().and_then(OsStr::to_str);
}

fn split_the_name(arguments: &[OsString]) -> (Option<&OsStr>, &[OsString]) {
    let Some((start_name, handed)) = arguments.split_first() else {
        return (None, &[]);
    };
    return (Some(start_name.as_os_str()), handed);
}
```

with its `use` lines,
`//region` markers and TSDoc as the folder's other files carry them,
and with `refuse_root_mode_misuse` in `handler_mode_dispatch.rs`,
where every other refusal is written:
it says the misuse and then the usage, and answers `EXIT_REFUSED`.

The service check follows,
so the order of the three checks is:
the one name,
the words,
the service's words.
Each reads arguments only,
which is why their order cannot change what any of them does,
and the words are checked first inside the one module,
which is what makes them the selector on every file.

### `zfs-root` alone refuses with usage

The words path needs a mode word,
so `git monochromatic-git-policy-cli zfs-root` is a refusal that the usage follows,
not a sweep.
Measured today,
the same line reaches the management arm and refuses with status 2
(`keep/probe-install-layouts.log`, the layout A case with the words and a mode);
with this change it is the root-mode arm's own refusal,
and its text names the modes and the installed path.

Only `start-probe` and `adopt-worktree-parent` check the effective user today
(`handler_mode_dispatch.rs:174` and `adopt_worktree_parent_mode.rs:315`),
so before this change a person who typed the group word and forgot the mode started a real sweep as root.
The refusal costs nothing a sweep needs:
ZED's start is the no-argument one,
and a unit always names its mode.

### What the name selects, and what the words select

The answer's two clauses are kept apart in the design like this:

- the words select a root mode on any file,
  which is what a unit,
  an adoption and a person use;
- the installed name adds ZED's no-argument sweep,
  and `--help` and `--version`,
  and turns every other word at that path into a refusal.

So under the recommended install layout (choice 3's option A)
both spellings reach the modes,
and the words remain what selects a mode:
a unit line works whether the file it starts is the installed name or another,
and the same unit line works against a stray copy.
That is the property the answer needs and the first revision's pair-dropping did not have.

### The guard change, exactly

Three guards change,
and none of them is the one the first revision wrote.

#### The hook guard on `main.rs`

`main_does_not_dispatch_on_its_start_name` becomes
`main_reads_the_one_start_name_through_root_mode_names`,
and its body becomes this:

```rust
// package/git-policy/cli/src/native/source_guard/hook_self_invocation_tests.rs
#[test]
fn main_reads_the_one_start_name_through_root_mode_names() {
    let code: Vec<&str> = code_lines(MAIN_SOURCE);
    let joined: String = code.join("\n");
    for forbidden in [
        "args_os().next()",
        "args().next()",
        ".nth(0)",
        "argv[0]",
        "arg0(",
        "start_name",
        "arguments[0]",
        "[0]",
        ".first(",
        ".last(",
        ".get(0)",
        "split_first",
        "current_exe",
        "file_name",
        "OsStr::new(",
        "OsString::from(",
        "starts_with(",
        "ends_with(",
        "contains(",
        "hook_dispatch",
        "hook_entry",
    ] {
        assert!(!joined.contains(forbidden), "main.rs contains {forbidden}");
    }
    const VECTOR_LINES: [&str; 3] = [
        "let arguments: Vec<OsString> = std::env::args_os().collect();",
        "root_mode_names::root_mode_start(arguments.as_slice())",
        "arguments.get(1..).unwrap_or_default()",
    ];
    let with_the_vector: Vec<&str> = code.iter().copied().filter(|line| { return names_the_argument_vector(line) > 0; }).collect();
    assert_eq!(with_the_vector.len(), VECTOR_LINES.len(), "main.rs names the argument vector on {} line(s): {with_the_vector:?}", with_the_vector.len());
    for line in with_the_vector {
        assert!(VECTOR_LINES.iter().any(|known| { return line.contains(known); }), "main.rs uses the argument vector in a way this guard does not name: {line}");
    }
    for known in VECTOR_LINES {
        assert!(joined.contains(known), "main.rs lacks {known}");
    }
    assert_eq!(joined.matches("args_os()").count(), 1, "main.rs reads the argument vector once");
    assert_eq!(joined.matches("root_mode_names").count(), 2, "main.rs names root_mode_names twice: its import and one call");
    assert!(joined.contains("entry::run_process("), "main.rs hands over to the wrapper's entry");
}
```

with one helper beside it,
because `mode_arguments` ends in `arguments` and a substring count would read it as a fourth use:

```rust
/// How many times one line names the vector itself (`arguments`), not a word that ends in it.
fn names_the_argument_vector(line: &str) -> usize {
    const NEEDLE: &str = "arguments";
    let bytes: &[u8] = line.as_bytes();
    let mut count: usize = 0;
    let mut from: usize = 0;
    while let Some(at) = line[from..].find(NEEDLE) {
        let index: usize = from + at;
        let before_is_a_word: bool = index > 0 && (bytes[index - 1].is_ascii_alphanumeric() || bytes[index - 1] == b'_');
        if !before_is_a_word {
            count += 1;
        }
        from = index + NEEDLE.len();
    }
    return count;
}
```

What this pins, and what it does not:

- it pins the three lines that may name the argument vector,
  so a fourth read,
  a second decision call,
  or a renamed vector fails on the count,
  and every spelling of the zeroth element fails on the list,
  which is what each of the second pass's variants trips over;
- it pins every spelling of the zeroth element the repository's vocabulary has,
  and forbids a name comparison's building blocks in `main` (`OsStr::new(`,
  `OsString::from(`,
  `starts_with(`,
  `ends_with(`,
  `contains(`);
- it does not make a denylist complete against arbitrary code,
  and it does not claim to:
  the reviewed three lines are what `main` may do with the vector,
  and any change to them is a change to this guard.

#### The scan rule's second id

`no_self_invocation_rules_tests.rs` keeps its `argv0-dispatch` rule byte for byte,
so the four old needles stay unexcused in every file,
and gains one rule beside it:

```rust
// package/git-policy/cli/src/native/source_guard/no_self_invocation_rules_tests.rs
Rule {
    id: "argv-vector-read",
    reason: "reads the argument vector, whose first element is the name the program was started under; the one collection belongs to main, which hands the whole vector to root_mode_names",
    matches: |line, _joined| {
        return line.contains("args_os()");
    },
},
```

and one `ALLOWED` row:

```rust
Allowed {
    file: "main.rs",
    rule: "argv-vector-read",
    why: "the one vector the owner's answer needs: main collects it once, names it on the three lines its own guard fixes, and hands it whole to root_mode_names",
},
```

The reasons this is the right shape:

- the row is keyed by file and by rule,
  so it excuses `args_os()` in `main.rs` and nothing else there,
  while `args_os().next()`, `args().next()`, `.nth(0)`, `argv[0]` and `arg0(`
  stay unexcused in `main.rs` and everywhere else;
- a release file that wants the vector now needs its own row and its own reason,
  which is a diff a reviewer reads;
- `every_justified_exception_still_matches_code` keeps both rows honest.
  Today's `main.rs` is the only release file that reads `args_os()`,
  and the second program's `main.rs`, which this change deletes, is the other.

#### The one-name guard on the names module

A new sidecar beside the other guards,
`source_guard/root_mode_name_tests.rs`,
declared in `source_guard/mod.rs`,
reads the module and holds it to one name:

```rust
// package/git-policy/cli/src/native/source_guard/root_mode_name_tests.rs
/// The one module allowed to read the name this program was started under.
const NAMES_SOURCE: &str = include_str!("../worktree_clone_mount_handler/root_mode_names.rs");

#[test]
fn the_one_start_name_is_written_in_one_place() {
    let code: Vec<&str> = code_lines(NAMES_SOURCE);
    let joined: String = code.join("\n");
    let with_the_literal: Vec<&str> = code.iter().copied().filter(|line| { return line.contains("history_event-"); }).collect();
    assert_eq!(with_the_literal.len(), 1, "one line may hold a history_event- literal: {with_the_literal:?}");
    for forbidden in ["[\"", "[&str", ".any(", "is_one_of", "matches!(", "format!", "concat!", "push_str", "to_ascii_", "to_lowercase"] {
        assert!(!joined.contains(forbidden), "root_mode_names.rs contains {forbidden}");
    }
    assert!(with_the_literal[0].contains(": &str = \"history_event-"), "the one literal is the constant: {}", with_the_literal[0]);
    assert_eq!(joined.matches("ZEDLET_START_NAME").count(), 2, "the constant is its declaration and one comparison");
    assert_eq!(joined.matches("file_name()").count(), 1, "the whole last path component is read once");
    assert_eq!(joined.matches("split_first()").count(), 1, "the program name is separated once");
}
```

The three planted modules in `keep/candidates/` show each rule firing:
a second literal,
an array of names,
and a name assembled with `concat!`.

#### Replayed

`keep/guard-replay-revision2.log`,
from `keep/guard_replay_revision2.rs`,
which carries the three predicates and the `ALLOWED` table.
Today's `main.rs` is the positive control for the guard that stands.

```text
== src/native/main.rs (today)
   current  hook guard: PASS
   revised  hook guard: FAIL (main.rs names the argument vector on 4 line(s) and 4 time(s), not 3)
   scan: 1 finding(s) [("argv-vector-read", 52)] -> 0 unexcused
== the proposed main
   current  hook guard: FAIL (the program name is skipped, not inspected)
   revised  hook guard: PASS
   scan: 1 finding(s) [("argv-vector-read", 36)] -> 0 unexcused
== the proposed main with a second name compared (V3)
   revised  hook guard: FAIL (main.rs contains "OsStr::new(")
== with the name passed to a second function (V4)
   revised  hook guard: FAIL (main.rs contains ".first(")
== with an arguments[0] suffix test (V5)
   revised  hook guard: FAIL (main.rs contains "arguments[0]")
== with the first element compared as bytes (V6)
   revised  hook guard: FAIL (main.rs contains ".first(")
== with std::env::args().next() (V7)
   revised  hook guard: FAIL (main.rs contains "args().next()")
   scan: 2 finding(s) [("argv-vector-read", 36), ("argv0-dispatch", 36)] -> 1 unexcused main.rs:36 [argv0-dispatch]
== with .nth(0) (V8)
   revised  hook guard: FAIL (main.rs contains ".nth(0)")
   scan: 2 finding(s) [("argv-vector-read", 36), ("argv0-dispatch", 36)] -> 1 unexcused main.rs:36 [argv0-dispatch]
== with a second root_mode_start call
   revised  hook guard: FAIL (main.rs names the argument vector on 4 line(s) and 4 time(s), not 3)
== the proposed root_mode_names.rs
   one-name guard: PASS
== a module with a second history_event- literal
   one-name guard: FAIL (2 line(s) hold a history_event- literal)
== a module with an array of names
   one-name guard: FAIL (root_mode_names.rs contains "[\"")
== a module with a name assembled by concat!
   one-name guard: FAIL (root_mode_names.rs contains "concat!")
== the second pass's own V3 to V8, as they stand
   V3, V4, V5, V6: FAIL (main.rs contains "start_name")
   V7: revised FAIL ("args().next()"), scan 1 unexcused main.rs:55 [argv0-dispatch]
   V8: revised FAIL (".nth(0)"), scan 1 unexcused main.rs:55 [argv0-dispatch]
```

So each of the second pass's six variants fails a guard,
and the variants that were rewritten against this revision's `main` fail too.
The full log has every line.

`SPEC.md:19101` becomes:

> Git never starts the wrapper for a hook:
> the wrapper writes no hook entry,
> passes no hooks path of a directory it made,
> and reads the name it was started under in one module only,
> the installed ZED history event's,
> where that name selects the clone-mount sweep and nothing else a Git command means.

`binary_external_hook_tests.rs:170` keeps proving that every Git hook event's name still behaves as `git`,
which is now a claim about every name but one.

## Choice 2: the group word units, adoption and a person spell

The answer fixes the shape of the words,
because `service run` is the precedent:
the wrapper's own word,
then a namespace word,
then the modes' grammar.
Which namespace word it is, is this choice's,
and a unit's `ExecStart` line,
an adoption's command and a person's command all read it.

Every option below is shown with the start it produces.
The words are not in the build yet,
so the "decides" line is the fixture's
(`keep/probe-dispatch-candidates.log`),
and the text after it is what the modes print today
(the real `--help` and `--version` of the built handler,
and the refusal the wrapper's words line answers today).

### Option A: `zfs-root`

```text
$ git monochromatic-git-policy-cli zfs-root --help
   decides: root-mode grammar="--help"
   prints:  usage: mount-git-worktree-clone-datasets [mode]
              (no argument)        run the clone-mount sweep the installed ZED event starts
              boot-reconcile       reconcile clone mounts after boot; the boot unit's ExecStart
              start-probe          run the start probe's arms and write this boot's result
              shutdown-unmount     unmount the verified clone mounts at shutdown
              holder-scan --dataset <name> [--nonce <32 hex digits>]
              adopt-worktree-parent --checkout <path> --device <count> --inode <count>
              --help               print this text
              --version            print this build's version
```

```ini
# /etc/systemd/system/reconcile-git-worktree-clone-mounts.service
ExecStart=/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets monochromatic-git-policy-cli zfs-root boot-reconcile
```

```sh
sudo /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets \
  monochromatic-git-policy-cli zfs-root adopt-worktree-parent \
  --checkout /home/user/Monochromatic --device 66306 --inode 1234567
```

```text
$ git monochromatic-git-policy-cli zfs-root
   decides: refused-before-the-grammar word="monochromatic-git-policy-cli zfs-root"
   today:   the management arm's usage, exit=2
```

Pros:

- names the route and the privilege,
  which is what the mode family is called everywhere else in this route's writing;
- no Git word it could be mistaken for,
  and long enough that running it is a deliberate act;
- one word for six modes,
  so a reader who knows `service` knows this.

Cons:

- the second longest of the four,
  and an adoption's line already carries three options;
- it pairs the route's name with the privilege's,
  so a reader has to learn which half is which.

### Option B: `mount`

```text
$ git monochromatic-git-policy-cli mount boot-reconcile
   decides: root-mode grammar="boot-reconcile"
```

```ini
ExecStart=/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets monochromatic-git-policy-cli mount boot-reconcile
```

Pros:

- the shortest that still says the work,
  and the family is the clone mounts (the design's own name for it and the route's `…-mounts` file names);
- a person types it more easily than the longer ones.

Cons:

- names only the mounting modes,
  while `start-probe` and `holder-scan` are not mounts;
- `mount` is a word any later mode family would also want,
  so it claims a broad name for a narrow set.

### Option C: `root`

```text
$ git monochromatic-git-policy-cli root boot-reconcile
   decides: root-mode grammar="boot-reconcile"
```

Pros:

- shortest, and honest about the privilege the modes need.

Cons:

- reads as a Git concept (a root commit, the root tree),
  so `git monochromatic-git-policy-cli root …` invites a wrong guess about what it does;
- says nothing about the ZFS route,
  so a person who does not already know the route learns nothing from it.

### Option D: `clone-mounts`

```text
$ git monochromatic-git-policy-cli clone-mounts boot-reconcile
   decides: root-mode grammar="boot-reconcile"
```

```ini
# the unit is already named this, so the word and the unit would agree
ExecStart=/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets monochromatic-git-policy-cli clone-mounts boot-reconcile
```

Pros:

- the route's own noun,
  which its configuration file (`git-worktree-clone-mounts.jsonc`)
  and its unit (`reconcile-git-worktree-clone-mounts.service`) already carry;
- names the work rather than the privilege,
  so a person reading a unit learns what runs;
- no Git word and no other family's word.

Cons:

- four of the six modes are clone mounts,
  while `start-probe` and `holder-scan` are not,
  which is the same objection as option B's, narrower;
- the longest of the four to type,
  twelve characters against `zfs-root`'s eight;
- says nothing about the privilege,
  which is what an adoption's line is about.

### Ranking of choice 2

Ranking:
A > D > B > C.

A over D:
both name the route,
and A names the privilege the words path exists for,
which is what a unit's `ExecStart` under root and an adoption under `sudo` are about;
D names the work and its noun covers four of the six modes,
while A's word covers the four modes and the two probes by naming where they run.
A is also the word this design's other sections use,
so the reader meets one spelling.

D over B:
both are about mounts,
and D names the whole route where B claims a word any later mount-shaped family would want,
while B's shortness is paid for by a name that will collide.

B over C:
both are shorter than A and D,
but `mount` at least says what the family does,
while `root` collides with a Git word's meaning.

The choice is separable from every other one here.
If the owner prefers a different word,
what changes is:
the constant in `root_mode_names.rs`,
the fixture's cases,
the unit text in step 19,
the wrapper's management help line,
`doctor`'s printed commands,
the release smoke's second arm,
`native:install`'s words check,
`SPEC.md`'s sentence about the words,
and the guest commands in `e2e/start-probe-guest.ts`.
Nothing about the name,
the layouts or the guards changes.

## Choice 3: what root's install holds

The review asked that root's regular file carry a mode's name,
not the wrapper's,
so that a root start under any other name fails with `No such file`.
The answer says units and adoption reach the modes by words.
Both hold at once,
and the layouts below are what fits them together.

What selects a mode under each layout,
with choice 1's recommended rule:

- under the ZED name in `/etc/zfs/zed.d`,
  ZED's own start selects a root mode in every layout,
  because ZED passes the link's name as `argv[0]` and that name is the zedlet's;
- under **option A's** file in `/usr/libexec`,
  the words select a mode,
  the name selects ZED's no-argument sweep and the two answers,
  and every other word is a refusal,
  so no argument vector at that path reaches Git;
- under **options B and C's** file in `/usr/libexec`,
  only the words select a mode,
  and any other argument vector at that path is the wrapper,
  so Git runs as root;
- under **option D**,
  each link carries its own name,
  so a link named for the zedlet behaves as option A and a link named otherwise as option B or C.

Each layout was run as the account in a throwaway `git init` repository,
with today's wrapper's bytes as the installed file
(`keep/probe-install-layouts.log`),
because the dispatch itself is choice 1's and this choice is about names on disk.
Today's bytes carry no dispatch,
so each block below shows what a start does now,
what the layout's name makes it do under this revision's rule,
and what it writes into the repository it stands in.

### Option A: one regular file under the ZED hook's own name

```text
/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets   regular, 0755, root
/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets -> ../../../usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets
```

```text
$ /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets
    today:   git's usage, exit=1; a new lock-holder record
    after:   the attaching sweep
$ .../history_event-mount_git_worktree_clone_datasets boot-reconcile
    today:   git: 'boot-reconcile' is not a git command, exit=1; a new record
    after:   a refusal with the usage, exit=2, no mode
$ .../history_event-mount_git_worktree_clone_datasets monochromatic-git-policy-cli zfs-root boot-reconcile
    today:   the management usage, exit=2; a new record
    after:   boot-reconcile
$ .../history_event-mount_git_worktree_clone_datasets --version
    today:   git version 2.56.1 and the wrapper's banner, exit=0; a new record
    after:   mount-git-worktree-clone-datasets 0.6.1-alpha.45, exit=0
$ .../history_event-mount_git_worktree_clone_dataset --version
    today:   No such file or directory, exit=127
$ .../mount-git-worktree-clone-datasets boot-reconcile
    today:   No such file or directory, exit=127
```

Pros:

- every start of the one root-owned file is a root-mode start,
  whatever its words,
  so a word,
  a typo,
  a stale line or a copied command at that path is a mode or a refusal,
  and never Git as root;
- the wrapper's own name is absent from root's folder,
  which is the review's request,
  and so is the modes' name,
  so both old spellings fail loudly;
- ZED follows the symlink and passes the link's name as `argv[0]`,
  which is the name rule's input;
- one file to verify the digest of and one to replace on upgrade;
- the unit line's words select the mode (choice 1's recommended rule),
  so the answer's second clause holds there.

Cons:

- a file named `history_event-…` in `/usr/libexec` reads as a zedlet copy,
  and a unit starting it reads oddly until one knows the rule;
- the installed path that the modes' messages print is long;
- `SERVICE_INSTALL_PATH`, whose name means "where service calls start it",
  is renamed (to `ROOT_MODE_INSTALL_PATH`), because nothing about it is a service call any more.

### Option B: one regular file named for the modes

```text
/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets   regular, 0755, root
/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets -> ...        symlink
```

```text
$ /usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets boot-reconcile
    today and after: git: 'boot-reconcile' is not a git command, exit=1; a new record
$ /usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets
    today and after: git's usage, exit=1; a new record
$ /usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets monochromatic-git-policy-cli zfs-root boot-reconcile
    after: boot-reconcile
```

Pros:

- the file's name,
  the version line and every refusal's program name agree;
- a unit's path reads for what it does.

Cons:

- a bare mode word at that path,
  which is exactly step 19's own line as the plan carries it today,
  runs the whole wrapper as root:
  Git, its configuration, its lock pass,
  and a record in whatever repository the caller stands in.
  That is the fail-open the review named;
- a start under that name with a word a person guessed (`boot-reconcil`)
  does the same,
  because neither that file's name nor a misspelled word selects a root mode;
- the words are the only way in at that path,
  so a unit line that loses them runs Git as root.

### Option C: one regular file named for the wrapper

```text
/usr/libexec/monochromatic-git-policy-cli/monochromatic-git-policy-cli   regular, 0755, root
/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets -> ...    symlink
```

```text
$ /usr/libexec/monochromatic-git-policy-cli/monochromatic-git-policy-cli boot-reconcile
    today and after: git: 'boot-reconcile' is not a git command, exit=1; a new record
$ /usr/libexec/monochromatic-git-policy-cli/monochromatic-git-policy-cli
    today and after: git's usage, exit=1; a new record
$ /usr/libexec/monochromatic-git-policy-cli/monochromatic-git-policy-cli --version
    today and after: git version 2.56.1 and the wrapper's banner, exit=0; a new record
```

Pros:

- a unit's path reads as the wrapper a person knows.

Cons:

- any root start under that name is the whole wrapper:
  no root mode at all,
  and Git as root,
  including a stale unit line that names the file and no words;
- it puts the wrapper's own bytes,
  reachable as a plain `git`,
  in root's install folder,
  which is the review's finding exactly.

### Option D: a name per file, by copy or by hard link

The same names as the first three options,
each a copy or a hard link of the one program.

```text
$ ln /usr/libexec/monochromatic-git-policy-cli/monochromatic-git-policy-cli /usr/libexec/monochromatic-git-policy-cli/holder-scan
$ /usr/libexec/monochromatic-git-policy-cli/holder-scan --dataset tank/checkout
    today and after: git's usage, exit=1, because that name selects nothing
```

Pros:

- no link can dangle, with hard links.

Cons:

- a copy per name costs a copy per upgrade and can drift;
- a hard link made before an upgrade that writes a new file and renames it over the old stays on the previous release,
  so a zedlet or a unit would run old bytes without a word saying so,
  which is the failure this route can least afford;
- the names multiply the fail-open:
  every name that is not the zedlet's is a Git start under root until the words are given.

### Ranking of choice 3

Ranking:
A > B > C > D.

A over B:
both keep one file,
but A's only root-owned file is a root-mode start whatever its words,
so a typo,
a stale line or a copied command at that path is a refusal or a mode and never Git as root,
while B's is Git as root for a bare word and for no argument at all.
The confusion A costs is a name a reader has to be told about once,
and the units' and adoption's lines are generated from the constants and printed with their values anyway.

B over C:
both fail open for a bare word at their own path,
and B's file at least names the modes it runs.
A stale line that names the wrapper's path fails with `No such file` under A and under B,
because no such file exists in either,
while under C that path exists and runs Git as root.
So B's fail-open is the arguments at its own path,
and C's is every start under its own name.

C over D:
C has one file and one upgrade step;
D multiplies both and adds a drift a person cannot see.

What none of them prevents,
and this design does not pretend otherwise:
a copy of the wrapper's bytes somewhere else,
started as root under a name of its own,
runs the whole wrapper
(`keep/probe-install-layouts.log`, the stray copy's two starts).
An account that can write root's install folder can replace any of these files,
which is why `doctor` checks owner,
mode,
the ancestor folders and the version the installed file prints.

The choice is separable from the others.
If the owner prefers B or C,
the unit lines keep one shape and only the path in them changes;
what is lost is the fail-closed property the review asked for,
and choice 1's ranking would have to say so for that option.

## Choice 4: where the dispatch sits

What a placement costs root is every start-time action after it that runs.

### Every start-time action of today's wrapper

In the order the code takes them,
with what a root-mode start does at choice 4's option A for each:

1.  `args_os` collection in `main` (`main.rs:52`).
    Read:
    the vector is what a root mode is decided from.
2.  The one-name check and the words check (this design's, in `root_mode_names`).
    Read:
    they are the decision.
3.  `service_main::is_service_start` (`main.rs:56`),
    which reads the arguments only.
    Skipped:
    the root-mode arm answers first.
4.  `panic_notice::install_payload_free_panic_hook`.
    Installed inside the root-mode arm, as "The panic hook a root mode installs" recommends.
5.  `vars_os` collection.
    Skipped:
    a root mode reads no variable through this collection (`entry`'s own arm would).
6.  `runtime_probe::is_requested` (`entry.rs:344`),
    a test-only variable.
    Skipped.
7.  `hooks_off::activate` (`entry.rs:348`),
    a process-wide switch.
    Skipped.
8.  `abandoned_locks::meet_the_locks_of_the_working_directory` (`entry.rs:353`):
    the working directory,
    `GIT_DIR`,
    one ancestor walk for a Git directory,
    then a record written into the common Git directory
    and any lock a proof says is abandoned removed.
    Skipped,
    and this is the one that writes:
    `keep/probe-install-layouts.log` shows today's bytes replacing that record on every start of every layout,
    `--version` included.
9.  `process_resolution_inputs` (`entry.rs:356`):
    `current_exe`,
    `current_dir`,
    `PATH`,
    `PATHEXT`,
    `XDG_RUNTIME_DIR`,
    the common Git paths.
    Skipped.
10. `refusal_when_started_as_git`:
    the chain marker in the environment,
    this start's record file,
    and the process table for the parent.
    Skipped.
11. `plan_invocation`'s first check,
    the forwarded-to-self marker:
    one environment value and one file identity.
    Skipped.
12. `plan_invocation`'s control stripping and global layout,
    arguments only.
    Skipped.
13. The management arm,
    `plan_management_with_section`:
    the repository configuration,
    the hooks scan,
    and the service.
    Skipped.
14. `repository_link_stop`:
    `HOME`,
    the working directory,
    and the identity of the 860 drive link.
    Skipped.
15. `resolve_real_git`:
    the `PATH` scan,
    file identities,
    and the service unit's recorded Git.
    Skipped.
16. `invocation_overlay` and `announced_query`.
    Skipped.
17. `git_facts`,
    which starts Git.
    Skipped.
18. `shipped_checks`,
    which reads `monochromatic-git-policy-cli.config.jsonc`.
    Skipped.
19. `run_wrapped_command`:
    policies,
    the service socket,
    Git LFS,
    the linked-worktree copy.
    Skipped.
20. `forward_action` and `stack_if_push`:
    the worktree copy,
    the auto-push,
    the LFS upload,
    the network.
    Skipped.

So before it knows the mode,
the entry reads the argument vector and nothing else:
no variable of its own,
no file,
and no descriptor.
The loader's page-in of the program and of libc is the only other read,
and "What else the modes keep, and what they lose" measures it.

What a root-mode start runs instead is today's handler:
the grammar,
the compatibility gate,
the mode,
and the two log waits at exit.

The probes show the difference with one binary and one working directory
(`keep/probe-install-layouts.log`):
every start of today's bytes in a repository replaces that repository's lock-holder record,
and a main-level dispatch writes none of them.
Under ZED the working directory is `/`,
so there is no repository to write into,
and a person's start from inside a checkout,
such as an adoption's,
is where the record would land.

### Option A: the first statements of `main`

The `#[cfg(target_os = "linux")]` block of "The rule, in the entry",
before the service check,
before the panic hook,
and before the environment.

```text
$ cd /home/user/Monochromatic
$ <the installed file> monochromatic-git-policy-cli zfs-root boot-reconcile
   the mode's own line, exit=0 or 2
   .git/cli-git-lock-holders: unchanged
```

(the order is `entry.rs`'s: the lock pass at `:353` is the first write and it is reached only through
`run_process`, and every start in `keep/probe-install-layouts.log` that reached it replaced a record)

Pros:

- reaches none of the other start-time actions,
  only the first;
- the same placement the service mode already proves,
  so the entry keeps one shape for three kinds of mode;
- nothing of the wrapper's own start can fail before root's work;
- the checks are the two the answer names,
  so no other placement can claim to follow it.

Cons:

- the entry now has three mode checks before the wrapper's path,
  and their order must be stated:
  the one name,
  the words (both inside `root_mode_names`),
  the service's words,
  each reading arguments only.

### Option B: after the panic hook and the environment

```text
$ <the installed file> monochromatic-git-policy-cli zfs-root --help
   the modes' usage, exit=0
   the environment: collected for a start that reads none of it
```

Pros:

- a panic in a root mode would print the wrapper's payload-free message without a second installation;
- the environment is already collected if a mode ever wants it.

Cons:

- `main` collects the whole environment for a start that reads none of it;
- no probe shows any benefit over A,
  and the hook can be installed inside the arm instead,
  which is what "The panic hook a root mode installs" recommends.

### Option C: at the top of `entry::run_process`

```text
$ <the installed file> monochromatic-git-policy-cli zfs-root boot-reconcile
   the mode's own line, exit=0; `run_process` must be handed the program name too
```

"At the top" is exactly one statement:
before `runtime_probe::is_requested` at `entry.rs:344`,
and so before `hooks_off::activate` (`:348`),
the lock pass (`:353`)
and `process_resolution_inputs` (`:356`).

What that costs:

- what option B costs:
  `main` has already installed the hook and collected the environment by the time `run_process` is called,
  so a check at its top saves none of that;
  the lock pass and everything after it are skipped, which is option B's shape too;
- a parameter the library does not have:
  `run_process` is handed the arguments after the program name
  (`main.rs:72`),
  so a dispatch there needs the vector whole or the name beside it,
  and its second caller cannot supply either.
  `merge_second_pass_finish.rs:431` (`run_nested`) calls `run_process` with the words of a Git command
  (`["commit", ...]`) and no program name,
  so the entry would need a parameter that one call site can only fill with "there is none".
  A root-mode start never re-enters there,
  so that is new state to keep honest for no measured gain.

Pros:

- one place decides everything,
  and `main` stays as thin as it is.

Cons:

- the cost is B's cost, not the lock pass's, which the first revision's text said wrongly;
- `run_process`'s signature changes,
  and the one in-process re-entry has no honest value for the new parameter;
- the entry would hand a start name into the library for a decision only the executable's own start needs.

### Option D: inside `plan_invocation`

A check made here,
where the arguments are already planned,
costs every action before it.

```text
$ <the wrapper> monochromatic-git-policy-cli zfs-root boot-reconcile
   Usage: git monochromatic-git-policy-cli check (--all | -- <pathspec>...) [--policy <id>]...
          ...
   exit=2
   .git/cli-git-lock-holders: a new record
```

(measured today, with the bytes this host has: the words reach the management arm, which refuses them,
and the start has already written into the repository it stands in)

Pros:

- nothing of `main` or `run_process` changes shape,
  so no other mode's start moves.

Cons:

- the runtime probe,
  the hook switch,
  the lock pass,
  the resolution and the started-as-Git check all run first,
  so root writes into whatever repository holds the working directory
  (`keep/probe-install-layouts.log`);
- `git_facts`,
  the configuration read and the forwarded Git are one branch away.

### The panic hook a root mode installs

The wrapper's hook prints one line naming the source location and never the panic's message,
because the message can hold bytes of a scanned file
(`invocation/panic_notice.rs`).
A root mode does not scan,
but its panic message can hold a dataset name,
a checkout path,
or a `ZFS` request's fields,
and it lands in root's journal under a unit.

- A root mode installs the wrapper's existing hook (recommended):
  one call inside the arm,
  no new text,
  and the notice's hint ("if you were running a Git command") reads oddly from a unit,
  which is a cost this design accepts rather than keeping a second hook text true.
- A root mode keeps the default hook:
  no code,
  and the panic's payload printed.

### Ranking of choice 4

Ranking:
A > B > C > D.

A over B:
B collects an environment nothing reads,
and installing the hook inside the arm is one line,
so B's only gain is gone.

B over C:
they cost the same start-time work,
so the ranking is about where the code lives:
B keeps the decision in the executable's entry,
which is where the answer's precedent (`service run`) puts it and where the vector already is,
while C moves it into the library on a parameter that its one in-process caller cannot fill.

C over D:
D adds the lock pass,
the resolution and the started-as-Git check before the mode,
and it is the only option whose cost grows with the caller's working directory.

## Choice 5: what `--version` and the refusals say

The modes' grammar,
its refusals and its version line name the program
`mount-git-worktree-clone-datasets`
(`handler_install_paths.rs:28`).
After the change no root-owned file need carry that name,
while every check reads those words:
`handlerVersionLine` (`bin/native-install-steps.ts:121`),
the release smoke's `grep`,
`doctor`'s comparison,
and the modes' own refusal texts.

### Option A: keep the modes' own name

```text
$ /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets --version
mount-git-worktree-clone-datasets 0.6.1-alpha.45
```

(the line the built handler prints today: `./handler --version` answers
`mount-git-worktree-clone-datasets 0.6.1-alpha.45`, exit 0;
no installed file answers it yet, and `keep/probe-install-layouts.log` shows
today's bytes answering `--version` with Git's own line instead)

Pros:

- every existing check,
  test and refusal text keeps working,
  and a check can tell "this build has the root modes" from the wrapper's own banner,
  which is `monochromatic-git-policy-cli 0.6.1-alpha.45, shadowing git at /usr/bin/git`;
- the name describes the mode family rather than a file,
  which stays true however the file is installed.

Cons:

- a person reads a name no command has,
  so every refusal and the version line must also print the words and the installed path.

### Option B: print the wrapper's name and a modes line

```text
$ /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets --version
monochromatic-git-policy-cli 0.6.1-alpha.45
root modes: boot-reconcile, shutdown-unmount, start-probe, holder-scan, adopt-worktree-parent, the sweep
```

Pros:

- the first line names the program a person runs.

Cons:

- its first line collides with the wrapper's own `git --version` banner,
  which already begins `monochromatic-git-policy-cli 0.6.1-alpha.45,`,
  so a check on the installed file's first line could no longer tell which program answered;
- `native:install`, `doctor` and the smoke all need new text,
  and the modes' refusals need a new program name.

### Option C: print the words and the version

```text
$ /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets --version
monochromatic-git-policy-cli zfs-root 0.6.1-alpha.45
```

(written, not run: no build answers the words yet)

Pros:

- the line is a spelling a person can type,
  so the answer's "a name no command has" objection to A is answered;
- it is unique:
  the wrapper's banner continues with the version and a comma,
  and this line continues with the group word,
  so one `grep` tells them apart;
- a refusal can begin with the same words and stay findable.

Cons:

- the group word (choice 1's rule, choice 2's pick) becomes part of the version line,
  so changing it changes the line a check reads;
- the modes' program name in every refusal grows from one thirty-four character family name
  to two words plus the family,
  and each refusal is already two sentences long;
- `handlerVersionLine`, `native:install`'s check, `doctor`'s comparison,
  the smoke's `grep` and the stand-in handler all change,
  where A changes none of them.

### Ranking of choice 5

Ranking:
A > C > B.

A over C:
both are unique lines a check can read,
and both let a check tell the modes' answer from the wrapper's banner;
A keeps one short name in every refusal and every existing check,
where C buys a runnable spelling at the price of a longer name in every message and a new line in five checks.
The design already requires the refusal to print the words and the installed path,
so a person who needs a command to type has it without the version line's help.

C over B:
both name something a person runs,
but C's line is unambiguous on its first word after the program's,
while B's first line is the wrapper's banner's prefix,
so a check would have to read the second line.

The choice is separable.
If the owner prefers C,
`handlerVersionLine` and the three checks change,
and nothing about the dispatch,
the layouts or the guards does.

## Choice 6: whether the account's `PATH` keeps a short name

Today a person can type `mount-git-worktree-clone-datasets adopt-worktree-parent …`
because the toolchain installs the second program.
After the change the account's toolchain installs one program,
and a root mode needs the words in any case.

### Option A: the installed path and the words, printed by `doctor`

```text
$ sudo /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets \
    monochromatic-git-policy-cli zfs-root adopt-worktree-parent \
    --checkout /home/user/Monochromatic --device 66306 --inode 1234567
   decides: root-mode grammar="adopt-worktree-parent --checkout /home/user/Monochromatic --device 66306 --inode 1234567"
```

and a person's read-only questions through the account's own wrapper:

```text
$ git monochromatic-git-policy-cli zfs-root --help
   decides: root-mode grammar="--help"
   prints:  the modes' usage, the words' spelling, and the installed path
```

`doctor` prints the adoption line with the checkout's device and inode filled in,
beside the three unit lines the setup writes,
and it prints the root-owned path,
never `sudo git monochromatic-git-policy-cli …`,
whose `git` is the account's file.

Pros:

- the path a person types is the file root's units run,
  so `--version` describes the bytes that will act;
- `doctor` already prints concrete commands it never runs;
- no other package is involved,
  and a `sudo` that preserves `PATH` finds no root-mode file there.

Cons:

- the long command,
  and every refusal inside the modes that names a follow-up command must be rewritten to it.

### Option B: the toolchain links a short name beside `git`

```text
$ sudo zfs-root monochromatic-git-policy-cli zfs-root adopt-worktree-parent \
    --checkout /home/user/Monochromatic --device 66306 --inode 1234567
   decides: root-mode grammar="adopt-worktree-parent --checkout /home/user/Monochromatic --device 66306 --inode 1234567"
```

(the link is the account's; the words are still needed, which is why they appear twice)

Pros:

- shorter than the installed path,
  so a person types fewer characters.

Cons:

- the link points at the account's copy,
  so the version a person reads describes a different file from the one root would run;
- a `sudo` that preserves `PATH` runs an account-writable file as root,
  which is one more path to an escalation this design does not need;
- the toolchain's plan expresses one post-install link per tool
  (`mpm-shadow`'s `postinstall.link` takes `sourceBin`, `linkName` and `linkDir`),
  so a second link needs that package's schema to change first,
  which is a cost to state and not the reason the ranking puts B second.

### Ranking of choice 6

Ranking:
A > B.

A over B:
A keeps one file behind every name,
and the thing a person reads is the thing root runs;
B's saving is typing,
and it moves the thing a person runs away from the thing root runs,
while adding an account-writable file that a `sudo` can execute.
B should come back as its own proposal if the long path bites in use,
and it should then link to root's copy.

## Choice 7: whether the release keeps the modes' test-only variables

The modes' code reads five variables,
all compiled into the release and all inert unless a control sets them
(`src/native/start_probe/probe_test_switch.rs:29` and `:35`,
`src/native/start_probe/probe_deadline.rs:69`,
`src/native/foundation/diagnostic_log.rs:40` and `:67`):

- `CLI_GIT_TEST_ONLY_START_PROBE`,
  which can disable the probe or make it hold a step;
- `CLI_GIT_TEST_ONLY_START_PROBE_KEEP_DESCRIPTOR`;
- `CLI_GIT_TEST_ONLY_START_PROBE_DEADLINE_MS`;
- `MONOCHROMATIC_VERBOSE` and `MONOCHROMATIC_WARN`,
  which change what is logged and nothing that is decided.

A sixth,
`CLI_GIT_TEST_ONLY_RUNTIME_BUILD_MILLIS`,
is read only by the lazy service runtime,
which no root mode builds today.

ZED cannot set any of them:
it builds the environment from the event and a fixed list,
and the event's own variables carry the `ZEVENT_` prefix.
A unit could set them with `Environment=`,
and so could anyone who edits the unit,
which is root.

### Option A: keep them

A control starts the installed file with a switch set and reads the record it wrote:

```text
$ CLI_GIT_TEST_ONLY_START_PROBE=disabled <installed file> monochromatic-git-policy-cli zfs-root start-probe
   today, as the account, the mode refuses before the switch is read:
   the start-probe mode runs as root, and this process's effective user is 1000. Nothing was read, mounted
   or recorded, because the record it writes decides whether a boot's clones are admitted for attachment,
   and an account that could write it could report a boot that never passed. Ways forward: run the mode as
   root, from the boot unit or by hand.
   as root, the record's verdict is skipped with its skip reason `test-switch`.
```

(printed today by the built handler; the root arm is the control's and is not run here)

Pros:

- the modes' controls keep running against the release build,
  which is what makes their guest arms evidence;
- the direction is fail-safe:
  a set switch at worst skips a record,
  and a verbosity variable changes lines.

Cons:

- a privileged program has switches nobody but a control needs.

### Option B: compile them out of the release

```text
$ CLI_GIT_TEST_ONLY_START_PROBE=disabled <installed file> monochromatic-git-policy-cli zfs-root start-probe
   the probe runs as it always does; the record carries no skip reason, because nothing read the variable
```

(written, not run: the switch is what a feature-less build would ignore)

Pros:

- the release binary holds no switch a test names.

Cons:

- a second build shape,
  so the guest arms would test a different binary from the released one,
  which is the reason the ranking puts B second;
- the crate already has one such precedent (`--features measurement-routes`),
  and a second would need its own tasks and lint passes,
  which is a cost to state beside that reason.

### Ranking of choice 7

Ranking:
A > B.

A over B:
keeping them preserves the one property that makes the guest arms evidence,
that the binary under test is the binary released;
B's gain is a release binary without test switches,
and its cost is a build shape whose arms test something else.
No probe here measures either direction,
and the design says so rather than claiming a measurement it did not make.

## What else the modes keep, and what they lose

Kept,
because the modes are the same code and the same grammar:

- the mode words,
  the adoption's three options,
  the holder scan's two,
  the usage text and the refusals;
- exit status 0 for a mode that did its work and 2 for a refusal;
- the compatibility gate before the first raw request,
  and the probe's skipped-record arm;
- the installed path spellings of the ZED hook and the configuration,
  and the modes' program name in messages and `--version` (choice 5);
- the two log waits at exit.

Changed:

- `--version` and `--help` are reached through the one name or the words,
  and the line is unchanged,
  so `doctor`'s comparison (step 18) and the release smoke can read it,
  and `command -v mount-git-worktree-clone-datasets` is already empty on this host;
- `native:install`'s handler check becomes a words check,
  because no family name reaches the account's `PATH`;
- every message that tells a person to run `mount-git-worktree-clone-datasets ...`
  names the words and the installed path instead;
- the `--help` paragraph that claims both install paths exist says who creates them and when,
  which the first review's M1 finding asked;
- a root mode no longer passes the lock sweep,
  so a mode started from inside a repository writes nothing into that repository
  (measured in "Every start-time action of today's wrapper").

### The two programs' sizes, at one commit

`package/git-policy/cli/target/release/monochromatic-git-policy-cli` is 28,947,720 bytes
and `.../mount-git-worktree-clone-datasets` is 3,386,376,
which is 8.55 times,
both built from the merge commit this round began at.

### The cold start, measured three ways

What the numbers are, said exactly,
because the first revision's labels were wrong and its two arms did different work:

- every measured start is a **page-cache-cold** start of the program's own file.
  On this host `/home` and the build directory are ZFS,
  so a start after `POSIX_FADV_DONTNEED` on that file alone is **ARC-warm** for the file's own blocks;
- only the first start after such an eviction is cold.
  A per-event start on a running system finds the pages in memory,
  and this round did not measure that;
- the units below are **major page faults**, not pages read;
  a major fault brings in at least one page and often more,
  and the guest's in-block counts are the byte measure.

The commands each arm ran, and what each did:

- arm `w`: the wrapper,
  `./wrapper monochromatic-git-policy-cli service run status`,
  which is the wrapper's main-level mode start,
  exiting 3 (`EXIT_NOT_LISTENING`) after the service's own setup and log wait;
- arm `h`: the handler,
  `./handler --version`,
  which is the handler's whole main, grammar and version answer,
  exiting 0;
- the proposed root-mode start is arm `w`'s work plus arm `h`'s work,
  so the pair below is the increase's lower bound and their sum is its estimate,
  and the sum over-counts by whatever the two arms share (the loader and libc).

The first revision round's log holds one sample this round does not repeat:
the handler's first run took 12.26 ms and read 1,744 blocks
while its other 24 runs read none and its median was 3.57 ms.
That is its one disk-cold sample,
and its median discarded it.
This round reports it apart here rather than folding it into a median.

Host,
page-cache-cold and ARC-warm,
41 rounds,
arms interleaved and alternating
(`keep/host-wrapper-service-against-handler.txt`, `keep/host-spread-handler-against-itself.txt`,
analysed in `keep/cold-start-analysis.txt`):

- `w`: median 5.69 ms, 421 major faults;
- `h`: median 2.05 ms, 78 major faults;
- `w - h` per round: median **+3.61 ms**, range 2.57 to 8.60 ms;
- `w / h` per round: median **2.81**, range 1.84 to 5.06;
- the control, `h` against itself: median -0.01 ms, ratio 1.00 (0.79 to 1.72),
  so the method resolves a difference of this size and not much smaller;
- a second pair as context: the same wrapper started with the words as they behave today
  (`./wrapper monochromatic-git-policy-cli zfs-root boot-reconcile`, exit 2 after the management arm's refusal)
  gave median 5.94 ms and 439 major faults, `+3.95 ms`, ratio 2.97.

Guest,
Ubuntu 24.04 (kernel 6.8.0-147, glibc 2.39, 4 CPUs),
the whole page cache dropped before every run,
21 rounds,
created and destroyed through the repository's own guest driver
(`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts create` and `mvm destroy`)
and measured by `keep/coldstart_drop.c`,
which syncs and writes 3 to `/proc/sys/vm/drop_caches` before every run.
The binaries reached the guest through `mvm push`,
which lands each file in the guest's shared folder rather than at the path it is given
(`doc/troubleshooting/mvm-push-writes-the-shared-folder-not-the-guest-path.md`),
so the guest copies each one to its own disk before measuring,
which is also why the measured bytes are read from ext4 and not from the share
(`keep/guest-disk-cold-wrapper-service-against-handler.txt`,
`keep/guest-disk-cold-spread-handler-against-itself.txt`):

- `w`: median 12.82 ms, 37 major faults, 11,384 in-block reads;
- `h`: median 7.45 ms, 15 major faults, 3,760 in-block reads;
- `w - h` per round: median **+6.06 ms**, range 0.50 to 27.60 ms;
- `w / h` per round: median **1.87**, range 1.05 to 4.00;
- the control, `h` against itself: median +0.46 ms, ratio 1.06 (0.46 to 2.96);
- the difference in disk reads is 3,760 blocks of 512 bytes against 11,384,
  which is 1.9 MB against 5.8 MB,
  so a disk-cold start of the wrapper reads about 3.9 MB more than the handler's.

The same guest measured with only each file evicted (no `drop_caches`),
which is page-cache-cold and disk-cold for that file but warm for the loader and libc,
gave `w` 11.91 ms and `h` 4.44 ms, `+7.33 ms`, ratio 2.91
(`keep/guest-page-cache-warm-wrapper-service-against-handler.txt`).

What that means for the decision:

- a root start after boot,
  or the first after an eviction,
  pays about 6 ms more than the second program did;
  the proposed start's estimate is arm `w` plus arm `h`,
  which the guest puts near 20 ms against the handler's 7.45 ms,
  and that sum counts the loader and libc twice,
  so it is the upper end of an estimate and not a measurement;
- a warm per-event start pays the pages the kernel already holds,
  which this round did not measure,
  and the design does not claim a figure for it;
- the start-path cost is the decision's accepted price;
  if it ever bites,
  the fix is a smaller early path,
  not a second program.

This round's numbers differ from the first revision round's
(wrapper 9.78 ms, handler 3.57 ms, ratio 2.68, pairs 1.46 ms and 7.65 ms for its controls)
because the host's load differed between the rounds,
which is why only the interleaved pair and the ratio are reported here,
and why the shape of the difference,
not its milliseconds,
is the finding.

### Lost

- the smaller program:
  8.55 times the bytes,
  and one more file in every page cache;
- a release that can pin or roll back the root side separately from the wrapper,
  which is the accepted cost of the placement the project already chose.
## What changes, and in which step

Every path below is re-read on `main` at `68c8dcaa21` with `rg`,
and the line numbers are that commit's.
`rg -c "mount-git-worktree-clone-datasets"` finds 122 lines in 55 files,
of which the crate's `doc/` records and one `perf/` measurement are history and stay as they are.

### One commit: the release ships one program

These cannot be separated,
because the guard reads the manifest and the release lists in one test,
and `toolchain:lock:check` compares the plan and the lock offline:

- `package/git-policy/cli/Cargo.toml`:
  remove the second `[[bin]]` (`:46` to `:48`) and its comment (`:34` to `:45`),
  and correct the comment that counts executables (`:54`);
- `package/git-policy/cli/src/native/mount_git_worktree_clone_datasets/main.rs`:
  deleted,
  with its folder;
- `package/git-policy/cli/src/native/lib.rs:403`:
  the module's doc paragraph, which says "the crate's second executable";
- `src/native/source_guard/release_bins_tests.rs`:
  its doc (`:1` to `:22`) and `RELEASED_BINS` (`:31`);
- `bin/native-release-bins.unit.test.ts`:
  the doc comment;
  its planted-second-bin control still requires its four disagreements and needs no change of logic;
- `package/crate-release/cli/releasable-crates.json:30`:
  `checkBins` holds one name;
- `.github/workflows/cargo-publish.yml`:
  the job comment (`:2430` to `:2443`),
  the `--bin` flags (`:2666`),
  the smoke step (`:2669` to `:2680`),
  and the second `bin=` copy line (`:2695` to `:2698`);
- `toolchain.plan.jsonc`:
  the row's `bins` holds one name (`:271` to `:274`),
  and `toolchain.lock.json` takes it through the tool that writes the lock offline,
  which is the command `toolchain:lock:check` runs without `--frozen-lockfile`,
  never by hand;
- `package/git-policy/cli/mise.toml`:
  the descriptions that count two executables (`:88`, `:102`, `:158`)
  and the e2e task named for the second program (`:268` to `:270`);
- `package/git-policy/cli/README.md`:
  `:11`, `:1879`, `:7133`, `:7140` and `:7983`;
  the root `README.md` holds no occurrence of the name,
  which the first revision's change list said wrongly;
- `package/git-policy/cli/e2e/README.md`:
  `:833`, `:863`, `:1089`, `:1090` and `:1093`,
  where the second program's guest task is described;
- `package/zfs/request-to-openzfs-kernel-device/e2e/README.md:539`,
  which says the fixture waits for step 5's binary;
- `package/git-policy/cli/SPEC.md`:
  "## The two executables and shipped policies" (`:249`) becomes
  "## The single executable and shipped policies" and says how the root modes are reached;
  `:255`, `:266`, `:7540`, `:21687` and `:21867` name the family through the words or the one file;
  `:19101`'s sentence is rewritten as "The guard change, exactly" gives it;
- `package/git-policy/cli/doc/step-5-second-executable-2026-11-20.md`:
  a note under its status that the second program was removed by this decision,
  with the date,
  keeping the record's own text;
- `package/git-policy/cli/doc/holder-scan-across-mount-namespaces-2026-11-22.md`
  and `package/git-policy/cli/doc/slot-identity-store-write-cost.md`:
  historical measurements that name the program as it was,
  so each keeps its text and gains a note giving the current spelling and the date
  (a moved identifier in a record keeps its original text);
- `package/git-policy/cli/perf/sweep-cost-2026-11-23-statmount-interleaved.json`:
  unchanged, for the same reason;
- `doc/decision/` and `doc/handover/`:
  untouched by this design.

### The same commit: the entry, the names module, the guards, and every test that started the second program

Deleting the `[[bin]]` breaks these at compile time,
so they land with the release lists:

- `src/native/main.rs`:
  the `#[cfg(target_os = "linux")]` block of "The rule, in the entry",
  the comment at `:53` corrected to name the one name and the words,
  and the hook installed inside the arm;
- `src/native/worktree_clone_mount_handler/root_mode_names.rs` (new):
  the two constants and `root_mode_start`,
  with its controls,
  and the folder's top doc listing it first;
- `src/native/worktree_clone_mount_handler/handler_mode_dispatch.rs`:
  `refuse_root_mode_misuse`, which says the misuse, the usage and answers `EXIT_REFUSED`,
  beside the refusals it already writes;
- `src/native/worktree_clone_mount_handler/handler_install_paths.rs`:
  `SERVICE_INSTALL_PATH` (`:32`) becomes `ROOT_MODE_INSTALL_PATH` and holds the one root-owned file's path,
  and `PROGRAM_NAME` (`:28`) stays for choice 5's option A;
- `src/native/worktree_clone_mount_handler/handler_mode_arguments.rs`:
  the four refusal messages that name the program (`:220`, `:223`, `:233`, `:245`)
  and `usage()` (`:265`, whose first line at `:267` names it too),
  whose last paragraph names both installed paths and becomes one file, its name and the words;
- `src/native/worktree_clone_mount_handler/handler_install_paths_tests.rs`
  and `handler_mode_arguments_tests.rs`,
  where the same strings are asserted;
- `src/native/source_guard/hook_self_invocation_tests.rs`:
  the test renamed and its body replaced, as "The hook guard on `main.rs`" gives it,
  with `names_the_argument_vector` beside it;
- `src/native/source_guard/no_self_invocation_rules_tests.rs`:
  the `argv-vector-read` rule and its one `ALLOWED` row;
- `src/native/source_guard/root_mode_name_tests.rs` (new)
  and `src/native/source_guard/mod.rs`'s declaration of it;
- `src/native/binary/binary_external_hook_tests.rs`:
  the doc comment of `a_start_name_that_used_to_be_an_entry_behaves_as_git` (`:170` to `:172`)
  says the one exception,
  and its body stays as it is;
- `src/native/testing/root_mode_alias.rs` (new):
  one writer of the installed name in a fixture folder,
  beside `testing/fixture_executable.rs`,
  so each test that starts a mode names one link and not one copy of the rule;
- the six test files whose `CARGO_BIN_EXE_mount-git-worktree-clone-datasets` lines are listed above:
  each starts `CARGO_BIN_EXE_monochromatic-git-policy-cli` through the fixture link,
  and its assertions on the output stay as they are;
- `src/native/binary/binary_service_mode_tests.rs:6`:
  the comment that counts two executables;
- `src/native/invocation/management_arguments.rs`:
  one line in `MANAGEMENT_USAGE` (`:47`) and one in `MANAGEMENT_HELP` (`:61`)
  for `monochromatic-git-policy-cli zfs-root <mode>`,
  and the tests that assert those texts
  (`invocation/management_arguments_tests.rs` and the two that compare the usage line);
- `src/native/health/doctor_zfs_route/identity_checks.rs:145` and `:164`:
  the two messages that tell an administrator to run the old program name;
- `src/native/health/doctor_zfs_route/handler_checks.rs`:
  `PLACEHOLDER_EXECUTABLE_PATH` (`:60`) becomes the settled root-mode path,
  `PLACEHOLDER_ZEDLET_PATH` (`:70`) becomes the installed name,
  and their controls (`route_test_facts.rs:304`, `handler_checks_tests.rs`) move with them;
- `e2e/start-probe-guest.ts:42`:
  `HANDLER` becomes the wrapper pushed into the guest plus the installed-name link its commands run,
  and every `${HANDLER} start-probe` line keeps its shape;
- `e2e/mount-git-worktree-clone-datasets-guest.ts`:
  its guest build loses the second program
  (the tarball excludes `target` at `:330`, and the guest compiles what it runs at `:340`),
  and the guest creates the installed-name link before the adoption arm runs;
- `e2e/start-probe-guest.ts:196`,
  whose `cargo build --release --locked --bins` builds one binary fewer.

### The same commit: what `native:install` and the release smoke check instead

`native:install`'s handler check becomes the words' check,
which is the contradiction's resolution:

```ts
// bin/native-install.ts, where the handler's check stands today (:447 to :460)
const modesReported = (await runGateCommandOrThrow({
  program: gitProgram,
  args: ['monochromatic-git-policy-cli', 'zfs-root', '--version',],
  deadlineMilliseconds: gitDeadlineMilliseconds,
  environment: {
    ...process.env,
    ...promptlessGitSettings,
  },
  workingDirectory: repository,
})).stdout;
printLine(modesReported.trimEnd());
if (!modesReported.includes(handlerVersionLine({ version, }))) {
  throw new NativeGateError(`${gitProgram} monochromatic-git-policy-cli zfs-root --version does not name ${version}, so this build does not answer the root modes the release asked for. What it printed: ${modesReported.trim() || '(nothing)'}.`);
}
```

`handlerProgram` keeps its use as `handlerVersionLine`'s prefix,
and its doc says the name is the mode family's, not a file's.
`bin/native-install.unit.test.ts:431`'s test
("refuses when the release archive moved only the wrapper")
expects the words' line instead of the program's,
and `bin/stand-in-install-programs-for-tests.ts` (`:169` to `:186` and `:335`)
teaches its stand-in `git` to answer the words and loses its stand-in handler.

The release smoke gains the two arms,
so a release cannot ship with either dispatch broken
(`.github/workflows/cargo-publish.yml:2669` to `:2680`):

```yaml
    - name: Smoke-run the executables
      shell: bash
      env:
        TARGET: ${{ matrix.target }}
      run: |
        set -euo pipefail
        wrapper="package/git-policy/cli/target/${TARGET}/release/monochromatic-git-policy-cli"
        help=$("$wrapper" monochromatic-git-policy-cli --help)
        printf '%s\n' "$help" | grep -q '^Usage: git monochromatic-git-policy-cli'
        links=$(mktemp -d)
        ln -s "$PWD/$wrapper" "$links/history_event-mount_git_worktree_clone_datasets"
        "$links/history_event-mount_git_worktree_clone_datasets" --version | grep -q '^mount-git-worktree-clone-datasets '
        "$wrapper" monochromatic-git-policy-cli zfs-root --version | grep -q '^mount-git-worktree-clone-datasets '
```

The first arm proves the one name selects the modes;
the second proves the words do;
both print the version line `doctor` and `native:install` compare.

### The plan and the design text

One commit,
after the entry and its tests:

- `doc/planning/cli-git-zfs-worktree-implementation-plan.md`:
  step 5 (`:929`) is rewritten as the undo,
  keeps its release list and loses the second executable from it,
  and its "What it builds" gains the one name and the words;
  `:1000`'s "The gate both binaries run" becomes one gate;
  step 18's `doctor` (`:2432`) gains the installation check "Steps 18 and 19" states;
  step 19 (`:2551`) names the one file, the symlink and the words,
  and its acceptance arm links the name and reads `--version`;
- `doc/planning/cli-git-zfs-worktree-design-proposal.md`:
  "Binding crate and compatibility" loses the separate-binary paragraph and keeps the "Keep inside cli"
  placement with this design's answer,
  "ZED reconciliation and mount safety" keeps its two installed paths and says they are one file and its name,
  and "Decided defaults" replaces the second binary's cost with the one measured here;
- `doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
  `cli-git-zfs-start-probe-design.md` and
  `cli-git-zfs-private-propagation-after-replacement-mount-design.md`
  keep their commands and gain a sentence that the name at the head of each is the one root-owned file
  (their `ExecStart`,
  `ExecReload` and probe lines name the modes' own name today,
  so each line changes to that file's path plus the words);
- `doc/planning/cli-git-no-self-invocation.md`,
  "The rule as built":
  the `argv[0]` bullet gains the one exception, citing the start-name record;
- `doc/planning/cli-git-resident-service.md`,
  the sentence that rejects a name-based dispatch:
  it stays true for the service and gains "except the ZED hook's installed name".

### Steps 18 and 19

They land in their own steps,
and nothing waits on them for the release to hold one program.

Step 18's `doctor` gains, in one place:

- the installed file exists at the root-mode path,
  is a regular file through `stat` (so the zedlet's symlink passes with its target's values),
  is root's,
  has its owner execute bit,
  and has no group or other write bit;
- the zedlet's name is one ZED's scan accepts (the class prefix and a non-letter after it);
- every ancestor folder from `/` down exists,
  is a directory,
  is root's and is not group- or other-writable,
  and a missing one prints the `install -d -o root -g root -m 0755` command that creates it
  (`/usr/libexec` is missing on this host today:
  `ls -d /usr/libexec` answers `No such file or directory`);
- the installed file's `--version` names the same version as the build `doctor` runs from,
  and a mismatch prints both and the lock row's version;
- its findings say what a broken or missing name costs (a sweep that never runs)
  and print the command that recreates it,
  through the root-owned path and never through the account's `git`;
- the three unit lines it prints, and the adoption line,
  carry the words and the checked-in constants.

Step 19 prints the one file and its name,
and its guest arm starts a unit by that path with the words and reads `--version` through it.
It says where the bytes come from and who runs it:

- the bytes are the lock row's `artifacts.<target>.url`,
  verified against that row's `sha256` before the file is installed,
  never a copy of the account's `~/.cache/monochromatic-git-policy-cli-bin/`;
- the step runs as root,
  by hand or by the runbook,
  and what root trusts is the lock file in a reviewed commit of this repository
  plus GitHub's TLS,
  which is the same trust the account's toolchain already places in that row;
  the design does not claim that the account's copy is safe to read as root.

## What the probes showed, and what they did not

### Shown

- ZED's exec shape,
  `argv[0]`,
  argument count,
  working directory,
  descriptors and built environment,
  by a replay of the source's own child step
  (`keep/probe-zed-replay.log`);
- the three dispatch rules' decisions on twenty-eight starts,
  including ZED's own,
  a unit's line,
  a bare mode word,
  `--help` and `--version`,
  the words with no mode word,
  two typos,
  a trailing slash and three dot forms of `argv[0]`,
  a `PATH` lookup,
  five near-miss names,
  a stray copy under another name and one under the zedlet's own name,
  an adoption's options,
  a caller-chosen `argv[0]`,
  an empty `argv[0]`,
  and the two vectors a shell cannot spell
  (`keep/probe-dispatch-candidates.log`);
- the three guards against today's `main.rs`,
  this revision's `main`,
  eight edits of it,
  the second pass's own V3 to V8,
  the names module and three planted modules,
  with every variant failing and each planted module failing a rule of its own
  (`keep/guard-replay-revision2.log`);
- three install layouts,
  a path typo,
  step 19's earlier line,
  a stray copy,
  and the lock-holder record each start writes into the repository it stands in,
  as the account
  (`keep/probe-install-layouts.log`);
- both programs' cold starts on this host with the command each arm ran,
  page-cache-cold and ARC-warm,
  with the method's own spread measured in the same way
  (`keep/host-*.txt`, `keep/cold-start-analysis.txt`);
- both programs' cold starts in an Ubuntu 24.04 guest,
  the whole page cache dropped before every run,
  with the same spread control,
  and the guest destroyed through `mvm`
  (`keep/guest-*.txt`, `keep/cold-start-analysis.txt`);
- the two programs' sizes at one commit,
  and `readelf --version-info`'s glibc floor for each.

### Not shown

- ZED itself:
  it needs root,
  `/dev/zfs` and a pool,
  so the replay stands in for its child step only.
  The environment and descriptor values come from the replay's own `execle`,
  not from a running ZED;
- a system unit start:
  the first revision's probe used `systemd-run --user`,
  so a system manager's `argv[0]` is read from systemd's documented behaviour and not measured here;
- any mode touching ZFS:
  this round ran no guest arm that mounts, unmounts or adopts,
  so the modes' own work is unchanged code and is not exercised here;
- a warm per-event start's cost:
  this round measured cold starts only,
  and the design says so instead of quoting a figure;
- a disk-cold start on the host:
  the host cannot drop its page cache without root,
  which is why the guest arm exists;
- a comparison of ARC-cold starts:
  the guest has no ZFS,
  so its numbers are a disk-cold floor on ext4,
  not a ZFS ARC measurement;
- whether `doctor`'s future installation check catches a link that a package manager renames instead of deleting;
- the release smoke arm itself:
  it is written here and not run,
  because this round changes no workflow;
- the two refusals' exact texts:
  `refuse_root_mode_misuse` is designed here and does not exist yet,
  so its wording is this design's, not a build's.

## The choices this proposal puts to the owner

Each is separable,
and each is ranked in its own section.
Items 2 to 7 are the choices;
item 1 is the rule this design recommends,
which is where the answer's two clauses meet.

1.  What selects a root mode:
    the words on any file, and under the installed name ZED's no argument, `--help` and `--version`,
    with every other word there refused (choice 1, A);
    the same without `--help` and `--version` under the name (B);
    or the first revision's rule, where the name wins and drops the words (C).
    The design recommends A.
2.  The group word the units and people spell:
    `zfs-root` (choice 2, A),
    `clone-mounts` (D),
    `mount` (B),
    or `root` (C).
3.  What root's install holds:
    one file under the ZED hook's name and nothing under the wrapper's (choice 3, A),
    one file named for the modes (B),
    one file named for the wrapper (C),
    or a copy or hard link per name (D).
    Under A the name selects ZED's start and the words select every mode;
    under B and C only the words select, and a stray argument at that path runs Git as root.
4.  Where the dispatch sits:
    the first statements of `main` (choice 4, A),
    after the panic hook and the environment (B),
    at the top of `run_process` (C),
    or inside `plan_invocation` (D),
    with the panic hook installed inside the arm (the recommendation).
    B and C now rank on where the code lives, not on start-time work:
    they cost the same.
5.  What `--version` and the refusals say:
    the modes' own name (choice 5, A),
    the words and the version (C),
    or the wrapper's name and a modes line (B).
6.  Whether the account's `PATH` keeps a short name:
    the installed path and the words, printed by `doctor` (choice 6, A),
    or a second toolchain link (B).
7.  Whether the release keeps the modes' test-only variables:
    keep them (choice 7, A),
    or compile them out (B).

This proposal's own ranking is A in every one of them.

## Appendix: where the probe transcripts are

Round folder `/home/user/temp/agent/zfs-root-modes-second-revision-2026-11-24/keep/`:

- `state-so-far.md`:
  what the round did, in order, and what it could not do;
- `probe-dispatch-candidates.log`,
  from `dispatch_probe.rs`:
  the three rules on twenty-eight starts;
- `probe-zed-replay.log`,
  from `zed_replay.c` and `replay_fixture.rs`:
  ZED's child step, replayed;
- `probe-install-layouts.log`:
  three layouts of the one root-owned file, as the account, with each start's lock-holder record;
- `guard-replay-revision2.log`,
  from `guard_replay_revision2.rs`:
  the three guards against every `main` text and every names module,
  with `candidates/` holding the texts;
- `host-spread-handler-against-itself.txt`,
  `host-wrapper-service-against-handler.txt`,
  `host-wrapper-words-against-handler.txt`:
  the host cold-start arms, raw;
- `guest-disk-cold-wrapper-service-against-handler.txt`,
  `guest-disk-cold-spread-handler-against-itself.txt`,
  `guest-page-cache-warm-wrapper-service-against-handler.txt`:
  the Ubuntu guest's arms, raw,
  from `coldstart_drop.c`;
- `cold-start-analysis.txt`,
  from `analyse_cold_start.ts`:
  the medians, pairs and ratios of every arm above;
- `guest-logs/`:
  the guest driver's own `mvm create` log.

The round's `scratch/` folder held the compiled fixtures,
the throwaway repository and the guest's working files,
and was removed when the round ended;
the kept sources rebuild every log.
The guest `zfs-root-modes-cold-start-2026-11-24` was created through the repository's own driver
and destroyed with `mvm destroy`,
and `mvm list` no longer names it.
