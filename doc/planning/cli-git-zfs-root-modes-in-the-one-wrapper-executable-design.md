# The ZFS route's root modes as modes of the one wrapper executable

## Status

A proposal, put to the owner.
It is not a decision,
and nothing here is approved or final.
It records what the project decided in
[the one-program record](../decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md),
designs how that decision is carried out,
and shows each option with a probe run in this round's scratch folder.
The decision record and the handover are untouched by this round.

## What the decision requires

The record answers "One program only, ever",
and its "What follows" is the requirement this design starts from:

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
  are this design's to propose.

What makes the last point the whole of the problem:
the wrapper is the `git` a person runs,
so its arguments are Git's arguments,
and an argument alone cannot mean "root mode" without taking a word away from Git.
And ZED hands a zedlet no arguments at all.

## What the code holds today

### The two entries

`package/git-policy/cli/src/native/main.rs` is the wrapper's entry.
It collects the arguments,
answers the service mode when the leading three words are
`monochromatic-git-policy-cli service run`
(`invocation::service_main::is_service_start`),
installs the payload-free panic hook,
collects the environment,
and hands everything to `invocation::entry::run_process`,
then waits for the log lines it handed to their own threads
(`end_after_the_log`).

`package/git-policy/cli/src/native/mount_git_worktree_clone_datasets/main.rs` is the second program's entry:
`args_os().skip(1)`,
`worktree_clone_mount_handler::run`,
the same two log waits,
and the exit.
Its grammar (`worktree_clone_mount_handler/handler_mode_arguments.rs`) takes
one leading word out of
`boot-reconcile`,
`start-probe`,
`shutdown-unmount`,
`adopt-worktree-parent`,
`--help` and `--version`;
an empty argument list is `AttachingSweep`;
anything else is a refusal that names the word and the modes.
`handler_mode_dispatch.rs` gives every mode the same two answers,
a line and status 0,
or a refusal line and status 2,
and runs the compatibility gate before a mode that will make a request.
`handler_install_paths.rs` holds the program's name,
the two installed paths,
and the configuration path.

The service mode is the precedent this design follows:
a mode chosen inside `main`,
before the panic hook,
before the environment is collected,
and before anything of the wrapper's own start runs.

### The release's lists, and what reads them

`package/git-policy/cli/Cargo.toml` declares two `[[bin]]` tables,
and its `[package.metadata.binstall]` table names `{ bin }`,
so one archive unpacks both programs.
Every list that counts them must agree with the manifest,
and `bin/native-release-bins.unit.test.ts` is the guard that says so:
the release job's `cargo build --bin` flags and its `bin=` copy lines in
`.github/workflows/cargo-publish.yml`,
`package/git-policy/cli/releasable-crates.json`'s `checkBins` for the crate,
and `toolchain.lock.json`'s `bins` row for `cargo:monochromatic-git-policy-cli`.
`toolchain.plan.jsonc` holds the same row as intent,
and `toolchain.lock.json` is written from the plan:
`mpm-shadow`'s update keeps every other row as it is and gives it "the plan's current bins",
so a changed plan row moves the lock row through the tool, offline (`toolchain:lock:check` runs
`install --lockfile-only --frozen-lockfile`).
`bin/native-install-steps.ts` holds `handlerProgram` and `handlerVersionLine`,
and `native:install` refuses unless `mount-git-worktree-clone-datasets --version` names the release;
`bin/stand-in-install-programs-for-tests.ts` carries a stand-in for it.

Four Rust test files start the second program by `env!("CARGO_BIN_EXE_mount-git-worktree-clone-datasets")`:
`binary/binary_mount_git_worktree_clone_datasets_tests.rs`,
`binary/binary_zfs_adoption_guest_tests.rs`,
`binary/binary_zfs_sweep_guest_tests.rs`
and `binary/binary_zfs_unmount_guest_tests.rs`.
`e2e/start-probe-guest.ts` names the built binary in its `HANDLER` constant,
and its guest commands run `${HANDLER} start-probe`.

The wrapper already carries the ZFS code.
`Cargo.toml` depends on
`monochromatic-zfs-request-to-openzfs-kernel-device` for the typed `/dev/zfs` requests,
and the wrapper runs the same compatibility gate before its first request,
which plan step 7 added.
So a root mode inside the wrapper adds no dependency and no library:
the decision's "links no OpenZFS library and starts where OpenZFS is absent" check stays as it is.
What the second program saved was code that runs, not code that is linked.

## What ZED and the units hand a start

### ZED runs a zedlet by path, with no arguments

From OpenZFS `zfs-2.4.4`,
`cmd/zed/zed_exec.c`:

- line 141 builds `path` as `"<zedlet_dir>/<prog>"`;
- lines 161 to 166 point descriptors 0, 1 and 2 at `/dev/null`
  when ZED runs in the foreground,
  which the host's unit does (`ExecStart=/usr/bin/zed -F`);
- line 167 puts the event descriptor on 3;
- line 168 is `execle(path, prog, NULL, env)`:
  the argument vector is the program name alone,
  and `argv[0]` is `prog`, the bare file name with no directory;
- line 421 matches a zedlet against the event class and requires a non-letter straight after it;
- line 339 decides a synchronous zedlet by `-sync-` right after the class prefix;
- `cmd/zed/zed_conf.c:378` to `:402` accept a file only when `stat` succeeds,
  it is a regular file,
  root owns it (unless `-f`),
  its owner execute bit is set,
  and it is not group- or other-writable;
- `cmd/zed/zed_conf.c:51` caps ZED at 16 zedlet processes.

`cmd/zed/zed_event.c:1013` to `:1021` build the child's whole environment,
and it is not inherited:
the event's nvlist,
a restricted list (`IFS`, `PATH` at the standard path, `ZDB`, `ZED`, `ZFS`, `ZINJECT`, `ZPOOL`,
`ZFS_ALIAS`, `ZFS_VERSION`, `ZFS_RELEASE`),
`TZ` when the parent has it,
`ZED_PID`,
`ZED_ZEDLET_DIR`,
`ZEVENT_SUBCLASS`
and the `ZEVENT_TIME_*` values.

The replay probe reproduced exactly that start for a fixture dispatcher (transcript in
`keep/probe-zed-replay.log`):

```text
PROBE argv0="history_event-mount_git_worktree_clone_datasets"
PROBE argc=0
PROBE env PATH="/usr/bin:/bin:/usr/sbin:/sbin"
PROBE env HOME=<unset>
PROBE fd0=open:true target:/dev/null
PROBE fd1=open:true target:/dev/null
PROBE fd2=open:true target:/dev/null
PROBE fd3=open:true target:.../install/zfs-zed.d/zevent.nvlist
```

Consequences a design cannot talk its way out of:

- a mode selector that needs an argument cannot be reached by the ZED hook,
  because ZED passes none;
- `argv[0]` is the bare file name,
  so any name-based rule must read the last path component and must work with no slash at all;
- ZED offers no way to add a variable of this repository's own to that environment,
  so a variable cannot select a mode.

`current_exe()` is not the name either:
under a symlink it answers the link's target,
which the probe printed
(`PROBE current_exe=.../bin/dispatch-rule_name_and_words`
after a start through `.../install/zfs-zed.d/history_event-...`).

### systemd runs what `ExecStart` spells

A transient unit started with
`ExecStart=/home/.../install/libexec/mount-git-worktree-clone-datasets boot-reconcile`
gave the program `argv[0]` as the full path as written,
and `boot-reconcile` as its only argument (transcript in `keep/probe-systemd-unit-start.log`):

```text
PROBE argv0="/home/.../install/libexec/mount-git-worktree-clone-datasets"
PROBE basename=mount-git-worktree-clone-datasets
PROBE decided=root:boot-reconcile
```

So a unit may pass a mode word,
and its `argv[0]` is a path,
which the same basename rule reads.

### What each launcher names

The launchers that start these modes each spell one of these lines, and choice 1 decides what they spell.
The lines in this section are the ones the design and the plan already carry,
with the program name this design keeps:

```ini
# /etc/systemd/system/reconcile-git-worktree-clone-mounts.service
ExecStartPre=-/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets start-probe
ExecStart=/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets boot-reconcile
ExecReload=/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets boot-reconcile
ExecStopPost=/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets shutdown-unmount
```

```sh
# the administrator's adoption, printed by `doctor` and never run by it
/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets \
  adopt-worktree-parent --checkout /home/user/Monochromatic --device <count> --inode <count>
```

```text
# /etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets, run by ZED with no argument
/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets
```

ZED's own start is the only one that cannot carry a word,
and it needs none: no argument is the sweep.
The `doctor` command a person reads is
`git monochromatic-git-policy-cli doctor`,
and the root commands it prints are the unit's lines and the adoption's line,
complete with the values it read.

### What that leaves for a mode selector

A start can be recognised by its arguments or by its name,
and ZED forces a name for the sweep start.
The rest of this design is what follows from that.

The zedlet's own file name is not free:
ZED matches a zedlet against the event class and requires a non-letter straight after it
(`cmd/zed/zed_exec.c:421`),
so the installed name stays
`history_event-mount_git_worktree_clone_datasets`
as the design already says.
A second name is therefore needed for the units and for a person,
which is the name the design already installs under `/usr/libexec`.

## Choice 1: how a start reaches a root mode

### Option A: an installed name selects the root-mode family, a word selects the mode

Two installed names belong to the family:
`mount-git-worktree-clone-datasets`
and
`history_event-mount_git_worktree_clone_datasets`.
A start whose `argv[0]` ends in one of them is a root-mode start.
The first word then names the mode,
and no word at all is the sweep,
which is what ZED's no-argument start needs.
Every other name,
including `git` and `monochromatic-git-policy-cli`,
runs the wrapper unchanged.

Probed (transcripts in `keep/probe-a-name-and-words.log`):

- the full path with no argument decides `root:attaching-sweep`;
- the full path with `boot-reconcile` decides `root:boot-reconcile`;
- a `PATH` lookup gives `argv0=mount-git-worktree-clone-datasets` and decides on its word;
- a relative `./mount-git-worktree-clone-datasets start-probe` decides `root:start-probe`;
- `some-other-name`, `monochromatic-git-policy-cli` and `git` decide the wrapper;
- the ZED replay decides `root:attaching-sweep`;
- a systemd unit decides on its word.

Pros:

- ZED's no-argument start reaches the sweep with no second file and no interpreter;
- the units and the ZED hook keep the exact names the design already prints,
  so step 19's unit text and the zedlet name do not change;
- one program, one file in the archive,
  and the two installed paths become two names of that file;
- the modes keep today's grammar and exit statuses, so the modes' own tests survive once their
  texts name the installed path.

Cons:

- the entry becomes sensitive to `argv[0]`,
  which no part of the wrapper reads today;
- a link that is missing or broken silently makes the zedlet do nothing
  (probed: ZED's `stat` fails on a broken link and logs one warning),
  so `doctor` has to check the two names,
  which is what "Steps 18 and 19" asks of it;
- a person who starts the wrapper by its own name with a mode word gets the wrapper,
  not the mode.

### Option B: namespaced words, with an installed zedlet script

The modes are reached by words inside the wrapper's own namespace,
for example
`monochromatic-git-policy-cli clone-mounts boot-reconcile`,
which keeps Git's word space intact because the wrapper already claims the word
`monochromatic-git-policy-cli`.
The zedlet at `/etc/zfs/zed.d/history_event-mount_git_worktree_clone_datasets`
is then a root-owned shell script that execs the installed wrapper with the sweep's word,
which is the distribution's own zedlet idiom (`/etc/zfs/zed.d` holds scripts).

Probed: a script that execs the wrapper by absolute path gives the wrapper the path as `argv[0]`
and the word as its argument
(`keep/probe-zed-replay.log`, the last section),
so the arrangement works.
Under the words-only rule alone, which reads no name,
the same replay decided the wrapper instead of the sweep.

Pros:

- no name sensitivity in the entry at all;
- a person can type the words from anywhere the wrapper is on `PATH`.

Cons:

- root executes a second file,
  which is a program by any reading a person would give the word,
  and the decision's point was that root should not;
- the script is a second thing to install, to keep in step, and to explain in `--help`,
  and the alpha.45 review already found the `--help` naming install paths that do not exist
  (`~/temp/agent/release-pass-wrapper-alpha-45-2026-11-23/zfs-route-reach/keep/review.md`, M1);
- a script adds an interpreter to root's path and its own failure modes.

### Option C: one installed name per mode, no words

`/usr/libexec/.../boot-reconcile`,
`/usr/libexec/.../shutdown-unmount`,
`/usr/libexec/.../start-probe`,
`/usr/libexec/.../adopt-worktree-parent`
and the sweep's name,
each a link to the one program,
with the name alone selecting the mode.

Probed: an applet name decides the mode with no arguments
(`keep/probe-a-name-and-words.log` for the applet rule, and `keep/probe-zed-replay.log`
for the replay under the installed zedlet name).

Pros:

- one mechanism only, and the same one ZED forces;
- no argument grammar to keep.

Cons:

- the modes stop being words a person can read in a help text,
  and a mistyped name is `No such file` instead of a refusal that lists the modes;
- the installed set grows from two names to five,
  each of which `doctor` must check and step 19 must create;
- `--version` and `--help` have no natural place,
  because they are not modes.

### Option D: a variable in the environment

A unit can set `Environment=`,
so a variable would reach a unit's start.
It cannot reach ZED's:
`cmd/zed/zed_event.c:1013` to `:1021` build the environment from the event and a fixed list,
and OpenZFS offers no setting for more.

Pros:

- none that the other options lack, for the modes this route has.

Cons:

- ZED still needs one of the other options for the sweep,
  so this option can only be added to one of them;
- the handler's own documentation states it "reads no configuration and takes no answer from the environment";
- a variable that selects a privileged mode is a seam an account can set for a start it causes.

### Ranking of choice 1

Ranking: A > B > C > D.

A over B:
both reach ZED's no-argument start,
but A keeps root executing one file,
and B adds a shipped script whose only job is to supply a word A can derive from the name.
B's single advantage, a command a person can type from anywhere,
is cheaper to get back with choice 4 than a second program is to own.

B over C:
both are name based,
but B lets a person read and type mode words,
keeps `--help`,
and keeps the modes' refusal texts,
while C turns every mode into a path and multiplies the installed set.

C over D:
C works with ZED and D does not,
so D is not a selector at all but an addition to one of the others,
and it adds an environment seam the modes have never had.

## Choice 2: what the installed names point at

### Option A: symlinks to a root-owned copy

Root installs the wrapper once,
at
`/usr/libexec/monochromatic-git-policy-cli/monochromatic-git-policy-cli`,
and the two family names are symlinks to that path.

Probed: ZED's scan follows a symlink,
so a link passes its regular-file, owner and mode tests with the target's values,
and a link whose target is gone fails the scan with one warning and nothing else
(`keep/probe-installed-file-kinds.log`).

Pros:

- root executes only root-owned bytes,
  which the design already requires of the handler's code and configuration;
- one copy of the program on the root filesystem,
  and a release upgrade replaces one file;
- the links survive an upgrade that writes the new file and renames it over the old one,
  because they name a path and not an inode.

Cons:

- a missing or broken link is silent to ZED,
  so `doctor` must check both names on every run;
- two links to create and to report.

### Option B: hard links

The same three names as three hard links of one inode.

Pros:

- nothing can break a name by moving a target;
- no symlink resolution at exec time.

Cons:

- an upgrade that replaces the file by rename leaves the hard links on the old release,
  so the zedlet would keep running the previous release until root re-links it,
  and nothing would say so;
- `/etc/zfs/zed.d` and `/usr/libexec` must be on one filesystem,
  which is true on the decided layout and is not guaranteed anywhere else.

### Option C: a copy per name

Three copies of a 28,595,720-byte program.

Pros:

- each name is independent,
  and no link can break.

Cons:

- three times the space on the root filesystem,
  and three files to replace at every root-side upgrade,
  each with its own chance to be left behind;
- the release's one program would exist in three places,
  which is the state the decision ended.

### Option D: a link into the account's toolchain install

The names point at the wrapper the toolchain installed under
`~/.cache/monochromatic-git-policy-cli-bin/`.

Pros:

- no root-side copy at all.

Cons:

- root would execute a file the account can rewrite,
  which is an escalation and contradicts the design's own root-ownership rule;
- the toolchain directory moves on every install.

### Ranking of choice 2

Ranking: A > B > C > D.

A over B:
both put root-owned bytes behind the names,
but A's names follow a replaced file and B's do not,
and a zedlet that runs last release's program without a word is the failure this route can least afford.

B over C:
B costs one inode,
C costs three copies and three upgrade steps that can drift.

C over D:
C is wasteful,
D hands root an account-writable path.

## Choice 3: where in the start the dispatch sits

What a placement costs root is every start-time action after it that runs.

### Every start-time action of today's wrapper

In the order the code takes them, with what a root-mode start does at the recommended placement
(choice 3 option A) for each:

1.  `args_os` collection in `main`.
    Read:
    a root mode needs it, and the name is in it.
2.  `service_main::is_service_start`, which reads the arguments only.
    Skipped:
    the name check comes first and answers, so a root-mode start never reaches it.
3.  `panic_notice::install_payload_free_panic_hook`.
    Skipped:
    the hook exists for the linked scanner's catch boundaries, and no scanner runs here.
4.  `vars_os` collection.
    Skipped:
    a root mode reads no variable.
5.  `runtime_probe::is_requested`, a test-only variable.
    Skipped.
6.  `hooks_off::activate`, a process-wide switch.
    Skipped.
7.  `abandoned_locks::meet_the_locks_of_the_working_directory`:
    the working directory,
    `GIT_DIR`,
    one ancestor walk for a Git directory,
    then a record written into the common Git directory
    and any lock a proof says is abandoned removed.
    Skipped, and this is the one that writes.
8.  `process_resolution_inputs`:
    `current_exe`,
    `current_dir`,
    `PATH`,
    `PATHEXT`,
    `XDG_RUNTIME_DIR`,
    the common Git paths.
    Skipped.
9.  `refusal_when_started_as_git`:
    the chain marker in the environment,
    this start's record file,
    and the process table for the parent.
    Skipped.
10. `plan_invocation`'s control stripping and global layout, arguments only.
    Skipped.
11. The management arm, `plan_management_with_section`:
    the repository configuration,
    the hooks scan,
    and the service.
    Skipped.
12. `repository_link_stop`:
    `HOME`,
    the working directory,
    and the identity of the 860 drive link.
    Skipped.
13. `resolve_real_git`:
    the `PATH` scan, file identities, and the service unit's recorded Git.
    Skipped.
14. `invocation_overlay` and `announced_query`.
    Skipped.
15. `git_facts`, which starts Git.
    Skipped.
16. `shipped_checks`, which reads
    `monochromatic-git-policy-cli.config.jsonc`.
    Skipped.
17. `run_wrapped_command`:
    policies,
    the service socket,
    Git LFS,
    the linked-worktree copy.
    Skipped.
18. `forward_action` and `stack_if_push`:
    the worktree copy,
    the auto-push,
    the LFS upload,
    the network.
    Skipped.
19. The action's execution:
    replacing this process with the real Git,
    or the coordinated run.
    Skipped.

What a root-mode start runs instead is today's handler:
the grammar,
the compatibility gate,
the mode,
and the two log waits at exit.

The probes show the difference with one binary and one working directory
(`keep/probe-wrapper-start-forwarded-word.log`):

- today's entry given `boot-reconcile` reached Git
  (`git: 'boot-reconcile' is not a git command`, status 1)
  and wrote a new `cli-git-lock-holders` record into `.git` on each run;
- the same entry given a configuration that names an unknown policy refused with its own
  `config-invalid` line,
  so the repository configuration was read;
- the same binary with a dispatch at the top of `main`, which is where the service mode sits,
  answered its own refusal,
  read no configuration,
  and changed nothing in `.git`.

Started exactly the way ZED starts a zedlet from inside a repository,
the wrapper without a name dispatch wrote one `cli-git-lock-holders` record per start
and started Git as its child, which under ZED is root
(`keep/probe-wrapper-under-zed-env.log`).

### Option A: the first statement of `main`

A name check on `args_os`'s first element,
before the service check and before the panic hook,
then `worktree_clone_mount_handler::run(&arguments[1..])`,
then the existing `end_after_the_log`.
The name comes from the argument vector and never from `current_exe`,
which answers the link's target (the ZED replay printed that).
Provisionally this is a small module of its own
(`src/native/worktree_clone_mount_handler/root_mode_names.rs`),
holding the two names and one predicate,
so the name list is written once and `doctor` can print it.

Pros:

- reaches none of the other start-time actions, only the first;
- the same placement the service mode already proves,
  so the entry keeps one shape for both modes of this kind;
- nothing of the wrapper's own start can fail before root's work.

Cons:

- the dispatch sits outside `entry::run_process`,
  so the entry now has two mode checks before the wrapper's path,
  and the order between them must be stated (the name check first, since it reads less).

### Option B: after the panic hook and the environment

Pros:

- a panic in a root mode would print the wrapper's payload-free message;
- the environment is already collected if a mode ever wants it.

Cons:

- `main` collects the whole environment for a start that reads none of it;
- no probe shows any benefit over A.

### Option C: at the top of `entry::run_process`

Pros:

- one place decides everything, and `main` stays as thin as it is.

Cons:

- the runtime probe's variable is read first,
  and `hooks_off::activate` and the lock pass follow,
  so the lock pass, its record write, and its possible lock removal run before root's mode.

### Option D: inside `plan_invocation`

A name check made here, where the arguments are already planned,
costs every action before it.

Pros:

- nothing of `main` or `run_process` changes shape, so no other mode's start moves.

Cons:

- the lock pass, the resolution and the started-as-Git check all run first,
  so root writes into whatever repository holds the working directory;
- `git_facts`, the configuration read and the forwarded Git are one branch away,
  and `keep/probe-wrapper-start-forwarded-word.log` and `keep/probe-wrapper-under-zed-env.log`
  show what that costs.

### Ranking of choice 3

Ranking: A > B > C > D.

A over B:
B collects an environment nothing reads and gains only a panic message shape.

B over C:
C adds the lock pass, which writes a lock-holder record into the repository whose directory a start
happens to be standing in.

C over D:
D adds the configuration read and a Git start before the mode,
and it is the only option that leaves the mode word indistinguishable from a Git subcommand.

## Choice 4: whether the account's `PATH` keeps a short name for the modes

Today the release's second program is installed by the toolchain,
so a person types `mount-git-worktree-clone-datasets adopt-worktree-parent ...`.
After the change the account's toolchain installs one program,
and the family names exist only under root's install.

### Option A: the root path only, and `doctor` prints it

A person runs
`/usr/libexec/monochromatic-git-policy-cli/mount-git-worktree-clone-datasets adopt-worktree-parent --checkout ...`,
and `doctor` prints that line complete,
with the values it read.

Pros:

- no change to the toolchain,
  whose plan expresses one post-install link per tool
  (`mpm-shadow`'s `postinstall.link` takes `sourceBin`, `linkName` and `linkDir`,
  and refuses any other key),
  and no second link to keep right;
- the name a person types is the name root installed, so a person and a unit run the same file;
- `doctor` already prints concrete commands it never runs.

Cons:

- the command a person must type is long,
  and every refusal text inside the modes that names a follow-up command must be rewritten to it;
- the e2e recipes that call the built binary by name must make the link themselves.

### Option B: the toolchain links the family name beside `git`

The account's install makes a second link,
so `mount-git-worktree-clone-datasets` stays on `PATH`.

Pros:

- today's ergonomics are kept;
- the name a person types stays short.

Cons:

- the toolchain's plan schema cannot express it today:
  `postinstall.link` is one object and `binAliases` maps an archive's alias program to a primary program,
  so this is a change in another package with its own owner and its own release;
- the account's link would point at the account's own copy,
  which is a different file from the one root's units run,
  so `--version` would prove the account's release and not the one root executes.

### Ranking of choice 4

Ranking: A > B.

A over B:
A needs no other package to change and keeps one file behind every name,
and the long command is printed by `doctor` complete.
B should come back as its own proposal if the long path turns out to bite in use,
and it should then link to root's copy rather than the account's,
which is a further reason not to decide it here.

## What else the modes keep, and what they lose

Kept, because the modes are the same code and the same grammar:

- the mode words, the adoption's three options, the usage text and the refusals;
- exit status 0 for a mode that did its work and 2 for a refusal;
- the compatibility gate before the first raw request, and the probe's skipped-record arm;
- the two installed paths, the configuration path and the program name in messages.

Changed:

- `--version` and `--help` are reached only through a family name,
  and they keep today's lines
  (`mount-git-worktree-clone-datasets <version>`),
  so `native:install`'s check and `doctor`'s version check keep working;
- every message that tells a person to run `mount-git-worktree-clone-datasets ...`
  names the installed path instead
  (`worktree_clone_mount_handler/handler_mode_arguments.rs`,
  `mode_awaiting_its_step.rs`,
  `sweep_preconditions.rs`,
  and `worktree_clone_mount/clone_slot_walk_refusals.rs:194`);
- the `--help` paragraph that claims both install paths exist says who creates them and when,
  which is M1's finding;
- a root mode no longer passes the lock sweep, so a mode started from inside a repository writes
  nothing into that repository.

Lost:

- the smaller program: 3,330,016 bytes against 28,595,720 at one commit, 8.59 times.
  The start-cost probe put the wrapper at a median 7.00 ms against the handler's 4.47 ms over 31 alternating
  pairs, with overlapping distributions on a loaded host,
  so the timing is context and the size is the fact (both in `keep/measurements.md`);
- a release that can pin or roll back the root side separately from the wrapper,
  which is the accepted cost of the placement the project already chose.

## What changes, and in which step

### One commit: the release ships one program

These cannot be separated,
because the guard reads the manifest and the release lists in one test,
and `toolchain:lock:check` compares the plan and the lock offline:

- `package/git-policy/cli/Cargo.toml`:
  remove the second `[[bin]]` and its comment,
  and correct the two comments that count executables
  (the binstall table itself predates step 5,
  which changed only what its comment says `{ bin }` names);
- `package/git-policy/cli/src/native/mount_git_worktree_clone_datasets/main.rs`: deleted;
- `package/git-policy/cli/src/native/lib.rs`: the module's doc paragraph, which says "second executable";
- `src/native/source_guard/release_bins_tests.rs`: the released list and its doc;
- `bin/native-release-bins.unit.test.ts`: the doc comment; its planted-second-bin control still
  requires four disagreements and needs no change of logic;
- `package/crate-release/cli/releasable-crates.json`: `checkBins` holds one name;
- `.github/workflows/cargo-publish.yml`: the job comment, the `--bin` flags, the smoke step's handler
  lines, and the second `bin=` copy line;
- `toolchain.plan.jsonc`: `bins` holds one name,
  and `toolchain.lock.json` is regenerated by the toolchain task that writes the lock offline,
  never by hand;
- `package/git-policy/cli/mise.toml`: the descriptions of `native:build`, `native:build:release`,
  `build` and `native:install` that count two executables,
  and the `e2e:mount-git-worktree-clone-datasets` task's own text if it names a binary;
- `bin/native-install-steps.ts` with `bin/native-install.unit.test.ts` and
  `bin/stand-in-install-programs-for-tests.ts`: the handler's `--version` check leaves the install,
  because no released archive places a family name on the account's machine;
- `README.md`: the release paragraph, the install section that says one archive carries two executables,
  the install note that refuses until a release carries the second program,
  and the source-layout paragraph;
- `package/git-policy/cli/SPEC.md`: "## The two executables and shipped policies" returns to
  "## Single executable and shipped policies" and says how the root modes are reached;
  "### The start-time ZFS compatibility gate" and "The start probe beside the gate"
  name the modes through the family name instead of a second program;
- `package/git-policy/cli/doc/step-5-second-executable-2026-11-20.md`: a note under its status that
  the second program was removed by this decision, with the date,
  keeping the record's own text.

### The same commit: the entry, and every test that started the second program

Deleting the `[[bin]]` breaks these at compile time,
so they land with the release lists of "One commit: the release ships one program":

- `src/native/main.rs`: the name check first, and the root-mode arm;
- `src/native/worktree_clone_mount_handler/root_mode_names.rs`: the two names and the predicate,
  with its sidecar test file;
- `src/native/testing/root_mode_alias.rs`: one writer of a family name in a fixture folder,
  beside `testing/fixture_executable.rs`,
  so each test that starts a mode names one link and not one copy of the rule;
- `src/native/binary/binary_mount_git_worktree_clone_datasets_tests.rs`,
  `binary_zfs_adoption_guest_tests.rs`,
  `binary_zfs_sweep_guest_tests.rs` and
  `binary_zfs_unmount_guest_tests.rs`:
  `CARGO_BIN_EXE_monochromatic-git-policy-cli` through the fixture link;
- `binary_service_mode_tests.rs`: the comment that counts two executables;
- `e2e/start-probe-guest.ts`: `HANDLER` becomes the wrapper pushed into the guest plus the link its
  commands run, and every `${HANDLER} start-probe` line keeps its shape;
- `e2e/mount-git-worktree-clone-datasets-guest.ts`: the same link inside the guest,
  beside the four guest test files named under "What the code holds today".

### The plan and the design text

One commit, after the entry and its tests:

- `doc/planning/cli-git-zfs-worktree-implementation-plan.md`:
  step 5 is rewritten as the undo,
  keeps its release list and loses the second executable from it,
  and its "What it builds" gains the entry's name dispatch;
  step 6's sentence "the gate both binaries run" becomes one gate;
  step 18's `doctor` gains the installation check "Steps 18 and 19" states;
  step 19 names the copy and the two links instead of two installs of a program,
  and its acceptance arm asserts that a family name answers `--version`;
- `doc/planning/cli-git-zfs-worktree-design-proposal.md`:
  "Binding crate and compatibility" loses the separate-binary paragraph and keeps the "Keep inside cli"
  placement with this design's answer,
  "ZED reconciliation and mount safety" keeps its two installed paths and says they are names of one file,
  and "Decided defaults" replaces the second binary's cost with the one measured here;
- `doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
  `cli-git-zfs-start-probe-design.md` and
  `cli-git-zfs-private-propagation-after-replacement-mount-design.md`
  keep their commands and gain a sentence that the name at the head of each is a link to the one program
  (their `ExecStart`, `ExecReload` and probe lines already name the family name, so no command changes).

### Steps 18 and 19

They land in their own steps, and nothing waits on them for the release to hold one program:

- step 18's `doctor` gains the check that the installed file exists,
  both family names resolve to it,
  each resolves to a root-owned, owner-executable file that is not group- or other-writable,
  and the zedlet's name is one ZED's scan would accept;
  its findings say what a broken or missing name costs (a sweep that never runs) and print the
  command that recreates it;
- step 19 prints the copy and the two links,
  and its guest arm starts a unit by the family name and reads `--version` through the same name.

### What does not change

- the crate links no OpenZFS library and starts where OpenZFS is absent;
- the modes' grammar, exit statuses, gate, probe and records;
- the two installed paths' spellings, so every unit and zedlet line already written stays correct;
- the ZED hook's placement, class prefix and synchronous or asynchronous choice;
- the event descriptor ZED puts on 3: the entry touches no descriptor, and no mode reads it today;
- the archive's layout rule: one declared `[[bin]]`, one `bin=` line, one file.

## What the probes showed, and what they did not

### Shown

- ZED's exec shape, `argv[0]`, argument count, descriptors and built environment, by a replay of the
  source's own child step, and `current_exe` resolving a symlink start away from the installed name;
- systemd's `argv[0]` for a unit's `ExecStart`, by a real transient user unit;
- each of the four dispatch rules deciding as described, from a full path, a relative path, a `PATH`
  lookup, a unit start and the ZED replay;
- ZED's scan accepting a symlink and skipping a broken one;
- today's wrapper given a mode word reaching Git, writing a lock-holder record and reading the
  repository configuration, and the same binary with a main-level dispatch changing nothing;
- the wrapper started as ZED starts a zedlet writing one record per start and starting Git as root;
- the two programs' sizes at one commit, and one timing comparison with its own spread.

### Not shown

- ZED itself: it needs root, `/dev/zfs` and a pool, so the replay stands in for its child step only.
  The environment and descriptor values come from the replay's own `execle`, not from a running ZED;
- a system unit start: the probe used `systemd-run --user`, so a system manager's `argv[0]` is read from
  systemd's documented behaviour and not measured here;
- any mode touching ZFS: this round ran no guest, so no request, mount, unmount or adoption was exercised;
- the first-start page-in of a 28.6 MB program against a 3.3 MB one on a cold cache: every timing run
  read a warm page cache, so the size ratio is not a cold-start cost;
- whether AppArmor's policy on this host confines the family names: no profile exists yet,
  and the rules file records an unexpected permission error rather than routing around it;
- whether `doctor`'s future installation check catches a link that a package manager renames instead
  of deleting.

## The choices this proposal puts to the owner

Each is separable, and each is ranked in its own section:

1.  How a start reaches a root mode: the installed name and a word (choice 1, A),
    namespaced words with a zedlet script (B),
    a name per mode (C),
    or an environment variable (D).
2.  What the names point at: symlinks to a root-owned copy (choice 2, A),
    hard links (B),
    a copy per name (C),
    or a link into the account's install (D).
3.  Where the dispatch sits: the first statement of `main` (choice 3, A),
    after the panic hook and the environment (B),
    at the top of `run_process` (C),
    or inside `plan_invocation` (D).
4.  Whether the account's `PATH` keeps a short name: the root path and `doctor`'s printed command
    (choice 4, A), or a second toolchain link (B).

This proposal's own ranking is A in every one of them.

## Appendix: where the probe transcripts are

Round folder `/home/user/temp/agent/zfs-root-modes-in-the-one-wrapper-executable-design-2026-11-24/`:

- `keep/measurements.md`: the numbers, the commands and the arms;
- `keep/probe-a-name-and-words.log`: the four rules from five launchers;
- `keep/probe-zed-replay.log`: ZED's child step, replayed;
- `keep/probe-systemd-unit-start.log`: a transient user unit's `argv[0]`;
- `keep/probe-installed-file-kinds.log`: what ZED's `stat` test sees of a link;
- `keep/probe-wrapper-start-forwarded-word.log`: the same binary, a mode word and a main-level dispatch;
- `keep/probe-wrapper-under-zed-env.log`: the wrapper started as ZED starts a zedlet;
- `keep/probe-start-cost-raw.txt`: the raw timings;
- `scratch/dispatch-fixture/`: the fixture dispatcher, the replay harness and its links;
- `scratch/zed-source/`: the OpenZFS `zfs-2.4.4` files this design's ZED facts were read from.
