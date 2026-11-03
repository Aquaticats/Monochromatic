# A main-checkout commit within native plus five seconds

The owner's bound: "commits vs native git should not take 5s longer." And on a report that main's commits took up to 184 s
because the disk was saturated: "that's not an excuse." And: "Commits should never fail."
This note splits the slow commit of 2026-11-03T10:33:28.419Z, compares a wrapped commit with a native one step by step, and
records what was changed, with the numbers before and after.

## The split of 184 s

The commit is the line of `.git/cli-git-commit-timing.jsonl` with `requestId` `af35c85f-8aaa-4962-8b2b-75a9bf09c1a1`
(main, release 0.5.23, `git commit --message=... -- doc/handover/cli-git-rust-implementation.md`):

- `totalMs` 184201, of which `prepareMs` 61384, `buildMs` 25043, `submitMs` 70977 (`answerWaitMs` 70976),
  `afterLandingMs` 26532, `gateMs` 194, `handoffMs` 34, `otherMs` 34. The parts add up to the total.
- The service's `answered` line for the same request (journal of `cli-git@fc85fdf7a58f866b7f7a84b44fe0b69a.service`, 05:33:01
  local): `queued_ms=0 lock_wait_ms=0 lock_place_ms=0 landing_ms=70976`, and inside the landing `reads_ms=4 objects_ms=12
  journal_ms=1 ref_ms=2 sync_ms=70955 lock_hold_ms=70958`.
  So the 71 s were not a queue and not a lock held by another program: the request started at once, and 99.97 percent of its
  landing was `sync_ms`, the follow-up the service runs while it holds Git's index lock (`follow_up` in
  `src/native/service_land.rs`): `git diff-tree -r --name-only -z --no-renames <old> <new>`, then `git reset -q <new>
  --pathspec-from-file=-` over a copy of the index. The journal shows both as running steps the whole time: the `diff-tree`
  past 5, 15 and 25 s, then the `reset` past 10, 20 and 30 s, and the answer at 05:33:01.
  Other sessions' requests waited behind this batch (for example `bf26e603-a76b-4e31-886a-e1d087a76110`, a request answered
  `branch-moved` after 32.6 s in the queue), not the other way round.
- The client side, from the lines the client printed: `prepareMs` is `git read-tree --reset <HEAD>` (past 20 s) and then
  `git add --all -- <the file>` (past 20 to 30 s) in the private index, plus the in-process policy work; `buildMs` is the
  one `git commit` of the scratch repository (signing is on in the person's global configuration, so every commit is built
  by Git, never by the service); `afterLandingMs` in 0.5.23 was the summary `git diff-tree --shortstat --summary -M <old>
  <new>` (past 25 s). The current source prints Git's own summary and starts no such step (`landed_summary.rs`).
- The same pattern holds for the other main commits of the window: 216363 ms (`sync_ms` 85344 of a `landing_ms` of 85427),
  168794 ms, and later 299988 ms with `prepareMs` 120013, `buildMs` 59790 and `submitMs` 120009 (`sync_ms` 119429).
  The linked worktrees of the same minutes took 2.7 to 4.4 s each, of which `buildMs` was 2.3 to 3.3 s, where the releases
  before 0.5.23 had 50 to 150 ms (the timing file's medians per release).

What every slow step has in common is measurable. Of the Git starts of one wrapped explicit-path commit, these read the index
and connect to the file-system monitor socket of the repository (`strace -f -e trace=execve,connect` of a wrapped commit on
a throwaway repository with the person's global configuration): `read-tree --reset`, `add --all`, the scratch repository's
`commit`, and in the service `diff-tree`, `reset` and the review's `diff-tree --no-commit-id`. Six of them for the commit
and its landing, seven with the review; native `git commit` makes three connections, all from one process. The Git starts
that read only refs and objects (`rev-list`, `rev-parse`, `update-ref`, `cat-file`) make none.
On the throwaway repository, with the repository's monitor not listening, `git diff-tree HEAD~1 HEAD` took 1.60 s and
`git rev-list --parents --max-count=1 HEAD` 0.00 s (a repository with about 110 000 files and directories; the permitted
command stopped that repository's own monitor, the only one stopped).
On the real host at 2026-11-03T11:12Z the main checkout had no listening monitor, three `git fsmonitor--daemon start`
processes of 12, 34 and 38 s of age at 99 percent of a processor each (their parent chain: a Git started with the wrapper's
hook-off settings by `jbcontext watch run`), 244 monitor processes in all, and a load average above 40.
This is evidence, not a proof: the 184 s run was never traced, and Git's monitor is left exactly as the person configured it
(`README.md`, "Git's own fsmonitor"): nothing here sets, stops or avoids it. What the change below does about it is only that
fewer Git processes start per commit, and so fewer of them read the index.

## A wrapped commit against a native one, step by step

On a throwaway repository of main's shape (20 670 tracked files, a 2.8 MB index, 53 000 ignored files, the person's global
configuration with a throwaway signing key, `core.fsmonitor = true` and `commit.gpgsign = true`), release 0.5.23 source at
the branch point, an explicit-path commit:

- Native `git commit --message=x -- <file>`: one process (plus `ssh-keygen`), 6.1 MB written (the real index and the temporary
  index, 2.9 MB each, and the objects), three connections to the monitor from that one process.
- Wrapped, before: 14 Git starts in the client and 10 in the service (the service's landing and its work after the
  command), six processes connecting to the monitor, 20.5 MB written by the client (three index rewrites in the private index
  and copies) and 5.7 MB by the service.
- Full-tree and full-index operations the wrapper added: `read-tree --reset` of the whole base tree into the private index;
  `ls-tree -r --full-tree` twice; the service's `diff-tree`; a copy of the 2.9 MB index three times (reflinks on this file
  system; real copies on one without them).

## Changes

Commits, oldest first, with their timestamps, are in the final report of the work; each has a red control first.

1. Tree-to-tree name lists in process (`repo_read_query.rs`, `repo_read_tree.rs`): `diff-tree -r --name-only -z
   [--no-renames] <old> <new>` and `diff-tree --no-commit-id --name-only -r -z <commit>` for a commit with exactly one parent.
   The walk compares two trees together and reads only the subtrees whose names differ, so it costs in proportion to
   what changed. A root commit, a merge commit, a file that became a directory and a gitlink are declined (Git answers).
   The service's follow-up and the review then start no `diff-tree`, and neither connects to the monitor.
2. A commit whose real index already equals its base starts no `read-tree` (`commit_index.rs`): the private index is the copy of
   the real index, the in-process `diff-index --cached` reader says the copy holds exactly the base tree, and the question the
   readers leave to Git takes the old way. Budgets `WARM` and `AMEND` in `binary_budget_tests.rs` lowered from one start to zero.
3. The in-process object reader keeps what it decoded (`repo_read_pack.rs`, 32 MiB per pack) and a process keeps the lists of
   its last three trees (`flatten_tree_shared`). Measured on the throwaway repository, an in-process commit made 302 920
   `pread64` calls and was 270 ms slower than the same binary with `CLI_GIT_IN_PROCESS_READS=off`; the dozen questions
   of one commit each opened the pack again and read the whole base tree through delta chains without sharing a base.
   With the keeps, 12 alternating pairs gave 434 ms against 399 ms.
4. `ls-tree -r` is left to Git: listing a whole tree in process was 30 ms slower than the start and touches no index.

## Measured

Method: two byte-identical throwaway copies of the repository above (each with its own commit service, started by the
measurement, and its own runtime directory, so the service units are not rewritten between samples). Native Git ran in both
copies (the control for the spread), the base binary in one and the changed binary in the other, one commit of each in turn,
twenty rounds, an explicit-path commit of one file, no wait for a quiet host: the load average was 17 to 22 and the
input and output pressure `some avg60` 3.9 percent during the runs.

- Native, copy of the base: median 63 ms, 61 to 115 ms. Native, copy of the change: median 61 ms, 58 to 103 ms.
  The spread of the same binary on unchanged input is under 5 ms around the median.
- Wrapped, base (the branch point of this work): median 651 ms, 632 to 711 ms, so 588 ms more than native.
- Wrapped, changed: median 437 ms, 423 to 818 ms (one sample at 818 ms), so 376 ms more than native.
  Phases from the timing file (medians): prepare 358 to 194 ms, gate 174 to 130 ms, build 47 ms, submit 66 to 62 ms.
- Git starts of one commit: client 13 and service 12 before, client 12 and service 10 after. Processes connecting to the
  monitor: six before (seven with the review) and three after, native one.
- Bytes written, client plus service: 26.2 MB before, 22.6 MB after, native 6.1 MB (the difference after the change is
  the private index written twice and the copies of the index, which reflink on this file system).
- Same binary, in-process reads on and off, twelve alternating pairs: 839 against 525 ms before the object keeps, 690
  against 424 ms with the delta keep, and 434 against 399 ms with the keep of the tree lists.

These are the quiet-load figures. They do not reproduce the 184 s; they show the work the wrapper adds when nothing stalls, and
the count of processes that can stall. The stall itself needs the real checkout and its monitor, which no measurement here may
touch. The bound is met on this host by a wide margin when nothing stalls (376 ms against 5000 ms) and is not shown to be met
when the processes that read the index wait for the monitor, which is the decision below.

## Left to the owner

- Git's monitor and the three-to-one gap: a wrapped commit still starts three Git processes that read the index (`add`, the
  scratch `commit`, the service's `reset`) where native starts one. Each is a start that waits when the monitor of the
  repository is not listening. Fewer would need the private index built without `add` (a hash in process, which would skip
  the person's clean filters, so it is not offered) or the scratch commit built without a worktree.
- Commit failure paths that load or a slow machine can reach: see the final report.
- The benchmark `native:perf` runs its fixtures with `GIT_CONFIG_GLOBAL=/nonexistent-global-config`, so it never sees signing
  or the monitor; its 5 s bound cannot catch what the person's commits pay.
