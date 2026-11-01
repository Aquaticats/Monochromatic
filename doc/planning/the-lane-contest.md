# The lane contest

The stage that was missing.

## What was actually absent

`compareDocumentLanes` is deterministic and answers one question:
do the two lanes' texts differ.
Over the six entries read on 2026-08-20,
four of sixteen slices agreed and
twelve differed.

`laneSelection` in the version 2 artifact has exactly one possible value,
`pending-human-decision`,
and that is deliberate:
a stated pending state rather than an absent field,
so "nobody has decided" and "this artifact predates the question" cannot be
confused.

The judge that declined 82% of lane contests was a scratch instrument written to
measure whether a contest was viable.
It was never a pipeline stage.

So the pipeline produced both lanes,
compared them,
and stopped.
Nothing chose.
That,
rather than any decision,
is what stood between the pipeline and a
document that can ship.

## Why the question had to change

`doc/audit/eight-entries-read-against-the-original.md` found the two lanes
failing in opposite directions:

-   The repair lane INHERITS.
    It edits the archive,
    so an invention no critic
    flagged ships untouched.

-   The translate lane DISCARDS.
    It owes the archive nothing,
    so accurate detail
    the archive knew and the source does not carry is lost,
    and where content
    lives only in pictures it has nothing to translate at all.

A judge asked "which of these is better?"
has to weigh an inherited invention
against a discarded detail with nothing to weigh them with.
An 82% decline rate is what that looks like from outside.

So the contest asks for the two findings that decide it,
per candidate,
and takes the choice as their consequence:

-   UNSUPPORTED:
    does this candidate state something the original does not?

-   DROPPED:
    does this candidate omit something the original does say?

Preference between two faithful renderings is explicitly NOT asked for.

## What the prompt commits to

THE ORIGINAL IS THE STANDARD,
per `doc/decision/translation-repair-output-goal.md`.
The archive rendering is shown as evidence about what the original says and as
wording worth keeping where it is right,
never as the thing a candidate is
scored against.

ACCURATE ADDED DETAIL IS NOT UNSUPPORTED.
The same decision record says detail a translator added is kept rather than
stripped,
and the reading found the translate lane stripping exactly that.
The prompt says so directly,
because a judge told only to check for unsupported
statements would call every such detail one.

DECLINING IS A VERDICT.
Two candidates that differ only in wording have no better one.
A judge forced to choose would be inventing a preference the evidence does not
carry,
and the stage records that separately from a judge that never answered:
those need opposite handling and would otherwise be indistinguishable.

## What the stage commits to

NO SINGLE MODEL DECIDES,
as everywhere else in this package.
Two voices must back a candidate,
and it must outpoll the other.

A TIE SHIPS NOTHING.
Deciding what a reader sees on a memorial page by which lane the code names
first is not a decision.

A DECLINE IS NOT A FINDING;
too few ballots is.
Three judges calling two candidates equally faithful is a settled slice.
Three judges never answering is not,
and only the second is reported.

## How it is measured

Thirteen slices from the eight read entries are a ground truth for the first
time:
each carries which lane the reading judged better,
and why.

The control arm asks the general-preference question over the same roster,
schema,
guard,
reader and counting rule.
Only the question differs,
which is the thing under test.

THE CAVEAT IS REAL AND STATED.
The ground truth is one agent's reading.
Earlier milestones calibrated agent grading against the owner's corrections
precisely because agent calls drift,
and several of the thirteen are close.
Agreement with that reading is a signal,
not correctness,
and the owner
reversing individual calls would move the score.

## What calibration found before it finished scoring

THE GUARD WAS STRICTER THAN ITS OWN CONTRACT,
and it cost voices.

`isLaneContestWire` documented itself as checking shape only,
and then demanded
that every member of `unsupported` and `dropped` name a candidate.
Judges do not always answer that way.
Two of the first sixty calibration voices filled the findings with the offending
phrases instead,
or annotated a candidate name with what was wrong:

-   `unsupported: ["at peace", "this one's for you", "even angry"]`,
    alongside `choice: "repair"`.

-   `unsupported: ["repair (changes ... and alters the dream request ...)"]`,
    alongside `choice: "neither"`.

Both replies carried a usable choice,
and both were discarded whole.
One slice settled on four ballots rather than six because of it.

THE CHOICE IS THE THING THE CONTEST COUNTS,
so no wording of a finding may cost
a voice.
The guard now checks shape.
The reader narrows the findings to the candidates they blame,
reading an
annotated name as naming that candidate and a longer word that merely begins
with one,
`repairing`,
as naming nobody.
The ballot keeps the findings verbatim beside the narrowed list,
so a judge
answering in phrases leaves an audit trail rather than reading as a judge that
found nothing.

WHY THE ROUNDS ALREADY SETTLED WERE KEPT.
The change only ever turns a lost voice into a kept one,
never the reverse,
which was checked case by case against the old predicate.
A round that settled on a full roster therefore settles identically under the
fix,
and only the one short round was re-run.

## The judge was reasoning correctly from the wrong evidence

THE ONE WRONG-LANE CALL HAD A CAUSE,
and it was not the judge.

Twelve rounds of the first calibration matched the reading on eight of the nine
slices where the reading named a lane,
declined none,
and named the opposite
lane exactly once.

Read against the Chinese,
that one slice turns on a name.
The source document's front matter declares an alias.
The archive renders it,
the repair lane keeps it,
the translate lane drops it,
and the contest chose the lane that dropped it.

Front matter is document-level and this stage is shown one slice.
So the declared name reached the judge nowhere in the original it was handed,
and appeared only in the archive and in one candidate.
Calling it unsupported is the correct inference from that evidence.
It is the wrong answer about the passage.

EVERY OTHER MODEL-FACING STAGE WAS ALREADY GIVEN THIS.
Critics,
refiners,
translators,
translate judges and the rendering audit all
receive `identityContext`,
built by `collectIdentityLines` from both sides' front
matter,
and have since the fix recorded as `M3 fix A`.
The lane contest was the only stage that was not,
because it was written after
that fix and nothing connected the two.

The subject now carries it,
the prompt renders it before the passages the way
the critic prompt does,
and the policy states what a declared name means:
carrying one is not unsupported,
and omitting one is a dropped detail.
The control arm receives the same block,
because the two arms may differ in the
question and in nothing else.

WHY THE MEASUREMENT WAS RESTARTED RATHER THAN FINISHED.
Every round of the first calibration was bought from a judge that could not see
this,
so the score it was heading for describes an instrument that no longer
exists.
Sixteen rounds of quota is worth less than a number that means something.

## The roster changed under the measurement too

The provider announced a replacement for one roster model and will retire the
older one shortly,
with no service level agreement,
so the retirement can land
without notice.
A retired id answers HTTP 404,
which is not in the transient retry set,
so
leaving it listed costs one lost voice per stage per call,
silently.
That already happened once,
on 2026-08-05,
with two other ids.

The replacement is identical on every field the catalog records and on every
capability the models endpoint reports,
checked against the live endpoint rather
than assumed,
and driven through the package's own client before the swap
landed.

THE PROVIDER'S OWN ALIASES STAY REFUSED,
and the reason recorded beside the
catalog is now the stronger one.
Double-seating a voting panel was the original reason.
The serious one is that an alias is a promise a very small operation can move:
a repointed alias changes which model votes with nothing in this repository
changing,
no build failing,
and no log line saying so.

## What the pass records, landed 2026-08-21

`laneSelection` had one kind,
`pending-human-decision`.
It now also carries `contested`,
holding one record per slice where the two
lanes left different wording.
The version 2 reader refuses unknown keys by design,
so the writer and the
reader moved together.

The verdict is a kind rather than the stage's raw `choice`,
because `neither`
means two unrelated things:
a roster that heard enough voices and backed no
candidate,
and a roster too few of whose voices arrived to settle anything.
`settled-neither` and `quorum-not-met` keep those apart.
Merging them would make a reader counting refusals count silence instead.
Both carry their ballots,
on the same footing as a win,
because a reader asking
why a slice shipped neither lane is looking exactly where a record without
ballots would be silent.

The reader recomputes each verdict from the ballots stored beside it and refuses
a disagreement,
which is the treatment the recorded lane comparison already
gets.
It also refuses a contest that does not answer exactly the slices the recomputed
comparison says the lanes worded differently.
Eligibility is derived from the two lane texts rather than from verdict names,
so it stays right if a verdict is ever added.

`LANE_CONTEST_QUORUM` is frozen by that recomputation.
Changing the number re-decides every contest already on disk and makes artifacts
settled under the old value refuse to parse,
so a different quorum is a new
artifact generation rather than a tuned constant.

Every pass now writes `contested`,
even when nothing differed.
"The roster was asked and nothing differed" and "nobody has asked" are different
facts,
and `pending-human-decision` from here on means only the second.

The contest has its own cache on its own version constant,
because every other
paid stage resumes and an uncached one would re-buy ballots on every resume and
write different ballots for identical inputs.
The key carries the archive rendering as well as the two candidates,
since the
judge is shown it as evidence.
Quorum failures are deliberately not cached:
an unheard roster is a transient
fact about a provider on one night,
not a property of the question.

## What is still owed after this

The question that is genuinely the owner's,
which the mechanism does not answer:
what a slice the contest declines should ship.
That is a values question about the release rather than a measurable one,
and it
is answerable now that the mechanism exists.
