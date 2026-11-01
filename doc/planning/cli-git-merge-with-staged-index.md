# Merge with another session's staged changes in the git wrapper

Status: implemented on 2026-10-29, on the branch `worktree-agent-afb65bb9babbe8ffa`, not yet merged.
The implementation deviates from the brief's recommended design on one point, which the user may veto:
the checkout moves with Git's own fast-forward update of the merge commit and not with the commit path's
compare-and-swap landing; see "Decision: Git's own two-way update, not a second landing".
The user's decision, verbatim: "git refusing the merge because other sessions having staged changes in main is a
problem we must own and make succeed with our wrapper."

## Problem

Several Claude sessions share one main checkout.
When one session has staged changes (the index differs from `HEAD`), a true (non-fast-forward) `git merge <branch>`
run by another session through the wrapper fails with native Git's refusal and the wrapper forwards it:

```text
error: Your local changes to the following files would be overwritten by merge:
  u.txt
Merge with strategy ort failed.
```

The merge touches none of the staged files in the common case, so the refusal protects nothing.
The wrapper classifies `merge` as an index writer (`src/native/index_writer_commands.rs`): it serializes the command
under the landing lock and forwards it, but it does nothing to isolate the index.

## Measurements: what native Git refuses

Measured on 2026-10-29 with native Git 2.56.0 (`/usr/bin/git`) in disposable repositories under
`~/temp/agent/merge-iso/`, harness `measure-native.ts` there.
Each repository has `main` and `feature` diverged (`feature` changes `f.txt` and `g.txt`, `main` changes `h.txt`),
or `feature` ahead of `main` for the fast-forward cases.
"Refused" means exit status 2 (or 1 for a fast-forward) with the message above and nothing changed.

True merge (case a):

- Staged modification of an unrelated file: refused.
- Staged addition of an unrelated file: refused.
- Staged deletion of an unrelated file: refused.
- All three together: refused, and the message names all three paths.
- Clean index (control): succeeds, `Merge made by the 'ort' strategy.`

Staged changes to files the merge touches (case b):

- Staged modification of `g.txt`, which the merge changes: refused with the same message.
  The refusal does not distinguish this from the unrelated case.

Unstaged worktree changes only (case c):

- Unstaged modification of an unrelated file: succeeds.
- Unstaged modification of `g.txt`, which the merge changes: refused, with a different message
  (`Please commit your changes or stash them before you merge. Aborting`).
- An untracked file that the merge does not touch: succeeds.

Fast-forward (case d):

- Staged modification of an unrelated file: succeeds, and the staged entry is kept.
- Staged modification of `g.txt`, which the fast-forward changes: refused (`Aborting`).
- Unstaged modification of an unrelated file: succeeds.
- `--no-ff` on a branch that could fast-forward, staged unrelated file: refused like a true merge.

Other shapes with a staged unrelated file (case e), all refused:

- `--squash`, `--no-commit`, `--no-ff`, `-m <message>`, `-X ours`: the same message as a true merge.
- Octopus (two branches): `Merge with strategy octopus failed.`, with the message after it.
- `-s ours`: `Merge with strategy ours failed.`
- Controls on a clean index: `--squash` and `--no-commit` succeed, stopping before the commit.

Conclusion: native Git refuses every merge that creates a commit when the index differs from `HEAD` at all,
whatever the staged paths are.
The check is "the index is not clean", not "a staged path is in the merge".
A fast-forward is different: it is a two-way tree update that keeps every entry the move does not write, and refuses
only on overlap.

The same refusal reproduces through the built wrapper (`target/debug/cli-git-native` linked as `git` ahead of
`/usr/bin` on `PATH`): the harness run with `WRAPPED_MERGE` set prints the identical message and exit status 2 for
cases a1 to a4, and the clean control succeeds.

## Decision: Git's own two-way update, not a second landing

The brief recommended computing the merge in isolation, creating the commit, and landing it with the commit path's
compare-and-swap and a computed post-index, with the worktree and index brought up to date for the merge's paths.
The measurements decide against building that, and the alternative below is implemented.

Evidence:

- Case d1 above shows native Git updating the worktree and index for exactly the paths a fast-forward writes,
  keeping every other staged entry, on the very index state that blocks a merge.
  Case d2 shows it refusing only on overlap.
  That is the operation the brief asks for ("bring the worktree and the real index up to date for exactly the paths
  the merge changed, leaving every other staged entry and every unstaged change as it was"), and Git already does it.
- A merge commit whose first parent is the current commit is exactly such a fast-forward target.
  So the merge commit is made from objects (`merge-tree --write-tree`, `commit-tree`) and the checkout moves with
  `git merge --ff-only <merge commit>`.
- The commit path's landing exists because its commit is prepared against a private index and shadow repository
  and must install a computed post-index.
  Reusing it for a merge needs a new journal operation kind, a post-index computed for merge-changed paths, and a
  worktree completion that handles deletions, mode changes, symbolic links and filters; the existing completion
  models blob replacement of added paths only.
  That is a second implementation of what `git merge --ff-only` does, with more ways to be inexact.
- Compare-and-swap is kept: Git's fast-forward moves the branch with `update_ref` against the old value it read, so a
  branch that moved makes it refuse.
  The window between Git's worktree and index update and its branch move is Git's own and is the same in native
  `git merge`; it is covered by a journal, below.

What is reused from the repository: `merge-tree --write-tree` and `commit-tree` as in
`src/native/auto_push_reconcile.rs` (`commit_merge`), signing with `-S` when `commit.gpgSign` is true, the
status-versus-incoming overlap check of `src/native/auto_push_dirty_paths.rs` (`status_paths`, `diff_paths`,
`overlapping_paths`, `listed`), the landing lock and lease of `src/native/index_writer_coordination.rs`, the owner
records and liveness of `src/native/transaction_owner.rs`, and the crash-marker phases of
`src/native/transaction_phase.rs`.
Differences from the auto-push merge that the brief's recommendation would have copied wrongly:
`--allow-unrelated-histories` is passed to `merge-tree` only when the user gave it (native Git refuses unrelated
histories), and `advance_local_branch` is not used because it refuses a detached `HEAD` and checks a pushed tip.

## Implementation

New modules in `src/native/`, each with a `_tests.rs` sidecar:

- `merge_options.rs`: the exact command lines that are completed; everything else forwards.
- `merge_isolation.rs`: the sequence, called from `run_coordinated_forward` in `index_writer_forward.rs` once the
  landing lock is held.
- `merge_isolation_message.rs`: `git fmt-merge-msg` input line per kind of name, and `git stripspace`.
- `merge_isolation_report.rs`: the three refusal findings.
- `merge_isolation_journal.rs`: the intent record `<git-dir>/cli-git-merges/<id>.json`.
- `merge_isolation_changes.rs` and `merge_isolation_recovery.rs`: recovery of a merge whose process died.

`SPEC.md` ("Merge beside staged changes" and "Recovery") has the step-by-step behavior, and `README.md` the user view.

Recovery is hooked where the commit path already recovers dead landings:
`recover_commit_transactions` (every invocation, one failed directory read when nothing is recorded) and
`recover_dead_landings` (under the landing lock), plus the start of every forwarded `merge`.
A record whose owner is alive never makes a command take the landing lock, because a nested wrapper run by the merge's
own `post-merge` hook would wait for its ancestor.

## Fallbacks to native Git (not guessed)

These forward unchanged, and native Git then refuses them with the staged work as before:
`--squash`, `--commit`, `--no-commit`, `-s`/`--strategy`, `-X`/`--strategy-option`, `--ff-only`, `--edit`/`-e`,
`--signoff`, `--cleanup`, `-F`, `--autostash`, `--verify-signatures`, `--abort`/`--continue`/`--quit`, any option
abbreviation, an octopus merge (several commit-ishes), a commit-ish written `<name>^`, `<name>~<n>`, `@{...}` or
`<rev>:<path>`, `merge.autostash`, `merge.verifySignatures`, `branch.<name>.mergeOptions`, and a merge that would open an
editor (a terminal on standard input and output without `--no-edit`; a notice names `--no-edit`).
Also forwarded: a clean index, a fast-forward, an up-to-date merge, unrelated histories without the flag,
`merge.ff=only`, an unborn `HEAD`, and any merge, rebase, cherry-pick or revert in progress.
A merge run inside another wrapper's landing lease (a hook or `rebase --exec`) is not coordinated at all, as before.

Behavior that differs from native Git on a clean checkout, by design:

- `pre-merge-commit`, `prepare-commit-msg` and `commit-msg` do not run.
  The merge commit is made from objects, and "the policies run on the merge as they do for the auto-push merge" means
  none run at commit time (SPEC "Auto-push": "The merge commit skips commit-time hooks and policies because it is made
  from objects").
  The push-time content scan still covers the merge when it is pushed.
  `post-merge` runs once.
- The branch reflog says `merge <branch>: Fast-forward`, not `merge <branch>: Merge made by the 'ort' strategy.`
- The linked-worktree ignored-state copy does not run around an isolated merge; the next forwarded command settles it.
- `merge-tree` reads the merge configuration itself; the wrapper reads only `merge.ff`, `merge.stat`, `commit.gpgSign`
  and the deferring keys.
  Measured for `merge.renormalize` (`~/temp/agent/merge-iso/renormalize.ts`): a CRLF file renormalized to LF on one
  side and edited on the other conflicts under both `merge-tree` and `git merge` by default, and merges to the identical
  tree under both with `merge.renormalize=true`.
  Other merge settings (rename detection, directory renames) were not compared and are unmeasured.

## Verification

Oracle (binary tests, the wrapper in a checkout with staged addition, modification and deletion against native Git
on a byte-for-byte copy without them): the merge commit has the same tree, both parents, author and committer names and
message, and the same standard output and standard error, for the default message, `--no-edit`, `-m`, several `-m`,
untidy whitespace in `-m`, `--log`, `--no-log`, `--quiet`, `--no-stat`, a tag, a remote-tracking branch and an
abbreviated commit name.
Other sessions' staged entries are identical in `ls-files --stage --debug` (stat data included); unstaged edits and
untracked files are identical byte for byte.
Refusals (staged, staged deletion, staged rename source, unstaged edit, conflict) leave the branch, every byte of the
index file and the worktree as they were, with no `MERGE_HEAD`, and `git fsck --strict` passes.
A paused merge serializes a concurrent commit, which then lands on top of it.

Crash controls (phase markers `merge-intent`, `merge-applied`, and states built by hand for the window inside Git's
own update): a kill before Git starts is discarded; a kill after the branch moved only loses its record; a kill after
Git wrote the worktree and index, or only some files, is completed by the next command with the other session's staged
entry byte-identical; a path holding someone else's content stops recovery with the record kept; an emptied or halved
record fails closed naming it, and a deleted record recovers nothing.

Guards shown failing (the code was broken, the tests run, FAIL seen, the code restored): the overlap check removed
(`a_merge_over_a_staged_path_is_refused_and_changes_nothing` and `other_kinds_of_overlap_are_refused_the_same_way`
fail); the completion of a started merge removed (the three completion unit controls and
`a_kill_inside_gits_update_is_completed_by_the_next_command` fail); the signing setting ignored
(`a_signing_repository_gets_a_signed_merge` fails); the message cleanup removed (the oracle fails on the untidy case);
`--squash` accepted (`shapes_that_cannot_be_made_exact_fall_back_to_native_git` and the option unit control fail);
the clean-index forward removed (`a_clean_index_merge_is_forwarded_unchanged` fails); the liveness check removed
(`a_live_owners_record_is_left_alone` fails).

Process counts, measured with `GIT_TRACE2_EVENT` start events on the wrapper's children
(`~/temp/agent/merge-iso/count-processes.ts`, debug build, version cache warm):

- A merge into a clean index starts one process more than before: `git diff-index --cached --quiet HEAD --`.
  Observed: `rev-parse` twice, `diff-index`, `merge`, then the merge's own `stash create` and `maintenance run --auto`.
  That one process is the least that can tell a clean index; the index cannot be read without Git.
- A fast-forward with staged unrelated work forwards to native Git after four more queries
  (`diff-index`, `config`, `rev-parse`, `merge-base`).
- A shape that falls back by its options (`--squash`) starts no extra process.
- The case that now succeeds starts 15 processes (queries, `merge-tree`, `fmt-merge-msg`, `stripspace`, `commit-tree`,
  `symbolic-ref`, `merge --ff-only`, and the stat); before, it failed after 11.
- Commands other than `merge` start the same number as before (`status`: 2).

## `git pull`

`git pull` hits the same refusal through its merge step, measured through the wrapper:
`error: Your local changes to the following files would be overwritten by merge: u.txt` with exit 2, after the fetch.
The same mechanism does not apply cleanly.
Git runs the merge as its own child (`git merge` from its exec path), so the wrapper never sees it; handling it means
re-implementing `pull`'s argument grammar, its fetch, its `--rebase` and `pull.ff` settings, and the `FETCH_HEAD`
description lines in the merge message, which is a second mechanism.
It is not done.
The way around it is `git fetch` followed by `git merge --no-edit origin/<branch>`, which the wrapper completes.
`git merge FETCH_HEAD` is not completed: its message names the fetched URL, so it forwards to native Git.
