# One Git child of a commit takes seconds while its neighbours take milliseconds, and the record's stretch figure hides it

Date:
 2026-11-12.
Subject:
 a wrapped `git commit` whose build phase took 13 999 ms and whose every named stretch stated 0 to 2 ms,
 because the time sat inside one Git child that the stretch subtracts,
 so a reader of `buildParts` alone saw a 14 s build explained by nothing.
Owner's rule this bears on:
 a commit through the wrapper is no more than five seconds slower than the same commit made by real Git
 (`package/git-policy/cli/SPEC.md`, the two places that say "five seconds").

## Symptom

One record of `monochromatic-git-policy-cli-commit-timing.jsonl.10` in this repository's `.git`
states, for a commit of `refs/heads/integrate/git-policy-cli-pass-17-for-alpha-14` at
`2026-11-12T14:13:13.131Z`:

- `totalMs` 14 579, `prepareMs` 176, `buildMs` 13 999, `submitMs` 356, `gateMs` 44;
- every entry of `prepareSpans` and of `buildParts` at most 33 ms, `shadow refs` at 2 ms;
- one entry of `steps`: `git for-each-ref`, `offsetMs` 179, `durationMs` 13 909;
- `quietMs` for that child at 853.499 ms, so its longest stretch with no bytes, no processor time and no
  task waiting for the disk was under a second.

The landed commit is `edf2e1a7500b4602b88db860da89d9024f573357`,
one parent (`739dadc734`),
one path (`package/git-policy/cli/src/native/worktree/worktree_removal_second_look.rs`, +45 lines, -25).
It concluded no merge,
and the merge it followed, `739dadc734` with two parents and seventeen paths against its first parent,
landed 45 seconds earlier with `buildMs` 52 and the same child at 5 ms
(`2026-11-12T14:12:38.078Z`, `totalMs` 962).

## Root cause

The time is not in the wrapper's own work: it is one real Git process that took 13.9 s while the
processes before and after it in the same commit took 2 to 72 ms.

The child is the shadow repository's ref snapshot, run once per wrapped commit and never by real Git:

- `package/git-policy/cli/src/native/shadow/shadow_refs.rs:107` `list_real_refs` runs
  `git for-each-ref --format=%(refname)%00%(objectname)%00%(symref)`;
- it is called from `write_shadow_refs` (`shadow_refs.rs:417`),
  which `carry_the_commit_parts` calls after marking the stretch
  (`shadow_create.rs:607`, the mark at `shadow_create.rs:628`);
- the build reaches it through `build_with_git` (`landing/service_native.rs:206`) and
  `take_or_create_shadow`.

The stretch stated 2 ms because both clocks subtract the children that began inside them:

```rust
let children: Duration = step_time_between(self.began, ended);
note_build_part(self.name, whole.saturating_sub(children), children);
```

(`src/native/commit/commit_build_part_durations.rs`, the `close` method; the same subtraction is in
`commit_prepare_part_durations.rs`).
Until 2026-11-12 the subtraction was all a part kept, so a stretch a child dominated read as an empty
stretch. Each part now also states `childrenMs`, the time of the children that began inside it
(`package/git-policy/cli/SPEC.md`, the field list of the record), and the record of this commit would
state `shadow refs` as `{"ms":2,"calls":1,"childrenMs":13909}`.

Earlier reading that was wrong:
"none of its named spans is over 33 ms, so the record does not say where the time went".
The record does say it, in `steps`, and `steps` covers the build phase:
the step list of every commit record holds exactly one `git for-each-ref`, and in 1488 of 1973
records it begins within 50 ms of the end of preparation, which is where the shadow's ref snapshot runs.

## What the measurements say

The stall is one process, not the host at that instant, and it recurs:

- 1973 of the 2015 commit records hold that child exactly once; its median is 8 ms, its ninetieth
  percentile 43 ms, and eight records are above one second (13909, 5492, 3815, 3045, 1877, 1733,
  1579, 1520 ms).
  Two of those commits had nothing else over 200 ms at all: the 13 909 ms one above, and
  `2026-11-11T23:37:11.979Z` (`totalMs` 2945, `buildMs` 1623, `git for-each-ref` 1579 ms).
- the same shape recurs in a disposable clone made on 2026-11-12 (`git clone --no-hardlinks`, a `HOME`
  and an `XDG_RUNTIME_DIR` of its own, a 4G-or-more scope per arm, the commit service started by hand
  on a socket of that runtime directory): one arm of a one-path commit took 12 005 ms with
  `prepareMs` 11 626 and one child `git read-tree` at 11 262 ms whose quiet gap was 617 ms,
  while its neighbours in the same commit took 4 to 31 ms.
- real Git stalls too: in 22 alternating pairs of the same commit shape real Git took 48 to 116 ms in
  most arms, 713 ms in one and 1043 ms in another, against a median of 98 to 136 ms.

So the wrapper adds several Git children to a commit that real Git does not start, and any one of them
can be the one the host stalls; the wrapper's own work is not where the seconds are.

## Verification

Version under test:
 the installed `monochromatic-git-policy-cli 0.6.1-alpha.13`,
 shadowing `/usr/bin/git` (Git 2.56.0),
 and the same source built with `native:build:release` for the `childrenMs` check.

Harness, in a disposable clone of this repository with its own `HOME` and `XDG_RUNTIME_DIR`:

```sh
# one arm of the one-path commit the recorded run landed, and the same commit by real Git
git -C "$CLONE" commit --message '<subject>' -- package/git-policy/cli/src/native/worktree/worktree_removal_second_look.rs
/usr/bin/git -C "$CLONE" commit --message '<subject>' -- package/git-policy/cli/src/native/worktree/worktree_removal_second_look.rs
```

Each arm restored the same starting state (first parent checked out, the change staged),
ran in its own `systemd-run --user --scope -p MemoryMax=8G` scope,
and the wrapper's own record was read from `<clone>/.git/monochromatic-git-policy-cli-commit-timing.jsonl`.
The push was skipped by naming no push remote (`remote.pushDefault` `.`), so no arm spent time on a network.

Catalogs:

- a commit whose child is stalled: wall time 5 to 14 s, one entry of `steps` over a second,
  every other child in milliseconds, and a quiet gap for the stalled child of hundreds of milliseconds;
- a commit whose children are not stalled: the recorded subject's shape took `buildMs` 3 to 12 ms and
  whole wall times of 418 to 2028 ms against 78 to 165 ms for real Git.

## Verified workarounds

Read a slow commit from `steps` first, then from `childrenMs`:
a phase whose parts are all small but whose phase figure is large has its time inside a child, and
`childrenMs` names which part held it.
A stalled child is told from work the wrapper does by its neighbours: in a stalled commit every other
child of the same commit is in milliseconds.

There is no workaround that makes a stalled child fast, and none is wanted:
the wrapper cannot tell a child that will be stalled from one that will not, and re-running the child
would put a second command's guess into a commit.

## What does not work

- A merge conclusion as the cause: the commit that concluded one landed 45 s earlier in 962 ms.
- The size of the ref store: the same `git for-each-ref` over the 1944 refs of this repository takes
  0.00 to 0.01 s today, and the clone's 1117 refs are no different in kind.
- The size of the index or the worktree: both clones hold the same 23 669 tracked paths.
- A lock or a wait inside the wrapper: the stalled child was never quiet for more than 853 ms in the
  recorded run and 617 ms in the clone's, so it was doing work of its own, not waiting on a lock.
- The push: no arm pushed, and the recorded commit's `submitMs` was 356 ms.

## Upstream filing decision

Nothing to file against Git: a Git process that takes seconds while its siblings take milliseconds is
the host starving that process, not a Git behaviour, and no reproduction exists without the host.
The part of this that was the wrapper's own was the record's stretch hiding its child, and that is
fixed here (`fix(git-policy-cli): a build stretch says how much of it a Git child took`).
