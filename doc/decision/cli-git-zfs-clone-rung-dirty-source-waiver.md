# cli-git ZFS clone rung: the dirty-source waiver

## Status

Accepted 2026-11-20 in the question tool,
recorded 2026-11-20T05:45:48-05:00.
It waives the project's merge bar for one measured cost of ZFS plan step 3
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
"Step 3"),
and it ends when the partial-clone proposal named in "When it ends" lands or is rejected.
The question and its context:
`doc/handover/questions-for-the-owner.md`,
section "ZFS step 3 makes a worktree add from a dirty checkout slower;
what then?".

## Decision

ZFS step 3 merges with the clone rung (rung 2 of the worktree add ladder) taking ZFS,
answering "A:
merge with cloning on".
The project's merge bar says a branch may not make anything `main` does well worse;
this record waives it for one cost only:
a `git worktree add` from a source checkout that is not clean takes about 0.2 s longer than on `main`.

## Why the cost exists

The clone rung makes the new worktree by cloning the files in the source checkout and installing the source's index,
so it first proves the source matches its commit
(`package/git-policy/cli/SPEC.md`,
"Rung 2 needs a clean source").
On `main` the wrapper does not take rung 2 on ZFS,
so it never runs that proof on this host.
With step 3 it does,
and a source with any uncommitted change fails the proof and ends in Git's checkout,
having spent the proof's time.
Other sessions keep this machine's main checkout dirty,
so that path is the common one here.

## The measurements it rests on

These come from step 3's rounds on this host,
with the branch against `main` built the same way,
arms alternating,
over 24,313 tracked files:
`~/temp/agent/zfs-step-3-integration-pass-findings-fixed-2026-11-19/keep/report-from-summary.md`
and `~/temp/agent/zfs-step-3-short-review-findings-after-guest-arm-2026-11-20/keep/report-from-summary.md`.

- Dirty source:
  0.80 to 1.59 times `main`'s time over 20 rounds in four runs,
  with medians 1.25,
  0.93,
  1.24 and 1.17.
  The proof's steps:
  `git ls-files` 79 to 133 ms against the control's 45 to 50,
  `git check-ignore` 36 to 47 ms,
  `git status` 36 to 57 ms,
  and an in-process stat walk of about 34 ms.
- Clean source:
  0.51 to 1.57 times `main`'s time over 23 rounds in five runs,
  median 0.69.
- Disk,
  from step 3's guest arm:
  a clean add of the crate's `src` (2,456 files of 34,719,432 bytes) saved 22,979,072 bytes against Git's checkout.

## When it ends

The project also chose,
the same day,
"Yes,
design and measure it" for a different clone path:
clone every tracked file the proof shows unchanged and let Git write only the changed ones,
so a dirty source still clones nearly every file.
That proposal goes to `doc/planning/` with measurements and comes back for acceptance before any code changes.
If it is accepted and lands with a dirty add no slower than `main`'s,
this waiver ends with it;
if it is rejected,
the waiver is put back to the project.

## The miss behind it

The clean-source requirement came from the btrfs clone rung.
The designs,
the plan,
the step 3 rounds,
their reviews and the coordinator all measured and reported the clean source as the main case,
on a machine whose main checkout is rarely clean.
The project read that as the same mistake as waiting for a quiet host,
and tightened rule `NQH` in `AGENTS.md` to name the dirty checkout beside the loaded host.
