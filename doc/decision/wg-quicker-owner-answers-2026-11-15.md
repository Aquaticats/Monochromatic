# Wg quicker owner answers 2026 11 15

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 45991 to 45992 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- wg lifecycle branch `cd83212379`,
  whose five privileged tasks never ran:
  typed,
  **"You run them now.
  sudo passwordless."**
  So the coordinating session runs the five tasks with `sudo` and merges when they pass.

From lines 46000 to 46006 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


Round eighteen (asked and answered between 21:03 and 21:05,
written here at 2026-11-15T21:05:52-05:00),
after the session's wrong description of the wg branch was corrected to the owner in the question itself:

- The four wg tests that change the host (`cli-quicker:test:integration:bypass` and `:route`,
  `cli-quicker-exempt:test:functional` and `:test:functional:protocols`):
  **"In a VM,
  none on the host (Recommended)"**.
  This replaces "You run them now.
  sudo passwordless.",
  which was given on the wrong description.
  Only `cli-quicker:test:integration:block` (no `sudo`,
  its own user and network namespaces) is run on the host.
- The wg branch `cd83212379`,
  with six reproduced findings of which four are the same code in `main`,
  one is worse in `main` and one is in code only the branch has:
  **"Merge;
  findings fixed on main (Recommended)"**.
  The merge is made once the one confined test passes;
  an author answers the six on `main`;
  nothing is deployed until they are answered,
  reviewed and the privileged tests have run in a VM.

From lines 46421 to 46421 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  Deployment still needs:
  this tip reviewed,
  and the privileged tests run in a VM against it (owner:
  "In a VM,
  none on the host").
