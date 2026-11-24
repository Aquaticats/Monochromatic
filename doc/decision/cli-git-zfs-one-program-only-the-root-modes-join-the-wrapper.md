# One program only: the ZFS route's root modes join the wrapper's executable

## Status

Decided by the owner on 2026-11-23,
through the question tool.
Recorded by the session;
how the root modes become modes of the one executable is still to be designed
(a proposal under `doc/planning/`),
and this record is the requirement that design starts from.

## What came before

The ZFS design gave the root side,
a ZED hook and the boot and shutdown units,
a program of its own,
`mount-git-worktree-clone-datasets`,
beside the wrapper `monochromatic-git-policy-cli`
(`doc/planning/cli-git-zfs-worktree-design-proposal.md`, "Binding crate and compatibility").
The owner had answered "Keep inside cli (Recommended)" for where it lives,
inside the wrapper's package rather than a package of its own.
Plan step 5 then added it to every release
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, "Step 5"),
although nothing installs or starts it until step 19.
The release pass over alpha.45 found its `--help` naming install paths that do not exist
(`~/temp/agent/release-pass-wrapper-alpha-45-2026-11-23/zfs-route-reach/keep/review.md`, M1).

## The question

Asked about the handler's words,
the owner answered with a question:
"What? Don't you think it shouldn't ship a 2nd program in the 1st place?"
The session then asked which of three the owner wanted:
hold the program out of releases until step 19,
keep shipping it as step 5 does,
or "One program only, ever",
described as "Fold the handler into the wrapper as a mode the root units call",
with the costs that root then runs the whole wrapper's code instead of a small program with none of Git,
LFS or network code,
that the design's split and its "wrapper starts without OpenZFS" check change,
and that it reopens a placement the owner decided.
The session ranked holding it out first.

## The answer

"One program only, ever"

## What follows

- The release holds one executable.
  The root modes (`boot-reconcile`,
  `shutdown-unmount`,
  `start-probe`,
  `adopt-worktree-parent` and the default sweep the ZED hook makes)
  become modes of `monochromatic-git-policy-cli`.
- Step 5's release changes are undone:
  the second `[[bin]]`,
  the wrapper's `checkBins` and `pinFiles` rows for it,
  the binstall row,
  the lock's `bins` row,
  and the archive's second program.
- The wrapper still links no OpenZFS library and starts where OpenZFS is absent;
  that check is not changed by this answer.
- How root reaches the modes (an argument,
  the name the program is started under,
  or another way),
  what the ZED hook and the units name,
  and how a root mode keeps the wrapper's Git,
  LFS and network paths from running,
  are the design's to propose.
- alpha.45 shipped with the second program,
  by the owner's "Let it finish (Recommended)" for that release;
  the change starts with the release after it.
- A later wish for another shipped program,
  such as an askpass helper,
  meets this answer first.
