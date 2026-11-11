# Commit latency under disk pressure

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

The owner's rules,
verbatim:
"commits vs native git should not take 5s longer."
"You should (almost) never be waiting."
"Disk pressure being high is not an excuse."
And on how to fix it:
"remove it or move it off the waiting path",
and "sometimes the work itself can be optimized.
And we
control every part of the wrapper,
including everything it embeds."
Host pressure is the condition to test under,
never the cause to report;
the cause is work the wrapper does on the
commit's foreground path that native Git does not do,
or does much less of.

This note records what was measured on the 0.5.28 base (`e7e304ffd6`),
which defects were found,
what was changed
with a red control first,
and what is left to the owner.
Branch:
`fix/git-policy-cli-commits-fast-under-disk-pressure`.

## Result of the third round (less work on the wait)

Branch `perf/git-policy-cli-less-foreground-work`,
from `main` at `05c16d73d2`,
2026-11-04.
The round removes or cheapens work the person waits for during `git commit`;
every change has a control that was seen to
fail first (the red commit is named),
and every figure in `Result of the third round (less work on the wait)` was measured on this host
under the load it had (load
average
about 9 during the runs),
never after waiting for a quiet one.

Method:
the paired harness `bin/native-perf.ts` as in "How it was measured",
on a fresh main-shaped fixture (a clone
of this
repository's `main` made with the Git LFS filters switched off so the harness's own Git sees a clean index:
21,065
tracked
files,
a 2,837,531 byte index,
this checkout's hooks),
twenty rounds per run with the first (cold) round of each run
left
out,
runs of the base build and of the branch builds alternating,
and a base-against-base spread read from the base runs
among themselves.
The base is the release build of `main` at `05c16d73d2`.
Native Git on this fixture takes 53 ms
(unsigned)
and 57 ms (signed),
warm.

- Spread of the same binary on unchanged input:
  the base's per-run medians of the paired delta were 104 to 113 ms
  (unsigned,
  eight runs) and 138 to 155 ms (signed,
  eight runs).
  The first branch build,
  which differs from the base
  only by
  the sync change below,
  measured 99 to 119 ms and 138 to 150 ms in its four runs:
  inside that spread,
  so the sync
  change has
  no warm effect on an idle disk,
  and no figure under about 10 ms is claimed from these runs.
- Paired delta,
  warm,
  pooled over the runs of each arm (median of wrapped minus native of the same round):
  - base:
    unsigned 108 ms over 152 pairs,
    signed 143 ms over 152 pairs (last four runs alone:
    110 and 146 ms).
  - sync change only (`96a151a936`):
    unsigned 106 ms,
    signed 145 ms over 76 pairs.
  - plus object reader order and cache-tree comparison (`c31b5daa60`):
    unsigned 86 ms,
    signed 120 ms over 76 pairs.
  - plus built-commit checks from the object store (`5baabfe665`,
    the tip):
    unsigned 78 ms,
    signed 109 ms over 76 pairs.
  - So the wrapped commit takes 2.5 times the native time unsigned and 2.9 times signed on the tip,
    against 3.0 and
    3.5 on
    the base.
- Cold (the pack,
  index and packed-refs pages of the fixture dropped before every sample,
  both sides;
  two runs of each):
  base 144 ms unsigned and 146 ms signed over 38 pairs;
  tip 117 and 121 ms. Native takes 195 and 200 ms cold.
- Per-step costs (client timing records and the service's `answered` lines,
  warm medians,
  base then tip):
  - `prepare`:
    105 then 79 ms unsigned,
    100 then 77 ms signed.
  - `build`:
    2 then 2 ms unsigned,
    42 then 41 ms signed (the scratch repository's `commit`,
    not changed here).
  - `submit`:
    32 then 26 ms unsigned,
    35 then 23 ms signed.
  - `gate`:
    11 then 10 ms unsigned,
    11 then 11 ms signed.
  - Service `landing_ms`:
    32 to 34 then 25 ms;
    `objects_ms`:
    10 then 0 to 3 ms;
    `index_update_ms` (named `sync_ms`
    before
    this round):
    17 to 20 then 18 to 19 ms,
    unchanged.
- Largest pair of any run of the tip:
  940 ms (unsigned) and 773 ms (signed);
  of the base:
  1,859 ms and 1,076 ms. No pair
  reached the 5 s bound.

What was changed,
in the order it was done:

- Git's pack sync in `fast-import` (`96a151a936`,
  red `2d5052c2f2`):
  see "Git's pack sync in `fast-import`".
  Counted
  in the
  service's `strace` (execve,
  fsync,
  rename) over six unsigned landings each:
  the base's `fast-import` made 6 syncs,
  the
  tip's none;
  the wrapper's own queued syncs were 43 in both.
  The control lands in a repository and reads Git's trace2
  `fsync`
  counters of every child:
  1 flush before the change,
  0 after;
  a kept pack (`fastimport.unpackLimit=0`) 2 before,
  0
  after;
  with `core.fsync` naming `loose-object`,
  `added` or `all`,
  or `core.fsyncObjectFiles=true`,
  the flushes stay.
  `config_fsync.rs` reads `core.fsync` as Git 2.56.0 does and `config_fsync_tests.rs` checks that reading against the
  flushes real Git makes for 22 values (loose object,
  reference and pack,
  one probe each),
  and
  `hardens_index` now asks the same reading.
- The answered line's `sync_ms` is `index_update_ms` (`819299f202`).
  It times removing the conclusion state and
  building and
  installing the new index,
  not a disk sync.
  Readers found by search:
  only this crate's controls and the documents
  (`README.md`,
  `SPEC.md`),
  and the notes in `doc/` that quote old records;
  nothing else parses the journal or the
  timing logs
  (`bin/native-perf*.ts` store the lines unparsed).
  Records already in the journal keep `sync_ms`;
  `index_update_ms_of` reads
  either name,
  and `a_record_from_before_the_rename_still_gives_its_index_update` pins it.
- The object reader searches packs before loose files (`d8b7f47ba9`,
  red `66819bb167`).
  A commit read about two thousand
  objects of its base tree and looked for a loose file of each first (1,985 failed `statx` calls in one traced commit);
  the control counts loose looks for 43 packed objects:
  43 before,
  0 after,
  and a loose-only object is still found.
- The cached diffs leave the directories the cache-tree proves equal unread (`c31b5daa60`):
  see "Less work in the
  client and
  the landing".
  Controls:
  the answers equal Git in every state of the cache-tree (valid everywhere,
  invalid along one
  path,
  root invalid with valid subtrees,
  directories added and removed,
  a mode change,
  a tree written for the staged state,
  index
  versions 3 and 4,
  SHA-1 and SHA-256,
  no cache-tree,
  a tree the cache-tree does not name);
  a path that became a
  directory,
  an
  intent-to-add entry,
  an unmerged path and a sparse index stay with Git or equal it;
  a node whose count does not
  match the
  index declines.
  The read-count control (40 directories,
  one staged file) decodes fewer entries than there are
  directories,
  and with the comparison switched off it fails.
- A commit Git built is checked from the object store (`90c649249b`):
  see "Less work in the client and the landing".

The pack sync,
the object order and the cache-tree comparison are reads and flushes that a stalled disk lengthens;
the
landing's `objects_ms` and the client's `prepare` are the steps that held them.
Nothing in this round makes the 5 s
bound
provable under a stall that cannot be produced here;
see "Evidence for raising the 5 s standard".

What is left on the client's wait,
measured on the tip (unsigned,
from the `execve` trace of one commit):
the Git `add`
(24 ms),
`ls-tree -r` of the base commit for the policies' baseline listing (20 ms;
`candidate/candidate_store.rs`,
a
policy-engine question),
`rev-parse` of the index path (8 ms),
`cat-file --batch` (6 ms) and `diff-index` (6 ms),
and
in the
service the in-process build of the new index (about 15 ms of the 18 ms `index_update_ms`,
a rewrite of the whole 2.8 MB
file).

## Result of the second round

The branch now holds the 0.5.28 base tip `f89fa63f10` (every wait has a bound),
the index recovery round 5 branch
`b47467ebd1` (a witness of the index file,
flush at release),
and the owner's rule for the commit protocol,
2026-11-03:
"Force at leaving,
else background" ("No forced write while you wait.
Here:
that attempt is lost on the power cut,
exactly as
with native Git;
your staged changes and files are intact,
so you run the commit again.
Anything the command already
promised
when it left (an error with a kept side ref,
a hand-over to the service) was forced to disk first and survives.").

- The person waits for no sync of the wrapper's own records in a landing that publishes (a signed commit,
  which Git
  builds,
  or
  an unsigned one):
  a trace of five signed landings shows 36 syncs,
  all made by the background thread,
  none by the
  landing
  thread and none by a Git child;
  five unsigned landings show 41,
  of which the 5 by Git children are `fast-import`'s
  pack sync
  (Git's own,
  see "What Git does by default").
  A landing that publishes nothing syncs what it wrote before it returns.
- Waited syncs per landing,
  by build and path:
  base 12 (11 of the wrapper,
  1 of Git's pack);
  0.5.27 about 4;
  the
  first-round
  tip 11;
  the merge of the two branches before this rule 8 of the wrapper (+1 of Git on the unsigned route);
  this
  tip 0 of the
  wrapper on the publishing paths (+1 of Git on the unsigned route only),
  7 queued;
  with `core.fsync=added` in the
  repository 1
  (the index,
  as native Git flushes it);
  a withdrawal at the lock ceiling 3 (the intent file,
  its directory,
  the
  lock marker),
  forced.
- Paired deltas,
  nineteen warm rounds,
  two alternations,
  median of the wrapped minus the native time of the same
  round:
  signed,
  tip 89 and 87 ms (native 118 ms,
  wrapped 207 and 205 ms,
  so 1.75 times native);
  the first-round tip 127
  and 133 ms;
  0.5.27 179 and 175 ms;
  base 326 and 336 ms. Unsigned,
  tip 54 and 50 ms (native 115 and 113 ms,
  wrapped 171 and 162
  ms,
  1.45
  times native);
  first-round tip 111 and 108 ms;
  0.5.27 138 and 137 ms;
  base 258 and 251 ms.
- The service's landing,
  median:
  tip 35 and 36 ms (signed),
  34 and 32 ms (unsigned),
  `journal_ms` 0,
  `sync_ms` 19 to
  20 ms;
  0.5.27 55 and 55 ms (signed),
  `sync_ms` 25 ms.
- A pair above one second still occurs on every build when the host stalls (tip:
  1,525 and 1,888 ms signed,
  1,536
  and 1,640 ms
  unsigned;
  base:
  6,219 ms signed and 3,403 ms unsigned).
  In the tip's runs the landing was not what stalled:
  the
  phases of the
  slow pairs show the client's `prepare` (1,517 ms) or `build` (914 ms),
  with the landing at 37 to 60 ms. A 1.5 s
  pause after
  each sample did not remove them (zp1:
  1,714 and 1,681 ms of 12 pairs).
  The largest pair of the base in these runs,
  6,219 ms,
  is over the owner's 5 s bound;
  no pair of the tip is.

## The merges of the second round

- `f89fa63f10` (every wait has a bound) merged without a conflict.
  Its `Clock::check` holds both this branch's first
  look
  (the child alone,
  then the whole tree) and its connected-silence window;
  `push_progress_tests.rs` holds both sets
  of controls.
- `b47467ebd1` (index recovery round 5) conflicted in four files,
  each resolved keeping both intents:
  - `service_land_copy.rs`:
    round 5's design replaces this branch's `PrivateCopy::make_durable`;
    the copy has no
    flush of its
    own,
    and the release flushes a changed copy that the update did not already flush (and installs an unchanged one
    not at
    all).
    Removed with it:
    this branch's flush after Git's `reset` in `build_index` and in `reset_file` for an empty
    list.
  - `service_journal.rs`:
    the start record is round 5's witness,
    written with no sync (the `sync-start-*` names are
    gone).
  - `service_land.rs`:
    round 5's imports,
    with `acquire_noting_wait_within` of the bound branch.
  - `service_index_lock_tests.rs`:
    `a_live_service_holders_lock_is_never_taken_over` keeps round 5's version,
    which
    is the
    stronger of the two:
    the waiting thread lets go of the lock (as this branch's fix did) and the main thread asks
    again
    until the waiter has let go,
    within 30 s,
    so it also holds when the first ask fails.
  - Two controls changed with the merge:
    `a_read_step_ended_at_its_ceiling...` switches the in-process readers off
    so that its
    hanging `rev-parse` is still started (the landing now reads the branch in process),
    and the list of syncs of a
    landing
    lost the start record's and the copy's.
- Round 5's note that the `.git` directory sync averaged 94 ms against 6.5 ms on the base:
  two traces of five signed
  landings
  here show the directory sync at a mean of 62 ms (max 279 ms) on the base and 109 ms (max 456 ms) on the
  first-round tip
  where the copy's own flush was gone,
  and the flushes of the copy and the directory together rose from about 84 to
  122 ms,
  which is consistent with the dropped flush moving its cost into the directory sync (five landings each;
  not a
  proof).
  The syncs are off the wait now,
  so what a directory sync costs only matters to the next commit that shares the
  file system.

## Result of the first round

The 0.5.28 base is worse than the installed 0.5.27,
in two ways.
Warm,
its commits take longer (the numbers under `Warm numbers first`).
And under a stalled disk it is more exposed:
it waits for twelve syncs per landing where 0.5.27 waits for about four
and
native Git for none (60 against 21 over five traced landings).
The extra syncs come with what 0.5.28 adds on the landing path (the in-process index writer with its directory
syncs,
the
start record and the sync before the completion record);
only the totals were traced for 0.5.27,
so which of the
twelve
are new was not itemized.
This branch removes one of those syncs and the other work below;
it does not remove the rest.
The tip still waits for eleven syncs per landing (56 over five),
which is 2.75 times the exposure of 0.5.27 under a
stall,
and its landing is still longer than 0.5.27's in `sync_ms` (47 against 25 ms,
signed).
Removing those syncs from the waiting path is the owner's decision (options B to E,
below),
so the base is a 0.5.28
blocker for stall exposure until the owner decides.
Warm,
the tip is faster than both.

Warm numbers first.
Two builds,
alternating arms,
nineteen warm paired rounds each (median of the wrapped time minus the native time of
the same round),
on a main-shaped fixture (20,740 tracked files,
2.8 MB index,
the main checkout's pack and hooks):

- Unsigned explicit-path commit:
  base 273 and 249 ms,
  0.5.27 139 and 138 ms (native 113 to 130 ms).
- SSH-signed explicit-path commit,
  which is what the person's commits are:
  base 298 and 292 ms,
  0.5.27 172 and 174
  ms (native 115 to 118 ms).
- The service's own account of the landing:
  median `landing_ms` 138 and 130 against 52 and 52,
  `sync_ms` 56 against
  24,
  and the client's `build` phase 22 against 2 ms (unsigned) and 90 against 41 ms (signed).
- Counts that do not depend on the disk (one traced unsigned commit):
  the base starts 12 fsyncs per landing against
  about 4 for 0.5.27 (60 against 21 over five landings),
  and its client makes 136,191 system calls against 90,708,
  with 5,890 file opens against 486.

The branch tip is faster than both,
with the same measurement (signed,
nineteen warm rounds,
two alternations):

- Signed:
  tip 123 and 122 ms of paired delta,
  0.5.27 172 and 174 ms,
  base 298 and 292 ms;
  native 115 ms,
  so the
  wrapped commit takes 2.07 times the native one (the base took 3.5 times).
- Unsigned (build at `85e7ac3a86`):
  tip 116 and 114 ms,
  0.5.27 140 and 139 ms,
  base 273 and 249 ms;
  native 112 ms.
- One traced unsigned commit,
  client side:
  6,884 `pread64` calls against 79,951,
  14,474 system calls against
  136,191,
  nine Git starts against ten (the wrapper's own start excluded).

No pair of any run reached the 5 s bound:
the largest was 3,198 ms (the base,
the second round of a fresh fixture,
unsigned),
and the largest of the tip was 1,675 ms (the first round of a fresh fixture,
signed),
then 989 ms
(unsigned).
Every delta of every pair is in "Every pair" below.

Whether the 5 s bound can hold when the disk stalls cannot be shown by these measurements,
because the stall cannot
be produced without synthetic load,
which is banned.
See "Evidence for raising the 5 s standard" for what the production record shows and what is left.

## How it was measured

The paired harness `bin/native-perf.ts` was extended,
not replaced:

- `--shape <repository>` makes every fixture a reflink copy of a prepared repository.
  The prepared one here is a clone of the main checkout's `HEAD` with `git clone --no-hardlinks --single-branch`,
  its remote removed (the harness refuses a shape that has a remote),
  living under `${HOME}/temp/agent`.
  The copy gets `core.untrackedcache`,
  a refreshed index and the main checkout's `post-checkout`,
  `post-commit`,
  `post-merge` and `pre-push` hooks (`--hooks-from`),
  with `git-lfs` on the PATH (`--tool`).
  Every commit edits the tracked `README.md`.
- Isolation,
  as the harness already had it:
  the run's own scratch root,
  its own `HOME` and `XDG_RUNTIME_DIR`,
  one
  commit service per wrapped fixture started by the harness,
  the person's session bus and unit directory never read
  or written,
  and no remote on any fixture,
  so no push can leave the machine.
- Arms alternate within a round,
  direct first in even rounds and wrapped first in odd ones,
  and the arms use two
  copies of the same shape.
  Every measured command ran in one transient scope with `CPUQuota=400%`,
  `MemoryMax=8G`,
  `TasksMax=300` to `400`
  and `IOWeight=10` (never above 10),
  the same for both arms of a pair;
  a low weight skews both arms alike.
- The host was not quiet:
  load average 6.9 to 66 during the runs,
  input and output pressure `some avg300` 2 to 24
  percent.
  Nothing waited for a quiet host.
  Spread of the same binary on unchanged input:
  the native arm stayed within about 10 ms of its median over nineteen
  rounds (for example 111 to 123 ms warm,
  274 to 295 ms cold).
- `--cold` drops the pages of the fixture's pack files,
  pack indexes,
  index and packed refs (`dd iflag=nocache
  count=0`,
  `posix_fadvise` `DONTNEED`,
  verified to take a 586 MB pack from fully resident to zero resident) before
  every sample on both sides.
  It is cache state of the fixture's own files and no load on the host.
- `--output` stores the wrapped commits' timing records and the service's `answered` lines (`evidence`),
  which split
  a slow sample into its phases;
  `--trace-directory` leaves an `strace` record of every sample and of each service
  for counting syncs,
  bytes and starts (the summary script lives in scratch,
  not in the repository).
- A fixture finding that matters for every later measurement:
  a reflink copy has new inodes and change times,
  so
  every index entry reads as changed until Git refreshes the index,
  and a signed commit on such a copy spent about
  four seconds in the scratch repository's `git commit` re-reading all 20,740 files (22,813 file opens in one traced
  commit).
  The person's index has been refreshed by their own `git status`;
  the fixtures now refresh theirs outside any
  timing.
  The first rounds measured before this change (the 0.5.27 against base comparison at load 40 to 66,
  with deltas of
  518 and 520 ms for the base) are marked as such below.
- Limits:
  the fixtures live on `/dev/mapper/crypt_sda` (btrfs,
  zstd) because scratch belongs under
  `${HOME}/temp/agent`,
  while the main checkout's `.git` is on `/dev/dm-0`;
  absolute seconds under pressure differ
  per device,
  counts do not.
  The first round of every run is a cold outlier on both arms and is reported apart from the warm rounds.

## What the production record says

The service's `answered` lines in the user journal,
2026-11-02T09:01:38Z to 2026-11-03T15:11:58Z,
1,621 landings:

- `landing_ms` median 67,
  90th percentile 440,
  99th percentile 16,323,
  maximum 119,836;
  87 landings of one second or
  more and 29 of five seconds or more.
- The steps with a wait for the disk dominate the slow landings:
  of 1,346,019 ms spent in the landings of one second
  or more,
  1,053,826 ms (78 percent) were `sync_ms` (the index follow-up that holds the lock) and 241,902 ms (18
  percent) were `journal_ms` (the intent write);
  `objects_ms` was 3,882 ms (0.3 percent).
  `sync_ms` reached 5 s or more in 14 landings (maximum 119,715 ms),
  `journal_ms` in 11 (maximum 40,188 ms).
- The 35.7 s commit of the brief (request `8b549c42-2df6-4afd-be20-85b8e296b98b`) had `submitMs` 8,227,
  of which the
  service's `journal_ms` was 8,003:
  one intent write,
  a file sync,
  a rename and a directory sync,
  waited for for
  eight seconds.
  Its 25.5 s of `build` and the 15.1 s and 8.3 s of `prepare` in the other two records have no service line;
  the
  client made no sync call in `prepare` (a traced commit shows none),
  and the sampler saw `read_bytes` of 10 MB and
  five major faults in the 16.6 s one.

Reading of this evidence:
one mechanism,
two instances.
Work that native Git does not do waits on the disk in the foreground:
the service waits for a chain of syncs,
and
the client reads about thirty times the bytes native Git reads,
so a host that has evicted its pages makes the
client wait on reads.
The three blow-ups hit three different phases because each phase holds some of that work.
This is a reading of timing records and a sampler,
not a reproduction:
the stall itself cannot be produced here.

## Defects, ranked by what was measured

Counts are of one traced unsigned wrapped commit unless said otherwise;
native Git for the same commit makes 39,958
system calls,
8 `pread64`,
no sync call and 16 process starts (including the `post-commit` hook and `git-lfs`).

1.  The service waits for twelve syncs per landing on the base (about four on 0.5.27,
    none for native Git):
    the
    intent file and its directory,
    the lock marker,
    the start record and its directory,
    the private copy of the
    index,
    the replacement file of the index update,
    the Git directory after the index rename,
    the completion record
    and its directory,
    the journal directory after the batch is marked done,
    and Git's own pack sync in
    `fast-import`.
    Each is a flush that a loaded disk can stall for seconds.
    Idle,
    they cost 21 ms each on average in the traced run (1.26 s over sixty syncs).
2.  The client reads 46.9 MB and makes 79,951 `pread64` calls where native Git reads 1.5 MB and makes 8:
    the
    in-process object reader (`repo_read_pack.rs`,
    embedded code) searched a pack index one name at a time with a
    system call per step,
    opened every pack with 256 reads of four bytes,
    and walked the whole tree of a commit
    (about 1,400 trees) for a question about one changed file.
    This is what a page cache under pressure turns into disk waits.
3.  A scan of every process the host runs,
    started by the watch of each Git child the wrapper starts,
    on its first
    look (`push_progress.rs`,
    `Clock::check` with `next_sample: now`).
    The scan reads a status file per process (841 files,
    7 ms on this host) and the wrapper starts fifteen to twenty
    Git children for one commit,
    most of them gone within ten milliseconds.
    It grows with the number of processes the host runs,
    which is large exactly when the host is under pressure.
    Removing it took the unsigned `build` phase from 22 to 2 ms,
    `ref_ms` from 10 to 2 ms,
    and with the change below
    the paired delta from 155 to 115 ms.
4.  The gate after the landing listed the whole tree of the base commit with `ls-tree -r` again and walked both full
    trees for the landed commit's changes:
    69 ms of the signed commit's gate,
    10 ms after.
5.  The service started a `rev-parse` of the branch for every landing in a spelling the in-process readers do not
    answer (one Git start of about 10 ms warm,
    a cold start of the Git executable when the host has evicted it).
6.  The client still starts nine Git processes (unsigned route) and the service three or four per landing;
    `add`,
    `cat-file`,
    `ls-tree`,
    `diff-index`,
    `rev-list` and `var` (twice) in the client,
    and for a signed commit the
    scratch repository's `commit`,
    `update-ref` and `symbolic-ref` remain (see the options).
7.  The first landing of a fresh fixture is slow in some runs,
    on every build:
    `sync_ms` of the first landing was 98
    to 361 ms on
    the base (five signed runs) and 117 to 1,537 ms on the tip (eight signed runs,
    three of them above one second),
    23 to 48 ms
    on 0.5.27 and the tip's unsigned runs,
    and one base run had a second-round sample at 3.2 s.
    A trace of the tip's first landing (`NATIVE_PERF_TRACE_CALLS=execve,fsync,rename`) shows the in-process update,
    no Git
    `reset`,
    and two syncs that took 178 ms (the replacement file of the index) and 298 ms (the Git directory after the
    index rename),
    where the same syncs take 6 ms later in the run.
    The fixture's own setup has just written a lot (the reflink copies,
    the refresh of four indexes),
    and a sync
    waits for
    the writeback that is pending on the file system:
    the production stall in miniature,
    on a quiet host.
    Whether the tip's tail is longer than the base's is not established;
    the base pays the same backlog in the sync
    it makes
    on the private copy,
    which the tip no longer makes.

## What was changed

Each change has a control that was seen to fail first,
then pass;
the commit of the red control is named.

- A pack lookup reads the names of one first byte at once and keeps them (`repo_read_pack.rs`).
  Red `a_lookup_reads_the_index_a_few_times` (`7a6fb07ca0`),
  green `f96769481f` and `62d7385748`;
  the control counts
  reads of the index of a real pack of 400 files:
  at most two per found name,
  one read per bucket in all.
  Embedded code:
  this is the object reader the crate links and owns.
- Opening a pack reads the header and the table of counts in one read.
  Red `opening_a_pack_reads_its_index_a_few_times` (`a6c0583525`,
  257 reads),
  green `beb2aa8232`.
  Embedded code.
- The in-process `diff-tree` of one commit reads only the subtrees that differ from its parent
  (`repo_read_query.rs`).
  Red `the_changes_of_a_commit_are_found_without_reading_the_unchanged_directories` (`ce27cde2ce`,
  86 entries
  decoded for a repository of 40 directories),
  green `49b5e1a0c2`;
  the answer still equals Git's in the differential
  controls.
  Embedded code.
- The files of a commit are listed by Git once per process (`candidate_store.rs`).
  Red `a_commit_listed_by_one_store_is_not_listed_again_by_another` (`39ff2056af`),
  green `07ddc2c336`.
- A landing no longer flushes the private copy of the index when the in-process update replaces it with a file that
  is flushed itself before its rename (`service_land_copy.rs`,
  `service_land_follow.rs`,
  `index_reset_file.rs`).
  Red `a_landing_waits_for_no_sync_that_protects_nothing` (`de5c3a9b99`),
  which lists the syncs of one landing by
  name;
  green `c5a7953c70`.
  A copy installed as it stands (no paths,
  or changed in place by Git) is still flushed once,
  so no protection was
  dropped.
  One of the twelve syncs and a forced 2.8 MB write are gone;
  eleven remain (ten when Git built the commit).
- A landing asks for the branch commit as `<ref>^{commit}`,
  the spelling the in-process readers answer
  (`service_land.rs`).
  Red `a_landing_reads_the_branch_it_moves_without_starting_git` (`ef21bfd223`),
  green `769975ef5e`.
- The watch of a started child looks at the child alone on its first look;
  the machine's processes are listed from
  the next sample on (`push_progress.rs`).
  Red `a_child_that_has_just_started_is_looked_at_without_listing_the_machine` (`921e199a5a`),
  green `85e7ac3a86`.
  What it changes in behavior:
  a child silent from its start shows no progress for one more sample interval before a
  stall window can be refreshed by a first sight;
  the stall window itself is unchanged.
- The harness (`cd592cf885`,
  `aa1f633677`,
  `ed2f907a3e`):
  the options under `How it was measured`,
  the evidence,
  the refresh of the
  fixtures.

## Left to the owner

The owner has decided the sync question for the commit protocol (see "Result of the second round"):
the options A to
E of the
first round are replaced by that rule.
What it leaves open is below.

### What Git does by default, and what this follows

Read for Git 2.56.0,
the installed manual page and the source at tag `v2.56.0` (`environment.c`
`parse_fsync_components`,
`write-or-die.h`):

- `core.fsyncMethod` defaults to `fsync` (`writeout-only` only on macOS).
- `core.fsync` defaults to `FSYNC_COMPONENTS_DEFAULT`,
  which is `(objects | derived-metadata) & ~loose-object`:
  `pack`,
  `pack-metadata` and `commit-graph`.
  Loose objects,
  references and the index are not flushed by default.
  (The
  manual page
  says "equivalent to `core.fsync=committed,-loose-object`" and "`committed` is currently equivalent to `objects`";
  the
  source defines `committed` as `objects | reference`,
  which the page does not say.
  Only the index matters here,
  and
  `index`,
  `added` and `all` are the only components that include it.)
- A `core.fsync` value starts again from that default at each entry,
  so the last entry decides alone;
  `none` clears
  the set
  only when it is the whole rest of the value;
  a word names every component it is a prefix of;
  the set is the default
  without the words with a `-`,
  plus the words without one.
- The service follows this for the index it writes in process (`ConfigSnapshot::hardens_index`):
  no flush by
  default,
  and
  the flush before the rename on the person's wait when the repository's `core.fsync` names the index.
  Git's own writes (`fast-import`'s pack,
  `update-ref`'s reference,
  the scratch repository's loose objects) follow the
  configuration because the service passes none of its own;
  the pack sync of `fast-import` is therefore still on the
  wait of an
  unsigned commit,
  6 to 8 ms idle.

### Which recovery cases now match native Git on a power cut

The rule changes nothing for a killed process:
the page cache keeps every write,
so the journal's recovery runs as
before.
On
a power cut,
a landing's records may be lost in any suffix of their order,
as native Git loses its last writes:

- The intent may be lost while the branch moved:
  the restart has nothing to finish,
  and the index may be behind the
  branch;
  the person's next command sees a branch that has the commit and an index that does not stage it (as after a native
  commit
  whose index write was lost),
  and nothing of their files is touched.
- The lock marker may be lost:
  a stale `index.lock` without the service's marker is a foreign lock to the next
  landing,
  which
  waits at the ceiling and withdraws with the commit kept on a side ref;
  the person removes the lock as after a
  power cut in
  native Git.
- The completion record may be lost while the index is installed:
  the restart finds the witness of the index file
  different
  (round 5),
  resets nothing and logs a notice;
  no staged path is overwritten.
- The done mark may be lost:
  the restart finishes the batch again,
  idempotently.
- The index file may be short or empty (its flush is native's,
  none by default):
  the same as for native Git.
- Withdrawals and errors at a bound keep their promise:
  the records they wrote,
  and the kept side ref's journal
  entry,
  are
  synced before the command returns (`a_withdrawal_forces_the_records_it_wrote`).
- A power cut during the sync of a landing that was already answered loses that landing's commit as native Git would
  lose it;
  the person runs the commit again.

### The queue and the next commit

The syncs of a landing run after its answer,
on one thread,
all handles of a landing at once.
A commit started while
they run
shares the file system with them;
on a loaded disk it may wait behind them (a sync that commits the file system's
log).
The
second round of every fresh run was the slowest pair in several runs of the tip (1,007 and 1,525 ms signed),
but a
1.5 s pause
between samples did not remove it,
and the slow phase was the client's `prepare` or `build`,
so the link is not
shown.
If it
is shown,
the next step is a cap on queued landings (the service holds a landing's answer until the queue is under
it),
which
is the owner's to weigh against the 5 s rule.

Measured 2026-11-04:
the tip,
with no pause between samples against a pause of 1.5 s (`--pause 1500`),
runs alternating
in time (three of each,
twenty rounds each,
the first round of each left out;
57 pairs per arm and scenario).
Unsigned:
76 ms without the pause and 78 ms with it;
signed:
106 and 107 ms. The service's per-phase medians are the same with and
without the pause (`queued_ms` 0,
`lock_wait_ms` 0,
`journal_ms` 0).
The tails flip:
pairs more than 40 ms above the
arm's own
median were 2 and 2 unsigned (without the pause,
with it) and 0 and 7 signed (five of those above 200 ms,
the largest
773
ms).
An earlier pair of runs that was not alternated in time had shown the opposite (0 of 38 pairs with the pause,
9
of 76
unsigned and 6 of 76 signed without it);
alternating in time removed it.
A commit
that starts while the previous landing's syncs run is therefore not shown to wait behind them.
No owner question is
asked and
the queue cap is not built.

### Git's pack sync in `fast-import`

What the code does now,
read at `service_land.rs` (`land_batch_body`) and traced on Git 2.56.0 in disposable
repositories (2026-11-04):

- The service runs `git fast-import --done --export-marks=<file>` inside the real repository,
  so the objects are written
  to the real object store,
  and the stream names `refs/worktree/cli-git-service/<seq>` as the branch it builds.
  Nothing in the landing code makes a `.keep` file;
  a search of `src/` finds none.
- `fast-import` writes a temporary pack,
  then (`builtin/fast-import.c`,
  `end_packfile`) either loosens it or keeps it.
  Up to `fastimport.unpackLimit` objects (default 100;
  `transfer.unpackLimit` is the fallback) it pipes the pack to
  `git unpack-objects` and deletes the pack.
  A landing of one changed file writes three objects (the commit and the trees on the changed path;
  the blobs were
  written by the client),
  so the common landing takes this branch.
  Trace of a three-object stream:
  `fsync(pack)`,
  one `unpack-objects` start,
  loose objects left,
  no pack file,
  no
  `.keep`.
- The sync that the plan calls "Git's pack sync" is therefore the sync of a temporary file that is deleted a moment
  later
  (`pack-write.c`,
  `fixup_pack_header_footer`,
  `FSYNC_COMPONENT_PACK`).
  It protects nothing,
  and the objects that remain are written by `unpack-objects` under the `loose-object` component,
  which Git's default `core.fsync` does not sync.
- Above the limit it keeps the pack:
  trace of a 150-file stream:
  two syncs (the pack and the index),
  a `.keep` file made
  by `fast-import` itself and unlinked when it exits,
  one pack left.
  So the `.keep` of the decision record lives only as long as the `fast-import` process.
- What protects the new objects between the import and the compare-and-swap is the scratch reference that the same
  `fast-import` run creates,
  not a `.keep` file.
  Checked on a disposable repository with a linked worktree:
  after `fast-import` built
  `refs/worktree/cli-git-service/1` in the main worktree,
  `git prune --expire=now` and `git gc --prune=now` run from the
  linked worktree kept the commit and all its objects.
- Record drift to report:
  `doc/decision/cli-git-concurrent-commits.md` lines 512 to 514 (a pack "kept with a `.keep`
  file until the compare-and-swap succeeds") and lines 118 to 121 ("every landing adds one pack to the real object
  store") describe a design this code does not have;
  small landings leave loose objects and no pack,
  and the guarantee
  is the scratch reference.
  The decision record is not edited here.
- The person's configuration in the main checkout sets none of `core.fsync`,
  `core.fsyncObjectFiles`,
  `core.fsyncMethod`,
  `fastimport.unpackLimit`,
  `transfer.unpackLimit`,
  so Git's defaults apply.

Options:

- Option A,
  keep the sync.
  Pro:
  nothing to change.
  Con:
  one flush of a file that is thrown away,
  6 to 8 ms idle,
  unbounded behind a stalled disk,
  on the person's wait
  of every unsigned commit,
  where native Git writes loose objects with no flush at all by default.
  Visible result:
  unchanged,
  one `fsync` in the trace of a landing.
- Option B,
  write the loose objects in process,
  no sync.
  Pro:
  no `fast-import` start and no `unpack-objects` start (two Git children of the three or four per landing),
  no
  sync.
  Con:
  new code that writes into the object store (tree edits against the base tree,
  deflate,
  the repository's hash
  algorithm,
  the rename into place),
  and the scratch reference must then be made some other way (a Git start,
  which
  gives back one of the two starts,
  or an in-process reference write,
  which is the "scratch refs in process" item and
  does not cover reftable repositories).
  Without a scratch reference the new objects are unreachable until the branch moves,
  which is native Git's own window
  but is weaker than what the landing has today.
  Visible result:
  a landing with no import child.
- Option C,
  keep `fast-import` and its scratch reference,
  and give that one child the `core.fsync` that native Git would
  apply to the loose objects it would have written.
  When the repository's effective set (or `core.fsyncObjectFiles=true`) names `loose-object`,
  the child is left as it
  is;
  otherwise it gets the person's set without `pack` and `pack-metadata`.
  Pro:
  removes the flush;
  keeps the protection;
  the person's own durability choice still decides.
  Con:
  keeps the two Git starts;
  one more configuration lookup (already in the landing's snapshot).
  Visible result:
  default configuration,
  zero `fsync` calls in the trace of `fast-import`,
  the same loose objects;
  `core.fsync=loose-object` or `added`,
  one sync as before.
- Option D,
  raise or lower `fastimport.unpackLimit`.
  Not an option:
  it moves the sync between the temporary pack and a kept pack,
  and does not remove it.
- Ranking:
  C > B > A.
  C over B:
  C removes the flush,
  which is the part of the work that a stalled disk makes unbounded,
  keeps the recorded protection (the scratch reference) with no new code that writes objects,
  and is small enough to
  prove with a control that goes red;
  B removes two more starts but needs the in-process reference write to keep the
  protection,
  so it is the follow-up that builds on the "scratch refs in process" item.
  B over A:
  B removes the same flush and two starts,
  and costs only the new writer.
- Pick:
  C,
  implemented in this round (agent's pick,
  veto open):
  `96a151a936`,
  red control `2d5052c2f2`.
  B is not built here:
  it needs the in-process reference write first.
- Result:
  the service's `strace` over six unsigned landings counts 6 syncs by `fast-import` on the base and none on the
  branch;
  the control reads Git's trace2 `fsync` counters of the landing's children (1 before,
  0 after;
  2 before and 0
  after for a kept pack);
  a repository that names `loose-object`,
  `added` or `all`,
  or sets
  `core.fsyncObjectFiles=true`,
  keeps its flushes.
  On an idle disk the warm paired delta did not move beyond the
  spread of
  the same binary (see "Result of the third round");
  the claim is the flush that a stalled disk can no longer hold on
  the
  person's wait,
  not a warm millisecond.
- Record drift for the coordinator (not edited here,
  DRR):
  `doc/decision/cli-git-concurrent-commits.md` lines 118 to 121
  and 512 to 514 describe a pack kept with a `.keep` file and one pack added by every landing.
  The landing code makes no
  `.keep` file;
  `fast-import` makes and removes its own,
  and a small batch leaves loose objects and no pack.
  The
  guarantee that holds is the scratch reference.
  The header of `maintenance_run.rs` was the same drift and is corrected.

### The service unit and the host's swap

The unit (`service_units/cli-git@.service`) sets no memory directive;
its comment records that a memory cap made Git
re-read pages.
`MemorySwapMax=0` (systemd.resource-control:
it limits the swap the unit's processes may use;
checked in the local
manual page) would keep the service's own pages and its Git children's out of swap.
Pro:
the service's 22 MB (186 major faults in the sampler of the 16.6 s commit) stays resident.
Con:
it covers the Git children too,
and was never measured to matter here;
swap use on the host was 7.7 GB of 80
GB.
A softer `MemoryLow=` protects against reclaim without forbidding swap and is not measured either.
Ranking:
not before a measurement shows service pages being swapped during a slow commit.

Measured 2026-11-04 (nothing set):
a sampler read `VmSwap` and the major-fault counter of every `cli-git-service`
process of the
host every 100 ms for 25 minutes (13,730 samples of each of 7 to 8 services,
started 03:53:39 -0500).
`VmSwap` was 0
kB in
every sample of every service (the idle pass before it also read 0 for all seven);
the resident size peaked at 12.8 to
29.8 MB per service;
the major faults of the window were 0 for every service.
The host held 6.7 GB in its compressed
swap
(zram) meanwhile,
so swap was in use by other processes and not by these.
The journal holds 21 landings in the window,
slowest 133 ms,
so no slow commit was among them:
this shows the services are not swapped out in ordinary use,
and it
cannot show what happens during a stall that cannot be produced here.
The 16.6 s commit's 186 major faults remain the
only
record of a swapped-in service,
from a sampler in an earlier round.
A directive would cover the Git children too and
has no
evidence to rest on,
so none is set.

### Less work in the client and the landing

- Compare the index with `HEAD` through the index's cache-tree extension (`index_cache_tree.rs`):
  the in-process
  `diff-index --cached` still reads about 1,400 trees of the base commit once per process,
  where a valid cache-tree
  names the unchanged directories without reading them.
  Pro:
  the largest remaining client read.
  Con:
  sparse indexes,
  intent-to-add and unmerged entries need their own cases.
- Replace `rev-list --parents --max-count=1` and `rev-list --objects <built> --not <old>` (the landing's checks of a
  commit Git built) with in-process reads of the commit and of the trees along its changed paths;
  Pro:
  two Git
  starts of the three or four per landing.
  Con:
  the second is the connectivity check that guards a pruned object,
  so the control must remove an object
  between the two reads.
- Write the scratch repository's `HEAD` and branch file in process instead of `update-ref` and `symbolic-ref`,
  and
  list the refs with the in-process reader instead of `for-each-ref`:
  Pro:
  three Git starts of the signed route.
  Con:
  reftable repositories stay on Git.
- Compute the author and committer lines in process instead of two `git var` starts:
  Pro:
  two starts.
  Con:
  Git's rules for a missing name or address,
  the `GIT_*` variables and `user.useConfigOnly` must be reproduced
  exactly.
- Ranking:
  cache-tree > in-process `rev-list` checks > scratch refs > `git var`,
  by the reads and starts each
  removes.

Status after the third round (2026-11-04):

- Cache-tree comparison:
  done (`c31b5daa60`),
  for `diff-index --cached` and its `--quiet` form
  (`repo_read_tree_cached.rs`,
  `repo_read_index.rs` keeps the `TREE` extension).
  A directory the cache-tree marks valid
  whose name equals the tree's is not read,
  on both sides;
  a count that does not match the index,
  a cache-tree that is
  absent,
  an unmerged entry,
  an intent-to-add entry and anything the reader already declined (sparse and split
  indexes) read
  the whole tree or leave the question to Git.
  Measured with the object reader order (`d8b7f47ba9`):
  the client's
  `prepare` went from 103 and 99 ms (unsigned,
  signed) to 83 and 76 ms.
- In-process checks of a commit Git built:
  done (`90c649249b`,
  `service_built_checks.rs`).
  The parents come from the
  commit object;
  a commit whose only parent is the branch tip is shown connected by walking its tree against the tip's
  tree
  (every differing blob must exist,
  every differing tree is read the same way).
  Only a proof is taken:
  a graft file,
  a
  shallow
  history,
  replacement objects,
  a merge,
  an amend,
  a root commit,
  an unreadable object or an object that looks missing
  runs
  the `rev-list` the landing always ran,
  so a refusal is Git's.
  Controls:
  no `rev-list` start for an ordinary commit
  (also
  with the objects only in a pack),
  two starts where grafts exist,
  a missing blob,
  tree and tree below a tree are
  refused as
  before,
  an object removed between the read of the commit and the check (the point `before-connectivity`) is refused,
  a
  merge goes to Git.
  The replaced commit's parents for an amend were listed by Git for every commit Git built even
  though
  only an amend uses them;
  they are read from the object store,
  and only for an amend.
  SHA-256 repositories use the same
  readers but no control of this item runs on one.
- Scratch repository refs in process:
  not done.
  The scratch repository's `update-ref`,
  `symbolic-ref` and `for-each-ref`
  are made under `src/native/commit/`,
  which another branch (`feat/git-policy-cli-commit-protocol-models`) is editing;
  a second
  edit there would conflict,
  and reftable repositories would stay on Git in any case.
- `git var` in process:
  not done.
  The two starts measure about 1.5 ms each in the `execve` trace;
  the rules to reproduce
  (missing name or address,
  the `GIT_*` variables,
  `user.useConfigOnly`,
  the date forms) are the largest risk of the
  four
  for the smallest gain.

## Evidence for raising the 5 s standard

No measured pair after this work reached 5 s,
so no raise is requested on measurements.
The evidence that bears on it:

- Production landings of 5 s or more:
  29 of 1,621,
  dominated by `sync_ms` and `journal_ms`;
  those steps still wait
  for ten syncs on the tip.
  Native Git waits for none,
  so while any sync stays on the foreground path the difference to native is the disk's
  stall,
  which was 119 s at most in the record and has no upper bound.
- Run-to-run spread:
  warm native 111 to 123 ms;
  warm wrapped on the tip 229 to 270 ms (signed);
  cold native 274 to
  295 ms.
  Spread of a stall cannot be measured without causing one.
- Remaining per-step costs on the tip (signed,
  medians):
  `prepare` 95 ms,
  `build` 41 ms,
  `submit` 80 ms (of which
  `sync_ms` 47 and `journal_ms` 12),
  `gate` 10 ms.
  The `sync_ms` and `journal_ms` are the ten syncs;
  they cannot go lower while the syncs stay on the path,
  and the
  rest is process starts (about 14 Git starts) and the work of the policies.
- Proposed number:
  none that is finite.
  A bound of "native plus 5 s" can hold only if the syncs leave the waiting path
  (options B to E);
  keeping all
  of them means the bound holds only while the disk's stall is under 5 s.
  Raising the number would not fix that,
  since the stall is not bounded;
  moving the syncs does.

Update after the owner's answer:
the syncs that were the only reason for "no finite bound" are now off the wait on a
landing
that publishes (none are waited for on the signed route),
so the argument for a raise from them is gone.
What
remains that the
person waits for and a stalled disk can lengthen:
Git's own pack sync of an unsigned commit's `fast-import` (6 to 8
ms idle),
the reads the client and the service make (cold:
native 274 to 295 ms against the base's 394 and 382 ms of delta
before the
second round),
and the writes of the private index copies.
None was measured above 5 s after the work.

## Every pair

Milliseconds,
rounds in order (the first is the cold first round of a fresh fixture),
direct then wrapped then
wrapped minus direct.
Runs `u*` are unsigned,
`w*` signed,
`c*` signed and cold.
`b` is the base,
`o` the installed 0.5.27,
`v` the build at `85e7ac3a86` (unsigned),
`w..f` the tip build (signed)
and `c..f` an earlier build of this branch (`49b5e1a0c2`,
signed,
cold).

```text
u1b commit-service
  direct  223 120 120 121 121 123 124 125 123 139 130 138 142 133 146 130 133 130 129 135
  wrapped 847 416 381 387 383 395 389 397 400 418 398 409 479 430 409 430 413 422 405 391
  delta   624 296 260 266 262 273 265 271 276 280 268 271 337 297 263 300 281 291 275 256
u2b commit-service
  direct  164 114 116 113 116 113 112 115 116 111 115 116 116 112 112 114 113 111 114 117
  wrapped 463 3311 358 363 358 366 371 619 367 366 363 362 711 388 356 360 355 358 357 364
  delta   299 3198 242 250 242 253 259 504 251 255 249 247 595 275 245 246 241 246 243 247
u1o commit-service
  direct  221 114 116 114 114 113 115 113 116 116 115 112 116 114 112 112 117 116 114 116
  wrapped 359 396 259 251 252 255 248 251 245 247 259 251 259 252 254 245 253 257 253 256
  delta   138 282 144 137 138 142 133 138 128 131 144 139 143 137 141 133 137 141 140 140
u2o commit-service
  direct  170 114 113 116 117 114 114 116 113 123 117 111 115 114 111 113 113 113 115 113
  wrapped 376 258 251 335 254 254 253 256 254 255 777 248 248 248 241 249 252 249 255 248
  delta   206 144 138 218 137 140 138 140 141 132 659 138 133 134 130 135 139 136 140 135
v1b commit-service
  direct  171 119 115 114 115 113 112 112 113 113 114 114 112 114 110 112 115 114 112 113
  wrapped 350 1077 237 228 211 226 226 232 231 231 226 233 227 229 227 230 228 226 228 228
  delta   179 959 122 114 96 113 114 121 118 118 111 119 115 116 117 118 113 112 116 115
v2b commit-service
  direct  170 112 113 111 111 111 111 112 114 112 115 108 113 111 112 114 115 112 109 113
  wrapped 350 1100 229 221 221 222 223 227 231 224 222 231 222 225 192 246 229 225 224 229
  delta   179 989 115 110 111 111 113 115 117 112 107 123 108 114 80 132 115 113 115 116
w1b commit-signed-service
  direct  170 118 121 118 118 119 117 118 118 122 118 116 115 118 115 116 117 116 121 117
  wrapped 847 456 410 413 419 425 425 412 420 408 417 403 414 411 416 412 415 414 417 414
  delta   678 338 289 294 301 306 308 294 302 285 299 288 300 293 301 296 298 298 296 297
w2b commit-signed-service
  direct  166 118 117 117 115 117 118 117 116 117 118 117 118 115 116 117 122 121 117 118
  wrapped 584 443 410 406 413 413 409 405 405 407 415 406 411 408 416 412 401 421 404 410
  delta   418 324 293 289 298 296 292 288 289 289 297 290 292 293 300 295 280 300 286 292
w1o commit-signed-service
  direct  175 116 116 116 116 117 116 115 112 117 116 118 114 116 119 114 116 115 117 115
  wrapped 411 289 297 285 291 288 285 287 289 294 286 288 296 285 293 278 293 284 292 286
  delta   236 173 182 168 176 171 169 172 176 177 170 170 182 169 174 165 176 169 175 171
w2o commit-signed-service
  direct  166 115 117 115 116 117 116 117 117 116 116 115 115 118 115 117 118 119 119 118
  wrapped 408 284 291 292 290 293 291 288 294 289 281 289 283 289 288 287 648 296 384 290
  delta   242 170 175 177 174 176 176 171 177 173 165 174 168 170 174 170 530 177 265 172
w1f commit-signed-service
  direct  171 114 113 116 115 113 115 116 115 118 116 114 116 118 116 113 115 115 117 119
  wrapped 1845 270 236 244 236 236 244 236 246 237 240 240 239 235 239 238 236 236 239 247
  delta   1675 155 123 128 121 123 129 120 131 119 124 125 122 117 123 125 121 122 123 128
w2f commit-signed-service
  direct  161 116 119 115 117 112 113 117 118 115 110 115 115 114 117 116 112 114 116 116
  wrapped 1542 264 235 238 242 240 239 234 239 236 233 235 230 239 239 233 234 269 236 236
  delta   1381 147 116 123 125 128 125 117 121 121 123 120 115 125 122 117 122 154 120 120
c1b commit-signed-service
  direct  317 281 281 279 283 281 274 295 280 284 288 285
  wrapped 883 733 675 643 695 628 683 623 680 639 682 630
  delta   567 452 394 363 412 348 408 327 400 355 394 345
c2b commit-signed-service
  direct  316 281 288 286 289 286 281 287 293 288 291 283
  wrapped 2003 680 682 660 671 592 673 626 706 589 676 626
  delta   1687 399 394 374 382 306 393 339 413 301 385 342
c1f commit-signed-service
  direct  2857 276 278 282 280 279 289 280 280 281 282 288
  wrapped 1756 562 564 518 562 534 557 509 567 516 561 527
  delta   -1101 287 286 236 281 255 269 229 287 235 279 239
c2f commit-signed-service
  direct  314 279 280 280 286 284 286 286 288 287 286 286
  wrapped 2251 2827 548 499 553 506 551 507 552 508 558 509
  delta   1937 2548 268 219 267 223 266 221 264 220 272 223
```

The second round (same fixture,
same scopes,
nineteen warm rounds plus the first round of each run):
`y*` unsigned
and `z*`
signed;
`b` is the base,
`o` the installed 0.5.27,
`p` the first-round tip (`ce711b0379`),
`f` the tip of this round.

```text
y1b commit-service
  direct  166 111 112 113 119 115 123 113 113 114 116 115 110 112 109 116 112 114 110 120
  wrapped 861 429 365 397 394 358 361 372 371 370 363 358 368 339 365 371 372 375 378 3523
  delta   695 318 253 284 275 244 237 259 258 256 247 244 258 227 256 255 260 260 268 3403
y1o commit-service
  direct  175 112 115 115 112 112 110 113 112 112 116 110 113 111 113 112 114 112 111 116
  wrapped 377 257 246 253 252 250 248 254 254 249 247 238 249 250 251 250 250 3069 254 259
  delta   202 145 131 138 140 138 137 142 142 137 131 128 136 140 138 138 136 2957 142 142
y1p commit-service
  direct  189 113 115 120 111 114 118 120 117 114 113 113 114 113 115 117 115 115 117 122
  wrapped 359 1505 221 245 226 223 299 257 225 222 226 227 232 223 226 224 219 226 228 223
  delta   171 1392 106 125 115 109 181 137 108 107 113 114 118 110 111 107 104 111 111 100
y1f commit-service
  direct  169 115 118 114 114 109 110 116 115 115 114 117 113 115 116 114 122 114 116 113
  wrapped 332 1651 267 175 159 178 162 168 168 183 165 171 166 168 171 172 163 466 167 184
  delta   162 1536 149 60 46 69 52 51 52 68 50 54 53 53 54 58 42 351 51 71
y2b commit-service
  direct  173 114 115 122 118 120 116 111 114 115 117 116 112 119 116 114 117 114 112 119
  wrapped 516 1046 368 359 425 368 366 358 367 366 367 358 365 366 2049 361 372 385 366 368
  delta   343 932 253 238 307 248 250 247 253 251 250 243 252 247 1934 246 254 271 254 249
y2o commit-service
  direct  167 119 118 117 119 114 115 113 112 115 116 111 112 114 113 112 113 114 114 113
  wrapped 369 253 285 256 264 2612 249 250 254 245 249 245 250 253 252 247 256 249 246 248
  delta   202 135 167 138 144 2498 134 137 142 130 133 134 138 139 138 135 143 135 132 134
y2p commit-service
  direct  165 113 112 110 112 108 108 110 114 111 110 114 113 114 114 111 114 113 109 114
  wrapped 343 1903 212 224 223 216 218 218 219 789 223 219 220 218 218 221 221 227 226 220
  delta   178 1790 100 114 111 108 110 108 105 678 113 105 107 105 104 110 107 113 116 106
y2f commit-service
  direct  162 114 112 113 114 119 112 113 116 112 111 116 109 110 115 113 108 113 118 114
  wrapped 317 1744 159 180 155 162 161 645 160 1752 161 163 159 161 162 174 159 163 163 164
  delta   155 1631 48 67 41 43 48 532 44 1640 50 46 51 51 46 62 51 49 45 50
z1b commit-signed-service
  direct  175 121 120 120 121 120 118 118 119 123 121 117 114 119 120 129 119 131 129 131
  wrapped 668 476 435 446 437 430 435 445 441 451 451 424 432 435 469 519 537 706 2169 530
  delta   493 355 316 326 316 310 317 326 322 328 330 307 318 316 349 390 417 575 2040 400
z1o commit-signed-service
  direct  172 122 117 116 121 126 123 125 125 119 122 127 119 118 122 128 126 124 124 118
  wrapped 417 302 295 302 302 299 297 308 300 299 297 296 299 299 318 300 307 329 298 292
  delta   244 179 178 186 181 173 174 183 175 180 176 169 180 181 196 173 181 206 174 174
z1p commit-signed-service
  direct  168 124 124 118 117 124 121 121 123 116 118 122 117 118 118 119 118 118 120 120
  wrapped 518 298 245 238 245 325 510 240 253 253 250 243 247 242 245 245 243 251 245 251
  delta   350 174 122 120 127 201 389 119 131 136 133 122 130 123 127 125 124 133 125 132
z1f commit-signed-service
  direct  169 119 124 119 113 123 122 120 118 119 117 118 118 115 117 121 116 118 118 118
  wrapped 346 1644 208 205 209 213 204 205 207 215 208 199 196 204 208 207 202 203 209 224
  delta   176 1525 84 86 95 91 82 86 89 96 91 80 77 89 91 86 87 85 91 106
z2b commit-signed-service
  direct  427 117 116 115 123 122 120 122 123 118 114 119 118 129 118 123 124 121 119 131
  wrapped 2800 492 434 438 438 445 456 453 443 448 454 452 446 487 528 636 927 532 867 6350
  delta   2373 376 317 323 315 324 336 331 320 330 339 333 329 358 411 512 803 411 748 6219
z2o commit-signed-service
  direct  172 118 116 488 123 125 123 127 120 120 121 120 120 122 124 120 119 120 124 127
  wrapped 421 289 295 295 423 298 294 301 299 298 298 293 290 356 299 292 302 292 305 299
  delta   249 171 179 -193 299 173 171 175 179 179 177 173 169 234 175 173 183 173 180 172
z2p commit-signed-service
  direct  175 121 116 118 120 120 115 120 119 123 121 123 169 126 122 121 120 120 138 120
  wrapped 511 273 240 250 248 247 252 295 308 260 308 330 771 247 244 245 242 253 1783 245
  delta   336 152 124 132 128 127 137 175 189 138 187 207 602 122 122 124 121 133 1645 124
z2f commit-signed-service
  direct  165 130 125 118 118 121 118 120 119 118 119 118 121 124 122 116 116 118 118 117
  wrapped 338 1137 203 217 199 217 198 202 202 205 198 207 202 205 207 203 2004 422 462 234
  delta   172 1007 78 98 81 96 80 81 83 88 79 89 82 81 85 87 1888 305 343 116
```

The earlier alternating runs at load 40 to 66,
before the fixtures refreshed their index (base against 0.5.27,
unsigned,
nineteen warm rounds):
base 518 and 520 ms of median delta (maxima 2,202 and 3,028 ms),
0.5.27 247 and 280
ms (maxima 1,328 and 1,898 ms).
The cold comparison of base against the build `49b5e1a0c2` (signed,
eleven warm cold rounds):
base 394 and 382 ms of
median delta,
the build 269 and 264 ms,
with native at 274 to 295 ms in all four runs.

## Not done, and why

- The scratch repository's refs and `git var` in process:
  see "Less work in the client and the landing",
  "Status after
  the
  third round".
- Option B of the pack sync (loose objects written in process):
  it needs the in-process reference write first.
- The queue cap:
  not shown,
  see "The queue and the next commit".
- The remaining client steps (`add` 24 ms,
  `ls-tree -r` 20 ms for the policies' baseline,
  `rev-parse` 8 ms,
  `cat-file --batch`
  6 ms,
  `diff-index` 6 ms) and the service's index build (about 15 ms):
  measured,
  not changed.
- The first-landing outlier (item 7):
  not isolated.
- The reproduction of an actual stall:
  banned as synthetic load;
  the production record is the only evidence of it.

## Controls that failed intermittently, not by this work

- `service_index_lock::tests::a_live_service_holders_lock_is_never_taken_over` aborted after the 120 s of the test
  runner
  (`SIGABRT`) in 4 of 8 runs of its module on the branch tip,
  in 1 of 4 on an earlier tip,
  in 1 of 5 after the base
  sources
  (`e7e304ffd6`) were checked out over it,
  and in 2 of 3 container gate runs;
  the first gate run of this work passed
  it.
  A stack of a hung run (`gdb -p`) shows the test thread polling in `wait_and_place` and the thread that raced it
  for the
  lock already ended:
  the thread returned its `IndexLock`,
  which lives until the join,
  so when it won the race the
  other
  acquisition waited for ever.
  It is a defect of the control,
  present on the base.
  The control now drops the lock in the thread;
  the module then passed 8 runs of 8.
- `service_sequencer::a_pathspec_commit_after_a_conflicted_cherry_pick_lands_and_carries_the_pick` failed once in a
  container gate run and passed alone and in a whole host run of the binary controls.
  Its count of finished batches reads `.done` files in a journal that sixteen services share,
  and the services' batch
  numbers come from the clock at their start:
  two of the services in the failing run answered with the same number
  (`batch=1793737952894759`),
  so one `.done` file stood for two batches and the count was one short.
- The change in `push_progress.rs` is 23 lines in `Meter` and `Clock`;
  the branch
  `fix/git-policy-cli-every-wait-has-a-bound`
  edits the same module,
  so the merge needs a look at `Clock::check`.

## Gate

The container gate (`native:test:container`) on the branch tip passed:
4622 tests run,
4622 passed,
7 skipped,
exit
status 0
(unit `gpc-gate-commits-fast-under-disk-pressure-20261103T160833-0500`).
The three gate runs before it failed on the two intermittent controls above and,
once,
on a clippy finding in this
work
(`chunks_exact` with a constant size,
fixed).

The gate of the second round,
on the tip that holds both merges and the owner's rule for syncs:
4703 tests run,
4703
passed,
7 skipped,
exit status 0 (unit `gpc-gate-commits-fast-under-disk-pressure-20261103T205122-0500`).

The gate of the third round,
on the branch `perf/git-policy-cli-less-foreground-work` at `5baabfe665` (the commits after it
change documents and a module header only):
4727 tests run,
4727 passed,
7 skipped,
Clippy clean,
exit status 0
(unit `gpc-gate-less-foreground-work-20261104T050534-0500`).
On the host before it,
the new and touched controls ran as
457 landing controls,
171 `repo_read` controls,
33 cache-tree and query controls,
127 snapshot,
landing-sync and durable
controls,
and 17 timing controls.

## Resume point (2026-11-04, review of the third round by the cross-vendor reviewer)

State:
`main` (5328f45ccc) is merged into the branch without conflicts.
Review file:
`/var/home/user/temp/agent/reviews/less-foreground-sol-1.txt`.
No gate has run on the fixed tip;
no gate unit is running.

Fixed,
each with its control (red first where the old code could show red):

- Quadratic proof of a wide directory:
  `the_proof_of_a_wide_directory_is_linear_in_its_width` (red:
  4,504,501 name steps for 3,000
  files,
  green:
  at most eight per entry).
- A matching cache-tree hiding a missing root tree:
  `a_root_tree_that_is_gone_is_not_hidden_by_a_matching_cache_tree` (red:
  the
  readers answered an empty diff),
  both diff paths,
  SHA-1 and SHA-256.
- A kept pack not flushed where native Git would flush the same objects loose:
  `a_kept_pack_gets_the_flush_native_gives_the_same_objects`
  (red:
  0 flushes),
  plus `the_value_with_packs_flushes_the_pack_and_its_index_and_keeps_the_other_components` against real Git and
  the cases in `the_import_gets_a_value_without_packs_only_where_loose_objects_are_not_flushed`.
- Effective history context:
  `every_setting_that_changes_the_history_git_shows_sends_the_checks_to_rev_list` (red:
  a
  `GIT_SHALLOW_FILE` answered from the store),
  with `GIT_GRAFT_FILE`,
  `GIT_REPLACE_REF_BASE`,
  `GIT_NO_REPLACE_OBJECTS`,
  the object
  directory variables,
  `--no-replace-objects` and `core.useReplaceRefs`;
  also pinned:
  a shallow file,
  a replacement object.
- Supported maintenance concurrency:
  written in `SPEC.md` ("Maintenance that may run while a commit is made").
  The statement in
  this document that "the guarantee that holds is the scratch reference" is too strong:
  the reference is a reachability root for a
  collector that reads references after it exists,
  and `prune --expire=now`,
  `gc --prune=now` and `repack -a -d` are not
  supported while a commit is made.
  The decision record's claim stays for the owner (not edited here).
- Pinned controls added:
  SHA-256 for the built-commit checks (loose,
  packed,
  missing tree),
  the built commit removed at
  `before-connectivity`,
  a built amend,
  split and sparse indexes in both formats on both diff paths,
  per-thread loose-look counter.

Measured after the fixes (new fixture of 21,070 files,
three alternating runs per arm,
warm,
pooled paired delta against native 52
and 56 ms):
base 111 ms unsigned and 146 signed;
previous tip 87 and 112;
fixed tip 85 and 123.
The fixed tip is within the spread
of the previous one (per-run medians 83 to 126 and 122 to 132 against 78 to 95 and 109 to 126),
so the walk change moved nothing
on this fixture,
where no directory on the changed path is wide.

Next:
run the gate on the tip
(`node /var/home/user/temp/agent/gate/launch-gate.ts <worktree> <label>`,
since renamed to `mise run //package/git-policy/cli:native:gate -- <worktree> <label>` on 2026-11-10,
with its verdict read by `native:gate:verdict`,
which replaces `wait-unit-gate.ts`);
then
`wait-unit-gate.ts`);
the one failure seen on the host,
`service_index_lock::tests::a_foreign_lock_is_described_with_the_process_that_has_it_open`,
passed alone and in a rerun (a timing-dependent control of the merged index reclamation work).
Not done:
a built root (unborn
branch) commit control;
the planning document's earlier "Result of the third round" figures predate these fixes except where
stated here.
