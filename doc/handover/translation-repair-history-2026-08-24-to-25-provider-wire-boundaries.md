# Translation repair history 2026-08-24 to 2026-08-25: what the providers send and how the pipeline reads it

The first reading of the zero-content anomaly on the deepseek seat and why it was superseded,
the finding that the response schema already reaches Charm Hyper and the client seam built for Synthetic,
the truncated stream that arrived as a success status and never retried,
and the duplicated block index on the wire that filed ballots as reasoning.

Part of [Translation repair history: 2026-08-24 to 2026-08-25](translation-repair-history-2026-08-24-to-25.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## SUPERSEDED: the deepseek zero-content anomaly, first reading (2026-08-25)

RESOLVED LATER THE SAME DAY.
The heading this section used to carry said the anomaly was not
resolvable from the log.
That is no longer true,
and the resolution is in
`doc/audit/every-volume-guard-is-blind-to-one-model.md` and in the section named
"Two more findings from the same run".
The counts here were right;
what was wrong was the conclusion drawn from them.

WHAT THIS SECTION STILL GETS RIGHT,
and why it is kept rather than deleted:

-   The method lesson.
    The instrument used for CAUSE here was a 40-line scan
    window,
    and a positive control on `hf:openai/gpt-oss-120b` showed it fired on
    39 percent of healthy calls.
    Nine samples against that baseline separate
    nothing.
    That reasoning was correct and is worth keeping.
-   The `hf:zai-org/GLM-5.2` distinction,
    which the later measurement depends on:
    its zero-content count EQUALS its cut count,
    so those zeros are unfinished
    calls rather than the `#211` signature.
    The later count reads only COMPLETED
    streams,
    which is why GLM shows zero there and why
    `deepseek-v4-pro-0813`,
    with 12 zero-content against 0 cuts,
    is the real
    anomaly.

Re-measured over the calibration's first 13 slices,
1176 stream lines parsed and none skipped.

### What the counts say

-   `qwen3.8-max`:
    100 calls,
    98 zero-content,
    17 cut.
    So 81 COMPLETED calls counted nothing.
-   `deepseek-v4-pro-0813`:
    101 calls,
    9 zero-content,
    0 cut.
-   `hf:zai-org/GLM-5.2`:
    100 calls,
    10 zero-content,
    10 cut.
    Zero-content EQUALS cut count.
-   `hf:Qwen/Qwen3.8-27B`:
    2 and 2.
    Every other seat:
    zero and zero.

GLM-5.2 and Qwen3.8-27B are the ordinary case,
streams that ended before content arrived.
GLM-5.2's ten carry a median of 2.3 million raw characters,
which is runaway reasoning meeting the straggler deadline,
not a parsing fault.
Every zero-content call on every seat carries reasoning above zero,
so none of them is a genuinely empty reply.

### Why the log cannot settle deepseek's nine

The only correlate available is whether a voice from that seat appears near the stream line.
That instrument has a measured baseline of 39 percent on HEALTHY calls:
`hf:openai/gpt-oss-120b` has 102 completed calls with content
and only 40 of them are followed by a voice within ten seconds,
because most streams are editor or refiner production calls that produce no ballot line at all.

Against that baseline:

-   `qwen3.8-max`'s zero-content calls score 35 percent,
    indistinguishable from healthy.
    That is independent confirmation of `#211`:
    its answers arrived,
    only the counter was fooled.
-   `deepseek-v4-pro-0813`'s nine score 11 percent,
    where the baseline predicts about three and a half.
    On nine samples that separates nothing.

An earlier reading of this scanned forty lines after each occurrence and reported the seat present in six of nine.
That was wrong:
the window spanned neighbouring calls and was catching another round's voice.

### What answers it, for free

The candidate ledger from `#212` records which model produced each candidate
and which cast each ballot,
with no timing correlation involved.
The first run carrying a ledger answers this without a single extra call.
Until then the honest state is:
measured,
unexplained,
nine events,
and not folded into `#211`.

## `#216` was half wrong, and reading the source before building found it (2026-08-25)

`#216` was opened on the finding that seventeen modules build a `role: 'system'` message
and not one of them puts its response schema into that message.
That is true of the PROMPT BUILDERS and false of what a model actually receives on Charm Hyper.

`anthropic-request.ts` routes every schema-bearing call through `renderToolSystemPrompt`,
which prints the whole schema into the Anthropic `system` field under
"THE EXACT SCHEMA OF THAT OBJECT",
followed by seven format rules.
One of those rules reads
"Pass the object itself.
Do not pass a string that contains JSON,
and do not escape its braces",
which is exactly the failure `#216` cited as evidence that the schema was missing.

So the correlation runs the opposite way to the mechanism the task assumed:

```text
Charm Hyper   schema IS in the system prompt    6 of 6 measured schema failures
Synthetic     schema is NOT in the prompt       0 measured schema failures
```

The owner's instruction was already implemented on the provider where the failures are.
This is worth stating plainly because the task's own evidence section reads as though
it were about to conclude the opposite,
and a later session would have believed it.

### What was genuinely missing

The Synthetic path had nothing of the kind.
`synthetic-client.ts` builds an OpenAI-compatible body carrying only the API-level
`response_format` field,
which a model that does not honour it never sees.
That is a real gap against the instruction,
and it is the half that was built.

`src/schema-prompt.ts` renders a block from the same `JsonSchemaResponseFormat`
the request puts on the wire,
so the prompt and the wire cannot drift.
It is idempotent,
it adds a system message where a call has none
rather than dropping the schema on the calls that state least,
and it handles a system message carrying parts
so a call that also sends a picture is not the one that loses its schema.
Its rules list is carried over from the Anthropic renderer,
which is the wording this codebase has already run in production.

`hyper-client.ts` is deliberately unchanged,
with a comment at the seam saying why.
An edit there was written and then reverted:
it would have stated the schema twice on every Hyper call.

### Why the client seam rather than the seventeen prompts

The task proposed editing each prompt builder to append a rendered block.
The seam is strictly better.
It derives the text from the exact value going on the wire,
it covers every caller that exists and every caller written later,
and there is no eighteenth prompt to remember.

### What is owed, stated as an open question rather than a prediction

The after-measurement.
Adding schema text lengthens every Synthetic system prompt,
which costs tokens
and could hurt as well as help.
Synthetic's schema-failure count is already zero,
so this change CANNOT be validated by that number falling.
What it can be measured on is a first Synthetic schema failure never appearing under load,
and the token cost,
which the `#210` spend ledger now makes readable.
Record the outcome;
do not claim an improvement.

## A stream the provider cut short was the one transport failure that never retried (`#228`, 2026-08-25)

Found by reading the running calibration's warnings rather than by suspecting anything:

```text
      4  MalformedCompletionError
      2  SyntaxError
```

All four are `panel gemma-4-26b-a4b-it`,
all four say
`anthropic stream ended without message_stop, voice lost`.
The two `SyntaxError` are the critic schema mismatches already recorded.
With the 13 straggler cuts in the same log,
the three classes account for every voice
the run never heard,
which is how this class was separated from the other two at all.

### Why it never retried, which is more interesting than that it did not

`hyper-client.ts` and `synthetic-client.ts` both call `exchangeWithRetry`
and then extract the completion AFTER it returns.
A truncated body arrives as HTTP 200.
That status is not in `RETRYABLE_STATUSES`,
so the ladder hands the reply straight back,
and extraction throws with nothing left that could re-dispatch it.

Every other transport failure carries a status the ladder recognises.
This one wears a success status,
which is exactly why it was invisible:
the ladder was working correctly and the failure was outside it.

### The fix follows a design that was already there

`attemptExchange` catches a thrown failure and asks ONE predicate,
`isSelfEndedStream`,
whether we caused it,
so that a runaway we cut is never re-dispatched.
Running the caller's read inside that same `try` puts a truncated body on exactly that path:
same catch,
same predicate,
same ladder.

So the terminator pass moved out of each extractor into an exported check,
`requireAnthropicTerminator` and `requireStreamTerminator`,
and `attemptExchange` gained an optional `verify` it runs before returning.

RULED OUT FIRST,
because retrying our own deliberate cut would be a bug:
the guards throw their own classes,
`StreamStalledError` and `StreamCutShortError`,
and an aborted drain THROWS with `partialText` rather than RETURNING a short body.
The only `return bodyText` in `stream-drain.ts` is the normal completion path.

### Measured from source, without a build

`dist/final/node` is held by the calibration,
so the boundary proof ran under
`node --experimental-strip-types` against `src/`,
with a fake transport and no key.
Both providers,
four cells each:

```text
  cut every time        attempts=5  MalformedCompletionError
  cut once, then whole  attempts=2  answered
  whole every time      attempts=1  answered
  HTTP 400 error page   attempts=1  SyntheticHttpError
```

`SyntheticHttpError` in that output is the class renamed `ProviderHttpError` on 2026-10-19.

Row two is the voice that used to be lost.
Rows three and four are the controls,
and row four is the one worth naming:
`wholeMessage` checks `isSuccessStatus` first,
so a non-success reply is never read as a stream.
Without that guard a 400 carrying an error page would report a parse failure
instead of the HTTP code,
which sends a reader to the prompt rather than to the provider.

### What the suites now assert, and what is still owed

Landed in `38a5178d7` (fix) and `05928328e` (tests).
Each client suite gained the two cases,
and the Hyper suite's existing terminator test
gained the attempt count it had been missing:
it ran on the production ladder,
paying about seven seconds of real backoff
to assert something the count now states outright.

The third path needs no new test.
Each suite already refuses a non-success reply whose body carries no terminator,
`out of credits` and `{"error":"slow down"}`,
so removing the `isSuccessStatus` guard inside `wholeMessage`
turns both into a parse failure about an error page.

NOT YET RUN.
The suites import `../dist/final/node/index.mjs`,
and rebuilding would swap the bundle out from under the pass in flight.
Owed at the same moment as `#224` and `#225`:
build once,
run the suites,
then prove each guard by removing it.

## `#211` is proved at the wire, and the fix is in (2026-08-25)

One call to `qwen3.8-max` on Charm Hyper,
shaped exactly like a production ballot request,
with the untouched SSE bytes kept at `~/temp/agent/capture-211.sse`.
HTTP 200,
17612 raw characters,
128 frames.

### What the provider actually sends

The diagnosis guessed the model declares a thinking block carrying its answer deltas.
The wire is narrower and stranger than that:

```text
content_block_start  index 0  {"type":"thinking"}
  ... 106 thinking_delta frames ...
content_block_stop   index 0
content_block_start  index 1  {"type":"tool_use","name":"candidate_ballot"}
content_block_start  index 1  {"type":"thinking"}          <- SAME INDEX, no stop between
  ... thinking_delta and input_json_delta interleaved, 17 of the latter ...
```

The provider opens index 1 as a tool call and then opens THE SAME INDEX again as thinking.
`openBlock` in `anthropic-delta-scan.ts` sets the block map unconditionally,
so the later declaration wins,
and `channelFor` then files every index 1 delta as reasoning,
including the `input_json_delta` frames carrying `{"best": 2 ...`,
which is the ballot.

That is the whole of the 70-zero-content-against-71-ballots signature.
The extractor in `anthropic-completion.ts` ignores blocks,
so it recovers the answer and the vote lands;
only the scanner that feeds the progress line and the runaway guard is fooled.

### The fix, and why this shape rather than the other one

`ANSWER_DELTAS` in `anthropic-delta-scan.ts` now exempts `input_json_delta`
from the thinking-block override.
A tool-call argument fragment cannot be deliberation:
it is the structured answer by construction,
filling a schema this pipeline sent.
`text_delta` is deliberately NOT exempt,
so the case the override was added for,
plain text deltas inside a thinking block,
still routes to reasoning.

The alternative was to keep the FIRST declaration in the block map.
That also routes this capture correctly,
but only because `tool_use` happened to arrive first.
The chosen shape holds whichever order the two declarations come in.

GFP-proven:
removing the carve-out turns the new case red,
restoring it turns it green.
The test fixture is the captured frame order,
duplicate `content_block_start` included.

### Measured on the captured bytes, before against after

The same 17612 characters replayed through the same scanner,
the carve-out being the only difference:

-   Without it:
    `content 0 chars, reasoning 1488 chars, unreadable 0`.
-   With it:
    `content 218 chars, reasoning 1270 chars, unreadable 0`.

The before state reproduces the production symptom exactly,
which is the zero content chars the log reports for 98 of this seat's 100 calls.
The 218 characters that move are the ballot,
and 218 plus 1270 equals 1488,
so nothing was invented or dropped:
it was only filed under the wrong heading.

### What it should buy on the next run

`stream-runaway-watch.ts` bounds the content channel and leaves reasoning alone,
so an answer filed as reasoning escaped the volume cap and ran to the straggler deadline.
`qwen3.8-max` was cut 12 times in 71,
the highest on the roster by two and a half times.
The prediction is that its cut rate falls toward the roster's.
NOT YET MEASURED:
the run in flight predates the fix.

### State

Parked with `#210` and `#212` in `~/temp/agent/spend-telemetry-210.tar.gz`,
now thirty-three files.
Lint clean,
types clean,
663 suites passing,
zero failures.
