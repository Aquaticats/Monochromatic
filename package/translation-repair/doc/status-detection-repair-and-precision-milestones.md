# Status: the detection, repair and precision milestones, measured 2026-07-17 to 2026-08-12

Milestone one (detection recall),
milestone two (repair restoration rate) and milestone three (detection precision,
not met),
each with the date and roster it was measured under and the reading rules the owner set for the figures.

Part of [Status](status.md),
which lists every file of this record.

## Detection, repair and precision milestones

Milestone one (detection) is complete:
the seven-critic ensemble reached 0.981 recall on seeded errors over the reference corpus,
gated by the seeded-error benchmark harness
(`runCriticBenchmark`,
which no runner called by 2026-09-29 and which was removed then,
ledger B30;
the recall benchmark measures detection now).
Read that figure with its date attached.
It was measured on 2026-07-17 over 54 seeds against a roster of seven models,
and that roster no longer exists:
the provider has since withdrawn two of them and offered one replacement,
so six critics run today.

A re-measure on 2026-08-10 detected 24 of 27 planted omissions,
a rate of 0.889,
over nine entries balanced across the three size bands,
with zero policy declines,
so all three misses are critics failing to see a seeded omission
rather than the panel correctly ruling one a source defect.

That figure is **not** the configured roster's recall,
and the run says so itself.
One model returned schema-invalid output 312 times across five roles during it,
and the critic stage never once reached its full roster:
72 chunks ran at five voices of six,
eight at three,
and one at none at all.
So 0.889 describes what the pipeline delivered while one of six critics was effectively absent.
Read it as a measurement of a degraded ensemble,
which is a real and current operating condition rather than a spoiled run,
and not as evidence about the roster as configured.

Do not read it against the milestone-one figure as a regression either.
The two runs differ in roster,
entry set,
seed count,
and several stages,
so no delta is attributable to any one of them,
and 24 of 27 against 53 of 54 is not a statistically established difference (two-proportion z of about 1.8).
All three misses also fall in a single entry,
which went 0 for 3 while the other eight went 24 for 24,
so the sample is one entry failing rather than a uniform detection rate.

Milestone two (repair) is complete:
the full loop (critics,
claim aggregation,
adjudication panel,
editable envelopes,
editor through a deterministic apply gate,
resolution checkers,
lexicographic candidate selection)
reached a probe-adjusted effective restoration rate of 0.98 over 100 seeded omissions across 21 budgeted live runs,
graded by a source-anchored bilingual restoration judge (three judge models,
conservative lower-median verdict).
Misses are attributed,
never averaged away:
a derivability probe rules whether each missed seed's information was fully derivable from the source at all,
so embellishment-capped partials and correct refusals of underivable content are excused,
and only genuine editor shortfalls count against the editor.
The one reproducible shortfall class
(long omissions restored compressed) drove a rule now promoted into the baseline editor prompt:
enumerate the omitted source sentences clause by clause.

Milestone three (detection precision) is **not** met.
Its gate is human-graded precision of at least 0.9 over a stratified sample of accepted issues.
Round three was graded on 2026-08-12 and returned 0.791 strict,
0.810 excluded,
and 0.814 lenient over 43 gradeable items drawn from a pool of 740 across 18 settled entries.
All three readings improved on round two's 0.740,
0.787 and 0.800,
and none reaches the bar.

Round three also found a defect in the sampling instrument itself.
Seven of the 50 drawn items repeat a defect already drawn at an earlier position,
which the grader marked `Duplicate` and the blind pre-grades had independently annotated the same way.
A duplicate is now its own verdict,
excluded from every denominator,
because the pipeline reporting one defect several times is a different failure
from reporting a defect that is not there,
and only the second is what precision measures.
Counting them as false positives had dragged strict to 0.680 while every other reading rose,
which described the instrument rather than the detector.
Read the milestone-two figures as recall claims only:
they say the ensemble finds seeded defects,
not that what it reports is right.
Nothing here should be taken as evidence that an accepted issue is a real one until this gate is measured and passes.

The **repair** half is not fit to be measured yet,
and that is a finding rather than a gap in the schedule.
Round three's repair sheet was deliberately left ungraded:
reading it showed repairs that fix their claim while deleting nearby source-supported content,
including a contributor credit removed by an edit asked only to change a colon,
and 21 of 50 edits replacing a span more than 1.35 times the quoted defect.
The introduced-defect probe,
which exists to catch exactly that,
reported no finding from any prober on every one of those repairs.
So a shadow-mode probe reading clean is currently false assurance,
not evidence,
and repair quality claims should be read as unestablished until that instrument is fixed.

Every milestone figure in this section comes from a graded measurement rather than a self-report.
Where a stage grades itself the figure is named as telemetry and excluded:
`runIntroducedDefectProbe` ships in shadow mode for exactly that reason,
and the checker stage's resolution rate is a stage self-report,
which is why repair quality is graded on its own human sheet instead.
