# cli-git design review

Independent review of the `monochromatic-git-policy-cli` crate (`package/git-policy/cli`) at commit `3f0bff8248`,
written 2026-11-01 by a reviewer who had not seen the design sessions.
The owner's questions, quoted: of "Git now runs the configured hooks itself inside the native commit", "what is going
on? Isn't this poor design?" and "just how many lurking horrors of architectural design are there?"
Nothing in the product was changed; this document is the only artifact.

## Verdict in brief

The shadow-repository transaction (private preparation, object migration, replay, capture order, subsumption,
journals, recovery) does not earn its complexity for the stated workload.
Three findings carry that verdict, each with its measurement in the sections that follow.

- In a 20,000-file clone a wrapped explicit-path commit takes a median 2740 ms (3043 ms with the repository's
  policies) against 45 ms for the real Git on the same file, and about 87 percent of the wrapped time (2377 of 2740 ms)
  is one defect, not the protocol: the private commit index is built with `read-tree`, so it carries no stat data and
  the native `git commit` inside the shadow rehashes every tracked file (2377 ms against 24.8 ms, 96 times, when the
  private index is a copy of the real index).
- The design's concurrency premise is that preparation is long and conflicts are rare.
  The recorded sweeps show about one lost landing race per commit at concurrency 8 (SPEC.md "Benchmark method"),
  so replay is the normal path, and the owner has since settled that the wrapper supports no external hooks at all
  (see the scope note), which removes the one long part of preparation.
  A serial design that builds each commit after taking a lock never replays, so the replay, capture-order,
  subsumption and starvation-reservation machinery has no reason to exist.
- A prototype of the serial plumbing sequence (no hooks, no policies, no signing, so a floor and not a result)
  committed in a median 64 ms alone and finished four concurrent commits in 299 to 404 ms, against 3430 to 4869 ms
  for the wrapper doing the same four.

The recommendation is not "rip it out first".
The owner's own instinct is right: "Shouldn't the proper fix be making our code faster?"
Fix the stat-less private index (small), then decide on the serial redesign (large) with these measurements.
Hook handling, the three self-start sites, the bounds and the repeated Git starts are separate, ranked defects.

## What was read and what was run

Read: `doc/decision/cli-git-concurrent-commits.md`, `doc/planning/cli-git-concurrent-commits.md`,
`package/git-policy/cli/doc/concurrent-commits-implementation-plan.md`, `SPEC.md` sections "Transaction protocol",
"Auto-push" and "Benchmark method", `doc/handover/wrapper-sync-attribution.md`,
`doc/planning/cli-git-spawn-and-sync-reduction.md`, the sync and gate sections of
`doc/handover/cli-git-rust-implementation.md`, and, from the unmerged branch `worktree-agent-a21d6fec9d0d04797`,
`doc/planning/cli-git-no-self-invocation.md` (commit `156415d696`) and the stat of `7cd936b6bc`, and the sections the
owner's hook corrections added to that handover on `main` after the base commit.
Source reads are cited as `file:line` relative to `package/git-policy/cli/src/native/`.

Important scope note: the hook change the owner quoted ("Git now runs the configured hooks itself") is not on `main`
or on this review's base.
It is commit `7cd936b6bc` on `worktree-agent-a21d6fec9d0d04797`.
On the base commit the commit hooks still go through the self-started dispatcher.
After the base commit, `main` (`99a7575f5a`) recorded the owner's corrections in
`doc/handover/cli-git-rust-implementation.md`, which I read after the first version of this report: the owner rejected
"Git runs your configured hooks directly", and then settled "B: Only hooks we declare, natively", meaning the wrapper
supports no external hook commands at all (Git-config `hook.<name>.*` commands included), every lifecycle stage is an
in-process policy or declared action, and a repository that still configures `hook.*` gets a loud explanation.
That handover also retracts a mistaken earlier entry ("preparation hooks are no longer supported", `f6817bb22f`).
This report treats the settled decision as the current state; the dispatcher and the branch are discussed as history.

Run, in a throwaway shallow clone of this repository under `~/temp/agent/dr/`
(20,218 tracked files, `/usr/bin/git` 2.56.0, installed wrapper `cli-git-native` 0.4.6, btrfs, a hermetic `HOME`
with no signing, no hooks configured in the clone except what Git's templates installed):

- Sequential and concurrent explicit-path commits, arms alternated and rotated per round
  (`exp-commit.ts`, 6 sequential rounds, 4 concurrent commits, 3 concurrent rounds), run as
  `systemd-run --user --unit=dr-exp-commit` with `MemoryMax=8G`, `CPUQuota=400%`, `TasksMax=3000`, `IOWeight=50`.
- `strace -f` of a wrapped commit, with and without a local bare remote, and `GIT_TRACE2_PERF` of the same runs.
- The refresh cost of a stat-less index against a stat-carrying one, alone and as the native commit step.
- The serial plumbing prototype under an in-process lock at 4 and 8 commits at once (`dr-exp-lean`, `dr-exp-lean2`).
- Forwarded read-only commands with and without the wrapper, `fsync` time inside a wrapped commit.

The load average was 62 to 126 on 28 CPUs for the whole review; no run waited for a quieter host.
Every unit was `--collect`ed and none remains; the one `fsmonitor--daemon` my first clone started was stopped.
The scripts are in `~/temp/agent/dr/` (scratch, not committed).
The gate (`mise run //package/git-policy/cli:native:test:container`) was not run: repeated full test runs are banned,
so the test and image findings rest on recorded measurements and on reading, and say so.

## Question 1: the shadow-repository transaction

### The stated reason, and what it was a reason for

The implementation plan's own account (`concurrent-commits-implementation-plan.md`, "Linked worktrees already
serialize every forwarded Git") is that concurrent commits collided on a wrapper lock: the TypeScript wrapper held
`index.lock` across every policy pass and the real commit, and the worktree-copy settlement lock serialized every
forwarded command with a 1 second budget.
Two of the three collisions the design answers were created by the wrapper itself, and the decision already removed
the settlement lock from the commit path.
The remaining real collision is native Git's: a second `git commit -- <path>` in the same worktree fails at once with
`index.lock` `EEXIST`.
I reproduced it: four concurrent plain commits of distinct files gave one success and three failures in each of three
rounds ("Another git process seems to be running in this repository").

The host already isolates agents by worktree: the shared checkout holds 142 linked worktrees (`ls .git/worktrees`),
each with its own index and branch, so the cross-worktree traffic is object writes and ref updates that Git handles
without a wrapper lock.
The design earns its keep only for several sessions committing into one working tree.
That is a real case, but it is the case a short lock solves.

### Measured cost of one wrapped commit

Sequential, one commit at a time, 6 alternated rounds each (milliseconds):

- real `git commit -- <file>`: min 38, median 45, p90 51.
- serial plumbing prototype (private copy of the real index, `add`, `write-tree`, `commit-tree`, compare-and-swap
  `update-ref`, `reset` of the real index): min 58, median 64, max 72.
- wrapper, repository with an empty policy list: min 2593, median 2740, max 3319.
- wrapper, this repository's policies: min 2938, median 3043, max 5530.

The wrapper is about 60 times the real Git in this repository, and the policies add about 300 ms.
The recorded figures fit the same cause.
The current harness (`bin/native-perf.ts`, `perf/native-latency-2026-10-20.json`) measures `commit-explicit-path` on a
fixture of thirty tracked files: wrapped median 179 ms against 5 ms direct, and the scenario is not enforced.
The older Node-era record (`perf/lifecycle-latency-2026-09-26-maintained.json`) used 2048 tracked files on tmpfs and
recorded 0.3 to 0.45 s of wrapper-added latency; 2048 files at the measured 0.12 ms per file is about 0.25 s.
That arithmetic is an estimate, not a measurement, but it explains why no recorded benchmark showed the problem:
the refresh cost grows with the file count and every fixture was tiny.

### The dominant defect: a stat-less private index

`commit_index.rs:192` to `commit_index.rs:199` builds the private index of every explicit-path commit with
`git read-tree <base>`.
An index built from a tree has no stat data, so any Git command that refreshes it must hash every tracked file.
The native `git commit` inside the shadow does exactly that.
Evidence:

- `GIT_TRACE2_PERF` of a wrapped commit shows the `index` region `label:refresh` taking 4.73 s of the 5.0 s the shadow
  `git commit` ran (a second run: 7.6 s of about 8 s).
  The refresh of a normal commit in the same trace took 0.013 s.
- `update-index --really-refresh` on a copy of the real index: min 28, median 31, max 34 ms over 5 rounds;
  on a `read-tree HEAD` index: min 2566, median 2588, max 3829 ms.
- The native commit step alone, private index from `read-tree` against private index from a copy of the real index
  with its timestamps kept (the same trick `copy_index_with_timestamps` already uses for index-mode commits,
  `commit_index.rs:175`): median 2377 ms (min 2140, max 4092) against median 24.8 ms (min 23.9, max 36.4).
  That is 96 times.

The fix is small: start the private index from a copy of the real index, then reset only the paths where the real index
differs from the base (`diff-index --cached` lists them, `update-index --index-info` rewrites them).
Wrapped time then drops to roughly 0.3 to 0.4 s, which is the remainder after subtracting the refresh from the
2740 ms median.
That remainder is still about eight times the real Git and is mostly process starts (finding 7).

### Measured cost of concurrency

Four commits of four different files started together in one worktree (milliseconds, one entry per round):

- real Git, no lock: one of four succeeded in every round.
- real Git under `flock`: 4 of 4 in rounds 1 and 2 with makespans 156 and 176; round 0 failed 3 of 4 because my own
  seeding commits ran outside the lock, so it is excluded.
- wrapper, empty policy list: makespans 4869, 3430, 3736; all four landed in every round.
- wrapper, repository policies: makespans 4293, 4235, 4530; all four landed.
- serial plumbing prototype under an in-process lock, 4 at once: makespans 299, 299, 299, 320, 404 and one 1066
  in the second run.
  A first run of 5 rounds showed one 11.8 s round that I cannot attribute (see "What I did not establish").
  At 8 at once: 601, 601, 601, 634, 731, 828.

The wrapper lands four commits in about 4 s where a serial lock around cheap plumbing needs about 0.3 s.
Parallel preparation does help the wrapper (four serial wrapped commits would take about 11 s) but only because the
work it parallelizes is its own 2.4 s rehash per commit.
With that defect fixed, the wrapper's serial part alone (the landing section, next) is as long as the whole of the
serial prototype's commit.

### The landing critical section

`SPEC.md` 1949 to 2008 holds both the landing lock and the real `index.lock` across object migration
(`pack-objects | index-pack`), a copy and recomputation of the real index, the journal writes and the compare-and-swap.
In the strace of one warm wrapped commit (strace inflates every figure) both locks were held for about 350 ms, and a
`git reset` that computes the post-index took 166 ms of it.
That grows with repository size.
The recorded fixture value (landing lock median 32 ms, `perf/concurrent-commit-latency-2026-09-26-maintained.json`)
is for a 2048-file tmpfs fixture and does not describe a 20,000-file repository.

### The premise of optimistic concurrency does not hold here

Optimistic preparation with replay pays off when preparation is long and the base rarely moves.

- Replay is the normal path under contention.
  `SPEC.md` 3753 to 3756 records, all at concurrency 8, 1.02 to 1.09 lost races per commit with the chosen setting
  `landing.reserveAfterLostRaces` = 1, and 2.5 and 3.0 with the settings 4 and 8.
  Each lost race is a `merge-tree`, a revalidation, possibly a `pre-commit` re-run, and for a signed commit another
  signature (the owner signs every commit with SSH: `commit.gpgsign=true` in the global configuration).
- Hooks, the part of preparation that can be long, were serialized by default even in the accepted design
  (`doc/decision/cli-git-concurrent-commits.md`, "Parallel preparation", last bullet), and under the owner's settled
  decision (no external hook commands) they are gone from preparation.
  What remains parallel is the policy pass (about 300 ms in this review, median of 6) and the signature (not
  measured here).
- The workload commits explicit paths with `-m`, so there is no editor.
- Every complication in the replay family exists because the commit is prepared against a base that moves: capture
  order (`SPEC.md` "Capture order"), subsumption, the synthetic merge base, the starvation reservation, the
  replay-time policy fingerprints, the `pre-commit` re-run outside the lock.
  None of them is needed when the commit is built after the lock is taken.

### What the design costs in code and checks

By line count in `src/native/` (counts include doc comments, which are dense in this crate):

- the families `commit`, `replay`, `transaction`, `recovery`, `capture`, `landing`, `shadow` and `private` hold
  about 21,800 lines of code and 15,400 lines of tests, of 87,600 and 93,600 in the whole crate;
- the owner-lock family (`owner_lock*`, `ownerless_directory`, `live_lock`, `process_identity`, `pid_namespace`,
  `recovery_dead_owner_locks`, `transaction_owner`, `transaction_lease`, `process_start`, `process_use`) adds about
  3,350 lines of code and 2,800 of tests.

Per commit it costs, from `binary_budget_tests.rs:45` to `binary_budget_tests.rs:76` (cold, no hooks): 39 Git starts
and 19 wrapper syncs; the real Git does 1 start and 0 syncs for the same commit
(`doc/handover/wrapper-sync-attribution.md`).
A warm commit adds a maintenance repack (4 more starts and 4 syncs) because each landing adds one pack.
My strace of a warm wrapped commit without a remote counted 46 `execve` calls (two of them the launcher and the
binary), 18 `fsync` calls, 37 `mkdir`, 28 `rename`, 18 `linkat` and 10 `symlink` calls, most of it lock, journal and
shadow bookkeeping.
With a remote and auto-push the same commit counted 60 successful `execve` calls and 20 `fsync` calls.

### Alternatives

Each is weighed on correctness (the policy sees exactly the committed bytes; crash safety; explicit-path semantics),
latency, Git starts and complexity.
The numbers marked "prototype" are the floor from my plumbing prototype, which skips hooks, policies and signing.

#### Option B: one bounded lock around the unchanged real `git commit`, policy on the staged index

The wrapper takes a per-worktree lock, runs policies, then runs `git commit -- <paths>`.

- Correctness: the policy reads the worktree or the staged index before Git reads the worktree again, so a file edited
  between the two is committed with bytes the policy never saw.
  Running the policy as a `pre-commit` hook would see Git's temporary index but means Git starts the wrapper, which
  the owner banned ("We should never invoke ourselves recursively").
  This is the disqualifying problem; B is rejected on correctness.
- Crash safety: Git's own, with the lock released by the kernel.
- Latency: 45 ms for the real commit plus policies; makespan for four was 156 to 176 ms (measured, no policy).
- Starts: about 3.

#### Option C: lock, private index from a stat-carrying copy, native commit in the real repository (recommended)

Under a per-worktree lock that the kernel releases on process death (`std::fs::File::lock`, probed on this
toolchain: a second handle's `try_lock` is refused while held and succeeds after the holder closes):

1.  Copy the real index with its timestamps; reset to the base only the paths where it differs from `HEAD`.
2.  `git add -- <paths>` into that private index (the bytes are captured here, in the lock).
3.  Evaluate policies on the private index's blobs, so they see exactly what will be committed.
4.  Run native `git commit` with `GIT_INDEX_FILE` set to the private index and no pathspec, against the real
    repository, with every hook off as the settled decision requires, so Git owns the template, message cleanup,
    signing, the reference update and the reflog, and the wrapper runs its declared post-commit actions afterwards.
5.  Update the real index entries of the committed paths (a short `index.lock`, using the existing foreign-lock wait).

- Correctness: the policy sees the blobs that Git commits, and no hook can change them between the check and the
  commit because none runs.
  Explicit-path semantics are unchanged because the private index is base plus the named paths.
  No replay exists, so there is no capture-order or subsumption rule to get wrong.
- Crash safety: a crash between the reference update and the real-index update leaves stale index entries for the
  committed paths, which is what native `git commit -o` leaves too.
  An intent record written before step 4 and repaired at the next lock acquisition would close it; I estimate a few
  hundred lines, which is a guess.
  The 14 journal syncs per warm commit protect only against power loss, never against `SIGKILL` (the page cache
  survives), and the commit data itself is not synced (the migration `index-pack` runs with `core.fsync=-pack`).
- Unprotected objects: blobs and trees are loose and unreferenced for the length of the lock.
  Git's default `gc.pruneExpire` of two weeks protects them from automatic maintenance; only a manual
  `git prune --expire=now` in that window deletes them, and native `git commit` has the identical exposure.
  The accepted design rejected this to survive `gc --prune=now`; the price was a pack per commit, three `index-pack`
  syncs and a geometric repack on the commits that follow.
- Latency (prototype floor): 64 ms alone; 299 to 404 ms for four at once; 601 to 828 ms for eight.
  The policy pass (about 300 ms measured) and the signature are inside the lock under C.
  Under the wrapper they are the only work that overlaps between commits, now that no external hook runs.
- Starts: the prototype used 7; with the native commit instead of `commit-tree` plus `update-ref`, about 8.
  In-process reads (Option E) cut the reads.
- Limits: an editor commit holds the lock while the editor is open.
  The owner's decision is needed: for agents (`-m`) it is irrelevant; for a human in the same tree it blocks others,
  who would wait behind the existing wait notices.
  An editor commit could instead keep the shadow path as a fallback, at the cost of keeping that code.
- Complexity: removes the shadow repository, alternates, object migration, `.keep` packs, replay, capture order,
  subsumption, reservation and the 247-cell tearing table (the hook lock and the dispatcher go with the owner's hook
  decision either way); keeps policies, candidates, config, the foreign-lock wait, auto-push.

#### Option D: per-agent worktrees as the isolation unit

- Correctness: perfect isolation (each agent has its own index, `HEAD` and branch).
- This already exists on the host (142 linked worktrees) and costs real things: a checkout of 20,000 files per agent,
  the ignored-state copying machinery (the `worktree` family is 6,950 code lines and 5,800 test lines in this crate),
  and an integration step, because agents that share one tree do so to see each other's edits.
- It does not replace C: the owner's workload includes several sessions in one tree.
  It complements C by removing most of the contention C has to absorb.

#### Option E: in-process object writing with the `gix-*` crates

The crate's `Cargo.lock` already carries `gix-index`, `gix-object`, `gix-lock`, `gix-tempfile`, `gix-hash`,
`gix-actor` and `gix-date`, through the scanner; `gix-odb` and `gix-ref` are not in it.

- Good fit: the read side (`cat-file`, `ls-files`, `diff-index`, `ls-tree`, `rev-parse`, `config`, `for-each-ref`).
  These are 27 of the 39 budgeted starts (`binary_budget_tests.rs:45`), and the owner has already queued this
  ("Invoke real git as rarely as possible").
  It pays under every option.
- Poor fit: writing commits.
  Replicating `git commit`'s hooks, template, cleanup, signing and conclusion state in process is the large
  surface the shadow design tried to avoid, and rewriting the index in process risks dropping extensions (untracked
  cache, split index, sparse, intent-to-add) that the wrapper's own SPEC already lists as hazards.
- Verdict: use it for reads and the private-index construction (stat transplant in memory), and keep the native
  `git commit` for the write.

### What would change my mind on question 1

- A declared native action or a policy that runs for seconds inside preparation: then preparation is long and
  parallel preparation has a real job.
  The settled hook decision removes external hooks, and this checkout has no `pre-commit` hook at all (`.git/hooks`
  holds `post-checkout`, `post-commit`, `post-merge`, `pre-push`), so today the longest preparation step is the
  policy pass, measured at about 300 ms.
  If the owner reverses the hook decision, this item flips.
- Sessions whose contract is "the bytes at invocation, not at lock time" for a file edited while the commit waits.
  The accepted design records that choice (capture at invocation); C captures at lock time, which is what native
  sequential commits do.
- Windows or network filesystems where `File::lock` is unreliable; then the owner-directory lock stays for those.
- A measurement that the serial lock's queueing delay under the owner's real commit rate is long.
  With C's hold of about 0.1 to 0.4 s the utilization at one commit per second is 10 to 40 percent; the owner's
  actual rate was not measured.

Confidence in the verdict: high that the current implementation is far slower than it needs to be (the 96 times is a
direct measurement and the trace2 region confirms the cause); medium-high that C removes the need for parallel
preparation (the floor is measured, the full C is not built); low on the exact makespans, which are floors.

## Question 2: other architectural defects, ranked by harm

The order is by harm observed in this review: first what costs every commit now, then what costs the owner's rules
and time, then what is structural but quieter.
Each entry gives evidence, the size of the fix (S under a day, M a few days, L a redesign), confidence, and what would
change my mind.

### 1. Stat-less private index makes every commit hash every tracked file

- Evidence: the section "The dominant defect"; `commit_index.rs:192`; trace2 `label:refresh` 4.73 s of 5.0 s;
  2377 ms against 24.8 ms.
- Fix: S.
  Copy the real index (timestamps preserved), then rewrite only the differing entries.
- Confidence: high.
- Would change if: the copy approach shows a fault the read-tree approach avoids (the racy-index hazard in
  `doc/troubleshooting/git-racy-index-copy.md` is handled by keeping timestamps, which index mode already does).

### 2. No measurement runs on a repository of realistic size

- Evidence: `SPEC.md` 3685 says the harness "no longer exist[s]", which is stale (`bin/native-perf.ts` exists), but the
  harness's fixture has thirty tracked files (`bin/native-perf-git.ts:182`) and its `commit-explicit-path` scenario is
  `enforced: false`; the older records used 2048 files on tmpfs with 2 CPUs.
  The budget tests (`binary_budget_tests.rs`) count starts and syncs but no wall time.
  That is how a 96 times regression survived slices that removed 4 to 14 syncs.
- Fix: S to M.
  Give the harness a fixture of tens of thousands of files and enforce a ratio against the real Git baseline.
- Confidence: high on the gap; medium on the explanation of why it survived.
- Would change if: a measurement on a large repository exists that I did not find.

### 3. Optimistic prepare, replay and land, instead of a lock

- Evidence: the verdict and the section "The premise of optimistic concurrency does not hold here"; lost races near 1
  per commit at concurrency 8; 21,800 lines of code.
- Fix: L (Option C).
- Confidence: medium-high.
- Would change if: the section "What would change my mind on question 1" is met.

### 4. Hook handling

On the base commit (history, superseded by the owner's settled decision):

- Hooks run through links to the wrapper itself (`SPEC.md` "Hook dispatcher shim"; `commit_command.rs:168` takes
  `std::env::current_exe()`): Git starts the wrapper per event, which re-enters through `argv[0]`, takes the hook lock
  and runs `git hook run`.
  This is the self-start the owner banned.

On branch `worktree-agent-a21d6fec9d0d04797` (commit `7cd936b6bc`, unmerged, and per the handover "must not merge in
its first form"):

- Git runs the hooks itself inside the native commit and the dispatcher is gone (1,776 lines removed, 1,020 added),
  which fixes the self-start.
- The hook lock is held around the whole native `git commit` child, so an open message editor keeps it; the handover
  says a waiting commit waits up to `CLI_GIT_LOCK_WAIT_SECONDS` (1800 s).
  Hooks no longer see `GIT_WORK_TREE`, and `GIT_CONFIG_PARAMETERS` changes.

My answer to "Isn't this poor design?": yes, and the owner reached the same conclusion from the other side.
That change removed one defect (the self-start) by making Git run user hooks inside the shadow, which exposed the
real problem: user hooks are written to run in an exclusive worktree with an exclusive index (stash, formatters,
`git add`), so running them in a parallel private context is only safe when serialized, and the lock then has to cover
whatever Git runs, editor included.
The hook lock was always a symptom of parallel preparation being unsound for hooks.
The owner's settled decision ("B: Only hooks we declare, natively") removes the cause: no external hook runs, the
dispatcher, hook lock, plan and lease go, and declared actions are wrapper code.
That decision is sound and also weakens the case for parallel preparation, since hooks were its only long step.

- Fix: the decision is made and an agent is carrying it; what remains is to carry the rollout order the handover
  gives (wrapper release with the native action, then the installer source, then removing the Git config hook).
- Confidence: high on the cause; medium on the editor behavior of the unmerged branch (read from the handover, not
  run).
- Would change if: the owner reverses the decision and lets repositories run parallel-safe external hooks, which
  `hooks.concurrentCommits` declared per repository.

### 5. Three more self-start sites and shell text on the base commit

Found by reading and confirmed in the strace of a push in a repository whose configuration has Git LFS keys:

- `entry.rs:614` (`lfs_hook_executable`) configures `hook.cli-git-lfs-pre-push.command` as `sh -c '<wrapper>
  cli-git lfs-pre-push'`.
  The strace of an auto-push shows `sh` starting the installed `cli-git-native` with `cli-git lfs-pre-push origin ...`
  as a child of `git push` (an `sh` and a second wrapper process per push).
  Without LFS keys (the rerun) there is no such start.
  The handover's newest direction replaces the hook with a parent-run `git lfs push`.
- `push_stacking.rs:142` runs `Command::new(request.own_executable)` for the merge push.
- `manual_push_probe.rs:229` writes a `#!/bin/sh` capture script for a dry-run push.
- Fix: S each, and already built in `156415d696` on a branch (phase 1: LFS upload through plain `git-lfs pre-push`,
  merge push in process, Clippy ban on `current_exe`); only the probe script remains open there.
- Confidence: high (seen in strace).
- Would change if: the installed 0.4.6 differs from the base commit's source in these sites (the strace is of the
  installed binary; the file and line references are of the base commit).

### 6. Bounds: one long default for every child, and notices that make a healthy slow child look hung

- `child_watch.rs:46` gives every child the wrapper starts for itself the same 30 minute limit; the lock waits use
  1800 s too (`CLI_GIT_LOCK_WAIT_SECONDS`); a push gets 30 s (`bounded_push.rs:46`) for fetch, hook upload and push
  together; queries 120 s (`bounded_query.rs:33`); a push waiter gives up at 95 s (`bounded_push.rs:53` margin) while a
  holder may run up to 10 reconcile rounds (`auto_push_reconcile.rs:39`) of 30 s each under the same lock.
  The no-self-invocation document already records the last mismatch.
- `wait_progress.rs:38` sets `NOTICE_GRACE` to 1 second.
  My first traced commit exited 0 and still printed to standard error: `cli-git: still waiting for git
  --git-dir=<shadow> ... pack-objects --revs --local --stdout --quiet (process N) after 1 seconds; stopping it after
  1800 seconds`.
  Under a load average of 60 to 126 any child over a second prints this; a healthy `pack-objects` looks hung.
  The same happened to `git cli-git check -- <this report>`, whose only output was `still waiting for
  monochromatic-lint ... after 1 seconds; stopping it after 60 seconds` for a check that then passed.
- A 30 second push deadline kills a healthy large Git LFS upload that runs inside the push as `pre-push`, and the
  failure is recorded as exit 124 and retried by the next commit.
  I could not exercise a slow upload (no LFS server in the throwaway).
- Fix: S.
  Bound by kind (read query, local write, network), print a child notice only past a size-derived expectation or a
  much longer grace, and derive the waiter limit from the holder's total budget (the document already proposes it).
- Confidence: high on the noise (observed); medium on the LFS harm (reasoned, not run).
- Would change if: a slow LFS upload inside the 30 second deadline succeeds in practice, or the owner wants a failed
  large upload to be retried by the next commit by design.

### 7. More Git starts than the work needs, and facts read repeatedly

- A wrapped commit with auto-push to a local bare remote and no LFS keys (the rerun, complete through
  `merge-base --is-ancestor` after the push): 60 `execve` calls, 58 of them children, among them 11 `rev-parse` and
  7 `config`; the push itself, from the first `config --get` to the last child, took about 0.5 s of strace time.
  The duplicates named in this entry were counted in the first push trace, which stopped at the LFS precheck before the push;
  the code path before the push is the same.
  After the one `config --list -z --show-scope` snapshot, auto-push runs six separate `git config --get` processes
  (`branch.<b>.remote`, `branch.<b>.pushRemote`, `remote.pushDefault`, `remote.<r>.push`, `push.default`,
  `branch.<b>.merge`); the long `rev-parse --is-bare-repository --git-dir --git-common-dir --show-toplevel
  --show-prefix` runs twice; `rev-parse --git-path index` twice; `rev-parse --verify refs/heads/<b>^{commit}` three
  times; `--git-common-dir` again for the push.
  Process starts matter here because the host had a fork-failure episode (598 cgroup limit events) and each start costs
  tens of milliseconds at this load.
- Fix: S per item, behind the queued in-process read facts (`RepositoryFacts`); the budget harness ratchets the count.
- Confidence: high.
- Would change if: an in-process replacement cannot match Git on packed or alternate object stores or unusual
  configuration (the handover's differential tests are the check).

### 8. Pack per commit, then a repack per commit

- Every landing adds one pack (`index-pack --keep`, `SPEC.md` 2063 to 2112), and in the strace of a warm commit
  `maintenance run --auto` then starts `repack -d -l -q --geometric=2`, a `pack-objects` and `multi-pack-index write`
  (4 starts, 4 syncs by the handover's count, about 96 ms of strace time).
  Direct Git writes loose objects and ran no maintenance in the same sequence.
  The handover notes the trigger is not isolated (pack count or midx) and that the first commit in a repository
  triggers none; I saw the repack on both warm traces and did not trace a cold one.
- Fix: M, moves with the alternates decision (loose or direct writes remove it).
- Confidence: medium (cause not isolated).
- Would change if: a repository with the pack count held constant still repacks on every commit, which would put the
  trigger elsewhere (the midx or the geometric rule).

### 9. Wrapper-owned locks reimplement what the kernel gives

- Evidence: capture, landing, reservation, hook, push and trust locks are directories created through a pending name,
  a synced `owner.json` and a rename (strace: `capture.lock.<uuid>.pending` then `rename`, `fsync`, `linkat`), with
  liveness decided from process birth identity, zombie handling and PID-namespace rules, a dead-owner recovery pass at
  every start, and a 20 ms polling wait (`owner_lock.rs:46`).
  That is about 3,350 code lines and 2,800 test lines.
  `std::fs::File::lock` is stable on the toolchain (`rustc 1.101.0-nightly`), releases on process death by the kernel
  (documented `flock` behavior; the death case was not run here), and blocks instead of polling.
- Git's own `index.lock` must stay an `O_EXCL` file with the foreign-holder wait.
- Fix: M for the wrapper-owned locks; the journals shrink with Option C.
- Confidence: medium.
- Would change if: the target filesystems or Windows ACL requirements make advisory locks unreliable for the owner
  (the decision lists Windows as supported), or a killed holder fails to release on this host.

### 10. Two or more mechanisms per failure mode

- Process exhaustion: retry of refused starts (`start_retry.rs`, 5 s budget) and a second loop that reruns Git commands
  by matching Git's English failure text (`transaction_git.rs:414`, `unable to create thread`, `cannot fork`).
  The no-self-invocation document itself notes that Git's messages are translated.
  The fix for both is fewer starts (finding 7); the host's 598 limit events were produced partly by the wrapper's own
  fan-out.
- Lock holder death: `core.lockfilePid` PID files, `owner.json` with birth identity, journal reconstruction, the
  foreign-lock evidence scan (`/proc` holders, `lsof`, Restart Manager), and recovery at every start.
- Waiting: polling loops in `owner_lock.rs`, `index_lock_wait.rs` (quadratic backoff with jitter), the landing loop,
  `auto_push_reconcile.rs` (10 rounds) and `worktree_copy_lock.rs`, each with its own cap and notice.
- Fix: M, mostly falls out of Option C and finding 9.
- Confidence: medium.
- Would change if: each loop guards a failure the others cannot see (for example the foreign-lock scan covers
  holders that are not wrapper processes, so it stays under every option).

### 11. Durability spent on journals while the data it describes is unsynced

- A wrapped commit makes 14 wrapper `fsync` calls after the reductions and the migration `index-pack` runs without a
  sync.
  Total time inside `fsync` in four traced commits: 155, 47, 58 and 160 ms of 4.6 to 23 s of wall (1 to 3 percent,
  strace-inflated).
  So it is not what makes commits slow today.
- The incoherence: the journals survive a power loss intact while the commit's objects and the reference update (Git's
  own, unsynced by default) may not, and recovery "does not notice" lost pack data (handover, accepted risk).
  Against `SIGKILL`, the normal condition here, they add nothing.
- Fix: S (drop to the barriers before side effects, already analyzed) or free with Option C.
- Confidence: medium.
- Would change if: the owner's loss model includes power loss on the machine's SSD (the project rules say a reboot
  can make data on it unreadable);
  then the journal syncs are a stated choice, and the unsynced objects are the gap to close instead.

### 12. Tests and the gate image

All from recorded measurements and reading; I did not run the gate.

- The gate builds an image whose dependency compile is one layer placed after `COPY package`
  (`bin/native-test-containerfile.ts`: the `cargo nextest ... --no-run` line follows `COPY package /work/package`),
  so any source edit invalidates it and recompiles every dependency, including the linked scanner.
  `podman history` of the `development` image shows a 448 MB compile layer, about 13 minutes after the layer before it;
  the handover records the image build as 16 of 18 minutes of a gate.
  Another agent is already working on image performance (images tagged `imgperf-*` exist), so this may be in flight.
- Tests that start many Git processes one after another, one case at a time against a shared repository, restoring
  state with more Git: 43.6 s against 0.5 s for `paired_branch_options_match_git`, 39.9 s against 1.2 s for
  `a_busy_remote_is_merged_into_until_it_settles` (handover, "Why the gate's tests are slow"); the fix of running
  independent cases on workers over per-case fixture copies is partly merged (`953246f1ec`).
- A fixed 30 second wait in `takes_over_from_a_pusher_killed_mid_push` and literal tallies such as `(248, 116, 12)`
  break the rule that tests derive expectations from constants (MGN).
- The 78 planted controls and the 247-cell tearing table are a real asset for the current design and become a cost
  when the design changes; Option C would retire the tearing table.
- Fix: S to M each.
- Confidence: medium (recorded, not re-measured).
- Would change if: a fresh gate run (outside this review's ban) shows the compile layer cached across source edits, or
  the slow tests already run on workers.

### 13. Smaller items

- The installed launcher re-executes the real binary; I measured the cost at 2 to 7 ms per forwarded command
  (`rev-parse HEAD` 5 ms against 2; `status --short` 69 against 62; `version` 4 against 3).
  Not a defect; recorded so nobody chases it.
- The shadow's per-commit setup is about five starts (`init --bare`, `for-each-ref`, `update-ref`, `symbolic-ref` and a
  `config` read) plus a directory tree, a config file and 8 to 10 symlinks, all of which exist only for the shadow.
- Index-mode commits copy the real index with timestamps (`commit_index.rs:175`) and explicit-path commits do not; the
  difference between the two paths is the defect of finding 1.

## What I did not establish

- The full Option C was not built; its numbers are the floor of a plumbing prototype without hooks, policies or
  signing.
  A signed commit adds an `ssh-keygen` start to every arm; the wrapper adds another per lost race.
- The throwaway had no signing, no hooks beyond Git's templates (the real-Git arm ran a template `post-commit` shell
  hook that prints a Git LFS notice, while the wrapper turns traditional hooks off; that costs the real arm a few
  milliseconds, which is irrelevant to a 2.7 s difference).
- A first run of the prototype's concurrency test had two rounds of 11.8 s and 42 s that I did not reproduce in a
  second run of 6 rounds (maximum 1.07 s).
  At a load average of 126 a scheduler stall is plausible; I cannot attribute it.
- Run-to-run spread is reported as min, median, p90 and max over 5 to 6 rounds; this is a small sample and the host was
  never quiet.
  The ratios in finding 1 are large enough (96 times against spreads of 1.5 times) that noise does not threaten them.
- The recorded per-commit numbers in `SPEC.md` come from older fixtures and Git 2.47.3; I did not re-derive them.
- No mutation, interop, Windows or macOS behavior was exercised.
- The unmerged branch was read through `git show`, not built.
- The first auto-push trace stopped at the wrapper's Git LFS precheck, because my throwaway had `lfs.*` keys and no
  `git-lfs` on its hermetic `PATH`.
  I removed the keys and reran once; that run completed the push and the post-push `merge-base`.
  The push record write and `last-pushed.json` handling were not examined separately.
- CPU quotas differ between my arms: the four-at-once wrapper arms ran in a unit with `CPUQuota=400%` and the
  prototype in units with `CPUQuota=200%`.
  The wrapper's rehash is CPU-bound and the prototype is not, so the quota disadvantages the arm I am measuring
  against, not the prototype; the real-Git arms ran in the first unit.
  I did not repeat the runs with equal quotas.
- The policy check of this report (`git cli-git check -- doc/planning/cli-git-design-review.md`, through the PATH
  wrapper) ran after the first commit, which I made with `/usr/bin/git`.

## Suggested order of work

1.  Fix finding 1 (stat-carrying private index) and add a wall-time ratchet on a 20,000-file clone (finding 2).
2.  Carry the owner's settled hook decision and the LFS rework (findings 4 and 5), which also remove the three
    self-start sites; the dispatcher branch is not to merge in its first form.
3.  Derive bounds by kind and quiet the one-second child notice (finding 6).
4.  Land the in-process reads behind `RepositoryFacts` and remove the repeated discovery (finding 7).
5.  Decide Option C with the owner using these floors, then build it behind the existing tests; retire shadow,
    replay and the tearing table only after the replacement passes the container suite.
