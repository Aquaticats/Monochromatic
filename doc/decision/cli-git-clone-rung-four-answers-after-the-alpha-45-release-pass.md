# The clone rung: four answers after the alpha.45 release pass

## Status

Decided by the owner on 2026-11-23,
through the question tool,
on the four choices the release pass over alpha.45's clone rung left
(`~/temp/agent/release-pass-wrapper-alpha-45-2026-11-23/clone-rung/keep/review.md`,
"For the owner to decide",
D1 to D4).
Recorded by the session.

## A source whose only change is untracked (D1)

The question:
a checkout whose only change is an untracked file counts as dirty,
so the add judges it by the 762 MB whole-tree rule rather than the clean rule;
on twenty random 8 MB files on ZFS the add then took 380 to 630 ms through Git's checkout against 34 to 53 ms through the
clone,
in interleaved rounds.

The answer:
"Judge it as clean (Recommended)"

So a source with no tracked change,
only untracked files or stale stat data,
takes the clean rule
(no size rule on Btrfs;
the clean mean rule on ZFS and on kinds not measured),
and the add's message says so.
The partial clone with zero changes has no measured shape of its own yet;
it is measured after it lands.

## The dirty-tree limit on Btrfs (D2)

The question:
for clean trees the owner chose no size rule on Btrfs,
while the dirty-tree limit,
measured on ZFS only,
still applies on Btrfs.

The answer:
"Yes,
after one Btrfs round (Recommended)"

So one guest round measures the dirty shapes on Btrfs,
and the limit is dropped there if the clone is no slower than Git's checkout in that round;
until the round has run,
nothing changes on Btrfs.

The round ran on 2026-11-24 and merged as `edfd13bd90`
(`doc/planning/cli-git-clone-rung-dirty-shapes-on-btrfs-measured.md`).
The clone's pooled ratio to Git's checkout was below 1 on every shape it measured
(0.24 to 0.73 as medians),
so the condition holds,
and the limit is dropped on Btrfs only;
ZFS and the kinds not measured keep it.
The change is built in the round `pi-clone-rung-drops-the-dirty-limit-on-btrfs`,
on the branch `fix/git-policy-cli-clone-rung-drops-the-dirty-limit-on-btrfs`.

## What the whole-tree limit sums (D3)

The question:
the 762 MB limit sums every tracked file's recorded size,
including files the clone hands to Git
(on this repository's index 770.7 MB,
of which 244 MB is content behind `filter=lfs`,
so 526.6 MB is what the clone copies).

The answer:
"Sum only what it clones (Recommended)"

So the limit counts the bytes the clone copies,
leaving out the paths the fence hands to Git.

## A file an earlier conversion left the same length (D4)

The question:
a file added through a clean filter that keeps its length,
with the filter removed later,
stays stat-clean in the source;
the clone copies the person's bytes while Git's checkout would write the stored blob,
and both trees report clean,
because the size check cannot tell equal size from equal bytes.
The session ranked hashing the copied files first,
documenting the limit second,
and refusing on old filter configuration third.

The answer:
"Document the limit"

So `package/git-policy/cli/SPEC.md` and its README say that the clone rung takes a stat-clean file of the blob's size as the
blob's bytes,
name the case above as the one it cannot see,
and say that Git's own `git status` reports that source clean too.
The add does not hash the files it copies.
