//region Corpus pass limits
// The time budgets and counts a corpus pass runs under, kept apart from the
// procedure that applies them so each carries its own history. The ceilings a
// launch may override are resolved by the entry file inside the refusal
// boundary; what is fixed lives here.

/**
 Minutes expressed in milliseconds, for the time budgets.
 */
export const PASS_MS_PER_MINUTE: number = 60_000;

/**
 Minutes after which no new entry starts, counted on `monotonicMs` like the
 per-entry hard cap's timer, so setting the system clock neither spends the
 budget nor refunds it, and time the machine spends suspended does not count
 (ledger B78).

 Was 25, which throttled the whole accumulation to about one entry per launch.
 The interaction that caused it: `BANDS` puts the large band first within a
 rank, so a run starts a large entry, that entry alone runs past 25 minutes,
 and this check then refuses to start anything else. Runs 010 and 011 both
 show exactly that, one settling a single entry and one settling none.

 A long budget lets a run chain several entries instead. It is scheduling
 only: it changes when a run stops starting work, never what the pipeline
 finds, so unlike the per-call deadline it can move without splitting the pool
 into incomparable cohorts. The per-entry hard cap still bounds any single
 runaway, and slice-level resumability means an entry cut by that cap resumes
 on the next run rather than restarting.

 Raised from 240 alongside the hard cap, and for the same measured reason.
 Recall run 001 spent 252 minutes settling SEVEN of nine entries under a
 four-hour budget and recorded the other two as skipped, coverage 0.778. The
 ensemble and the naturalness lane only make each entry slower, so holding
 240 would have shrunk that further. Twelve hours leaves room for a full
 nine-entry pass.

 A skipped entry is lost coverage in the verdict, not saved money: the plan
 is flat rate, quota regenerates faster than runs spend, and the user
 confirmed cost does not matter.

 Raised from 720 because twelve hours could not clear the corpus in ONE
 invocation, and every extra invocation was fragmenting the pool. Measured
 from artifact mtimes across an evening: about 27 minutes per entry over a
 clean stretch and about 53 averaged over a whole span including stalls. At
 92 pending entries that is 41 to 81 hours, so a twelve-hour budget settles
 roughly 13 to 26 and stops, and reaching the full corpus needs four to seven
 resumes. Each resume re-reads HEAD, so under a policy of restarting whenever
 a fix lands, each one stamped a new commit: that is precisely how one
 directory came to hold 22 entries across four tips.

 Three days covers the pessimistic rate with room to spare. It is not a
 prediction that a run will take three days; the resume guards at startup
 (`assertArtifactsPlaceable`, `assertResumableSchemaGeneration`,
 `assertBuildGenerationResumable`) are what protect the pool now, and this
 only stops the BUDGET from being the thing that forces a fragmenting resume.
 */
const SOFT_BUDGET_MINUTES = 4_320;

/**
 Minutes ONE entry may run before its exchanges abort, on the timer clock,
 which a step of the system clock does not move and which does not count a
 suspend.
 Per entry, not per run: the ceiling was previously armed once for the
 whole loop, so an entry that started near the soft budget got only the
 remaining sliver, and Arita (12 slices, ~68 min) could never finish. A
 fresh timer per entry gives each its full budget regardless of start
 time. Entries far larger than the cap clears (aiyysk 77 slices,
 hulicaijia 65, ...) still exceed any single-run ceiling and need
 slice-level resumability, tracked separately.

 Raised from 90 on measurement rather than on feel. Recall run 001 timed
 seven entries end to end: per-slice rate ran 3.25 min at best, 5.56 at the
 median, and 8.56 at the worst, and its longest entry took 74.7 minutes for
 12 slices. The old 90 was therefore ALREADY marginal before this branch
 changed anything: at the worst observed rate a 12-slice entry needs 103
 minutes and would have been cut. The measured median also confirms the
 ~5.5 min/slice figure the old comment claimed.

 That rate is PRE-ENSEMBLE. It predates per-envelope judge rounds, the
 chunk-level round, and the whole naturalness lane, every one of which only
 adds. How much they add is unmeasured, so this is a bound against runaway
 rather than a tuned value: 180 clears 21 slices even at the worst observed
 rate, and 32 at the median.

 Cost is not the constraint being traded here. The plan is flat rate and
 quota regenerates faster than runs spend, and the user confirmed cost does
 not matter, so the thing a low cap actually costs is entries covered per
 run. Slice-level resumability means a capped entry resumes next run, so a
 generous cap risks time and never work.
 */
export const HARD_CAP_MINUTES: number = 420;

// RAISED FROM 180 TO 420 on 2026-08-17, on a measurement rather than on
// the reasoning in `HARD_CAP_MINUTES`'s TSDoc, which had only a bound against
// runaway to offer. That
// measurement timed the two-lane shape end to end and found 4 to 6 entries hitting the
// 180-minute cap, all of them clearing at 7 hours. Every argument in that
// TSDoc's paragraph on cost points the same way: cost is not the constraint, slice-level
// resumability means a capped entry resumes rather than dies, so the cap buys
// nothing except a shorter run and costs entries covered by it.

/**
 Soft budget in milliseconds.
 */
export const SOFT_BUDGET_MS: number = SOFT_BUDGET_MINUTES * PASS_MS_PER_MINUTE;

/**
 Entry ids previewed on the `--plan` line.
 */
export const PLAN_PREVIEW_COUNT: number = 5;

//endregion Corpus pass limits
