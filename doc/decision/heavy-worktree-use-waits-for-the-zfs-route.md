# Heavy worktree use waits for the zfs route

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 43575 to 43592 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  branches it builds on. WHAT A NEW WORKTREE COSTS NOW, measured on those ten adds, one after another, through the session's
  wrapper build (`0.6.1-alpha.44`, built by cargo): 13.0 to 58.4 s each (median 24.6 s), each copying 38,698 ignored
  entries; one new worktree is 5,820 MiB by `du`; the pool's `bclonesaved` rose from 802 G to 847 G over the ten, about
  45 G that this record does not attribute, because builds and containers clone blocks too. That wrapper build is of
  2026-11-17T18:33:13-05:00, older than step 3's merge (`83d844942e`, 2026-11-19T21:23:36-05:00) and every later ZFS step,
  so an add in this session still takes the route the owner's hold of 2026-11-18 was about. 34 worktrees were then under
  `.claude/worktrees` on the new drive, where the owner's bound for the ZFS steps is 64 (`git worktree list` also prints 22
  registrations marked `prunable`, whose folders were on the old drives; a count of 56 that this session first wrote here
  took them for worktrees). Four superseded integration worktrees, each shown clean, pushed and with no process working in
  it, were removed and their branches kept (`integrate-clone-rung-partial-clone`,
  `integrate-fence-size-read-beside-source-check`, `integrate-fence-size-read-with-findings-fixed`,
  `integrate-object-reader-declined-branches-controls-and-lows`), leaving 30. The hold on other work stands, and the owner
  is asked about it with these figures. Guests, corrected: the first answer to the owner said one more guest fits and two do
  not, from swap alone; the unit that holds the guests may keep 33 GiB resident and held 2.5 GiB, so about four more guests
  of 8 GiB fit by that cap, and what gives first is the host's memory, which builds share. Five design runs were sent
  notes: the lock and binding-record designs, that plain `git` is the wrapper, so a probe is labelled as run through it and
  Git's source decides what Git does (a probe of Git alone may run inside a gate image already present, never on the host);
  the deferred-destroy design, that two phrases its brief quoted as the proposal's are the handover's headings; the

From lines 43620 to 43624 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  the hold on other work the owner chose no option and wrote: "What? Why are we still creating worktrees the old way? I
  thought we've implemented enough?" So the hold stands unanswered, and the answer given was: the `git` every shell in the
  checkout runs was the cargo build of `main` at `899b8e7cbd` from 2026-11-17, older than every ZFS step; of the merged
  steps only step 3 changes an add; the clone route for an add is step 10, with the machine's one-time setup in step 19.
  Asked when to switch that build, the owner answered "B: switch now". THE SESSION'S WRAPPER WAS SWITCHED at

From lines 43641 to 43648 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  unit, which was stopped. MEASURED ON ONE REAL ADD AFTER THE SWITCH (a detached worktree of `main`, diagnostics on, then
  removed): 67.6 s. The route, in the wrapper's words: "Git checks the worktree out (no rung applies; snapshot:
  Some("/home/user/Monochromatic is not a btrfs subvolume"); clone: None)", from a checkout it found "snapshotted with 4
  uncommitted paths". Git's checkout of 24,685 files took about 9 s, and the copy of 38,700 ignored entries 57.6 s (walk
  3.1 s, copy by eight workers 18.4 s, settling and modes 33.2 s; 32,404 entries cloned). So the new build does not make
  an add from this checkout faster (one sample, against the old build's ten of 13.0 to 58.4 s under other load, so no
  ratio is claimed): the checkout's uncommitted edits keep every rung out, which the partial clone branch addresses, and
  the wait is the per-entry copy of ignored files, which step 10's clone replaces. ALL TEN DESIGN ROUNDS ENDED `idle`

From lines 44357 to 44363 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  merged: IN MAIN as `308d3103d4`; the file is 673 lines. Still open for the owner: the run's suggested rule letting
  agents add a moved-path note to a decision record. By the owner's
  answers of
  2026-11-19T17:10:00-05:00 (questions file) the LFS review and the provider rebuild finish and nothing else off the
  ineligible option ranked first in the layout checker). By the owner's answers of
  2026-11-19T17:10:00-05:00 (this entry) the LFS review and the provider rebuild finish and nothing else off the
  ZFS path starts, and owner questions off the ZFS path wait in the questions file (new rule `QWP`).

From lines 44417 to 44418 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  instructions, three report claims). Held by the owner's "Finish, then hold" (2026-11-19T17:10:00-05:00) until the
  ZFS implementation was moving; it was by about 18:00 (step 1 and three designs going). GOING: the next round,

From lines 44596 to 44599 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **Held until a worktree in use frees, by the owner's "We shouldn't use worktrees heavily before we implement the
  zfs parts for the git wrapper." (2026-11-18):** four worktrees are in use now (the fold-in's, kept until its gate
  review; the short review's; the LFS audit's; the prototype's), and each new one copies the checkout's ignored files
  (38,685 entries for the prototype's). Waiting on that: the live quota-endpoint check (task 396, which can run in

From lines 44658 to 44658 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  owner's "Finish, then hold" of 2026-11-19: the rebuild `pi-rate-limit-fallback-hybrid-rebuild` (DeepSeek on

From lines 45590 to 45592 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

(25 of them) names the refusal, and in the user journal only the probe does. The owner, plain text: "I turned on the permission
model by default. And we should figure out how to work with it, not disable it." and "Shouldn't the probe be in rust
with hyperfine anyway?" So nothing here clears `NODE_OPTIONS`, and the disk probe becomes a Rust program timed by
