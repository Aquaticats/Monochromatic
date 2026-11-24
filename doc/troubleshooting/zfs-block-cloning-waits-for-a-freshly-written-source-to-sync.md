# Zfs block cloning waits for a freshly written source to sync

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 42088 to 42095 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  `native:test:host` cannot run here without `cargo-nextest` on the checkout's `PATH`.
  The coordinator read on the host:
  `zfs_bclone_enabled` 1,
  `feature@block_cloning` active,
  `zfs_bclone_wait_dirty` 1.
  OpenZFS `zfs-2.4.4`
  `module/zfs/zfs_vnops.c:1839` to `:1857` (read;
  excerpt in
  `~/temp/agent/openzfs-bclone-wait-dirty-source-check-2026-11-19/keep/`):
  a block created in the current transaction
  group makes `dmu_read_l0_bps` answer `EAGAIN`,
  and with that setting the clone waits for the next group to sync and
  retries,
  so a freshly written source waits and a synced one does not.
  The review's sources were freshly written;
  how
  much of its 1.2 s this explains is unmeasured,
  and the run was steered with these lines.
  The wrapper removed the
  step 3 worktree when the pass merged the branch (its landed-worktree removal),
  so it was made again at the same path.

From lines 42101 to 42108 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  (363 turns) at `6c72f75b3e` (six commits;
  that folder's `keep/report-from-summary.md`).
  Measured against `main`
  built the same way,
  alternating:
  the clean-source add was 1.76 to 2.23 times `main` (about 90 per cent of it the
  one-thread per-file clone,
  not the checks and not a wait);
  after cloning the tracked files in parallel slices under
  the measured eight-worker ceiling,
  0.51 to 1.57,
  median 0.66.
  `zfs_bclone_wait_dirty` measured:
  a source rewritten
  within the transaction group costs at most about 0.7 s once (median +288 ms over a synced one).
  The dirty-source add
  stays 0.98 to 1.59 times `main` (about +0.2 s:
  the checks `main` skipped on ZFS);
  two attempts reverted or priced out.
  The `/proc` control now fails with the rule reverted (shown red here);
  README,
  plan step 3's guest-arm needs,
  `SPEC.md`
  and the `doctor` line corrected;
  `cargo fmt` and oxlint lows dismissed as `main`'s.
  The 27 GB of timing repositories
