# The wrapper never starts itself

Note 2026-11-04T22:23:58-05:00: the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders, logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

Status: phase 1 built in this branch (LFS upload run by the parent, merge push, bans, lock budget); phase 2 (the hook
dispatcher) is another change. The start of the real Git itself is covered by the section "The wrapper never starts
itself as Git (2026-11-03T10:49:47-05:00)", which supersedes the section "No runtime marker that refuses a nested
start". What the wrapper does when `PATH` has no real Git after it is decided in the section "Falling back to the Git
the commit service was set up with (2026-11-03T11:26:52-05:00)": it uses the Git its commit service's unit records,
with a notice. That is the coordinator's pick under the owner's delegation, open to the owner's veto.

## The rule

The owner's instruction, quoted: "We should never invoke ourselves recursively. That's bad design. Audit, remove, and
ban that design aggressively." and "There are clean ways to have extensive explanatory error messages even with the
ban." The owner then added: "we don't have to use any shell!"

The rule as built:

- The wrapper never starts its own executable, whether as a child, as a Git hook command or `-c` argument,
  as a shim it writes, or through dispatch on the name it was started as (`argv[0]`).
- The wrapper never writes shell text: no `sh -c`, no script file, no shell function, no `printf` or `$?` in a hook.
- An explanation of a failure is printed by the parent process that owns the child, from facts the parent can read
  itself. Nothing that Git or a hook starts is asked to explain anything.

The rule is not in `AGENTS.md`. It lives here, in `package/git-policy/cli/README.md`,
in `package/git-policy/cli/SPEC.md` and in the mechanical bans (section "The bans").

## Audit inventory

Every way found by which the wrapper binary is started because of something the wrapper did. File and line numbers
are from the commit before this change (`5561c8b7cb`) unless a path says "now".

### Removed in phase 1

- The LFS pre-push hook. `lfs_pre_push.rs` `hook_arguments` wrote `hook.cli-git-lfs-pre-push.command='<wrapper>'
  cli-git lfs-pre-push`; Git ran it through `sh -c`; `entry.rs:289` recognised the words, and `run_hook_entry`
  started `git-lfs pre-push`. The call sites were `entry.rs:702`, `push_stacking.rs:194`,
  `auto_push_single_flight.rs:226` and `commit_command.rs:228`. An intermediate version of this change replaced the
  command with the plain words `git-lfs pre-push` and read Git's trace2 events to explain a failure; that design is
  gone too (see "Who runs `git-lfs`, and who explains"). Now no hook of any kind is injected: the parent runs
  `git-lfs push <remote> <refs>` itself before the push (`lfs_upload.rs`).
- The merge push of an integrated race. `push_stacking.rs:142` was `Command::new(request.own_executable)` running
  `push` with `CLI_GIT_STACKED_PUSH=1` in a cleared environment, so that the merge passed the push gates. This was a
  real recursive start that the first inventory did not list, because it looked like a use of `current_exe`. Now
  `stacked_merge_push.rs` runs `plan_invocation` for the merge push inside the running process, with the marker set only
  in the environment handed to the planner, and runs the planned action as a child of the same process.
- `push_stacking.rs:211` `current_exe`. It only produced the path for the start above; gone with it.
- The exec-forward site (`entry.rs:701-719`). A push in a repository with LFS keys was forwarded by process
  replacement, leaving no parent to explain a failure. Now such a push runs as a child like a stackable one
  (`Action::ForwardPush` with `lfs` set and `stack` unset when it is not stackable). Pushes in repositories
  without LFS keys are still process replacements.

### Kept, classified

- `real_git.rs` `process_resolution_inputs` calls `std::env::current_exe`. Identification: the path is excluded from the
  search for the real Git so that the wrapper never selects itself. Nothing is started from it. It carries
  `#[allow(clippy::disallowed_methods)]` with that reason.
- `service_route.rs` `remember_own_executable` and the generated unit (2026-11-04). The commit service is a mode of the
  wrapper's executable, so the unit the wrapper writes names the wrapper's own path in `ExecStart` followed by `cli-git
  service run`. This is identification: the wrapper writes the path into a file the user's service manager reads, and starts
  nothing; the manager starts the service. The mode is chosen by those arguments (`service/service_main.rs`), never by the name
  the program was started as. Service mode refuses a `--git` that is the wrapper itself, a link or copy of it, or a launcher
  (`doc/planning/cli-git-resident-service.md`, "One executable"). Veto-open.
- `entry.rs` `forwarded_to_self` and `FORWARD_TARGET_VARIABLE`. A guard that reports a wrapper found as its own
  real Git. It prevents an accidental loop; it starts nothing. It covered only a target that is the running
  executable; the launcher incident of 2026-11-03 showed that it was not enough, and `started_as_git.rs` was added
  beside it.
- `CLI_GIT_STACKED_PUSH`. Still set, now in the planner's environment and in the overlay of the merge push, so Git
  and its hooks can tell a merge push. It is not a start.
- `hooks_off.rs`. Points `core.hooksPath` at a directory that never exists, so traditional hook files
  (including the shim `git lfs install` writes) never run through the wrapper's Git children.
- Test support. Controls that start a helper copy of the test binary go through one excused helper per test crate
  (`test_support.rs` `this_test_binary`, `binary_support.rs` `this_test_binary`; one local allow in
  `candidate_consumer_tests.rs`, which is its own test crate). Binary-level controls start the wrapper on purpose;
  that is the subject under test, not a design of the wrapper.

### Left for phase 2 (the hook dispatcher)

- `main.rs` reads `args_os().next()` to dispatch on the name the wrapper was started as.
- `commit_command.rs:168` calls `current_exe` for the commit hook dispatcher.
- `hook_dispatch*.rs`, `commit_preparation.rs`, `commit_revalidation.rs`, `commit_conclusion.rs` and
  `hook_dispatch_entries.rs` implement the dispatcher.

Another change removes these. This document gets its own section for that phase in that branch; the allow-list entries
in `no_self_invocation_tests.rs` for `main.rs` and `commit_command.rs` go with it, and the stale-entry test enforces it.

### Found, not self-invocation

- `manual_push_probe.rs` `hook_source` wrote a `#!/bin/sh` script. It started nothing recursive, but it was shell text
  written by the wrapper, which the owner's rule forbids. It is gone; see "The manual-push probe" in the decisions.
- `doctor.rs` holds the text git-lfs itself writes as its hook shim, only to recognise it. The wrapper never writes it.
  On the allow-list with that reason.

## Measurements (this host)

- Git prepends its exec path to `PATH` for the programs it starts (`setup_path` in Git's `exec-cmd.c`). A hook run by
  real Git therefore sees `/usr/libexec/git-core/git` for `git`, so `git-lfs` started from a hook calls the real Git,
  not the wrapper. No `PATH` shim is needed or wanted. The control
  `binary_lfs_parent_tests.rs` `git_starts_the_upload_and_no_wrapper_stands_between` records the executable of every
  ancestor of the stand-in `git-lfs`: the nearest is Git and exactly one wrapper is above it, the process the caller
  started.
- A `hook.<name>.command` without shell metacharacters is started by Git directly; the stand-in saw its parent
  process be Git, with no `sh` between. Git's own trace records `use_shell: true` for it, which is Git's request
  flag; the run itself needs no shell.
- `git push --dry-run` runs the configured `pre-push` hook (a dry run uploaded nothing only because the wrapper does
  not add the hook to dry runs), and it contacts the remote (a `git-receive-pack` child starts before the hook).
- A failing `pre-push` hook gives exit status 1 and `error: failed to push some refs to '<url>'` for every hook
  status tried (1 and 7), the same as a remote refusal. The text is translated (German run:
  `Fehler: Fehler beim Versenden einiger Referenzen nach ...`). Neither status nor text identifies the hook.
- With `GIT_TRACE2_EVENT` naming a file, Git writes for each hook a `child_start` event (`child_class` `hook`,
  `hook_name` `pre-push`, `argv[0]` the command string) and a `child_exit` event with the `code`, in any language.
  `-c trace2.eventTarget=...` was measured to write nothing on this host; the environment variable is what works.
  These measurements decided against the intermediate trace2 design; the final design uses none of them.
- The remote value in the update line is the remote's advertised value for the ref, all zeros for a new ref, not the
  local remote-tracking ref (first push: zeros; second: the advertised old tip).

## Decisions

### Who runs `git-lfs`, and who explains

The owner's question, quoted: "Why do hooks we control even need spawning at all?" The injected hook was a hook the
wrapper controls, and it is what forced a start (first of the wrapper, then of a shell, then of trace parsing and
forgery resistance). Options, with the ranking required by the repository rules:

- A. The parent runs `git-lfs push [--all] <remote> [<ref>...]` as one child before the push, and starts the push with
  no hook. Pros: the parent owns the whole step and sees the status, so the explanation needs no detection, no
  trace and no hook; no injected configuration; the child sits in the same bounded process tree as every other child
  (deadline, stop, kill); the same code serves the typed push, the merge push, the auto-push and its reconcile pushes.
  Cons: the remote and refs must be read from the push's arguments instead of being handed over by Git (two short
  Git queries for a bare push); a refspec form the wrapper cannot resolve falls back to uploading everything.
- B. A plain hook command with a trace2 reading by the parent (the intermediate version of this change). Pros: Git
  passes the exact update lines. Cons: a hook is still injected, the explanation needs Git's event stream, a caller's
  own `GIT_TRACE2_EVENT` disables it, and the attribution depends on trace2 event fields.
- C. A plain hook and a text match on Git's `failed to push some refs` line. Cons: translated, and identical for
  remote refusals, other hooks and rules, so it blames the wrong cause.
- D. A hook that reports its own status through shell text. Banned by the owner.

Ranking: A > B > C > D. A over B because A has no hook, no second source of truth and no detection to forge, where B
keeps all three. B over C because B at least attributes the failure correctly. C over D because C is shell-free,
though wrong in some cases. A was built.

Why option 4 of the first brief (the parent runs `git-lfs` itself) is viable after all: it was rejected for needing
ref-update lines and a remote's advertised values. `git lfs push` is a different entry point that needs neither: it
takes refs, reads no update lines, filters out objects the local clone's remote-tracking refs already reference, and
objects are content addressed, so an upload that sends a superset is idempotent (measured with git-lfs 3.8.1, and by
the controls that run the real `git-lfs` against an inline LFS server).

What the upload sends (`lfs_upload.rs` `plan_upload`, reading the push with the shared option table):

- An explicit repository (or `--repo`) and refspecs name the remote and refs (the source of each refspec; a deletion
  names nothing; `tag <name>` is `refs/tags/<name>`).
- `--all`, `--branches`, `--tags`, `--mirror` and a glob refspec upload everything, with `--all`.
- A bare push takes the remote the way Git does (`branch.<name>.pushRemote`, `remote.pushDefault`,
  `branch.<name>.remote`, then `origin`) and the current branch (`HEAD` without one); `push.default=matching`
  uploads everything. Two short queries, only for a push that needs them in a repository with LFS keys.
- A push that bypasses the upload (`--no-verify` in any accepted abbreviation, `GIT_LFS_SKIP_PUSH`, the caller's
  `-c hook.cli-git-lfs-pre-push.enabled=false` switch), a dry run, or only deletions uploads nothing.
- The child runs with the real Git's directory first on `PATH` (Git does the same for hooks), so a `git` it starts is
  the real Git, and in the directory a `-C` option selects.

The `git-lfs` lookup (`lfs_push_precheck.rs`) refuses before anything starts when no `PATH` directory holds an
executable `git-lfs`, with the same words as before, and honours the bypasses above. It reads the invocation's `PATH`.

Cost: an LFS repository's push starts one more child (`git-lfs`) and, for a bare push, two short Git queries. A
repository without LFS keys pays nothing: its pushes still replace the process.

### No runtime marker that refuses a nested start

Superseded on 2026-11-03T10:49:47-05:00 by the marker of "The wrapper never starts itself as Git". The text that
follows is the decision as it stood, kept as the record of what was weighed.

A refusal such as "if `CLI_GIT_NESTED` is set, do not run" is not added. A user's own hook or script legitimately
runs `git status` through an absolute path or an installed wrapper; a runtime refusal would break that, and the
forward target guard (`forwarded_to_self`) already covers the one accidental loop. The design is removed and banned
at build time instead.

What the later incident showed (see the section "The wrapper never starts itself as Git"): the forward target guard
compared the selected file with the running executable, and a toolchain launcher is another file, so the one
accidental loop it covered was not the only one. The objection to a runtime refusal was that it would refuse
legitimate nesting. The marker of that section does not: it refuses only the process that a wrapper started as Git,
or the child of the wrapper that started it, and a hook, alias or editor of the real Git has another process in
between.

### The manual-push probe

What it is for: the manual-push policies (forbidden strings and the like) must scan exactly what a typed `git push`
would publish, so before the push is forwarded the wrapper learns the ref updates Git would send: for each, the local
object, the remote ref and the remote's current object. It used to learn them from a private dry-run push whose
`pre-push` hook was a shell script that copied Git's update lines to a file, then confirmed each destination with
`git ls-remote --refs`.

Replacement: `git push --dry-run --porcelain` with every hook off and `core.abbrev=no`. Measured on Git 2.56.0: it
contacts the remote, prints one `To <url>` block per destination, one tab-separated line per ref
(`<flag>\t<from>:<to>\t<summary>`) and `Done`; the summary of an update holds the full old and new object names, and a
new ref, a deletion and a refusal hold a bracketed word. The parser is `manual_push_porcelain.rs`; the probe takes
local objects from the summary, or from one `git rev-parse` for new refs, resolves the remote Git would pick for a
bare push with the same rule the upload uses (`lfs_upload::resolve_push_remote`), maps the destinations to the push
URLs with `git remote get-url --push --all`, and confirms every destination with `ls-remote`. Behaviour kept: the
race check (a reported old value that `ls-remote` contradicts stops the probe), deletions with no local object,
up-to-date pushes with no updates, an unreachable destination failing the probe. Behaviour improved: every Git child
of the probe is now bounded by the query deadline, where the dry-run push and the `ls-remote` had none. Behaviour
that differs: a report that does not state the remote's value (a deletion, a refusal) takes it from `ls-remote` alone,
so the race check does not cover those refs; the probe no longer needs a private scratch directory.

### What the explanation can and cannot be confused with

The parent prints the explanation because it saw its own child fail; nothing is parsed from output. A remote that
prints text imitating the explanation does not add one (control
`a_remote_refusal_is_not_blamed_on_the_upload_even_when_it_imitates_the_explanation`), another configured `pre-push`
hook of the caller that fails is not blamed on the upload, and a failed upload never starts the push, so Git's
"failed to push some refs" line does not follow it.

## The wrapper never starts itself as Git (2026-11-03T10:49:47-05:00)

### What was observed

Release 0.5.26, 2026-11-03: with no real Git after the wrapper on `PATH`, the wrapper ran itself as the real Git and
each copy did the same, a chain of wrapper processes, each one the child of the one above it and each command line
longer than its parent's (each level adds its own `-c core.hooksPath=/dev/null/cli-git-no-hooks` option and a
`-c hook.<event>.enabled=false` option for every hook event). It reached the 100 tasks its control group allowed. The
setup: `PATH` held a directory with a `cargo` link, a rustup toolchain `bin` and `.cache/cli-git-bin`, and no
`/usr/bin`;
`.cache/cli-git-bin/git` was a symlink to the toolchain launcher `.cache/toolchain-launchers/cli-git-native` (an
executable file of the `mpm-shadow` binary, with a `cli-git-native.launch` record beside it naming the real wrapper
executable). The owner's rule it broke: "We should never invoke ourselves recursively."

Reproduced with a copy of the real launcher, a record that names the debug build, and a `PATH` holding only a
symlink to it, inside `systemd-run --user --scope --property=TasksMax=24`. `git rev-parse HEAD` grew a chain of
wrapper processes (17 counted, the command lines 1223, 2291, 3359 and 4427 bytes, growing by 1068 bytes per level)
until the scope held 24 tasks and refused the next. `git --version` filled the scope the same way, and the wrapper
logged each start refused for lack of processes (`start_retry.rs` tries again for a bounded time, which is why a
chain at a cap looks held and not stopped).

### Cause

- `real_git_candidate.rs` `classify_candidate` decided that a `PATH` candidate was the wrapper by inode
  (`same_file`), by canonical path and by byte equality (`identical_content`), and otherwise by the first four bytes
  (an executable signature means real Git) or by markers of the TypeScript wrapper in a script. The launcher is a
  different file with different bytes and a native signature, so it was real Git.
- The wrapper then started it as Git. The launcher read its record and replaced itself with the real wrapper
  executable, a third file. That wrapper looked for Git, found the launcher again, and started it again. The first
  start in the observed chain was a child, not a replacement: `rev-parse` first reads the repository layout through
  a child Git (`repository_location.rs`, `--git-common-dir --show-toplevel --show-prefix`), and that child was the
  next wrapper.
- `entry.rs` `plan_invocation` had one guard, `forwarded_to_self`: stop when the forward-target variable names this
  executable (`same_file`). The variable named the launcher, and this executable is the real wrapper behind it, so
  the guard never matched. The one case it covered, a copy that is its own target, is the case the earlier controls
  tested.
- The version text and the start disagreed because they were produced under different `PATH` values, not by two code
  paths. `real_git.rs` `candidate_sequence` promotes a conventional location (`/usr/bin/git`) only when `PATH`
  exposes it. The reporter's shell had `/usr/bin` and the banner named it; the failing process had none. Under the
  launcher-only `PATH`, `git --version` loops the same way (measured in the capped scope, above).

### Rule

The wrapper never starts a file as Git when that file is, or leads to, a cli-git wrapper, and a cli-git executable
that a wrapper started as Git stops at once. There are two layers, because file identity cannot be complete (a copy
of another release, a launcher with no record, a link through a directory the wrapper cannot read) and the start
itself can always be recognised.

- Selection (`real_git_candidate.rs` `is_wrapper_launcher`): a candidate with a record `<name>.launch` beside it, or
  beside the file its links finally reach, is a launcher of the wrapper when the record's `toolId` ends in this
  crate's name, or its `real` is this executable. It is classified as a wrapper and skipped, like a link or a copy.
- Arrival (`started_as_git.rs`): every start of the real Git sets `CLI_GIT_STARTED_AS_GIT` to the starter's process
  number, its birth identity (`process_identity.rs`, `current_birth_identity`) and the mode (`replace` for the exec
  start, `child` for every other). `run_process` stops the process before it plans anything when the mode is
  `replace` and the process number is its own with the same birth, or the mode is `child` and the marker names its
  parent with the same birth. The exit status is 2, and the message names the executable, the starter and how it
  started this process, the file the starter chose as Git, the `PATH` it saw, why it stops, and the ways out.
- The failure when no real Git is found lists every candidate with its verdict, the `PATH` value, and the ways out
  (install Git; add the directory of the real Git to the `PATH` of the program that ran the command; remove an entry
  that should not be used). `cli-git-service` refuses a `--git` that is a launcher of the wrapper.

### Why the marker names the process and not only the variable

The earlier decision refused a blanket runtime refusal because a hook or alias that runs an installed wrapper below
a real Git is legitimate. A variable the real Git passes to everything below it cannot tell that case from the loop;
the starter's process number can. A wrapper below a real Git has the real Git (or its shell) as its parent, never
the starter, and the process of a replaced starter is Git itself, so a hook of it has a different number. The birth
identity keeps a reused process number from being taken for the starter. An identity that cannot be read decides by
the numbers alone, because stopping one start too many is repaired by reading the message, and missing a start is
not.

Options weighed, with the ranking the repository rules require:

- A. Selection by the launch record plus the arrival marker with the starter's number (built). Pros: stops the
  observed chain before any start, and stops anything the record does not reveal at the first arrival; no false stop
  for hooks, aliases or editors below a real Git; costs a variable lookup per ordinary start and one small file read
  per candidate examined. Cons: a program that the wrapper starts as Git, that is not itself a wrapper but starts
  one without a replacement (a user script `git` that runs the wrapper as a child), stands between the starter and
  the arriving wrapper, so the parent test misses it; selection catches such a script only when it carries a record
  or a marker string.
- B. The marker alone, as a flag: any cli-git executable that finds the variable stops. Pros: catches the script
  case. Cons: refuses every legitimate nested use below a real Git, which the earlier decision rejected on purpose.
- C. Selection only. Pros: the smallest change. Cons: cannot see another release, a launcher with no record or a
  script; the observed class of failure remains.
- D. A depth counter in the environment with a limit. Pros: bounds every shape. Cons: bounds the damage and does not
  prevent the start, and the limit is a number nobody measured; it also stops legitimate deep nesting.

Ranking: A > B > C > D. A over B because B reintroduces the false stops that the earlier decision avoided; B over C
because B at least stops the chain at its second link in every shape; C over D because C starts nothing in the
observed class while D lets a bounded chain run.

### Where the wrapper starts the real Git

Every place that starts the real Git sets the marker (the list is `rg "Command::new"` over the release source,
checked 2026-11-03):

- `forwarding.rs` `git_command`, the shared builder (child), and `replace_process_with_real_git` (replace). Callers:
  forwarded commands, queries, the commit steps, the auto-push, maintenance, the service's landing and index work.
- `auto_push_single_flight.rs` `git_command`, the push builder (child).
- `git_system_config.rs` `ask_git_var` (`git var`, with a cleared environment) and `management.rs` `status_git_dir`
  (`rev-parse`), the two starts that did not go through a builder.
- Not Git: `lfs_upload.rs` starts `git-lfs`, which finds Git through a `PATH` whose first entry is the real Git's
  directory; `package_repair_command.rs` starts package managers; `process_start.rs` and `index_lock_holders.rs`
  start `ps`, `powershell.exe` and `lsof`. None starts the wrapper, and none is a start of Git.
- The hook shims: the wrapper writes none (see "The bans"). The shims `git lfs install` writes are never run through
  the wrapper's Git, because every start carries `core.hooksPath` below `/dev/null`.
- The service: `service_main.rs` takes `--git`, starts Git only through `git_command` (marked), and now refuses a
  `--git` that is a launcher of the wrapper. A unit written while the flaw stood would hold the launcher path; the
  unit text includes `--git`, so `service_provision.rs` `in_place` finds it different from the text the next wrapper
  writes, and the next wrapper writes it again.

### Controls

- `binary_never_self_start_tests.rs`, behind a counting stand-in launcher that stops the chain itself after
  `START_CAP` starts (no control relies on the host to stop a process): a symlink to a launcher, a launcher named
  `git`, a chain of symlinks to a launcher, another release behind an unrecorded launcher (child start and replacing
  start), and a symlink, a hard link and a copy of the wrapper alone on `PATH`. Before the fix four of the five
  failed with 7 to 13 starts of the stand-in and exit status 97 (or `stand-in: start cap reached`), and the fifth
  failed for lack of the explanation. After it, the launchers are never started and the wrapper exits 2 with the
  message.
- The arrival marker shown red once: with the check disabled (`if true || ...`) the unrecorded-launcher control
  chained to the stand-in's cap and failed with `stand-in: start cap reached`, while the three launcher controls
  still passed because the selection layer stops those. The check was restored.
- Unit controls: `started_as_git_tests.rs` (marker format, both modes, a reused number, an unreadable identity, the
  live process, the message, the builders), `real_git_candidate_tests.rs` (a native launcher with a record, through
  its own name and through links; another tool's launcher; a record whose `real` is this executable; broken
  records), `real_git_tests.rs` (every candidate and the `PATH` in the failure).

### Not covered

- A user script named `git` that the wrapper starts as Git and that runs the wrapper as a child, with no record and
  no marker string: the arriving wrapper's parent is the script, not the starter. The script is the cause; the
  message of a wrapper that cannot find Git names every candidate, so it shows the script as the entry to remove.
- A `PATH` with no real Git is a stop only when no usable record exists. The decision that replaced "a separate
  decision" is the section "Falling back to the Git the commit service was set up with"; the wrapper still does not
  fall back to `/usr/bin/git` or any other fixed location that `PATH` does not expose.

## Falling back to the Git the commit service was set up with (2026-11-03T11:26:52-05:00)

Status: built and verified in the branch `feat/git-policy-cli-falls-back-to-installed-git`. This is the coordinator's
pick under the owner's delegation, made before the build and recorded here at the time above, open to the owner's veto
when the owner is back. The delegation, quoted: "Pick
options as if I had picked them and invite veto when I say I'm back."

### The pick

When no real Git follows the wrapper on `PATH`, the wrapper falls back to the real Git it was set up with, with a
one-line notice on standard error, instead of stopping. The reason: native Git in that `PATH` would not be found at all,
but once the wrapper is found the person expects `git` to work. The owner's rules, from
`package/git-policy/cli/DESIGN.md` (2026-11-01): "Commits should never fail." and "never leak internal things that could
be resolved to the user".

The case behind it: the agent that fixed the self-start found a `PATH` that held the wrapper's `git` but no `/usr/bin`,
while `git --version` in the person's own shell said "shadowing git at /usr/bin/git" because that shell's `PATH` had it.
Before the pick, that `PATH` ended in the stop of the previous section.

### What records a real Git, and what does not

There is no installation record of the real Git a wrapper was installed over. Read, 2026-11-03:

- "shadowing git at" (`announced_query.rs`) prints the path that `PATH` resolution returned in that process; nothing
  stores it.
- The launch record `<launcher>.launch` of `mpm-shadow` (`package/mpm-shadow/cli/src/toolchain/launcher.rs`) holds
  `toolId`, `lockFile`, `real` and `scope`; its `real` is the wrapper's own executable, never the real Git.
- The unit `cli-git@.service` that `service_provision.rs` writes below `$XDG_RUNTIME_DIR/systemd/user` holds
  `ExecStart=... --git <path> ...`, the path being the real Git the wrapper resolved when it last provisioned the commit
  service. It is the one file that names a real Git, and it names exactly one.
- The version cache (`git_version_cache.rs`, `$XDG_CACHE_HOME/cli-git/git-version-<hash>`) holds the canonical path of
  every Git the wrapper asked for its version, one file per Git, with no order and no meaning of "installed over".

### Options for the record, ranked

- A. The `--git` of the commit service's unit (built). Pros: one file, one path, written by the wrapper from the Git it
  resolved, in a form `unit_texts` renders so a test can follow the writer; read only when `PATH` offers no Git, so the
  ordinary start pays nothing. Cons: the unit lives in the runtime directory, so it exists only after a commit has
  provisioned the service in this login and is gone after the last logout; a unit written while the earlier flaw stood
  can hold a launcher (refused by the checks below).
- B. A durable record written whenever `PATH` resolution succeeds, below the state directory. Pros: survives logout,
  and would exist before the first commit. Cons: a new file with its own writing, replacing and staleness rules, and a
  write or a read-and-compare on the path of every command, against "Invoke real git as rarely as possible."; not
  asked for.
- C. The version cache entries. Pros: survive logout. Cons: many entries with no order, so choosing one is a guess; a
  cache may be deleted at any time and is not a statement of what was installed over.
- D. A fixed conventional location such as `/usr/bin/git` when `PATH` hides it. Pros: works with no prior state.
  Cons: a guess, and the instruction for this change ("Use a recorded path, not a guess") rules it out.

Ranking: A > B > C > D. A over B because A exists today with no new state and no cost on the ordinary start, and B's
only gain (surviving logout) is a limit the person can see and fix by adding the directory to `PATH`. B over C because
B is a statement the wrapper writes about the Git it used, while C is a pile of entries from which one would be
guessed. C over D because C at least comes from a Git that really ran on this account, while D names a path nobody
recorded.

### Mechanism

- `real_git_record.rs` reads the record: the unit file `service_unit_file(runtime_dir)`, bounded to 64 KiB, as UTF-8,
  and the word after `--git` on the first `ExecStart=` line that has one. A relative path is no record.
  `process_resolution_inputs` names the unit file only below an absolute `XDG_RUNTIME_DIR`
  (`ResolutionInputs.service_unit`).
- `real_git_resolution.rs` `resolve_real_git` (re-exported by `real_git.rs`) is the one resolution every start uses.
  It runs `search_path_for_real_git` (the old search of `PATH`, unchanged),
  and only on failure reads the record and judges it with `classify_candidate`, the check
  every `PATH` candidate passes: the file must exist and be an executable file, and is refused when it is this
  executable (inode, canonical path or bytes), a launcher of it (record `.launch` beside it or beside the file its links
  reach), or a script that starts the TypeScript wrapper. Another release of the wrapper behind a native header passes
  the check; the arrival marker of the previous section stops it at its first instruction, as it does for a `PATH`
  candidate.
- A refusal returns the old failure with `RealGitNotFound.recorded`, and the message gains one line:
  `<path>: recorded by the commit service unit <unit> as the Git it starts, <verdict>`.
- The result type is `ResolvedGit { path, notice }`, so a caller cannot take the path and drop the notice. The notice
  travels as the leading text of the planned action's standard error (`action.rs` `with_leading_notice`): in
  `plan_invocation` for every forwarded command, query and commit; in `management.rs` for `doctor`, the direct commands,
  `service status` and `convert-worktree-to-subvolume`. The merge push that a stacked push plans inside the same process
  carries the stacking marker and does not repeat the notice.
- The service: `cli-git-service` is never started by the wrapper and takes its Git from `--git` in its unit. The wrapper
  writes that from the path it resolved, so a fallback path reaches the service as it reached the wrapper, and the
  service's own refusal of a launcher (previous section) still guards a unit written earlier. A service started with no
  `--git` keeps the default `/usr/bin/git`; the wrapper never starts it that way.

### The notice

Exact text (one line, ending in a newline), as `fallback_notice` writes it:

```text
cli-git: no real Git follows this wrapper on PATH (<PATH>); used <path>, the Git recorded in <unit> as the one the commit service starts; to silence this notice, add <directory of path> to the PATH of the program that ran this command.
```

It names what happened (no real Git on this `PATH`, which `PATH`), what was done (which file, from which unit) and the
one way to make it go away. An empty or unset `PATH` is written `empty or unset`. There is no "install Git" in the
notice, because Git exists. It is printed on every invocation that uses the fallback, because each invocation is a
separate process with nothing to say that an earlier one spoke; the person silences it by fixing `PATH`.

### Controls

Red first. Commit `7229bebd5a` added `binary_installed_git_fallback_tests.rs`; run against the unchanged resolver it
ended `7 tests run: 2 passed, 5 failed`: the two that passed were "no record keeps the stop" and "a real Git on `PATH`
is used and the record is neither started nor announced"; the five that failed were the fallback with a stand-in Git,
the fallback with the real Git, and the three refusals (the stop was already there, but the failure did not yet name
the record).

- `binary_installed_git_fallback_tests.rs` (built executable, hermetic `PATH` of the wrapper's directory only, the unit
  written with the product's own `unit_texts` below a fixture runtime directory; no control touches the real systemd
  user directory, the session bus, the production service or the real `.git`):
  `a_path_with_only_the_wrapper_and_a_recorded_real_git_runs_the_command_once_through_it_with_the_notice` (a counting
  stand-in is started once, Git's answer reaches the caller, the notice is the only line on standard error and names the
  `PATH`, the file and the directory to add),
  `the_recorded_real_git_answers_a_command_that_reads_the_repository_and_the_notice_stays_off_standard_output`,
  `a_real_git_on_path_is_used_and_the_record_is_neither_started_nor_announced`,
  `no_record_keeps_the_stop` (no unit, and no runtime directory variable),
  `a_record_naming_the_wrapper_or_a_copy_of_it_is_refused_and_keeps_the_stop` (the wrapper file, the `git` link to it, a
  byte copy),
  `a_record_naming_a_launcher_of_the_wrapper_is_refused_without_a_start_and_keeps_the_stop` (the counting stand-in
  launcher is started zero times),
  `a_record_naming_a_missing_or_non_executable_file_is_refused_and_keeps_the_stop`.
- `real_git_fallback_tests.rs`, `real_git_record_tests.rs` and `entry_fallback_tests.rs` (116 unit tests passed in the
  run of these files with the resolver, entry and management neighbours, 164 after adding the launcher, marker,
  provisioning and announcement neighbours; the 7 binary controls and 15 neighbours passed, clippy `-D warnings` clean):
  the resolver, the reading of
  the unit (the product's text, bounds, relative path, first line wins), the inputs from `XDG_RUNTIME_DIR`, the notice
  in the plans of a version query, a plain forward, a help and `service status`, no notice for a management command
  that needs no Git, no repeat for the merge push of a stacked push, and `with_leading_notice` itself.
- Every run of the built executable is bounded by the shared supervisor (`binary_supervisor.rs`) and by the
  `systemd-run --user --scope` that held the test command, with `TasksMax`; the counting stand-ins end a chain by
  themselves.

### Open questions for the owner

- Veto: keep the stop instead of the fallback. Options: keep the fallback (picked), return to the stop of the previous
  section. Ranking: fallback > stop, because a found wrapper that cannot run Git is the failure the owner's rules name,
  and the notice keeps the cause visible; the stop is the safer reading if the owner wants `PATH` mistakes loud.
- A record that survives logout (option B above). Ranking: leave as built > build B, because B adds state and a cost on
  every command for a limit the notice already tells the person how to remove; B first if the owner finds the
  logged-out case common.
- A fixed `/usr/bin/git` for the case that no unit exists yet. Ranking: no > yes, because the instruction was a
  recorded path and not a guess, and a wrong Git is worse than a stop with a message.
- The default `--git /usr/bin/git` of `cli-git-service` when the option is missing, and its refusal of a `--git`
  that is only a launcher (it still accepts the wrapper itself, a copy, or a missing file). Ranking: leave > tighten
  to the checks of a `PATH` candidate > remove the default, because the wrapper always writes `--git` from the path
  it resolved, so the service's own checks guard only a hand-written or older unit; tightening is a small separate
  change, and removing the default would break development runs.
- One notice per invocation versus once per login. Ranking: per invocation > once, because a once-only notice needs a
  file that outlives the process and a decision about when it expires.

## Hang analysis: the owner's suspicion of a concurrent LFS deadlock

The owner's words: "I suspect sometimes the wrapper hangs in post-commit auto-push due to concurrent LFS deadlock.
Either fix it or prove me wrong." No stall was observed; the notice `waiting for process N to finish pushing ...;
giving up after N seconds` is printed when a wait begins and names its limit, so it records a queue and not a
give-up.

### Lock graph of the post-commit auto-push

Reading of `commit_command.rs`, `post_commit.rs`, `auto_push.rs`, `auto_push_single_flight.rs`, `bounded_push.rs`,
`process_tree.rs`, `lfs_pre_push.rs` and `owner_lock.rs`:

- Startup recovery runs in every wrapper process before configuration loads. It takes no lock that a push holds.
- The commit transaction holds the landing lock (`commit_landing_loop.rs`) while it lands; auto-push starts after
  `run_commit_transaction` returns, so the landing lock is not held during the push.
- Auto-push holds one per-branch owner lock (`<common dir>/cli-git/push/<key>.lock`) from its decision to push until
  it has written the record. Under it run: bounded queries, one bounded push (with the upload inside Git), and, for a
  stale rejection only, up to `MAX_RECONCILE_ATTEMPTS` rounds of one bounded fetch plus one bounded push.
- A waiter polls that directory and gives up at `lock_wait_limit`. A dead or reused holder is retired by
  `owner_lock_holder_is_alive` (checks the process birth identity, treats zombies as dead, and handles another PID
  namespace).
- The processes below the push (Git, `git-lfs`, a transfer agent, the Git commands `git-lfs` runs) take Git's own
  locks (`index.lock`, ref locks) and LFS's own files. None of them takes or waits for a wrapper lock: since this
  change, none of them can start the wrapper at all. In the old design the nested start at the hook entry
  (`entry.rs:289`) ran after `hooks_off::activate()` and before recovery, configuration or any lock, so it also could
  not wait for a lock the parent held.

So the wrapper's wait-for graph has no cycle through the upload: the push tree waits for nothing the wrapper holds, and
the wrapper waits for the push tree only through bounded waits.

### Experiments

- `binary_lfs_hang_tests.rs`
  `two_worktrees_committing_at_once_with_a_slow_upload_both_finish_without_waiting_for_each_other`:
  two worktrees of one repository commit at the same moment, both with LFS keys and a stand-in `git-lfs` that takes
  its delay. Both exit 0, neither prints a wait notice, both branches reach the origin, nothing is left. Not
  reproduced: no hang.
- `a_detached_upload_helper_holding_the_output_open_does_not_hold_the_commit`: the stand-in starts a helper in a new
  session whose parent has gone, holding the push's output file open, and itself never exits. The wrapper stops the
  push at its deadline and returns within the deadline plus `STOP_GRACE`. Refuted: the push output goes to an
  anonymous regular file that is read to its current end (`bounded_push.rs` `read_output`), so no holder of the file
  can delay the read. The helper itself is a leak that the tree walk cannot see after its parent exited; it is a leak
  and not a hang. Making such helpers findable would need the wrapper to become a child subreaper, which makes it
  responsible for reaping adopted processes; not done.
- Every other pipe read in the source (`git_metadata.rs`, `manual_push_probe.rs`, `manual_push_range.rs`,
  `child_watch.rs`, `candidate_reader.rs`, `index_lock_holders.rs`, `process_identity.rs`)
  is outside the auto-push path.
  Two of them, the probe's dry-run push and its `ls-remote`, have no deadline of their own and run before an explicit
  push; a remote that never answers would hold such a push until Git's own timeouts end it. This is a finding for the
  owner, not changed here.

### The four hypotheses

- Detached descendant keeping the pipe open: refuted above for the auto-push (a file, not a pipe).
- Waiting for EOF after the kill: the stop path killed the child and then called a blocking `wait`. A child in
  uninterruptible I/O is not collected until the kernel releases it, so that wait could outlast every bound. Fixed:
  `stop_push` now polls `try_wait` for at most the grace and leaves the child to be collected when the process ends.
  Not reproducible without a stuck kernel state; covered by the unchanged stop controls.
- A stalled transfer holding the push for the whole deadline while waiters queue: the first waiter joins the failed
  record when the holder's push is stopped. What was found by reading is that the waiter's limit covered one push,
  while a holder that meets a stale rejection ran up to `MAX_RECONCILE_ATTEMPTS` rounds of a fetch and a push, and now
  an upload as well, under the same lock, so a legitimate holder could outlast a waiter, which then reported a timed-out
  wait while the holder was still working. Fixed: `holder_budget.rs` gives everything a holder does under the lock one
  time budget (`HOLDER_BUDGET_DEADLINES` push deadlines, `bounded_push::holder_budget`); every bounded step is cut to
  what is left (`run_bounded_push`, `run_bounded_query`), no step starts once the budget is spent, and the waiter's
  default limit is derived from it: the budget plus two stop graces (the step in flight when the budget runs out
  needs the grace to end and the kill's collection another) plus `LOCK_WAIT_MARGIN`. Controls:
  `the_lock_wait_outlasts_the_holders_whole_push`, `a_step_after_the_budget_is_spent_is_not_started` and
  `a_step_under_a_budget_is_stopped_when_the_budget_runs_out`. The owner's own `CLI_GIT_PUSH_LOCK_WAIT_SECONDS` still
  replaces the derived limit.
- Liveness of a holder record: the check compares process birth identity, treats zombies and exited processes as dead
  and has a rule for another PID namespace (`process_identity.rs`, `owner_lock_record.rs`). A holder that is alive and
  stuck is outlasted by the waiter's limit and then reported and left in place. Refuted as a source of a hang.

### The two failures under load

- `an_auto_push_to_a_server_that_hangs_up_keeps_the_commit_and_holds_nothing` failed at about 9.4 seconds in four
  containers at once. The panic text was not captured. This host shows 0.5 seconds idle and 1.3 seconds under a
  quarter-CPU cap (`systemd-run --user --property=CPUQuota=25%`), which is not enough slowdown to reproduce.
  No 10-second bound was found in the test: the wrapped commits are bounded by `COMMIT_BOUND_SECONDS`,
  the server and tripwire by `DEADLINE_MILLISECONDS`, the direct `git` helper calls have no bound, and
  its second commit sets a one-second push deadline, which is also the one-second deadline of every Git query of that
  commit (`query_deadline` is the smaller of the push deadline and `QUERY_CEILING`), so a query slowed past a
  second under load makes that commit report that it could not run. The closing checks of
  `assert_nothing_left` (no shadow repository, `garbage: 0` in `count-objects`, no pack leftovers) are also load
  sensitive candidates. Which of these failed is not determined; the next gate failure needs `--failure-output final`
  to capture the text. Nothing observed points to a real stall.
- `an_auto_push_with_a_git_lfs_that_never_exits_is_stopped_with_everything_it_started` uses the same one-second
  deadline for the commit and then the second commit; the same remark applies.

### Every bound that limits a hang

- `CLI_GIT_PUSH_DEADLINE_SECONDS` / `DEFAULT_PUSH_DEADLINE` in `bounded_push.rs`: one push, fetch or upload (30
  seconds on main at the time of writing).
- `HOLDER_BUDGET_DEADLINES` (`bounded_push.rs`) push deadlines: all of a lock holder's steps together.
- `STOP_GRACE` (5 seconds): after the polite stop and again after the kill.
- `QUERY_CEILING` (120 seconds) with `query_deadline`: every Git query of the auto-push.
- `LOCK_WAIT_MARGIN` (60 seconds) and `lock_wait_limit` (holder budget, two stop graces, margin);
  `CLI_GIT_PUSH_LOCK_WAIT_SECONDS`: a waiter.
- `MAX_PUSH_OUTPUT` (4 MiB): a flooding push is stopped.
- `LOW_SPEED_LIMIT` and `LOW_SPEED_TIME` (1000 bytes per second for 60 seconds): Git ends a stalled HTTP transfer.
- `MAX_RECONCILE_ATTEMPTS` (10): rounds of merge and push under one lock.
- `CLI_GIT_CHILD_WAIT_SECONDS` and `CLI_GIT_LOCK_WAIT_SECONDS` (1800 seconds in the notices): other children and the
  index and landing waits.
- The hook trace read stops at 16 MiB.

The waiter's notice now also says how long the holder has held the lock, the holder's command line (Linux), and that its
push is stopped at the push deadline (`lock_holder_facts.rs`).

## Review follow-up and corrective note

An independent review of head `02b4862916` (text: `review-lfs.md` in the scratch directory) found, and this branch fixed
on new commits, the gaps that the earlier commit messages do not mention. Commit messages are not amended; this note is
their correction.

- `6e747cd1a4`, `26627b5bcc` and `156415d696` describe a plain hook command with a trace2 reading. That design was
  removed by `9a03d59467`; no injected hook or trace reading exists at head.
- `9a03d59467` says the remote and refs come from the push arguments. It also needed `remote.<name>.push`, the first
  positional winning over `--repo`, `--repo` abbreviations, path remotes, tag and glob expansion, the full-ref branch
  name, the global `-c` options and `--git-dir` reaching `git-lfs`, signals reaching it, and the caller's `PATH` order
  for `git-lfs`. All are read or passed now (`lfs_upload_plan.rs`, `lfs_upload.rs`).
- `02b4862916` says the probe parses Git's own report and keeps the race check. The report was refused when Git added
  its `Would set upstream` line (`-u`, `--set-upstream`, `push.autoSetupRemote` all failed), and `-q` emptied it, which
  skipped every manual-push policy. Both are fixed with tests (`binary_push_probe_tests.rs`, the probe unit tests);
  the race check does not cover a deletion or a refusal, as the commit says.
- A holder-budget guard had no test; three now fail when the guard is removed (`enter`, the query guard, the early
  end of the budget). A step the budget left no time for now says so instead of "did not finish".
- The wait notice shows only the holder's program, its subcommand and its working directory.
- Submodules: with `--recurse-submodules=on-demand` or `only` Git pushes the submodule itself and the wrapper does not
  upload a submodule's LFS objects; it prints a notice naming the way forward instead of refusing a push Git accepts.
- Git starts per manual push, new branch, fast-forward, bare push: measured in the review at 20, 21 and 23 against
  baselines 18, 20 and 20. This branch drops the push URL query for a single destination; the numbers after that
  change are in the follow-up report. The probe's dry run and `ls-remote` are bounded by the push deadline (30 seconds
  by default, `CLI_GIT_PUSH_DEADLINE_SECONDS`), which is the same bound the push itself would get.

## Second review (static, cross-vendor) and what was done

Each finding was first reproduced with real Git 2.56.0 and real git-lfs 3.8.1 in a throwaway repository (a local LFS
server logs every batch and upload) against the merged head; the fixes were run against the same scenarios.

- Reproduced and fixed: pointers published without objects when a remote-tracking ref of this clone hides an object the
  destination lacks (a changed push URL; without `--all` git-lfs scans only past the tracking refs); a source with a
  colon (`:/message:refs/heads/new`) read as a deletion, in the plan and in the probe; the matching refspec `:` read as
  a deletion; a pattern whose star crosses `/` (`feature/nested`) missed; `remote.<name>.mirror`; several push URLs; a
  relative path typed as the remote; a relative `PATH` entry under `-C`; a failed ref listing read as "nothing to
  upload" (now an error and a refusal); a credential-stripped `To` address used to reach the remote in the probe (now
  only a typed URL or path is its own location, a remote name asks for its push URLs).
- Could not happen end to end: a valueless `-c hook.cli-git-lfs-pre-push.enabled` (real Git itself refuses the push with
  `missing value`); the switch's name is now matched as Git matches it (section and variable ignore case, the subsection
  is exact) and a later value overrides an earlier one.
- Test findings fixed: the two-worktree control now needs the uploads to overlap (each waits for the other to start) and
  compares each branch with its new commit; a signal sent to the wrapper is relayed and the child is gone; the
  `push.autoSetupRemote` control is a bare push of a branch with no upstream and checks the upstream.
- Not changed: the interactive upload has no deadline of its own (a person is watching it and can end it with a signal,
  which is relayed); the lock waiter limit (finding 12) belongs to code that the lock-free service replaces; the
  `git` start counts are as before: the plan reads its configuration from the snapshot already taken (no start), a
  named remote costs one `remote get-url` in the probe, and `--all` makes git-lfs scan the pushed refs fully.

## The bans

- `clippy.toml` in `package/git-policy/cli` bans `std::env::current_exe` (the file also repeats the repository root
  file's entries, because Clippy reads the nearest file only). Remaining uses carry
  `#[allow(clippy::disallowed_methods, reason = ...)]` naming what they identify.
- `src/native/no_self_invocation_tests.rs` scans the release source for: the running executable's path, `argv[0]`
  reads, shell text, any hook or hook command definition, management words passed to a process,
  and `Command::new` on the wrapper's own path.
  Each allow-list entry names its file and rule and must still match code.
- Both guards were shown to fail: a planted `std::env::current_exe()` failed Clippy with the rule's note, and planted
  shell and hook-command text failed the scan with `file:line [rule]`. Both plants were removed.

## Phase 2: the hook dispatcher

Filled by the change that removes the argv[0] dispatcher and the commit-path hook dispatch.
