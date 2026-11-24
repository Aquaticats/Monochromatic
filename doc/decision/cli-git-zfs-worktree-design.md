# cli-git ZFS worktree design

## Status

Accepted 2026-11-19 in the question tool, under "Answered in the question tool, recorded 2026-11-19T16:22:16-05:00",
with its open items still open. That section left `doc/handover/questions-for-the-owner.md` on 2026-11-19 once this
record held its answers; it reads as recorded at `38ed22c5ea`.
The evidence,
the probes,
and the rejected options live in
[`doc/planning/cli-git-zfs-worktree-design-proposal.md`](../planning/cli-git-zfs-worktree-design-proposal.md).
The execution record is
[`doc/handover/cli-git-rust-implementation.md`](../handover/cli-git-rust-implementation.md).
The place the implementation interface will live is `package/git-policy/cli/SPEC.md`.

Amended 2026-11-20:
the open item "Staging under `~/.cache` and the wrapper's cleanup" is designed,
accepted that day,
and recorded in
[`cli-git-zfs-staging-under-cache.md`](cli-git-zfs-staging-under-cache.md);
the other open items stay open.

## Decision

The project accepted the ZFS worktree route for the Git wrapper, answering "Accept now (Recommended)":
the design is accepted with its open items still open.
The design keeps the checkout at its existing path and uses account-owned slots over their Git registrations.
The project chose "ZED, Clones, reflink. There is a reason we have a doctor subcommand."
The Git wrapper owns Git validation and registration,
unprivileged snapshot and clone creation,
the removal workflow and the same-dataset reflink fallback.
The root-installed ZED handler and its lifecycle unit own clone mount and unmount,
not Git or automatic deletion.
The account executes managed clone destruction under delegated `destroy`,
answering "Delegated destroy (Recommended)".
A normal authorized removal destroys the clone and its own snapshots in that invocation,
answering "B: Destroy at once".

Three further answers of 2026-11-19 settle what the design left open:

- The explicit cancel command, "Keep the command (Recommended)".
  A guarded `cancel-worktree-removal <slot>` stays as the manual way back,
  including for a removal whose wrapper was killed after it recorded the unmount or that ran past its deadline.
  It reports any snapshots already destroyed and cannot restore them.
- The deferred-destroy check, "Also check first (Recommended)".
  The check that the origin is not marked for deferred destroy also runs before the first snapshot delete,
  so such a removal refuses before this run deletes anything.
  The check before the head destroy stays, and a mark set between the two remains the accepted residual.
- The never-end set, "Spare the caller (Recommended)".
  The removal never ends the caller and the processes of its session,
  the shell that ran the command and that shell's other jobs.
  When the caller's session holds the clone, the removal refuses and says to leave the folder and run the command
  again.
  Other holders are still ended, as "Kill or otherwise remove or destroy the holders whenever --force is specified"
  says.

## What still gates shipping

The two owed VM measurements still gate shipping,
and they decide only the sentences already conditional on them:
forcing the removal of a clone a holder keeps,
and the refusal of a removal whose origin is marked for deferred destroy.
Both are owed under
[Forced removal and deferred origins][owed]
in the proposal.
The proposal states each part the decisions leave without a design,
and most of those parts as open items; this record does not copy them.

[owed]: ../planning/cli-git-zfs-worktree-design-proposal.md#forced-removal-and-deferred-origins

## Moved from the old handover

From lines 43960 to 43964 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  the staging re-check (and its round if it finds something), the question round. The owner, asked
  in the question tool at 2026-11-19T19:49:44-05:00 whether to put the three designs' questions now or after these second
  re-checks, answered "Wait for re-checks (Recommended)": they go in one question-tool round once all three re-checks
  pass, or after one more quick round if a re-check finds something (each first re-check had found the questions
  themselves wrong: a false premise under staging's Q2, lines deciding the owner's choices in the start probe, an

From lines 1798 to 1846 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  H1 and H2 weighed after the two research documents landed (read whole, review lines 19 to 206): both accepted. H1's
  location is decided, its fill is the clones document's item 7 sequence, its descriptor attach is measured in item 3;
  its VM gates 1 (the path-mount control), 3 (a rename loop over many mounts), 6 (a path swapped before unmount) and 7
  (a user namespace attaching the clone, `FS_USERNS_MOUNT`) stay open. H2's dataset and path are the owner's answer A,
  its pnpm consequence is measured; still owed are the checkout dataset's own boot mount (an entry the account can
  replace while unmounted: `chattr +i` or `mountpoint=legacy` with the descriptor attach) and a rule for nested
  datasets such as `zpcachyos/nosnap/vm-builder-output` under the checkout. Corrections of 2026-11-18 from the Sonnet
  gate review of the proposal (`~/temp/agent/zfs-worktree-design-proposal-gate-review-2026-11-18/review.md`, check B):
  item 7's sequence never ran end to end (the probe snapshotted before registering and mounted with `mount -t zfs`;
  dated note in the clones document); gate 5's setuid half is open too (only a device open was refused); the review's
  H1 sentence about detecting a renamed slot after attach is answered by the clones document (the attach reached the
  old inode), its H2 point 4 is wrong (`vm-builder-output` sits under `zpcachyos/nosnap`, nested by path only), and its
  H2 point 5 is wrong for `virtualStoreType: global` (zero payload bytes in the checkout). The owner then added `AGENTS.md` rule `WTU` (d507a55f0e) and
  the wrapper refusal for later (issue 6313), and asked for three digs, all GPT 6.2 Sol through Pi, **going since
  2026-11-18T19:46:28-05:00, 19:47:23 and 19:47:48 in this order**: the search for an earlier `AGENTS.md` worktree-place rule (unit
  `pi-agents-md-worktree-place-rule-search-1`, read-only in the main checkout, report
  `~/temp/agent/agents-md-worktree-place-rule-search-2026-11-18/report.md`); whether clones under
  `.claude/worktrees` conflict with ZFS (unit `pi-zfs-clones-under-the-checkout-research-1`, worktree
  `.claude/worktrees/zfs-clones-under-the-checkout-research`, doc
  `doc/planning/cli-git-zfs-clones-mounted-under-the-checkout.md`, VM probes; it ended `model-error` after 77 turns at
  2026-11-18T20:03:08-05:00 on OpenAI's filter, "This content was flagged for possible cybersecurity risk", right after
  reading its own log of the planted-link probes; its VM probes had all run, with results in
  `~/temp/agent/zfs-nested-checkout-mounts-2026-11-18-scratch/keep/`, no guest left and nothing committed; resumed from
  its session to write the document, unit `pi-zfs-clones-under-the-checkout-research-2`, run folder `run-2`, with the
  path-safety section asked as requirements rather than probe steps; main's driver refuses `--idle-seconds=0` until the
  idle branch lands, so it runs with 1; then moved to DeepSeek by rule `RKD`, recorded under the dig's landing below); and pnpm and other tools with the
  checkout as its own dataset (unit `pi-checkout-dataset-and-pnpm-research-1`, worktree
  `.claude/worktrees/checkout-dataset-and-pnpm-research`, doc `doc/planning/cli-git-zfs-checkout-dataset-and-pnpm.md`;
  it may leave a measurement folder under `~/.cache` that it must delete). **Next:** read the three, weigh the review,
  fold everything into the proposal with a writer run, then a Sonnet gate review of the folded proposal (rule `LHZ`:
  a design put to the owner is a gate), then the owner's acceptance through the question tool; the checkout-as-dataset
  choice waits for the pnpm answer (owner: "I'm not sure yet").
- **Two of the three ZFS digs landed in main:** `doc/planning/cli-git-zfs-clones-mounted-under-the-checkout.md`
  (merge of `docs/planning-zfs-clones-mounted-under-the-checkout`, 6e175a8d09; probes by GPT, written by DeepSeek
  under rule `RKD`, 41 turns; logs and scripts in `~/temp/agent/zfs-nested-checkout-mounts-2026-11-18-scratch/keep/`;
  the guest was made by hand with qemu because the mvm template build failed, and is gone) and
  `doc/planning/cli-git-zfs-checkout-dataset-and-pnpm.md` (merge of `docs/planning-checkout-dataset-and-pnpm`,
  7b646d71e9 and df9304ed19; GPT, 117 turns; it left nothing under `~/.cache`). The pnpm result: with the global
  virtual store, pnpm imports package payload only inside the home store, so a checkout dataset at the same path moved
  zero payload bytes into the checkout and four paired offline installs took 0.984 to 1.101 times their home controls
  (control spread 1.091); a clone's links into the home store break by path depth and the wrapper's relink repairs
  them; ranking shared home store > in-tree store > separate cache store. The checkout-as-dataset question goes to the
  owner with these results (answered A; then "who owns .claude/worktrees" answered B; decision entries below). The
  AGENTS.md search ended on its wall clock (67 turns) after writing its report at 20:09: no rule before `WTU`, none
  removed; the 2026-11-04 handover (031cfafaa6) shows a rule was proposed and answered "Wrapper fix only" (recorded in
  the planning page, "Where worktrees live"). **The old drives are not mounted:** the machine booted at
  2026-11-18T12:27:21-05:00 and `/mnt/old-sata` and `/mnt/old-nvme` are empty folders; `lsblk` shows `sda` and
  `nvme1n1p2`/`nvme1n1p6` as LUKS with nothing opened, so they are intact but locked. Unlocking needs the owner; they
  are needed for task 386's comparisons before the wipe, not for anything going now.
