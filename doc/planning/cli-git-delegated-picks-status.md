# Status of the coordinator's delegated picks for the Git wrapper

A read-only check of the picks the coordinator made under the owner's delegation of 2026-11-03
("Pick options as if I had picked them and invite veto when I say I'm back."),
run on 2026-11-04 against main at `c342673984`,
the commit design at `fix/git-policy-cli-commits-never-fail@6d053403db` (revision 19)
and cargo-shadow at `feat/cargo-shadow-cli@a1c6ea1ae8`.
Short forms:
`H` is `doc/handover/cli-git-rust-implementation.md`,
`AUD` is `doc/planning/cli-git-shipped-limits-audit.md`,
`NUW` is `doc/planning/cli-git-no-unbounded-waits.md`,
`MAR` is `doc/planning/cli-git-merge-auto-resolution.md`,
`D19` and `HIST` are the design and its review history on the design branch,
`CSU` is `doc/planning/cargo-shadow-update.md` on the cargo-shadow branch.
Line numbers are those the check read and may have moved since.

## Answered by the owner on 2026-11-04

Recorded in `H`,
"Owner decisions in force",
"Answers after the restart (2026-11-04)":
the shipped branch-write ceiling (never cut a branch write),
non-Git tools during `git worktree add` (silence plus a ceiling),
the design's R-never (reclaim the lane's own stale `index.lock`),
and the CM7 carve-out (CM7 unchanged;
the package docs record that the test images take tools from pacman).

## Built on `fix/git-policy-cli-branch-write-and-tool-ceilings` (2026-11-04)

Three lines for the items under the Answered by the owner on 2026-11-04 heading,
with the owner's words:

- Branch write:
  "Never cut a branch write (Recommended)" and "Short extra wait (Recommended)",
  "7 s (Recommended)".
  Built:
  a ceiling only for the exact forms the cut experiment proved (`NUW`,
  "The cut experiment"),
  none for any other form;
  a person's command inside such a step stops at the step's class bound plus 7 s ("Class ceiling + 7 s (Recommended)");
  the commit inside a stuck branch write returns at its own bound plus `SHORT_EXTRA_WAIT` (7 s) with the not-known error;
  a commit behind it is withdrawn at its own `ANSWER_CEILING` naming the write;
  a service stuck in a write never restarts itself.
- Non-Git tools:
  "Silence plus a ceiling (Recommended)" and "5x (Recommended)".
  Built:
  `REPAIR_TOOL_CEILING` 18 minutes,
  `LINT_CEILING` 4 minutes,
  `REVIEW_CALL_CEILING` 1 minute,
  `REVIEW_CEILING` 2 minutes,
  each five times the longest measured run rounded up to whole minutes (cold-store runs were shorter than the warm maxima).
- Pick 7 follow-up (`AUD`,
  2976 to 2991):
  done.
  `FOREIGN_OWNER_PATIENCE` is `2 * LOCK_CEILING`,
  20 minutes,
  derived in code.
  Its expiry no longer authorizes a takeover:
  the owner chose "Refuse and name it (Recommended)" on 2026-11-04 (round 3 of the branch
  write),
  and the commands that find an old record from another PID namespace stop,
  name it and change nothing.

## Picks the owner kept on 2026-11-04

Every pick in this section was put to the owner through the question tool in grouped questions,
and each group was answered "Keep all (Recommended)" (`H`,
"Answers after the restart (2026-11-04)").
None is open any longer;
the unmeasured numbers are still to be replaced by measured values as the logs fill.

### Waits and bounds (`NUW`, "Picks", shipped in 0.5.28)

- Quiet window of a connected push:
  150 s (picked B over A 30 s and C 264 s;
  unmeasured);
  a push with an open connection and no progress for 150 s is ended and queued,
  and the command exits 0.
  Ranking B > A > C:
  A is below the longest observed push (72 s);
  C holds later pushes of the branch for minutes.
- Foreign `index.lock` ceiling:
  10 minutes (`src/native/service/wait_ceilings.rs:30`,
  unmeasured);
  a commit waiting on another program's `index.lock` (for example an open editor of `git commit`) is withdrawn with an error and a side ref.
  Alternatives:
  30 minutes (moves the number),
  or telling a working holder from an idle one (needs a measure of a foreign process's work).
- Idle waits of the service have no timed bound;
  it exits after 3 hours idle.
- Numbers for the owner to approve (not picks):
  answer ceiling 20 minutes,
  start ceiling 60 s,
  service-hold and withdraw ceilings 10 minutes (`NUW`,
  "Bound values");
  `UNASKABLE_HOLD_AGE` 150 s (`AUD` 2376 to 2384,
  ranking a > c > b);
  the margin above the longest quiet gap of each quiet window.

### Git steps (`AUD` 2450 to 2466)

- A Git step that goes quiet is ended and redone once (R1),
  then fails with the reason;
  over redoing until it ends (R2) and doubling the window each round (R3).
  Ranking R1 > R2 > R3.
  Revision 19 says only "its step redone" (`D19` 1852).

### Shipped-limits picks that survived (`AUD`)

- Pick 1 (2291 to 2312):
  limits that change a result or a duration need a measurement or an owner decision;
  polls,
  display caps,
  chunk sizes and reader fallbacks are listed once as exempt.
  Ranking B > A > C.
- Pick 2 remnants:
  a lock holder whose pid file names a gone process ends the wait at once (a > b);
  polling when no file watch can be set;
  asking the service manager to start an inactive unit after the start wait;
  keeping a scratch file from being aged out during a long wait.
- Pick 3 (2392 to 2419):
  a commit built while another program moves the branch is rebuilt on the new head;
  the interim loop counts stay until the new protocol.
  Ranking C > B > A.
  The owner's "Stack on the new head" covered amend,
  conclusion and revert only (`D19` 1940 to 1962).
- Pick 5 remnants (2890 to 2912):
  a push with no connection is stopped after a 30 s window (uncited until quiet time is logged);
  the stop grace and the 4 MiB output cap are to be measured;
  the 120 s read-only query ceiling sits with the Git steps.
- Pick 7 (2562 to 2585,
  follow-up 2976 to 2991):
  done on `fix/git-policy-cli-branch-write-and-tool-ceilings`:
  the patience is twice the lock ceiling,
  20 minutes,
  derived in code (see the "Built on `fix/git-policy-cli-branch-write-and-tool-ceilings`" section).
- Picks 8,
  13,
  14,
  15 and their follow-ups (2587 to 2611,
  2786 to 2846,
  3019 to 3043):
  measured numbers (batch 256,
  local queries 8 and remote 4,
  lock age margin 7 s,
  free-space preflight per entry).
- Pick 11 (2710 to 2761):
  audit history kept up to 4 MiB and never dropped while it names a saved copy that exists;
  timing logs keep one generation.
  Ranking C > B > A.
- Pick 12 (2763 to 2784):
  an LFS server 5xx is outside our control;
  only the service's push retry plan retries it;
  a typed `git push` ends at the first 5xx with an explanation.
  Ranking C > B > A.

### Shipped lock removal (`H`)

- Q1:
  a lock a reboot left with no pid file is reported with the `rm` line,
  not cleared (no ranking recorded).
- Q2:
  the sweep after a stopped child judges every reference lock of the repository,
  not only those born during the child's life (no ranking recorded).
- Q3:
  its notice goes to the diagnostic log on standard error.

### Commit design picks revision 19 still contains (`D19` 2123 to 2162,
options in `HIST` 481 to 711)

- Rev 6 Q1:
  a path staged back to its already-consumed entry while commits are pending is left out of the plain commit,
  with a notice (native Git would include it);
  over B (capture every differing staged entry,
  which can revert a pending change) and C (set the commit aside until the person names paths).
- Rev 6 Q3:
  after an owner death,
  `update-ref` runs again,
  so a hook's external effect on "prepared" may repeat;
  over waiting for the person's confirmation.
- Rev 6 Q4:
  the instant inside `checkout` between the index write and the `HEAD` change can leave a rare staged change on the new branch (`git restore --staged` undoes it);
  over declining every follow-up after any foreign index write.
- A-kill (rev 8):
  a Git child that ignores the terminate signal is killed;
  its lock stays for the person,
  named in the notice.
- B-hand (revisions 8 to 11):
  a wedged own service is not restarted automatically;
  the command ends at the foreground bound with an error naming the unit and the restart command;
  over B-restart (review finding R8-1).
- C-kind (rev 9):
  `update-ref`,
  `commit` and anything that can run a hook are never cancelled.
  Ranking C-kind > C-freeze > C-sample.
- H-version (rev 9):
  a slow `pre-commit` hook runs once per version of the request.
- H-episode (rev 11):
  after repeated outside kills,
  the hook reruns once per recovery episode with a notice.
- G-order (rev 11):
  the saved recovery copy is released only after the maintenance registry is read again.
- N-silent (rev 11):
  a live owner retrying after a proven store failure prints nothing.
- Adopted for the design without asking (`D19` 244 to 253,
  2177 to 2189):
  protected programs run only under the service;
  a merge that cannot carry a staged or edited path as text leaves the person's version and writes the merged one beside it;
  a staged-only version is saved before the merge replaces it;
  a hold ref only for requests that wait past `HOLD_AFTER`;
  an invocation nonce in every request;
  signed commits re-signed in the background.

### Word-level second merge pass and pull (`MAR` 1980 to 2118)

- Q1:
  a conflicting autostash is applied as Git does and the stash and `UU` entries are carried back into the real repository (carry-back A,
  `src/native/conflict/autostash_carry.rs`);
  the dry-run decline (B) was retired.
- Q2:
  a pull with nothing stated,
  and `--ff-only`,
  stay with native Git's whole pull.
- Q3:
  the working-tree update after a fetch that moved the checked-out branch waits for the service's hold and any `index.lock` holder,
  bounded by `LOCK_CEILING`.

### cargo-shadow (`CSU`; branch only)

- The local base is the upstream branch,
  not `HEAD` (402).
- Release age and trust apply to versions new since the base,
  locally and frozen (434).
- CI stops on a policy refusal (436).
- The window in which a cargo that is not the shadow builds on an unapproved lockfile is accepted (439).
- The restore uses `RENAME_EXCHANGE` with an undo (228 and 368).
- A strict cutoff with exemptions (457).
- A `--breaking` failure after the catalog write restores the catalog (460);
  a killed `--breaking` run's manifests are journaled (461).
- One workspace `flock` replaced the earlier no-lock design (368).
- Chosen,
  unmeasured bounds on cargo-shadow's own calls (the user's command is exempt):
  cargo 30 minutes,
  Git 60 s,
  `file-enforcer write` 5 minutes,
  HTTP 429 pause 2 minutes,
  dump download 1 hour (349 to 352);
  the kept-files warning point 1,000 (377).

## Superseded,
decided or landed

- Superseded by owner answers:
  pick 2's unbounded wait and its follow-up (by the 10-minute lock ceiling);
  pick 5's "Wait while connected" (by "Bound it,
  keep retrying");
  pick 6's unbounded client wait (by the 20-minute answer and 60 s start ceilings);
  M-A merge order (by "Branch first,
  files after");
  P-wait (by "Merge anyway,
  unstage x");
  G-root (by G-order);
  automatic restart (withdrawn for B-hand).
- Decided by the owner:
  pick 4's principle ("End Git,
  hold hooks");
  pick 10 ("Keep pruning by age and size");
  P-carry and W-detect;
  cargo-shadow pick 9;
  the person's own programs ("No limit;
  Ctrl-C ends it").
- Landed:
  release contents (0.5.28 published 2026-11-04T03:09:07Z).
- Not an owner decision:
  shipped lock removal Q4 (long lines,
  left to the lint-debt pass).
