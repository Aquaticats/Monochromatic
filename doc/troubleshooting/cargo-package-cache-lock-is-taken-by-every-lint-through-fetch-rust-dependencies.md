# Cargo package cache lock is taken by every lint through fetch rust dependencies

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 24954 to 24957 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **A cargo home of its own,
  a second run's reading**:
  a clean `cargo check` took 67 s with no wait on the lock.
  It needs the `[build] build-dir` line copied from `~/.cargo/config.toml`,
  or builds land in the worktree's own `target/`,
  and a fixture made there is inside this repository,
  which fails one control for a reason of its own.

From lines 25073 to 25080 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **The cargo lock,
  measured by that run**:
  with a cargo home of its own in its scratch folder
  (`registry` and `git` linked to the shared ones,
  `config.toml` copied) no command waited on the lock;
  the whole-package lint that never started the round before (a wait of 529 s,
  killed) finished in 36.95 s,
  and compiles took 11 to 29 s "against 8m49s–3250s under the shared lock".
  This corrects the section of 13:08:25,
  whose heading said builds do not wait on each other:
  compiling does not hold the exclusive lock,
  but every `cargo build` starts with a resolution that takes it,
  so a build queues behind every fetch and resolution going.
  What is not shown:
  that two cargo homes sharing one `registry` stay sound when both unpack a crate neither has yet.

From lines 25115 to 25146 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

#### The cargo lock agents wait on is taken by every lint, through `fetch:rust-dependencies` (2026-11-13T13:08:25-05:00)

Not complete:
the cause of the one long wait is not found and no remedy is chosen.
Corrected at 13:14:
this section first said builds do not wait on each other.
Compiling holds only the shared lock,
but each build starts with a resolution under the exclusive one,
so builds do queue to start;
a run's measurement is in the section of 13:14:14.
Read and measured by the coordinating session.

- Cargo's source (`src/util/cache_lock.rs` on its `master`) names two lock files in the cargo home.
  `.package-cache` is taken exclusively "while downloading packages and doing resolution";
  `.package-cache-mutate` is taken shared "while a build is running",
  and the exclusive one "does not interfere" with it.
  So two builds compile side by side;
  only resolution,
  download and `cargo fetch` queue.
- At 13:06:00 the kernel's lock table (`lslocks`) showed two cargo processes holding the shared lock at once and nobody on the exclusive one.
- Every wait in the feeds of the runs of 2026-11-13 up to 13:05 (thirteen lines,
  in four runs) reads `Blocking waiting for file lock on package cache`,
  which is the exclusive one.
  Those read in context (the index-lock run's) were printed inside the root task `fetch:rust-dependencies`.
- That task (`mise.toml`) runs `cargo fetch --locked` once for each tracked `Cargo.lock`,
  thirty-five of them at `9f6f8312c4`,
  and `lint:monochromatic-lint` depends on it (decision of 2026-11-07,
  "Fetch then warn",
  in the comment beside the task).
  So each lint in each worktree takes the exclusive lock thirty-five times.
- One fetch of the wrapper's lock file with everything already downloaded:
  0.89 s (one reading,
  13:06).
  One whole pass of the task with sixteen agents going:
  22.1 s,
  with two waits on the lock (one reading,
  13:06:45 to 13:07:07).
- Not explained:
  the wait of 529 s one run reported at 12:16.
  Twenty-two seconds a pass does not make it unless many passes queue or one holder kept the lock for minutes.
  A recorder,
  started at 13:07:58,
  writes who holds and who waits on the lock every fifteen seconds for four hours
  (unit `record-who-holds-the-cargo-package-cache-lock-20261113`,
  log `~/temp/agent/cargo-package-cache-lock-measured-with-a-cargo-home-of-its-own-2026-11-13/keep/who-held-and-who-waited-from-2026-11-13T1310.jsonl`).
- The earlier plan (a cargo home for each run) is not shown to be needed for builds;
  one run made itself a cargo home in its scratch folder,
  which costs it a download of every crate.
- Next:
  read the recorder's log for the longest holder,
  then choose between making the fetch skip a lock file whose packages are all unpacked
  (read without the lock) and leaving the task as it is.
  A change of order between fetch and lint is the owner's (the decision of 2026-11-07).
