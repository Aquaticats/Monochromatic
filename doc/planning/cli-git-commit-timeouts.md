# Commit time-outs in the git wrapper: measurements and bounds

Scope: `package/git-policy/cli`, the commit path of the wrapper installed as `git`.
The owner's reports: "Some commits can time out" and "1800s for git pack-objects is laughably inappropriate."
This record answers where a commit's wall-clock goes, what the slow Git child really was, and how each child is bounded.

## Method

- Repository: a full local clone (copied objects, no hard links, 594 refs, 30 088 commits, 20 207 tracked files,
  seven packs of 648 MiB) of this repository in `~/temp/agent/gct/clone1`, never the real checkout.
  Its push URL was set to `DISABLED` before any wrapped commit.
- Commit: a docs-only explicit-path commit, `git commit --quiet -m m -- README.md`, through the release wrapper,
  hermetic environment, no hooks configured in the clone.
- Timing: `strace -f -ttt -e trace=execve` records every start and its exit, so each Git child has its own duration.
  `GIT_TRACE2_PERF` named the slow region inside the slow child.
- Arms were alternated in the order base, control, new, three rounds twice.
  The control is the unchanged base binary copied under another name, so base against control is the run-to-run spread.
- Load: the host ran at a load average of 80 to 145 on 28 cores throughout, as it normally does here.
  No synthetic load was generated and no quiet host was waited for.
- Scratch scripts (not committed): `trace-commit.ts`, `ab.ts`, `pick.ts`, `refresh-probe.ts` in `~/temp/agent/gct`.

## Where a commit's wall-clock went

Whole wrapped commit under strace, six runs per arm:

- base: 5.1 to 11.8 s, median about 6.2 s.
- control (same binary): 5.2 to 7.1 s, median about 6.0 s.
  The spread of an unchanged binary is therefore about 2 s, and the first cold run after a checkout took 20.8 s.
- new: 1.0 to 2.7 s, median about 1.5 s.

Children of the commit path, typical and worst over the runs of the base arm (seconds; the 11 `rev-parse` starts and
other repeated queries are listed once):

- `git commit` in the shadow repository (the real commit, hooks dir pointed at the wrapper's shims):
  4.3 to 9.8 s, 19.6 s on the cold run. Everything else together was under 1.5 s.
- `read-tree <base>` into the private index: 0.09 to 0.28 s.
- `reset --quiet <oid> -- :(top,literal)README.md` (real index update): 0.13 to 0.43 s.
- `ls-tree -r -z --full-tree <base>` (twice): 0.05 to 0.13 s.
- `write-tree`: 0.02 to 0.08 s. `add --all -- README.md`: 0.02 to 0.04 s.
- `ls-files --stage` (four starts), `diff-index` (four), `diff`, `diff-tree`: 0.007 to 0.15 s each.
- `rev-parse` (eleven starts), `config` (nine), `show`, `symbolic-ref`, `for-each-ref`, `update-ref`: 0.003 to 0.035 s.
- `git init --bare --template=` of the shadow: 0.005 to 0.029 s.
- `index-pack --stdin --keep=...`: 0.003 to 0.044 s.
- `pack-objects --revs --local --stdout --quiet`: 0.010 to 0.026 s in every arm, 0.065 s at worst in any run.
- `maintenance run --auto --quiet --detach`: 0.005 to 0.009 s
  (it detaches; the repack it starts runs 0.2 to 1.2 s later, off the critical path).

## Root cause of the slow commit: not pack-objects

The brief's hypothesis for `pack-objects` was an unbounded history walk. It is false.
`landing_objects.rs` already sends `<new_oid>` and `^<old_base>` on standard input, and the child ran 10 to 26 ms
against a 30 000-commit history and a 370 674-object store.
The `mgn.log` and `handover-retract.log` evidence files each hold exactly one notice for `pack-objects` ("after 1
seconds") and no repeat at 11 seconds, so that child ran at least 1 s and less than 11 s on the loaded real checkout.
A 10 ms process taking more than a second is run-queue latency at a load average above 50, not work.
A minute-long `pack-objects` was not reproduced. Candidates not tested: a detached maintenance repack from the
previous commit running at the same time, a large loose-object count, and the unborn-branch case, the one landing
with no negative revision.

The slow child is the shadow `git commit`.
`GIT_TRACE2_PERF` shows `index label:refresh` at 5.2 s over 20 206 entries (`refresh/sum_lstat`, `sum_scan`).
The private index of an explicit-path commit was built by `git read-tree <base>` into a fresh index file.
Entries made that way carry no cached stat data, so `git commit` has to read and hash every file of the worktree to
decide that it is unchanged. The cost grows with the worktree and with host load, and a cold page cache made it 19.6 s.

Live confirmation on the real checkout: while this record was committed through the installed (unfixed) wrapper,
the shadow `git commit` ran for more than 640 s with 5 s of CPU time, in uninterruptible disk wait
(`blk_mq_get_tag`), reading and hashing the worktree. That child runs with the terminal attached and no bound, so it
never printed the "still waiting" line; the commit looked hung for ten minutes.
This, not `pack-objects`, is how a commit takes minutes on a loaded host and why "some commits time out".

Probe on the same index and repository (three rounds each):

- `read-tree HEAD` into a fresh index, then `update-index --refresh`: 2.9 s.
- copy of the real index, then `read-tree --reset HEAD`, then `update-index --refresh`: 6 to 7 ms.
- `read-tree -m HEAD` on a copy behaves the same as `--reset`.

Git's own partial commit does the second: it resets a copy of the real index to `HEAD` with a one-way merge, which
keeps the stat data of every entry the two share.

## Fixes

### Private index seeded from the real index

`commit_index.rs`: an explicit-path commit copies the captured real index to the private commit index (timestamps
preserved, from the open file so a rename of another index over the path cannot give the copy its times) and runs
`read-tree --reset <base>` on it. Entries unchanged since the base keep their stat data; unmerged entries of the real
index are discarded. The tree the private index produces is pinned by the existing `commit_index` tests against the
incumbent, which pass.

The copy keeps one thing Git's own partial commit does not act on: an assume-unchanged or skip-worktree mark.
After the reset such an entry still carries the mark, and `git add --all -- <path>` then skips a modified
assume-unchanged file that the commit named (the commit exits 0 without it) and refuses a skip-worktree one
(exit 1, "matched paths that exist outside of your sparse-checkout definition"). An independent review found and
reproduced both. The fix is a safeguard, not a flag edit: `index_flags.rs` reads the entries of the captured index
in-process (no Git start; versions 2 to 4, SHA-1 and SHA-256, everything checked against real Git by its tests) and the
copy is used only when every entry was read and none is marked. A marked index, a split index, or a file the reader does
not fully understand builds the private index the old way, from the base alone, so those commits keep their previous
behaviour and cost. The earlier sentence "flags included, exactly as in Git's partial commit" was wrong about the
commit result and is withdrawn.

Reviewed and left as is: the fsmonitor and untracked-cache extensions are carried by the copy; the shadow
repository's own config enables neither, so Git ignores them when it reads the copy (by reading, no daemon was run).
`core.sharedRepository` is not applied to the shadow's subdirectories as `git init` applied it; the shadow root is
mode 0700 afterwards in both flows, so no other account reaches them.

After, same arms and clone (seconds): shadow `git commit` 0.16 to 0.49 against 4.3 to 9.8; whole commit 1.0 to 2.7
against 5.1 to 11.8 (control 5.2 to 7.1).

### `git init` of the shadow removed

`shadow_skeleton.rs` creates the seven directories and the `HEAD` of an empty bare repository (files ref backend) in
the wrapper. The shadow `config` and `alternates` were already written by the wrapper over what `git init` made, and
every later shadow step sets `HEAD`. A differential test compares the entries with real `git init --bare --template=`
(everything but `config`) and checks that real Git accepts the result. Reftable repositories still start `git init`.
Spawn budget: `init` goes from 1 to 0 in all four scenarios of `binary_budget_tests.rs`; every other count is
unchanged (the `config` read that follows the old `init` is the wrapper's own read of the repository format).

### Children left alone, with reasons

- `pack-objects`, `index-pack`: no cheaper in-process equivalent exists without a new dependency (the crate links no
  pack writer or reader; `gix-index` is only a transitive dependency of the scanner), and replacing them by loose
  objects was rejected earlier (it loses the `.keep` protection and three named tests).
  Measured cost is 10 to 50 ms together, so there is nothing to win.
- `reset` on the real index and `update-ref` on the real ref: both write files Git itself locks and logs
  (`index.lock`, reflogs, the `reference-transaction` hook, packed and reftable stores). Doing them in-process would
  copy that protocol, and no proof by the tearing sweep covers a re-implementation.
- `maintenance run --auto --detach`: 5 to 9 ms and detached; its decision rules are Git's.
- `rev-parse`, `ls-files`, `diff-index`, `cat-file`, `for-each-ref`, `config`, `symbolic-ref`, `show`, `diff-tree`,
  `ls-tree`, `rev-list`, `merge-base` are the read-only query children; they belong to the query-children owner and
  their counts and durations are listed above for that work.

## Bounds by class

`child_class.rs` classes each Git step by its subcommand; `child_watch.rs` stops the child at the class bound.

The first version put `reset`, `read-tree`, `update-index`, `ls-files`, `diff`, `diff-index` and `show` in the short
class on the evidence of a stat-clean index. A stat-dirty index (every file touched, content unchanged) changes that.
Three runs each, 20 207 files, warm page cache, load above 80, milliseconds:

- `reset -q HEAD -- README.md`: 27 to 101 clean, 2 930 to 5 771 dirty.
- `update-index --refresh`: 26 to 32 clean, 2 806 to 2 889 dirty.
- `ls-files --modified`: 57 to 58 clean, 2 803 to 2 862 dirty.
- `diff-index HEAD`: 17 to 51 clean, 93 to 102 dirty (compares stat first, hashes only on a size or mode difference).
- `diff-index --cached HEAD`, `ls-files --stage`, `write-tree`, `rev-list --count HEAD`, `merge-base HEAD origin/main`,
  `diff --stat --summary HEAD~1 HEAD`: 3 to 44 clean and 3 to 24 dirty, unchanged by the state of the worktree.

A killed refresh is discarded, so a rerun repeats the same work: only a bound sized for the dirty case helps.
The classes are therefore:

- Quick query step: bound 60 s, first notice after 5 s. Only steps that read refs, commits and trees, or write one
  ref or one tree from the index as it is: `rev-parse`, `rev-list`, `symbolic-ref`, `for-each-ref`, `ls-tree`,
  `cat-file`, `diff-tree`, `merge-base`, `write-tree`, `update-ref`, `config`, `remote`, `version`.
  Worst measured 44 ms, unaffected by the worktree. A `reference-transaction` hook cannot slow `update-ref` or
  `symbolic-ref` here, because named and traditional external hooks do not run (owner decision).
- Bulk step: bound 300 s, first notice after 10 s. Every other step: `pack-objects`, `index-pack`, `add`,
  `hash-object`, `maintenance`, and the index-versus-worktree steps `reset`, `read-tree`, `update-index`, `ls-files`,
  `diff`, `diff-index`, `show`, `ls-remote`, plus any subcommand not named. There is no second list: a step is a
  query only when it is named a query, so a new subcommand gets the longer bound instead of an early stop.
- The object reader (`cat-file --batch`) is a bulk step on purpose: one request can name a large blob, and its subject
  now carries the class like every other step (a single `cat-file` query through `run_git` is a quick query step).
- The user's `git commit` with its hooks is not bounded (its runtime is the user's), unchanged.
- `CLI_GIT_CHILD_WAIT_SECONDS` still overrides both classes with one value. No per-class variables were added.
- The first notice comes after the class grace, never later than the notice interval and never later than half the
  bound, so a bound set below the grace still prints a notice before the step is stopped.
- The notice and the timeout sentence name the command and its class, the bound, what to run next and how to raise it.
  The sentence is shared with the object reader, so it says "if the step belonged to a commit".

A step stopped at its bound is an ordinary failed Git step: `GitFailure` ends the transaction like a non-zero exit,
and the recovery tables (tearing sweep, kill, signal) accept every state a killed child leaves.

No retry of a timed-out query was added. The existing `start_retry` machinery retries a refused start (`EAGAIN`) with a
paused budget; it does not fit a child that ran and hung, and a second mechanism was ruled out.

## Other ways a commit can time out

- Lock waits (`live_lock.rs`, `CLI_GIT_LOCK_WAIT_SECONDS`, 1800 s): a wait behind another commit whose holder is alive.
  A whole commit now takes 1 to 3 s here against 5 to 12 s, so a queue clears sooner, but a holder can also sit in
  the user's hooks, whose runtime nobody here can bound, so the number is left unchanged. The notice names the holder
  and the place in line, and the command leaves the lock in place so it can simply be run again.
- `index_lock_wait.rs` shares that limit for a live `index.lock` holder, with the same reasoning.
- Signal guard (`commit_preparation.rs`, `GUARD_WAIT_LIMIT` 1200 tries of 50 ms): contention between threads of one
  process. A wrapper process runs one Git child at a time, so production never waits there; only library tests do.
  It is not a commit time-out path.
- Pre-commit stage queue (issue 1811 notices): bounded by the same lock limit; unchanged.
- The bound that was wrong is the child bound (30 minutes for everything), fixed above.

## Tests and guard evidence

- `child_class_tests.rs`: classes of every named subcommand behind global options, unknown steps get the longer bound,
  `maintenance` is object work, the variable sets every class, an unusable value falls back to the class default,
  and the relations between bounds and graces (object bound above plumbing bound, every grace below its bound).
- `child_watch_tests.rs`: the timeout sentence is derived from the class default and the variable name, and the terms
  of a class carry its grace (held to the notice interval) and its bound.
- `shadow_skeleton_tests.rs`: differential against real `git init --bare --template=`, acceptance by real Git,
  a second creation over a torn directory, a missing parent reported.
- `commit_index_stat_tests.rs`: an unchanged entry keeps its stat data, a plain `read-tree` leaves none (positive
  control), a named path is staged from the worktree.
- `binary_child_bound_tests.rs`: a real wrapped commit under `strace` with every Git start delayed past a one-second
  bound fails with the bound, the variable and the class named, and `verify_recovery` accepts the repository.
- Guard failures shown (break, run, restore): `class_of` without the plumbing branch fails the plumbing-class test;
  the skeleton without `refs/tags` fails the differential test; the private index without the copy fails the stat
  test; the terms with the grace not held to the class fail the terms test; the sentence without "run the command
  again" fails the sentence test; `maintenance` added to the plumbing list fails `maintenance_is_object_work` (and
  only that test); the variable ignored by `limit()` fails `child_bound` after 114.8 s (no step then reaches the
  60 s default, every start is only 3 s late, so about forty delayed starts outlast the control's own 120 s outer
  bound and the sentence is never printed).
- Passing: `budget` (4), `recovery_tearing` (25, shadow tearing included), `fault_kill` (8), `fault_signal` (4),
  `landing` (17), `landing_crash` (11), `wait_notice` (9), and the unit tests of `child_class`, `child_watch`,
  `shadow_skeleton`, `shadow_create`, `commit_index` and `transaction_git`. Clippy with warnings denied is clean.

## Review follow-up: tests added and guards shown failing

- `index_flags_tests.rs`: the reader against `git ls-files -v` for versions 2, 3 and 4, a SHA-256 repository, names
  of every length (every padding), a path longer than one directory name, intent-to-add entries (not a mark), a split
  index (unreadable), an empty index (plain) and truncated, wrong-version and non-index bytes (unreadable).
- `commit_index_stat_tests.rs`: a named assume-unchanged file is committed, an unnamed one stays as the base has it,
  a named directory takes its assume-unchanged files, a named skip-worktree file does not fail the commit, and only a
  plain index seeds the private index.
- `child_watch_tests.rs`: the reader subject carries its class, the terms of a class stay inside the interval and the
  bound, a bound below the grace pulls the first notice inside it.
- `transaction_git_tests.rs`: the production entry takes its terms from the request's own subcommand.
- `binary_child_bound_tests.rs`: the class label in the sentence is the one the production classification gives the
  command that was stopped; the notice interval is tied to the bound.
- Guard breaks, each shown to fail the named tests and restored: the reader answering "plain" for every index fails the
  five assume-unchanged, skip-worktree, SHA-256, split and damaged-index tests and the "only a plain index seeds" test;
  the terms of a request always taking the bulk class fails `the_terms_of_a_request_follow_its_subcommand`; the grace
  without the half-bound cap fails `a_short_bound_pulls_the_first_notice_inside_it`; `reset` added to the query list
  fails the two class tests. The unnamed assume-unchanged test passes under every break by design: it guards a
  regression in the other direction.
- Not testable cheaply: the copy taking its times from the open file (a rename between a copy and a stat has no seam);
  it is a read of the handle instead of the path, which the existing copy tests exercise.

## The shadow commit is still unbounded and silent: proposal

`run_native_commit` (`commit_preparation.rs`) runs the shadow `git commit` with the terminal attached through the
signal guard, so it is not watched and prints nothing, which is how a hash-everything refresh looked like a hang.
With the stat seeding it is fast unless the real index is itself stat-dirty (a fresh clone, a restored tree), when the
copy carries that state and the child hashes every file again.
It must stay interactive only for the message editor (no external hooks run). A minimal change, not made here because
the file belongs to the preparation owner and the guard has no way to report the child's process identity yet:
when the commit arguments carry a message source (`-m`, `--message`, `-F`, `--file`, `-C`, `--no-edit`), run the child
under a `ChildWatch` with notice-only terms (a limit no wait reaches) so the usual "still waiting for ... (process N)"
line appears at the bulk grace; when an editor will open, do nothing. That needs `SignalGuard::run_child` to hand
back the child's identifier before it waits.
