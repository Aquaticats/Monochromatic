# Translation repair history 2026-08-21 to 2026-08-23: why no runaway guard saw a 185 times emission

The classification of the missed emission as a gap rather than a regression,
three reasons every existing guard could not see it,
how the probe was validated,
the census that sets a source-relative bound,
the fix threaded along an existing path,
what the tests prove,
and the repair lane left uncovered.
The two finished tasks its opening names are not recorded in any other file of this record.

Part of [Translation repair history: 2026-08-21 to 2026-08-23](translation-repair-history-2026-08-21-to-23.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## #184: why no runaway guard saw a 185x emission

### The classification: a gap, not a regression

Both #119 and #120 are DONE and neither regressed.
Every guard they built is structurally unable to reach this event,
for three independent reasons,
each measured rather than read off the code.

FIRST,
LENGTH.
Both repetition detectors are gated behind `MIN_CHARS_FOR_VERDICT`,
which is `MIN_WINDOWS_FOR_VERDICT * WINDOW_STRIDE`,
131072 characters.
The volume bound from #156 is 32000.
The emission was 10381 characters:
7.9 percent of the repetition bar and 32.4 percent of the volume cap.
No guard could form a verdict at all.

SECOND,
SHAPE.
The guards detect a loop,
and this is not a loop.
It is one span of 5032 characters said exactly twice,
at offsets 135 and 5188,
covering 48.5 percent of the emission,
and then the generation stopped on its own.
Every detector rests on the premise stated in `stream-recurrence-watch.ts`:
"a genuine loop never stops,
so it always crosses the bar eventually."
This one stopped.

THIRD,
PHASE,
and this is the finding worth keeping.
The ratio detector samples 64-character windows on a 32-character grid.
The duplication's period is 5053,
which is not a multiple of 32,
so the two copies never land on the same grid offsets.
Measured on the real emission:

    stride 32, the detector's own grid   323 windows, distinct 1.0000, none seen twice
    stride  1, every offset            10318 windows, distinct 0.5184, 4969 seen twice

The duplication is plainly there at stride 1 and perfectly invisible at stride 32.
Lowering the length bar would not have helped:
the detector would still have called this text healthy at any length,
because on its own sampling grid the text genuinely never repeats.

The recurrence detector misses it for a related reason of its own.
Its comment guarantees a sliding search only for periods up to 2048,
the room left in the buffer,
and its decision record accepts
"any period shorter than the buffer", 4096.
The [1024,
3072] interval in the same comment bounds something else:
the LENGTH of a back-to-back duplicated block that can produce hits.
A period of 5053 is past all three numbers,
so the earlier copy has scrolled out of the buffer before the later one arrives.

### How the probe was validated before any of that was believed

The first run reported "neither detector fires" and the positive control ALSO failed,
which under QPC means the run said nothing at all.
Two controls were added before any conclusion was drawn:
a tight loop,
which flags by recurrence at exactly 131072 characters,
and varied prose,
which stays healthy at 1173015 characters.
Only then did the null on the real text become evidence.

A second null was caught the same way.
`countPhrases` was called with `phraseWords` where the parameter is `length`,
and returned an empty map for every input,
including the document
already known to carry 871 repeated phrases.
Zero everywhere is a harness result,
not a measurement.

### One hypothesis raised and refuted

The word instrument sees the duplication clearly:
1800 words,
918 distinct 12-word phrases,
871 of them repeated.
The obvious explanation was that the two copies differ in whitespace,
so words match while characters do not.
Flattening every whitespace run to one space and re-measuring refutes it:
distinct stays at 1.0000 and no period appears.
The cause is phase,
not whitespace.

### The seam the fix needs already exists and is deliberately unused

`watchRunaway` accepts `contentCap`,
defaulted to `CONTENT_OVERRUN_CAP`.
Nothing in the pipeline passes one.
`stream-drain.ts` says so in as many words,
and gives the reason:
threading it "would mean a parameter through this function and every caller,
and the measurement says one bound clears every role by more than twice."

That rationale answers a different question than the one #184 asks.
It is about whether a ROLE that legitimately emits more should raise its own bound.
It is not about a bound relative to the SOURCE,
which is the only kind that can tell a long legitimate passage
from a 56-character line that produced 10381.

### What that settles of the four questions the task said to answer first

MID-STREAM OR AFTER:
mid-stream,
and the mechanism is already there.
The watch already counts produced characters per channel
and already accepts a per-call bound,
so a producer that knows its source length
can pass `sourceChars * limit` and nothing on the retry path changes.
That matters because #120 found six defects in the retry layer,
and this shape does not touch it.

WHAT RATIO:
`sliceImplausibility` already answers this and is already tested.
`IMPLAUSIBLE_MAX_RATIO` is 10,
the largest legitimate observed was 9.27,
and this emission is 185.4,
past the endpoint by more than eighteen times.
The predicate compares `targetChars / sourceChars` on the original,
which is exactly the comparison a producer-side bound needs.

STILL OPEN:
what happens after refusal,
and whether the multiplier
used for a live bound should equal the endpoint used for a settled judgment.
A live bound cuts a call that might still have recovered,
while the settled predicate judges finished text,
so they need not be the same number.

### The census that sets the bound

Measured over every artifact in all 22 run directories:
947 candidate emissions,
772 of them freshly produced,
each against `alignment.sourceCodePoints`,
which is the denominator a live guard has at call time.
The repair lane's slices carry no alignment record,
so this covers the translate lane only.

BY SOURCE LENGTH,
the tail is entirely a short-source effect:

    source   0..64    n=340  p50 3.31  p90 19.24  max 270.02
    source  64..128   n=379  p50 3.17  p90  4.70  max  96.13
    source 128..256   n=185  p50 2.92  p90  3.81  max  17.63
    source 256..1024  n= 43  p50 3.75  p90  4.09  max   4.36

The median barely moves,
2.92 to 3.75 across every bucket:
a correct translation is about three times its Chinese source at any length.
Only the tail moves,
and only where the denominator is small enough
for one runaway to produce an enormous quotient.

THE LEGITIMATE MAXIMUM,
taken as the largest ratio the judges actually SELECTED,
because a candidate that lost is not evidence of a legitimate size.
Among 144 shipped fresh emissions:

    270.0x  source  41  produced 11071   vub171-20260822/Zha_Ke c1
    183.2x  source  56  produced 10259   vub171-20260822/Zha_Ke c2
    136.0x  source  41  produced  5576   readable-20260820/Zha_Ke c1
    135.6x  source  41  produced  5559   vub-run1-20260821/Zha_Ke c1
     99.8x  source  56  produced  5591   vub-run1-20260821/Zha_Ke c2
     98.2x  source  56  produced  5497   readable-20260820/Zha_Ke c2
      9.4x  source 232  produced  2170   163b-verify/dogesir_ c2
      5.2x  source  64  produced   335   vub-run1-20260821/Acheron c1

Six runaways,
then a full order of magnitude of empty space,
then everything else.
The largest legitimate shipped ratio is 9.4,
which independently corroborates the 9.27 recorded in `contest-size-note.ts`
from a different census over 11 settled artifacts.
Two measurements taken different ways agree on where legitimate output stops.

### What bound that supports

Scored as `produced > max(floor, source * K)` against the shipped fresh emissions:

    floor  512  K  8   cuts 7 of 144 shipped,  44 of 772 produced
    floor  512  K 12   cuts 6 of 144 shipped,  43 of 772 produced
    floor 1024  K 16   cuts 6 of 144 shipped,  39 of 772 produced
    floor 1024  K 24   cuts 6 of 144 shipped,  32 of 772 produced
    floor 2048  K 16   cuts 6 of 144 shipped,  36 of 772 produced

Every setting from K 12 to K 24 cuts the same six,
and those six are the runaways.
The bound is insensitive across that whole range,
which is the same flat-tail argument `MAX_INCUMBENT_TO_SOURCE_RATIO` was set on.

CHOSEN:
floor 1024,
K 16.
K 16 because the codebase already carries 16 for the sibling incumbent-to-source ratio,
so the pipeline holds one number rather than two that mean nearly the same thing.
It sits above the settled endpoint of 10 on purpose:
a live cut ends a call that might still have recovered,
while `sliceImplausibility` judges text that is already finished,
so the live bound must be the looser of the two.
Floor 1024 keeps the bound away from short sources,
where three times a 22-character heading is 66 characters
and any proportional bound would be far too tight to be safe.

At that setting the recorded event is cut at 1024 characters rather than 10259,
which is where the saving is.

### Three ratio instruments already exist, and none watches production

    MAX_INCUMBENT_TO_SOURCE_RATIO  16     incumbent vs source, guards PAIRING, runs after the stage
    IMPLAUSIBLE_MAX_RATIO          10     target vs source, judges SETTLED text for the size note
    CONTENT_OVERRUN_CAP            32000  absolute, live, per call

The missing one is produced against source,
live,
per call.
That is what #184 adds,
and the three neighbours are what calibrate it.

### What happens after a refusal, settled by what already happens

No new policy is needed.
`stage-round.ts` turns any stream error into `voice: { heard: false }`,
a silence recorded for that model,
and the ensemble absorbs it.
An overrun is already handled this way,
so a source-relative cut costs the offending model its candidate for that slice and nothing else.
#88's hand-back is a different level:
it covers a slice the whole stage returned invalid,
not one voice among several.

### The fix, and where it was threaded

`produced-volume-bound.ts` holds the bound and what it rests on.
The translate producer computes `producedVolumeBound({ sourceChars })` for each slice
and hands it to the call as `maxAnswerChars`.

THREADED ALONG THE PATH `exchangeTimeoutMs` ALREADY TRAVELS,
which is what keeps this small and keeps it away from the retry layer:
producer,
`stage-quorum`,
`stage-round`,
`stage-call`,
the client,
the transport,
the drain.
Optional at every hop,
so a caller that knows nothing about its input size
is policed at the module default exactly as before.
Nothing on the retry path changed,
which matters because #120 found six defects there.

The `stream-drain.ts` comment that explained why nothing passed a bound
is rewritten in the same commit,
so the code no longer contradicts its own record.

### What the tests prove, and how they could fail

Both controls ride in one run over the same fixture:
an 8192-character answer,
well under the 32000 default so only a per-call bound can end it.

    with maxAnswerChars 2048   overrun at 2210 content chars, 4608 raw chars pulled
    with no bound named        completed, 8590 content chars, 17522 raw chars pulled

The bounded call ends having pulled a quarter of the bytes,
and the unbounded one drains the identical fixture whole,
so the refusal is the bound doing the work rather than anything about the fixture.
That case proves the LAST hop and no other.
It hands `maxAnswerChars` straight to `drainBody`,
so it says nothing about whether the producer computes a bound,
nor whether the hops between forward one.
A sentence claiming otherwise stood here and was wrong.
`#185` built the test that earns the claim
and recorded which hops each test actually pins.

GATE:
suite 536 PASS / 0 FAIL / exit 0,
`lint` 0 warnings 0 errors,
`lint:types` exit 0.

### What this does not cover

The repair lane.
Its slices carry no alignment record,
so the census could not measure a produced-to-incumbent ratio for it
and no bound is passed on its calls.
Every observed runaway was in the translate lane,
so this is a scope statement rather than a known gap,
but a repair-lane runaway would still be policed only by the absolute default.
