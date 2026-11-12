# A fast-forward merge that carried nothing while a diverged merge carried everything

## What was wrong

The wrapper carried a person's unsaved work over a merge that creates a commit,
and handed every fast-forward to native Git whole
(`src/native/merge/merge_isolation.rs`, the early `Bail::Forward` for
`fast_forwardable && !merge_commit_wanted`).
Native Git's fast-forward writes exactly the paths the incoming commits change,
and at those paths it

- overwrote an ignored untracked file of the person's with no copy and no notice;
- refused an unsaved or staged edit with
  `error: Your local changes to the following files would be overwritten by merge` and exit 2;
- removed a file standing at an ancestor of a path it adds, because it needs a directory there;
- removed a named pipe at a path it adds, with no copy.

The diverged shape carried the first two, and kept the third's bytes nowhere either.
A person could not tell beforehand which shape their merge would take,
so the same work in the same worktree was kept or lost
depending on whether the other side happened to be strictly ahead.

`git pull` was the same: `pull_isolation.rs` skipped the isolated merge
whenever the fetched head only fast-forwarded.

## What the wrapper does now

`src/native/merge/merge_isolation_a_fast_forward_in_the_way.rs` asks one question
about the paths the incoming commits write and their ancestors:

- `git --no-optional-locks status --porcelain=v1 -z --untracked-files=all --ignored=traditional`
  with one `:(top,literal)` pathspec per written path, in bounded argument batches;
- the edits an `assume-unchanged` or `skip-worktree` entry hides
  (`merge_isolation_marks_the_status_hides.rs`);
- a `symlink_metadata` of each written path and of each directory on the way to it,
  which also names a directory, a pipe, a socket or a device that `git status` does not list.

An answered "nothing stands there" keeps native Git's own fast-forward, handed over at once.
Anything else is carried by the same steps a merge commit takes
(`merge_isolation_carry_a_fast_forward.rs`),
except that the branch moves to the incoming commit itself:
no merge tree, no message, no editor, no commit.

## Measurements

Host: this machine under its usual load (`/usr/bin/git` 2.56.0, load average around 90),
worktree of this repository (23,646 index entries, 599,599 ignored files under the tree).
Fifteen interleaved samples of each arm, medians in milliseconds;
a bare `git rev-parse HEAD` is the control for how much of a difference the host makes
between samples (median 21.9, min 17.3, max 61.4).

- whole-tree `git status --porcelain=v1 -z --untracked-files=all`: median 102.9
- scoped `git status ... --ignored=traditional -- <20 paths>`: median 47.8
- scoped `git status ... -- <20 paths>` without the ignored answers: median 39.6
- `git diff-files --quiet -- <20 paths>`: median 33.7
- `git diff-index --quiet HEAD -- <20 paths>`: median 30.1
- `git ls-files --others --exclude-standard -z -- <20 paths>`: median 30.5
- `git ls-files --others --ignored --exclude-standard -z -- <20 paths>`: median 27.9
- `git diff-tree -r --name-only HEAD HEAD`: median 34.4

So the question is one start, about 24 ms above a bare start on a worktree of this size,
and the alternative of four precise queries costs 122 ms.
The whole plain fast-forward through the wrapper gains that one start:
an isolated attempt that forwards keeps the prologue it had
(`rev-parse`, `merge-base`, `diff-tree`, `git config`)
and replaces the staged-work `diff-index` it no longer needs with the question.
On a small fixture repository the difference is inside the run-to-run spread of the same binary.

## What Git prints, measured

`git merge --ff-only --no-edit -m <text> <commit>`, run under
`GIT_REFLOG_ACTION=merge <branch>`, prints and records exactly what
`git merge --no-edit -m <text> <branch>` prints and records, for a fast-forward:

- default, `--stat`, `--ff-only`, `--log`:
  `Updating <a>..<b>`, `Fast-forward`, then the diffstat with its `create mode`,
  `delete mode` and `mode change` lines;
- `--no-stat`, `-n`: the two lines alone;
- `-q`: nothing;
- with `-m <text>`: `Fast-forward (no commit created; -m option ignored)`,
  and the same suffix in the reflog entry.

The child is given the message and no output-hiding option,
because no Git run the wrapper starts carries `--quiet`
(`README.md`, "No `--quiet` on the wrapper's own Git runs").
The person's `-q`, `--stat` and `--no-stat` hide what they hide for Git
by cutting what is printed (`merge_isolation_the_fast_forward_child.rs`).

## What the copies a merge keeps beside a path cannot hold, and what is still refused

A file standing at an ancestor of a path the merge writes is kept whole beside it:
a copy of it is written before Git's update,
and the entry stays where it is until that update removes it,
so a merge that stops before the update leaves the checkout as it was;
a copy whose bytes the entry still holds is removed by the next command as a duplicate
(`merge_isolation_a_file_where_the_merge_puts_a_directory.rs`).

Two shapes are still refused, and both are recorded as refusals that can end a merge nonzero:

- a directory standing where the merge puts a file, holding anything the merge does not delete itself;
- a named pipe, a socket or a device at a path the merge writes or deletes, or at an ancestor of one.

The copies a merge keeps beside a path move one entry and verify it by its bytes:
the object its bytes make before the move, and the same object again before the private name goes.
A directory is a tree rather than bytes,
so a copy of it could neither be checked against what was judged nor ended by any command,
and Git's own update would delete it with nothing to show the person where its files went.
Making the copies hold a directory or a special entry needs that proof to come from somewhere else:
the identity of the whole entry, a record that names every path below it,
and a decision that ends it.
Until then the refusals stay, and they are what protects those entries.

## What the window costs

A merge killed between the copy of an ancestor entry and Git's own update
leaves the original entry in place and the copy beside it.
The next command's kept-copy sweep finds the entry still holding the copy's bytes
and removes the copy as a duplicate,
and the merge's own recovery then meets the ancestor path when it installs the merge's files:
a file standing where a directory must go stops that install with the unfinished record named,
and the person's entry is still in their checkout.
Nothing is lost, and the merge needs somebody to move that entry away before it can finish.

## How this was measured and run

The probes are in `/var/home/user/temp/agent/ff-carry`:

- `gate-cost.ts` (the arms above, interleaved with the control);
- `git-output.ts` and `git-child-output.ts` (native Git's words against the child's, five option shapes);
- `ffchoice.ts` (`--ff`, `--ff-only` and `--no-ff` tie-break in the order typed, measured against Git 2.56.0);
- `status-shapes.ts` (`git status` and `git ls-files` over an ignored file, an ignored directory,
  an ignored symbolic link, an ignored named pipe, an untracked file, a modified and a staged one,
  a deleted one, and an ignored file at the ancestor of an added path:
  status lists no pipe at all, and lists the file below an ignored directory as `!! build/out.bin`).
