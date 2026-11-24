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
and what the owner has answered since,
about the boot mount provider and the ordering of plan step 19 in
[the step 19 record](../decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md)
and about which checkouts the route serves in
[the step 10 record](../decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md).
Every claim about a program this repository does not own is read from that program's source,
and every claim about this repository's code is re-read on `main` at `f3d81e0910`,
the commit this round merged into its branch.
The decision records and the handover are untouched by this round.

This is the third revision,
the round of 2026-11-24T14:05:00-05:00.
It replaces the text of the second revision
(`37143a1daf`),
which a same-vendor pass then read again.
That pass is not an independent review:
it is a second reader of the same vendor,
and its file is
`~/temp/agent/gate-review-zfs-root-modes-second-revision-2026-11-24/keep/read-of-the-second-one-program-revision.md`.
It found one High,
seven Medium and thirteen Low items,
and this revision answers every one of them in
"What this revision changed,
and why".
It also adds the two modes the owner's answers since need,
and re-poses every choice with a command,
a ranking and a separability statement.

Every claim in this design says what it rests on,
in one parenthesized label:

- (run FILE),
  a command this round ran,
  whose output is that file in this round's `keep/`;
- (replay FILE),
  an earlier round's kept transcript,
  replayed by this round with an identical result;
- (source PATH:LINE),
  a file in this repository,
  read on `main` at `f3d81e0910`;
- (upstream PATH:LINE),
  another program's source,
  read through `gh`;
- (record PATH),
  an accepted decision record;
- (design),
  this proposal's own reasoning,
  with no probe or record behind it.

## What this revision changed, and why

### The High finding, and the guard's bypasses

The read's H1 was that the one-name guard did not hold the names module to one name:
it pinned one `history_event-` literal line and three counts,
and its four variants (a second constant,
an inline second name,
and a suffix test and a contains test built from the constant)
all passed it while changing what the module decides.
The guard now pins the whole comparison as one exact line,
pins the module's complete set of string literals,
and forbids `ends_with(`,
`starts_with(`,
`contains(` and `rsplit` there.
All four variants fail the new guard,
and this round adds four of its own that keep the exact comparison and reach for each
forbidden spelling instead,
so each of the four bans has a witness that fails on it alone
(run keep/guard-replay-revision3.log).

The read's M1 was that the printed hook guard was not the predicate the replay checked:
`named_times` and the `root_mode_start(` count were in the replay and not in the printed text,
and three bypasses passed everything
(an alias of `args_os` through `use ... as`,
a read of `/proc/self/cmdline`,
and `std::env::args().take(1)`).
The printed guard now carries every predicate the replay applies,
compares the three vector lines whole instead of by substring,
and adds `/proc/self/cmdline` to its needle list.
Two typed bans do what a text guard cannot:
`package/git-policy/cli/clippy.toml` gains rows for `std::env::args` and `std::env::args_os`,
with one allow on `main`'s collection and three in the test files that start the test binary.
The alias and the `args()` call fail the ban,
measured in a probe crate that carries those two rows and the crate's own file
(run keep/clippy-bans.log).
All six of the read's own bypasses now fail the text guard,
and two of them fail the typed ban independently
(run keep/guard-replay-revision3.log).

### The seven Medium findings

- M2,
  "every start of the file is a root-mode start" is false for another `argv[0]`:
  it is now "every start under that name",
  and the probe carries the link and `exec -a` cases.
  This round ran the compiled module under links named `git`,
  `monochromatic-git-policy-cli` and `mount-git-worktree-clone-datasets`,
  and under `exec -a git`:
  each decides the wrapper,
  while the same file under its own name refuses a bare word
  (run keep/names-starts.log).
- M3,
  the check order:
  the words are read before the one name,
  and the service's words after both,
  and that order decides a real case.
  A unit's line at the installed path works only because the words are read first:
  name-first would refuse its words.
  The false "order cannot change what any of them does" sentence is gone
  (cases 3 and 28;
  run:
  keep/dispatch-cases.log).
- M4,
  the change list misses companions:
  it now names
  `no_self_invocation_tests.rs:191`'s `samples` array and its `:208` length assertion,
  `worktree_clone_mount_handler/mod.rs`'s `pub mod` line and its top doc,
  `source_guard/mod.rs`'s declaration of the new guard,
  `code_lines`' move from `hook_self_invocation_tests.rs` to `source_guard/mod.rs`,
  the three test files that need a Clippy allow,
  and the third pinned line of `main`.
- M5,
  citations that do not hold:
  every line number in this design is re-derived with `rg` on `f3d81e0910`,
  and each moved one is given its current value.
  `SPEC.md`'s sentence is at `:19261`,
  not `:19101`,
  and the read's own `:19235` was that read's main (`2d9fa02ea5`);
  the askpass change moved it to `:19248`,
  and `30c3037046` to `:19261`.
  The second `[[bin]]` is `:46` to `:48` and its comment is `:39` to `:45`;
  `:35` to `:37` are the wrapper's own table,
  which a literal delete of `:34` to `:45` would remove.
  `add_zfs.rs:68` is reached from `add_preflight.rs:113`.
  The README's five hits are `:11`,
  `:1912`,
  `:7188`,
  `:7195` and `:8038`.
  `lib.rs`'s "second executable" paragraph is `:415` to `:428`.
  The replay excerpt is regenerated from this round's run.
- M6,
  choice 1's A-over-B reason:
  `doctor`'s installed-file check does not read `--version` today
  (`handler_checks.rs:229` reads owner and mode only,
  and `rg -- '--version|version_line' src/native/health/` finds none),
  so the reason is replaced.
  What supports A is that the release smoke and a person can check the installed bytes with one command
  that carries no words,
  which is what makes the smoke's first arm a proof of the name dispatch.
  A also gives the name two more spellings,
  `--help` and `--version`,
  and the answer's literal text is B.
  The proposal discloses that,
  and the choice stays the owner's.
- M7,
  the cold-start section:
  the "lower bound" and "estimated sum" sentences are gone,
  because the two arms run different programs and neither bounds the proposed start;
  no absolute time is reported as a finding,
  only paired differences and ratios beside a control;
  and the warm per-event start,
  which the second revision left unmeasured,
  is measured this round.
  Both warm starts are under a millisecond and their difference is smaller than the control's
  own spread,
  so this round reports that it cannot tell them apart
  (this round's warm logs;
  run:
  keep/host-warm-*.txt and keep/warm-analysis.txt).

### The thirteen Low findings

- Choice 3's option D example is measured now:
  a copy of today's bytes named `holder-scan`,
  started with `--dataset tank/checkout`,
  answers `unknown option: --dataset` with Git's usage and status 129
  (run keep/today-installed-name.log).
- Choice 2's printed usage is the build's real text,
  copied from a run of the built program,
  with the two new modes' lines added where they will stand
  (run keep/handler-usage.log).
  The `decides` lines say which probe covers them.
- The edit counts are this round's own:
  seven edits of `main` (V3 to V9),
  six more (hv1 to hv6),
  eleven names modules (the three planted,
  the read's four,
  and four of this round's),
  listed in the replay log.
- `mount-git-worktree-clone-datasets` is 33 characters.
- The version line's five readers are named once,
  in choice 5,
  and nowhere else.
- The modes' list is stated as the records have it:
  the one-program record names four modes and the sweep,
  the grammar has `holder-scan` beside them,
  and this design adds two more.
- The occurrence count is this round's:
  122 lines in 54 files without this proposal and 148 in 55 with it.
- No relative pointer survives:
  each one names the heading,
  the path or the symbol.
- The `LiveGate` account says "not yet merged into the line that review read",
  and the commit dates are verified:
  `b01af8c3bd` (2026-11-23T20:54:55-05:00) is not an ancestor of `c2701d58a5`
  (2026-11-23T21:45:28-05:00),
  and it reached `main` at `bee6531780` (2026-11-24T00:51:32-05:00)
  (source)
  from the three commits' `git log` lines.
- Choice 4's option A block says which of its two lines is a prediction.
- Choices 1,
  4,
  6 and 7 now carry separability statements,
  and choice 2's list gains the modes' usage and refusal texts.
- B over C stops claiming a difference that B shares.
- The misuse message for `--version x` names the word that cannot be honoured,
  which is `x` and not `--version`,
  because the read measured that today's rule would print `--version`
  (the `--version extra` line;
  run:
  keep/names-starts.log).

### What the owner's answers since add

Two modes,
designed in "The two modes these answers add":

- `record-checkout --checkout <path>`,
  the root command that records a checkout,
  which is choice 1's option E of
  [the step 10 record](../decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md)
  (record doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md);
- `mount-path-descendant --dataset <dataset>`,
  which
  [the step 19 record](../decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md)
  names as the later replacement for the nested dataset's unit
  (record doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md).

The step 19 answer also fixes what this design says about AppArmor:
the ZED hook's needs are rules in `/etc/apparmor.d/local/zed`,
and the rule that matters here names the one root-owned file ZED starts,
because that file's name is choice 3's.
That is in "Steps 18 and 19".

## What the decision requires

The one-program record answers "One program only,
ever",
and its "What follows" is the requirement this design starts from
(record doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md):

- the release holds one executable,
  and the root modes
  (`boot-reconcile`,
  `shutdown-unmount`,
  `start-probe`,
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

The record's list is four modes and the sweep.
The grammar of the program this design folds in has five modes,
`holder-scan` beside the four
(source package/git-policy/cli/src/native/worktree_clone_mount_handler/handler_mode_arguments.rs:41),
and `holder-scan` comes along unchanged.
Two more modes are added by the answers since,
which is the count this design's change list carries:
seven modes and the sweep.

The start-name record's answer has two clauses,
and everything in choice 1 follows from the pair
(record doc/decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md):

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
It collects the arguments once,
skipping the program name (`main.rs:52`),
answers the service mode when the leading words are
`monochromatic-git-policy-cli service run` (`main.rs:56`,
`invocation::service_main::is_service_start`),
installs the payload-free panic hook,
collects the environment,
and hands everything to `invocation::entry::run_process` (`main.rs:72`),
then waits for the log lines it handed to their own threads
(`end_after_the_log`).
Every line number here is re-read on `f3d81e0910`
(source package/git-policy/cli/src/native/main.rs).

`package/git-policy/cli/src/native/mount_git_worktree_clone_datasets/main.rs` is the second program's entry,
deleted by this design:
`args_os().skip(1)` (`:49`),
`worktree_clone_mount_handler::run`,
the same two log waits,
and the exit.
Its grammar (`worktree_clone_mount_handler/handler_mode_arguments.rs:319`) takes
one leading word out of the five `MODE_WORDS` (`:41`),
`--help` and `--version`;
an empty argument list is `AttachingSweep`;
anything else is a refusal that names the word and the modes (`:223`).
`handler_mode_dispatch.rs:317` to `:338` gives every mode the same two answers,
a line and status 0 (`EXIT_DONE`,
`:40`),
or a refusal line and status 2 (`EXIT_REFUSED`,
`:45`),
and reads the compatibility gate before a mode that will make a request
(`run_behind_at` at `:317`).
`handler_install_paths.rs:28` holds the program's name,
`:32` and `:36` the two installed paths,
and `:40` the configuration path.

The service mode is the precedent this design follows,
and the answer says so:
a mode chosen inside `main`,
before the panic hook,
before the environment is collected,
and before anything of the wrapper's own start runs.
It differs from the words path in three ways this design must state,
because they are what a reader will compare:

- `service run` also answers the earlier command word `cli-git`
  (`service_main.rs:74`,
  `EARLIER_SERVICE_WORDS`);
  the root words answer only `monochromatic-git-policy-cli`,
  because no earlier release has a root-mode unit to keep working;
- `service` needs a third word,
  `run`;
  so does the words path need a mode word,
  which is why `zfs-root` alone refuses (choice 1);
- `service` has no no-argument form at all,
  while the installed name's no argument is ZED's own start.

### The release's lists, and what reads them

`package/git-policy/cli/Cargo.toml:35` to `:48` declares two `[[bin]]` tables,
and its `[package.metadata.binstall]` table (`:56` to `:58`) names `{ bin }` (`:58`),
so one archive unpacks both programs.
Every list that counts them must agree with the manifest,
and `bin/native-release-bins.unit.test.ts` is the guard that says so:
the release job's `cargo build --bin` flags (`.github/workflows/cargo-publish.yml:2666`)
and its `bin=` copy lines (`:2695` to `:2698`),
`package/crate-release/cli/releasable-crates.json:30`'s `checkBins` for the crate,
`toolchain.lock.json`'s `bins` row for `cargo:monochromatic-git-policy-cli`,
and `src/native/source_guard/release_bins_tests.rs:31`.
`toolchain.plan.jsonc` holds the same row as intent (`:271` to `:274`),
and `toolchain.lock.json` is written from the plan:
`mpm-shadow`'s update keeps every other row as it is and gives it "the plan's current bins",
so a changed plan row moves the lock row through the tool,
offline (`toolchain:lock:check` runs
`install --lockfile-only --frozen-lockfile`).
The crate's `pinFiles` row holds only the fuzz crate,
so the decision's "pinFiles row" has nothing left to undo
(source package/crate-release/cli/releasable-crates.json:29).

`bin/native-install-steps.ts:104` holds `handlerProgram` and `:121` `handlerVersionLine`.
`bin/native-install.ts:448` runs the handler by bare name through `PATH`
and `:458` refuses unless its `--version` line names the release.
`bin/stand-in-install-programs-for-tests.ts:169` to `:186` writes the stand-in handler those tests install,
and `:335` lists it beside the stand-in wrapper.

Six Rust test files start the second program by
`env!("CARGO_BIN_EXE_mount-git-worktree-clone-datasets")`,
at ten sites:

- `binary/binary_mount_git_worktree_clone_datasets_tests.rs:106`;
- `binary/binary_zfs_adoption_guest_tests.rs:118`;
- `binary/binary_zfs_holder_scan_guest_tests.rs:290` and `:892`;
- `binary/binary_zfs_step9_m_arms_tests.rs:200` and `:561`;
- `binary/binary_zfs_sweep_guest_tests.rs:230`,
  `:243` and `:875`;
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
`worktree_clone_add_route/add_zfs.rs:68` calls `LiveGate.verdict()` inside `OpenedDevice::open()`,
reached from `add_preflight.rs:113`,
both release code.
The call arrived in `b01af8c3bd` (2026-11-23T20:54:55-05:00),
which was not yet merged into the line the first gate review read
(`c2701d58a5`,
2026-11-23T21:45:28-05:00),
which is why that review found no gate and why its `rg` was right for its commit.
So a root mode inside the wrapper adds no dependency and no library:
the decision's "links no OpenZFS library and starts where OpenZFS is absent" check stays as it is,
and `worktree_clone_mount_handler` is already `#[cfg(target_os = "linux")]`
(source package/git-policy/cli/src/native/lib.rs:429).

### The sentence and the two guards

Three places say the wrapper never reads its start name,
and the answer changes all three for exactly one name:

- `SPEC.md:19261`,
  in "#### External hooks":
  "and the executable does not read the name it was started under".
- `src/native/source_guard/hook_self_invocation_tests.rs:74`,
  `main_does_not_dispatch_on_its_start_name`,
  which forbids `args_os().next()`,
  `start_name`,
  `current_exe`,
  `arg0`,
  `file_name`,
  `hook_dispatch` and `hook_entry` in `main.rs`,
  requires `args_os()` exactly once (`:80`),
  requires `args_os().skip(1)` (`:81`)
  and requires `entry::run_process(`.
- `src/native/source_guard/no_self_invocation_rules_tests.rs:26`,
  the `argv0-dispatch` scan rule,
  whose needles are `args_os().next()`,
  `args().next()`,
  `.nth(0)`,
  `argv[0]` and `arg0(`
  (`:29`).

Beside them,
`src/native/main.rs:53` explains the service mode as chosen
"never by the name this program was started as",
and `binary/binary_external_hook_tests.rs:174` proves that a start under each Git hook event's name behaves as `git`.
Both stay true for every name but the one,
which is what the guard change has to express.

The scan's `ALLOWED` rows are keyed by file and rule
(`no_self_invocation_rules_tests.rs`,
`unexplained` and `stale`),
so a row excuses every needle of its rule in its file.
That is why the argument-vector read needs a rule of its own (section "The guard change,
exactly").

### The wrapper's management help, which has no line for the words

`src/native/invocation/management_arguments.rs:46` holds `MANAGEMENT_USAGE` and `:59` `MANAGEMENT_HELP`,
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
because the root-mode arm does not exist yet
(run keep/today-lines.log):

```text
$ ./target/release/monochromatic-git-policy-cli monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic
Usage: git monochromatic-git-policy-cli check (--all | -- <pathspec>...) [--policy <id>]...
       git monochromatic-git-policy-cli fix (--all | -- <pathspec>...) [--policy <id>]...
       git monochromatic-git-policy-cli doctor
       git monochromatic-git-policy-cli service (status | settle-earlier-batches | stop-units-of-removed-worktrees)
       git monochromatic-git-policy-cli kept-references [--prune]
       git monochromatic-git-policy-cli migrate-to-subvolume [--dry-run | --rollback | --remove-old]
       git monochromatic-git-policy-cli convert-worktree-to-subvolume [--dry-run | --abandon] (--all | <worktree>...)
       git monochromatic-git-policy-cli --help
exit=2
```

So the change needs one line in each of those two constants,
and the tests that assert them
(`invocation/management_arguments_tests.rs`,
and `invocation/entry_command_word_tests.rs:108` and `:113`,
which compare the usage line)
move with it.
Without that line the words work and nothing teaches them.

## What a launcher hands a start

### ZED runs a zedlet by path, with no arguments

From OpenZFS `zfs-2.4.4`
(upstream cmd/zed/zed_exec.c):

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
with a fixture writing down what it was handed
(from `keep/zed_replay.c` and `keep/replay_fixture.rs`;
run:
keep/zed-shape-report.txt):

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
PROBE fd3=~round/scratch/probes/names/fx3/zed.d/zevent.nvlist
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

Measured this round on this host's user manager,
with a transient service and a transient scope
(run keep/unit-argv0.log):

```text
== a transient user service: ExecStart in a unit
PROBE argc=2
PROBE argv[0]=/home/user/temp/agent/zfs-root-modes-third-revision-2026-11-24/scratch/probes/argv0/argv_view
PROBE argv[1]=boot-reconcile

== a transient user scope: the command line is the unit's
PROBE argc=2
PROBE argv[0]=/home/user/temp/agent/zfs-root-modes-third-revision-2026-11-24/scratch/probes/argv0/argv_view
PROBE argv[1]=boot-reconcile
```

So a unit passes words,
and its `argv[0]` is the path as written,
which the one name rule reads.
A system manager's start is not measured here,
because that needs root;
the user manager and the system manager both read `ExecStart` with the same parser,
which is the one thing this round does not prove
(design).

### A caller can choose `argv[0]`, and not when ZED or systemd starts it

`exec -a` sets anything,
and ZED's `execle` and systemd's `ExecStart` fix the value.
A start whose `argv[0]` a lower-privileged caller chose as the zedlet's name is a root-mode start as that caller,
which is harmless for an account and is why none of this design's launchers passes a caller's choice through
(cases 26 and 34;
run:
keep/dispatch-cases.log).

### An empty `argv[0]` and a NULL vector are one empty argument

`execve` with an empty vector,
and `execve` with one empty element,
both reach the program as a single empty argument.
Measured on this kernel
(case 35;
run:
keep/dispatch-cases.log,
through `keep/exec_empty_argv.c` and `keep/argv_view.c`).

```text
    start: exec_empty_argv argc0 ./dispatch-probe   (execv with argv = {NULL})
PROBE decided=wrapper (the service check, then entry::run_process)
    start: exec_empty_argv empty0 ./dispatch-probe  (execv with argv = {"", NULL})
PROBE decided=wrapper (the service check, then entry::run_process)
```

The kernel inserts the empty element
(the `bprm->argc == 0` block of `fs/exec.c`,
upstream source),
so the design's rule decides both without indexing anything:
the last component of `""` is none,
which is not the zedlet's name,
so both are the wrapper.
ZED never does either:
its `execle` passes one element.

### The bytes must fit the launcher's libc

A root-owned file that a guest cannot start is no evidence,
so this design's measurements read the floor each program links against:
the wrapper needs at most `GLIBC_2.39` and the handler `GLIBC_2.34`
(`readelf --version-info`),
and the Ubuntu 24.04 guest the second revision measured in has 2.39,
so both host-built binaries start there
(replay keep/cold-start-analysis-rerun.txt).
That is what made a guest measurement possible at all in the second revision round,
where the first revision round's Fedora guest refused every route in.

## Choice 1: what selects a root mode

### The three rules probed

Each of the three rules carries the same answer to the second revision's Medium finding:
the words with no mode word after them refuse with the usage,
as `service` needs `run`,
so `zfs-root` alone never starts a sweep.
They differ in what a word does under the installed name.

Option A,
the rule this design recommends:

- the two words `monochromatic-git-policy-cli zfs-root`,
  followed by a mode word,
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

`keep/dispatch-cases.log` (run) is the three rules and nothing else,
from `keep/dispatch_probe.rs`,
run over thirty-five starts.
Cases 1 to 26 are the second revision's twenty-six and give its log's seventy-eight decisions,
line for line
(the two decision lists,
`keep/kept-first-78.txt` and `keep/mine-decided.txt`,
compared line for line,
replay).
The cases that decide the question:

- ZED's own child step,
  the bare name and no argument (case 1):
  A,
  B and C all decide the attaching sweep.
- the installed path,
  the words and `boot-reconcile`,
  which is a unit's `ExecStart` (case 3):
  all three decide `boot-reconcile`.
  Under A this works because the words are read before the name;
  a name-first reading of A's own refusal would refuse this line,
  which is what makes the order load-bearing.
- the installed path,
  a bare `boot-reconcile`,
  which step 19's earlier line and a person's guess use (case 4):
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
  C hands the grammar `monochromatic-git-policy-cli`,
  which the grammar refuses as no mode.
- the installed path,
  the mode word misspelled (case 9):
  all three hand the grammar `boot-reconcil`,
  which refuses it.
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
  and a contains test would have taken two of them and a suffix test one.
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
- the two new modes through the words (cases 27 and 28):
  all three decide them.
- the record mode written under the installed name (case 29):
  A and B refuse the word;
  C hands the grammar `record-checkout`.
- the installed name reached through links named `git`,
  `monochromatic-git-policy-cli` and `mount-git-worktree-clone-datasets` (cases 30 to 33):
  all three decide the wrapper,
  because none of those names is the zedlet's.
- a caller-chosen `argv[0]` that is not the name,
  with the words and a mode (case 34):
  all three decide the mode,
  which is the words clause at work on a file no name selects.
- an empty vector and a vector of one empty element (case 35):
  all three decide the wrapper.

The real kernel starts of the compiled module agree:
the installed file under a link named `git` decides the wrapper,
and the same file under its own name refuses a bare word
(run keep/names-starts.log).

### Ranking of choice 1

Ranking:
A > B > C.

A over B:
both fail closed for the installed file,
and both let the words select a mode on any file,
which is the answer's second clause.
A answers `--help` and `--version` under the installed name,
so the release smoke's first arm and a person's check of the installed bytes are one command with no
words in them,
and that is what proves the name dispatch before any sweep runs.
B is the answer's literal text,
and its price is one extra word in every such check;
its gain is a rule with one fewer arm,
which is a smaller rule,
not a stronger one.
A's cost,
disclosed,
is two more spellings under the name:
`--help` and `--version`.

B over C:
C accepts a third spelling the answer never names,
a bare mode word under the installed name,
and it drops the words pair there,
so under the recommended layout the words would not be what selects a mode at all
(the second revision's High finding).
C is also the only one of the three that decides a bare word under a stray copy of the zedlet's name (case 22),
where A and B refuse.

The choice is separable.
If the owner picks B,
what changes is four lines of `root_mode_names.rs` and the release smoke's first arm,
which then has to spell the words;
if the owner picks C,
the words pair is dropped under the name and the unit lines still work,
at the price of the property stated in "B over C".
Nothing about the layouts,
the dispatch's placement or the guards changes.

### The rule, in the entry

The whole argument vector goes to one module and one function,
so `main` reads no element and compares no name:

```rust
// package/git-policy/cli/src/native/main.rs, the import and the first statements of main
#[cfg(target_os = "linux")]
use git_policy_cli::worktree_clone_mount_handler::root_mode_names::{self, RootModeStart};
use std::ffi::OsString;

fn main() {
// The one argument-vector read the owner's answer needs; `clippy.toml` bans the call in every
// other file, and this allow is the one it names.
#[allow(clippy::disallowed_methods)]
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
}
```

`src/native/worktree_clone_mount_handler/root_mode_names.rs` is the one place the name,
the words and the grammar's argument list meet.
It holds three things:

- the constant `ZEDLET_START_NAME`,
  the installed ZED history event's own name;
- the constant `ROOT_GROUP_WORD`,
  choice 2's word;
- the function `root_mode_start`,
  which answers one of three things:
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

The module,
whole:

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
    return RootModeStart::Refused(RootModeMisuse::WordUnderTheZedletName(word_that_cannot_be_honoured(handed)));
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
a `root_mode_names_tests.rs` beside it covering the table in "What each decides",
and with `refuse_root_mode_misuse` in `handler_mode_dispatch.rs`,
where every other refusal is written:
it says the misuse and then the usage,
and answers `EXIT_REFUSED`.

The last line calls one helper the second revision did not have:

```rust
/// The first word this start cannot honour: `--help x` and `--version x` name `x`, because the
/// first word alone is accepted and the second is what the name does not allow.
fn word_that_cannot_be_honoured(handed: &[OsString]) -> &OsStr {
    let first: &OsStr = handed[0].as_os_str();
    if handed.len() > 1 && (first == OsStr::new("--help") || first == OsStr::new("--version")) {
        return handed[1].as_os_str();
    }
    return first;
}
```

and a refusal that names the whole start,
because a person who typed `--version x` reads why:

```text
$ /etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets --version x
`x` is an argument this start does not take: under this file's own name only no argument,
`--help` and `--version` are answered. Nothing was done. Ways forward: run the file with no argument for
the clone-mount sweep, with `--help` for the modes' usage, or with `--version` alone; every mode is reached
through the words, as `monochromatic-git-policy-cli zfs-root <mode>`.
```

The words of this refusal are this revision's,
and no build prints them yet (design).

The service check follows the block,
so the order of the three checks is:
the words,
the one name,
the service's words.
That order is load-bearing twice over:
the words are read before the name,
which is what lets a unit's line work at the installed path,
and both are read before the service's words,
which is what refuses `monochromatic-git-policy-cli service run` under the zedlet's name
(the `service words` line;
run:
keep/names-starts.log).

### `zfs-root` alone refuses with usage

The words path needs a mode word,
so `git monochromatic-git-policy-cli zfs-root` is a refusal that the usage follows,
not a sweep.
Measured today,
the same line reaches the management arm,
which answers its usage and status 2,
and writes a lock-holder record into the repository it stands in
(run keep/today-lines.log);
the second revision's `keep/probe-install-layouts.log` counted the record per start);
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
and its body becomes this,
which is the predicate the replay applies,
predicate for predicate:

```rust
// package/git-policy/cli/src/native/source_guard/hook_self_invocation_tests.rs
#[test]
fn main_reads_the_one_start_name_through_root_mode_names() {
    let code: Vec<&str> = code_lines(MAIN_SOURCE);
    let joined: String = code.join("\n");
    for forbidden in [
        "args_os().next()",
        "args().next()",
        "std::env::args(",
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
        "cmdline",
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
        "match root_mode_names::root_mode_start(arguments.as_slice()) {",
        "let handed: &[OsString] = arguments.get(1..).unwrap_or_default();",
    ];
    let with_the_vector: Vec<&str> = code.iter().copied().filter(|line| { return names_the_argument_vector(line) > 0; }).collect();
    assert_eq!(with_the_vector.len(), VECTOR_LINES.len(), "main.rs names the argument vector on {} line(s): {with_the_vector:?}", with_the_vector.len());
    for line in with_the_vector {
        assert!(VECTOR_LINES.contains(&line.trim()), "main.rs uses the argument vector in a line this guard does not name: {line}");
    }
    for known in VECTOR_LINES {
        let exact: usize = code.iter().filter(|line| { return line.trim() == known; }).count();
        assert_eq!(exact, 1, "main.rs holds {exact} line(s) equal to {known:?}");
    }
    assert_eq!(joined.matches("args_os").count(), 1, "main.rs names args_os once");
    assert_eq!(joined.matches("args_os()").count(), 1, "main.rs reads the argument vector once");
    assert_eq!(joined.matches("root_mode_start(").count(), 1, "main.rs makes one decision call");
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

`code_lines` moves from this file to `source_guard/mod.rs` as `pub(super)`,
because the names guard needs it too and a second copy would be a second predicate.

What this pins,
and what it does not:

- it pins the three lines that may name the argument vector,
  as whole lines,
  so a fourth read,
  a second decision call,
  a stuffed line,
  or a renamed vector fails on the count or on the equality,
  which is what each of the read's six variants now trips over;
- it pins every spelling of the zeroth element the repository's vocabulary has,
  forbids a name comparison's building blocks in `main` (`OsStr::new(`,
  `OsString::from(`,
  `starts_with(`,
  `ends_with(`,
  `contains(`),
  and forbids the other ways to reach the vector (`std::env::args(`,
  a second `args_os`,
  `cmdline`);
- it does not make a denylist complete against arbitrary code,
  and it does not claim to:
  the reviewed three lines are what `main` may do with the vector,
  and any change to them is a change to this guard.

The typed half is what the text cannot see.
`package/git-policy/cli/clippy.toml` gains two rows beside the `current_exe` ban:

```toml
  { path = "std::env::args", reason = "the program name this process was started under is one decision in one module (doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md); every other file that wants the arguments takes them as a parameter" },
  { path = "std::env::args_os", reason = "the program name this process was started under is one decision in one module (doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md); main's one collection is the allow this row names" },
```

and the allows are four:
the one collection in `main`,
and one in each test file that starts the test binary,
which the crate's other `current_exe` allows are the precedent for:

- `src/native/binary/binary_supervisor_tests.rs:755`;
- `src/native/push/child_watch_tests.rs:1000`;
- `src/native/landing/service_durable_tests.rs:241`.

(run keep/clippy-bans.log):
a probe crate carrying those rows and the crate's own file refuses a call through a
`use std::env::args_os as ...` alias,
refuses `std::env::args()`,
and accepts the one statement that carries the allow.)

#### The scan rule's second id

`no_self_invocation_rules_tests.rs` keeps its `argv0-dispatch` rule,
so the four old needles stay unexcused in every file,
gains `/proc/self/cmdline` as a needle there,
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

`/proc/self/cmdline` goes in `argv0-dispatch` rather than here,
because that rule has no allows:
a file that reads the vector through the kernel's copy is a start-name read,
and `main` may not do it.

The reasons this is the right shape:

- the row is keyed by file and by rule,
  so it excuses `args_os()` in `main.rs` and nothing else there,
  while `args_os().next()`,
  `args().next()`,
  `.nth(0)`,
  `argv[0]`,
  `arg0(`
  and `/proc/self/cmdline` stay unexcused in `main.rs` and everywhere else;
- a release file that wants the vector now needs its own row and its own reason,
  which is a diff a reviewer reads;
- `every_justified_exception_still_matches_code` keeps both rows honest.
  Today's `main.rs` is the only release file that reads `args_os()`,
  and the second program's `main.rs`,
  which this change deletes,
  is the other.

`no_self_invocation_tests.rs:191`'s `samples` array gains a seventh pair,
`("argv-vector-read", "let arguments = std::env::args_os().collect::<Vec<_>>();")`,
and `:208`'s `assert_eq!(RULES.len(), samples.len())` carries it without another edit.

#### The one-name guard on the names module

A new sidecar beside the other guards,
`source_guard/root_mode_name_tests.rs`,
declared in `source_guard/mod.rs`,
reads the module and holds it to one name and one comparison:

```rust
// package/git-policy/cli/src/native/source_guard/root_mode_name_tests.rs
/// The one module allowed to read the name this program was started under.
const NAMES_SOURCE: &str = include_str!("../worktree_clone_mount_handler/root_mode_names.rs");

/// The whole comparison: the start name's last component, against the one constant.
const COMPARISON: &str = "return start_name.and_then(last_component) == Some(ZEDLET_START_NAME);";

/// Every string literal the module may hold.
const LITERALS: [&str; 4] = [
    "history_event-mount_git_worktree_clone_datasets",
    "zfs-root",
    "--help",
    "--version",
];

#[test]
fn the_one_start_name_is_written_in_one_place() {
    let code: Vec<&str> = code_lines(NAMES_SOURCE);
    let joined: String = code.join("\n");
    let comparisons: usize = code.iter().filter(|line| { return line.trim() == COMPARISON; }).count();
    assert_eq!(comparisons, 1, "the whole-component comparison is written on {comparisons} line(s), and it is written once, as {COMPARISON:?}");
    for forbidden in ["ends_with(", "starts_with(", "contains(", "rsplit"] {
        assert!(!joined.contains(forbidden), "root_mode_names.rs contains {forbidden}");
    }
    for forbidden in ["[\"", "[&str", ".any(", "is_one_of", "matches!(", "format!", "concat!", "push_str", "to_ascii_", "to_lowercase", "r#", "\\\\"] {
        assert!(!joined.contains(forbidden), "root_mode_names.rs contains {forbidden}");
    }
    let mut literals: Vec<String> = string_literals(joined.as_str());
    literals.sort();
    literals.dedup();
    let mut allowed: Vec<String> = LITERALS.iter().map(|literal| { return String::from(*literal); }).collect();
    allowed.sort();
    assert_eq!(literals, allowed, "root_mode_names.rs holds the string literals {literals:?}, and the module may hold only {LITERALS:?}");
    assert_eq!(joined.matches("ZEDLET_START_NAME").count(), 2, "the constant is its declaration and one comparison");
    assert_eq!(joined.matches("file_name()").count(), 1, "the whole last path component is read once");
    assert_eq!(joined.matches("split_first()").count(), 1, "the program name is separated once");
}
```

with the literal reader beside it,
which is a tripwire rather than a parser,
and is why the module may hold no escape and no raw string:

```rust
/// Every double-quoted string in the code lines, in order.
fn string_literals(joined: &str) -> Vec<String> {
    let mut literals: Vec<String> = Vec::new();
    let mut started: Option<usize> = None;
    for (index, character) in joined.char_indices() {
        if character != '"' {
            continue;
        }
        match started {
            None => started = Some(index),
            Some(from) => {
                literals.push(String::from(&joined[from + 1..index]));
                started = None;
            }
        }
    }
    return literals;
}
```

The literal set is compared as a set,
not as a count,
so the two help words repeated in the extra-word helper are not a second name,
while any spelling outside the four fails.
The planted and variant modules show each rule firing on its own:
a second literal,
an array of names,
a name assembled with `concat!`,
a second constant,
an inline second name,
a suffix test and a contains test built from the constant,
and four of this round's that keep the exact comparison and reach for `ends_with(`,
`starts_with(`,
`contains(` and `rsplit` apart from it
(run keep/guard-replay-revision3.log).

#### Replayed

(run keep/guard-replay-revision3.log),
from `keep/replay_revision3.rs`,
which is the three guard bodies of "The hook guard on `main.rs`",
"The scan rule's second id" and
"The one-name guard on the names module",
with one change each so a driver collects a verdict:
`assert!` becomes a recorded refusal.
Today's `main.rs` is the positive control for the guard that stands;
it passes the current guard and fails the new one on four vector lines.

```text
== today's main.rs (the positive control that must fail the new guard)
FAIL  main: .../src/native/main.rs : main.rs names the argument vector on 4 line(s)
      scan: 0 finding(s) []

== the proposed main.rs (the positive control that must pass)
PASS  main: proposed_main.rs
      scan: 1 finding(s) [("argv-vector-read", 39)]

== the second pass's V3 to V9, each must fail
FAIL  main: v3_second_name_compared_in_main.rs : main.rs contains "OsStr::new("
FAIL  main: v4_the_name_passed_to_a_second_function.rs : main.rs contains ".first("
FAIL  main: v5_arguments_zero_suffix_test.rs : main.rs contains "arguments[0]"
FAIL  main: v6_first_element_bytes_compared.rs : main.rs contains ".first("
FAIL  main: v7_std_env_args_next_read.rs : main.rs contains "args().next()"
FAIL  main: v8_nth_zero_read.rs : main.rs contains ".nth(0)"
FAIL  main: v9_a_second_decision_call.rs : main.rs names the argument vector on 4 line(s)

== the read's owned bypasses (hv1 to hv6), each must fail
FAIL  main: hv1_same_line_stuffing.rs : main.rs uses the argument vector in a line this guard does not name: "let handed: &[OsString] = arguments.get(1..).unwrap_or_default(); let _name = arguments.iter().next();"
FAIL  main: hv2_alias_of_args_os.rs : main.rs names args_os 2 time(s)
FAIL  main: hv3_proc_cmdline.rs : main.rs contains "cmdline"
FAIL  main: hv4_iter_next_own_line.rs : main.rs names the argument vector on 4 line(s)
FAIL  main: hv5_env_args_take.rs : main.rs contains "std::env::args("
FAIL  main: hv6_second_names_fn.rs : main.rs names root_mode_names 3 time(s): its import and one call

== the proposed names module, which must pass
PASS  names: root_mode_names.rs

== the proposed names module with this revision's extra-word refusal, which must also pass
PASS  names: proposed_root_mode_names.rs

== the three planted names modules, each for a rule of its own
FAIL  names: planted_array_of_names.rs : the whole-component comparison is written on 0 line(s)
FAIL  names: planted_derived_name.rs : root_mode_names.rs contains "concat!"
FAIL  names: planted_two_literals.rs : root_mode_names.rs holds the string literals ["--help", "--version", "history_event-mount_git_worktree_clone_datasets", "history_event-other", "zfs-root"]

== the read's four names-module variants (mv1 to mv4), each must fail
FAIL  names: mv1_second_name_other_prefix.rs : the whole-component comparison is written on 0 line(s)
FAIL  names: mv2_suffix_test_with_the_constant.rs : the whole-component comparison is written on 0 line(s)
FAIL  names: mv3_inline_second_literal.rs : the whole-component comparison is written on 0 line(s)
FAIL  names: mv4_contains_test_with_the_constant.rs : the whole-component comparison is written on 0 line(s)

== this round's own witnesses for the four forbidden comparison spellings
FAIL  names: w1_suffix_test_beside_the_comparison.rs : root_mode_names.rs contains "ends_with("
FAIL  names: w2_split_on_rsplit.rs : root_mode_names.rs contains "rsplit"
FAIL  names: w3_prefix_test_beside_the_comparison.rs : root_mode_names.rs contains "starts_with("
FAIL  names: w4_contains_test_beside_the_comparison.rs : root_mode_names.rs contains "contains("
```

So each of the second pass's six bypasses fails,
each of its four module variants fails,
and every rule has a witness that fails on it alone.
The full log has every line and every scan finding.

The second revision's own replay still passes its text and still fails today's file,
which is how this round knows the harness reads the guards and not the file names
(run keep/revision2-harness-over-every-text.log).

`SPEC.md:19261` becomes:

> Git never starts the wrapper for a hook:
> the wrapper writes no hook entry,
> passes no hooks path of a directory it made,
> and reads the name it was started under in one module only,
> the installed ZED history event's,
> where that name selects the clone-mount sweep and nothing else a Git command means.

`binary_external_hook_tests.rs:174` keeps proving that every Git hook event's name still behaves as `git`,
which is now a claim about every name but one.

## Choice 2: the group word units, adoption and a person spell

The answer fixes the shape of the words,
because `service run` is the precedent:
the wrapper's own word,
then a namespace word,
then the modes' grammar.
Which namespace word it is,
is this choice's,
and a unit's `ExecStart` line,
an adoption's command and a person's command all read it.

What the modes print does not depend on the word,
except in the last paragraph of the usage,
so it is printed once here.
The text is the build's,
copied from a run of the built handler,
with the two new modes' lines where they will stand
(run keep/handler-usage.log):

```text
usage: mount-git-worktree-clone-datasets [mode]
  (no argument)        run the clone-mount sweep the installed ZED event starts
  boot-reconcile       reconcile clone mounts after boot; the boot unit's ExecStart
  start-probe          run the start probe's arms and write this boot's result
  shutdown-unmount     unmount the verified clone mounts at shutdown; the boot unit's ExecStopPost
  holder-scan --dataset <name> [--nonce <32 hex digits>]
                       report which mount namespaces hold that clone, and every read the scan could not complete
  adopt-worktree-parent --checkout <path> --device <count> --inode <count>
                       record an administrator-identified replacement .claude/worktrees folder
  record-checkout --checkout <path>
                       record this checkout's path, dataset, GUID, container and pool for the route
  mount-path-descendant --dataset <name>
                       mount one nested dataset at its own mountpoint, under the recorded checkout
  --help               print this text
  --version            print this build's version
The same build answers these modes through the words monochromatic-git-policy-cli zfs-root <mode>,
is installed as <the root-owned file choice 3 names>, and answers ZED by the name
<the installed ZED history event>.
The modes that make requests open /dev/zfs through this repository's binding, never a CDDL library and never the zfs program.
```

The two added lines are this design's (design),
and no build prints them yet,
and the last paragraph is the change that makes the words discoverable
(choice 5 keeps the program's name in the first line and in every refusal).
Everything else is the build's own text,
which the second revision's version of this block abbreviated;
the second line of `holder-scan` and the two closing paragraphs are quoted here because they exist.

Every option in this choice is shown with the start it produces.
The words are not in the build yet,
so the `decides` line is the fixture's
(cases 3,
27 and 28;
run:
keep/dispatch-cases.log),
and the `prints` line is what the modes print today
(run keep/handler-usage.log).

### Option A: `zfs-root`

```text
$ git monochromatic-git-policy-cli zfs-root --help
   decides: root-mode grammar="--help"
   prints:  the usage printed at the head of this choice, with the words and the root-owned file
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
- one word for seven modes,
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
  while `start-probe`,
  `holder-scan` and `record-checkout` are not mounts;
- `mount` is a word any later mode family would also want,
  so it claims a broad name for a narrow set.

### Option C: `root`

```text
$ git monochromatic-git-policy-cli root boot-reconcile
   decides: root-mode grammar="boot-reconcile"
```

Pros:

- shortest,
  and honest about the privilege the modes need.

Cons:

- reads as a Git concept (a root commit,
  the root tree),
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

- four of the seven modes are clone mounts,
  while `start-probe`,
  `holder-scan` and `record-checkout` are not,
  which is the same objection as option B's,
  narrower;
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
D names the work and its noun covers four of the seven modes,
while A's word covers the modes and the probes by naming where they run.
D's noun is also the one the record mode's new reader does not use:
the checkout record is not a mount,
so the word A covers it and D does not.
The picks are a naming taste,
and this reason is the delivery that separates them (design).

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

- `ROOT_GROUP_WORD` in `root_mode_names.rs`,
  one string literal in the one-name guard's `LITERALS`,
  and the fixture's cases;
- the unit text in plan step 19,
  the nested dataset's unit,
  and the four documents that carry unit lines;
- the wrapper's management help line in `MANAGEMENT_USAGE` and `MANAGEMENT_HELP`;
- the modes' usage's last paragraph,
  and every refusal that names the words:
  the group-word refusal (`refuse_root_mode_misuse`),
  the grammar's unknown-mode message,
  the usage's trailing sentence,
  and `doctor`'s printed adoption line;
- the release smoke's second arm and `native:install`'s words check;
- `SPEC.md`'s sentence about the words;
- the guest commands in `e2e/start-probe-guest.ts`.

Nothing about the name,
the layouts or the guards changes.

## The two modes these answers add

Two accepted answers need modes this design did not have when the second revision was written.
Both are modes of the fold this design makes,
so their words,
arguments and refusals are this design's to state
(record doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md
and doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md).

### What the grammar promises, and what these two keep

The grammar's principle,
as the build keeps it
(source package/git-policy/cli/src/native/worktree_clone_mount_handler/handler_mode_arguments.rs:319):

- one leading word names the work,
  and the words are a closed list whose refusal names every member;
- every input is `--name <value>`,
  in the order the help prints,
  and a missing value,
  a repeated option or an unreadable argument refuses rather than guesses;
- an empty argument list is the attaching sweep,
  which is ZED's own start and nothing else;
- a mode that did its work answers 0,
  and every refusal answers 2 (`EXIT_REFUSED`),
  after a line saying what was not done and every way forward.

Both new modes meet that principle.
They also meet a rule the old modes did not have to:
each one reads or writes one root-owned record,
and the second reads what the first wrote.

### `record-checkout --checkout <path>`

```text
$ /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets \
    monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic
recorded: /home/user/Monochromatic is zpcachyos/nosnap/home-user-Monochromatic (GUID 1234567890123456789),
in pool zpcachyos, beside container zpcachyos/nosnap/clones; the record is <the record's path>
```

The line's words are this revision's (design).
`keep/names-starts.log` shows the compiled module hands the grammar exactly
`["record-checkout", "--checkout", "/home/user/Monochromatic"]` through the words (run),
and `keep/dispatch-cases.log` cases 27 and 29 show the same for both files.

What it records,
which is what the step 10 answer names:

- the checkout's path;
- its dataset's name and GUID,
  read from the kernel's table and from the dataset's own properties;
- the container dataset the configuration names,
  or the one beside the checkout when the configuration leaves it out;
- the pool,
  which is the dataset name's first component.

What it proves before it writes:

- `--checkout` is text and an absolute path,
  or the grammar's own refusals answer;
- the path is a mount point of a ZFS dataset,
  and that dataset's `mountpoint` property is the path itself,
  which is the precondition step 10's design states
  (`doc/planning/cli-git-zfs-step-10-open-choices.md`,
  "The checkout path must be its dataset's own mount point");
- the container is a dataset in the same pool,
  and the checkout's dataset is not inside the container (the route places the container beside it);
- the caller is root,
  through the same refusal `start-probe` and the adoption answer a caller with
  (`handler_mode_dispatch.rs:174`,
  `adopt_worktree_parent_mode.rs:315`);
- the record's folder exists,
  is root's,
  and is not group- or other-writable;
- the compatibility gate answers before the first `/dev/zfs` request (`run_behind_at`,
  `handler_mode_dispatch.rs:317`).

Its refusals,
each naming what was not done and the way forward:

- `checkout-not-a-mount-point`,
  because the add's gate would compare the record against a folder rather than a mount;
- `checkout-not-on-zfs`,
  because the route serves ZFS and a record for anything else would make the gate pass on a checkout it cannot clone;
- `container-not-beside-the-checkout`,
  because a container inside the checkout is the shape the route's layout excludes;
- `checkout-record-differs`,
  when the record already exists and a proved field differs,
  because two checkouts on one machine is the bound the readiness record still has;
- `checkout-record-unwritable`,
  when the folder or the file cannot be root's.

A second run over the same checkout replaces the record only when every field is equal,
so a re-record after a rename refuses and names the field,
and a person learns that the checkout is not the one the record serves.

The record is written by this mode and by nothing else.
Its path is one more constant in `handler_install_paths.rs`,
proposed as `/var/lib/monochromatic-git-policy-cli/git-worktree-clone-checkout.json`,
which is the root-owned store folder the slot identities already live in
(source package/git-policy/cli/src/native/worktree_clone_mount/slot_identity_store.rs:44).
Its format is the reader's:
the add's gate,
the sweep and the probe change in step 10's build,
not in this design's commit,
and that build owns the field names and the version.
This design fixes one property and one only:
the mode is the writer,
and every reader refuses a record whose proved fields no longer match the kernel's.

### `mount-path-descendant --dataset <name>`

```text
$ /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets \
    monochromatic-git-policy-cli zfs-root mount-path-descendant --dataset zpcachyos/nosnap/vm-builder-output
mounted: zpcachyos/nosnap/vm-builder-output at /home/user/Monochromatic/package/vm-builder/cli/output
```

```text
$ … mount-path-descendant --dataset zpcachyos/nosnap/vm-builder-output
already mounted: zpcachyos/nosnap/vm-builder-output at /home/user/Monochromatic/package/vm-builder/cli/output
```

No build prints either line (design).
The dataset and its mountpoint are the ones the step 19 record names
(record doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md).

What it does:

- reads the checkout's record,
  and refuses when there is none;
- reads the dataset's `mountpoint` and `mounted` properties;
- mounts the dataset at its own mountpoint when it is not mounted,
  and answers 0 with the `already mounted` line when it is,
  which is what makes an `ExecStart=` re-run harmless;
- changes no other dataset,
  mounts nothing under a dataset it was not given,
  and unmounts nothing (that is `shutdown-unmount`'s work).

What it refuses:

- `dataset-outside-the-checkout`,
  which is the refusal the step 19 design names:
  the dataset is not a descendant of the recorded checkout dataset,
  or its `mountpoint` is not under the recorded checkout path;
- `dataset-without-a-mountpoint`,
  when the property is `none`,
  `-` or `legacy`,
  because such a dataset is mounted by someone else's rule and this mode would be guessing;
- `no-recorded-checkout`,
  when the record is absent,
  because without it the mode cannot prove the mountpoint descends from a served checkout;
- `dataset-already-mounted-elsewhere`,
  when the kernel shows the dataset mounted at a path other than the one its property names.

It runs behind the gate and as root,
like every mode that writes.

### The question the step 19 design leaves to this one

That design's "What each part waits on" says the mode needs this design to say
"whether a mode may change a dataset's state by name at all"
(`doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`,
What each part waits on).

The answer is yes,
bounded by the record:

- a mode may mount one dataset by name when the name is a descendant of the recorded checkout dataset,
  the dataset's own `mountpoint` is under the recorded checkout path,
  and the dataset is not the checkout itself;
- a mode may not unmount,
  destroy,
  snapshot or change a property of a dataset by name,
  and nothing in this design's two modes does;
- the record is the only thing that makes "descendant of the checkout" a proof rather than a guess,
  which is why this mode refuses before `record-checkout` has run.

### The conflicts the pair carries

- **`--checkout` on two modes.**
  `adopt-worktree-parent --checkout <path>` records the replacement `.claude/worktrees` folder's identity;
  `record-checkout --checkout <path>` records the checkout's route facts.
  Both name the same path in practice,
  and the mode word is what says which proof is made.
  The usage line of each names its own record,
  and `doctor` prints the adoption's line for an adoption and the record's line for the record,
  so a person copying the wrong one reads a line about the other job.
- **`--dataset` on two modes.**
  `holder-scan --dataset <name>` reports which mount namespaces hold a clone;
  `mount-path-descendant --dataset <name>` mounts one nested dataset.
  Both take a full dataset name and both refuse the same malformed names through
  `ArgumentRefusal::DatasetName`,
  and each usage line says what it does with the name.
- **Two sources for the checkout.**
  The configuration file can still name `cloneMounts.checkout` today
  (source package/git-policy/cli/src/native/worktree_clone_mount_handler/sweep_configuration.rs).
  Once the record exists,
  the record is what the route serves,
  and a configuration value that disagrees is a refusal naming both,
  not a silent choice.
  That reader change is step 10's build's,
  with the format.
- **A mode that needs another mode's output.**
  `mount-path-descendant` refuses until `record-checkout` has run.
  That is a new ordering between modes,
  and it is stated in both usage lines and in `doctor`'s output,
  so a boot that runs the nested dataset's unit before the record exists reads `no-recorded-checkout`
  rather than mounting a dataset it cannot place.

### Ranking of the record mode's word

The word is the one thing here the answers do not fix,
so it is posed like choice 2.
The three candidates are the same command under a different name.

Option A,
`record-checkout`:

```text
$ git monochromatic-git-policy-cli zfs-root record-checkout --checkout /home/user/Monochromatic
   decides: root-mode grammar="record-checkout --checkout /home/user/Monochromatic"
```

Option B,
`record-route`:

```text
$ git monochromatic-git-policy-cli zfs-root record-route --checkout /home/user/Monochromatic
   decides: root-mode grammar="record-route --checkout /home/user/Monochromatic"
```

Option C,
`checkout-record`:

```text
$ git monochromatic-git-policy-cli zfs-root checkout-record --checkout /home/user/Monochromatic
   decides: root-mode grammar="checkout-record --checkout /home/user/Monochromatic"
```

Ranking:
A > B > C.

A over B:
A names the thing that is recorded,
which is what the add,
the sweep and `mount-path-descendant` read,
and B names the route,
which the group word `zfs-root` already said.
What A delivers is a mode whose word a person can match to the record file's name (design).

B over C:
both are two words about the route or the record,
and B reads as a verb and an object,
while C reads as a noun phrase,
which the grammar's other words never are (`boot-reconcile`,
`start-probe`,
`holder-scan`,
`adopt-worktree-parent`).
The difference C costs is the habit a reader already has (design).

The choice is separable:
the word is one member of `MODE_WORDS`,
one line of the usage and one of the help,
one match arm and its refusal texts,
the step 10 build's reader,
and `doctor`'s printed line.
Nothing about the record's shape or the other modes changes.

## Choice 3: what root's install holds

The review asked that root's regular file carry a mode's name,
not the wrapper's,
so that a root start under any other name fails with `No such file`.
The answer says units and adoption reach the modes by words.
Both hold at once,
and the four layouts are what fits them together.

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

The name rule reads `argv[0]`,
not the file,
so what a layout buys is stated per start name and never per file.
The evidence is the compiled module under real starts:
the installed-name file,
reached through links named `git`,
`monochromatic-git-policy-cli` and `mount-git-worktree-clone-datasets`,
decides the wrapper in every case,
and the same file under its own name refuses a bare word
(run keep/names-starts.log).
A copy,
a link or an `exec -a` with another name is outside every name rule's reach,
and that is true of all four layouts.

Each layout was run as the account in a throwaway `git init` repository,
with today's wrapper's bytes as the installed file
(`keep/today-installed-name.log`,
run),
because the dispatch itself is choice 1's and this choice is about names on disk.
Today's bytes carry no dispatch,
so each option's block shows what a start does now,
what the layout's name makes it do under this revision's rule,
and what it writes into the repository it stands in.

### Option A: one regular file under the ZED hook's own name

```text
/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets   regular, 0755, root
/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets -> ../../../usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets
```

```text
$ /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets
    today:   git's usage, exit=1; a marker in the repository's `.git/cli-git-lock-holders/`
    after:   the attaching sweep
$ .../history_event-mount_git_worktree_clone_datasets boot-reconcile
    today:   git: 'boot-reconcile' is not a git command, exit=1; the same marker
    after:   a refusal with the usage, exit=2, no mode
$ .../history_event-mount_git_worktree_clone_datasets monochromatic-git-policy-cli zfs-root boot-reconcile
    today:   the management usage, exit=2; the same marker
    after:   boot-reconcile
$ .../history_event-mount_git_worktree_clone_datasets --version
    today:   git version 2.56.1 and the wrapper's banner, exit=0; the same marker
    after:   mount-git-worktree-clone-datasets 0.6.1-alpha.45, exit=0
$ .../history_event-mount_git_worktree_clone_dataset --version
    today:   No such file or directory, exit=127
$ .../mount-git-worktree-clone-datasets boot-reconcile
    today:   No such file or directory, exit=127
```

Pros:

- every start **under that name** is a root-mode start,
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
- `SERVICE_INSTALL_PATH`,
  whose name means "where service calls start it",
  is renamed (to `ROOT_MODE_INSTALL_PATH`),
  because nothing about it is a service call any more.

### Option B: one regular file named for the modes

```text
/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets   regular, 0755, root
/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets -> ...        symlink
```

```text
$ /usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets boot-reconcile
    today and after: git: 'boot-reconcile' is not a git command, exit=1; a marker
$ /usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets
    today and after: git's usage, exit=1; a marker
$ /usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets monochromatic-git-policy-cli zfs-root boot-reconcile
    after: boot-reconcile
```

Pros:

- the file's name,
  the version line and every refusal's program name agree;
- a unit's path reads for what it does.

Cons:

- a bare mode word at that path,
  which is exactly step 19's earlier line as the plan carries it,
  runs the whole wrapper as root:
  Git,
  its configuration,
  its lock pass,
  and a marker in whatever repository the caller stands in.
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
    today and after: git: 'boot-reconcile' is not a git command, exit=1; a marker
$ /usr/libexec/monochromatic-git-policy-cli/monochromatic-git-policy-cli
    today and after: git's usage, exit=1; a marker
$ /usr/libexec/monochromatic-git-policy-cli/monochromatic-git-policy-cli --version
    today and after: git version 2.56.1 and the wrapper's banner, exit=0; a marker
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
$ cp <the wrapper> /usr/libexec/monochromatic-git-policy-cli/holder-scan
$ /usr/libexec/monochromatic-git-policy-cli/holder-scan --dataset tank/checkout
    today and after: unknown option: --dataset, then git's usage, exit=129, because that name selects nothing
$ /usr/libexec/monochromatic-git-policy-cli/holder-scan
    today and after: git's usage, exit=1
```

This round measured it (`keep/holder-scan-name.log`,
run):
Git's unknown-option branch prints the line and calls `usage()`,
which exits 129,
and the no-argument form exits 1.
The second revision's version of this block gave exit 1,
and was not run.

Pros:

- no link can dangle,
  with hard links.

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
and C's is every start under its own name,
which B shares,
plus a path that exists where the wrapper's name is expected.

C over D:
C has one file and one upgrade step;
D multiplies both and adds a drift a person cannot see.

What none of them prevents,
and this design does not pretend otherwise:
a copy of the wrapper's bytes somewhere else,
started as root under a name of its own,
runs the whole wrapper
(cases 20 and the stray other name;
run:
keep/dispatch-cases.log and keep/names-starts.log),
and a start of any of these files under a caller-chosen `argv[0]` that is not the zedlet's
is the wrapper as that caller.
An account that can write root's install folder can replace any of these files,
which is why `doctor` checks owner,
mode,
the ancestor folders and the version the installed file prints.

The choice is separable from the others.
If the owner prefers B or C,
the unit lines keep one shape and only the path in them changes;
what is lost is the fail-closed property the review asked for,
and choice 1's ranking would have to say so for that option.
An AppArmor rule is what the layout makes necessary either way,
and "Steps 18 and 19" states it:
the rule in `/etc/apparmor.d/local/zed` names the file the zedlet resolves to,
so a layout that changes that file's path moves one line with it.

## Choice 4: where the dispatch sits

What a placement costs root is every start-time action after it that runs.

### Every start-time action of today's wrapper

In the order the code takes them,
with what a root-mode start does at choice 4's option A for each:

1.  `args_os` collection in `main` (`main.rs:52`).
    Read:
    the vector is what a root mode is decided from.
2.  The words check and the one-name check (this design's,
    in `root_mode_names`).
    Read:
    they are the decision.
3.  `service_main::is_service_start` (`main.rs:56`),
    which reads the arguments only.
    Skipped:
    the root-mode arm answers first.
4.  `panic_notice::install_payload_free_panic_hook`.
    Installed inside the root-mode arm,
    as "The panic hook a root mode installs" recommends.
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
    today's bytes create `.git/cli-git-lock-holders/` and a marker on every start of every layout,
    `--version` included
    (run keep/today-installed-name.log).
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
and "What else the modes keep,
and what they lose" measures it.

What a root-mode start runs instead is today's handler:
the grammar,
the compatibility gate,
the mode,
and the two log waits at exit.
A new mode's own reads are on top of that and are the mode's business,
not the dispatch's.

### Option A: the first statements of `main`

The `#[cfg(target_os = "linux")]` block of "The rule,
in the entry",
before the service check,
before the panic hook,
and before the environment.

```text
$ cd /home/user/Monochromatic
$ <the installed file> monochromatic-git-policy-cli zfs-root boot-reconcile
   the mode's own line, exit=0 or 2
   .git/cli-git-lock-holders: no marker this start made
```

The second line is a prediction,
not a run:
the block returns before `entry::run_process` is called,
which is the only route to the lock pass at `entry.rs:353`,
and every start in `keep/today-installed-name.log` that reached `run_process` created the marker (run).

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
  the words,
  the one name (both inside `root_mode_names`),
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
  the lock pass and everything after it are skipped,
  which is option B's shape too;
- a parameter the library does not have:
  `run_process` is handed the arguments after the program name
  (`entry.rs:342`,
  `main.rs:72`),
  so a dispatch there needs the vector whole or the name beside it,
  and its second caller cannot supply either.
  `merge/merge_second_pass_finish.rs:429` (`run_nested`) calls `run_process` with the words of a Git command
  (["commit",
  ...]) and no program name,
  so the entry would need a parameter that one call site can only fill with "there is none".
  A root-mode start never re-enters there,
  so that is new state to keep honest for no measured gain.

Pros:

- one place decides everything,
  and `main` stays as thin as it is.

Cons:

- the cost is B's cost,
  not the lock pass's,
  which the first revision's text said wrongly;
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
   .git/cli-git-lock-holders: a marker this start made
```

(measured today,
with the bytes this host has:
the words reach the management arm,
which refuses them,
and the start has already written into the repository it stands in,
(run keep/today-lines.log)

Pros:

- nothing of `main` or `run_process` changes shape,
  so no other mode's start moves.

Cons:

- the runtime probe,
  the hook switch,
  the lock pass,
  the resolution and the started-as-Git check all run first,
  so root writes into whatever repository holds the working directory
  (run keep/today-installed-name.log);
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

The choice is separability-limited in one direction and clean in the other.
A,
B and C move the call the hook guard pins,
so each of them rewrites the three lines that guard fixes,
and the guard text moves with the choice.
D keeps every pinned line and adds a call after `plan_invocation`'s checks,
so the guard text does not move for it;
what D needs instead is a reason the lock pass and the started-as-Git check run for a root start.

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

(The line the built handler prints today:
`./handler --version` answers `mount-git-worktree-clone-datasets 0.6.1-alpha.45`,
exit 0,
(run keep/handler-usage.log);
no installed file answers it yet,
and `keep/today-installed-name.log` shows (run)
today's bytes answering `--version` with Git's own line instead.)

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
root modes: boot-reconcile, shutdown-unmount, start-probe, holder-scan, adopt-worktree-parent, record-checkout, mount-path-descendant, the sweep
```

Pros:

- the first line names the program a person runs.

Cons:

- its first line collides with the wrapper's own `git --version` banner,
  which already begins `monochromatic-git-policy-cli 0.6.1-alpha.45,`,
  so a check on the installed file's first line could no longer tell which program answered;
- `native:install`,
  `doctor` and the smoke all need new text,
  and the modes' refusals need a new program name.

### Option C: print the words and the version

```text
$ /usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets --version
monochromatic-git-policy-cli zfs-root 0.6.1-alpha.45
```

The line is written and not run (design),
and no build answers the words yet.

Pros:

- the line is a spelling a person can type,
  so the answer's "a name no command has" objection to A is answered;
- it is unique:
  the wrapper's banner continues with the version and a comma,
  and this line continues with the group word,
  so one `grep` tells them apart;
- a refusal can begin with the same words and stay findable.

Cons:

- the group word (choice 1's rule,
  choice 2's pick) becomes part of the version line,
  so changing it changes the line a check reads;
- the modes' program name in every refusal grows from one 33-character family name
  to two words plus the family,
  and each refusal is already two sentences long;
- `handlerVersionLine`,
  `native:install`'s check,
  `doctor`'s comparison,
  the smoke's `grep` and the stand-in handler all change,
  where A changes none of them.

### Ranking of choice 5

Ranking:
A > C > B.

A over C:
both are unique lines a check can read,
and both let a check tell the modes' answer from the wrapper's banner;
A keeps one short name in every refusal and every existing check,
where C buys a runnable spelling at the price of a longer name in every message and a new line in
five checks:
`handlerVersionLine`,
`native-install`'s comparison,
`doctor`'s comparison,
the release smoke's `grep`,
and the stand-in handler's line.
The design already requires the refusal to print the words and the installed path,
so a person who needs a command to type has it without the version line's help.
The cost is a build cost and not what C delivers,
so it is not why C ranks here;
what C fails to deliver is a shorter message in the errors a person meets most (design).

C over B:
both name something a person runs,
but C's line is unambiguous on its first word after the program's,
while B's first line is the wrapper's banner's prefix,
so a check would have to read the second line.

The choice is separable.
If the owner prefers C,
`handlerVersionLine` and the five checks change,
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
   prints:  the modes' usage, the words' spelling, and the root-owned path
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

(the link is the account's;
the words are still needed,
which is why they appear twice)

Pros:

- shorter than the installed path,
  so a person types fewer characters.

Cons:

- the link points at the account's copy,
  so the version a person reads describes a different file from the one root would run;
- a `sudo` that preserves `PATH` runs an account-writable file as root,
  which is one more path to an escalation this design does not need;
- the toolchain's plan expresses one post-install link per tool
  (`mpm-shadow`'s `postinstall.link` takes `sourceBin`,
  `linkName` and `linkDir`),
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

The choice is separable.
If the owner prefers B,
one toolchain plan row and one `native:install` step change,
and nothing about the mode words,
the layouts or the guards does;
what changes is which file a person's command names.

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
and the event's own variables carry the `ZEVENT_` prefix
(run keep/zed-shape-report.txt).
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

(printed today by the built handler;
the root arm is the control's and is not run here)

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

The line is written and not run (design),
and the switch is what a feature-less build would ignore.

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
and the design says so rather than claiming a measurement it did not make
(it shows ZED cannot set one,
which is neither direction's evidence;
run:
keep/zed-shape-report.txt).

The choice is separable,
in one direction.
Keeping the variables changes nothing but the release build's contents,
so it can be decided on its own at any time.
Compiling them out is not separable from the guest arms:
each guest arm sets a switch,
so B needs `e2e/start-probe-guest.ts` to build and test the feature build instead,
and the arms' evidence changes from the release's bytes to a second build's.

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
  names the words and the installed path instead,
  while the program's name in the opening sentence of a refusal stays (choice 5,
  option A);
- the `--help` paragraph that claims both install paths exist says who creates them and when;
- a root mode no longer passes the lock sweep,
  so a mode started from inside a repository writes nothing into that repository
  (measured in "Every start-time action of today's wrapper");
- the modes' usage's last paragraph names the words and the one root-owned file;
- two modes join the list,
  and the checkout record is the one thing they add to what a boot must have.

### The two programs' sizes, at one commit

Built from the merge commit this round began at,
`package/git-policy/cli/target/release/monochromatic-git-policy-cli` is 29,168,472 bytes
and `.../mount-git-worktree-clone-datasets` is 3,418,256,
which is 8.53 times
(the log also holds the build's own `ls`;
run:
keep/handler-usage.log).

### The cold start, measured four ways

What the numbers are,
said exactly,
because the first revision's labels were wrong and its two arms did different work:

- every measured start but the last is a **page-cache-cold** start of the program's own file.
  On this host `/home` and the build directory are ZFS,
  so a start after `POSIX_FADV_DONTNEED` on that file alone is **ARC-warm** for the file's own blocks;
- only the first start after such an eviction is cold.
  A per-event start on a running system finds the pages in memory,
  which is the warm arm in "The cold start,
  measured four ways";
- the units are **major page faults**,
  not pages read;
  a major fault brings in at least one page and often more,
  and the guest's in-block counts are the byte measure.

The commands each arm ran,
and what each did:

- arm `w`:
  the wrapper,
  `./wrapper monochromatic-git-policy-cli service run status`,
  which is the wrapper's main-level mode start,
  exiting 3 after the service's own setup and log wait;
- arm `h`:
  the handler,
  `./handler --version`,
  which is the handler's whole main,
  grammar and version answer,
  exiting 0;
- the warm arm `w` to `h` is the same pair with nothing evicted between runs.

No arm is the proposed start.
A root-mode start runs `w`'s dispatch and then `h`'s mode,
and this round did not build that start,
so the pairs and ratios in this section are what two programs cost,
never a bound or an estimate of the third.
The arithmetic that would make them one is what the second revision stated,
and this revision withdrew it (design).

Host,
page-cache-cold and ARC-warm,
41 rounds,
arms interleaved and alternating
(the second revision's host arms,
`keep/host-wrapper-service-against-handler.txt` and
`keep/host-spread-handler-against-itself.txt`,
analysed in `keep/cold-start-analysis-rerun.txt`,
replay).

- `w`:
  median 5.69 ms,
  421 major faults;
- `h`:
  median 2.05 ms,
  78 major faults;
- `w - h` per round:
  median **+3.61 ms**,
  range 2.57 to 8.60 ms;
- `w / h` per round:
  median **2.81**,
  range 1.84 to 5.06;
- the control,
  `h` against itself:
  median -0.01 ms,
  ratio 1.00 (0.79 to 1.72),
  so the method resolves a difference of this size and not much smaller;
- a second pair as context:
  the same wrapper started with the words as they behave today
  (`./wrapper monochromatic-git-policy-cli zfs-root boot-reconcile`,
  exit 2 after the management arm's refusal)
  gave median 5.94 ms and 439 major faults,
  ratio 2.97.

Guest,
Ubuntu 24.04 (kernel 6.8.0-147,
glibc 2.39,
4 CPUs),
the whole page cache dropped before every run,
21 rounds,
created and destroyed through the repository's own guest driver
(`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts create` and `mvm destroy`)
and measured by `keep/coldstart_drop.c`,
which syncs and writes 3 to `/proc/sys/vm/drop_caches` before every run
(the second revision's guest arms,
`keep/guest-disk-cold-wrapper-service-against-handler.txt` and
`keep/guest-disk-cold-spread-handler-against-itself.txt`,
replay).

- `w`:
  median 12.82 ms,
  37 major faults,
  11,384 in-block reads;
- `h`:
  median 7.45 ms,
  15 major faults,
  3,760 in-block reads;
- `w - h` per round:
  median **+6.06 ms**,
  range 0.50 to 27.60 ms;
- `w / h` per round:
  median **1.87**,
  range 1.05 to 4.00;
- the control,
  `h` against itself:
  median +0.46 ms,
  ratio 1.06 (0.46 to 2.96),
  whose range overlaps the arm's,
  so the guest's method resolves the direction and not the size;
- the difference in disk reads is 3,760 blocks of 512 bytes against 11,384,
  which is 1.9 MB against 5.8 MB,
  so a disk-cold start of the wrapper reads about 3.9 MB more than the handler's.

The same guest measured with only each file evicted (no `drop_caches`),
which is page-cache-cold and disk-cold for that file but warm for the loader and libc,
gave `w` 11.91 ms and `h` 4.44 ms,
ratio 2.91
(replay keep/guest-page-cache-warm-wrapper-service-against-handler.txt).

The warm per-event start,
which the second revision left unmeasured,
measured this round on this host with the same harness and nothing evicted,
120 rounds,
arms interleaved
(this round's warm arms,
`keep/host-warm-wrapper-service-against-handler.txt`,
`keep/host-warm-spread-handler-against-itself.txt` and `keep/warm-analysis.txt`,
run).

- `w`:
  median 1.17 ms,
  no major faults;
- `h`:
  median 0.65 ms,
  no major faults;
- `w - h` per round:
  median **+0.48 ms**,
  range 0.08 to 3.67 ms;
- `w / h` per round:
  median **1.72**,
  range 1.09 to 3.26;
- the control,
  `h` against itself:
  median +0.01 ms,
  ratio 1.02,
  range 0.50 to 1.83.

What that means for the decision:

- a warm per-event start,
  which is what each history event pays,
  costs both programs about a millisecond on this host,
  and this round does not resolve the gap between them:
  the pair's 0.48 ms median is smaller than the control's own per-round spread,
  though every one of the pair's rounds was positive,
  so the warm gap is at most about half a millisecond,
  and no figure for the proposed start is claimed from it;
- a root start after boot,
  or the first after an eviction,
  is where the difference lives:
  the wrapper's cold start is about twice the handler's on this host and 1.87 times it in the guest,
  with the arm's own range stated beside each;
- the start-path cost is the decision's accepted price;
  if it ever bites,
  the fix is a smaller early path,
  not a second program.

One sample of the earlier round is kept apart rather than folded into a median:
the handler's first run took 12.26 ms and read 1,744 blocks,
while its other 24 runs read none
(replay keep/cold-start-analysis-rerun.txt).

This round's host numbers differ from the first revision round's
(wrapper 9.78 ms,
handler 3.57 ms,
ratio 2.68,
pairs 1.46 ms and 7.65 ms for its controls)
because the host's load differed between the rounds,
which is why only the interleaved pair and the ratio are reported here,
and why the shape of the difference,
not its milliseconds,
is the finding.

### Lost

- the smaller program:
  8.53 times the bytes,
  and one more file in every page cache;
- a release that can pin or roll back the root side separately from the wrapper,
  which is the accepted cost of the placement the project already chose.

## What changes, and in which step

Every path in this section is re-read on `main` at `f3d81e0910` with `rg`,
and the line numbers are that commit's.
`rg -c "mount-git-worktree-clone-datasets"` finds 148 lines in 55 files,
of which the crate's `doc/` records and one `perf/` measurement are history and stay as they are,
and this proposal is one of the 55;
without it the count is 122 lines in 54 files
(run keep/occurrences.txt).

### One commit: the release ships one program

These cannot be separated,
because the guard reads the manifest and the release lists in one test,
and `toolchain:lock:check` compares the plan and the lock offline:

- `package/git-policy/cli/Cargo.toml`:
  remove the second `[[bin]]` (`:46` to `:48`) and its comment (`:39` to `:45`),
  and correct the comment that counts executables (`:54`);
- `package/git-policy/cli/src/native/mount_git_worktree_clone_datasets/main.rs`:
  deleted,
  with its folder;
- `package/git-policy/cli/src/native/lib.rs:415` to `:428`:
  the module's doc paragraph,
  which says "the crate's second executable" (`:421`);
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
  the smoke step (`:2671` to `:2680`),
  and the second `bin=` copy line (`:2695` to `:2698`);
- `toolchain.plan.jsonc`:
  the row's `bins` holds one name (`:271` to `:274`),
  and `toolchain.lock.json` takes it through the tool that writes the lock offline,
  which is the command `toolchain:lock:check` runs without `--frozen-lockfile`,
  never by hand;
- `package/git-policy/cli/mise.toml`:
  the descriptions that count two executables (`:88`,
  `:102`,
  `:158`)
  and the e2e task named for the second program (`:268` to `:270`);
- `package/git-policy/cli/README.md`:
  `:11`,
  `:1912`,
  `:7188`,
  `:7195` and `:8038`;
- `package/git-policy/cli/e2e/README.md`:
  `:833`,
  `:863`,
  `:1089`,
  `:1090` and `:1093`,
  where the second program's guest task is described;
- `package/zfs/request-to-openzfs-kernel-device/e2e/README.md:539`,
  which says the fixture waits for step 5's binary;
- `package/git-policy/cli/SPEC.md`:
  "## The two executables and shipped policies" (`:249`) becomes
  "## The single executable and shipped policies" and says how the root modes are reached;
  `:255` and `:266` name the family through the words or the one file;
  `:7652`,
  `:21812` and `:21992` name a mode's command and become the words;
  `:19261`'s sentence is rewritten as "The guard change,
  exactly" gives it;
- `package/git-policy/cli/doc/step-5-second-executable-2026-11-20.md`:
  a note under its status that the second program was removed by this decision,
  with the date,
  keeping the record's own text;
- `package/git-policy/cli/doc/holder-scan-across-mount-namespaces-2026-11-22.md`
  and `package/git-policy/cli/doc/slot-identity-store-write-cost.md`:
  historical measurements that name the program as it was,
  so each keeps its text and gains a note giving the current spelling and the date;
- `package/git-policy/cli/perf/sweep-cost-2026-11-23-statmount-interleaved.json`:
  unchanged,
  for the same reason;
- `doc/decision/` and `doc/handover/`:
  untouched by this design.

### The same commit: the entry, the names module, the guards, and every test that started the second program

Deleting the `[[bin]]` breaks these at compile time,
so they land with the release lists:

- `src/native/main.rs`:
  the `#[cfg(target_os = "linux")]` block of "The rule,
  in the entry",
  its `#[allow(clippy::disallowed_methods)]` on the one collection,
  the comment at `:53` corrected to name the one name and the words,
  and the hook installed inside the arm;
- `src/native/worktree_clone_mount_handler/root_mode_names.rs` (new):
  the word that cannot be honoured,
  the two constants and `root_mode_start`,
  with its controls,
  and the folder's top doc listing it first;
- `src/native/worktree_clone_mount_handler/root_mode_names_tests.rs` (new):
  the cases in "What each decides" as unit tests,
  including the two the shell cannot spell;
- `src/native/worktree_clone_mount_handler/mod.rs`:
  `pub mod root_mode_names;` first in the list,
  and the top doc's "the second executable" paragraph (`:2` to `:12`) rewritten;
- `src/native/worktree_clone_mount_handler/handler_mode_arguments.rs`:
  `MODE_WORDS` (`:41`) gains `record-checkout` and `mount-path-descendant`,
  `parse` (`:319`) gains their arms and options,
  the four refusal messages that name the program (`:220`,
  `:223`,
  `:233`,
  `:245`),
  `usage()` (`:265`,
  whose first line at `:267` names it,
  and whose last paragraph names both
  installed paths and becomes the one file and the words),
  and its tests;
- `src/native/worktree_clone_mount_handler/handler_install_paths.rs`:
  `SERVICE_INSTALL_PATH` (`:32`) becomes `ROOT_MODE_INSTALL_PATH` and holds the one root-owned file's path,
  `PROGRAM_NAME` (`:28`) stays for choice 5's option A,
  and the checkout record's path joins them as one constant;
- `src/native/worktree_clone_mount_handler/handler_install_paths_tests.rs`
  and `handler_mode_arguments_tests.rs`,
  where the same strings are asserted;
- `src/native/worktree_clone_mount_handler/record_checkout_mode.rs` (new):
  the record mode's proof,
  refusal list and line,
  with its controls,
  in the shape `adopt_worktree_parent_mode.rs` has;
- `src/native/worktree_clone_mount_handler/mount_path_descendant_mode.rs` (new):
  the mount mode's read of the record,
  its idempotence,
  its refusals and its line,
  with its controls;
- `src/native/worktree_clone_mount_handler/handler_mode_dispatch.rs`:
  `refuse_root_mode_misuse`,
  which says the misuse,
  the usage and answers `EXIT_REFUSED`,
  and the two new modes' dispatch arms;
- `src/native/source_guard/hook_self_invocation_tests.rs`:
  the test renamed and its body replaced,
  as "The hook guard on `main.rs`" gives it,
  with `names_the_argument_vector` beside it,
  and `code_lines` moved out;
- `src/native/source_guard/mod.rs`:
  `pub(super) fn code_lines`,
  and the declaration of `root_mode_name_tests`;
- `src/native/source_guard/root_mode_name_tests.rs` (new),
  with its literals and its literal reader;
- `src/native/source_guard/no_self_invocation_rules_tests.rs`:
  the `argv0-dispatch` needle `/proc/self/cmdline`,
  the `argv-vector-read` rule and its one `ALLOWED` row;
- `src/native/source_guard/no_self_invocation_tests.rs`:
  a seventh `samples` pair (`:191`),
  which `:208`'s length assertion carries;
- `src/native/source_guard/no_self_invocation_tests.rs`'s planted-source controls,
  which fire the new rule once and no more;
- `package/git-policy/cli/clippy.toml`:
  the two `std::env::args` rows;
- `src/native/binary/binary_supervisor_tests.rs:755`,
  `src/native/push/child_watch_tests.rs:1000` and
  `src/native/landing/service_durable_tests.rs:241`:
  one `#[allow(clippy::disallowed_methods, reason = "…")]` each,
  matching the crate's existing allows;
- `src/native/binary/binary_external_hook_tests.rs`:
  the doc comment of `a_start_name_that_used_to_be_an_entry_behaves_as_git` (`:174`)
  says the one exception,
  and its body stays as it is;
- `src/native/testing/root_mode_alias.rs` (new):
  one writer of the installed name in a fixture folder,
  beside `testing/fixture_executable.rs`,
  so each test that starts a mode names one link and not one copy of the rule;
- the six test files whose `CARGO_BIN_EXE_mount-git-worktree-clone-datasets` lines
  "The release's lists,
  and what reads them" lists:
  each starts `CARGO_BIN_EXE_monochromatic-git-policy-cli` through the fixture link,
  and its assertions on the output stay as they are;
- `src/native/binary/binary_service_mode_tests.rs:6`:
  the comment that counts two executables;
- `src/native/invocation/management_arguments.rs`:
  one line in `MANAGEMENT_USAGE` (`:46`) and one in `MANAGEMENT_HELP` (`:59`)
  for `monochromatic-git-policy-cli zfs-root <mode>`,
  and the tests that assert those texts
  (`invocation/management_arguments_tests.rs`,
  `invocation/entry_command_word_tests.rs:108` and `:113`);
- `src/native/health/doctor_zfs_route/identity_checks.rs:145` and `:164`:
  the two messages that tell an administrator to run the old program name;
- `src/native/health/doctor_zfs_route/handler_checks.rs`:
  `PLACEHOLDER_EXECUTABLE_PATH` (`:60`) becomes the settled root-mode path,
  `PLACEHOLDER_ZEDLET_PATH` (`:70`) becomes the installed name
  (which is also today's ZED hook name,
  so the two stand-ins become one),
  and their controls (`route_test_facts.rs:304`,
  `handler_checks_tests.rs`) move with them;
- `e2e/start-probe-guest.ts:42`:
  `HANDLER` becomes the wrapper pushed into the guest plus the installed-name link its commands run,
  and every `${HANDLER} start-probe` line keeps its shape;
- `e2e/mount-git-worktree-clone-datasets-guest.ts`:
  its guest build loses the second program
  (the tarball excludes `target` at `:330`,
  and the guest compiles what it runs at `:340`),
  and the guest creates the installed-name link before the adoption arm runs;
- `e2e/start-probe-guest.ts:196`,
  whose `cargo build --release --locked --bins` builds one binary fewer.

The two new modes' own files are in this commit because the grammar,
the dispatch and the record's writer are one shape,
and the readers land in step 10's build with the record's format.
Until then `record-checkout` writes a record nothing reads and
`mount-path-descendant` refuses `no-recorded-checkout`,
which is a safe half:
the plan's step 10 build turns the readers on,
and the step 19 build changes its nested dataset's unit.

### The same commit: what `native:install` and the release smoke check instead

`native:install`'s handler check becomes the words' check,
which is the contradiction's resolution:

```ts
// bin/native-install.ts, where the handler's check stands today (:444 to :465)
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
and its doc says the name is the mode family's,
not a file's.
`bin/native-install.unit.test.ts:417`'s test
("refuses when the release archive moved only the wrapper")
expects the words' line instead of the program's,
and `bin/stand-in-install-programs-for-tests.ts` (`:169` to `:186` and `:335`)
teaches its stand-in `git` to answer the words and loses its stand-in handler.

The release smoke gains the two arms,
so a release cannot ship with either dispatch broken
(`.github/workflows/cargo-publish.yml:2671` to `:2680`):

```yaml
    - name: Smoke-run the executable
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
  step 10 gains the record's reader and its format;
  step 18's `doctor` (`:2464`) gains the installation check "Steps 18 and 19" states;
  step 19 (`:2583`) names the one file,
  the symlink and the words,
  and its acceptance arm links the name and reads `--version`;
- `doc/planning/cli-git-zfs-worktree-design-proposal.md`:
  "Binding crate and compatibility" loses the separate-binary paragraph and keeps the "Keep inside cli"
  placement with this design's answer,
  "ZED reconciliation and mount safety" keeps its two installed paths and says they are one file and its name,
  and "Decided defaults" replaces the second binary's cost with the numbers measured here;
- `doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
  `cli-git-zfs-start-probe-design.md` and
  `cli-git-zfs-private-propagation-after-replacement-mount-design.md`
  keep their commands and gain a sentence that the name at the head of each is the one root-owned file
  (their `ExecStart`,
  `ExecReload` and probe lines name the modes' own name today,
  so each line changes to that file's path plus the words);
- `doc/planning/cli-git-no-self-invocation.md`,
  "The rule":
  the `argv[0]` bullet gains the one exception,
  citing the start-name record;
- `doc/planning/cli-git-resident-service.md:3324`,
  the sentence that rejects a name-based dispatch:
  it stays true for the service and gains "except the ZED hook's installed name";
- `doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`,
  "What each part waits on":
  the two entries that point at this design's modes gain their words and arguments,
  and the nested unit's `ExecStart=` gains the mode's line.

### Steps 18 and 19

They land in their own steps,
and nothing waits on them for the release to hold one program.

Step 18's `doctor` gains,
in one place:

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
  `ls -d /usr/libexec` answers `No such file or directory`,
  (`keep/apparmor-and-folders.txt`,
  run);
- the installed file's `--version` names the same version as the build `doctor` runs from,
  and a mismatch prints both and the lock row's version;
- the checkout's record exists,
  names a dataset and GUID the kernel agrees with,
  and its container is under the same pool,
  because every later step that names a checkout reads it;
- its findings say what a broken or missing name costs (a sweep that never runs)
  and print the command that recreates it,
  through the root-owned path and never through the account's `git`;
- the three unit lines it prints,
  the nested dataset's line,
  the adoption's line and the record's line
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

Step 19 also writes the AppArmor rule,
which the owner's answer puts in `/etc/apparmor.d/local/zed`
(record doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md).
What that file must hold,
and why the design cannot leave it implicit:

- this host has no `local/zed` yet
  (`keep/apparmor-and-folders.txt`,
  run),
  and `/etc/apparmor.d/zed` ends with `include if exists <local/zed>`,
  so every rule the hook needs is one this step adds;
- the profile gives `/etc/zfs/zed.d/{,*} r,` and no execute permission,
  so a zedlet cannot be started at all until a rule grants it;
  the rule therefore names the file AppArmor resolves the zedlet's path to,
  which under choice 3's option A is the root-owned file in `/usr/libexec`;
- AppArmor's documented default is to resolve symlinks before matching a rule,
  and this round did not measure that,
  so the ZED-enable step proves the rule with the audit log before ZED is enabled,
  and a rule that names the symlink instead would fail closed and be visible there;
- the rule carries what the sweep needs on top of exec:
  `capability dac_read_search`,
  and the mount rules for `/home/user/Monochromatic` and the clone container,
  because ZED's profile holds `capability sys_admin` and `/dev/zfs rw` already
  (`keep/apparmor-and-folders.txt` shows both lines,
  run);
- the file's name and the rule's target move together with choice 3,
  so a later layout change is one line in two files,
  and `doctor`'s step 18 check reads whether the rule names the file that exists.

## What the probes showed, and what they did not

### Shown

- ZED's exec shape,
  `argv[0]`,
  argument count,
  working directory,
  descriptors and built environment,
  by a replay of the source's own child step
  (run keep/zed-shape-report.txt);
- a transient user service's and a user scope's `argv[0]`
  (run keep/unit-argv0.log);
- the three dispatch rules' decisions on thirty-five starts,
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
  the two new modes' words,
  the record mode under the installed name,
  links named `git`,
  the wrapper and the modes' family,
  a caller-chosen `argv[0]`,
  an empty `argv[0]`,
  and the two vectors a shell cannot spell
  (run keep/dispatch-cases.log);
- the compiled names module under real starts by path,
  through links,
  with `exec -a`,
  through the ZED child step,
  with a non-UTF-8 and an empty `argv[0]`
  (run keep/names-starts.log);
- the three revised guards against today's `main.rs`,
  this revision's `main`,
  seven edits of `main`,
  the read's six bypasses,
  the names module,
  three planted modules,
  the read's four module variants and four of this round's own,
  with every variant failing and each rule having its own witness
  (run keep/guard-replay-revision3.log);
- the typed bans in a probe crate carrying the crate's own `clippy.toml` and the two new rows,
  through a `use ... as` alias and a direct `std::env::args()`
  (run keep/clippy-bans.log);
- today's bytes under the installed name in a throwaway repository,
  with the status of each start and the marker the lock pass leaves
  (run keep/today-installed-name.log);
- a copy named `holder-scan` answering `unknown option: --dataset` and exit 129
  (run keep/holder-scan-name.log);
- both programs' cold starts on this host with the command each arm ran,
  page-cache-cold and ARC-warm,
  with the method's own spread measured in the same way
  (`keep/host-*.txt` and `keep/cold-start-analysis-rerun.txt`,
  replay);
- both programs' warm starts on this host,
  nothing evicted,
  with the same spread control
  (`keep/host-warm-*.txt` and `keep/warm-analysis.txt`,
  run);
- both programs' cold starts in an Ubuntu 24.04 guest,
  the whole page cache dropped before every run,
  with the same spread control,
  and the guest destroyed through `mvm`
  (replay keep/guest-*.txt);
- the two programs' sizes in this round's build,
  and `readelf --version-info`'s glibc floor for each.

### Not shown

- ZED itself:
  it needs root,
  `/dev/zfs` and a pool,
  so the replay stands in for its child step only.
  The environment and descriptor values come from the replay's own `execle`,
  not from a running ZED;
- a system unit start:
  this round measured a transient user service and a user scope,
  so a system manager's `argv[0]` is the same `ExecStart` parse and not measured here;
- any mode touching ZFS:
  this round ran no guest arm that mounts,
  unmounts or adopts,
  so the modes' own work is unchanged code and is not exercised here;
  the two new modes do not exist,
  so their lines and refusals are this design's words;
- the checkout record's readers and format:
  they land in step 10's build,
  and nothing here has read or written one;
- the proposed root-mode start's cold or warm cost:
  no arm runs it,
  and this design claims no figure for it;
- a disk-cold start on the host:
  the host cannot drop its page cache without root,
  which is why the guest arm exists;
- a comparison of ARC-cold starts:
  the guest has no ZFS,
  so its numbers are a disk-cold floor on ext4,
  not a ZFS ARC measurement;
- whether AppArmor resolves the zedlet's symlink before matching the `local/zed` rule:
  the ZED-enable step proves it with the audit log;
- whether `doctor`'s future installation check catches a link that a package manager renames instead of deleting;
- the release smoke arm itself:
  it is written here and not run,
  because this round changes no workflow;
- the root-mode refusal's and the two new modes' exact texts:
  `refuse_root_mode_misuse`,
  `record_checkout_mode` and `mount_path_descendant_mode` do not exist yet,
  so their wording is this design's,
  not a build's.

## The choices this proposal puts to the owner

Each is separable,
and each is ranked in its own section.
Item 1 is where the answer's two clauses meet,
and the design recommends it;
items 2 to 7 are the second revision's other choices;
item 8 is the word of the mode the step 10 answer adds.

1.  What selects a root mode:
    the words on any file,
    and under the installed name ZED's no argument,
    `--help` and `--version`,
    with every other word there refused (choice 1,
    A);
    the same without `--help` and `--version` under the name (B);
    or the first revision's rule,
    where the name wins and drops the words (C).
    The design recommends A.
2.  The group word the units and people spell:
    `zfs-root` (choice 2,
    A),
    `clone-mounts` (D),
    `mount` (B),
    or `root` (C).
3.  What root's install holds:
    one file under the ZED hook's name and nothing under the wrapper's (choice 3,
    A),
    one file named for the modes (B),
    one file named for the wrapper (C),
    or a copy or hard link per name (D).
    Under A the name selects ZED's start and the words select every mode;
    under B and C only the words select,
    and a stray argument at that path runs Git as root.
4.  Where the dispatch sits:
    the first statements of `main` (choice 4,
    A),
    after the panic hook and the environment (B),
    at the top of `run_process` (C),
    or inside `plan_invocation` (D),
    with the panic hook installed inside the arm (the recommendation).
    B and C rank on where the code lives,
    not on start-time work:
    they cost the same.
5.  What `--version` and the refusals say:
    the modes' own name (choice 5,
    A),
    the words and the version (C),
    or the wrapper's name and a modes line (B).
6.  Whether the account's `PATH` keeps a short name:
    the installed path and the words,
    printed by `doctor` (choice 6,
    A),
    or a second toolchain link (B).
7.  Whether the release keeps the modes' test-only variables:
    keep them (choice 7,
    A),
    or compile them out (B).
8.  The words of the mode that records a checkout:
    `record-checkout` (the record mode's ranking,
    A),
    `record-route` (B),
    or `checkout-record` (C).

This proposal's own ranking is A in every one of them.

## Appendix: where the probe transcripts are

Round folder `/home/user/temp/agent/zfs-root-modes-third-revision-2026-11-24/keep/`:

- `state-so-far.md`:
  what the round did,
  in order,
  and what it could not do;
- `dispatch-cases.log`,
  from `dispatch_probe.rs`:
  the three rules on thirty-five starts,
  with cases 1 to 26 identical to the second revision's twenty-six;
- `kept-first-78.txt` and `mine-decided.txt`:
  the two decision lists the case-by-case comparison reads;
- `zed-shape-report.txt`,
  from `zed_replay.c` and `replay_fixture.rs`:
  ZED's child step,
  replayed;
- `names-starts.log`,
  from the compiled `root_mode_names.rs` and `run_mine.sh`:
  real starts by path,
  through links,
  with `exec -a`,
  through the ZED child step,
  and with an empty argument;
- `guard-replay-revision3.log`,
  from `replay_revision3.rs`:
  the three guards of this revision over every `main` text and every names module;
- `revision2-harness-over-every-text.log`,
  from `guard_replay_revision2.rs`:
  the second revision's own predicates,
  replayed to check the harness;
- `clippy-bans.log`,
  with `clippy.toml` and `clippy-probe-main.rs`:
  the two typed bans in a probe crate carrying the crate's `clippy.toml`;
- `today-lines.log`,
  `today-installed-name.log` and
  `holder-scan-name.log`:
  today's bytes in a throwaway repository,
  under the installed name and under a copy named `holder-scan`;
- `handler-usage.log`:
  the built handler's `--help`,
  `--version` and sizes;
- `unit-argv0.log`:
  a transient user service's and a scope's `argv[0]`;
- `apparmor-and-folders.txt`:
  the AppArmor profile lines this design depends on,
  and `/usr/libexec`'s absence;
- `host-wrapper-service-against-handler.txt`,
  `host-spread-handler-against-itself.txt` and
  `host-wrapper-words-against-handler.txt`:
  the second revision's host cold-start arms,
  replayed;
- `guest-disk-cold-wrapper-service-against-handler.txt`,
  `guest-disk-cold-spread-handler-against-itself.txt` and
  `guest-page-cache-warm-wrapper-service-against-handler.txt`:
  the second revision's Ubuntu guest arms,
  replayed;
- `host-warm-wrapper-service-against-handler.txt` and
  `host-warm-spread-handler-against-itself.txt`:
  this round's warm arms;
- `cold-start-analysis-rerun.txt`:
  the second revision's arms,
  re-analysed;
- `warm-analysis.txt`:
  this round's warm arms,
  analysed;
- `occurrences.txt`:
  the `mount-git-worktree-clone-datasets` counts;
- `proposed_main.rs`,
  `root_mode_names.rs` and
  `proposed_root_mode_names.rs`:
  the three texts the guards are proved against;
- `variants/`:
  the read's six bypasses (hv1 to hv6) and four module variants (mv1 to mv4),
  and this round's four forbidden-spelling witnesses (w1 to w4);
- `dispatch_probe.rs`,
  `exec_empty_argv.c` and `argv_view.c`:
  the sources every dispatch case rebuilds from;
- `zed_replay.c`,
  `replay_fixture.rs`,
  `run_mine.sh`,
  `coldstart_drop.c` and
  `analyse_cold_start.ts`:
  the harnesses the logs above came from.

The round's `scratch/` folder held the compiled guards,
the fixtures and the throwaway repository,
and was removed when the round ended;
the kept sources rebuild every log.
No guest was created this round:
the Ubuntu guest's logs are the second revision's,
replayed here with an identical analysis.
