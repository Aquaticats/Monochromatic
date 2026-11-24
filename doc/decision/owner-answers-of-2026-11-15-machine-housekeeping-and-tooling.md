# Owner answers of 2026 11 15 machine housekeeping and tooling

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 45955 to 45972 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


The machine:

- OpenSnitch's reject rules for freshly built programs:
  **"You remove the piled-up rules and you add regex allow rules so we'd never have to worry about it again."**
  The session reads the rule files first and asks the scope of the allow rules before changing the firewall.

Rounds twelve to fourteen,
written here at 2026-11-15T19:56:30-05:00:

- The 3,606 `rn-<number>-0` and 134 `rt-<number>-0` folders at the top of `~/temp/agent`:
  **"Delete those older than a day,
  and fix the tests (Recommended)"**.
- The IDE and a blank at the end of a wrapped row:
  **"Adopt the CSS rule (Recommended)"**.
- Container images held only by exited containers:
  **"Yes,
  after 24h."**
  (the option offered a week).
- The order of work:
  **"Pass 55 now,
  then the remaining questions (Recommended)"**.
- The firewall's allow rules,
  asked by folder:
  **"No no no. It should be regex allow rules matching specific programs.
  Allowlisting a whole worktree is never good.
  Like for example how I allowlisted codex."**
  Asked again by program,
  ticked:
  **"This repository's five tools as built"**,
  **"Tool managers in agents' throwaway homes"**,
  and typed:
  **"You're still proposing allowlist too much on the launchers.
  What happened to "allowlist per program"?
  For example,
  tokei doesn't need network."**
  So:
  one regex per program,
  the program's name at the end of the pattern,
  in the launcher folders too;
  no pattern ends in "any name".
  The session twice offered a wider rule than the owner's stated principle;
  the second time after being told.
- The two root-owned folders in an old build folder:
  **"Remove both,
  and fix the tests (Recommended)"**.

From lines 45974 to 45982 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- A token source for `mise` inside agents:
  **"Anonymous until a limit is hit (Recommended)"**.
- Four older picks,
  each ticked as confirmed:
  **"Port three fixes,
  leave the branch"** (copy-in-service),
  **"Launcher records heal on next run"**,
  **"Retire old-style build folders after 7 idle days"**,
  **"Limits share the sweep's reading,
  with those numbers"**.
- Four more:
  ticked **"Renewal stop has a measured ceiling"**,
  **"A commit does not wait for an earlier flush"**,
  **"The two tokio choices stand"**;
  and for the linter,
  typed:
  **"No,
  we need to follow oxlint's --fix.
  Oxlint isn't whitespace-only by default."**
  So `monochromatic-lint --fix` follows oxlint's `--fix` and the whitespace-only default is reversed.
  That option also carried the blockquote continuation,
  which the answer does not speak to;
  it stays as built until asked alone.
- Four small actions,
  each ticked:
  **"Add the four retired codes to your local file"**,
  **"Keep SSH_AUTH_SOCK blocked"**,
  **"Mount unit's stop succeeds when nothing is mapped"**,
  **"Remove the dead old-name units and 0.5.34 folder"**.

From lines 45998 to 45998 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- monochromatic-lint,
  a line the fixer breaks inside a blockquote:
  **"Add `>` to the new line (Recommended)"**.

From lines 33400 to 33408 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

`922fa7d1fb` merged the move tool's fixes,
and `cargo-shadow-lean.timer` is running again.
The old trees,
the source snapshots and the received first-pass copies of the move were deleted on 2026-11-10,
under the owner's "860 move:
You delete it."
and "Delete both (Recommended)".
No rollback to the NVMe remains.

From lines 45973 to 45973 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- `hall-monitor/daemon` and its lint rule:
  **"Move hall-monitor to package-paused altogether."**
