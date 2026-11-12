# A fast-forward merge that removes what a diverged merge refuses

## What was wrong

The wrapper handed every merge that would fast-forward to native Git whole
(`src/native/merge/merge_isolation.rs`,
the early `Bail::Forward` for `fast_forwardable && !merge_commit_wanted`),
ahead of every read it makes of the person's work.
Native Git's fast-forward writes exactly the paths the incoming commits change,
and at those paths it

- overwrote an ignored untracked file of the person's with no copy and no notice;
- refused an unsaved or staged edit with
  `error: Your local changes to the following files would be overwritten by merge`;
- removed a file standing at an ancestor of a path it adds,
  because it needs a directory there;
- removed a named pipe at a path it writes or deletes,
  with no copy.

The diverged shape refused the named pipe before writing,
so one worktree with one set of unsaved work was treated two ways
depending on whether the other side happened to be strictly ahead,
which a person cannot see beforehand.

`git pull` had the same gap:
`index_writer_forward.rs` skipped the isolated merge
whenever the fetched head only fast-forwarded.

## What the wrapper does now

A fast-forward is native Git's whole and stays as cheap as it was,
handed over at once.
Before it is handed over,
the paths the incoming commits change are asked one question
(`src/native/merge/merge_isolation.rs`,
`merge_isolation_a_special_file_the_merge_changes.rs`):
does a named pipe,
a socket or a device stand at one of them.
Such an entry holds no bytes,
and Git's own update removes it without a copy,
so the merge refuses it with the line and exit 1 a merge that creates a commit already used,
and nothing in the worktree,
the index or the branch changes.

`--ff-only` is read exactly (`FastForwardChoice::Only`).
A merge under it that can fast-forward is handed over after that question,
and one that cannot is native Git's own refusal,
`fatal: Not possible to fast-forward, aborting.` with exit 128,
so the option keeps the promise it states.
`merge.ff=only` asks the same question on its fast-forward.

A `git pull` stated to merge now always sends its merge half through the isolated merge
(`pull_isolation.rs`),
so a fast-forward pull is asked the same question before native Git's own fast-forward removes an entry.
A fetched head that cannot fast-forward under `--ff-only` keeps native Git's own refusal.

## What is not guarded, and why

The wrapper keeps no copy of anything here.
The project's rule,
as the owner restated it on 2026-11-12 about 10:33
(`doc/handover/cli-git-rust-implementation.md`),
is that a command which could destroy a person's data is refused and names what it would destroy,
and that the guard's `--no-enforce-*` option runs it as real Git would;
it is not that the wrapper saves a copy and lets the command run.
So an ignored untracked file at a path native Git's fast-forward writes is still overwritten by native Git,
exactly as `git merge` does without the wrapper,
and an unsaved edit there is still refused by native Git with its own
`error: Your local changes ... would be overwritten by merge`.
The carries of the diverged path are being listed and turned into refusals by another agent,
case by case;
this record covers the fast-forward path alone.

The two refusals the audit found in the diverged path stay refusals,
in both shapes:
a directory standing where the merge puts a file
(refused before the decision,
so a fast-forward refuses it too),
and a named pipe,
a socket or a device at a path the merge writes or deletes.
Real Git deletes an ignored directory and exits 0 there,
which measurement showed,
and `README.md` no longer says native Git refuses such a checkout.

## Measurements

The plain fast-forward through the wrapper costs what it did,
measured on a two-file fixture with the two binaries taking turns in one loop,
twenty samples each
(`/var/home/user/temp/agent/ff-carry/latency.ts`,
load average 52):

- before this change (0.6.1-alpha.12):
  median 40.3 ms,
  min 22.4,
  max 673.2;
- after it:
  median 36.8 ms,
  min 21.4,
  max 176.5.

The question the carry used to ask is off the fast-forward path.
While it was there it cost about 21 ms on a worktree of this repository's size,
one scoped `git status --ignored` over the written paths (21.6 ms)
plus the index marks (11.5 ms),
against 78.6 ms for a whole-tree status over 23,677 index entries and 88,595 ignored paths
(`/var/home/user/temp/agent/ff-carry/gate-cost.ts`).
The guard that remains costs one `symlink_metadata` per written path,
with no Git start of its own.

## The audit's shapes after this change

- A named pipe at a path a fast-forward writes or deletes:
  refused,
  exit 1,
  nothing changed
  (`f07_merge_refuses_where_git_succeeds.ts`,
  `a_fast_forward_refuses_a_named_pipe_at_a_path_it_adds`).
- An ignored directory where the merge puts a file:
  refused,
  exit 1,
  named
  (`a_fast_forward_refuses_a_directory_where_it_puts_a_file`).
- An ignored untracked file at an added path,
  an unsaved edit,
  a staged edit:
  native Git's own fast-forward,
  which overwrites the ignored file and refuses the edits with its own words.
  The wrapper takes no copy and prints nothing of its own
  (`a_fast_forward_with_an_ignored_file_at_an_added_path_is_still_native_gits`,
  `a_fast_forward_over_an_unsaved_edit_is_still_native_gits`).

## Evidence

`/var/home/user/temp/agent/ff-carry` holds the audit's reproduction copies
(`reproductions/`,
`run-repros.sh`),
the "before" binary (`binaries/before-alpha-12`),
the measurement scripts (`latency.ts`,
`gate-cost.ts`)
and this run's logs.
