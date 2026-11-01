# Translation repair history 2026-08-05 to 2026-08-07: sensitivity checks of the probe and the checkers

The quiet first reading of the probe and the planted-damage checks that followed:
the probe can hear,
the checkers can say no but miss collateral damage,
the corrections after the first run used the wrong configuration,
the withdrawn under-crediting finding,
and why rejected issues cannot reach the probe denominator.

Part of [Translation repair history: 2026-08-05 to 2026-08-07](translation-repair-history-2026-08-05-to-07.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Sensitivity of the probe and the checkers, measured

### Early signal from run 007: the probe has claimed nothing at all

Verified from the LIVE slice cache rather than waiting for an artifact.
Cached `ChunkRepairOutcome` values carry the probe report,
so the serialization
can be checked while the entry is still in flight:

```text
chunk=0 regions=1 probe=heard=3/3 probedRegions=1
chunk=1 regions=1 probe=heard=3/3 probedRegions=1
chunk=2 regions=2 probe=heard=3/3 probedRegions=2
chunk=3 regions=4 probe=heard=3/3 probedRegions=4
```

Serialization is correct:
all nine tally keys present,
`probedRegions` matching
`repairRegions` exactly,
full roster heard every time.

THE PART TO WATCH:
across 8 regions and 24 prober verdicts,
every verdict was
`no-introduced-defect-found`.
Zero corroborated,
zero contradicted,
zero
unanchored,
zero claims of any kind.

Two readings and they are not equally comfortable.

The benign one:
these edits really are clean.
They survived an editor ensemble,
a judge selection,
and a checker stage before the probe ever saw them,
so a low
damage rate is what a working pipeline should produce.

The uncomfortable one:
I OVER-CORRECTED.
The whole design fought one failure
mode,
a prober reporting the pre-existing defect because every region contains
one by construction,
and it fought it three ways at once:
no `clean` verdict,
pre-existing issues shown and labelled as not findings,
and a verbatim-quote
requirement.
A probe that never claims anything is not a conservative probe,
it
is an instrument with no reading,
and `majorityIntroduced=0` across the round
would be indistinguishable from a stage that is silently broken.

NOT acted on yet,
deliberately:
8 regions is far too small to retune a prompt,
and a restart now costs the round's progress for a guess.

WHAT TO DO WITH IT:
watch the claim counts as entries settle.
If the round ends
with zero claims of ANY kind across every region,
the probe has not measured
anything and must not be reported as evidence that repairs are clean.
Say so in
the verdict.
The diagnostic that separates the two readings is the CONTRADICTED
count:
a probe that is looking and failing to anchor produces contradicted and
unanchored claims,
while a probe that has been talked out of claiming produces
neither.
Zero of everything is the shape that indicts the prompt.

### The probe is not deaf: sensitivity measured, not assumed

`mise run //package/module/translation-repair:probe-sensitivity` (moved on 2026-10-18 to
`//package/translation-repair/probe:probe-sensitivity`)
(commit adds `corpus-run/probe-sensitivity.ts`).
Cat-themed fixtures only,
no corpus text,
writes nothing,
three model calls.

Run 007's first eight regions produced no claims at all,
which fits two very
different stories,
and waiting for the round to end separates them not at all.
Injecting damage does.
Result:

```text
envelope/clean         heard=3/3  noneFound=3   (no claims)
envelope/omitting      heard=3/3  removal=3
envelope/contradicting heard=3/3  corroborated=3
```

Perfect discrimination.
Zero false positives on a replacement that fixes its defect and introduces
nothing;
unanimous detection of a dropped clause;
unanimous detection of an inverted meaning.
Every claim anchored well enough for the deterministic screen to uphold it,
with nothing contradicted and nothing unanchored.

THE CONCLUSION THAT MATTERS:
run 007's zeros are the benign reading.
Those repairs are clean,
and a round reporting `majorityIntroduced=0` can be
reported as evidence rather than as an instrument with no reading.
The warning recorded in the previous section is DISCHARGED,
and the discharge
is measured rather than argued.

SECOND RESULT,
unplanned:
this independently validates the two-direction screen.
The omitting region is exactly the case a forward-only quote requirement could
never anchor,
and it came back `removal=3` rather than `unanchored=3`.
Had the omission fix not landed,
this check would have shown a probe that misses
the likeliest damage class,
and the whole round's zeros would have looked like
the broken reading.

MINOR,
not acted on:
two of three probers left `category` and `severity` empty
on their claims while still quoting usable evidence.
The fields are telemetry,
nothing reads them,
and the schema requires them so
they arrive as empty strings rather than missing.
Worth a prompt line only if a later analysis wants to group claims by class.

KEEP THIS CHECK.
It is cheap,
needs no corpus,
and answers the one question a
quiet instrument always raises.
Run it whenever the probe prompt changes.

### The checkers can say no, and are blind to collateral damage

`mise run //package/module/translation-repair:checker-sensitivity` (moved on 2026-10-18 to
`//package/translation-repair/bench:checker-sensitivity`).
Same experiment as the probe check,
aimed at the older and more load-bearing
stage.
Cat fixtures,
no corpus text,
three calls.

```text
genuinely-fixed    heard=3  fixed=3 notFixed=0 worse=0  resolved=true
untouched          heard=3  fixed=0 notFixed=3 worse=0  resolved=false
fixed-but-damaged  heard=3  fixed=3 notFixed=0 worse=0  resolved=true
```

FIRST RESULT,
and it settles a standing doubt.
Sol read the 98.1 percent resolution rate (2215 of 2257 accepted issues) as
"direct evidence that the current checker task is permissive and poorly
discriminating".
It is not.
Handed a candidate that is the DEFECTIVE TEXT ITSELF,
unrepaired in every
respect,
all three checkers answered `not-fixed`.
A rubber stamp cannot do that.
The 98.1 percent therefore measures the repairs,
not the checkers,
and may be
quoted as such.

SECOND RESULT,
which is the one worth acting on.
The third case fixes the stated defect and DROPS a clause while doing it:
`...windowsill, and she wakes when the sun moves.` becomes
`The cat sleeps on the windowsill.`
All three checkers called it `fixed`.
Not one voted `worse`.

That is the same damage,
in the same shape,
that the introduced-defect probe
caught 3 of 3 as `removal-corroborated` in its own sensitivity check.
The two
stages were handed equivalent fixtures and answered oppositely.

So the gap the probe was built for is now MEASURED rather than argued:
checkers answer the narrow question they are asked,
"is this accepted issue
gone",
and do not report damage the repair caused on its way.
`worse` is the
verdict that feeds `regressedKnownIssues`,
and it did not fire on real
collateral damage,
which is exactly the weakness recorded on task 53 and now
demonstrated instead of inferred.

WHAT THIS DOES NOT LICENSE:
it is one fixture,
and it says nothing about how
often such damage occurs in real repairs.
Run 007's first eight regions suggest
rarely.
The gate question still needs the round's numbers and is still the
user's.

### Stage sensitivity checks, as a practice

Two of these were built in one sitting and each settled a question that had been
argued for weeks:
whether the introduced-defect probe can hear anything,
and whether the checkers can say no.
Both were cheap,
both used cat fixtures with no corpus text,
both wrote nothing,
and each cost three model calls.

THE SHAPE,
worth reusing:
hand a stage a case where the right answer is obvious and known in advance,
plus a control where the opposite answer is right,
and see whether it distinguishes them.
An ensemble stage that agrees with itself proves nothing;
one that answers a planted case correctly proves it is reading.

WHEN TO REACH FOR IT:
whenever a rate is being quoted as evidence
(98.1 percent resolved,
zero introduced defects)
and nobody has shown the stage producing it can return the other answer.
A quiet instrument and a broken instrument look identical from the outside,
and
waiting for more data separates them not at all.

DELIBERATELY NOT BUILT:
the same check for the adjudication panel.
It is the most load-bearing stage for the milestone,
since precision IS the gate
and round two failed it at 0.740,
so a panel that waved through an obviously false claim would be the single most
valuable finding available.
It was skipped anyway,
because round three MEASURES this directly and better:
the human grades every sampled accepted issue as real or false,
with written
rationale saying why,
over fifty real claims rather than one planted one.
A synthetic panel check would answer a weaker version of a question whose
stronger answer is already arriving.
Reach for it only if the graded sheet leaves the cause of the false positives
unclear.

#### Correction: the first sensitivity run tested the wrong configuration

The result stands,
but the evidence for it did not,
and the gap was mine.

The first run passed `issues: []`,
so the sheet rendered "(none recorded)".
PRODUCTION NEVER DOES THAT.
Every real region arrives with the accepted issues it was cut for,
printed under
"PRE-EXISTING DEFECTS THIS EDIT TARGETED (these are NOT your findings)",
and that line is one of the three defenses against a prober reporting the old
defect.
It is therefore also the likeliest single thing to talk a prober out of
reporting anything at all.
Measuring sensitivity without it described a stage that does not run.

Re-run as a controlled pair,
every region probed twice:

```text
clean          prior=absent  noneFound=3      prior=shown  noneFound=3
omitting       prior=absent  removal=3        prior=shown  removal=3
contradicting  prior=absent  corroborated=3   prior=shown  corroborated=3
```

Identical in both conditions.
Showing the prior defect suppresses reporting THAT defect without suppressing
reporting NEW damage,
and does not induce a false positive on the clean control
either.
The defense does exactly the job it was designed for and nothing more.

So run 007's zeros remain the benign reading,
now established under the
configuration that actually runs rather than a simplified one.

THE LESSON,
worth more than the result:
a sensitivity check inherits every
simplification its harness makes.
Ask what the production call passes that the
check does not,
BEFORE reporting the check as decisive.
I reported it as
decisive first.

#### The checker check had the same fidelity gap, and closing it inverted the worry

The single-issue cases asked whether the stage CAN discriminate.
Production
never asks it that way:
`repair-chunk.ts` passes every accepted issue of a chunk
in ONE call,
and the 98.1 percent rate was only ever measured on mixed sheets.
A checker that keeps up on one issue and agrees with everything on seven would
produce that rate while proving nothing,
so the single-issue result could not
carry the conclusion on its own.

Mixed sheet,
three issues over one candidate that fixes the first and leaves the
second standing,
with a third that was never in the text at all:

```text
mixed-sheet/tense    truth=fixed      fixed=1 notFixed=2  resolved=FALSE
mixed-sheet/meaning  truth=not-fixed  fixed=0 notFixed=3  resolved=false
mixed-sheet/absent   truth=absent     fixed=1 notFixed=2  resolved=false
```

THE WORRY INVERTS.
The concern was permissiveness.
What the mixed sheet shows is
the opposite:
the checkers UNDER-credited a genuinely fixed issue,
two of three
calling the repaired tense gloss unfixed.
The gloss really was gone.
What else
was in the candidate was a glaring meaning error,
and the plausible reading is
contamination:
a defect elsewhere in the text drags verdicts down on unrelated
issues sharing the sheet.

They also mostly refused a fabricated defect,
2 of 3 answering not-fixed for
something never present,
which is the right answer available to them.

WHAT THIS IS AND IS NOT.
One sheet,
one deliberately adversarial fixture,
and a
candidate carrying a far louder defect than any real editor would leave.
It does
NOT establish that production under-credits:
the observed 98.1 percent rate is
hard to reconcile with strong under-crediting,
so either real chunks rarely
carry an unfixed issue beside a fixed one,
or the effect is fixture-specific.

WHY IT MATTERS ANYWAY:
if the effect is real at any scale it biases
`resolvedIssueIds` DOWNWARD on multi-issue chunks,
which feeds `resolvedTotal`
and `resolvedHighSeverity` into candidate selection.
That direction makes the
pipeline ship fewer repairs than it earned,
which is the safe direction to err
but is still a measurement error.
Worth a proper look with real chunk shapes if
round three's resolution counts look low against its repair grades.

#### CORRECTION: the under-crediting did not replicate

The previous section reported that checkers under-credited a genuinely fixed
issue on a mixed sheet,
two of three calling the repaired tense gloss unfixed,
and recorded it as a direction to watch that could bias `resolvedIssueIds`
downward.

Rerun with IDENTICAL inputs:

```text
mixed-sheet/tense    first run  fixed=1 notFixed=2  resolved=false
mixed-sheet/tense    rerun      fixed=3 notFixed=0  resolved=true
```

It did not reproduce.
That was run-to-run variance,
and the finding is
WITHDRAWN.

The isolation case,
added to attribute a cause that turned out not to exist,
still answers its own question and is worth keeping:

```text
all-fixed/tense    fixed=3  resolved=true
all-fixed/meaning  fixed=3  resolved=true
all-fixed/absent   fixed=3  resolved=true
```

Three issues on one sheet,
all genuinely repaired,
all credited unanimously.
SHEET SIZE DOES NOT DEGRADE CHECKER ACCURACY,
which was the concern worth
ruling out,
and it is now ruled out on the shape production actually uses.

THE ONE CONSISTENT IMPERFECTION,
seen in both runs:
the fabricated defect
(`adjudicated/absent`,
describing a dog that appears nowhere in either text)
drew one `fixed` vote out of three,
both times.
The majority correctly answered
`not-fixed`,
so the tally lands right,
but one checker in three will affirm a
defect that never existed.
Quorum absorbs it.
Worth remembering if the checker
roster ever shrinks below three.

THE LESSON,
and it is mine:
I reported an n=1 observation from a STOCHASTIC
ensemble as a finding.
This entire pipeline exists because individual models are
unreliable,
and every stage in it votes for exactly that reason.
A single
adverse draw is the least surprising thing such a system can produce.
Replicate
before recording,
especially when the observation is the interesting one.

#### The probe result held to the same standard that killed the other one

Withdrawing the checker finding for lack of replication while continuing to
assert the probe finding would have been selective skepticism:
keep the result
that flatters the work,
discard the one that does not.
Both were low-n.
So the
probe check was rerun under the same scrutiny.

Every line identical to the first run:

```text
clean          absent noneFound=3     shown noneFound=3
omitting       absent removal=3       shown removal=3
contradicting  absent corroborated=3  shown corroborated=3
```

Two runs,
two prior conditions,
unanimous across all four cells per region.
THE PROBE FINDING SURVIVES the standard that refuted the other one,
which is the
only reason it may now be quoted.

The durable point is the symmetry,
not the outcome.
An adverse result rerun and
a favourable result taken on faith is how a measurement programme talks itself
into whatever it started out believing.
Rerun both,
or neither.

#### Why rejected issues cannot reach the probe denominator

A worry worth writing down because the answer is not local to the reader.
`result.issues` carries EVERY adjudicated issue,
rejected and needs-human
included,
not only the accepted ones.
`readArtifactProbe` filters on
`repairDisposition === 'shipped'`,
so if a rejected issue could ever carry that
disposition it would land in a denominator the repair sheet never grades.

It cannot,
and the reason lives two modules away.
`deriveEditableEnvelopes`
filters to accepted issues BEFORE cutting any envelope,
so `EditableEnvelope`
`issueIds`,
and therefore `RepairRegion.issueIds`,
only ever name accepted
issues.
`buildIssueRecords` gives an issue the regions naming it,
a rejected
issue is named by none,
and `judgeDisposition` answers `no-region` for an empty
region list before it ever considers selection or blocking.

Verified by reading those three,
not inferred from the filter.
