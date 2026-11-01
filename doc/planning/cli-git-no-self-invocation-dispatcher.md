# cli-git: no self-invocation, phase 2: the commit hook dispatcher

This is the dispatcher section of the self-invocation ban, kept in its own file so merges stay clean.
The pointer file `doc/planning/cli-git-no-self-invocation.md` belongs to the lfs and push phase.

The human owner's rule, quoted: "We should never invoke ourselves recursively. That's bad design. Audit, remove, and
ban that design aggressively."
A later correction, quoted: "What the hell, we don't have to use any shell!"
So no generated script or shell text of ours, and Git never starts the wrapper for a hook.

## What the dispatcher was

For every commit the wrapper wrote a `hooks` directory inside the transaction directory.
It held `plan.json` and, when the configuration could define a hook by name, three symbolic links named
`pre-commit`, `prepare-commit-msg` and `commit-msg`, each pointing at the wrapper's own executable.
The native `git commit` in the shadow repository ran with `-c core.hooksPath=<tx>/hooks` and with
`-c hook.<event>.enabled=false` for all four commit events.
Git found the link as the traditional hook of the event and started it, so Git started the wrapper.
`main.rs` read `argv[0]`, and `hook_dispatch.rs` recognized the entry by its name, took the hook lock, restored
`GIT_CONFIG_PARAMETERS`, exported the worktree root and the preparation lease, and ran
`git hook run --ignore-missing <event>` with a second `core.hooksPath`.
The pre-commit re-run after a replay used the same links.

The self-invocation was a design property with its own failure modes: a lock taken by a restarted process,
an environment rebuilt from a plan file, an executable path recorded at plan time, and a start-name branch in the
entry point that any symbolic link with the right name could trigger.

## Facts measured on this host

Git 2.56.0 (`/usr/bin/git`), run by hand in disposable directories under `${HOME}/temp/agent`.

- Message protocol, configured hooks, plain `git commit`.
  `prepare-commit-msg` gets `.git/COMMIT_EDITMSG message` for `-m` and `-F`, `.git/COMMIT_EDITMSG template` for `-t`,
  and `.git/COMMIT_EDITMSG commit HEAD` for `--amend --no-edit`.
  The message file differs by flavor: with `-t` or `-e` it already holds Git's status comment block
  (`# Please enter the commit message ...`, `# On branch ...`, `# Changes to be committed:`) when
  `prepare-commit-msg` reads it, and after the editor `commit-msg` reads the edited text with the same comment block.
  `--no-verify` skips `pre-commit` and `commit-msg` but still runs `prepare-commit-msg` and `post-commit`.
  A rewrite by `prepare-commit-msg` is what `commit-msg` reads, and a rewrite by `commit-msg` is what is committed.
  `GIT_EDITOR` is `:` when no editor runs and the editor command when one does.
- `hook.<event>.enabled=false` skips configured hooks only; a traditional hook in `core.hooksPath` still runs.
  This is why the old design needed both switches.
- `git hook run --ignore-missing pre-commit` honors the user's `hook.pre-commit.enabled=false` itself.
  The old dispatcher forced `hook.<event>.enabled=true` and so had to check the user's switch first.
- A native `git commit` in a shadow-shaped repository (own `config` with an `[include]` of the real config and
  `core.worktree`, `--git-dir` only) runs the configured hooks, does not run a hook file in the real hooks directory when
  `core.hooksPath` names a directory that cannot exist, and gives hooks `GIT_DIR=<shadow>`, `GIT_INDEX_FILE=<private>`.
- `GIT_WORK_TREE` for hooks.
  With `--work-tree=<abs>` Git rewrites it to `.` for hooks; a hook that changes into `sub` then sees `<repo>/sub` as
  the top level (`doc/troubleshooting/git-private-admin-dir-hook-environment.md`, Behavior B).
  Without `--work-tree`, with the shadow's own `core.worktree`, the variable is unset, the hook starts in the
  worktree root, and `git rev-parse --show-toplevel` from `sub` prints the root and `--show-prefix` prints `sub/`.

## Options

### Option i: the wrapper runs every hook itself with `git hook run`

The wrapper would run `git hook run --ignore-missing pre-commit` before the commit and the message hooks around it,
and finish `git commit` with Git's own hooks disabled.

- Pros.
  No hook entry, and the lock could wrap each run exactly as before.
- Cons.
  The message hooks cannot be replayed from outside Git.
  The wrapper would have to reproduce `prepare_to_commit`: the source argument (`message`, `template`,
  `commit HEAD`, and `merge` and `squash` that depend on repository state), the status comment block for `-t` and
  `-e`, the editor that runs between `prepare-commit-msg` and `commit-msg`, the cleanup mode, `--no-verify`
  skipping only two of the three, and the message-file path (`COMMIT_EDITMSG` of the shadow).
  Every flag combination of `git commit` becomes wrapper logic that must track Git's releases.
  Finishing the commit "with hooks disabled" then needs the final message, which exists only inside Git.
- Verdict.
  Possible for `pre-commit` and `post-commit`, which already run this way (`post_commit_hook.rs`, and the re-run in
  `commit_revalidation.rs`).
  Not possible for `prepare-commit-msg` and `commit-msg` without reimplementing `git commit`.

### Option ii: a non-wrapper entry that execs the real Git's `hook run`

A POSIX `sh` entry that `exec`s the real Git, with the lock and environment handled around the commit.
Rejected by the owner ("we don't have to use any shell"), and it would still be Git starting a file we generated.

### Option iii: Git runs the configured hooks natively; the wrapper owns the lock and the environment

Git already runs configured hooks (`hook.<name>.event` with `hook.<name>.command`) inside `git commit`, with its own
message protocol.
The wrapper stops silencing them, writes no entry, and keeps traditional hooks off with a hooks path that cannot exist.
It holds the hook lock around the one Git child that may run them, exports the lease to that child, and keeps
`post-commit` for the post-landing run it already does.

- Pros.
  Maximum fidelity: Git itself builds the message file, the arguments, the editor step and the cleanup, for every flag.
  Nothing starts the wrapper.
  No plan-file environment restoration, no recorded executable path, no start-name branch.
  Fewer processes: the three dispatcher processes and their three `git hook run` children are gone.
- Cons.
  The hook lock is held for the whole native commit, so an open message editor holds it.
  The old design took it per hook event, and the SPEC said "an open message editor never holds it".
  No scope between "per event" and "whole commit" exists without Git calling back into something of ours for each
  event, which is the thing banned.
  Hooks see the wrapper's own `-c` options in `GIT_CONFIG_PARAMETERS` (see "Environment a hook sees" below).
  A hook process orphaned by a `SIGKILL`ed wrapper no longer keeps the lock, because the wrapper was its owner, where
  the dispatcher process stayed alive as the hook's parent.
  A terminating signal is relayed to Git by the wrapper's signal guard, so only an uncatchable kill leaves an orphan,
  and recovery removes the shadow the orphaned commit writes into.
- Mitigations.
  The lock is taken only when the configuration can define a hook by name and an enabled preparation event exists, so
  a commit with no hook takes no lock and adds no system call.
  `hooks.concurrentCommits: true` skips it, and so does an inherited preparation lease.
  A waiting commit prints the usual slow-wait notice and gives up at `CLI_GIT_LOCK_WAIT_SECONDS` (30 minutes by
  default), so an editor left open does not block others forever.

### Option iv: pre-commit by the wrapper, message hooks by Git

Run `pre-commit` through `git hook run` under its own short lock and let Git run the message hooks.
Rejected: it splits one protocol across two mechanisms, still needs the lock around the editor for the message hooks,
and adds a hook runner start per commit for nothing.

## Choice

Option iii.
Best fidelity for the least code, and nothing in it needs Git to start the wrapper.
The one regression is the lock scope around an open editor; it is deliberate, bounded, documented and switchable.

## Design

- `commit_preparation.rs`: the native commit runs
  `<kept global options> --git-dir=<shadow> -c core.hooksPath=/dev/null/cli-git-no-hooks commit ...`.
  The hooks-off option is given again after the caller's options so a `-c core.hooksPath` of the caller cannot bring
  traditional hooks back into a commit (the old `-c core.hooksPath=<tx>/hooks` won over the caller's too).
  There is no `--work-tree`: the shadow's `config` already names the work tree.
  `post-commit` is switched off by `[hook "post-commit"] enabled = false` in the shadow's own `config`, after the
  include, because the wrapper runs it after landing.
  A file value reaches Git without touching `GIT_CONFIG_PARAMETERS`; a `-c hook.post-commit.enabled=false` would have
  been inherited by every process a hook starts (measured in the first version of this change: a hook running
  `/usr/bin/git commit` would have had its own `post-commit` silenced).
  Known limit: a caller's own `-c hook.post-commit.enabled=true` beats the file value and runs `post-commit` in the
  shadow as well.
  The child gets `CLI_GIT_PREPARATION_LEASE`, and the hook lock is held around it.
- `commit_revalidation.rs`: the `pre-commit` re-run after a replay is `git --git-dir=<shadow> hook run
  --ignore-missing pre-commit` under the same lock and lease, skipped when no `pre-commit` hook can run.
- `post_commit_hook.rs`: unchanged apart from sharing the lock and the disabled-event check with `hook_run.rs`.
- `hook_run.rs`: the shared decisions (can an event run a hook, is the lock needed), the lock, the lease variable.
- `hook_plan_record.rs`: writes `plan.json` into `<tx>/hooks` and nothing else.
  The file stays because `has_valid_preparation_lease` validates an inherited lease against it and the tearing table
  pins its recovery; its fields no longer include the executable path, the config parameters or the worktree root.
- Removed: `hook_dispatch.rs`, `hook_dispatch_entries.rs` (the entry writers, `hook_entry_name`, the Windows naming),
  the start-name branch of `main.rs`, the config-parameter quoting and restoration, `PreparationHooks`, and the
  recognition of `<tx>/hooks` as a wrapper hooks path in the traditional-hooks warning.

## Environment a hook sees

Measured with a configured hook for each of the four events that records its arguments and environment, run through the
wrapper binary of the base commit and through the binary of this change (host paths shortened).

- `cwd`: the worktree root, before and after.
- `GIT_DIR`: the shadow, before and after.
- `GIT_INDEX_FILE`: the transaction's `commit.index`, before and after.
- `GIT_EDITOR`: `:` without an editor, before and after.
- `CLI_GIT_PREPARATION_LEASE`: set, before and after.
- `GIT_WORK_TREE`: the absolute worktree root before, unset after.
  A hook that changes directory now finds the right top level through `GIT_DIR` and the shadow's `core.worktree`
  (`from-sub: top=<repo> prefix=[sub/]` before and after), and no longer inherits a relative `.`.
- `GIT_CONFIG_PARAMETERS`: before, the caller's `-c` values plus the inner run's `hook.<event>.enabled=true` and a
  `core.hooksPath=<tx>/no-hooks`; after, the hooks-off option, the caller's `-c` values and the hooks-off option again,
  nothing else.
  The hooks-off values reach hooks of every other wrapped Git command the same way (SPEC, "Real Git starts with
  traditional hooks off"), and a nested `/usr/bin/git` in a hook keeps traditional hooks off.
- `post-commit`: unchanged, the real worktree with the real index and the landed commit's author.

## Counts

`binary_budget_tests.rs`, strace over the whole process tree, Git 2.56.0.

- Commit with no hook: unchanged (every pinned count passes unchanged).
- Commit with one configured hook (`ONE_CONFIG_HOOK`): `hook run` starts 4 to 1 (only the post-landing
  `post-commit`), wrapper-process sync calls 23 to 21 (two lock acquisitions instead of four), `sh` starts 1,
  every other Git start unchanged.
  The wrapper no longer starts itself: three process starts less per configured-hook commit.
- `binary_hook_shim_tests.rs` trace: all four configured hooks start 1 hook runner, where they started 4.

## Guard

`hook_self_invocation_tests.rs` (the source scan; it holds no magic count, only the forbidden tokens it exists for) scans the sources of the commit path for an entry writer (`symlink(`, `hard_link(`,
`current_exe`, `write_hook_dispatch`), a hooks path built from a transaction directory, and the former vocabulary, and
scans `main.rs` for a start-name read.
`binary_hook_tests.rs` proves the behavior: the transaction's hooks directory holds `plan.json` only while a hook
runs, and a wrapper link named like a former entry, inside or outside a transaction, behaves as `git`.
The guard was shown failing on the real sources, then restored (the guard was committed first, and `git diff` of the
two planted files was empty after the restore).
A start-name read planted in `main.rs` (`let start_name = std::env::args_os().next();`) made
`hook_self_invocation_tests::main_does_not_dispatch_on_its_start_name` fail with `main.rs contains args_os().next()`.
An entry writer planted in `hook_plan_record.rs` (`std::os::unix::fs::symlink(...)` of `pre-commit` after the plan
write) made `hook_self_invocation_tests::the_commit_path_writes_no_entry_and_names_no_hooks_path_of_its_own` fail with
`hook_plan_record.rs contains symlink(`, and `hook_plan_record::tests::the_hooks_directory_holds_only_the_plan` fail
as well.
With both removed, all of them pass again.

## Left on an allow-list

Nothing from this phase.
Everything in the commit path works with Git running the user's hook commands directly.
The remaining self-invocation, `git cli-git lfs-pre-push` started by Git as a push hook, belongs to the lfs phase.

## Not covered here

- The planted-control table `bin/native-planted-controls-entry.ts` was updated for the new guards, and every `from` text
  was checked to occur once in its source, but the table runs only in the container gate; the coordinator runs it.
- Files over the 300-line limit that this change touched and did not create, all over it before the change:
  `commit_preparation.rs` (414), `commit_conclusion.rs` (345) and test sidecars beside the commit path.
  `hook_dispatch_plan.rs` shrank from 604 to 281 and gave its event-switch reads to `hook_event_switches.rs`.
  New files are all under 300 lines.
- Windows: the crate does not build for Windows, so the removed `<event>.exe` naming had no gate; nothing replaces it.
