# Hook-route findings on `main`

Status:
audit,
2026-11-09T02:30:00-05:00.
Audited revision:
`main` at `4fcd560cea`.

The reading was made through the worktree
`/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/pi-hook-route-findings-on-main`
(head `70b67c6ef2`,
fast-forwarded to `main` before this page was written).
Every source file this page cites is identical on `main` and on that head:
`git diff --stat HEAD main --` over the route's files reported nothing.
Linux only;
macOS and Windows branches are named where they exist and are not audited,
which the owner's "Ignore Win and mac until I say so."
covers.
No production code was changed and no test was committed.
The one control was appended to the test list of `src/native/push/mod.rs`,
was run through `mise run //package/git-policy/cli:native:test:host`,
and was removed after its output was taken;
a copy is in `${HOME}/temp/agent/pi-hook-audit/hook_route_audit_tests.rs`,
and its output is quoted in "Reproductions".

Scope:
every place the wrapper starts,
skips,
shims or installs a Git hook,
found with
`rg 'hooksPath|hook_shim|run_hook|hooks_off|NO_HOOKS' package/git-policy/cli/src/native`:
`invocation/hooks_off.rs`,
`invocation/forwarding.rs`,
`invocation/entry.rs`,
`transaction/transaction_git.rs`,
`push/auto_push_single_flight.rs`,
`push/bounded_push.rs`,
`push/stop_of_a_push_and_sweep_of_its_locks.rs`,
`push/manual_push_probe.rs`,
`commit/commit_preparation.rs`,
`shadow/shadow_git.rs`,
`health/traditional_hook_warning.rs`,
`health/external_hooks.rs`,
`health/doctor.rs`,
`service/service_main.rs`,
plus the two binary suites that pin the behavior
(`binary/binary_hooks_off_tests.rs`,
`binary/binary_hooks_off_escape_tests.rs`)
and the guard that keeps a hook entry out of the commit path
(`source_guard/hook_self_invocation_tests.rs`).

Purpose:
the four owner rules this route carries are
"Commits should never fail.",
"never discard user data",
"nothing should ever wait forever or indefinitely",
and
"We should never invoke ourselves recursively...".
The owner's decision on hooks themselves is
"Only hooks we declare,
run natively;
no external hook commands"
("Stop supporting them and port anything in.",
chosen label
"B:
Only hooks we declare,
natively",
2026-11-01T02:13:03-05:00,
`doc/handover/cli-git-rust-implementation.md`,
"Hooks and pushes",
and `SPEC.md`,
"Real Git starts with traditional hooks off").
This page answers,
for that decision,
which hooks still run and where,
what ends the wait when one hangs,
what a declined hook leaves behind,
and which of the route's own statements are not true of the code.

## What the wrapper does with hooks

The wrapper runs no hook program of its own and declares none.
Two argument-level switches carry that,
and a third fact limits it:

- Every real Git the wrapper starts gets
  `-c core.hooksPath=/dev/null/cli-git-no-hooks`
  (`invocation/hooks_off.rs:43`,
  a directory below a device,
  so nothing can create it)
  and `-c hook.<event>.enabled=false` for each of the 28 hook events of Git 2.56.0
  (`invocation/hooks_off.rs:46-77`).
  The first stops a hook file in `.git/hooks` or in a configured hooks directory.
  The second stops a hook the configuration defines
  (`hook.<name>.event` with `hook.<name>.command`),
  including a hook whose friendly name collides with an event name.
- Both switches are placed after the caller's global options and before the subcommand
  (`invocation/hooks_off.rs:174-199`),
  so a `-c`,
  a `--config-env` or an inherited `GIT_CONFIG_PARAMETERS` cannot put a hook back;
  measured on Git 2.56.0 in "What I checked and found sound".
- `git config` starts that only report or change configuration go without the switches
  (`invocation/hooks_off.rs:100-131`),
  because the wrapper itself reads `core.hooksPath` that way,
  and `git config --edit` keeps them because its editor inherits them.

The result,
for the three commands the brief names:

- A commit runs no hook at all.
  The native commit is built in the shadow repository with a private index copy
  (`commit/commit_preparation.rs:139-146`),
  and it repeats the hooks path after the caller's global options and after the shadow's own,
  so the shadow's config and the caller's `-c` both lose.
  A forwarded `git commit` is exec'd with the same switches.
- A merge runs no hook:
  the wrapper's isolated merge works from objects,
  a merge commit it makes skips commit-time hooks by construction (`SPEC.md:3627`),
  and a forwarded `git merge` carries the switches.
- A push runs no client-side hook,
  `pre-push` included,
  for the typed push,
  for the auto-push,
  for the wrapper's three dry-run probes,
  and for the stacked push.
  A push to a local path or `file://` is the one case where a hook of the person's runs:
  the setting does not follow the push into the remote,
  so the remote's own `receive-pack` runs its `pre-receive`,
  `update` and `post-receive`
  (`binary/binary_hooks_off_tests.rs:233`,
  `SPEC.md:4592-4594`).

## Where a hook is skipped or started, and what bounds the wait

### The skip

No process is started for a hook,
so there is no wait to bound on the client side.
`hooks_off::activate()` runs once per process before any start
(`invocation/entry.rs:342`,
`service/service_main.rs:257`),
and `hooks_off::add_arguments` adds the switches to every start built by
`invocation/forwarding.rs:86-103`
(`run_real_git` at `:187`,
`replace_process_with_real_git` at `:230`)
and by `push/auto_push_single_flight.rs:153-160`.
Every transaction start reaches it through
`transaction/transaction_git.rs:113` (`build_command`)
and `push/auto_push_single_flight.rs:153`.

The waits those starts get are the ordinary step bounds,
not hook bounds:

- A transaction step started at `transaction/transaction_git.rs:388`
  (`run_git_watched`)
  is stopped on a stall,
  at the ceiling of its class,
  or by a stop of the service,
  and the paths that may be run twice are run again once
  (`transaction/transaction_git.rs:347-380`).
- The native commit started at `commit/commit_preparation.rs:188-221`
  has `SCRATCH_COMMIT_STEP_CEILING` when the message came with the command
  and none while an editor is open.
- A forwarded command is exec'd
  (`invocation/entry.rs:367`),
  so the wrapper is gone and nothing of its own can end it.
- An announced query waits with inherited streams and no watch
  (`invocation/forwarding.rs:187-215`).

### The one place a hook of the person's runs

A push to a local path starts the remote's `receive-pack`,
and the hooks of that remote are children of the push.
When the wrapper runs the push for itself
(the auto-push,
and the service's push and its retry plan),
that push is started by `push/bounded_push.rs:408` and watched by the stall clock:

- the output goes to anonymous files
  (`push/bounded_push.rs:426-429`),
  so a hook that keeps an inherited stream open,
  or leaves a child holding it,
  holds nothing up;
- the window is 30 seconds of no sign of work with no connection open
  (`push/push_deadline_and_patience_settings.rs:51`,
  `CLI_GIT_PUSH_DEADLINE_SECONDS` overrides it);
- past it,
  `push/bounded_push.rs:439` calls the stop,
  which records the push's descendants before signalling
  (`push/stop_of_a_push_and_sweep_of_its_locks.rs:170`),
  sends `SIGTERM` to the push and to every descendant
  (`:186-187`),
  waits the grace,
  and sends `SIGKILL` to every descendant
  (`:213`).
  The comment at `:207` names the hook as one of the things that may never exit.

A typed push is exec'd,
so it has no watch and its remote hook is never signalled by the wrapper.

### The two warnings and the scan

- The commit prints one non-blocking `configuration-warning` when the configuration sets
  `core.hooksPath` to anything but the wrapper's own value
  (`health/traditional_hook_warning.rs:140`,
  `:118-121` for the suppression).
  It reads only the configuration snapshot
  (`config/config_snapshot.rs:612`,
  the query at `:645-648`),
  so it costs no process and no file system call,
  and it never looks at a directory.
- The commit prints one non-blocking `configuration-warning` when the configuration defines
  `hook.<name>.event` or `hook.<name>.command`
  (`health/external_hooks.rs:203`),
  and drops a hook whose own `hook.<name>.enabled` is false
  (`:141-147`).
- `git monochromatic-git-policy-cli doctor` scans
  `<git-common-dir>/hooks` and the directory a configured `core.hooksPath` names
  (`health/doctor.rs:96-108`),
  locating them with three Git queries
  (`:274-320`),
  and reports an executable regular file or a link per line
  (`:154-181`),
  with exit 0 for none or only Git LFS shims,
  1 for any other hook
  (`:494-518`),
  and 2 when the repository cannot be located
  (`:537-543`).
  A missing directory and a path that is not a directory both count as no hooks
  (`:157`),
  so `core.hooksPath=/dev/null` is handled.

## Findings ranked by what they risk

### 1. A stopped push ends a local-path remote's hook after the refs have landed

Where:
`push/stop_of_a_push_and_sweep_of_its_locks.rs:168-217`,
reached from `push/bounded_push.rs:439`,
against `SPEC.md:2532`,
`:2995`,
and `push/child_watch.rs:36-39`.
Reproduced,
in "A stalled `post-receive` hook".

The sequence:

1.  The repository's `origin` is a local path or a `file://` URL whose target holds a
    `post-receive` hook,
    for example a deploy,
    a mirror,
    or a ledger append.
2.  A commit's auto-push,
    or the service's push of a landed commit,
    starts `git push` through `push/bounded_push.rs:408`
    with the default 30 second window,
    or the window `CLI_GIT_PUSH_DEADLINE_SECONDS` sets.
3.  The push delivers the pack and the remote updates the ref,
    and then runs `post-receive`,
    which waits on something that shows no sign of work in the push tree:
    a lock another process holds,
    a queue slot,
    a person's approval,
    a `sleep`,
    or a wait on a stalled disk that `/proc/<pid>/io` cannot show.
4.  The window passes with no bytes and no processor time anywhere in the tree,
    so `push/bounded_push.rs:439` stops the push,
    and `stop_of_a_push_and_sweep_of_its_locks.rs:187` and `:213` end the hook with
    `SIGTERM`,
    then `SIGKILL` after the grace.

The wrong result:
the ref is on the remote and the commit is published,
the person's hook is killed in the middle of its work,
and the person is told the push failed.
The reported line is the one quoted in "A stalled `post-receive` hook":
it says the push made no progress,
that the service keeps it in its retry plan,
and to push again.
Pushing again does not help and does not restart the hook:
measured on Git 2.56.0,
a push with nothing to update runs no server hook at all,
so the interrupted deploy or ledger append is simply never finished,
while the branch says it was published.
The same stop also ends a `pre-receive` hook of the person's,
so the statements that defend the watch,
`SPEC.md:2532` ("The person's hook is never ended."),
`:2995` ("Hooks are never ended by the wrapper."),
and the module comment at `push/child_watch.rs:36-39`,
are not true of a local-path push.
`SPEC.md:3786` records the stop itself for this case
("its hook is a child of the push") and stops it "as before",
so the code follows that sentence and the other two are overbroad wording,
not a violated decision.
What no record states is the consequence above:
the branch is published and the command still reports failure,
`SPEC.md:2540-2542` names the condition under which the broad sentences would be true
("If hooks are ever started below a watched step,
a descendant that is a hook must count as work"),
and the push route's own audit repeats the broad sentence
(`doc/planning/cli-git-push-route-findings-on-main.md:1333-1337`,
"so a push cannot wait on a prompt or on a hook nobody sees"),
so the wording has already spread.
The code has no way to tell a hook from Git,
which is what `SPEC.md:2536-2538` says it never needed.

Suggested fix (a suggestion,
not a decision):
one of these,
in the order that keeps the most of the owner's "Stop only on no progress":

- treat a local-path push as connected and give it no stall stop,
  since its remote's work is in the same process tree and has no connection to watch;
- or count a `receive-pack` that has reached its hook phase as work,
  which is observable from the push's own output
  (`remote:` lines and the sideband are already parsed for refusals);
- or,
  if the stop stays,
  say plainly in `SPEC.md`,
  `README.md` and the stop's line that a local remote's hooks are ended with the push,
  and take the sentence out of `push/child_watch.rs`,
  so the next reader does not build on it.

### 2. Inside a nested invocation, `doctor` and the commit warning read the wrapper's own hooks path

Where:
`invocation/hooks_off.rs:174-199` (the switches travel as command-scope configuration),
`health/doctor.rs:287` (a raw `git config --type=path --get core.hooksPath`),
`health/traditional_hook_warning.rs:118-121`
(the warning,
which suppresses the wrapper's own values),
and `health/external_hooks.rs:141-147`.
Read from the code;
the inheritance itself is reproduced in "The switches reach a child Git".

The sequence:

1.  A Git the wrapper started runs a program of the person's:
    an editor of `git config --edit`,
    a clean or smudge filter,
    an alias the person wrote with `!`,
    or a pager.
    Every one of them inherits the wrapper's `-c` switches as
    `GIT_CONFIG_PARAMETERS`
    (`binary/binary_hooks_off_escape_tests.rs:124` proves the editor case),
    and a `git` it starts through `PATH` is the wrapper again,
    which `SPEC.md:4661-4664` relies on on purpose.
2.  That nested wrapper answers a configuration question.
    `doctor`'s `core.hooksPath` query
    (`health/doctor.rs:287`)
    and the commit warning's snapshot query
    (`config/config_snapshot.rs:645-648`)
    go without the wrapper's switches by design,
    so they see the inherited command-scope entries as the highest-precedence values.
3.  Whether the in-process reader answers or declines decides what is read:
    it honours `GIT_CONFIG_COUNT`/`GIT_CONFIG_KEY_n`
    (`repo_read/repo_read_query_refs_tests.rs:51-65`)
    and does not read `GIT_CONFIG_PARAMETERS`,
    and it declines a listing with a conditional include
    (`:66-68`).
    When it declines,
    the answer comes from real Git,
    whose last entry for the key is the wrapper's own
    `/dev/null/cli-git-no-hooks`.

The wrong result:
with a conditional include anywhere in the configuration
(`includeIf.gitdir:`,
`includeIf.onbranch:` or `includeIf.hasconfig:`,
which the reader leaves to Git),
a repository that keeps its hooks in `core.hooksPath=.myhooks`
gets no commit warning at all
(the wrapper's value is recognised and suppressed at
`health/traditional_hook_warning.rs:119`),
and `doctor` scans `/dev/null/cli-git-no-hooks` instead of `.myhooks`,
lists nothing,
and exits 0,
so the one command that exists to find these hooks reports there are none.
The crate holds no reader of `GIT_CONFIG_PARAMETERS` at all
(`rg 'GIT_CONFIG_PARAMETERS' package/git-policy/cli/src` finds it only in prose,
at `lfs/lfs_upload.rs:16` and `health/doctor_fsmonitor.rs:180`),
so a query the in-process reader does answer behaves as if the entry were absent,
and the two readers of one key then disagree.
That disagreement is read from the code and was not run;
the conditional-include case is the one that reaches real Git,
which is what makes the wrong result above happen.

Suggested fix (a suggestion,
not a decision):
filter the value the way the warning already does,
with `is_wrapper_hooks_path`,
at `health/doctor.rs:287`
and in `configured_hooks`,
so a value the wrapper itself injected is never read as the person's,
and add a control that runs `doctor` from inside a wrapper-started editor.

### 3. A hook file that is gone by the time it is looked at fails the whole `doctor` run

Where:
`health/doctor.rs:154-181`,
the `symlink_metadata` at `:168`,
and `find_traditional_hooks` at `:189-196`.
Read from the code only;
no seam exists to place the removal between the listing and the look.

The sequence:

1.  `git monochromatic-git-policy-cli doctor` lists a hooks directory
    (`health/doctor.rs:155`) and gets the entry names it holds.
2.  Before the loop reaches one of them,
    that name is removed or replaced by something that no longer resolves:
    the person deletes `pre-commit` in another terminal,
    a script reinstalls the hooks,
    or a `git-lfs` upgrade rewrites the directory,
    which the same command's own text says happens again and again
    (`SPEC.md:482-484`).
3.  `std::fs::symlink_metadata(found.path())` returns `ENOENT`,
    and the `?` at `:168` ends the scan with an error.

The wrong result:
`doctor` prints `git monochromatic-git-policy-cli doctor: <io error>` and exits 2
(`health/doctor.rs:537-543`),
so a run that had already found the repository reports that it could not examine it,
lists none of the hooks it did look at,
and says nothing about the ones that are there.
The report a person reads after a commit warning is then empty of information while the
hooks are still in place.

Suggested fix (a suggestion,
not a decision):
treat `ErrorKind::NotFound` from `symlink_metadata` as "that entry is not a hook now"
and go on,
as the same function already does for a missing directory at `:157`,
and keep every other failure an error.

### 4. A configured hook the author already switched off at the event level is still reported

Where:
`health/external_hooks.rs:141-147`,
with the warning it feeds at `:203`,
and `git hook list <event>`,
which Git 2.56.0 uses to report the same state as `event-disabled`.
Read from the code;
the Git side is reproduced in "The event switch disables every hook of its event".

The sequence:

1.  A repository defines `hook.control.event=pre-commit` and
    `hook.control.command=<script>`,
    and then switches the event off the way Git offers:
    `git config hook.pre-commit.enabled false`.
2.  Every wrapper command that runs a hook event passes
    `-c hook.pre-commit.enabled=false` anyway (`invocation/hooks_off.rs:46-77`),
    so the hook never runs,
    and Git reports it as `event-disabled`
    (`git hook list pre-commit`).
3.  The wrapper's own filter asks only about `hook.<friendly-name>.enabled`
    (`health/external_hooks.rs:143`),
    not about the event,
    so `configured_hooks` keeps the hook.

The wrong result:
every commit prints the external-hooks warning,
naming `hook.control` and its event,
and tells the person to remove keys that ask for nothing,
and `git monochromatic-git-policy-cli doctor` lists the hook and exits 1
(`health/doctor.rs:516`),
so a script that gates on `doctor`'s status fails for a repository that has already
disabled what it is being told about.
The same filter also keeps a hook that has an event and no command,
which Git never runs.

Suggested fix (a suggestion,
not a decision):
drop a hook whose every event is one the snapshot holds
`hook.<event>.enabled=false` for,
and drop a hook with no command,
in the same loop that drops `hook.<name>.enabled=false`.

## Reproductions

All fixtures are throwaway directories under `${HOME}/temp/agent/pi-hook-audit`.
The real Git used in the probes is `/usr/bin/git` (Git 2.56.0),
which the crate's own controls do too
(`testing/test_support.rs:19`).

### A stalled `post-receive` hook

The control builds a work repository,
a bare remote with a `post-receive` hook that records itself,
writes its own pid,
and then sleeps,
and runs the wrapper's own bounded push
(`push/bounded_push.rs:361`)
with a two second window instead of the default 30,
so the control is quick and nothing depends on timing luck.
It asserts that the hook ran,
that the remote's `refs/heads/main` equals the local `HEAD`,
that the attempt's exit code is `STOPPED_EXIT_CODE`,
and that the hook's pid no longer exists.
It passed:

```text
mise run //package/git-policy/cli:native:test:host -- hook_route_audit --no-capture
Compiling monochromatic-git-policy-cli v0.6.1-alpha.5 (...)
 Finished `test` profile [unoptimized + debuginfo] target(s) in 3m 52s
 Starting 1 test across 1 binary (4870 tests skipped)
running 1 test
THE PUSH REPORTED: monochromatic-git-policy-cli: git push made no progress for 2 seconds and was stopped (limit=stall, bytes_moved=46026, elapsed_seconds=3); set CLI_GIT_PUSH_DEADLINE_SECONDS to allow longer. The push is queued: the commit service keeps it in its retry plan and tries it again. Hint: if you were pushing over a slow or offline connection, your commit is already saved; push again when the connection works.
test push::hook_route_audit_tests::a_stalled_post_receive_hook_leaves_the_branch_published_and_the_push_failed ... ok
        PASS [   3.094s] (1/1) push::hook_route_audit_tests::a_stalled_post_receive_hook_leaves_the_branch_published_and_the_push_failed
     Summary [   3.097s] 1 test run: 1 passed, 4870 skipped
```

The control passed because its assertions describe what the code does,
not what the page above says it should do.
`bytes_moved=46026` is the pack the push delivered.

A second probe,
on real Git alone,
shows that the advice in that line cannot repair the hook:
a push with nothing to update runs no server hook.

```text
$ /usr/bin/git push --porcelain origin main          # already up to date
To ../remote.git
=	refs/heads/main:refs/heads/main	[up to date]
Done
runs of pre-receive: 0
```

### The switches reach a child Git

The wrapper's `-c` values become `GIT_CONFIG_PARAMETERS` in every child,
and the child's own `git config --get` reports them,
which is what finding 2 rests on:

```text
$ /usr/bin/git -c core.hooksPath=/dev/null/cli-git-no-hooks \
    -c alias.probe='!/usr/bin/git config --get core.hooksPath' probe
/dev/null/cli-git-no-hooks
```

### The event switch disables every hook of its event

`git hook list` reports the state Git would act on,
for a hook named `control` and one named `bar`,
both on `post-checkout`,
and a hook named `y` on `pre-commit`:

```text
$ /usr/bin/git -c hook.post-checkout.enabled=false hook list post-checkout
event-disabled	control
event-disabled	bar
$ /usr/bin/git -c hook.post-checkout.enabled=false hook list pre-commit
y
$ /usr/bin/git -c hook.y.enabled=false hook list pre-commit
disabled	y
$ /usr/bin/git -c hook.y.enabled=false hook list post-checkout
control
bar
```

A traditional hook file is not covered by that switch and needs the hooks path,
which is why the wrapper passes both:

```text
$ /usr/bin/git -c hook.post-checkout.enabled=false checkout main   # file hook
file-hook-ran
$ /usr/bin/git -c core.hooksPath=/dev/null/cli-git-no-hooks checkout main
(no marker)
```

## What I could not determine

- Whether a hook that starts the wrapper in the same repository can wait behind the outer
  command's own work.
  A hook's `git` is not stopped by the recursion guard on purpose
  (`SPEC.md:4661-4664`),
  so a `post-receive` hook that runs `git commit` through the wrapper in the pushing
  repository starts a wrapper command of its own while the push is in flight.
  Whether that second command waits for the service,
  and whether the service waits for the push in turn,
  needs the built service and its socket,
  which this audit did not run.
  The stall stop above would end such a wait,
  so the outcome would be a killed hook rather than a hang,
  but I did not measure it.
- Whether Git 2.39,
  which the container gate runs,
  has the event-level `hook.<event>.enabled` at all,
  and whether its local-path `receive-pack` also ignores the setting.
  Both were measured on 2.56.0 only.
  If 2.39 lacks the event switch,
  a configured hook would run under the wrapper there,
  which no gate would notice,
  because `binary_hooks_off_tests.rs` runs on the host's Git.
- Whether the 30 second window is long enough for the hook a person actually writes.
  No log of a stopped push that reached a remote's hook exists,
  and I did not read the timing records for one.

## What I checked and found sound

- The event list is complete for Git 2.56.0:
  `invocation/hooks_off.rs:46-77` lists all 28 names of
  `Documentation/githooks.adoc` at `v2.56.0`,
  and the control `every_listed_event_is_a_hook_event_of_real_git`
  (`invocation/hooks_off_tests.rs:181`)
  asks Git about each one.
- The switches win against everything a caller can write:
  the options follow every global option
  (`invocation/global_arguments.rs:99-176`),
  and `SPEC.md:4523-4532` is right that a later `-c` wins over an earlier one.
  Measured on Git 2.56.0:
  a caller's `-c core.hooksPath=<hooks>` written before the subcommand runs no hook
  (`binary/binary_hooks_off_tests.rs:118`),
  an alias that carries its own `-c` runs no hook
  (`binary/binary_hooks_off_escape_tests.rs:74`),
  and an inherited `GIT_CONFIG_PARAMETERS` or
  `GIT_CONFIG_COUNT`/`GIT_CONFIG_KEY_n` pair for `core.hooksPath`
  loses to the wrapper's later `-c`:

```text
$ GIT_CONFIG_PARAMETERS="'core.hookspath'='<repo>/.git/hooks'" \
    /usr/bin/git -c core.hooksPath=/dev/null/cli-git-no-hooks checkout --detach HEAD
(no marker)
$ GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.hookspath GIT_CONFIG_VALUE_0='<repo>/.git/hooks' \
    /usr/bin/git -c core.hooksPath=/dev/null/cli-git-no-hooks checkout --detach HEAD
(no marker)
```

- An unknown global option cannot be used to place the switches early:
  `global_layout` reports it at
  `invocation/global_arguments.rs:167-171`,
  the switches then go first,
  and Git itself refuses the command,
  so no hook runs.
  Measured:
  `--super-prefix` is not a global option of Git 2.56.0
  ("unknown option:
  --super-prefix=x",
  exit 129),
  and every global option of its usage line is in
  `VALUE_OPTIONS` or `INLINE_OPTIONS`
  (`invocation/global_arguments.rs:48-86`).
- `core.fsmonitor` is left alone and reaches no hook:
  in Git 2.56.0 `fsmonitor-settings.c:104-146`
  reads `core.fsmonitor` as a boolean (builtin monitor) or as a path,
  and never through `find_hook`,
  so neither the hooks path nor an event switch reaches it,
  which is what `SPEC.md:4540-4547` says and
  `binary/binary_hooks_off_escape_tests.rs:102` controls.
- The commit path repeats the hooks path itself:
  `commit/commit_preparation.rs:139-146` puts it after the caller's global options and after
  the shadow's own `--git-dir`,
  and `shadow/shadow_git.rs:88-93` puts the shadow's own
  `core.hooksPath` before it,
  so the shadow's config can only lose.
  The source guard that keeps a hook entry writer out of the commit path exists and
  names the sources it covers
  (`source_guard/hook_self_invocation_tests.rs:24-42`,
  with the planted-token control at `:113-121`).
- The wrapper's dry-run probes run no server hook:
  measured on Git 2.56.0,
  `git push --dry-run --porcelain` to a local bare remote runs
  `pre-receive` zero times,
  so the three probes
  (`push/manual_push_probe.rs:75-100`,
  `SPEC.md:4601-4605`)
  are side-effect free on the remote,
  and their own `core.hooksPath` and `hook.pre-push.enabled=false`
  are redundant with,
  and earlier than,
  the ones every start carries.
- A push that is up to date runs no server hook either,
  so a retry of a push the wrapper stopped cannot re-run a `post-receive` hook.
- The push's output goes to anonymous files
  (`push/bounded_push.rs:426-429`),
  so a hook that keeps an inherited stream open,
  or leaves a child holding it,
  holds nothing up;
  the file is read back after the wait (`:441`),
  and the size cap (`:102`) is what a flood is judged by.
- The four starts that do not go through `git_command` run no hook:
  `invocation/management_service_repository_selection.rs:114` (`rev-parse`),
  `repo_read/git_system_config.rs:75` (`git var`),
  and the LFS program's own starts
  (`lfs/lfs_upload.rs:242`,
  `:327`),
  which is `git-lfs` and not Git.
- `doctor` handles the two unusual `core.hooksPath` values that a person really sets:
  `/dev/null` and a directory that does not exist are both "no hooks"
  (`health/doctor.rs:157`),
  a relative value resolves where Git runs hooks
  (`:274-320`,
  measured:
  `git config --type=path --get core.hooksPath` prints `.myhooks` unchanged and Git runs the
  hook from a subdirectory all the same),
  a link counts whatever it points at,
  a dangling one included,
  and a plain file without an execute bit does not
  (`:154-181`).
- The Git LFS shims are recognised byte for byte and never run through the wrapper,
  and a regenerated `pre-push` shim does not upload twice
  (`binary/binary_hooks_off_tests.rs:249`).
- The control that a failing hook file blocks nothing is in the crate
  (`binary/binary_hooks_off_tests.rs:152`),
  and a remote hook that declines a push is reported as Git words it,
  which `push/push_retry.rs:122-124` keeps as an allowed refusal.

## What to inspect and how to respond

Read in this order:

1.  Finding 1,
    and answer whether a local remote's hook may be ended with the push.
    If it may,
    `SPEC.md:2532`,
    `:2995` and `push/child_watch.rs:36-39` have to say so,
    because they are the sentences the watch's design rests on.
    If it may not,
    the push's stall rule needs the exception.
2.  Finding 2,
    and answer whether a nested `doctor` may disagree with a nested commit warning.
    The smallest fix is one filter,
    and the two functions that need it are named there.
3.  Finding 3 and finding 4 are the small ones;
    each is a suggestion,
    and each says which existing branch or loop to extend.

Nothing in this page is a decision.
No production code was changed.
