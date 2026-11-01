# Translation repair history 2026-08-05 to 2026-08-07: the probe's trigger rate on corpus artifacts

What the probe did on real settled artifacts:
the first artifact,
then the rate at five and at seven entries,
and the lesson not to quote a rate whose denominator is a single entry.

Part of [Translation repair history: 2026-08-05 to 2026-08-07](translation-repair-history-2026-08-05-to-07.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The probe on its first real artifact

### First real artifact: the probe fires on corpus data

`AmbeR_the_anpa` settled and `score-probe` read it:

```text
PROBE  entries=1 shippedRecords=35 unprobedRecords=0 regions=13
       majorityIntroduced=1 minorityIntroduced=1 noneIntroduced=11
CLAIMS added=1 dropped=2 contradicted=0 unanchored=0 degradedRosterRegions=0
```

FOUR THINGS THIS SETTLES.

The plumbing works end to end on real output,
not only on fixtures:
35 shipped records,
0 unprobed,
every one carrying telemetry the reader could
parse.

THE DEDUPE IS NOT A DETAIL.
Thirty-five shipped records collapsed to THIRTEEN
distinct regions,
a factor of 2.7.
Summing over records rather than envelopes
would have inflated every count by that much,
and nothing downstream would have
looked wrong.

The probe is not silent on real data.
The earlier "eight regions,
zero claims"
reading was a small sample and has now moved:
three claims across thirteen
regions,
one region drawing a majority and one a minority.

THE OMISSION DIRECTION IS EARNING ITS PLACE.
Two of the three claims are
`dropped`,
meaning wording present before the edit and absent after.
Under the
forward-only screen this session started with,
those two claims could not have
anchored at all.
The fix advisor caught is now validated on corpus data rather
than on a cat fixture.

ALSO WORTH NOTING:
zero contradicted and zero unanchored.
Every claim the
probers made resolved against the region it was about,
so the verbatim-quote
requirement is being honored rather than worked around.

WHAT IT DOES NOT SETTLE:
one entry.
`majorityIntroduced=1` of 13 regions is
about 8 percent,
which is the rate a gate would have blocked,
and whether those
blocks would have been RIGHT is exactly what the repair grades decide.
Do not
read 8 percent as a defect rate;
read it as the population `refutedByHuman` will
be measured against.

## The probe's trigger rate at seven entries and at five

### Trigger rate at seven entries: numerator still one

```text
PROBE  entries=7 shippedRecords=188 unprobedRecords=0 regions=72
       majorityIntroduced=1 minorityIntroduced=5 noneIntroduced=66
CLAIMS added=5 dropped=2 contradicted=0 unanchored=0 degradedRosterRegions=0
```

THE NUMERATOR HAS NOT MOVED ONCE.
Across 1,
5,
6,
and 7 entries the count of
regions a gate would have blocked has stayed at exactly 1 while the denominator
went 13,
67,
68, 72.
Reading that as roughly 1.4 percent is the wrong emphasis;
the shape is that a SECOND blocking region has not appeared at all across
seventy-odd regions of real corpus output.

WRONG,
AND KEPT HERE AS THE MISTAKE IT WAS.
A second blocking region did appear,
at 18 entries,
and by 38 entries the count was 8 across 508 regions,
about 1.6%.
"Roughly 1.4 percent" was the right emphasis all along and this paragraph
talked itself out of it.
One event cannot distinguish a rare thing from an
absent one,
and dressing a small numerator up as a qualitative shape is how a
sample size gets mistaken for a finding.
The corrected series and the reasoning
are in `doc/decision/introduced-defect-probe-gating.md`.

WHAT IT MEANS FOR ISSUE #53.
Every option on the table (fall back to a
runner-up candidate,
salvage by dropping confirmed-defective operations and
revalidating,
reject the whole chunk) is machinery that runs only when the probe
confirms a defect.
At one region per seven entries,
a salvage pass that must
reapply from the original target and rerun judging,
checking,
probing,
measurement,
and selection buys one region of preserved repair per seven
entries.
That is the cost-per-trigger the decision turns on,
and it is measured
rather than projected.

WHAT IT DOES NOT MEAN.
This is the rate a gate would FIRE at,
not a defect
rate.
Whether that single blocking region was right is what the repair grades
decide,
and `refutedByHuman` is the cell that answers it.

ALSO CONFIRMED HERE:
the tightened `requireRecord` (which now refuses an array)
read all seven settled artifacts without throwing,
so that change is verified
against real data rather than only against fixtures.

### The trigger rate at five entries, and why the one-entry reading misled

Five entries settled (`AmbeR_the_anpa`,
`Arita`,
`Acheron`,
`Anilovr`,
`Chinatsu_Suzuki`) and `score-probe` read all of them:

```text
PROBE  entries=5 shippedRecords=179 unprobedRecords=0 regions=67
       majorityIntroduced=1 minorityIntroduced=5 noneIntroduced=61
CLAIMS added=5 dropped=2 contradicted=0 unanchored=0 degradedRosterRegions=0
```

THE HEADLINE IS THE DENOMINATOR.
`majorityIntroduced` did not move at all:
it
was 1 at one entry and is still 1 at five.
The region count went from 13 to 67.
So the rate a gate would fire at is about 1.5 percent of regions,
not the 8
percent the first entry suggested,
and the single blocking region found so far
is one region across five entries rather than one per entry.

THIS IS THE CONSTRAINT ANY DESIGN OPTION MUST CLEAR.
Every option on the table
for issue #53 (fall back to another editor candidate,
salvage by dropping
confirmed-defective operations and revalidating,
reject the whole chunk) is
machinery that only runs when the probe confirms a defect.
At roughly one region
per five entries,
a salvage pass that must reapply from the original target and
rerun judging,
checking,
probing,
measurement,
and selection buys one region's
worth of preserved repair per five entries.
Cost per trigger is the number to
argue about,
and it is now measured rather than guessed.

WHAT MOVED INSTEAD IS THE MINORITY COLUMN:
`minorityIntroduced` went from 1 to
5,
tracking the region count almost exactly.
Probers keep making claims;
what
stays rare is a MAJORITY of them agreeing on the same region.
That is the shape
you would expect from an instrument with real but noisy sensitivity,
and it is
the opposite of the "silently broken,
always negative" failure the sensitivity
check was built to rule out.

STILL ZERO contradicted and ZERO unanchored across 67 regions.
The
verbatim-quote requirement is holding at scale,
not just on the first entry.

THE METHOD LESSON,
AGAIN:
this session already withdrew a checker finding taken
off n=1,
wrote down that n=1 on a stochastic ensemble proves nothing,
and then
quoted an n=1 probe rate as a design constraint anyway.
The rule is not "be
careful with small samples",
it is DO NOT QUOTE A RATE WHOSE DENOMINATOR IS ONE
ENTRY.
Re-run `score-probe` as entries settle;
it costs no quota and reads only
local artifacts,
so there is never a reason to be working from the stale one.
