# Commit latency under disk pressure

The owner's rules, verbatim: "commits vs native git should not take 5s longer."
"You should (almost) never be waiting."
"Disk pressure being high is not an excuse."
And on how to fix it: "remove it or move it off the waiting path", and "sometimes the work itself can be optimized.
And we
control every part of the wrapper, including everything it embeds."
Host pressure is the condition to test under, never the cause to report; the cause is work the wrapper does on the
commit's foreground path that native Git does not do, or does much less of.

This note records what was measured on the 0.5.28 base (`e7e304ffd6`), which defects were found, what was changed
with a red control first, and what is left to the owner.
Branch: `fix/git-policy-cli-commits-fast-under-disk-pressure`.

## Result first

The 0.5.28 base is worse than the installed 0.5.27, in two ways.
Warm, its commits take longer (below).
And under a stalled disk it is more exposed: it waits for twelve syncs per landing where 0.5.27 waits for about four
and
native Git for none (60 against 21 over five traced landings).
The extra syncs come with what 0.5.28 adds on the landing path (the in-process index writer with its directory
syncs, the
start record and the sync before the completion record); only the totals were traced for 0.5.27, so which of the
twelve
are new was not itemized.
This branch removes one of those syncs and the other work below; it does not remove the rest.
The tip still waits for eleven syncs per landing (56 over five), which is 2.75 times the exposure of 0.5.27 under a
stall,
and its landing is still longer than 0.5.27's in `sync_ms` (47 against 25 ms, signed).
Removing those syncs from the waiting path is the owner's decision (options B to E, below), so the base is a 0.5.28
blocker for stall exposure until the owner decides.
Warm, the tip is faster than both.

Warm numbers first.
Two builds, alternating arms, nineteen warm paired rounds each (median of the wrapped time minus the native time of
the same round), on a main-shaped fixture (20,740 tracked files, 2.8 MB index, the main checkout's pack and hooks):

- Unsigned explicit-path commit: base 273 and 249 ms, 0.5.27 139 and 138 ms (native 113 to 130 ms).
- SSH-signed explicit-path commit, which is what the person's commits are: base 298 and 292 ms, 0.5.27 172 and 174
  ms (native 115 to 118 ms).
- The service's own account of the landing: median `landing_ms` 138 and 130 against 52 and 52, `sync_ms` 56 against
  24, and the client's `build` phase 22 against 2 ms (unsigned) and 90 against 41 ms (signed).
- Counts that do not depend on the disk (one traced unsigned commit): the base starts 12 fsyncs per landing against
  about 4 for 0.5.27 (60 against 21 over five landings), and its client makes 136,191 system calls against 90,708,
  with 5,890 file opens against 486.

The branch tip is faster than both, with the same measurement (signed, nineteen warm rounds, two alternations):

- Signed: tip 123 and 122 ms of paired delta, 0.5.27 172 and 174 ms, base 298 and 292 ms; native 115 ms, so the
  wrapped commit takes 2.07 times the native one (the base took 3.5 times).
- Unsigned (build at `85e7ac3a86`): tip 116 and 114 ms, 0.5.27 140 and 139 ms, base 273 and 249 ms; native 112 ms.
- One traced unsigned commit, client side: 6,884 `pread64` calls against 79,951, 14,474 system calls against
  136,191, nine Git starts against ten (the wrapper's own start excluded).

No pair of any run reached the 5 s bound: the largest was 3,198 ms (the base, the second round of a fresh fixture,
unsigned), and the largest of the tip was 1,675 ms (the first round of a fresh fixture, signed), then 989 ms
(unsigned).
Every delta of every pair is in "Every pair" below.

Whether the 5 s bound can hold when the disk stalls cannot be shown by these measurements, because the stall cannot
be produced without synthetic load, which is banned.
See "Evidence for raising the 5 s standard" for what the production record shows and what is left.

## How it was measured

The paired harness `bin/native-perf.ts` was extended, not replaced:

- `--shape <repository>` makes every fixture a reflink copy of a prepared repository.
  The prepared one here is a clone of the main checkout's `HEAD` with `git clone --no-hardlinks --single-branch`,
  its remote removed (the harness refuses a shape that has a remote), living under `${HOME}/temp/agent`.
  The copy gets `core.untrackedcache`, a refreshed index and the main checkout's `post-checkout`, `post-commit`,
  `post-merge` and `pre-push` hooks (`--hooks-from`), with `git-lfs` on the PATH (`--tool`).
  Every commit edits the tracked `README.md`.
- Isolation, as the harness already had it: the run's own scratch root, its own `HOME` and `XDG_RUNTIME_DIR`, one
  commit service per wrapped fixture started by the harness, the person's session bus and unit directory never read
  or written, and no remote on any fixture, so no push can leave the machine.
- Arms alternate within a round, direct first in even rounds and wrapped first in odd ones, and the arms use two
  copies of the same shape.
  Every measured command ran in one transient scope with `CPUQuota=400%`, `MemoryMax=8G`, `TasksMax=300` to `400`
  and `IOWeight=10` (never above 10), the same for both arms of a pair; a low weight skews both arms alike.
- The host was not quiet: load average 6.9 to 66 during the runs, input and output pressure `some avg300` 2 to 24
  percent.
  Nothing waited for a quiet host.
  Spread of the same binary on unchanged input: the native arm stayed within about 10 ms of its median over nineteen
  rounds (for example 111 to 123 ms warm, 274 to 295 ms cold).
- `--cold` drops the pages of the fixture's pack files, pack indexes, index and packed refs (`dd iflag=nocache
  count=0`, `posix_fadvise` `DONTNEED`, verified to take a 586 MB pack from fully resident to zero resident) before
  every sample on both sides.
  It is cache state of the fixture's own files and no load on the host.
- `--output` stores the wrapped commits' timing records and the service's `answered` lines (`evidence`), which split
  a slow sample into its phases; `--trace-directory` leaves an `strace` record of every sample and of each service
  for counting syncs, bytes and starts (the summary script lives in scratch, not in the repository).
- A fixture finding that matters for every later measurement: a reflink copy has new inodes and change times, so
  every index entry reads as changed until Git refreshes the index, and a signed commit on such a copy spent about
  four seconds in the scratch repository's `git commit` re-reading all 20,740 files (22,813 file opens in one traced
  commit).
  The person's index has been refreshed by their own `git status`; the fixtures now refresh theirs outside any
  timing.
  The first rounds measured before this change (the 0.5.27 against base comparison at load 40 to 66, with deltas of
  518 and 520 ms for the base) are marked as such below.
- Limits: the fixtures live on `/dev/mapper/crypt_sda` (btrfs, zstd) because scratch belongs under
  `${HOME}/temp/agent`, while the main checkout's `.git` is on `/dev/dm-0`; absolute seconds under pressure differ
  per device, counts do not.
  The first round of every run is a cold outlier on both arms and is reported apart from the warm rounds.

## What the production record says

The service's `answered` lines in the user journal, 2026-11-02T09:01:38Z to 2026-11-03T15:11:58Z, 1,621 landings:

- `landing_ms` median 67, 90th percentile 440, 99th percentile 16,323, maximum 119,836; 87 landings of one second or
  more and 29 of five seconds or more.
- The steps with a wait for the disk dominate the slow landings: of 1,346,019 ms spent in the landings of one second
  or more, 1,053,826 ms (78 percent) were `sync_ms` (the index follow-up that holds the lock) and 241,902 ms (18
  percent) were `journal_ms` (the intent write); `objects_ms` was 3,882 ms (0.3 percent).
  `sync_ms` reached 5 s or more in 14 landings (maximum 119,715 ms), `journal_ms` in 11 (maximum 40,188 ms).
- The 35.7 s commit of the brief (request `8b549c42-2df6-4afd-be20-85b8e296b98b`) had `submitMs` 8,227, of which the
  service's `journal_ms` was 8,003: one intent write, a file sync, a rename and a directory sync, waited for for
  eight seconds.
  Its 25.5 s of `build` and the 15.1 s and 8.3 s of `prepare` in the other two records have no service line; the
  client made no sync call in `prepare` (a traced commit shows none), and the sampler saw `read_bytes` of 10 MB and
  five major faults in the 16.6 s one.

Reading of this evidence: one mechanism, two instances.
Work that native Git does not do waits on the disk in the foreground: the service waits for a chain of syncs, and
the client reads about thirty times the bytes native Git reads, so a host that has evicted its pages makes the
client wait on reads.
The three blow-ups hit three different phases because each phase holds some of that work.
This is a reading of timing records and a sampler, not a reproduction: the stall itself cannot be produced here.

## Defects, ranked by what was measured

Counts are of one traced unsigned wrapped commit unless said otherwise; native Git for the same commit makes 39,958
system calls, 8 `pread64`, no sync call and 16 process starts (including the `post-commit` hook and `git-lfs`).

1. The service waits for twelve syncs per landing on the base (about four on 0.5.27, none for native Git): the
   intent file and its directory, the lock marker, the start record and its directory, the private copy of the
   index, the replacement file of the index update, the Git directory after the index rename, the completion record
   and its directory, the journal directory after the batch is marked done, and Git's own pack sync in
   `fast-import`.
   Each is a flush that a loaded disk can stall for seconds.
   Idle, they cost 21 ms each on average in the traced run (1.26 s over sixty syncs).
2. The client reads 46.9 MB and makes 79,951 `pread64` calls where native Git reads 1.5 MB and makes 8: the
   in-process object reader (`repo_read_pack.rs`, embedded code) searched a pack index one name at a time with a
   system call per step, opened every pack with 256 reads of four bytes, and walked the whole tree of a commit
   (about 1,400 trees) for a question about one changed file.
   This is what a page cache under pressure turns into disk waits.
3. A scan of every process the host runs, started by the watch of each Git child the wrapper starts, on its first
   look (`push_progress.rs`, `Clock::check` with `next_sample: now`).
   The scan reads a status file per process (841 files, 7 ms on this host) and the wrapper starts fifteen to twenty
   Git children for one commit, most of them gone within ten milliseconds.
   It grows with the number of processes the host runs, which is large exactly when the host is under pressure.
   Removing it took the unsigned `build` phase from 22 to 2 ms, `ref_ms` from 10 to 2 ms, and with the change below
   the paired delta from 155 to 115 ms.
4. The gate after the landing listed the whole tree of the base commit with `ls-tree -r` again and walked both full
   trees for the landed commit's changes: 69 ms of the signed commit's gate, 10 ms after.
5. The service started a `rev-parse` of the branch for every landing in a spelling the in-process readers do not
   answer (one Git start of about 10 ms warm, a cold start of the Git executable when the host has evicted it).
6. The client still starts nine Git processes (unsigned route) and the service three or four per landing; `add`,
   `cat-file`, `ls-tree`, `diff-index`, `rev-list` and `var` (twice) in the client, and for a signed commit the
   scratch repository's `commit`, `update-ref` and `symbolic-ref` remain (see the options).
7. The first landing of a fresh fixture is slow in some runs, on every build: `sync_ms` of the first landing was 98
   to 361 ms on
   the base (five signed runs) and 117 to 1,537 ms on the tip (eight signed runs, three of them above one second),
   23 to 48 ms
   on 0.5.27 and the tip's unsigned runs, and one base run had a second-round sample at 3.2 s.
   A trace of the tip's first landing (`NATIVE_PERF_TRACE_CALLS=execve,fsync,rename`) shows the in-process update,
   no Git
   `reset`, and two syncs that took 178 ms (the replacement file of the index) and 298 ms (the Git directory after the
   index rename), where the same syncs take 6 ms later in the run.
   The fixture's own setup has just written a lot (the reflink copies, the refresh of four indexes), and a sync
   waits for
   the writeback that is pending on the file system: the production stall in miniature, on a quiet host.
   Whether the tip's tail is longer than the base's is not established; the base pays the same backlog in the sync
   it makes
   on the private copy, which the tip no longer makes.

## What was changed

Each change has a control that was seen to fail first, then pass; the commit of the red control is named.

- A pack lookup reads the names of one first byte at once and keeps them (`repo_read_pack.rs`).
  Red `a_lookup_reads_the_index_a_few_times` (`7a6fb07ca0`), green `f96769481f` and `62d7385748`; the control counts
  reads of the index of a real pack of 400 files: at most two per found name, one read per bucket in all.
  Embedded code: this is the object reader the crate links and owns.
- Opening a pack reads the header and the table of counts in one read.
  Red `opening_a_pack_reads_its_index_a_few_times` (`a6c0583525`, 257 reads), green `beb2aa8232`.
  Embedded code.
- The in-process `diff-tree` of one commit reads only the subtrees that differ from its parent
  (`repo_read_query.rs`).
  Red `the_changes_of_a_commit_are_found_without_reading_the_unchanged_directories` (`ce27cde2ce`, 86 entries
  decoded for a repository of 40 directories), green `49b5e1a0c2`; the answer still equals Git's in the differential
  controls.
  Embedded code.
- The files of a commit are listed by Git once per process (`candidate_store.rs`).
  Red `a_commit_listed_by_one_store_is_not_listed_again_by_another` (`39ff2056af`), green `07ddc2c336`.
- A landing no longer flushes the private copy of the index when the in-process update replaces it with a file that
  is flushed itself before its rename (`service_land_copy.rs`, `service_land_follow.rs`, `index_reset_file.rs`).
  Red `a_landing_waits_for_no_sync_that_protects_nothing` (`de5c3a9b99`), which lists the syncs of one landing by
  name; green `c5a7953c70`.
  A copy installed as it stands (no paths, or changed in place by Git) is still flushed once, so no protection was
  dropped.
  One of the twelve syncs and a forced 2.8 MB write are gone; eleven remain (ten when Git built the commit).
- A landing asks for the branch commit as `<ref>^{commit}`, the spelling the in-process readers answer
  (`service_land.rs`).
  Red `a_landing_reads_the_branch_it_moves_without_starting_git` (`ef21bfd223`), green `769975ef5e`.
- The watch of a started child looks at the child alone on its first look; the machine's processes are listed from
  the next sample on (`push_progress.rs`).
  Red `a_child_that_has_just_started_is_looked_at_without_listing_the_machine` (`921e199a5a`), green `85e7ac3a86`.
  What it changes in behavior: a child silent from its start shows no progress for one more sample interval before a
  stall window can be refreshed by a first sight; the stall window itself is unchanged.
- The harness (`cd592cf885`, `aa1f633677`, `ed2f907a3e`): the options above, the evidence, the refresh of the
  fixtures.

## Left to the owner

None of these was implemented, because each drops or moves a sync that protects a person's committed or staged data,
or changes what the wrapper promises.
The ranking weighs the worst wait the person sees, then the risk to data, then the work.

### The ten syncs that remain on the foreground path

What each protects, from `service_journal.rs`, `service_index_hold.rs`, `index_reset_publish.rs`: the intent
(`sync-intent-file`, `sync-intent-dir`) lets a restart finish a batch whose branch moved; the lock marker
(`sync-lock-marker`) lets a restart reclaim the service's own stale lock; the start record (`sync-start-file`,
`sync-start-dir`) stops a replay from resetting paths another session staged; the replacement file
(`sync-index-publish`) and the Git directory (`sync-index-dir`) make the installed index durable; the completion
record (`sync-indexed-file`, `sync-indexed-dir`) stops a replay over what was staged after the lock was given up;
the journal directory (`sync-journal-dir`) makes the done mark durable before the record is removed.
Native Git forces none of these (measured: no sync call in a traced native commit on the fixture).
All of them matter only for a power loss or a kernel crash; a killed process leaves the page cache intact.

- Option A, keep all ten: no risk, and the worst wait stays the longest stall of the disk times ten (production:
  `sync_ms` up to 119 s).
- Option B, make the done mark durable after the answer: the rename that marks the batch done and its directory sync
  (`sync-journal-dir`) come after the reply, still before the next request on that journal.
  Pro: one sync leaves the wait with no change to what is protected, because the intent stays durable before the
  branch
  moves, the completion record stays durable before the index lock is given up, and a restart finishes an open batch.
  Con: only one sync of ten; a kill between the reply and the sync leaves a batch the restart finishes again, which
  is the
  case it already handles, and the window needs a control.
- Option C, overlap independent syncs on threads (the intent write with the lock placement, whose marker is flushed
  too).
  Pro: waits overlap, so a stall is paid once and not twice or three times.
  Con: the order the landing documents must be reproduced, and every kill point control has to be extended.
- Option D, drop the directory syncs where the file system makes the file's sync cover its directory entry (btrfs
  logs the parent directory of a file created and renamed since the last commit when the file is synced).
  Pro: four syncs leave.
  Con: this reading of btrfs is from memory, not from the kernel source, and must be checked in
  `fs/btrfs/tree-log.c` before anyone relies on it; it also depends on the file system.
- Option E, native-equal: none of the ten, with the journal written in the page cache only.
  Pro: nothing waits on the disk.
  Con: a power loss can lose a landing's bookkeeping, so a replay can reset paths another session staged.
- Ranking: C > B > D > A > E.
  C shortens the worst wait the most without changing what is protected, at the price of a larger change than B; B is
  small and safe but removes one sync; D removes more but rests on an unverified reading of a file system; A is the
  status quo; E removes the protection the journal exists for.

### Git's pack sync in `fast-import`

For a commit the service builds itself, `git fast-import` syncs the pack it writes.
A commit Git built (signed commits) takes another route and has no such sync.
Option A: keep it.
Option B: write the objects as loose files in process, as native Git does, with no sync (native default), which also
removes a Git start and the `unpack-objects` child.
Pro of B: one sync and two processes fewer.
Con: the objects are not synced before the branch moves, as in native Git, and it is new code that writes into the
object store.
Ranking: B > A.

### The service unit and the host's swap

The unit (`service_units/cli-git@.service`) sets no memory directive; its comment records that a memory cap made Git
re-read pages.
`MemorySwapMax=0` (systemd.resource-control: it limits the swap the unit's processes may use; checked in the local
manual page) would keep the service's own pages and its Git children's out of swap.
Pro: the service's 22 MB (186 major faults in the sampler of the 16.6 s commit) stays resident.
Con: it covers the Git children too, and was never measured to matter here; swap use on the host was 7.7 GB of 80
GB.
A softer `MemoryLow=` protects against reclaim without forbidding swap and is not measured either.
Ranking: not before a measurement shows service pages being swapped during a slow commit.

### Less work in the client and the landing

- Compare the index with `HEAD` through the index's cache-tree extension (`index_cache_tree.rs`): the in-process
  `diff-index --cached` still reads about 1,400 trees of the base commit once per process, where a valid cache-tree
  names the unchanged directories without reading them.
  Pro: the largest remaining client read.
  Con: sparse indexes, intent-to-add and unmerged entries need their own cases.
- Replace `rev-list --parents --max-count=1` and `rev-list --objects <built> --not <old>` (the landing's checks of a
  commit Git built) with in-process reads of the commit and of the trees along its changed paths; Pro: two Git
  starts of the three or four per landing.
  Con: the second is the connectivity check that guards a pruned object, so the control must remove an object
  between the two reads.
- Write the scratch repository's `HEAD` and branch file in process instead of `update-ref` and `symbolic-ref`, and
  list the refs with the in-process reader instead of `for-each-ref`: Pro: three Git starts of the signed route.
  Con: reftable repositories stay on Git.
- Compute the author and committer lines in process instead of two `git var` starts: Pro: two starts.
  Con: Git's rules for a missing name or address, the `GIT_*` variables and `user.useConfigOnly` must be reproduced
  exactly.
- Ranking: cache-tree > in-process `rev-list` checks > scratch refs > `git var`, by the reads and starts each
  removes.

## Evidence for raising the 5 s standard

No measured pair after this work reached 5 s, so no raise is requested on measurements.
The evidence that bears on it:

- Production landings of 5 s or more: 29 of 1,621, dominated by `sync_ms` and `journal_ms`; those steps still wait
  for ten syncs on the tip.
  Native Git waits for none, so while any sync stays on the foreground path the difference to native is the disk's
  stall, which was 119 s at most in the record and has no upper bound.
- Run-to-run spread: warm native 111 to 123 ms; warm wrapped on the tip 229 to 270 ms (signed); cold native 274 to
  295 ms.
  Spread of a stall cannot be measured without causing one.
- Remaining per-step costs on the tip (signed, medians): `prepare` 95 ms, `build` 41 ms, `submit` 80 ms (of which
  `sync_ms` 47 and `journal_ms` 12), `gate` 10 ms.
  The `sync_ms` and `journal_ms` are the ten syncs; they cannot go lower while the syncs stay on the path, and the
  rest is process starts (about 14 Git starts) and the work of the policies.
- Proposed number: none that is finite.
  A bound of "native plus 5 s" can hold only if the syncs leave the waiting path (options B to E above); keeping all
  of them means the bound holds only while the disk's stall is under 5 s.
  Raising the number would not fix that, since the stall is not bounded; moving the syncs does.

## Every pair

Milliseconds, rounds in order (the first is the cold first round of a fresh fixture), direct then wrapped then
wrapped minus direct.
Runs `u*` are unsigned, `w*` signed, `c*` signed and cold.
`b` is the base, `o` the installed 0.5.27, `v` the build at `85e7ac3a86` (unsigned), `w..f` the tip build (signed)
and `c..f` an earlier build of this branch (`49b5e1a0c2`, signed, cold).

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

The earlier alternating runs at load 40 to 66, before the fixtures refreshed their index (base against 0.5.27,
unsigned, nineteen warm rounds): base 518 and 520 ms of median delta (maxima 2,202 and 3,028 ms), 0.5.27 247 and 280
ms (maxima 1,328 and 1,898 ms).
The cold comparison of base against the build `49b5e1a0c2` (signed, eleven warm cold rounds): base 394 and 382 ms of
median delta, the build 269 and 264 ms, with native at 274 to 295 ms in all four runs.

## Not done, and why

- The plain landing's `rev-list --parents` and connectivity check, the scratch repository's refs, `git var`, the
  cache-tree comparison: options above.
- The first-landing outlier (item 7): not isolated.
- The reproduction of an actual stall: banned as synthetic load; the production record is the only evidence of it.

## Controls that failed intermittently, not by this work

- `service_index_lock::tests::a_live_service_holders_lock_is_never_taken_over` aborted after the 120 s of the test runner
  (`SIGABRT`) in 4 of 8 runs of its module on the branch tip, in 1 of 4 on an earlier tip, in 1 of 5 after the base sources
  (`e7e304ffd6`) were checked out over it, and in 2 of 3 container gate runs; the first gate run of this work passed it.
  A stack of a hung run (`gdb -p`) shows the test thread polling in `wait_and_place` and the thread that raced it for the
  lock already ended: the thread returned its `IndexLock`, which lives until the join, so when it won the race the other
  acquisition waited for ever.
  It is a defect of the control, present on the base.
  The control now drops the lock in the thread; the module then passed 8 runs of 8.
- `service_sequencer::a_pathspec_commit_after_a_conflicted_cherry_pick_lands_and_carries_the_pick` failed once in a
  container gate run and passed alone and in a whole host run of the binary controls.
  Its count of finished batches reads `.done` files in a journal that sixteen services share, and the services' batch
  numbers come from the clock at their start: two of the services in the failing run answered with the same number
  (`batch=1793737952894759`), so one `.done` file stood for two batches and the count was one short.
- The change in `push_progress.rs` is 23 lines in `Meter` and `Clock`; the branch
  `fix/git-policy-cli-every-wait-has-a-bound`
  edits the same module, so the merge needs a look at `Clock::check`.

## Gate

The container gate (`native:test:container`) on the branch tip passed: 4622 tests run, 4622 passed, 7 skipped, exit status 0
(unit `gpc-gate-commits-fast-under-disk-pressure-20261103T160833-0500`).
The three gate runs before it failed on the two intermittent controls above and, once, on a clippy finding in this work
(`chunks_exact` with a constant size, fixed).
