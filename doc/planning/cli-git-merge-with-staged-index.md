# Merge with another session's staged changes in the git wrapper

Status: in implementation on 2026-10-29.
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
