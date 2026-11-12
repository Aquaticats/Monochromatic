# Where a commit's time goes on 0.6.1-alpha.11, measured against 0.6.1-alpha.10

What this answers:
whether the wrapper's 0.6.1-alpha.11 release made a commit slower than 0.6.1-alpha.10 on this machine,
and if not,
where the seconds an agent waits actually go.

The bar is the owner's,
and it is the one `doc/planning/cli-git-commit-latency-under-many-agents.md` states:
a commit through the wrapper takes no more than 5 s longer than plain Git,
and commits never fail.

## The question, and what the log seemed to say

The timing log of the real repository (`monochromatic-git-policy-cli-commit-timing.jsonl`,
one line per wrapped commit in `.git`) reads,
for landed commits:

- `0.6.1-alpha.10`,
  its last hour before the upgrade (2026-11-12T00:00Z to 01:02Z,
  51 landed):
  median 2,645 ms,
  ninetieth percentile 14,391 ms.
- `0.6.1-alpha.11`,
  from its install (2026-11-12T01:02Z onward,
  40 landed by 01:49Z):
  median 11,583 ms,
  ninetieth percentile 34,132 ms.

The two windows were not measured under the same load.
The release gate's container build ran through alpha.11's window,
the load average stood near 120 on 8 CPUs,
and `/proc/pressure/io` reported `full avg10` near 37,
while alpha.10's window includes quiet hours.

## The medians of the phases cannot be added, and the log does add up

Reading the record as "add the phase medians and compare with the median total" is an error of arithmetic,
not a finding:
the median of a sum is not the sum of the medians when one commit is slow in one phase and the next in another.
On this log:

- alpha.11's median of each commit's own phase sum is 10,554 ms against a median of 11,583 ms of `totalMs`.
- The remainder the phases leave (`otherMs`) has median 0 ms and ninetieth percentile 2,086 ms,
  so the record did account for its own total in the log,
  except for the tail this record now names (below).

What does stand out in the log is that every phase grew by roughly the same factor,
which is what a machine whose disk got slower does,
and not what a new step in one phase does.

## The releases under the same load, alternating

The comparison that answers the question is the two released binaries making the same commit in the same place,
one after the other,
under the load the machine had.

Method:

- Both releases were unpacked from their GitHub release archives
  (`monochromatic-git-policy-cli-v0.6.1-alpha.10` and `-alpha.11`,
  `x86_64-unknown-linux-gnu`),
  each into a folder of its own.
- The fixture is a repository built for this measurement only:
  23,564 tracked files in 235 directories,
  an index of 3,023,840 bytes against the real repository's 3,261,963,
  and the real repository's commit-relevant configuration
  (`commit.gpgsign`,
  `gpg.format=ssh`,
  `core.fsmonitor` and `core.untrackedcache` as this account has them).
  It sits on the same btrfs volume as the real repository.
- Each binary has a `HOME`,
  an `XDG_RUNTIME_DIR` and a commit service of its own,
  started directly on the socket the client derives,
  so provisioning takes its fast path and no service manager was involved
  (none was started on this machine).
- Every round makes one commit through each binary in turn,
  reversing the order each round,
  after three discarded warm-up commits per binary.
  The command is the same one file commit each time,
  `git commit --message ... -- notes.md`.
- The raw figures are in
  `package/git-policy/cli/perf/commit-alpha-10-against-alpha-11-2026-11-12-alternating.json`.

Thirty rounds,
2026-11-12T02:06Z to 02:09Z,
load average 96,
`io full avg10` 36:

- alpha.10:
  30 commits,
  all landed.
  `totalMs` median 522,
  ninetieth percentile 5,035,
  lowest 194,
  highest 8,923.
  Interquartile range 1,698 ms.
  Phases:
  `prepareMs` median 100,
  `buildMs` 40,
  `submitMs` 186 (ninetieth percentile 4,915),
  `provisionMs` 0.
- alpha.11:
  30 commits,
  all landed.
  `totalMs` median 1,600,
  ninetieth percentile 4,990,
  lowest 229,
  highest 8,960.
  Interquartile range 2,323 ms.
  Phases:
  `prepareMs` median 106,
  `buildMs` 40,
  `submitMs` 144 (ninetieth percentile 2,579),
  `provisionMs` 0.
- The round-matched difference (alpha.11 minus alpha.10):
  median 218 ms,
  tenth percentile -5,918 ms,
  ninetieth percentile 4,066 ms,
  from -8,430 to 8,695 ms.
  alpha.11 was the slower of the two in 19 of the 30 rounds.

So the difference between the releases does not exceed the spread between repeated runs of one binary:
the median difference is 218 ms while one binary's own runs range over 8.7 s.

A second run added plain Git as a third arm,
so the bar is measured too
(`package/git-policy/cli/perf/commit-alpha-10-against-alpha-11-2026-11-12-with-native-git.json`,
21 rounds,
2026-11-12T02:45Z,
rotating the order so each arm runs first,
second and third equally):

- plain Git:
  median 61 ms,
  ninetieth percentile 113,
  highest 485.
- alpha.10:
  median 306 ms,
  ninetieth percentile 1,672,
  highest 10,020,
  interquartile range 254.
- alpha.11:
  median 231 ms,
  ninetieth percentile 607,
  highest 1,980,
  interquartile range 69.

Every commit of both releases stayed far inside the bar in the fixture,
and alpha.11 was the cheaper of the two there.

## The log by hour, which is what the claim "it did not last" rests on

Reading the whole log of this repository at 2026-11-12T04:35Z,
the median `totalMs` of landed commits by release and clock hour:

- 0.6.1-alpha.10:
  16Z 9,720 ms (23 commits),
  17Z 8,248 (15),
  18Z 2,423 (28),
  19Z 4,775 (67),
  20Z 3,739 (46),
  21Z 4,380 (46),
  22Z 1,771 (87),
  23Z 1,631 (94),
  00Z 2,542 (49).
- 0.6.1-alpha.11:
  01Z 11,254 (47),
  02Z 4,206 (57),
  03Z 2,178 (51).
- 0.6.1-alpha.12:
  03Z 14,878 (13),
  04Z 10,692 (53).

So alpha.11's first hour was several times its own later hours,
and alpha.10 alone ranged from 1,631 ms to 9,720 ms at the median between one hour and the next:
an hour on this machine says more about what else ran in it than about the binary.
alpha.12's second hour is still 10,692 ms over 53 commits,
so the shape "slow for one hour and then fine" is not what alpha.12 shows yet,
and no reading of these hours alone can say whether a change of release or the work beside it did it.
That is what the fixture below separates.

## The changeover, and plain load, in one fixture

The release's first hour was slow and the hours after it were not,
and the same shape repeated when 0.6.1-alpha.12 was installed on 2026-11-12 at 22:36 local (03:36Z):
4,331 ms for the install's own sample commit,
then 11.5 s to 13.6 s for the next three.
So the question is not which release is slower but whether a change of release costs the commits that follow it.

That question was measured in one disposable repository with four linked worktrees,
one `HOME`,
one runtime directory and one installed path,
with a supervisor that starts the service from that path whenever it ends
(the stand-in for the user manager's socket activation,
which this machine's rules forbid starting).
The installed binary is replaced by one rename,
as an install replaces it,
between rounds:

- A change of release:
  the bytes at the installed path become the other release's,
  and every worktree's service is still the old release's.
- The control:
  the same write with the bytes already installed,
  so nothing retires and nothing restarts.
- Both were repeated four times each,
  alternating alpha.11 and alpha.12,
  and the first commit in each of the four worktrees after each write is the sample.
- The raw figures are in
  `package/git-policy/cli/perf/commit-changeover-2026-11-12-four-worktrees.json`.

Sixteen commits after a change of release,
15 landed:

- wall time median 8,920 ms,
  ninetieth percentile 25,800 ms.
- Every landed one met the old release's service:
  `releasesSeen` names both releases and `updateRounds` is 1 to 55.
- `provisionMs` is **0** in all fifteen,
  so the time is not the units being written or the bus being asked.
- The client's reach holds it:
  `startWaitMs` up to 8,580 ms with up to 98 connection tries and 59 refused.
- The new service's first answer holds it:
  `answerWaitMs` up to 32,199 ms.
- The client's own build holds it:
  `shadow pool` up to 6,604 ms,
  because a service of a new process has no pool yet.
- One commit reported 19,849 ms of `otherMs`,
  time no phase named:
  the remainder this branch's `closingMs` and the named hand-off now account for.

Sixteen commits after the same bytes were written again,
all landed:

- wall time median 1,200 ms,
  ninetieth percentile 3,869 ms.
- `updateRounds` is 0 and `releasesSeen` names one release in every one of the sixteen.

Forty-eight steady commits beside them:
`totalMs` median 922 ms,
wall time median 1,570 ms.

So a change of release costs about six seconds at the median over the control,
and its spread is the machine's:
the control's own ninetieth percentile is 3.2 times its median,
and the steady commits' is 18 times theirs.
The marker is not the noise though:
every commit after a change met a service of the other release and no commit of the control did.

Two commits failed during the run,
one of them at the scratch `git commit` ceiling
with `ssh-keygen` running for over ten seconds:
signing a commit takes seconds on this machine at this load,
which is what the ceiling and its branch are about.

## What the fixture cannot show, and does show, about the real repository

The fixture is not the real repository,
and three differences matter:

- The real repository has 137 worktrees.
  Each has its own commit service and its own shadow pool,
  and a commit in one worktree pays for the first use of its own service.
- The real repository's queue
  (`.git/monochromatic-git-policy-cli/work`) held,
  on 2026-11-12,
  7,538 records in `done`,
  508 in `failed`,
  16 in `claimed` and 1 in `pending`.
- Its worktrees hold the trash parent every human-facing command scans
  (`.claude/cli-git-trash`),
  and a commit is a human-facing command.

The third difference is a change of this release.
`name_unaccounted_trash` (`src/native/worktree/worktree_trash_unaccounted.rs`) is new in 0.6.1-alpha.11,
and it opened every record in `pending`,
`claimed`,
`recovering` and `failed` to find the few that record a trash.
A trace of one ordinary commit in this repository through the installed alpha.11,
with `strace --follow-forks --syscall-times`,
names the queue 534 times:
508 opens under `failed`,
16 under `claimed`,
1 under `pending`,
and the rest of the directories.
That is 525 record reads per commit to use one,
and it runs in `show_push_failures`,
before `run_commit_command`,
so no line of the timing record ever showed it.

What that costs depends on how the disk is doing,
and this machine's own trace of a commit shows the spread of a single call:
in the same trace one `openat` of the repository took 7.354 s,
one `statx` 1.784 s,
and `git update-index` 12.099 s.

Measured in the fixture with a queue of the real size
(508 failed,
16 claimed,
1 pending,
in a linked worktree,
which is where agents commit),
the scan is not distinguishable from the noise:
*wall time minus the record's own `totalMs`* was 13 ms (alpha.10),
14 ms (alpha.11) and 7 ms (this branch)
with 12 rounds each,
and a machine-readable command such as `git log --oneline` never runs the scan at all
(11 to 14 ms on every binary).
A queue of 525 small records whose blocks are in the page cache costs milliseconds;
the same 525 opens on a saturated disk are what makes a command take seconds.
The figures are in
`package/git-policy/cli/perf/commit-queue-scan-2026-11-12-three-binaries.json` and
`package/git-policy/cli/perf/commit-queue-worktree-2026-11-12-three-binaries.json`.

## The time the record did not name

Two stretches of a commit were inside `totalMs` and inside no phase:

- The push's own records,
  written and promoted between the post-commit gate and the hand-off.
  Measured in the real log:
  `otherMs` reached 8,057 ms on alpha.11 and 14,401 ms on alpha.10
  while `scratchRemovalMs` and `stderrMs` were both 0,
  and the trace of one real commit shows the sequence exactly there:
  the queue's state directories made,
  the verdict written,
  `held/<id>` renamed to `pending/<id>`,
  the wake.
- The command's own unwinding after its last phase,
  before the record is written.

Both are now phases:
the promotion and a blocked gate's take-back are the hand-off phase's,
and the unwinding is `closingMs`.
`CommitTiming::accounting` states what the phases name and what they leave,
and two controls fail when the remainder passes a stated tolerance:
`commit::commit_timing::tests::an_unnamed_wait_passes_no_tolerance`,
and `commit_timing::the_phases_account_for_the_whole_of_a_wrapped_commit`,
which reads a real commit's record and allows 250 ms unnamed.

## What was changed

- `perf(git-policy-cli): a command opens only the queue records it can use` (`fc835cd1bc`):
  the trash scan passes over a record whose name is another kind before opening it,
  so a commit opens the one trash record instead of 525 records.
  The control is `worktree::worktree_trash_unaccounted::tests::the_scan_reads_the_trash_records_alone`,
  which reads a queue of every kind and counted 37 reads before the change and 1 after.
- `perf(git-policy-cli): a promotion moves its record without making the queue's directories` (`4544f0a056`):
  `promote` renames the held record first and makes the queue's state directories only when the rename has nowhere to
go,
  which removes a dozen metadata calls from every commit's hand-off.
  The control is `landing::work_item_holding::tests::a_promotion_moves_the_record_without_making_a_directory`,
  which fails on the code before this change with "the one rename the promotion needs is a rename".
- `perf(git-policy-cli): the timing record names the time its phases do not cover` (`aa5017034b`):
  the two named phases,
  `Accounting`,
  `closingMs`,
  and the two controls above.

## What was left to other work

- The seconds a commit spends inside its own preparation (the private index's copy and stat check,
  the scratch tree) are a property of the repository's size and of the disk,
  and they are the same code in both releases.
  `perf/git-policy-cli-commit-landing-object-write-off-the-commits-wait` owns the landing object write.
- Provisioning,
  and the first commit after an install waiting for a service of the old release,
  belongs to `perf/git-policy-cli-first-commit-after-an-install-does-not-wait-for-the-old-service`.
  What this record measured for it:
  `provisionMs` is 0 in every commit taken after a change of release,
  so the wait is neither the units nor the bus;
  it is the client's reach for the new service (`startWaitMs`,
  `connectTries`,
  `refusedConnects`)
  and the new service's first answer (`answerWaitMs`),
  which are what its change moves.
  The empty shadow pool a new service starts with (`buildParts.shadow pool` up to 6,604 ms)
  belongs to `perf/git-policy-cli-commit-service-hands-a-commit-its-shadow`.
- The shadow pool filling on the commit's path
  (`buildParts.shadow skeleton`,
  2,389 ms at the ninetieth percentile on alpha.11)
  belongs to `perf/git-policy-cli-commit-service-hands-a-commit-its-shadow`.
- No file these branches own was touched here.
  The nearest is `src/native/invocation/wrapped_command_notices_snapshots_and_manual_push.rs`,
  which calls the scan and belongs to the shadow-pool branch;
  the filter was made inside `name_unaccounted_trash` instead,
  and that file was not edited.

## What this could not determine

- Whether the scan,
  rather than the disk,
  is what made the hours after the install slow.
  The evidence says the count is new and large and the machine's per-call tail is seconds,
  but no measurement on this machine separates 525 cheap opens from 525 expensive ones without saturating the disk,
  and saturating it is what the owner's rules forbid.
- How much of the real repository's commit time is the 137 worktrees.
  The changeover fixture had four,
  and per-worktree services were not compared against a single shared one.
- How long the real machine takes to start a service of a new release.
  The fixture's supervisor starts one when it notices the old one has ended,
  which is not the manager starting it on the first connection,
  so the `startWaitMs` figures there are the harness's and not the machine's.
  What the service itself spends (its first answer,
  its empty pool) is the program's either way.
- The 20 s provisioning ceiling and the scratch `git commit` ceiling are two other branches' work and were left alone.
