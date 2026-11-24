# Mpm shadow and cargo shadow owner answers 2026 11 15

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 45930 to 45946 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


mpm-shadow and cargo-shadow:

- Processes that hide what they run: **"Age rule, gap accepted (Recommended)"**.
- The removal's three limits (24 GiB, one hour, two seconds): **"Approve as proposed (Recommended)"**.
- llama.cpp builds: **"Limits: Only keep the newest 2 of any tool."**, and asked whether a pinned version goes too: **"Newest two plus pinned (Recommended)"**.
- A damaged row in a lock file: **"Whole file, and name it (Recommended)"**.
- A control group around a launched program: first **"I said to not hard depend on systemd, and graceful enhancement and graceful degration."**,
  then, asked for the reading: **"Where available, with a named exception list"**.
  The session's first wording of that question misstated the owner's rule as "stop depending on systemd"; the rule is no hard dependency.
- A `current` link that names an unfinished version: **"Launcher refuses with words (Recommended)"**.
- Lock files read again before each deletion: **"Read again before each deletion (Recommended)"**.
- An install that meets a removal of its archives: **"Wait up to ten seconds, then refuse (Recommended)"**.
- Flat downloads: **"Delete flat downloads (Recommended)"**.
- A good copy held aside while an install never finishes: **"Put it back when the install is proven dead (Recommended)"**.
- cargo-shadow's cleanup limits (296 GiB free, six hours, thirty minutes, pinned toolchains included when old): **"Approve as proposed (Recommended)"**.
- cargo-shadow's test task on a machine with no user service manager: **"Its own non-zero status (Recommended)"**.

From lines 45983 to 45988 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


Rounds fifteen to seventeen (asked between 20:25 and 20:39, written here at 2026-11-15T20:40:01-05:00), the last of the older questions:

- mpm-shadow, an install that finds another command holding the lock of the launchers and links it is about to change:
  **"Ten seconds, then refuse (Recommended)"**. One number with the archive answer; nothing is written without the lock; the message names the holder.
  So the stale-launchers branch's `Option<Guard>` (going on without the lock) is not what a merge keeps.

From lines 46381 to 46381 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  The brief gives the author one subject for this round, what a dead, starting or silent service costs a command (the connection's owner first, then the 120 s ceiling, the start killed, the owed push, the relink, `service stop`, two services, the named pipes, what the detached service inherits, the retirement and `SIGHUP`), each with a control that drives the wrapper's own start and fails at the reviewed tip first. The controls, guards and words of parts three, two-a and two-b are named as the round after it. It quotes the owner's "I said to not hard depend on systemd, and graceful enhancement and graceful degration." and asks the author to quote what `SPEC.md` and the recorded decisions say a command does behind a dead or silent service, and to name any case they leave open.

From lines 46391 to 46391 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  The brief has the author settle per finding, by a run on `main`'s build and on its tip, whether the tip is worse than `main` (several "brought" findings are protections `main` lacks altogether), fix those and every false sentence first, write what a cleanup cannot see as stated limits, correct the size cleanup to the owner's "Age rule, gap accepted", and then give each shipped protection a control shown to fail. The review's worktree is removed.
