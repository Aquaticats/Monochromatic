# Runaway call termination: defects found by probing the detector on 2026-08-18

What probing the built detector and drain found on 2026-08-18:
the progress line that repeats one count,
long-period cycling that gets through,
a runaway error that names no model,
the retry layer that turns one runaway into five,
and the list of what was still owed.

Part of [Ending runaway calls on our side of the wire](translation-repair-runaway-call-termination.md),
which lists every file of this record.

## What the progress line still cannot say

TWO FIELDS CARRY THE SAME NUMBER.
`reportStreamProgress` prints `N raw chars` from the idle guard's counter
and `N delivered chars` from `partialText.length`,
and both count the raw event stream:
`drainBody` accumulates undecoded chunks and returns the body itself,
so the text it hands back IS the wire format.
Measured across the pass log,
235 of 235 progress lines have the two equal,
with no line where they differ.

THIS MATTERS BECAUSE THE SECOND FIELD READS AS THOUGH IT MEANT GENERATED TEXT,
which is the figure every open question here needs and the one nothing records.
The scanner already computes it:
`watchRunaway` feeds each channel's detector exactly the generated characters,
and the detectors already hold their own totals.
Only the reporting stops short.

THE OUTCOME LABEL ALSO CONFLATES TWO ENDINGS.
`StreamOutcome` is `completed` or `cut`,
and a call this guard deliberately terminates is reported as `cut`,
the same word a stall gets.
A reader counting cuts to measure stalls would count our own terminations among them,
which is the same mistake the sub-kind item guards against,
one layer further out.

## The detector catches short cycling and lets long cycling through

FOUND 2026-08-18 by probing the built detector directly,
with no rebuild and no quota.
The probe is kept at `~/temp/agent/degeneration-period-probe.mjs`.

WHAT DECIDES DETECTION IS ARITHMETIC,
not how repetitive the text is.
Windows are taken every 32 characters,
so for text repeating with period P
the sample can only ever hold P divided by the greatest common divisor of P and 32 distinct windows.
The predicted count matched the observed one in all fifteen cases tried.
A verdict of degenerate needs that count at or below about 409,
which makes the effective rule
"the repeated unit is shorter than roughly 410 characters",
loosened only when the unit happens to be a multiple of 32.

TWO NEIGHBOURING PERIODS LAND ON OPPOSITE SIDES:

```text
period  distinct  ratio   verdict
   409       409  0.0999  degenerate
   410       205  0.0500  degenerate
   500       125  0.0305  degenerate
   501       501  0.1223  healthy
   503       503  0.1228  healthy
  1001      1001  0.2444  healthy
  5000       625  0.1526  healthy
 20000       625  0.1526  healthy
```

A model looping a 501-character paragraph forever is called healthy,
while the same loop one character shorter is caught,
and nothing about the text is different in kind.

THE CONTROL RAN FIRST,
per `QPC`.
Text from the same generator that never repeats scores 1.0000 and reads healthy,
so the low ratios this probe printed are the loop rather than the fixture.
A first attempt at this probe padded a single sentence to length,
which made the block internally repetitive
and returned the same ratio for every period,
proving nothing;
that fixture was replaced before any of these numbers were read.

WHAT IS STILL COVERED,
and it is the case that was asked for:
the thinking runaway of the form "I will output.
I will output.",
at period 15,
scores 0.0037 and is refused.
Paragraph-scale looping is the gap,
and it costs exactly as much per hour as phrase-scale looping.

A FIX THAT DOES NOT DEPEND ON THE PERIOD.
Keep the trailing text itself,
and ask whether the most recent few thousand characters occur EARLIER in it.
A stream looping with any period shorter than the buffer answers yes,
whatever the period's arithmetic,
and ordinary prose never repeats a block that long verbatim.
It is one native substring search over a bounded buffer,
run occasionally rather than per chunk.

ONE HIT MUST NOT BE A VERDICT,
and this is the part that would go wrong if written quickly.
Reasoning traces in this pipeline restate whole source slices and whole candidates verbatim,
so a model quoting a long candidate twice inside one thinking trace is ORDINARY WORK
and a single recurrence would read as a loop.
Aborting on that would cost a voice for doing its job,
which is the harm the calibration section spent a whole document avoiding.
The verdict has to require the recurrence to PERSIST,
tripping on several consecutive checks as the stream grows,
which a genuine loop does forever and a quotation does once.

It complements the ratio rather than replacing it:
the ratio catches short cycling on a small sample,
the recurrence test catches the long periods the ratio cannot see.

## Two more things the reporting gets wrong, both shown at the boundary

DRIVEN THROUGH `drainBody` ITSELF rather than reasoned about,
by importing the built artifact and feeding it a thinking runaway.
What came back:

```text
[reportStreamProgress] stream hf:whiskers: cut, firstByte 12ms, maxGap 1ms,
  1376256 raw chars, 0 unreadable frames, 1376256 delivered chars,
  opening "data: {\"id\":\"chatcmpl-tabby\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index"

error message : ": ended a runaway call, reasoning channel repeated itself at 0.0037 distinct over 131475 characters"
carries label?: NO label property
```

THE RUNAWAY ERROR NAMES NOTHING.
It is constructed with `response.url` where every other path uses the `label` parameter,
and `label` exists precisely because the partial-text work found that attributing a stream to the endpoint
rather than to the model makes a per-model figure unreadable.
Here the message opens with a bare colon because a constructed `Response` has no url;
in production it would open with the chat-completions endpoint,
which is the same string for every model in the roster.
`StreamCutShortError` carries `label` as a property and this one carries none,
so which model ran away cannot be recovered from the error at all.

THE OPENING EXCERPT CANNOT SHOW WHAT THE MODEL SAID.
`stream-cut.ts` states the excerpt's purpose in as many words:
the opening tells a thinking block from an answer from an empty cut,
which is the whole diagnostic question.
It shows `partialText`,
and `partialText` is the raw event stream,
so the first 80 characters are always `data: {"id":"` and whatever envelope follows.
Not usually,
but in every case,
because every server-sent event body begins that way.
The id itself varies by sender:
the fixture this section quotes shows `chatcmpl-tabby`
and production shows a bare hexadecimal string,
so grep for the `data: {"id":` prefix rather than for either id.
The model's words are in there,
JSON-escaped and spread across frame boundaries,
and none of them are in the excerpt.

Both have the same root as the two equal counters:
what the drain hands back is the wire format,
and three separate readers treat it as though it were generated text.

## The guard ends one runaway and the retry layer starts four more

FOUND 2026-08-18 by driving `exchangeWithRetry` from the built artifact
with a transport that fails the way the drain fails.
No quota and no network.

`drainBody` cancels the reader and throws `StreamDegenerateError`.
It does NOT abort the caller's signal,
because the termination is ours rather than the caller's steering.
`attemptExchange` decides what is transient by reading `exchange.signal.aborted`,
which is false here,
so the runaway is captured as weather and re-dispatched.

AT THE PRODUCTION POLICY THAT IS FIVE CALLS,
not one:

```text
transport failure: StreamDegenerateError: ... ; retrying in  669ms (attempt 1 of 5)
transport failure: StreamDegenerateError: ... ; retrying in 1635ms (attempt 2 of 5)
transport failure: StreamDegenerateError: ... ; retrying in 2988ms (attempt 3 of 5)
transport failure: StreamDegenerateError: ... ; retrying in 6937ms (attempt 4 of 5)
transport called 5 times over 12242ms, ended as StreamDegenerateError
```

Each of those attempts would run until the model degenerates again,
which takes at least the 131000 generated characters a verdict needs,
so the guard built to stop wasted work MULTIPLIES IT BY FIVE
and adds twelve seconds of backoff on top.

THE DECISION DOCUMENT ALREADY STATED THE RULE and the code does the opposite:
a stall is worth retrying,
and a model that has begun repeating itself will repeat itself again.
That was written about how the loss is FILED.
The retry layer was already retrying it,
which is the same mistake one layer earlier and far more expensive.

CHECKED AND CLEARED at the same time,
so it is not re-investigated:
wrapping a failure in `StreamCutShortError` does NOT break caller-abort handling.
`attemptExchange` reads the SIGNAL rather than the error's identity,
so straggler abandonment and user steering still propagate untouched
however the drain has wrapped them.
A fixture whose signal is not aborted does see a cut retried,
but that is the fixture rather than production.

## What is still owed

-   NAME THE MODEL ON A RUNAWAY,
    which the error currently cannot do,
    and carry `label` as a property as the cut error already does.
-   SHOW GENERATED TEXT IN THE OPENING EXCERPT rather than the envelope,
    so the excerpt answers the question it was added to answer.
-   STOP THE RETRY LAYER RE-DISPATCHING A RUNAWAY,
    which today turns one into five.
    This is the most expensive of the open items and the cheapest to fix.
-   CATCH LONG-PERIOD LOOPING,
    which the distinct-window ratio cannot see.
    Held until the targeted pass releases the producing path.
-   REPORT GENERATED CHARACTERS PER CHANNEL on the progress line,
    replacing the field that repeats the raw count.
    Held until the targeted pass releases the producing path.
-   NAME OUR OWN TERMINATION in `StreamOutcome` rather than filing it as a cut,
    so a stall figure read off a log counts stalls.
-   RECORD THE NEW LOSS CAUSE with its own sub-kind,
    as voice loss already records one.
    A call ended this way must not be filed with a stall:
    a stall is worth retrying,
    and a model that has begun repeating itself will repeat itself again.
-   MEASURE THE ENVELOPE RATIO and revisit the 131000 character bar.
    The bar is set from artifact evidence
    because the only length telemetry in production counts raw event bytes,
    and the ratio to generated characters has never been measured.
    It is measurable today without touching the frozen path,
    by capturing one stream per model outside the pipeline
    and counting both sides with this package's own scanner.
-   VALIDATE AGAINST A REAL DEGENERATE REPLY.
    Partial-text retention landed,
    and an aborted call now keeps what it received,
    so the material exists as soon as one occurs,
    but none has occurred yet and the detector is still fixture-validated only.
