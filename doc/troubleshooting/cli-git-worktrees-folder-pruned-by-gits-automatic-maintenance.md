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

A whole container gate on 2026-11-19 failed this control at its own removal (issue 6325):

```text
thread 'shadow::shadow_pool_member::tests::an_entry_no_commit_reads_that_the_repository_lost_is_no_refusal' (139634) panicked at src/native/shadow/shadow_pool_member_tests.rs:362:55:
gone: Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

The control writes four files and a `worktrees/another` folder into the fixture's `.git`.
It makes a shadow through the creation and then removes each entry.
The four files were still there.
The `worktrees` folder was gone.
Line 362 is the `remove_dir_all` of that folder
and not one of the four `remove_file` calls above it.
The control passed alone and in 222 `shadow::` tests of the same image.

## Root cause

Five steps.
Each is read from the source at the tag `v2.56.1` of `git/git`.

### 1. Every commit Git makes starts the repository's automatic maintenance

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

`auto_detach` is `maintenance.autoDetach`.
Its fallback is `gc.autoDetach`.
Its fallback is true.
So the default is `--detach`.
The commit waits only for the process that detaches.
The detached process goes on in the background.

### 2. A manual run selects the strategy that holds `worktree-prune`

`initialize_task_config` in `builtin/gc.c` (lines 1579-1584) picks the task list:

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
Any other run gets the geometric strategy (lines 1502-1530).
That strategy lists `worktree-prune` as a manual task.
The task table at lines 1331-1335 wires that label to its function and its auto condition.

### 3. That task's condition is met by a registration Git cannot read

`worktree_prune_condition` (lines 350-384) opens `$GIT_DIR/worktrees`.
It asks `should_prune_worktree` about one entry.
The limit is one by default.
It answers true when that entry is prunable.

`should_prune_worktree` in `worktree.c` (from line 937)
answers true for an entry with no `gitdir` file (lines 970-973).
The expiry date does not matter for it:

```c
	if (stat(gitdir.buf, &st)) {
		strbuf_addstr(reason, _("gitdir file does not exist"));
		rc = 1;
		goto done;
	}
```

A folder that is not a directory at all takes the same path (lines 960-963).
A `locked` file in the registration makes Git keep it (lines 965-968).
A `gitdir` that names an existing path is kept.
One that names a missing path is pruned
unless the registration holds an `index` file newer than the expiry.

### 4. `git worktree prune` then removes the empty folder

The task runs `git worktree prune --expire <gc.worktreePruneExpire>`
(`maintenance_task_worktree_prune` in `builtin/gc.c` at lines 338-346).
The expiry defaults to `3.months.ago`.
`prune_worktrees` in `builtin/worktree.c` (lines 212-246)
removes each prunable registration.
It then calls `delete_worktrees_dir_if_empty` (line 244).
That is one `rmdir` of `$GIT_DIR/worktrees` whose failure is ignored (lines 164-169).

### 5. What that meant for the control

The fixture's own `git commit` in `repository_of` started step 1's detached process
before the control wrote its entries.
Run alone,
that process finished before the write.
Under a loaded gate it landed after the write and before the removal.
That is exactly the window the control has.

The wrapper's own code removes nothing here.
A shadow links no `worktrees` entry (`shadow_links.rs` with `SHARED_ENTRY_NAMES`).
A shadow's own `config` sets `gc.auto = 0` and `maintenance.auto = false`.
So no commit in a shadow starts this upkeep at all (`SPEC.md` has the shadow's config).

## Verification

Version under test:
Git 2.56.1.
This host's `/usr/bin/git` reports it.
So does the gate image's `git --version`.
The source read is the tag `v2.56.1` of `git/git`.

The order can be forced with one command in any repository that holds such a folder:

```sh
mkdir --parents .git/worktrees/another
git maintenance run --auto --no-detach
test ! -e .git/worktrees
```

`--no-detach` keeps the same tasks on the caller's path.
The outcome is the detached run's outcome without waiting for the background process.

The crate's controls run the same order and assert both outcomes:

- `an_upkeep_of_the_repository_takes_a_folder_that_holds_no_gitdir`:
  a folder with no `gitdir` goes and the empty folder behind it goes too.
- `the_state_the_member_control_writes_is_state_the_upkeep_keeps`:
  a real linked worktree's registration stays and the four files stay.

The second control fails on the folder the control wrote before this fix.
That is the red this record closes.

The failing control with the upkeep forced between its write and its removal
panicked at its own `remove_dir_all` every time.
The message was `gone: Os { code: 2, kind: NotFound }`.
It ran in 0.03 s with no host load.
It passes now that the fixture registers a real linked worktree.

What works and what fails,
by entry:

- A `worktrees/<id>` directory with no `gitdir` file:
  pruned and the folder goes with it.
- A `gitdir` file naming an existing path:
  kept.
- A `gitdir` file naming a missing path with no `index` file:
  pruned.
- A `gitdir` file naming a missing path with an `index` file newer than the expiry:
  kept.
- A `locked` file of any content:
  kept whatever `gitdir` says.
- A file or a link at that name instead of a folder:
  pruned.

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
3. Remove the entry tolerating `NotFound` when a control only needs it absent at the end.
   Tradeoff:
   the control can no longer tell its own removal from another process's.
   It stops pinning who took the entry.

## What does not work

- Blaming the wrapper's own cleanup.
  Nothing in the wrapper removes `worktrees` outside the `git worktree remove` and
  `git worktree prune` handling.
  This control starts neither.
  The wrapper says the same in `worktree_remove.rs`:
  "Git removes the `worktrees` directory when the last registration goes".
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

- Constraint 1 (is it upstream's fault):
  no.
  This is documented and intended behavior.
  `git-maintenance(1)` documents `--auto` as running tasks whose thresholds are met.
  `git-config(1)` documents `gc.worktreePruneExpire`.
  `maintenance.worktree-prune.auto` changes the task's own condition.
  The report would be about our fixture writing a folder Git is written to remove.
- Constraint 5 (would they fix it):
  no,
  for the same reason.
  The remaining constraints are not reached.
  A filing needs all six.

The record stays for the next session with no draft to post.

## What this round did not run

- The permalinks below were not loaded in `agent-browser`.
  The line numbers were read from the tagged files fetched for this record.
- Only Git 2.56.1 was read.
  The `worktree-prune` task and the `maintenance.worktree-prune.auto` key are newer
  than the `git maintenance` of older releases.
  An older Git's automatic maintenance is not covered here.

## Citations

All at the tag `v2.56.1` of `git/git`.

- `prepare_auto_maintenance`:
  `https://github.com/git/git/blob/v2.56.1/run-command.c#L1956-L1985`
- the `worktree-prune` entry of the task table:
  `https://github.com/git/git/blob/v2.56.1/builtin/gc.c#L1331-L1335`
- the task and its condition:
  `https://github.com/git/git/blob/v2.56.1/builtin/gc.c#L338-L384`
- the strategy a manual run selects:
  `https://github.com/git/git/blob/v2.56.1/builtin/gc.c#L1502-L1530`
- that selection:
  `https://github.com/git/git/blob/v2.56.1/builtin/gc.c#L1579-L1584`
- `should_prune_worktree`:
  `https://github.com/git/git/blob/v2.56.1/worktree.c#L937-L1000`
- `delete_worktrees_dir_if_empty`:
  `https://github.com/git/git/blob/v2.56.1/builtin/worktree.c#L164-L169`
- `prune_worktrees`:
  `https://github.com/git/git/blob/v2.56.1/builtin/worktree.c#L212-L246`
