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
Every option is shown with a probe run in this round's scratch folder,
and every fact about a program this repository does not own is read from that program's source.
The decision records and the handover are untouched by this round.

This revision is the round of 2026-11-24T01:16:30-05:00.
It replaces the earlier text of the same file,
whose choice 1 offered four ways to reach a root mode and recommended reading a name the repository had banned.
The owner answered that question,
and the gate review of the earlier text,
kept under
`~/temp/agent/gate-review-zfs-root-modes-in-the-one-wrapper-executable-2026-11-24/`,
found fifteen further things;
section "What the owner answered, and what the gate review found" says what became of each.

## What the owner answered, and what the gate review found

### The start-name answer

The start-name record's "The start name, for the ZED hook only" answers "Lift it for the ZED hook only (Recommended)".
What it fixes is narrow,
and the whole of choice 1 follows from it:

- one file name,
  the installed ZED history event's,
  selects a root mode,
  compared as the whole last path component;
- the boot and shutdown units,
  adoption and a person reach the root modes by words in the wrapper's own namespace,
  as `service run` does;
- `SPEC.md`'s sentence and the two source guards change to allow exactly that one name and nothing else.

The record also hands the design a request of the review's to adopt or not:
the root-owned regular file should carry the mode's name rather than the wrapper's,
so that a root start under any other name fails with no such file instead of running Git as root.
Section "Choice 3" adopts it and measures what each layout does.

### The review's findings, and what this revision does with each

The review's own words,
then this revision's answer.
Its High finding is the owner's question, already answered,
so it is the first item.

#### The name selector reverses a written contract

The review called this its High finding.
The owner lifted the ban for one name,
and section "The guard change, exactly" replays both guards' predicates against today's `main.rs`,
the proposed `main`,
and three variants,
so the change to each guard is stated as the single assertion it is.
`SPEC.md:19050`,
`src/native/source_guard/hook_self_invocation_tests.rs:74` to `:83`,
`src/native/source_guard/no_self_invocation_rules_tests.rs:25` to `:35`,
`src/native/main.rs:53` and
`src/native/binary/binary_external_hook_tests.rs:170` are all in the change list,
with the text each one gets.

#### A root-owned file under the wrapper's own name fails open

The review's Medium finding, adopted.
Choice 3's option A installs one regular file under the ZED hook's name and nothing under the wrapper's,
so a typo, a stale line or a mistyped mode word at that path is the modes' own refusal,
and a name nobody installed is `No such file or directory`.
Probed for three layouts as the account,
in a throwaway repository,
with a stray copy beside them.

#### The change list is incomplete and contradicts itself

The review's Medium finding.
Four corrections,
each shown against `main` now:

- six Rust test files start the second program by
  `env!("CARGO_BIN_EXE_mount-git-worktree-clone-datasets")`,
  not four and not five:
  `binary/binary_mount_git_worktree_clone_datasets_tests.rs:105`,
  `binary/binary_zfs_adoption_guest_tests.rs:118`,
  `binary/binary_zfs_holder_scan_guest_tests.rs:290` and `:892`,
  `binary/binary_zfs_step9_m_arms_tests.rs:200` and `:561`,
  `binary/binary_zfs_sweep_guest_tests.rs:229` and `:242`,
  and `binary/binary_zfs_unmount_guest_tests.rs:238`;
  each is a binary-level control under `src/native/binary/binary_tests.rs`.
- `native:install`'s handler check does leave the account's install,
  because no released archive places a family name on the account's machine
  (`bin/native-install.ts:448` runs `handlerProgram` by bare name);
  the `--version` line it reads stays,
  and the check becomes a words check on the installed `git`,
  which keeps a check instead of deleting one,
  as "The same commit: what `native:install` and the release smoke check instead" states.
- the release smoke gets a replacement arm that links the one name to the built wrapper and reads `--version`,
  and a second arm that reads `--version` through the words.
- `SPEC.md:19050`'s sentence is rewritten to the text the answer allows.

#### Root's copy has no stated provenance or version check

The review's Medium finding, folded in:

- the bytes come from the lock row's archive:
  `toolchain.lock.json`'s `cargo:monochromatic-git-policy-cli` row holds
  `route: "archive"`,
  `artifacts.linux-x64.url` and its `sha256`,
  and `bins`;
  step 19 downloads that URL,
  verifies the digest,
  and installs the one file it extracts,
  never a copy of the account's;
- `doctor` runs the installed file's `--version` as the account,
  compares its version with the build `doctor` itself runs from,
  and reports both when they differ;
- `/usr/libexec` does not exist on this host
  (`ls -d /usr/libexec` and `stat` both answer `No such file or directory`),
  so `doctor` checks every ancestor folder and prints the command that creates the missing ones.

#### Cold-start cost is read but not measured

The review's Medium finding, measured here
as a ratio to the handler's,
interleaved,
with the method's own spread measured first.
The guest the brief asked for could not receive or run the binaries
(the four exact refusals are in "What the probes showed, and what they did not"),
so the measurement ran on this host with each file's own cached pages evicted before every run.
Numbers in "What else the modes keep, and what they lose".

#### ZED's working directory is `/`, not a repository

The review's Low finding holds,
and this revision says so:
`keep/zed-source/zed.c:238` to `:241` sets `umask(0)` then `chdir("/")`.
The replay harness now does the same,
and the earlier round's Z1 probe (a zedlet start from inside a repository)
is named in "Every start-time action of today's wrapper" as what it was:
not a ZED start.
The lock-holder write belongs to a person's start from inside a checkout,
such as an adoption.

#### "`argv[0]` ends in a family name"

The review's Low finding.
The wording is now the whole last path component,
and the fixture's near-miss starts show five names that a suffix test would have taken and this rule does not.

#### The selector is caller-controlled

The review's Low finding, probed and stated:
`bash -c 'exec -a <zedlet name> ...'` decides a root mode,
which is harmless as the account,
and ZED fixes `argv[0]` (`zed_exec.c:168`),
while systemd passes the path as written.
A root launcher that lets a lower-privileged caller choose `argv[0]` would matter,
and none of this design's launchers does.

#### "A root mode reads no variable" is false

The review's Low finding, corrected:
the modes' own code reads five variables,
listed in section "Choice 7",
and ZED cannot set any of them.
The direction is fail-safe.

#### Placement A installs no panic hook

The review's Low finding,
made a sub-choice of choice 4's placement:
a root mode installs the wrapper's existing payload-free hook,
because a root mode's panic message can hold a dataset name or a path.

#### The kept `probe-zed-replay.log` is stale

The review's Low finding, replaced.
Its `PATH` was the harness's own constant;
`/usr/include/paths.h` holds `_PATH_STDPATH` as `/usr/bin:/bin:/usr/sbin:/sbin`,
and this round's rerun prints that,
`cwd=/`,
`argc=0` and the three descriptors on `/dev/null`.

#### Source ranges are loose

The review's Low finding, corrected to
`zed_event.c:1007` to `:1022` for the environment's construction,
`zed_conf.c:373` to `:405` for the scan's tests,
`zed_exec.c:161` to `:168` for the child step,
and `zed_exec.c:421` for the class match.

#### The design overstates the gate the wrapper runs

The review's Low finding, and it does not hold on `main`.
The wrapper does run the compatibility gate before its first request:
`worktree_clone_add_route/add_zfs.rs:67` calls `LiveGate.verdict()` inside `OpenedDevice::open()`,
and `add_route_run.rs:206` calls that on the wrapper's add route,
both release code.
The review looked for callers of `before_requests`,
which only `LiveGate::verdict` calls.
`SPEC.md`'s "### The start-time ZFS compatibility gate" sentence stands.

#### The start-time list omits the forwarded-to-self check

The review's Low finding, added as the twentieth action,
and the list in "Every start-time action of today's wrapper" counts twenty.

#### True `argc = 0` and an empty `argv[0]` were not run

The review's Low finding, run here.
`execv(path, {NULL})` reaches the program as one empty argument
(the kernel or glibc inserts the empty string;
a C program under the same `execv` prints `argc=1 argv0=`),
and the rule decides the wrapper without indexing anything.
Both cases are in the fixture's transcript.

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

What makes the problem hard is unchanged:
the wrapper is the `git` a person runs,
so its arguments are Git's arguments,
and an argument alone cannot mean "root mode" without taking a word away from Git.
And ZED hands a zedlet no arguments at all.

## What the code holds today

### The two entries

`package/git-policy/cli/src/native/main.rs` is the wrapper's entry.
It collects the arguments once (`main.rs:52`),
answers the service mode when the leading three words are
`monochromatic-git-policy-cli service run`
(`invocation::service_main::is_service_start`),
installs the payload-free panic hook,
collects the environment,
and hands everything to `invocation::entry::run_process`,
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
`handler_mode_dispatch.rs` gives every mode the same two answers,
a line and status 0,
or a refusal line and status 2,
and runs the compatibility gate before a mode that will make a request.
`handler_install_paths.rs:28` to `:42` holds the program's name,
the two installed paths,
and the configuration path.

The service mode is the precedent this design follows,
and the answer says so:
a mode chosen inside `main`,
before the panic hook,
before the environment is collected,
and before anything of the wrapper's own start runs.

### The release's lists, and what reads them

`package/git-policy/cli/Cargo.toml:35` to `:48` declares two `[[bin]]` tables,
and its `[package.metadata.binstall]` table (`:54` to `:59`) names `{ bin }`,
so one archive unpacks both programs.
Every list that counts them must agree with the manifest,
and `bin/native-release-bins.unit.test.ts` is the guard that says so:
the release job's `cargo build --bin` flags (`.github/workflows/cargo-publish.yml:2666`)
and its `bin=` copy lines (`:2695` to `:2700`),
`package/crate-release/cli/releasable-crates.json:30`'s `checkBins` for the crate,
`toolchain.lock.json`'s `bins` row for `cargo:monochromatic-git-policy-cli`,
and `src/native/source_guard/release_bins_tests.rs:31`.
`toolchain.plan.jsonc` holds the same row as intent,
and `toolchain.lock.json` is written from the plan:
`mpm-shadow`'s update keeps every other row as it is and gives it "the plan's current bins",
so a changed plan row moves the lock row through the tool,
offline (`toolchain:lock:check` runs
`install --lockfile-only --frozen-lockfile`).
`bin/native-install-steps.ts:104` holds `handlerProgram` and `:121` `handlerVersionLine`,
and `bin/native-install.ts:448` refuses unless `mount-git-worktree-clone-datasets --version` names the release,
run by bare name through `PATH`.

Six Rust test files start the second program by
`env!("CARGO_BIN_EXE_mount-git-worktree-clone-datasets")`,
at the lines section "The review's findings, and what this revision does with each" lists.
`e2e/start-probe-guest.ts:42` names the built binary in its `HANDLER` constant
and its guest commands run `${HANDLER} start-probe`;
`e2e/mount-git-worktree-clone-datasets-guest.ts` drives the second program with the same built path.

### The ZFS code already in the wrapper, and the gate it runs

The wrapper already carries the ZFS code.
`Cargo.toml` depends on
`monochromatic-zfs-request-to-openzfs-kernel-device` for the typed `/dev/zfs` requests,
and the wrapper runs the same compatibility gate before its first request,
which plan step 7 added:
`worktree_clone_add_route/add_zfs.rs:67`,
reached from `add_route_run.rs:206`.
So a root mode inside the wrapper adds no dependency and no library:
the decision's "links no OpenZFS library and starts where OpenZFS is absent" check stays as it is.
What the second program saved was code that runs,
not code that is linked.

### The sentence and the two guards

Three places say the wrapper never reads its start name,
and the answer changes all three for exactly one name:

- `SPEC.md:19050`, in "#### External hooks":
  "the executable does not read the name it was started under".
- `src/native/source_guard/hook_self_invocation_tests.rs:74`,
  `main_does_not_dispatch_on_its_start_name`,
  which forbids `args_os().next()`, `start_name`, `current_exe`, `arg0`, `file_name`,
  `hook_dispatch` and `hook_entry` in `main.rs`,
  requires `args_os()` exactly once,
  requires `args_os().skip(1)`
  and requires `entry::run_process(`.
- `src/native/source_guard/no_self_invocation_rules_tests.rs:25`,
  the `argv0-dispatch` scan rule,
  whose needles are `args_os().next()`, `args().next()`, `.nth(0)`, `argv[0]` and `arg0(`.

Beside them,
`src/native/main.rs:53` explains the service mode as chosen
"never by the name this program was started as",
and `binary/binary_external_hook_tests.rs:170` proves that a start under each Git hook event's name behaves as `git`.
Both stay true for every name but the one,
which is what the guard change has to express.

## What a launcher hands a start

### ZED runs a zedlet by path, with no arguments

From OpenZFS `zfs-2.4.4`
(`keep/zed-source/`):

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
`PATH` at the standard path,
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
(`keep/probe-zed-replay.log`):

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
PROBE fd3=.../zed.d/zevent.nvlist
PROBE decided=root-mode: the attaching sweep (no word)
```

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
which the earlier round's replay printed.

### systemd runs what `ExecStart` spells

A transient unit started with
`ExecStart=<path> boot-reconcile`
gave the program `argv[0]` as the full path as written,
and `boot-reconcile` as its only argument,
which the earlier round's transcript (`probe-systemd-unit-start.log` in its `keep/`) shows.
So a unit may pass words,
and its `argv[0]` is a path,
which the one name rule reads.

### A caller can choose `argv[0]`, and not when ZED or systemd starts it

`exec -a` sets anything,
and ZED's `execle` and systemd's `ExecStart` fix the value.
A start whose `argv[0]` a lower-privileged caller chose as the zedlet's name is a root-mode start as that caller,
which is harmless for an account and is why none of this design's launchers passes a caller's choice through.

## Choice 1, answered: the one name, and the words

### The rule

A start is a root-mode start when either holds:

- its `argv[0]`'s whole last path component is exactly
  `history_event-mount_git_worktree_clone_datasets`,
  the installed ZED history event's own name;
- its first two arguments are exactly
  `monochromatic-git-policy-cli zfs-root`.

Nothing else is a root-mode start.
Every other start,
including `git`, `monochromatic-git-policy-cli` alone,
and the same two words in another order,
is the wrapper unchanged.

The name is compared whole after the last `/`,
so `mount-git-worktree-clone-datasets`,
`history_event-mount-git-worktree-clone-datasets`,
`xhistory_event-mount_git_worktree_clone_datasets`,
`history_event-mount_git_worktree_clone_datasets.bak` and
`History_event-mount_git_worktree_clone_datasets` are all the wrapper.
The words are compared as whole arguments,
so `zfs-rooot` is not the group word and neither is `zfs-root-2`.

### What the arguments are read as

Under either rule the modes' own grammar reads the arguments,
unchanged from `handler_mode_arguments.rs`:

- no word at all is the attaching sweep,
  which is what ZED's no-argument start needs;
- one mode word is that mode,
  with the adoption's and the holder scan's options as today;
- `--help` prints the modes and the installed path;
- `--version` prints the version line;
- anything else is the refusal that names the word and the modes,
  status 2.

Under the name rule a leading pair of the words is dropped first,
so one unit line works whether the file it starts is the ZED hook's name or another name.
A word under the name rule, with no word pair before it, is read as today,
which keeps every existing mode text and every mode control true.

### The dispatch, in the entry

```rust
// package/git-policy/cli/src/native/main.rs, the first statements of main
let arguments: Vec<OsString> = std::env::args_os().collect();
let start_name: Option<&std::ffi::OsStr> = arguments.first().map(OsString::as_os_str);
let handed: &[OsString] = if arguments.is_empty() { &[] } else { &arguments[1..] };
if let Some(mode_arguments) = git_policy_cli::worktree_clone_mount_handler::root_mode_names::root_mode_arguments(start_name, handed) {
    git_policy_cli::panic_notice::install_payload_free_panic_hook();
    let code: i32 = git_policy_cli::worktree_clone_mount_handler::run(mode_arguments.as_slice());
    end_after_the_log(code);
}
// The service check, the panic hook, the environment and entry::run_process
// are the code that stands today, with `arguments` replaced by `handed`.
```

`root_mode_names::root_mode_arguments` is the one place the name,
the words and the grammar's argument list meet,
and it lives in `src/native/worktree_clone_mount_handler/root_mode_names.rs`:
a file holding the three constants
(the ZED hook's name,
the group word,
and the wrapper's own word taken from `wrapper_names_old_and_new::COMMAND_WORD`,
the constant `service run`'s words already use)
and one predicate.
The folder already links `foundation/` in its `uses/` set,
so no link is added,
and the folder's own top doc gains the file first,
before `handler_install_paths.rs`,
because nothing else uses it and it uses only `foundation`.

The service check follows,
so the order of the three checks is:
the one name,
the words,
the service's words.
Each reads arguments only,
which is why their order cannot change what any of them does.

### Probed

`keep/probe-dispatch-decided.log`,
from `keep/fixture_dispatch_decided_rule.rs` and `keep/exec_empty_argv.c`.
The fixture is the rule of "The rule" and nothing else,
and it prints what the proposed `main` would decide.
It decided each of these as written:

- the installed zedlet path with no argument:
  the attaching sweep;
- the same file with `monochromatic-git-policy-cli zfs-root boot-reconcile`:
  `boot-reconcile`;
- the same file with `boot-reconcile` alone:
  `boot-reconcile`;
- the bare name through a `PATH` lookup with no argument:
  the attaching sweep;
- a symlink start through `/etc/zfs/zed.d/...` with `--version`:
  the `--version` arm;
- the words through the wrapper's own name:
  the word's mode;
- the words with no word after them:
  the attaching sweep;
- a person's adoption through the words:
  `adopt-worktree-parent` and its three options;
- five near-miss names:
  the wrapper;
- a trailing slash after the name:
  the wrapper;
- the wrapper's own name, with no argument and with `boot-reconcile`:
  the wrapper;
- `exec -a <zedlet name>` with `boot-reconcile`:
  `boot-reconcile`;
- an empty `argv[0]`:
  the wrapper;
- `execv(path, {NULL})`:
  the wrapper.

The bare-word case matters:
with the name rule in place,
a start of the installed file with a bare `boot-reconcile` is that mode,
where today it is `git: 'boot-reconcile' is not a git command`
(`keep/probe-naming-as-the-account.log`).
The name rule is what makes the installed file safe,
and it changes what a bare word at that one path means.

### The guard change, exactly

`keep/guard-replay.log`,
from `keep/guard_replay.rs`,
which carries both guards' predicates verbatim.
Today's `main.rs` is the positive control.

```text
== src/native/main.rs (today)
   current   hook guard: PASS
   proposed  hook guard: FAIL (the one start name read is decided by root_mode_names)
   current   argv0-dispatch rule: 0 line(s) hit
   proposed  argv0-dispatch rule: 1 line(s) hit -> 52: let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
== the proposal's main
   current   hook guard: FAIL (main.rs contains start_name)
   proposed  hook guard: PASS
   current   argv0-dispatch rule: 0 line(s) hit
   proposed  argv0-dispatch rule: 1 line(s) hit -> 7: let arguments: Vec<OsString> = std::env::args_os().collect();
== the proposal with the token renamed
   current   hook guard: FAIL (the program name is skipped, not inspected)
   proposed  hook guard: PASS
== the review's V2 (a separate name read, skip(1) kept)
   current   hook guard: FAIL (main.rs contains args_os().next())
   proposed  hook guard: FAIL (main.rs contains args_os().next())
   proposed  argv0-dispatch rule: 3 line(s) hit
== a two-name family compared in main
   proposed  hook guard: FAIL (the one start name read is decided by root_mode_names)
```

So the hook guard's change is exactly two things:
`start_name` leaves the forbidden list,
because the variable now holds it,
and the `args_os().skip(1)` assertion becomes

```rust
assert!(joined.contains("root_mode_names::root_mode_arguments("), "the one start name read is decided by root_mode_names");
```

with the test renamed `main_reads_its_start_name_only_for_the_zedlet`.
Its other assertions stay as they are,
including `args_os()` read exactly once,
the ban on `args_os().next()`, `current_exe`, `file_name`, `arg0`, `hook_dispatch` and `hook_entry`,
and the `entry::run_process(` requirement,
so `main` can neither compare a path itself nor read a second name.

The scan rule's change is the needle `"args_os()"`,
so no release file reads the argument vector without an `ALLOWED` entry,
and one entry:

```rust
Allowed {
    file: "main.rs",
    rule: "argv0-dispatch",
    why: "the one name the owner lifted the ban for: the argument vector's first element goes to root_mode_names, which compares its whole last path component against the ZED hook's installed name and nothing else",
},
```

Over the release sources that needle hits `main.rs` and, today, the second program's `main.rs`,
which this change deletes;
`every_justified_exception_still_matches_code` keeps the entry honest.

The what of the rule,
one name and whole,
is proved by the predicate's own sidecar controls rather than by text:
the near-miss names,
the empty name,
no name at all,
and the trailing slash are its cases,
and `binary_external_hook_tests.rs:170` keeps proving that every Git hook event's name still behaves as `git`.

`SPEC.md:19050` becomes:

> Git never starts the wrapper for a hook:
> the wrapper writes no hook entry,
> passes no hooks path of a directory it made,
> and reads the name it was started under in one place only,
> the installed ZED history event's,
> whose start is the clone-mount sweep's.

### What the answer costs

- `main` reads the program name,
  which no part of the wrapper did before;
  the guards narrow that to one name and one call.
- A copy of the wrapper under the ZED hook's name is a root-mode start,
  wherever it stands,
  which is why `doctor` owns the installed name's file rather than the bytes' identity.
- A start under the words of a stray copy still runs the wrapper,
  as choice 3's probe shows;
  no rule short of a root check prevents that,
  and a root check would break the wrapper under `sudo`.
- The words are a second spelling to keep true;
  the fixture's table and the release smoke both exercise them.

## Choice 2: the words units, adoption and a person spell

The answer fixes the shape of the words,
because `service run` is the precedent:
the wrapper's own word,
then a namespace word,
then the modes' grammar.
What the namespace word is,
is this choice's,
and a unit's `ExecStart` line,
an adoption's command and a person's command all read it.

### Option A: `zfs-root`

```ini
# /etc/systemd/system/reconcile-git-worktree-clone-mounts.service
# <the installed file> is choice 3's one regular file, at the path its option names.
ExecStart=/usr/libexec/monochromatic-git-policy-cli/<the installed file> monochromatic-git-policy-cli zfs-root boot-reconcile
```

```sh
sudo /usr/libexec/monochromatic-git-policy-cli/<the installed file> \
  monochromatic-git-policy-cli zfs-root adopt-worktree-parent \
  --checkout /home/user/Monochromatic --device 66306 --inode 1234567
```

```text
git monochromatic-git-policy-cli zfs-root --help      # a person, as the account
```

Probed:
the fixture decides every one of them (`keep/probe-dispatch-decided.log`, the words' cases).

Pros:

- names the route and the privilege,
  which is what the mode family is called everywhere else in this route's writing;
- no Git word it could be mistaken for,
  and long enough that running it is a deliberate act;
- one word for seven modes,
  so a reader who knows `service` knows this.

Cons:

- the longest of the three,
  and an adoption's line already carries three options.

### Option B: `mount`

```text
git monochromatic-git-policy-cli mount boot-reconcile
```

Pros:

- the shortest that still says the work,
  and the family is the clone mounts (the design's own name for it,
  `mount-git-worktree-clone-datasets`, and the route's `…-mounts` file names);
- a person types it more easily than the longer two.

Cons:

- names only the mounting modes,
  while `start-probe` and `holder-scan` are not mounts;
- `mount` is a word any later mode family would also want,
  so it claims a broad name for a narrow set.

### Option C: `root`

```text
git monochromatic-git-policy-cli root boot-reconcile
```

Pros:

- shortest, and honest about the privilege the modes need.

Cons:

- reads as a Git concept (a root commit,
  the root tree),
  so `git monochromatic-git-policy-cli root …` invites a wrong guess about what it does;
- says nothing about the ZFS route,
  so a person who does not already know the route learns nothing from it.

### Ranking of choice 2

The earlier text had no words form at all,
because it reached units and adoption by the modes' own file name,
so this choice is new and has no earlier ranking to change.
The answer is what raised it.

Ranking:
A > B > C.

A over B:
both are clear,
but `mount` names part of the family and would be claimed by any later mount-shaped mode,
while `zfs-root` names the whole of it,
and its length is paid once per unit line and once per adoption.

B over C:
both are shorter than A,
but `mount` at least says what the family does,
while `root` collides with a Git word's meaning.

The choice is separable from every other one here.
If the owner prefers a different word,
only the fixture's cases,
the unit text in step 19,
`doctor`'s printed commands and the release smoke's second arm change.

## Choice 3: what root's install holds

The review asked that root's regular file carry a mode's name,
not the wrapper's,
so that a root start under any other name fails with `No such file`.
The answer says units and adoption reach the modes by words.
Both can hold at once,
and four layouts fit them together.
Each was probed as the account in a throwaway `git init` repository
(`keep/probe-naming-as-the-account.log`, `keep/probe-records.log`),
with today's wrapper's bytes as the installed file,
because the dispatch itself is choice 1's and this choice is about names on disk.

### Option A: one regular file under the ZED hook's own name

```text
/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets   regular, 0755, root
/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets -> ../../../usr/libexec/... (symlink)
```

Units and adoption start the libexec file with the words of choice 2:

```ini
ExecStart=/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets monochromatic-git-policy-cli zfs-root boot-reconcile
```

Probed,
as the account,
with today's bytes
(so "Git" here is what those bytes do until choice 1's dispatch lands;
the decision itself is the fixture's,
in `keep/probe-dispatch-decided.log`):

- a typo of the name: `No such file or directory`, status 127;
- the modes' own name at that path: `No such file or directory`,
  so step 19's earlier line cannot run anything;
- a bare mode word: today `git: 'boot-reconcile' is not a git command`, status 1,
  and with the name rule it is `boot-reconcile`;
- the namespace word misspelled after the name: not possible,
  because the name rule answers first and the grammar reads the rest,
  so it is the grammar's own refusal with the modes listed,
  status 2;
- a stray copy of those bytes under any other name, no argument:
  `git`'s usage, status 1;
  with `boot-reconcile`: `git: 'boot-reconcile' is not a git command`, status 1.

Pros:

- every start of the one root-owned file is a root-mode start,
  whatever its words,
  so no word,
  typo,
  stale line or copy at that path reaches Git as root;
- the wrapper's own name is absent from root's folder,
  which is the review's request,
  and so is the modes' name,
  so both old spellings fail loudly;
- ZED follows the symlink and passes the link's name as `argv[0]`,
  which is exactly the name rule's input (probed last round and again here);
- one file to verify the digest of and one to replace on upgrade.

Cons:

- a file named `history_event-…` in `/usr/libexec` reads as a zedlet copy,
  and a unit starting it reads oddly until one knows the rule;
- the installed path that the modes' messages print is long;
- `SERVICE_INSTALL_PATH`, whose name means "where service calls start it",
  has to be renamed,
  because nothing about it is a service call any more.

### Option B: one regular file named for the modes

```text
/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets   regular, 0755, root
/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets -> ...        symlink
```

Probed:
the same starts,
and the difference is the line the plan already carries:

```text
$ /usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets boot-reconcile
git: 'boot-reconcile' is not a git command. See 'git --help'.
exit=1
```

and the run wrote a new record into `.git/cli-git-lock-holders/`
in the repository it stood in:

```text
the repository's record before: 1009476.39b89168284b59ad3ee00a9b5ddd2139
the repository's record after:  1015946.58262a42d6b22e68a696351011248757
```

Pros:

- the file's name,
  the version line and every refusal's program name agree;
- a unit's path reads for what it does.

Cons:

- a bare mode word at that path,
  which is exactly step 19's own line as the plan carries it today,
  runs the whole wrapper as root:
  Git,
  its configuration,
  its lock pass.
  That is the fail-open the review named;
- a start under that name with a word a person guessed (`boot-reconcil`)
  does the same,
  because neither that file's name nor a misspelled word selects a root mode.

### Option C: one regular file named for the wrapper

```text
/usr/libexec/monochromatic-git-policy-cli/monochromatic-git-policy-cli   regular, 0755, root
/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets -> ...     symlink
```

Probed:

- `monochromatic-git-policy-cli boot-reconcile`: `git: 'boot-reconcile' is not a git command`, status 1,
  and a new lock-holder record;
- `monochromatic-git-policy-cli` with no argument: `git`'s usage, status 1,
  and a new lock-holder record.

Pros:

- a unit's path reads as the wrapper a person knows.

Cons:

- any root start under that name is the whole wrapper:
  no root mode at all,
  and Git as root;
- it puts the wrapper's own bytes,
  reachable as a plain `git`,
  in root's install folder,
  which the review's finding is exactly about.

### Option D: a name per file, by copy or by hard link

The same names as choice 3's first three options,
each a copy or a hard link of the one program.

Pros:

- no link can dangle, with hard links.

Cons:

- a copy per name costs a copy per upgrade and can drift;
- a hard link made before an upgrade that writes a new file and renames it over the old stays on the previous release,
  so the zedlet would run old bytes without a word saying so,
  which is the failure this route can least afford.

### Ranking of choice 3

The earlier text's choice 2 ranked symlinks to a root-owned copy above hard links, copies
and a link into the account's install,
and the review changed the name half of it:
the regular file carries the ZED hook's name rather than the wrapper's,
which is this choice's option A.
The ranking underneath is unchanged,
so options C and D keep the earlier reasons,
and the review's version and ancestor checks are what make a link that a package manager replaces visible.

Ranking:
A > B > C > D.

A over B:
both keep one file,
but A's only root-owned file is a root-mode start whatever its words,
so a typo,
a stale line or a copied command at that path is a refusal or a mode and never Git as root,
while B's is Git as root for a bare word.
The confusion A costs is a name a reader has to be told about once,
and the units' and adoption's lines are generated from the constants and printed with their values anyway.

B over C:
B's file at least names the modes it runs,
and a start under it with the words works;
C's file is the wrapper under the wrapper's name,
so the fail-open covers every start under it,
words or not.

C over D:
C has one file and one upgrade step;
D multiplies both and adds a drift a person cannot see.

What none of them prevents,
and this design does not pretend otherwise:
a copy of the wrapper's bytes somewhere else,
started as root under a name of its own,
runs the whole wrapper.
An account that can write root's install folder can replace any of these files,
which is why `doctor` checks owner,
mode,
the ancestor folders and the version the installed file prints.

The choice is separable from the others.
If the owner prefers B,
the unit lines keep one shape and only the path in them changes.

## Choice 4: where the dispatch sits

What a placement costs root is every start-time action after it that runs.

### Every start-time action of today's wrapper

In the order the code takes them,
with what a root-mode start does at choice 4's option A for each:

1.  `args_os` collection in `main`.
    Read:
    a root mode needs it,
    and the name and the words are in it.
2.  The one-name check and the words check (this design's).
    Read:
    they are the decision.
3.  `service_main::is_service_start`,
    which reads the arguments only.
    Skipped:
    the name check and the words check answer first.
4.  `panic_notice::install_payload_free_panic_hook`.
    Installed for a root mode, as "The panic hook a root mode installs" recommends.
5.  `vars_os` collection.
    Skipped:
    a root mode reads no variable through this collection.
6.  `runtime_probe::is_requested`,
    a test-only variable.
    Skipped.
7.  `hooks_off::activate`,
    a process-wide switch.
    Skipped.
8.  `abandoned_locks::meet_the_locks_of_the_working_directory`:
    the working directory,
    `GIT_DIR`,
    one ancestor walk for a Git directory,
    then a record written into the common Git directory
    and any lock a proof says is abandoned removed.
    Skipped,
    and this is the one that writes.
9.  `process_resolution_inputs`:
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
    which reads
    `monochromatic-git-policy-cli.config.jsonc`.
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
(`keep/probe-records.log`):

```text
the repository's record before: 1009328.b3e9ce10ab6649811d16dbaa130501b4
a bare Git word through the wrapper:                        exit=1  records now: 1009476.39b89168284b59ad3ee00a9b5ddd2139
the root-mode words through the wrapper (today):             exit=2  records now: 1009500.29dd7ef878268144d370612ea345d0dd
the service mode with a word it does not know:               exit=3  records now: 1009510.f1bdf38644de5ede9efb3c802ffcd7eb
```

Each start wrote a record named for its process and removed the dead one,
except the service mode,
which wrote nothing:
that is the main-level placement,
measured.

Started exactly the way ZED starts a zedlet,
with `/` as the working directory,
today's wrapper exits 1 and writes nothing,
because there is no repository to write into (`keep/probe-zed-replay.log`).
The same start with the caller's working directory kept wrote one record,
and that variant is not a ZED start.

### Option A: the first statements of `main`

The two checks and the root-mode arm of "The dispatch, in the entry",
before the service check,
before the panic hook,
and before the environment.

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
  the words,
  the service's words,
  each reading arguments only.

### Option B: after the panic hook and the environment

Pros:

- a panic in a root mode would print the wrapper's payload-free message without a second installation;
- the environment is already collected if a mode ever wants it.

Cons:

- `main` collects the whole environment for a start that reads none of it;
- no probe shows any benefit over A,
  and the hook can be installed inside the arm instead,
  which is what "The panic hook a root mode installs" recommends.

### Option C: at the top of `entry::run_process`

Pros:

- one place decides everything,
  and `main` stays as thin as it is.

Cons:

- the runtime probe's variable is read first,
  and `hooks_off::activate` and the lock pass follow,
  so the lock pass,
  its record write,
  and its possible lock removal run before root's mode;
- the entry would hand a start name into the library for a decision the library could not otherwise need.

### Option D: inside `plan_invocation`

A check made here,
where the arguments are already planned,
costs every action before it.

Pros:

- nothing of `main` or `run_process` changes shape,
  so no other mode's start moves.

Cons:

- the lock pass,
  the resolution and the started-as-Git check all run first,
  so root writes into whatever repository holds the working directory;
- `git_facts`,
  the configuration read and the forwarded Git are one branch away,
  and `keep/probe-naming-as-the-account.log` shows what the management arm's refusal costs:
  a new lock-holder record per start.

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

The review leaves the earlier order as it was and narrows the reason for A over C and D:
under ZED the working directory is `/`,
so the lock pass finds no repository,
and the cost C and D carry is a person's start from inside a checkout,
such as an adoption.
It also adds the panic-hook question,
answered inside A.

Ranking:
A > B > C > D.

A over B:
B collects an environment nothing reads,
and installing the hook inside the arm is one line,
so B's only gain is gone.

B over C:
C adds the lock pass,
which writes a lock-holder record into the repository whose directory a start
happens to be standing in,
which `keep/probe-records.log` shows happening on a start whose mode never runs.

C over D:
D adds the configuration read and a Git start before the mode,
and it is the only option that leaves the mode word indistinguishable from a Git subcommand.

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
$ /usr/libexec/monochromatic-git-policy-cli/<the installed file> --version
mount-git-worktree-clone-datasets 0.6.1-alpha.46
```

Pros:

- every existing check,
  test and refusal text keeps working,
  and a check can tell "this build has the root modes" from the wrapper's own banner;
- the name describes the mode family rather than a file,
  which stays true however the file is installed.

Cons:

- a person reads a name no command has,
  so every refusal and the version line must also print the words and the installed path.

### Option B: print the wrapper's name and a modes line

```text
monochromatic-git-policy-cli 0.6.1-alpha.46
root modes: boot-reconcile, shutdown-unmount, start-probe, holder-scan, adopt-worktree-parent, the sweep
```

Pros:

- the line names the program a person runs.

Cons:

- it collides with the wrapper's own `git --version` banner,
  which already holds `monochromatic-git-policy-cli <version>,`,
  so a check on the installed file's line could no longer tell which program answered;
- `native:install`, `doctor` and the smoke all need new text,
  and the modes' refusals need a new program name.

### Ranking of choice 5

This choice is new:
the answer makes the modes reachable through one name and the words,
so a file no longer carries the program's name,
and the earlier text never had to choose what `--version` says.

Ranking:
A > B.

A over B:
A keeps one unique line that proves the modes answered,
which is the whole reason `native:install` and `doctor` read it,
and the cost is one added sentence in the refusals naming the words and the path.

## Choice 6: whether the account's `PATH` keeps a short name

Today a person can type `mount-git-worktree-clone-datasets adopt-worktree-parent …`
because the toolchain installs the second program.
After the change the account's toolchain installs one program,
and a root mode needs the words in any case.

### Option A: the installed path and the words, printed by `doctor`

```text
sudo /usr/libexec/monochromatic-git-policy-cli/<the installed file> \
  monochromatic-git-policy-cli zfs-root adopt-worktree-parent \
  --checkout /home/user/Monochromatic --device 66306 --inode 1234567
```

and a person's read-only questions through the account's own wrapper:

```text
git monochromatic-git-policy-cli zfs-root --help
```

`--help` prints the mode words,
the words' spelling,
and the installed path;
and `doctor` prints the adoption line with the checkout's device and inode filled in,
beside the three unit lines the setup writes.

Pros:

- no change to the toolchain,
  whose plan expresses one post-install link per tool
  (`mpm-shadow`'s `postinstall.link` takes `sourceBin`, `linkName` and `linkDir`,
  and refuses any other key);
- the path a person types is the file root's units run,
  so `--version` describes the bytes that will act;
- `doctor` already prints concrete commands it never runs.

Cons:

- the long command,
  and every refusal inside the modes that names a follow-up command must be rewritten to it.

### Option B: the toolchain links a short name beside `git`

Pros:

- shorter to type.

Cons:

- the toolchain's schema cannot express it today,
  so another package changes and releases first;
- the link would point at the account's copy,
  so `--version` would describe a different file from the one root runs;
- a short name linked to an account-writable file could be run as root by a `sudo` that preserves `PATH`,
  which is one more path to an escalation this design does not need;
- it still needs the words,
  so it saves typing, not thinking.

### Ranking of choice 6

The earlier ranking stands,
and the review adds one reason to it:
a short name linked to the account's copy would let a `sudo` that preserves `PATH` run an account-writable file as root,
which is one more path to an escalation this design does not need.

Ranking:
A > B.

A over B:
A needs no other package,
keeps one file behind every name,
and its command is printed complete with the values read;
B's savings are typing only,
and it moves the thing a person runs away from the thing root runs.
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
CLI_GIT_TEST_ONLY_START_PROBE=disabled <the installed file> monochromatic-git-policy-cli zfs-root start-probe
```

and the record's verdict is skipped with its skip reason `test-switch`.
A unit's or a person's start that sets none of them behaves as it does today.

Pros:

- the modes' controls keep running against the release build,
  which is what makes their guest arms evidence;
- the direction is fail-safe:
  a set switch at worst skips a record,
  and a verbosity variable changes lines.

Cons:

- a privileged program has switches nobody but a control needs.

### Option B: compile them out of the release

Pros:

- the release binary holds no switch a test names.

Cons:

- a second build shape,
  so the guest arms would test a different binary from the released one;
- the crate already has one such precedent (`--features measurement-routes`),
  and a second would need its own tasks and lint passes,
  for no measured behaviour at root.

### Ranking of choice 7

The earlier text said a root mode reads no variable at all,
and the review found five it does read,
so this choice is new.

Ranking:
A > B.

A over B:
keeping them costs nothing a probe can show,
and removing them breaks the one property that makes the guest arms evidence,
that the binary under test is the binary released.

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
  names the words and the installed path instead
  (`worktree_clone_mount_handler/handler_mode_arguments.rs`,
  `mode_awaiting_its_step.rs`,
  `sweep_preconditions.rs`,
  and `worktree_clone_mount/clone_slot_walk_refusals.rs:194`);
- the `--help` paragraph that claims both install paths exist says who creates them and when,
  which is M1's finding;
- a root mode no longer passes the lock sweep,
  so a mode started from inside a repository writes nothing into that repository
  (measured in "Every start-time action of today's wrapper").

Lost:

- the smaller program:
  3,354,480 bytes against 28,957,504 at one commit,
  8.63 times.
- its cold first start:
  the wrapper's cold start took a median 9.78 ms and 420 major page faults,
  the handler's 3.57 ms and 72,
  measured on this host with each file's own pages evicted before every run,
  arms interleaved,
  25 rounds
  (`keep/wrapper-against-handler.txt`).
  The paired difference is the finding:
  the wrapper was a median 6.04 ms slower than the handler,
  and in the slowest round it was 13.16 ms slower while in the fastest it was 2.61 ms faster.
  That pair is read against two controls measured the same way:
  the same file against itself gave a paired median of 1.46 ms,
  with rounds between 9.89 ms in one arm's favour and 14.24 ms in the other's,
  over 21 rounds;
  and `/bin/true` against the wrapper moved the pair by a median 7.65 ms,
  between 6.30 ms and 14.00 ms,
  so the method does show a difference of this size.
  So a cold root start pays about six milliseconds more than the second program did,
  and 348 more pages read,
  for every ZFS event and every boot,
  and a waiting caller waits on the sweep's receipt.
  The absolute medians are context, not the finding:
  the wrapper's own median moved between 9.78 ms and 13.51 ms across this round's arms,
  so only the interleaved pair is comparable.
  The start-path cost is the decision's accepted price;
  if it ever bites,
  the fix is a smaller early path,
  not a second program.
- a release that can pin or roll back the root side separately from the wrapper,
  which is the accepted cost of the placement the project already chose.

## What changes, and in which step

### One commit: the release ships one program

These cannot be separated,
because the guard reads the manifest and the release lists in one test,
and `toolchain:lock:check` compares the plan and the lock offline:

- `package/git-policy/cli/Cargo.toml`:
  remove the second `[[bin]]` (`:46` to `:48`) and its comment (`:39` to `:45`),
  and correct the two comments that count executables
  (`:44` and `:54`);
- `package/git-policy/cli/src/native/mount_git_worktree_clone_datasets/main.rs`:
  deleted;
- `package/git-policy/cli/src/native/lib.rs`:
  the module's doc paragraph,
  which says "second executable";
- `src/native/source_guard/release_bins_tests.rs:31`:
  the released list and its doc;
- `bin/native-release-bins.unit.test.ts`:
  the doc comment;
  its planted-second-bin control still requires four disagreements and needs no change of logic;
- `package/crate-release/cli/releasable-crates.json:30`:
  `checkBins` holds one name;
  its `pinFiles` holds only the fuzz crate,
  so the decision's `pinFiles` row has nothing left to undo;
- `.github/workflows/cargo-publish.yml`:
  the job comment (`:2432` to `:2442`),
  the `--bin` flags (`:2666`),
  the smoke step (`:2669` to `:2680`),
  and the second `bin=` copy line (`:2697` to `:2698`);
- `toolchain.plan.jsonc`:
  `bins` holds one name,
  and `toolchain.lock.json` takes it through the tool that writes the lock offline,
  which is the command `toolchain:lock:check` runs without `--frozen-lockfile`,
  never by hand;
- `package/git-policy/cli/mise.toml`:
  the descriptions of `native:build` (`:98`),
  `native:build:release` (`:102`) and `build` (`:158`) that count two executables,
  and `native:install`'s description (`:88`), which names both version checks the install runs;
- `README.md`:
  the release paragraph (`:11`),
  the install section that says one archive carries two executables (`:1865`, `:7967`),
  the install note that refuses until a release carries the second program (`:7117`, `:7124`),
  and the source-layout paragraph;
- `package/git-policy/cli/SPEC.md:249`:
  "## The two executables and shipped policies" returns to
  "## Single executable and shipped policies" and says how the root modes are reached;
  "### The start-time ZFS compatibility gate" and "The start probe beside the gate"
  name the mode family through the one name or the words instead of a second program;
- `package/git-policy/cli/doc/step-5-second-executable-2026-11-20.md`:
  a note under its status that the second program was removed by this decision,
  with the date,
  keeping the record's own text.

### The same commit: the entry, the guards, and every test that started the second program

Deleting the `[[bin]]` breaks these at compile time,
so they land with the release lists:

- `src/native/main.rs`:
  the two checks and the root-mode arm, as section "The dispatch, in the entry" writes them,
  the comment at `:53` corrected to name the one name and the words,
  and the hook installed inside the arm;
- `src/native/worktree_clone_mount_handler/root_mode_names.rs` (new):
  the three constants and the predicate,
  with its sidecar control file,
  and the folder's top doc listing it first;
- `src/native/source_guard/hook_self_invocation_tests.rs`:
  the test renamed,
  one word out of its forbidden list,
  and one assertion replaced,
  as the guard replay states;
- `src/native/source_guard/no_self_invocation_rules_tests.rs`:
  the `args_os()` needle and the one `ALLOWED` entry;
- `src/native/binary/binary_external_hook_tests.rs`:
  the doc comment of `a_start_name_that_used_to_be_an_entry_behaves_as_git` says the one exception,
  and its body stays as it is;
- `src/native/testing/root_mode_alias.rs` (new):
  one writer of the installed name in a fixture folder,
  beside `testing/fixture_executable.rs`,
  so each test that starts a mode names one link and not one copy of the rule;
- the six test files "The change list is incomplete and contradicts itself" names:
  `CARGO_BIN_EXE_monochromatic-git-policy-cli` through the fixture link;
- `binary_service_mode_tests.rs:6`:
  the comment that counts two executables;
- `e2e/start-probe-guest.ts:42`:
  `HANDLER` becomes the wrapper pushed into the guest plus the installed-name link its commands run,
  and every `${HANDLER} start-probe` line keeps its shape;
- `e2e/mount-git-worktree-clone-datasets-guest.ts`:
  its guest build loses the second program (its tarball excludes `target`,
  so the guest compiles what it runs at `:326` to `:328`),
  and the guest creates the installed-name link before the adoption arm runs.

### The same commit: what `native:install` and the release smoke check instead

`native:install`'s handler check becomes the words' check,
which is the contradiction's resolution:

```ts
// bin/native-install.ts, where the handler's check stands today
const modesReported = (await runGateCommandOrThrow({
  program: gitProgram,
  args: ['monochromatic-git-policy-cli', 'zfs-root', '--version'],
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

The release smoke gains the two arms,
so a release cannot ship with either dispatch broken:

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
  step 5 is rewritten as the undo,
  keeps its release list and loses the second executable from it,
  and its "What it builds" gains the one name and the words;
  step 6's sentence "the gate both binaries run" becomes one gate;
  step 18's `doctor` gains the installation check "Steps 18 and 19" states;
  step 19 names the one file, the symlink and the words,
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
  the `argv[0]` bullet gains the one exception,
  citing the start-name record;
- `doc/planning/cli-git-resident-service.md`,
  the sentence that rejects a name-based dispatch:
  it stays true for the service and gains "except the ZED hook's installed name".

### Steps 18 and 19

They land in their own steps,
and nothing waits on them for the release to hold one program:

- step 18's `doctor` gains, in one place:
  the installed file exists at the root-mode path,
  is a regular file through `stat` (so the zedlet's symlink passes with its target's values),
  is root's,
  has its owner execute bit,
  and no group or other write bit;
  the zedlet's name is one ZED's scan accepts (the class prefix and a non-letter after it);
  every ancestor folder from `/` down exists,
  is a directory,
  is root's and is not group- or other-writable,
  and a missing one prints the `install -d -o root -g root -m 0755` command that creates it
  (`/usr/libexec` is missing on this host today);
  the installed file's `--version` names the same version as `WRAPPER_VERSION`,
  the build `doctor` runs from,
  and a mismatch prints both and the lock row's version;
  its findings say what a broken or missing name costs (a sweep that never runs)
  and print the command that recreates it.
  The tree's `PLACEHOLDER_EXECUTABLE_PATH` and `PLACEHOLDER_ZEDLET_PATH` become the settled constants,
  and the zedlet's placeholder name (`history_event-monochromatic_git_policy_cli`) becomes the installed one.
- step 19 prints the one file and its name,
  and its guest arm starts a unit by that path with the words and reads `--version` through it.
  It says where the bytes come from:
  the lock row's `artifacts.<target>.url` and `sha256`,
  verified before the file is installed,
  never a copy of the account's `~/.cache/monochromatic-git-policy-cli-bin/`.

### What does not change

- the crate links no OpenZFS library and starts where OpenZFS is absent;
- the modes' grammar,
  exit statuses,
  gate,
  probe and records;
- the ZED hook's placement,
  class prefix and synchronous or asynchronous choice,
  and its installed path's spelling;
- the event descriptor ZED puts on 3:
  the entry touches no descriptor,
  and no mode reads it today;
- the archive's layout rule:
  one declared `[[bin]]`,
  one `bin=` line,
  one file;
- the configuration's path.

## What the probes showed, and what they did not

### Shown

- ZED's exec shape,
  `argv[0]`,
  argument count,
  working directory,
  descriptors and built environment,
  by a replay of the source's own child step
  (`keep/probe-zed-replay.log`),
  and `current_exe` resolving a symlink start away from the installed name;
- each of the decided rule's cases deciding as section "Choice 1, answered" states,
  including five near-miss names,
  a trailing slash,
  a caller-chosen `argv[0]`,
  an empty `argv[0]` and `execv` with `argv = {NULL}`
  (`keep/probe-dispatch-decided.log`);
- both source guards' predicates against today's `main.rs`,
  the proposal's `main`,
  the review's V2 and a two-name family,
  so each guard's change is the assertion the replay names
  (`keep/guard-replay.log`);
- three install layouts,
  a path typo,
  step 19's own line,
  a namespace typo and a stray copy,
  as the account in a throwaway repository
  (`keep/probe-naming-as-the-account.log`);
- which starts write a lock-holder record,
  and that a main-level dispatch writes none
  (`keep/probe-records.log`);
- the wrapper's cold start against the handler's,
  with the method's spread and a positive control measured first
  (`keep/spread-control.txt`, `keep/positive-control.txt`, `keep/wrapper-against-handler.txt`);
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
  the earlier round's probe used `systemd-run --user`,
  so a system manager's `argv[0]` is read from systemd's documented behaviour and not measured here;
- any mode touching ZFS:
  this round ran no guest arm,
  so no request,
  mount,
  unmount or adoption was exercised;
- the cold start in a guest:
  two guests were created and destroyed through mvm,
  and every route for the bytes into the guest was refused by the guest's SELinux domain for the guest agent.
  The four refusals,
  in order,
  are in `keep/state-so-far.md`:
  a read of the virtiofs share,
  a connection to the host on three ports,
  `getenforce` and `setenforce 0`,
  and `guest-file-open` for `/root`,
  followed by `Permission denied` on executing a file under `/var/tmp`.
  The glibc floor is not the blocker:
  the wrapper needs at most `GLIBC_2.39` and the handler `GLIBC_2.34`, and the guest has 2.42;
- whether AppArmor's policy on this host confines the one name:
  no profile exists yet,
  and the rules file records an unexpected permission error rather than routing around it;
- whether `doctor`'s future installation check catches a link that a package manager renames instead of deleting;
- the release smoke arm itself:
  it is written here and not run,
  because this round changes no workflow.

## The choices this proposal puts to the owner

Each is separable,
and each is ranked in its own section.
Item 1 is the answer this design carries out;
items 2 to 7 are the choices.

1.  The one name and the words:
    the ZED hook's installed file name alone selects a root mode,
    and `monochromatic-git-policy-cli zfs-root <word>` reaches it from a unit,
    an adoption or a person (choice 1).
2.  The group word the units and people spell:
    `zfs-root` (choice 2, A),
    `mount` (B),
    or `root` (C).
3.  What root's install holds:
    one file under the ZED hook's name and nothing under the wrapper's (choice 3, A),
    one file named for the modes (B),
    one file named for the wrapper (C),
    or a copy or hard link per name (D).
4.  Where the dispatch sits:
    the first statements of `main` (choice 4, A),
    after the panic hook and the environment (B),
    at the top of `run_process` (C),
    or inside `plan_invocation` (D),
    with the panic hook installed inside the arm (the recommendation).
5.  What `--version` and the refusals say:
    the modes' own name (choice 5, A),
    or the wrapper's name and a modes line (B).
6.  Whether the account's `PATH` keeps a short name:
    the installed path and the words, printed by `doctor` (choice 6, A),
    or a second toolchain link (B).
7.  Whether the release keeps the modes' test-only variables:
    keep them (choice 7, A),
    or compile them out (B).

This proposal's own ranking is A in every one of them.

## Appendix: where the probe transcripts are

Round folder `/home/user/temp/agent/zfs-root-modes-after-the-start-name-answer-2026-11-24/`:

- `keep/state-so-far.md`:
  the build,
  the guest refusals with their exact text,
  and what is left;
- `keep/probe-dispatch-decided.log`:
  every start the rule decides, with five near-miss names, a trailing slash, an empty name
  and `execv` with `argv = {NULL}`,
  from `keep/fixture_dispatch_decided_rule.rs` and `keep/exec_empty_argv.c`;
- `keep/guard-replay.log`:
  both guards against five `main.rs` texts,
  from `keep/guard_replay.rs`, `keep/proposed_main.rs`, `keep/v2_main.rs`, `keep/family_main.rs`
  and `keep/no_start_name_main.rs`;
- `keep/probe-naming-as-the-account.log`:
  the three layouts and the stray copy,
  as the account;
- `keep/probe-records.log`:
  which starts write into the repository they stand in;
- `keep/probe-zed-replay.log`:
  ZED's child step replayed with `chdir("/")`,
  from `keep/zed_replay.c` and `keep/replay_fixture.rs`,
  and the real wrapper started both ways;
- `keep/spread-control.txt`, `keep/positive-control.txt`, `keep/wrapper-against-handler.txt`:
  the cold-start runs, from `keep/coldstart.c`;
- the earlier round's `keep/zed-source/`, under
  `/home/user/temp/agent/zfs-root-modes-in-the-one-wrapper-executable-design-2026-11-24/`:
  the OpenZFS `zfs-2.4.4` files this design's ZED facts are read from.

The round's `scratch/` folder,
where the fixtures,
their links,
the throwaway repository and the two guests' work lived,
was removed when the round ended;
the kept sources rebuild it.
