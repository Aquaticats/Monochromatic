# Translation repair history 2026-08-15 to 2026-08-19: the window trial's pre-flight and plumbing, its blockers, the comments filed on the wrong tracker and the state the next session inherits

The pre-flight and plumbing for running production selection with a wider window,
the review finding that the two-arm design does not measure the window,
the missing verdict at document scale,
the six comments filed on the wrong tracker,
and the state the next session inherits.

Part of [Translation repair history: 2026-08-15 to 2026-08-19](translation-repair-history-2026-08-15-to-19.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The window trial's blockers and the state the next session inherits

### `#108`'s pre-flight, and the capability it turned out to need

Before spending quota,
the run was checked against what exists.

ALREADY THERE:
`runTranslateStage` puts the incumbent on the ballot and rotates
the slate by a hash of the slice,
so production selection over a named slice is
invocable and the incumbent's win rate is not also a measure of position
preference.

WAS MISSING,
NOW BUILT:
the stage had NO parameter for surrounding source,
so
`#108`'s wide arm could not have been run at all.
`neighbouringSourceText`
mirrors what `judge-fidelity.ts` already does,
caveat label included,
and is
absent by default so every measurement taken before it still describes the sheet
production sends.
The narrow-sheet guard was shown to fail before being trusted,
and the first falsification was thrown away for failing the wrong way:
removing
the guard entirely made the sheet builder crash on an undefined text,
so the test
went red without its assertion ever being reached.
Redone with a defined
placeholder,
it fails on the assertion itself.

STILL TO BUILD,
and it is the only thing left before the run:
`translate-probe`
is hardcoded to one entry with a fixed slice count and takes no arguments.
It
needs an entry plus slice list,
and a flag for the wider window.

### The plumbing `#108` needs is done, and here is the one file left

`settleTranslateSlice` now takes an optional `neighbouringSourceText` and passes
it to `runTranslateStage`,
which passes it to the judges as context they are not
asked to render.
Absent by default at every level,
so the lane behaves exactly as
it did and every measurement taken before tonight still describes what production
sends.

WHAT REMAINS IS ONE PROBE,
and writing it deliberately rather than at five in the
morning is the reason it is not written.
Twice tonight a rushed classification
shipped a design error that only an acceptance test caught,
both times by
deciding a slice's class from the slice alone;
the lesson is worth one night's
delay.

THE SPEC,
so the next session does not re-derive it:

1.  Walk the pinned corpus,
    run `classifyDisplacement` per entry,
    and collect
    every flagged slice with the class that flagged it.
    Reuse the probe's own
    reading rather than a copy of the thresholds.
2.  For each flagged slice,
    call `settleTranslateSlice` TWICE:
    once as it stands,
    once with `neighbouringSource({ slices, sliceIndex, },)` supplied.
    Same
    client,
    same rosters,
    same slice.
3.  Record per slice:
    the class,
    whether each arm replaced the archive text,
    and
    the decision the stage reported.
    `TranslateSliceRecord` already carries what
    is needed.
    Append each result durably AS IT LANDS,
    and on restart skip every
    (entry,
    slice,
    arm) triple already recorded.
    See what the cache does and
    does not do,
    below.
4.  Tally the replacement rate PER CLASS PER ARM.
5.  ON THE FIRST FLAGGED SLICE ONLY,
    before letting the remaining seventy-nine
    proceed,
    read the wide arm's judge sheets out of the run log and confirm
    they carry the `SURROUNDING ORIGINAL` label.
    This is a fail-fast guard on the
    live wiring,
    not the proof;
    the proof is now a test,
    below.
6.  DEDUPLICATE BEFORE COUNTING.
    Relocation candidates are ADJACENCIES,
    so one
    slice can appear in more than one candidate.
    The unit of trial is an
    (entry,
    slice position) pair;
    carry the candidate metadata alongside rather
    than running the same slice twice and counting the same model result twice.
7.  RUN MATCHED ORDINARY SLICES AS A NEGATIVE CONTROL,
    drawn from slices the
    screen did NOT flag.
    Without them a general context-induced conservatism,
    judges keeping the incumbent more often simply because they were shown more
    text,
    is indistinguishable from the window doing its job on relocations.
    The three flagged non-relocation classes are not this control:
    they are
    flagged too.
8.  REPORT PAIRED TRANSITIONS,
    replace-to-keep and keep-to-replace,
    rather than
    two aggregate rates.
    Two rates that match can hide equal traffic in both
    directions,
    which is not the window working.

THE HOP IS NOW PINNED BY A TEST,
which is what item 5 stopped having to carry.
`translate-slice.unit.test.ts` drives `settleTranslateSlice` with a recording
client and asserts the window reaches EVERY judge sheet,
NO translator sheet,
and neither when no caller supplies one.
Shown to fail:
deleting the forwarding
spread in `translate-slice.ts` fails it with `expected +0 to equal 5`,
which is
precisely the silent null the run would otherwise have bought.
A translator must
not see the window because one that rendered the neighbouring original would be
marked down for covering content that was never its slice.

THE CACHE QUESTION HAD TWO HALVES AND ONLY ONE IS ANSWERED.

THE HALF THAT IS ANSWERED was a real defect rather than a worry.
The key covered
models,
identity context,
source,
incumbent,
mode and the governance flag,
and
NOT the window,
so two arms would have shared a key.
The window is now part of
the key,
APPENDED ONLY WHEN PRESENT so a key computed without one is
byte-identical to what it always was and no settled slice in the corpus is
discarded.
That file had no tests at all;
it now has six.

THE HASHES THIS DOCUMENT QUOTED FOR THAT CHECK,
`0522d446...` and `09bc539a...`,
came from an ad-hoc fixture whose inputs were never written down,
so nobody could
reproduce them.
Against the fixture now pinned in
`translate-slice-key.unit.test.ts`,
absent,
explicit `undefined` and empty all
give `1f6e97d2...` and a supplied window gives `60a7a398...`.
The relationship
the old figures were quoted for held;
the figures themselves named nothing.

THE WINDOWLESS HASH IS NOW A LITERAL IN THE TEST,
because every other case in
that file compares two keys,
and a change that moved BOTH sides would pass all of
them while silently discarding every settled slice in the pinned corpus.
Shown to
fail:
appending one field to the serialized array fails the pinned hash and
nothing else in the file.

THE HALF THAT IS NOT is where that key is READ,
and this document previously
implied the fix alone de-risked the run.
It does not,
because the fix protects a
path the probe does not take.
The only call site of `translateSliceKey` is the
document driver,
`translate-document.ts:252`;
`settleTranslateSlice` imports no
store at all,
and its two mentions of caching are comments.
So a probe that
calls `settleTranslateSlice` directly,
which is what step 2 says,
consults no
cache.
The arms cannot collide,
which is the good half,
but nothing resumes
either:
at roughly fifteen hundred real calls,
a run that dies at hour three
restarts from zero.

The key change stays regardless.
It is correct for the driver path and for any
later windowing there,
and it cost four tests.
It simply is not the thing that
makes the probe survivable.

WHAT MAKES IT SURVIVABLE is step 3's own durable append,
and the infrastructure
for it already exists:
`openNamespacedCache` in
`src/corpus-run/slice-cache-namespace.ts` is generic over its stored value and
takes a lane prefix plus its own generation marker.

ONE TRAP IF THAT ROUTE IS TAKEN,
and it is the same neighbour-discriminator
error that has now bitten three times.
`CLAIMED_PREFIXES` in that file defines
the repair lane as everything NOT in the list.
A new lane that invents a prefix
without registering it there is therefore adopted by the repair lane,
whose
`discardNamespace` deletes it on the next generation change.
Registering the
prefix is what claims the files.
A plain append-only JSONL beside the run,
keyed by entry,
slice and arm,
avoids the question entirely and is enough for a
one-off measurement;
prefer it unless the probe needs something the lane
machinery gives.

### The two-arm design does not measure the window, and this is the blocker

FOUND BY REVIEW BEFORE THE RUN,
which is the second defect this probe has shed
without costing a call.
The spec above says to call `settleTranslateSlice` twice
per slice and compare.
Traced through `translate-stage.ts`,
that does not compare
what it claims to.

WHAT THE SECOND CALL ACTUALLY REDOES.
`runTranslateStage` gathers translator
voices at line 188,
repairs invalid candidates at 208 and builds the slate at
222.
The judges only see it at 382,
and the window is rendered into their
evidence at 410.
So the translators run AGAIN on the second call,
and they are
never shown the window.
The two arms therefore differ in two things at once:
the
evidence the judges read,
and a freshly sampled set of candidates for them to
read it against.
Any difference in replacement rate is divisible between them,
and nothing in the record says which.

AND THE JUDGES ARE STOCHASTIC TOO,
so the null this probe could report is not
"the window changed nothing" but "the change was smaller than a spread nobody
measured".
A two-arm design cannot separate those either.

WHAT THE DESIGN HAS TO BECOME,
in order of what each buys:

1.  A THIRD ARM IS REQUIRED whatever else changes:
    narrow,
    run twice.
    Its two
    readings are the run-to-run band,
    and the narrow-to-wide difference means
    nothing until it is bigger than that band.
    This is the positive-control
    discipline the displacement work already used,
    applied to a comparison
    rather than a detector.
2.  HOLDING THE SLATE FIXED is worth a refactor.
    Produce the candidates once per
    slice,
    then judge that exact slate three times.
    It removes the translator
    variance entirely,
    which makes the narrow pair measure judge noise alone,
    and it is CHEAPER:
    80 slices times four production calls plus three judgings
    of six is about 1760 exchanges,
    against roughly 2300 for three unpaired
    arms.
    The cost is splitting `runTranslateStage` into produce and judge
    halves,
    and that file is already 527 lines,
    so the split has to land as
    sibling modules rather than as one longer file.

RECOMMENDED:
do both,
in that order,
and treat the third arm as
non-negotiable.
Option 2 without option 1 still reports an unbanded difference;
option 1 without option 2 works but costs more calls and yields a noisier band.

WHAT THE PROBE MUST RECORD PER SLICE,
widened for the same reason.
A binary
replaced-or-not hides two things the review named:
two different fresh
candidates both count as "replaced" though the judges chose differently,
and the
alignment guard can refuse a wide-arm replacement so a real change in the ballot
lands as `changed: false`.
Record the chosen TEXT and the winning candidate's
producer alongside the flag,
or the tally cannot tell a window that changed the
verdict from one that changed nothing.

FOUR OUTCOMES,
NOT ONE FLAG,
since `changed: false` currently means any of them:
the judges kept the incumbent;
no usable voice answered;
a candidate happened to
equal the incumbent;
or the alignment guard refused a replacement the stage had
selected.
Record which.
Folding a degraded stage into "kept" is the reading that
makes a longer,
slower wide prompt look like a window that works.

AND REFUSE A DEGRADED ARM RATHER THAN RECORDING IT.
Direct settlement skips the
document driver's protections,
so a slice where no translator was heard still
returns an unchanged record and looks like an ordinary keep.
Give each arm its
own fresh signal of equal duration,
never one shared entry deadline whose
remaining time penalises whichever arm runs second,
require the intended
participation,
and record heard model ids and retry counts per arm.
Randomise
which arm runs first,
deterministically,
and run the two adjacent.

COUNT SEPARATELY,
do not fold into the rate:
slices where the stage returns
before judging at all.
An empty candidate set and a sole-incumbent slate both
return early,
so both arms agree trivially and neither read the window.
Folded
into the denominator they dilute the effect toward zero.

BUDGET:
roughly 80 flagged slices.
Two unpaired arms was 1500 exchanges;
the
three-arm paired design is about 1760 for the flagged slices ALONE.
The matched
ordinary controls are not in that figure and have to be added to it:
however many
are drawn,
each costs a full three-arm trial.
Every call is real,
so run it
detached and let it notify.

A WIDE ARM WITH AN EMPTY WINDOW IS NOT A WIDE TRIAL.
A lone slice legitimately
yields `''`,
the stage then renders the narrow sheet,
and the trial would record
a narrow judging in the wide column.
The early-return bucket does not catch it,
because the stage does judge.
Give those their own bucket and keep them out of
the wide denominator.

READING IT:
these instructions live HERE,
in this section,
and nowhere else.
They are not on any GitHub issue:
the numbering incident recorded below means
GitHub `#108` is a dependency audit,
and task-tracker item 108 carries a pointer
back to this section rather than a copy.
Per class;
near-floor candidates
hand-checked rather than counted;
an untranslated slice beside a high one treated
as a possible whole-section move;
and the two transcription suspects reported
apart.

THE ONE CODE STEP BEFORE ANY OF IT,
which nothing else in this document records.
`corpus-run/translate-probe.ts` is hardcoded to `PROBE_ENTRY = 'XingZ60'` with a
fixed slice count and takes no arguments at all.
It has to accept an entry plus a
slice list,
so it can be pointed at the flag list,
and a switch for supplying the
neighbouring source.
That is small,
and it is the only code step left before the
run other than the design changes named above.

### Document scale has no verdict yet

DOCUMENT SCALE IS STILL UNANSWERED,
and `#106` is where it belongs.
The slice
verdict exists and is cheap:
`classifyDisplacement` calls a slice `untranslated`
on source at least 150 characters against target at most 60,
deterministically
and with no model.
Four such slices in the pinned corpus,
all in `shi_Yumiaoya`,
at ratios near 0.01,
where the English side is a bare heading.
That is a FLOOR
rather than an estimate:
a section rendered as a heading plus a one-line stub
carries more than 60 target characters and is not counted.

What has no verdict is the DOCUMENT.
`shi_Yumiaoya` is 9795 Chinese characters
against 1630 English,
an aggregate of 0.31,
and `documentBaseline` already
reports which entries fall outside the believable band:
12 of 91 do.
Nothing
turns that reading into a stated verdict,
and no lane consumes any of it.
The
probe prints.

### Six comments were filed on the wrong tracker, and the numbering is why

RECORDED SO IT DOES NOT RECUR.
Everything in this document numbered `#60` to
`#108` is a LOCAL TASK TRACKER item.
Those numbers also exist as GitHub issues in
this repository,
and they are entirely different tickets:
`#84` is an oxlint
`ignorePatterns` workaround,
`#106` is CLI scaffolding,
`#107` and `#108` are
dependency audits.
Six comments about this work were posted onto those four
tickets overnight under the assumption that the numbers referred to GitHub.

The bodies were saved,
then each comment was rewritten in place to a short note
saying it was misfiled and where the work lives.
Rewritten rather than deleted
because deletion was refused,
which was the better outcome:
the correction is now
auditable rather than invisible.
Nothing was lost,
and the two findings those
comments carried that this document did not already hold,
the `PROBE_ENTRY`
step and the document-scale gap,
are recorded above.

ONLY `#431` AND `#432` ARE REAL GITHUB ISSUES for this work,
because they were
created here rather than referenced.
Anything else numbered in this document
belongs to the task tracker and must not be commented onto GitHub.
Before
posting to a GitHub issue from this work,
read its title back and confirm it
matches the subject.

### State the next session inherits, verified rather than assumed

MEASURED AT COMMIT `f44f82b60`,
not carried forward from earlier in the night:

-   `mise run //package/module/translation-repair:test:unit` (moved on 2026-10-18 to
    `//package/translation-repair/<package>:test:unit`) exits 0.
    The whole
    unit suite,
    not a selection:
    1104 lines of output,
    zero `[error]` lines,
    zero assertion failures.
-   `mise run //package/module/translation-repair:lint` (moved on 2026-10-18 to
    `//package/translation-repair/<package>:lint`) reports 0 warnings and
    0 errors.
-   Working tree is clean except four generated artifacts,
    described below.
-   `HEAD` equals `origin/translation-repair-rebased`;
    auto-push is doing its
    job and nothing is stranded locally.

FOUR FILES ARE DIRTY AND ARE NOT MINE.
Running `mise run file-enforcer` to
regenerate `CLAUDE.md` also rebuilt three plugin bundles and one jar:

    package/claude-code-plugin/bash-output-filter/bundle/node/index.mjs
    package/claude-code-plugin/guardrail/bundle/node/index.mjs
    package/claude-code-plugin/terminal-title/bundle/node/index.mjs
    package/intellij-plugin/islands-black/islands-black.jar

The change is real code rather than a timestamp:
a bundled shell lexer's
`if (lineEnd === -1 && ...)` became `if ((lineEnd === -1 || lineEnd > len) && ...)`,
a bounds fix that arrived with a dependency.
It is unrelated to this task,
so it
was left uncommitted rather than folded into a translation-repair commit or
reverted.
Someone should decide whether that dependency bump is wanted.

TWO `sol` REVIEWS ARE STILL ALIVE AND STILL EMPTY:
`bt8g6brhj` at 5 hours 18
minutes and `b64uex7px` at 3 hours 43 minutes,
each with 00:00:00 of CPU.
Their
siblings the same night finished in minutes.
Treat both as stalled,
do not kill
them without being asked,
and re-check any eventual output against current
source before acting:
both prompts predate tonight's changes.
