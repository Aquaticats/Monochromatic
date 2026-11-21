# Git 2.56.1 removes a `worktrees` registration with no `gitdir`

What this record answers:
why a folder a test wrote into a fixture's `.git/worktrees` was gone before the test removed it,
which code removed it and under which order,
how to force that removal,
and what a fixture or a person writes instead.
The controls that pin it are in
`package/git-policy/cli/src/native/shadow/shadow_pool_member_tests.rs`.
The shipped facts are in `package/git-policy/cli/SPEC.md`
under the entries a commit reads.

## Symptom

A whole container gate failed this control at its own removal
(issue 6325 is the report,
and this message is the second of the two gates it names):

```text
thread 'shadow::shadow_pool_member::tests::an_entry_no_commit_reads_that_the_repository_lost_is_no_refusal' (139634) panicked at src/native/shadow/shadow_pool_member_tests.rs:362:55:
gone: Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

The control writes four files and a `worktrees/another` folder into the fixture's `.git`.
It makes a shadow through the creation and then removes each entry.
The four files were still there.
The `worktrees` folder was gone.
Line 362 is the `remove_dir_all` of that folder
and not one of the four `remove_file` calls before it.
The control passed alone and in 222 `shadow::` tests of the same image.

## Root cause

Each claim about Git's code is read from the commit that tag `v2.56.1` names,
`03b41523c576f26ad34085378717a0c1b287470f` of `git/git`,
and every link in Citations opens that commit.

### 1. Every commit Git makes starts the repository's automatic upkeep

`prepare_auto_maintenance` in `run-command.c` (lines 1956-1985)
builds the command every commit runs after it wrote its data:

```c
	maint->git_cmd = 1;
	maint->odb_to_close = r->objects;
	strvec_pushl(&maint->args, "maintenance", "run", "--auto", NULL);
	strvec_push(&maint->args, quiet ? "--quiet" : "--no-quiet");
	strvec_push(&maint->args, auto_detach ? "--detach" : "--no-detach");

	return 1;
```

It starts no child at all under `if (!enabled) return 0;` (lines 1961-1967),
where `enabled` is `maintenance.auto` and,
when that key is unset,
`gc.auto > 0`.
`auto_detach` is `maintenance.autoDetach`,
then `gc.autoDetach`,
then true,
so the default is `--detach`.
The commit waits only for the process that detaches.
The detached process goes on in the background.

`daemonize` is what makes the detached run keep the maintenance lock
(`setup.c`,
lines 2211-2238):

```c
		default:
			/*
			 * We're in the parent process, so we drop ownership of
			 * all tempfiles to prevent us from removing them upon
			 * exit.
			 */
			reassign_tempfile_ownership(parent_pid, child_pid);
			exit(0);
```

The maintenance lock is one of those tempfiles,
so it stays held in the child's name after `git commit` returned.

A run that cannot take that lock exits 0 at once and says nothing under `--auto`
(`maintenance_run_tasks` in `builtin/gc.c`,
lines 1401-1414):

```c
	if (repo_hold_lock_file_for_update(r, &lk, lock_path, LOCK_NO_DEREF) < 0) {
		/*
		 * Another maintenance command is running.
		 *
		 * If --auto was provided, then it is likely due to a
		 * recursive process stack. Do not report an error in
		 * that case.
		 */
```

So a forced run under a held lock is silent and does nothing,
and a control that only calls it proves nothing.

### 2. A manual run selects the strategy that holds `worktree-prune`

`initialize_task_config` in `builtin/gc.c` picks the task list (lines 1578-1584):

```c
	if (opts->schedule) {
		strategy = none_strategy;
		type = MAINTENANCE_TYPE_SCHEDULED;
	} else {
		strategy = geometric_strategy;
		type = MAINTENANCE_TYPE_MANUAL;
	}
```

A scheduled run gets no task by default.
Any other run gets the geometric strategy (lines 1502-1529),
which lists `worktree-prune` as a manual task.
The task table (lines 1331-1335) wires that label to its function and its auto condition:

```c
	[TASK_WORKTREE_PRUNE] = {
		.name = "worktree-prune",
		.background = maintenance_task_worktree_prune,
		.auto_condition = worktree_prune_condition,
	},
```

### 3. That task's condition is met by a registration Git cannot read

`worktree_prune_condition` (lines 350-386) opens `$GIT_DIR/worktrees`,
reads `maintenance.worktree-prune.auto` (one by default),
and asks `should_prune_worktree` about one entry.
It answers true when that entry is prunable.

`should_prune_worktree` in `worktree.c` (lines 937-1030)
resolves the entry first (`strbuf_realpath`,
line 957),
then answers true for an entry with no `gitdir` file (lines 970-974):

```c
	if (stat(gitdir.buf, &st)) {
		strbuf_addstr(reason, _("gitdir file does not exist"));
		rc = 1;
		goto done;
	}
```

The expiry date does not matter for it.
A path that is not a directory at all takes the same path (lines 961-965).
A `locked` file in the registration makes Git keep it (lines 966-969).
A `gitdir` that names an existing path is kept.
One that names a missing path is pruned
unless the registration holds an `index` file newer than the expiry (lines 1013-1021).

### 4. `git worktree prune` then removes the empty folder

The task runs `git worktree prune --expire <gc.worktreePruneExpire>`
(`maintenance_task_worktree_prune`,
lines 338-348):

```c
	prune_worktrees_cmd.git_cmd = 1;
	strvec_pushl(&prune_worktrees_cmd.args, "worktree", "prune", "--expire", NULL);
	strvec_push(&prune_worktrees_cmd.args, cfg->prune_worktrees_expire);

	return run_command(&prune_worktrees_cmd);
```

The expiry defaults to `3.months.ago`.
`prune_worktrees` in `builtin/worktree.c` (lines 212-246)
removes each prunable registration:

```c
	while ((d = readdir_skip_dot_and_dotdot(dir)) != NULL) {
		char *path;
		strbuf_reset(&reason);
		if (should_prune_worktree(the_repository, d->d_name, &reason, &path, expire))
			prune_worktree(d->d_name, reason.buf);
```

It then calls `delete_worktrees_dir_if_empty` (line 244),
which is one `rmdir` of `$GIT_DIR/worktrees` whose failure is ignored (lines 164-169):

```c
static void delete_worktrees_dir_if_empty(void)
{
	char *path = repo_git_path(the_repository, "worktrees");
	rmdir(path); /* ignore failed removal */
	free(path);
}
```

### 5. What that meant for the control

The fixture's own `git commit` in `repository_of` started step 1's detached run
before the control wrote its entries.
Run alone,
that run is expected to finish before the write;
under a loaded gate it is inferred to have landed after the write and before the removal.
Neither was observed here:
on a mostly idle host the integration pass kept a hand-written folder through 40 rounds of 150 ms,
while a folder written before a plain `git commit` was gone 2 ms after the commit returned,
and the forced foreground run took it every time the lock was free (40 of 40 rounds in that pass).
That is the window the control had.

The wrapper's own code removes nothing here.
A shadow shares no `worktrees` entry (`shadow_links.rs`,
`SHARED_ENTRY_NAMES`).
A shadow's own `config` sets `gc.auto = 0` and `maintenance.auto = false`
(`shadow_config.rs`).
So no commit in a shadow starts this upkeep at all.

The fixture now states those two keys in its own config (`repository_of`),
so the commits it makes start no child,
and each control runs that upkeep itself in the foreground and asserts what it did.

## Verification

Version under test:
Git 2.56.1.
This host's `git --version` reports it,
and so does the gate image's.
The source read is commit `03b4152` (tag `v2.56.1`).

### The order, forced

One command in any repository that holds such a folder:

```sh
mkdir --parents .git/worktrees/another
git maintenance run --auto --no-detach
test ! -e .git/worktrees
```

`--no-detach` keeps the same tasks on the caller's path,
so the outcome is the detached run's without waiting for the background process.
A run that cannot take `objects/maintenance.lock` does nothing and exits zero (step 1),
so this harness shows the task only while no other upkeep holds the lock.

### The catalog, entry by entry

Each entry was run against Git 2.56.1 on this host,
with a plain `worktrees` folder the upkeep had to take beside it as the positive control:
that folder went in every case.

- A directory with no `gitdir` file:
  pruned,
  and the empty `worktrees` folder behind it goes too (run).
- A `gitdir` file naming an existing path:
  kept (run).
- A `gitdir` file naming a missing path with no `index` file:
  pruned (run).
- A `gitdir` file naming a missing path with an `index` file newer than the expiry:
  kept (run).
- A `gitdir` file naming a missing path with an `index` file older than the expiry:
  pruned (read,
  `worktree.c:1013-1021`).
- A `locked` file:
  kept,
  with or without a `gitdir` file (run).
- A plain file at that name:
  pruned (run).
- A link at that name:
  Git resolves it first (line 957),
  so the folder it names is what it judges and takes,
  and the link itself stays behind pointing at nothing (run).
  A link to a registration another entry also names is pruned as a duplicate,
  by `prune_dups` (line 240) (run).

### The controls that pin it now

`repository_of` sets `maintenance.auto = false` and `gc.auto = 0` in the fixture's config
before its first commit,
so the fixture starts no detached upkeep and nothing holds the lock the controls would compete for.

- `an_upkeep_of_the_repository_takes_a_folder_that_holds_no_gitdir`:
  the folder goes,
  and the empty folder behind it goes too.
  That is the forced run's own effect,
  so a run that did nothing fails here instead of passing.
- `the_state_the_member_control_writes_is_state_the_upkeep_keeps`:
  a real linked worktree's registration stays and the four files stay,
  and a folder named beside them with no `gitdir` file goes,
  which is the evidence that the forced run took a task at all.
- `an_entry_no_commit_reads_that_the_repository_lost_is_no_refusal`:
  the member control itself,
  which now registers a real linked worktree
  and removes the folder itself,
  so the loss happens when the control says.

The second control fails on the folder the control wrote before this fix.
That is the red this record closes.

The failing control with the upkeep forced between its write and its removal
panicked at its own `remove_dir_all` every time
(`gone: Os { code: 2, kind: NotFound }`,
in 0.03 s with no host load).
It passes now that the fixture registers a real linked worktree.

With `objects/maintenance.lock` held by hand,
which is the case the earlier shape of these controls met by luck:
the forced run exits 0 with nothing on either stream and the folder stays,
so the first control's assertion fires and the second control's positive-evidence assertion fires;
with the lock removed the same run takes the folder and leaves the registration.
A held lock is a loud failure now,
never a silent pass.

## Verified workarounds

1. Register a real linked worktree when a control needs another worktree's state.
   The command is `git worktree add --quiet --detach <path>`.
   Tradeoffs:
   one Git start and a checkout inside the fixture.
   The registration then names a checkout the control owns.
2. Write the folder by hand with a `gitdir` file naming an existing path.
   A `locked` file works as well.
   Tradeoffs:
   invented state no Git command made.
   Its target must exist for as long as the control needs the entry.
3. Turn the fixture's own upkeep off,
   then run it yourself and assert its effect.
   The keys are `maintenance.auto = false` and `gc.auto = 0` in the fixture's config,
   and the run is `git maintenance run --auto --no-detach`.
   Tradeoffs:
   the fixture no longer exercises the automatic launch `git commit` performs,
   and the forced run proves nothing unless the control asserts what it took,
   because a run that skips its work exits zero and prints nothing.
4. Remove the entry tolerating `NotFound` when a control only needs it absent at the end.
   Tradeoff:
   the control can no longer tell its own removal from another process's.
   It stops pinning who took the entry.

## What does not work

- Blaming the wrapper's own cleanup.
  Nothing in the wrapper removes `worktrees` outside the `git worktree remove` and
  `git worktree prune` handling,
  and this control starts neither.
  The wrapper says the same in `worktree_remove.rs`:
  "Git removes the `worktrees` directory when the last registration goes".
  The wrapper's landing does start the same `maintenance run --auto` in the real repository
  (`src/native/landing/service_work.rs`,
  `maintenance_awaiting` at line 704,
  through `src/native/landing/maintenance_run.rs`),
  as native `git commit` would,
  and a run there takes the geometric strategy,
  `worktree-prune` among its tasks.
  No control of the shadow module reaches it:
  a shadow shares no `worktrees` entry,
  a shadow's config starts no upkeep,
  and the invocation capture runs only `rev-parse` and `symbolic-ref`.
  So the cause above stands.
  The wrapper's own removals of `worktrees` registrations sit in the worktree-removal code
  (`worktree_remove.rs`,
  `worktree_removal_finishes_an_interrupted_destroy.rs`
  and `worktree_removal_around_begun.rs`),
  none of them on a commit or creation path.
- Blaming the timing log's rotation.
  The file the control wrote is five bytes.
  The rotation moves a file aside at one mebibyte.
- Blaming a fixture-path collision between test processes.
  `nextest` runs one test in one process.
  The failing run reported `1 failed; 8014 filtered out`.
  The four files the control wrote survived.
  So nothing removed the fixture tree.
- Waiting for the flake.
  The order can be forced,
  which the commands above show.

## Upstream filing decision

Nothing to add to the Git tracker.

`.out-of-scope/` has no Git entry (checked 2026-11-20).
A duplicate search on the same day,
`gh search issues --repo git/git "maintenance worktree-prune"`,
`gh search prs --repo git/git "worktree prune maintenance"` and
`gh search issues --repo git/git "worktrees directory removed automatically"`,
returned nothing in either open or closed state.

1.  Is it really upstream's fault?
    No.
    This is documented and intended behavior.
    `git-maintenance(1)` documents `--auto` as running tasks whose thresholds are met.
    `git-config(1)` documents `gc.worktreePruneExpire`.
    `maintenance.worktree-prune.auto` changes the task's own condition.
    The report would be about our fixture writing a folder Git is written to remove.
2.  Can upstream fix it?
    Not applicable after constraint 1:
    there is no defect to fix.
3.  Are they supporting this use case?
    Yes,
    as far as it is a request:
    `git maintenance run --auto` is the documented way to run the same tasks by hand.
4.  Would the repo welcome our contribution?
    Not evaluated,
    because there is nothing to contribute.
    `git/git` carries no `CONTRIBUTING.md`;
    its contribution rules are `Documentation/SubmittingPatches`.
5.  Will they likely fix it?
    No.
    The behavior is what its documentation states,
    and the task's own condition key exists to change it.
6.  Have we prototyped a minimal fix?
    Not applicable:
    constraint 1 failed,
    so there is no upstream patch to write.

The record stays for the next session with no draft to post.

## What this record did not run

- Only Git 2.56.1 was read.
  The `worktree-prune` task and the `maintenance.worktree-prune.auto` key are newer
  than the `git maintenance` of older releases.
  An older Git's automatic maintenance is not covered here.
- The natural occurrence of the original flake was not reproduced here.
  The order was shown by holding the lock by hand and by a commit's own detached upkeep.
- The catalog was run on this host's Git.
  The gate image's Git reports the same version,
  but the gate did not run the catalog.

## Citations

All at commit `03b41523c576f26ad34085378717a0c1b287470f`,
which tag `v2.56.1` names.

- `prepare_auto_maintenance`:
  `https://github.com/git/git/blob/03b41523c576f26ad34085378717a0c1b287470f/run-command.c#L1956-L1985`
- `daemonize` handing the tempfiles to the child:
  `https://github.com/git/git/blob/03b41523c576f26ad34085378717a0c1b287470f/setup.c#L2211-L2238`
- the maintenance lock,
  and a run that skips under `--auto`:
  `https://github.com/git/git/blob/03b41523c576f26ad34085378717a0c1b287470f/builtin/gc.c#L1401-L1414`
- the `worktree-prune` entry of the task table:
  `https://github.com/git/git/blob/03b41523c576f26ad34085378717a0c1b287470f/builtin/gc.c#L1331-L1335`
- the task and its condition:
  `https://github.com/git/git/blob/03b41523c576f26ad34085378717a0c1b287470f/builtin/gc.c#L338-L386`
- the strategy a manual run selects:
  `https://github.com/git/git/blob/03b41523c576f26ad34085378717a0c1b287470f/builtin/gc.c#L1502-L1529`
- that selection:
  `https://github.com/git/git/blob/03b41523c576f26ad34085378717a0c1b287470f/builtin/gc.c#L1578-L1584`
- `should_prune_worktree`:
  `https://github.com/git/git/blob/03b41523c576f26ad34085378717a0c1b287470f/worktree.c#L937-L1030`
- `delete_worktrees_dir_if_empty`:
  `https://github.com/git/git/blob/03b41523c576f26ad34085378717a0c1b287470f/builtin/worktree.c#L164-L169`
- `prune_worktrees`:
  `https://github.com/git/git/blob/03b41523c576f26ad34085378717a0c1b287470f/builtin/worktree.c#L212-L246`
