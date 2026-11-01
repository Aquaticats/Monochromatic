# Translation repair handover snapshots: 2026-08-25 to 2026-08-27

Part of the [current translation repair handover](translation-repair.md).

## Contents

This record is kept as one file per subject or span,
each of at most 1,000 lines and named for what it holds.
New text goes into the file whose subject it belongs to;
a file that would pass the limit gives up a subject to a new file,
named for that subject and listed here.

- [Translation repair handover snapshots: the hf-prefix misroute that halved the roster, 2026-08-25 to 2026-08-26](translation-repair-handover-2026-08-25-to-27-hf-prefix-misroute-235-2026-08-25-to-26.md):
  The diagnosis of the defect tracked as 235,
  kept for its evidence:
  five Charm Hyper models sent to Synthetic and failing on every call,
  why nobody noticed,
  what it voided,
  the three instruments that misled,
  and where to pick the work up.
- [Translation repair handover snapshots: the writer seating and the live runs of the re-attempt queue, 2026-08-24](translation-repair-handover-2026-08-25-to-27-writer-seating-and-reattempt-queue-live-runs-2026-08-24.md):
  The editor and refiner seating after the forty-round calibration with its reach checks,
  and the three live runs that verified the entry re-attempt queue (196):
  the cap floor,
  the stall,
  and an entry that settled and published.
- [Translation repair handover snapshots: the full-roster calibration run in flight, 2026-08-25](translation-repair-handover-2026-08-25-to-27-full-roster-run-progress-and-power-projection-2026-08-25.md):
  How the forty-slice full-roster run behaved while it ran:
  the launch and its do-not-rebuild rule,
  wall time per slice and the revision that should not have happened,
  the refiner lane firing,
  and the 200 power projection that flipped on measured accrual.
  Sections:
  - [The run in flight](translation-repair-handover-2026-08-25-to-27-full-roster-run-progress-and-power-projection-2026-08-25.md#the-run-in-flight)
  - [What a slice actually costs in wall time, and a revision that should not have happened](translation-repair-handover-2026-08-25-to-27-full-roster-run-progress-and-power-projection-2026-08-25.md#what-a-slice-actually-costs-in-wall-time-and-a-revision-that-should-not-have-happened)
  - [The refiner lane fires, and the two empty slices were not a signal](translation-repair-handover-2026-08-25-to-27-full-roster-run-progress-and-power-projection-2026-08-25.md#the-refiner-lane-fires-and-the-two-empty-slices-were-not-a-signal)
  - [The `#200` power projection flips, on measured accrual (2026-08-25)](translation-repair-handover-2026-08-25-to-27-full-roster-run-progress-and-power-projection-2026-08-25.md#the-200-power-projection-flips-on-measured-accrual-2026-08-25)
- [Translation repair handover snapshots: what the run costs in credits and the spend telemetry built for it, 2026-08-25](translation-repair-handover-2026-08-25-to-27-credit-cost-and-spend-telemetry-210-2026-08-25.md):
  The credit estimate and its fourfold correction,
  the finding that no run ever recorded its token spend (210),
  the two defects only the consumer could find,
  and the dated price table with its per-seat report,
  controls and mutations.
  Sections:
  - [What the run costs in credits, and the correction that came with measuring it](translation-repair-handover-2026-08-25-to-27-credit-cost-and-spend-telemetry-210-2026-08-25.md#what-the-run-costs-in-credits-and-the-correction-that-came-with-measuring-it)
  - [Nothing we have ever run recorded its token spend (`#210`)](translation-repair-handover-2026-08-25-to-27-credit-cost-and-spend-telemetry-210-2026-08-25.md#nothing-we-have-ever-run-recorded-its-token-spend-210)
  - [Two defects that only writing the consumer could find](translation-repair-handover-2026-08-25-to-27-credit-cost-and-spend-telemetry-210-2026-08-25.md#two-defects-that-only-writing-the-consumer-could-find)
  - [A run's cost is now attributable per seat, and the run in flight will never be (2026-08-25)](translation-repair-handover-2026-08-25-to-27-credit-cost-and-spend-telemetry-210-2026-08-25.md#a-runs-cost-is-now-attributable-per-seat-and-the-run-in-flight-will-never-be-2026-08-25)
- [Translation repair handover snapshots: qwen3.8-max output booked as thinking, the volume guard blind to it, and what stragglers may cost (211), 2026-08-25](translation-repair-handover-2026-08-25-to-27-qwen38-max-211-thinking-accounting-and-straggler-cost-2026-08-25.md):
  The 211 thread:
  the symptom,
  the diagnosis from logs and source,
  the drop authorization and its cut-rate confound,
  the production-scale confirmation that no volume guard can see the model,
  the corrected prediction,
  and the straggler and serialization findings it redirected (213,
  214,
  215),
  with the deepseek anomaly and the tool-schema finding (216).
  Sections:
  - [`qwen3.8-max` books all of its output as thinking (`#211`)](translation-repair-handover-2026-08-25-to-27-qwen38-max-211-thinking-accounting-and-straggler-cost-2026-08-25.md#qwen38-max-books-all-of-its-output-as-thinking-211)
  - [`#211` diagnosed from logs and source, without spending a call (2026-08-25)](translation-repair-handover-2026-08-25-to-27-qwen38-max-211-thinking-accounting-and-straggler-cost-2026-08-25.md#211-diagnosed-from-logs-and-source-without-spending-a-call-2026-08-25)
  - [Authorization to drop weak models, and why the worst-looking seat is not one (2026-08-25)](translation-repair-handover-2026-08-25-to-27-qwen38-max-211-thinking-accounting-and-straggler-cost-2026-08-25.md#authorization-to-drop-weak-models-and-why-the-worst-looking-seat-is-not-one-2026-08-25)
  - [The measurement](translation-repair-handover-2026-08-25-to-27-qwen38-max-211-thinking-accounting-and-straggler-cost-2026-08-25.md#the-measurement)
  - [Why that is worse than a telemetry error](translation-repair-handover-2026-08-25-to-27-qwen38-max-211-thinking-accounting-and-straggler-cost-2026-08-25.md#why-that-is-worse-than-a-telemetry-error)
  - [The correction to `#211`'s recorded prediction](translation-repair-handover-2026-08-25-to-27-qwen38-max-211-thinking-accounting-and-straggler-cost-2026-08-25.md#the-correction-to-211s-recorded-prediction)
  - [What it redirects](translation-repair-handover-2026-08-25-to-27-qwen38-max-211-thinking-accounting-and-straggler-cost-2026-08-25.md#what-it-redirects)
  - [Two tasks this opened](translation-repair-handover-2026-08-25-to-27-qwen38-max-211-thinking-accounting-and-straggler-cost-2026-08-25.md#two-tasks-this-opened)
  - [Corrected count for `#213`](translation-repair-handover-2026-08-25-to-27-qwen38-max-211-thinking-accounting-and-straggler-cost-2026-08-25.md#corrected-count-for-213)
  - [Two more findings from the same run (2026-08-25)](translation-repair-handover-2026-08-25-to-27-qwen38-max-211-thinking-accounting-and-straggler-cost-2026-08-25.md#two-more-findings-from-the-same-run-2026-08-25)
- [Translation repair handover snapshots: closing out the full-roster run, 2026-08-25](translation-repair-handover-2026-08-25-to-27-closing-out-the-full-roster-run-2026-08-25.md):
  What to do when the run exits:
  the verified landing sequence for the parked work with the commit trap,
  the three measurements owed afterwards,
  the runway showing budget would not stop the run,
  and the two source fixes plus the production-ready signal still owed.
  Sections:
  - [Landing sequence for the parked work, verified ready (2026-08-25)](translation-repair-handover-2026-08-25-to-27-closing-out-the-full-roster-run-2026-08-25.md#landing-sequence-for-the-parked-work-verified-ready-2026-08-25)
  - [The run will finish, and budget is not what would stop it (2026-08-25)](translation-repair-handover-2026-08-25-to-27-closing-out-the-full-roster-run-2026-08-25.md#the-run-will-finish-and-budget-is-not-what-would-stop-it-2026-08-25)
  - [Two source fixes found while the run was live, both owed (2026-08-25)](translation-repair-handover-2026-08-25-to-27-closing-out-the-full-roster-run-2026-08-25.md#two-source-fixes-found-while-the-run-was-live-both-owed-2026-08-25)

## Current stop condition

Historical evidence only.
Candidate M failed on 2026-09-01,
and no corpus run,
model call,
spent-prompt retry,
or successor implementation is authorized by this snapshot.

## Superseded handover snapshot through 2026-08-27

Status:
the two-lane pipeline and all five per-slice overlap drivers are built;
whole-package suite passed 832 cases before live measurement.
The corpus overlap smoke pair settled and verified `keyword233` at overlap 1 and 4;
overlap 4 reduced normalized wall time from `0.383` to `0.291`.
The `Toka_ls` overlap-4 arm then settled 15 slices at normalized `0.147`,
but actual output reading found page not publishable:
linked death paragraph was recorded `gap-remains` and knowingly omitted.

That blocker is fixed in source.
Pass now asks whole-document coverage roster about every source-only slice;
admission requires verdict `absent` plus page shortfall or missing destination;
any remaining unfilled passage fails before artifact and publication.
Inherited invisible variants are folded before preparation.
Focused suites,
lint and five guard-failure mutations are green.
The matched old-digest serial `Toka_ls` arm settled in 313.24 minutes at normalized `0.420`;
overlap 4 reduced wall time 66.7 percent and normalized wall time 64.9 percent,
with same 61 unheard voices in both arms.
First whole-package suite exposed `UnfilledPageError` missing from names-only message inventory;
`d5b448754` classified its entry id,
counts and slice indexes.
Rerun passed 834 cases with zero failures;
log `~/temp/agent/buildAndTest-toka-gap-fix-v2-20260827.log`.
Current ten-model coverage control passed:
two targeted cuts flipped from `carried` to `absent` with 10 and 9 votes,
third moved to `partly-carried`,
and all three equal-size decoys received zero absence votes.
Both providers stayed wet.
Fixed-build `Toka_ls` overlap-4 rerun restored linked paragraph,
settled 15 wordings with zero silent,
retained source URL and removed all U+FEFF.
At 180-second grace strict reread still failed inherited first-person intrusion:
repair fixed it,
translate retained it,
and 8 heard contest voices split 4 to 4.
Same-digest overlap-4 rerun at 300-second grace recovered page:
15 complete slices,
35 voices unheard,
127.43 minutes wall over 15.15 call-hours,
normalized `0.140`,
with both providers wet.
Strict first and independent readings find no blocker or major;
page is publishable as-is.
Declined-archive seam remains open:
slices 3 and 11 had archive declined,
consolidation bought nothing,
and assembly revived archive.
No old-digest `Toka_ls` page is readiness evidence.
`doc/audit/translation-repair-output-reading-20260826.md` carries full trace;
`doc/planning/translation-repair-corpus-overlap-measurement.md` carries arm protocol.

WHAT THIS FILE IS.
What a session starting today needs,
and nothing else.
Capped at two thousand lines.
When it reaches that cap,
its oldest sections move verbatim into
`doc/handover/translation-repair-history.md`,
which has no cap.

WHY THE SPLIT.
This file reached twenty thousand lines,
at which point it stopped being a handover
and became an archive nobody could read at the start of a session.
The history keeps every measurement;
this keeps the working set.

THERE IS NO RELEASE DEADLINE.
The owner withdrew the one that used to sit here,
on the grounds that being aware of it
made the work worse:
it bought lower quality and,
more than once,
took longer anyway.
Do not reintroduce a date,
do not infer schedule pressure from anything in the archive,
and do not let "time remaining" enter any decision.
Quality is the only standing constraint.

ADVISOR IS A STANDING INSTRUCTION.
The owner has asked for it twice,
the second time in these words:
"Please call advisor as usual.
This is standing instruction."
Call `advisor` at the normal checkpoints:
before committing to an approach,
when stuck,
when changing approach,
and once more before declaring anything done.
An earlier session barred the tool on the grounds that it forwards the whole transcript
and the transcript names corpus entries from an unlicensed archive.
That bar is superseded and must not be reinstated.
The owner knows what the transcript carries,
has said so about this corpus more than once,
and has made the call.
Re-raising it at every invocation is the failure mode to avoid:
the concern is recorded here once,
and that is the whole of it.

WHAT BELONGS HERE.
Open tasks and their state,
decisions still in force,
defects not yet fixed,
and any measurement a reader would otherwise re-run.
What belongs in the history:
closed work whose conclusion is already encoded in the code,
and superseded reasoning kept only for its evidence.

## Five sections aged out into the history (2026-08-25)

Moved verbatim into `doc/handover/translation-repair-history.md` when this file reached its cap,
because each is closed work whose conclusion is already encoded in the code:

-   The writer calibration's coverage report,
    verified live.
-   The settled artifact speaks one vocabulary,
    as generation 3 (`#94`).
-   The stamped index is `sliceIndex`,
    as generation 4 (`#204`).
-   The read-any-generation dispatch never learned generation 3 (`#206`).
-   The 53 indirectly-reached modules,
    branch by branch (`#209`).

## Charm Hyper got credit, the full roster is running, and a run's cost is now measured (2026-08-25)

The owner bought 10,000 hypercredits.
`budget-sample` confirmed it live before anything was launched:

```text
METERS synthetic=wet hyper=wet syntheticWeekly=97.09290877272727%
syntheticFiveHour=2750/2750 syntheticThrottled=no hyperBalance=10000
```

Both providers wet at once,
which is the window `#200` had been waiting for since the 14-slice run of 2026-08-24 settled no seat.

Filed in:

- [Translation repair handover snapshots: the full-roster calibration run in flight, 2026-08-25](translation-repair-handover-2026-08-25-to-27-full-roster-run-progress-and-power-projection-2026-08-25.md)
- [Translation repair handover snapshots: what the run costs in credits and the spend telemetry built for it, 2026-08-25](translation-repair-handover-2026-08-25-to-27-credit-cost-and-spend-telemetry-210-2026-08-25.md)
- [Translation repair handover snapshots: qwen3.8-max output booked as thinking, the volume guard blind to it, and what stragglers may cost (211), 2026-08-25](translation-repair-handover-2026-08-25-to-27-qwen38-max-211-thinking-accounting-and-straggler-cost-2026-08-25.md)

## No volume guard can see `qwen3.8-max`, and stragglers may cost more than serialization (2026-08-25)

Measured on the live full-roster calibration at 15 of 40 slices,
from the run log alone.
Full working in `doc/audit/every-volume-guard-is-blind-to-one-model.md`,
committed as `d16a616e3`.
This run predates the parked `#211` fix,
so every number here is a clean pre-fix baseline.

Filed in:

- [Translation repair handover snapshots: qwen3.8-max output booked as thinking, the volume guard blind to it, and what stragglers may cost (211), 2026-08-25](translation-repair-handover-2026-08-25-to-27-qwen38-max-211-thinking-accounting-and-straggler-cost-2026-08-25.md)
- [Translation repair handover snapshots: the full-roster calibration run in flight, 2026-08-25](translation-repair-handover-2026-08-25-to-27-full-roster-run-progress-and-power-projection-2026-08-25.md)

## Corpus text in commits: what the owner decided, and the rule (2026-08-25)

Raised because a sweep found corpus text committed to a PUBLIC repository,
and the working rule carried across sessions said it should never be there.
`doc/audit/corpus-text-reached-a-public-repository.md` has the measurements.
The short version:
fifteen documents under `doc/` carry 185 lines of Chinese
source text,
plus English memorial sentences that a script-keyed scan cannot see,
on `origin/translation-repair-rebased`,
none of it on `origin/main`,
publicly readable since 2026-08-20.

THE OWNER DECIDED NOTHING IS REMOVED.
Exposing corpus text is sometimes fine,
because the owner is friends with the
people who run the site the corpus comes from.
So no file changes,
no history is rewritten,
nothing is force-pushed,
and the repository stays public.
A session that rediscovers this must not reopen it,
must not offer to scrub it,
and must not treat it as an incident.
The relationship that makes it fine is not visible from inside the repository,
which is why it is written here.

### The rule going forward

The instruction attached to that decision was to take extra care not to expose
more in commits later.
Permission is not indifference,
and it is not retroactive cover for adding more
without thinking.

-   Corpus text enters a commit only when it carries evidence nothing else can carry,
    and only as much of it as the evidence needs.
    Where a rendering changed a tense,
    or an aligner paired the wrong headings,
    the wording IS the finding and quoting it is right.
    Where the point is a count,
    a rate,
    a slice index or a model's behaviour,
    it is not,
    and the id and index say it better.

-   Never quote because quoting was easier than naming.
    That is the failure mode that produced 185 lines:
    each one was individually defensible and nobody was counting.

-   A document ABOUT the exposure quotes nothing.
    Naming files and counts is the whole discipline,
    so an audit that reprints the evidence repeats the fault it records.

-   Everything else about handling the corpus is unchanged.
    It is read at runtime from the pinned clone by `git show <sha>:<path>`,
    corpus files are never committed,
    and artifacts and grading sheets stay under run directories outside the repository.

### Why this is not an `AGENTS.md` rule

Proposed there and declined by the owner as too narrow to be worth a repo-wide
shortcode:
it is a rule about one corpus,
in one package,
under one relationship.
It belongs where the people working on that package will read it.

## Corpus exposure is not a blocker, and sanitization is the last step (2026-08-25)

Supersedes the caution in the corpus-exposure sections of this handover.
The owner set the plan after `#456` was measured:

-   COMMIT THE CORPUS HOWEVER IS CONVENIENT.
    Do not spend effort avoiding corpus text in commits,
    do not shrink a quote to dodge exposure,
    and do not treat a corpus-bearing diff as something to route around.
    Nothing about corpus text blocks any commit on this branch.

-   SANITIZATION HAPPENS ONCE,
    AT THE END.
    When the pipeline is production ready,
    tell the owner with the `AskUserQuestion` tool rather than in ordinary prose,
    because that is the signal they are waiting for.
    They will disable branch protection temporarily,
    and the corpus content committed along the way gets sanitized then.

-   THE DENY-LIST IS BUILT AND NOT INSTALLED.
    `doc/decision/corpus-deny-list-for-forbidden-strings.md` has the design and the evidence.
    Installing 10206 literal rules today would cost about two minutes of rule compilation
    on every single commit,
    because the scanner recompiles the runtime rules file per scan.
    `#456` is the fix.
    The deny-list waits for it rather than taxing every commit,
    and it waits without cost,
    since sanitization at the end covers the same ground.

WHAT THIS DOES NOT CHANGE.
The corpus is still read at runtime from the pinned clone,
corpus files are still never committed as corpus files,
and artifacts and grading sheets still live under run directories outside the repository.
The change is that quoting inside our own documents is now unremarkable.

FOR THE SANITIZATION ITSELF (rendering-8,
2026-08-26):
the persisted runs of the settled rendering audit,
`<runs dir>/rendering-audit-settled/*.json`,
carry document spans and model prose in every row's `report`
(`locator.text`,
`focus.text` on both sides,
each voice's `reason`),
so they are corpus-bearing whatever the `textIdentity` digest beside them says;
the module notes that once claimed otherwise were corrected in `744890056`.
