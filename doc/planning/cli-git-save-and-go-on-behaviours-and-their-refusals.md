# The wrapper's save-and-go-on behaviours, and the refusals that replace them

## Why this exists

The owner restated on 2026-11-12 what "never discard user data" means
(`doc/handover/cli-git-rust-implementation.md`,
"The owner on what \"never discard user data\" means",
and the answers of 2026-11-12T10:35:07-05:00 below it):

> I do want to say "never discard user data" means to block potentially destructive commands with a
> --no-enforce-* opt-out, not that we save everything myself.

This page lists every place the wrapper,
for a command a person typed,
saved,
moved aside,
copied,
stashed or carried something of the person's so that a command could run
that would otherwise have destroyed it or that real Git would have refused.
The owner's answer of the same day
("Redirect everything") makes the shipped cases into refusals with an opt-out,
and names the cost:
commands that work today start being refused,
and it is a large change to make without looking at each case.
This is that look.

The branch `refactor/git-policy-cli-guards-refuse-with-an-opt-out-where-the-wrapper-saved-and-went-on`
converts one case
(the second one below)
and stops before the merge's carry,
which every agent of this repository merges through daily.
The plan for the merge is under its own section.

## How each fact below is labelled

- **run**: shown by running the wrapper on a disposable repository,
  with real Git's answer in the same state beside it.
  Every fixture had its own `HOME`,
  its own `XDG_RUNTIME_DIR` (mode `0700`)
  and a `bin/git` link to the wrapper,
  under `${HOME}/temp/agent/guards-inventory-2026-11-12/`;
  the runs of the installed release are in that folder's `out/*.md`.
  The installed release moved from `0.6.1-alpha.13` to `0.6.1-alpha.15` during the work;
  each quoted run names the release it used.
- **code**: read from the code named beside it.

## What is out of scope

The owner's brief excludes:

- what the wrapper does for itself and not for a typed command
  (its own removal of a landed worktree,
  its own records and scratch references);
- a copy whose only purpose is to make a write atomic
  (`direct_fix_install_writes.rs`,
  the `scratch` folder of a commit).
- the three behaviours an answer of the same day keeps as they are,
  listed at the end of this page.

---

## 1. `git clean` over a nested Git repository

**Status: listed, another agent's.**

The typed command and the state:
`git clean -ffdX` in a linked worktree that holds an ignored directory which is its own Git repository.
A second `-f` makes Git remove a nested repository whole,
with its own history and its uncommitted files,
and the outer worktree sees only one ignored directory,
so nothing in `git status` says what is about to go.

What the installed release does (**run**, `0.6.1-alpha.15`):

```text
$ git clean -ffdX                      # built wrapper
exit: 0
monochromatic-git-policy-cli: this clean would delete the Git repository nested and everything in it, so it was moved whole to
  <common>/monochromatic-git-policy-cli/discarded/2026-11-12T16-06-54/nested
  before the clean runs; the rest of the clean runs as typed. Look at what it holds with `git -C .../nested status`, and delete it yourself when you no longer need it.

$ git clean -ffdX                      # real git, same state
exit: 0
Removing nested/
```

The nested repository was gone from the worktree in both runs;
below the common Git directory the wrapper's copy held `nested/only-here.txt` and `nested/.git`.
Real Git deletes the only copy;
the wrapper moves it beside the audit log and lets the clean run.

What the refusal would be:
`clean` would be refused naming each nested repository it would delete,
with `--no-enforce-worktree`,
which runs the clean exactly as real Git runs it and keeps nothing
(the same opt-out the other `clean` forms already use,
`README.md`,
"Discarding commands").
The saving code is `policy_linked_worktree_nested_repositories.rs`,
`command_clean_nested_repositories.rs` and
`conflict/discarded_nested_repository.rs` (**code**).

What stops working:
nothing in this repository's scripts or `mise.toml` tasks runs `clean -ffdX`
(search over `*.toml`, `*.ts` and `*.jsonc` for `clean -f` patterns and for `--ours`/`--theirs` finds no use of either);
agents here clean with `mise run` tasks or by name.

Size of the change:
small.
One guard,
two call sites and their tests,
and the nested-repository half of the folder of saved copies.

Another agent holds this case:
`fix/git-policy-cli-clean-counts-no-force-as-git-does-and-saves-a-repository-made-while-it-saves`
(reframed at about 10:38 to refuse with the opt-out).

## 2. A `checkout` or `restore` that takes a side of a conflicted file

**Status: converted in this branch.**

The typed command and the state:
a merge (or a rebase of either backend,
a cherry-pick,
a revert) stopped with `a.rs` conflicted,
and the person edited `a.rs` by hand,
as a person resolving it does.
`git checkout --ours -- a.rs`,
`git checkout --theirs -- a.rs`,
`git checkout <rev> -- a.rs`,
`git checkout --merge -- a.rs`,
`git restore --ours|--theirs|--merge -- a.rs` and
`git restore --source=<rev> -- a.rs`
replace that file.

What the installed release does (**run**, `0.6.1-alpha.15`):

```text
$ git checkout --theirs -- a.rs        # built wrapper
exit: 0
monochromatic-git-policy-cli: saved your copy of a.rs to
  <common>/monochromatic-git-policy-cli/discarded/2026-11-12T16-30-18/a.rs
  before this command replaces it (a.rs was conflicted).

$ git checkout --theirs -- a.rs        # real git, same state
exit: 0
```

`a.rs` held the person's resolution before both runs and `topic\n` after them;
the wrapper's copy held the resolution,
real Git kept nothing.
The wrapper ran the command in the **main** worktree,
where the same policy rejects every other discarding form (**run**: the same line without a conflict,
and `git restore --staged --theirs -- a.rs`,
are rejected with `linked-worktree-only`).

The refusal that replaces it (**run**, branch build):

```text
monochromatic-git-policy-cli: git checkout with paths, --force, --patch, --ours or --theirs is rejected in the main git
worktree: it overwrites files with other content, so your edits to them are gone. The merge in progress left `a.rs` with an
unmerged index entry, and it holds the resolution you have written so far, which exists nowhere else: this command replaces
its content, and it keeps no copy of it. To settle the conflict and keep what you wrote, edit the file and commit the result
(git commit --message <message> -- <paths>); `git show :1:<path>`, `:2:<path>` and `:3:<path>` print the base and the two
sides, and git diff prints the file as it stands. If you have decided to replace it on purpose, save a copy yourself
(git diff -- <path> > backup.patch) and pass --no-enforce-worktree directly after the subcommand: the command then runs
exactly as real Git runs it, and no copy is kept.
```

It is one line in the finding's `message`;
the wrapping above is for reading.
It names each unmerged file the line would replace,
the operation that waits,
that the version on disk exists nowhere else,
and the opt-out.
With `--no-enforce-worktree` the command runs as real Git runs it
(**run**: exit `0`, `a.rs` holds what real Git left in an identical fixture,
no copy under the Git directory,
one audit line with `"outcome":"hatch"`).
A path is named only when the line's own token reads as one plain file below the working directory,
so a directory,
a wildcard,
pathspec magic,
`--patch`,
`--force`,
`--pathspec-from-file` and an index-only `restore --staged` keep the refusal and claim no path (**run**).
Nothing in the wrapper reads the copy this case used to make;
the folder of saved copies stays for the abort and nested-repository cases (**code**:
`save_copies`'s only callers are `abort_discard_guard.rs` and the nested-repository save).

What stops working,
and for whom:

- A person or an agent who resolves a conflict by taking a side now gets a refusal.
  They either
  `--no-enforce-worktree`,
  or edit the file and commit it.
  That is the owner's answer of 2026-11-12T10:35:07-05:00 applied to this case.
- Nothing in this repository's scripts,
  `mise.toml` tasks or agent documents (`AGENTS.md`, `.agents/`, `doc/agent/`) runs these forms
  (**run**: a search over `*.toml`, `*.ts`, `*.jsonc` and those documents for `--ours`/`--theirs` and
  `restore --source` finds no use).
  Three records measured the old behaviour and now describe one that is gone;
  they are listed under "Documents that still describe the old reading".
- The doctor and the audit still report the folder of saved copies,
  and still honour a `saved-conflict` line an earlier release wrote (**code**:
  `conflict/discard_audit_pins.rs` matches the word it always matched).

Size of the change:
medium.
About 1,500 lines of the wrapper and its tests go,
one message and one reading of the command line arrive,
and no other case builds on the copy.

## 3. The abort forms of `merge`, `rebase`, `cherry-pick`, `revert` and `am`

**Status: listed, another agent's.**

The typed command and the state:
a linked worktree stopped in a merge with `a.rs` conflicted,
and the person's own resolution of `a.rs` written by hand.
`git merge --abort` restores the state before the merge and keeps nothing.

What the installed release does (**run**, `0.6.1-alpha.15`):

```text
$ git merge --abort                    # built wrapper
exit: 0
monochromatic-git-policy-cli: this worktree holds changes besides the merge in progress, and git merge --abort discards them; the wrapper saved them first, so nothing you wrote is lost.
  copies are under <common>/monochromatic-git-policy-cli/discarded/2026-11-12T16-32-20
  a.rs => <common>/monochromatic-git-policy-cli/discarded/2026-11-12T16-32-20/a.rs
  git merge --abort runs now and restores the state before the merge.

$ git merge --abort                    # real git, same state
exit: 0
```

`a.rs` held the resolution before both runs and the branch's own content after them;
the wrapper's copy held the resolution,
real Git kept nothing.

In the **main** worktree the same command is rejected and nothing is saved
(**run**: `linked-worktree-only`,
"it restores the state before the merge and discards the edits made since").

What the refusal would be:
the abort form is refused,
the refusal names every path whose content the abort would discard,
and `--no-enforce-worktree` runs the abort exactly as real Git runs it and keeps nothing.
The saving code is `conflict/abort_discard_guard.rs` with its reading in
`repository_facts_abort_discard.rs` and `rule_abort_operation.rs` (**code**).

What stops working:
an agent or a person who aborts a merge after resolving part of it now has to save the resolution first,
or pass the opt-out and lose it.
Nothing in this repository's scripts or `mise.toml` tasks runs an abort form
(**run**: a search over `*.toml` and `*.ts` for
`(merge|rebase|cherry-pick|revert|am) --abort` finds none).

Size of the change:
medium.
The verdict, its facts and its messages are one guard on six command forms;
the copy machinery (`conflict_sides.rs`, `conflict_side_source.rs`) is shared with case 1,
so it stays until that one goes too.

Another agent holds this case:
`fix/git-policy-cli-abort-and-skip-are-refused-with-an-opt-out-where-they-would-discard-work`,
whose base is `7cd39662bf`
(the owner's answers of 2026-11-12T10:32:10-05:00 and 10:35:07-05:00).
This branch does not touch it.

## 4. A merge or a pull that carries a person's uncommitted edits

**Status: listed with a plan, not changed.**

### The typed command and the state

`git merge <branch>` (or `git pull`) where the worktree holds the person's own work at a path the merge writes:
an unsaved edit,
a staged edit,
an edit hidden by the `assume-unchanged` or `skip-worktree` mark,
or an untracked file at a path the merge adds.
Real Git refuses a merge that would overwrite local changes
and refuses one whose untracked file stands where the merge writes.

### What the installed release does

**run**, `0.6.1-alpha.15`, a diverged merge with an unsaved edit to `a.rs`:

```text
$ git merge --no-edit topic            # built wrapper
exit: 0
Merge made by the 'ort' strategy.
 a.rs | 2 +-
 1 file changed, 1 insertion(+), 1 deletion(-)
monochromatic-git-policy-cli: git merge topic: your changes to a.rs touch the lines the merge changes, so the file holds
both sides between conflict markers and the merge was made. Resolve the markers in that file, then run `git add -- a.rs`.

$ git merge --no-edit topic            # real git, same state
exit: 2
error: Your local changes to the following files would be overwritten by merge:
	a.rs
Please commit your changes or stash them before you merge.
Aborting
Merge with strategy ort failed.
```

After the wrapper `a.rs` held both sides between conflict markers;
after real Git it held the person's edit,
unmerged.
A run killed at a 300 s bound left the person's bytes on
`refs/cli-git/carried/<n>` and a copy beside the path (**run**),
which is the carry's own recovery.

Where the fast-forward path is concerned the wrapper forwards to Git,
and Git's own refusal is the answer (**run**:
`git merge` of a fast-forward with an unsaved edit to a written path exits `2` with Git's words under the wrapper too;
another branch,
`fix/git-policy-cli-fast-forward-merge-and-pull-carry-what-a-diverged-merge-carries`,
is making the fast-forward path refuse where the diverged path refuses).

### What the refusal would be

`git merge` and `git pull` would be refused while the worktree holds work at a path the merge writes,
naming every such path
(the same list the capture reads today),
with `--no-enforce-worktree` forwarding the command to real Git exactly as Git runs it.
Under the opt-out the command then gets **Git's own** answer:
a refusal naming the same paths for a tracked or staged change and for an untracked file in the way,
and,
for a path the index marks `assume-unchanged` or `skip-worktree`,
a refusal that **destroys the person's file** (**run**: real Git alone,
with `a.rs` marked `assume-unchanged` and edited,
printed "Your local changes to the following files would be overwritten by merge" and left `a.rs` holding the
index's content, `base\n`,
not the person's `my edit hidden by the mark\n`).

### Where the wrapper's behaviour would equal Git's, and where a guard is still needed

Equal after the change,
so no wrapper code is needed:

- an unsaved edit to a tracked path the merge writes:
  Git refuses and names the path (**run**);
- a staged edit to such a path:
  Git refuses the same way;
- an untracked file at a path the merge adds:
  Git refuses and names the file (**run**, via the pull probe below);
- an edit at a path the merge does **not** write:
  Git runs the merge and keeps the edit,
  which is the state the wrapper's carry also leaves.

A guard is still needed,
because real Git does not protect these:

- a path whose index entry carries `assume-unchanged` or `skip-worktree`:
  Git's own check skips it,
  the merge replaces the file under the mark,
  and it can do that while it also prints a refusal (**run** above).
  The wrapper compares the worktree copy with the index entry itself
  (`merge_isolation_marks_the_status_hides.rs`),
  so this path must be refused before Git starts.
- a mode the person set at a path the merge deletes,
  and a file the person wrote while the merge ran:
  the wrapper reads each path again just before the write that replaces it
  (`merge_isolation_take_asides.rs`) and names what it kept.
  Refusing instead of carrying keeps the naming,
  which is what the owner's rule asks for.

### What goes, what stays, and in what order

The carry is not one module but a family (**code**):

- `merge/merge_isolation_carry.rs` and its parts:
  `merge_isolation_carry_apply.rs`,
  `merge_isolation_carry_kept_path.rs`,
  `merge_isolation_carry_landed.rs`,
  `merge_isolation_carry_layers.rs`,
  `merge_isolation_carry_notice.rs`,
  `merge_isolation_carry_parts.rs`,
  `merge_isolation_carry_permissions.rs`,
  `merge_isolation_carry_place.rs`,
  `merge_isolation_carry_plans.rs`,
  `merge_isolation_carry_reads.rs`,
  `merge_isolation_carry_writes.rs`;
- the asides that make room for the merge's writes:
  `merge_isolation_take_asides.rs`,
  `merge_isolation_take_aside_names.rs`;
- the copies a merge keeps beside a path:
  `merge_isolation_kept_copies.rs`,
  `merge_isolation_kept_copies_decisions.rs`,
  `merge_isolation_kept_copies_words.rs`;
- the recovery of a killed carry:
  `merge_isolation_recovery_carry.rs`,
  and the carry's half of `merge_isolation_recovery_records.rs`;
- the pull's kept copies of untracked files in the way
  (`merge/pull_untracked_paths_in_the_way.rs`,
  `merge/pull_untracked_paths_in_the_way_kept_copies.rs`) if that case is converted with it;
- the second pass over a carried layer:
  `merge_second_pass_worktree.rs`'s carried-layer entry points
  (the second pass itself stays for a conflict the merge leaves).

Stays:

- `merge_isolation*.rs`'s isolation of a merge,
  its journal,
  its recovery,
  and the notice that a merge was saved for a later command;
- `merge_isolation_changes.rs` (the reading of which paths hold the person's work),
  which becomes the refusal's input;
- `merge_isolation_marks_the_status_hides.rs`,
  because a marked path is where Git keeps nothing;
- `merge_second_pass*.rs` and the word merge,
  for conflicts the merge leaves;
- `pull_isolation.rs`'s fetch-then-merge shape,
  which exists so a staged index,
  a held `index.lock` and a late ref update stop failing the command.

Order:

1. Add the refusal and the opt-out,
   naming the paths the capture read,
   with its controls red first:
   a merge over an unsaved edit,
   a merge over a staged edit,
   a merge over a marked path,
   and a merge that touches nothing of the person's (which must still run).
2. Stop writing the person's layers back:
   delete the carry writes and the `refs/cli-git/carried/<n>` reference,
   and leave the person's own file to Git's own refusal.
3. Delete the carry records and their recovery,
   then the kept copies and their decisions.
4. Delete the asides and their recovery,
   once the isolation no longer writes a path the person touched.
5. Delete the pull's kept copies if that case is converted.
6. Delete the tests with each module,
   and rewrite the binary controls:
   `binary/binary_merge_carries_edits_tests.rs`,
   `binary_merge_carries_edits_kill_tests.rs`,
   `binary_merge_carries_edits_moved_tests.rs`,
   and the `merge_isolation_carry_*_tests.rs`,
   `merge_isolation_kept_copies_tests.rs`,
   `merge_isolation_take_asides*` tests and
   `pull_untracked_paths_in_the_way_tests.rs`
   go;
   `binary_merge_isolation_tests.rs`,
   `binary_merge_never_fails_tests.rs`,
   `binary_merge_notice_tests.rs` and
   `merge/merge_isolation_recovery_tests.rs` stay,
   with their carry cases replaced by refusal cases.
7. Rewrite `README.md`'s "Carrying the person's own work" and `SPEC.md`'s merge sections,
   which state the owner's 2026-11-02 answer "Carry edits forward" as current.

### What breaks for whom

Every agent of this repository merges through the wrapper daily,
and a merge carries an uncommitted edit whenever one stands at a path the merge writes.
After the change,
an agent whose worktree holds an unsaved edit at a path the merge writes gets a refusal and must commit or stash it first,
which is what real Git has always asked of a person,
and what the owner's rule asks the wrapper to ask.
The agent-facing briefs
(`package/pi-shared/subagent-driver/doc/rules-every-pi-brief-carries.md` says "Never restore,
stash,
reset or revert anyone else's work")
do not teach the carry,
so no brief has to change.

Decisions of record that rest on the carry,
which a conversion has to amend through the owner rather than here
(`doc/handover/cli-git-rust-implementation.md`,
"Owner decisions in force"):

- "Carry edits forward (Recommended)" of 2026-11-02,
  which this page's plan contradicts;
- "Auto merging should never fail." and
  "Manual merge should never fail too.",
  which the carry was built to serve
  (`doc/decision/` holds the merge's records;
  this branch does not edit them).

## 5. `git pull` when an untracked file stands where the fetch adds a path

**Status: listed, not converted (it never lets a command run).**

The typed command and the state:
`git pull` whose merge would write a path an untracked file occupies.

What the installed release does (**run**, `0.6.1-alpha.15`):
in a plain fixture the pull is forwarded to Git,
and Git's own refusal names `new.txt`;
nothing is copied.
The kept copies belong to the pull the wrapper isolates
(`pull_isolation.rs`),
which the fixture did not reach (**code**:
`keep_the_untracked_paths_in_the_way` is called only on the isolated pull's own branch move,
`merge/pull_isolation.rs:595`).

This is not a save-and-go-on:
the pull still stops,
and the copy protects a recovery the person may run later
(`git reset --hard`,
`README.md`,
"Pull").
Converting it would remove a protection against Git's own recovery words,
which name a reset that deletes the untracked file.
It is listed here because it copies something of the person's;
it is not a candidate for a refusal.

## 6. A repository exempted by a tool-cache variable

**Status: listed, not converted (it is the opposite of a save); another agent's.**

The typed command and the state:
`git reset --hard` in a repository below a directory `OUT_DIR` names.

What the installed release does (**run**, `0.6.1-alpha.15`):

```text
$ git reset --hard                     # built wrapper, OUT_DIR set
exit: 0
HEAD is now at b7838f2 second
monochromatic-git-policy-cli: this repository is exempt from the worktree guard because OUT_DIR names <dir>, which contains
both its Git directory and its worktree. Nothing was measured and nothing was saved, so the command runs without that guard.
```

`a.rs` held the person's uncommitted edit before the command and `committed\n` after it;
no copy was made.
There is no save here to convert:
the guard does not run.
Turning the exemption into a refusal is the exempt-repository case,
held by
`fix/git-policy-cli-exempt-repository-saves-untracked-files-and-nested-repositories-before-it-discards`
(reframed at about 10:38 to refuse with the opt-out).

## 7. `git worktree remove` and `git worktree prune`: the snapshot and the held commit

**Status: listed, excluded by the brief (the wrapper's own records and scratch references).**

Two things are copied before Git runs the removal (**code**):

- the unshown push-failure notices and the unfinished push records,
  into `<common>/monochromatic-git-policy-cli/removal-snapshots/<instance>`
  (`worktree/worktree_removal_rescue.rs`),
  because the worktree's own Git directory goes with it;
- a commit a reference lock holds,
  onto `refs/cli-git/kept/<batch>`
  (`worktree/worktree_removal_held_commits.rs`),
  because the removal deletes the per-worktree reference that named it.

Both are the wrapper's own records and scratch references,
which the owner's brief puts outside this inventory.
Nothing of the person's files is copied:
a dirty worktree is Git's own refusal,
and the wrapper's removal refuses too
(`worktree_clean.rs`).
The removal of a landed worktree is the wrapper's own and stays asynchronous
(the owner's answer of 2026-11-12,
about 10:33).

## 8. `git worktree add`: the leave-out trash

**Status: listed, excluded: the source keeps every byte.**

A snapshot of the main checkout carries nested worktrees,
other checkouts,
caches and run records,
so the add moves them out of the **copy** into `.claude/cli-git-trash` inside the new worktree and hands that trash to
the commit service,
which empties it (**code**:
`worktree/worktree_trash.rs`,
`worktree_trash_dispose.rs`;
**run**, not reproduced: the source's entries stay where they are,
and the new worktree is the only thing that loses them).
No command of the person's is enabled by this,
and the source loses nothing,
so it is not a save-and-go-on.

## 9. The behaviours that stay as they are

Each is an answer of 2026-11-12,
and no branch of this task changes it:

- `git clean -fX` and `git clean -fx` of ignored files delete,
  exit `0` and leave one audit line;
  nothing is copied
  ("Keep allow and log", 2026-11-12T10:27:45-05:00).
  **run** (`0.6.1-alpha.15`):
  in a linked worktree,
  `git clean -fX` printed `Removing ignored.txt`,
  exited `0`,
  left no copy,
  and wrote one line with `"form":"CleanIgnoredOnly","outcome":"allowed","dirty":null`.
  In the main worktree the same command is still rejected.
- A batch whose scratch reference cannot be proven is committed and says so (2026-11-12T10:27:45-05:00).
- A commit against a commit service that does not answer fails with a clear message
  (2026-11-12T10:32:10-05:00).

## What the inventory did not find

No other place where the wrapper saves,
moves aside,
copies or carries something of a person's for a command the person typed.
The commands the brief names were each read:

- `checkout` and `restore`: case 2, and the refusal of an ignored file a forced form would replace
  (`ignored_file_message`), which refuses instead of saving;
- `switch`: only rejections,
  and it has no form that takes a side of a conflict;
- `reset`: rejections,
  and the exempt repository (case 6);
- `stash`: rejections,
  `git stash branch` moving an entry into its own branch,
  and Git's own stash, which is the person's command and not the wrapper's copy;
- `rebase`: the abort forms (case 3) only;
  a rebase carries nothing of its own and passes `--autostash` to Git as Git wrote it;
- `worktree remove` and `worktree prune`: case 7;
- `worktree add`: case 8,
  plus the ignored-state copy into the new worktree,
  which is a copy the new worktree receives and nothing of the source loses;
- `git monochromatic-git-policy-cli migrate-to-subvolume` and
  `convert-worktree-to-subvolume`:
  a carry of files newer in the original and a rollback,
  and both already refuse until `--discard-left-out` or `--discard-unreadable` is given.
  No command runs over a loss.

## Documents that still describe the old reading

- `README.md`,
  "Discarding commands": the abort paragraph still says the wrapper saves the person's own work before an abort,
  which is case 3 and is not this branch's;
  and the paragraph about the folder of saved copies describes a folder the abort and nested-repository cases still use.
- `SPEC.md`,
  "The abort forms while their operation waits": the same,
  with `saved-abort` records.
- `SPEC.md`,
  "Merges and commits never fail" and the merge sections around it,
  and `README.md`,
  "Carrying the person's own work": the merge's carry (case 4).
- `doc/planning/cli-git-shipped-limits-audit.md`,
  "Defaults for saved conflict copies":
  its only-copy finding is about the conflict-side save this branch removed
  ("`save_copies` copies the worktree file as it is when the person runs `git checkout --theirs` or `--ours`"),
  and the audit it measured will no longer grow that way.
  The question it leaves open is for the owner.
- `doc/planning/cli-git-discarding-commands-route-findings-on-main.md`,
  which records that "`git checkout --theirs -- a.rs` during a merge conflict saved the hand-edited file".
- `doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
  which records two `git checkout --ours` commands during one merge.
- `doc/handover/cli-git-logging.md`,
  whose "Moment" bullet about the prune pass uses "a `checkout --theirs` family command is not a commit"
  as its example of the command that saves.
- `doc/handover/cli-git-rust-implementation.md`:
  the entries that briefed the conflict-side save and its copies.
  The handover is a record;
  the rule it now carries is quoted at the top of this page.

## The converted case, as it was done

1. The controls first,
   through the built executable,
   failing on the unchanged tree:
   `conflict_side::a_conflict_side_is_refused_naming_the_file_and_the_opt_out` and
   `conflict_side::a_linked_worktree_refuses_the_conflict_side_and_saves_nothing`
   both failed with the notice `saved your copy of a.rs to ... before this command replaces it (a.rs was conflicted)`
   instead of a refusal.
   The third control,
   `conflict_side::the_opt_out_replaces_the_conflicted_file_as_real_git_does`,
   passed before and after,
   because the opt-out already skipped the policy and saved nothing.
2. The refusal,
   the reading of which of the line's paths are unmerged,
   and the message (`conflict_side_guard.rs`,
   `rule_linked_worktree_messages.rs`,
   `policy_linked_worktree.rs`).
3. The deleted saving code and its controls,
   each named in the commit:
   `conflict_side_notice.rs` (both notices),
   `conflict_side_decision.rs` (`decide_sides`, `SideDecision`, `magic_name`, `magic_on_conflicted`;
   `literal_path` moved into `conflict_side_guard.rs`),
   `magic_note` and `mode_note` in `conflict_side_modes.rs`,
   the copy calls and their failure paths in `conflict_side_guard.rs`,
   `AuditOutcome::SavedConflict`'s writer,
   and the controls of
   `binary/binary_conflict_side_tests.rs` and
   `binary/binary_conflict_side_hardening_tests.rs` that asserted a save,
   a copy failure,
   the folder's pruning,
   the doctor's report of it,
   and the mode and magic notes.
   `AuditOutcome::SavedConflict`'s word stays,
   because `discard_audit_pins.rs` reads the lines an earlier release wrote.
4. The documents: `SPEC.md`'s
   "Conflicted files during a merge, rebase, cherry-pick or revert" rewritten as a refusal,
   its copy mechanics moved to "The folder of saved copies",
   the rule written as "What never discarding a person's data means",
   and `README.md`'s discarding section rewritten for the same case.

**run**, the branch build, the same fixture as the installed run above:

```text
$ git checkout --theirs -- a.rs        # branch build
exit: 1
{"schemaVersion":1,"sequence":0,"type":"finding","trigger":"pre-forward","policyId":"linked-worktree-only",
 "severity":"error","code":"linked-worktree-only/linked-worktree-required",
 "message":"... The merge in progress left `a.rs` with an unmerged index entry ... no copy is kept ... pass --no-enforce-worktree
 directly after the subcommand ..."}
a.rs: unchanged
copies under the Git directory: none
audit: "form":"CheckoutPaths","outcome":"rejected"

$ git checkout --no-enforce-worktree --theirs -- a.rs
exit: 0
a.rs: "topic\n" (what real Git leaves in the same state)
audit: "form":"CheckoutPaths","outcome":"hatch"
```

## Limits of this inventory

- The isolated pull (case 5) was not reached by a fixture;
  its kept-copy path is read from the code.
- One merge run took more than 300 s and was killed,
  which is the shape of issue 6287 (a read of a process's descriptor record) and not of the merge;
  the run that completed took about five minutes in the same fixture and was not measured against a control.
- The installed wrapper moved from `0.6.1-alpha.13` to `0.6.1-alpha.15` while this page was written;
  every quoted run names its release.
