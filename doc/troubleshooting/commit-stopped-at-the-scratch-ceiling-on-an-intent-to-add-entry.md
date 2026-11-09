# git-policy-cli 0.6.1-alpha.5 stops a commit at its 3 s scratch ceiling when the index holds an intent-to-add entry

A commit whose pathspec named a file staged with `git add -N` never landed here:
the wrapper ran its scratch `git commit`,
that step hashed every file of the work tree
(233 MB of LFS content through the `git-lfs` clean filter),
and the wrapper stopped it at its 3 s ceiling on every retry.

## Symptom

Every `git commit` in the worktree failed with the same message,
and nothing was committed:

```text
[warn] [monochromatic-git-policy-cli] [runNativeCommit] the scratch git commit was stopped after 3006 ms at its ceiling of 3006 ms
monochromatic-git-policy-cli: the `git commit` this commit runs in its scratch repository was stopped after 3 seconds, the most a commit whose message came with the command may take (3 seconds, three times the slowest `git commit` step this repository's commit timing log holds); nothing was committed, and your index and files are as they were. The commit signs the commit because `commit.gpgsign` or `-S` asks for it, so the signing program is the first thing to check. Run the same `git commit` command again.
```

The message names signing,
which was not the cause:
`ssh-keygen -Y sign` took 0.01 s under the same load,
and the same repository committed normally from other worktrees at the same moment.

Raising the ceiling only moved the wall.
With `CLI_GIT_SCRATCH_COMMIT_CEILING_SECONDS=60` the wrapper printed its progress notices,
and they named the process that never ended:

```text
monochromatic-git-policy-cli: still waiting for the `git commit` this commit runs in its scratch repository, which signs the commit because `commit.gpgsign` or `-S` asks for it; `git-lfs` (process 900281) runs below it, 50 seconds so far; it is stopped if it lasts 60 seconds in all.
```

## Root cause

The wrapper builds a private index for its scratch commit and prefers a copy of the real index,
because a private index without the real index's cached stat data makes the scratch commit read and hash the whole work tree
(`package/git-policy/cli/src/native/commit/commit_index.rs:236-247`).

Whether the real index is copied is decided by one read of its flags
(`package/git-policy/cli/src/native/commit/commit_index_copying.rs:64`):

```rust
pub fn seeds_private_index(captured_index: &Path) -> bool {
    let Ok(bytes) = std::fs::read(captured_index) else {
        return false;
    };
    return read_index_flags(bytes.as_slice()) == IndexFlags::Plain;
}
```

`IndexFlags::Plain` requires that no entry carries assume-unchanged,
skip-worktree or intent-to-add
(`package/git-policy/cli/src/native/index/index_flags.rs:127-132`):

```rust
if be16(bytes, at.checked_add(head)?)? & (SKIP_WORKTREE | INTENT_TO_ADD) != 0 {
    marked = true;
}
```

An intent-to-add entry is what `git add -N` leaves behind (`INTENT_TO_ADD: u16 = 0x2000`,
same file,
line 48).
One such entry makes the read answer `Marked`,
the copy is skipped,
the private index is built from the base alone,
and the scratch commit then refreshes that index by hashing the work tree,
which runs the clean filter of every LFS-tracked file:
1,569 of them here,
measured at 0.16 s each for one 600 KB file (`git hash-object --path=… …`),
so minutes rather than the 3 s the step is allowed.

The commit record says which way the index was built.
`${repository}/.git/monochromatic-git-policy-cli-commit-timing.jsonl` held the failing commit as

```json
{"outcome":"stopped","totalMs":6748,"prepareMs":3676,"attempts":1,"indexBuilds":[{"kind":"first","statDataKept":false}]}
```

and the commit that landed a minute later,
from another worktree,
as

```json
{"outcome":"landed","indexBuilds":[{"kind":"first","statDataKept":true}]}
```

The earlier reading here,
that the repository move had left stale stat data,
was wrong:
`git status` in the affected worktree took 2.53 s and changed nothing,
and `git ls-files -v` showed every entry as `H`,
no assume-unchanged and no skip-worktree.

## Verification

- Version:
  `monochromatic-git-policy-cli` release `0.6.1-alpha.5`,
  recorded in the failing commit's timing line.
- Worktree:
  a linked worktree of this repository whose index held five entries staged with `git add -N`.
- Fails cleanly:
  a commit naming those paths,
  three times in a row,
  3,006 ms ceiling,
  `statDataKept:false`,
  nothing committed.
- Fails slowly:
  the same commit at a 60 s ceiling,
  stopped with `git-lfs` still running at 50 s.
- Works cleanly after `git add` of the same five paths (which clears the intent-to-add flag),
  the ceiling left at its shipped 3 s:
  the commit landed in 17.9 s of whole-step time with `statDataKept:true`.
- Control that the ceiling is not just too low for this host:
  `git status` in the same worktree took 2.53 s,
  and `ssh-keygen -Y sign` 0.01 s,
  while another worktree's commit of similar size landed inside the same ceiling at the same moment.

## Verified workarounds

- Stage the file's content instead of naming it with `git add -N`:
  `git add <path>` then `git commit -m <message> -- <paths>`.
  Nothing shifts semantically:
  the commit's pathspec still decides which paths land,
  and an intent-to-add entry has no content to commit anyway.
  Tradeoff:
  a `git add` of every new file means its content sits in the index before the commit,
  which the repository's own rule for new files asks for regardless.
- When the ceiling must rise for a genuinely slow step,
  `CLI_GIT_SCRATCH_COMMIT_CEILING_SECONDS` sets it for that invocation
  (`package/git-policy/cli/src/native/commit/native_commit_waiting_notice.rs:45`).
  Tradeoff:
  it removes the bound that catches a hung commit step,
  so it is worth naming in the report of whatever needed it.

## What does not work

- Retrying the commit:
  the index still holds the intent-to-add entry,
  so the same 3 s stop comes back every time.
- `git status` or `git update-index --refresh` first:
  they change nothing,
  because the stat data was never stale.
- Raising the ceiling alone:
  the step hashes the whole work tree,
  which takes minutes here,
  so no affordable ceiling covers it.
- Reading the message's signing hint at face value:
  signing was fast,
  and the process that hung was `git-lfs`.

## Upstream filing artifact

Nothing to file:
the tool is this repository's own,
and its own README already says the copy of the real index is skipped
when an index carries a mark.
