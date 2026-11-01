# Audit ledger: provider streams, usage counts and retries

Provider fields read otherwise than the provider documents them:
stream events,
usage counts,
the refusals a retry decision reads,
and the rounds that waited on a stream that sent only comment lines or on a seat abandoned again and again.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B87: an Anthropic stream's ending read off a word, and its error event read as a cut stream

Found in T8's twenty-second batch,
reading `anthropic-completion.ts` against the current streaming documentation
(platform.claude.com,
"Event types"),
red in `57ffcf2dd`,
fixed in the commit adding this entry.
`requireAnthropicTerminator` counted a body as whole when any `data:` payload held the word `message_stop` in quotes.
A body cut inside its last frame,
after `{"type":"message_stop"` and before the closing brace,
passed:
the retry ladder (`wholeMessage` in `hyper-client.ts`) returned it,
and `extractAnthropicCompletion` then refused it as a frame that is not JSON,
after the ladder,
so the call was lost with retries left.
A frame holding that word as a value
(a tool's name)
passed too,
and on a stream cut before its end handed the fold half an answer.
The stream's error event,
which the documentation shows in place of the rest of a failing message,
was not read at all:
with no terminator after it,
the refusal named a cut connection;
with one after it,
the fold returned what came before as the answer.

`requireWholeAnthropicMessage` replaces it,
since moved with the frame reading into `anthropic-whole-message.ts`,
which has its own tests,
when the B89 fix took `anthropic-completion.ts` past the line limit.
Each payload is read as a frame by `readFrame`,
which the fold shares;
the terminator is a frame whose `type` is `message_stop`,
and an error event refuses the message wherever it stands.
The refusal names the error's type when that type is a protocol word
(lower-case ASCII letters and underscores,
at most 64 characters),
and "unnamed" otherwise,
since the error class promises to quote nothing from the body.

The B23 census of fixed-word searches listed this site
(`b23-needle-census.txt`,
`anthropic-completion.ts:428`)
and recorded no class for it.
Its quotes bound the word,
so as a word search it read as bounded;
the defect was of another kind,
a frame's type read off its text instead of its parse.

Calls made here are open to veto:

- only the ending is read before the ladder returns:
  a body whose terminator arrived was delivered whole,
  and a frame inside it that does not parse is refused by the fold,
  unretried,
  read as a formatting defect a retry would repeat and pay for again
  (an inference,
  not measured);
  `requireStreamTerminator`,
  the OpenAI-shaped sibling,
  reads the same scope;
- an error event is retried as a cut stream is.

Open when this entry was written,
closed in B202:
an error the provider calls `invalid_request_error` is retried as an overloaded one is,
spending attempts that cannot succeed.

Recurrence:
which frame a payload is gets read off its parse;
`mistake-prevention.md`,
"Structure read off the parse".

## B88: prompt tokens that left out the cache

Found in T8's twenty-second batch,
reading `foldUsage` against the prompt-caching documentation
(platform.claude.com),
red in `9a885953d`,
fixed in the commit adding this entry.
Anthropic's `input_tokens` counts only the tokens after the last cache breakpoint;
the documentation gives the prompt as `cache_read_input_tokens + cache_creation_input_tokens + input_tokens`.
`extractAnthropicCompletion` reported `input_tokens` alone as `prompt_tokens`,
so a call that read or wrote the cache was counted,
and priced,
as a short one.
The fold now keeps each usage field's last report (`USAGE_SERIES`),
a null read as absent,
and the prompt is the sum of the three.

Reach:
only `hyper-client.ts` calls this reader in production,
and Charm Hyper's captured stream (2026-08-24) carries no cache field,
so production counts do not move.
The one capture that does carry one is OpenRouter's Messages endpoint
(2026-09-03,
measurement only):
its expected prompt count moved from 574 to 702.
Whether that gateway's `input_tokens` already counts cached tokens was not settled:
today's DigitalOcean prices for the model matched neither reading of the capture's own cost.
The payload store keeps only the extracted counts,
so no stored reply carries a cache field to replay.
`corpus-run/hyper-price.ts` prices every prompt token at the input rate,
so a cached call is now over-estimated where it was under-estimated;
its cache rates are recorded and unused.

Calls made here are open to veto:

- the documented sum is followed,
  though the one gateway capture carrying a cache count could not be checked against it.

Recurrence:
a provider's field is read as its documentation defines it;
`mistake-prevention.md`,
"Provider fields read by their documentation".

## B89: Anthropic answer fragments dropped without a word

Found in T8's twenty-second batch,
reading the stretch census-lxT62v lists cold at `anthropic-completion.ts:242`
(a `content_block_delta` frame with no delta returned nothing)
against the streaming documentation,
red in `7b5a9d2ac`,
fixed in the commit adding this entry.
The documentation gives every `content_block_delta` frame a `delta`,
every `text_delta` a `text` and every `input_json_delta` a `partial_json`.
`foldDelta` passed over a frame with no delta,
and `stringField` read a text or `partial_json` that was not a string as empty,
so the answer came back with a piece missing and nothing said;
a prose answer missing words would then be judged and placed as whole.
The same reader refuses a frame that is not JSON,
on the grounds that the answer assembled from the rest is missing an unknown piece,
and a frame that parses but lacks its fragment is the same case.
Now `foldDelta` refuses the first,
and `answerFragmentOf` the second,
each with a `MalformedCompletionError` naming the field.
A delta of a type the fold does not read,
and one naming no type,
are still passed over:
either may be a type the provider added later,
which the documentation asks a reader to handle gracefully.

The B29 census of silent fallbacks read nullish fallbacks,
parameter defaults and caught errors;
a reader returning an empty string for a field of the wrong type is none of those,
so this file's `stringField` was not in its reach.

Recurrence:
a frame that parses but lacks the fragment its kind carries is refused as a frame that does not parse is;
`mistake-prevention.md`,
"Provider fields read by their documentation".

## B91: a delta type the Anthropic scanner does not read, claimed as reasoning in a thinking block

Found in T8's twenty-second batch,
reading the stretch census-lxT62v lists cold at `anthropic-delta-scan.ts:450`
(`DELTA_TEXT_FIELDS.get(deltaType) ?? ''`),
red in `a1761139e`,
fixed in the commit adding this entry.
`channelFor` returned `reasoning` for every delta type inside a thinking block,
the types the scanner does not read included,
and the text was then read from the field the second table named,
or from the field named by the empty string where it named none.
A `signature_delta`,
which the streaming documentation sends at the end of every thinking block,
reached that fallback;
it yields nothing only because it carries no field named by the empty string.
The two tables were kept apart on a claim their own note made,
that one table could not hold two types sharing a channel under different fields;
a table of pairs holds them.
`DELTA_READINGS` now pairs each type's channel with its field,
and `routingFor` returns a type the scanner does not read as unread before a block can lend it a channel.
The same commit reads the scanner's held-back line with `nonNullishOrThrow`:
a split yields one piece at least,
so `lines.pop() ?? ''`,
cold at line 524 in census-lxT62v,
named a case that cannot happen.

Recurrence:
a table split in two lets one half answer where the other has nothing;
a note claiming a design is forced is checked against the smallest table that would hold it.

## B93: delta frames the Anthropic scanner could not read, passed over uncounted

Found in T8's twenty-second batch,
writing a case for the stretch census-lxT62v lists cold at `anthropic-delta-scan.ts:406`
(a `content_block_delta` frame with no delta returned nothing),
red in `ac4980b81`,
`840a41698` and `5a28319ed`,
fixed in the commit adding this entry.
The `DeltaScanner` contract in `stream-delta-scan.ts` describes `unreadableFrames` as payload lines that could not be read,
which should stay at zero.
The Anthropic scanner counted only a payload that did not parse
or parsed to something other than an object.
A delta frame with no delta object returned nothing,
and a delta of a type the scanner reads whose text field was absent or not a string
was read through `stringField` as empty and dropped,
so the tally stayed at zero for frames the completion reader refuses (B89).
`stream-drain.ts` writes that tally into the progress line it logs when a stream completes,
and calls no completion reader
(a search of it for the reader names exits 1);
`hyper-client.ts` reads the body with `extractAnthropicCompletion` after the retry ladder returns,
so a body the reader then refused was logged with no unreadable frame.
Now `readDelta` counts both,
reading the routed field's raw value with a type check
in place of `stringField`,
which cannot tell an absent field from an empty one.
The count follows what the scanner reads,
not what the reader refuses:
a `thinking_delta` with no `thinking` string is counted,
since the scanner files that channel,
although the reader passes thinking over.
A block start is read for its declaration alone,
so one naming no index,
carrying no block,
or whose block names no type string
yielded nothing it was read for and is counted too;
the reader folding no block start has no bearing on this tally.
The same red cases found that such a start at an index already declared
kept the earlier declaration when it carried no block,
and forgot it when its type was not a string:
the later block's deltas were read under the block before it in the first case.
A start at an index now supersedes the declaration there whether or not it can be read,
as a readable restart already did
(the 2026-08-25 capture noted at `ANSWER_DELTAS`).
An empty fragment is read,
not unreadable,
a delta frame naming no index is read by its own type,
and a type the scanner does not read is passed over whatever it holds,
as the streaming documentation asks of types added later.

Recurrence:
each reader of a wire counts or refuses a frame it reads and cannot,
judged by what it reads that frame for;
`mistake-prevention.md`,
"Provider fields read by their documentation".

## B94: Anthropic usage reported a zero for a count the stream never sent

Found in T8's twenty-second batch,
reviewing the lenient-arm case committed in `08635488c`,
which expected `prompt_tokens` 0 from a body that reported only `output_tokens`;
red in `4e82b6d7a`,
fixed in the commit adding this entry.
`usageOf` reported usage once any count series held a report,
and `latestOf` read a series never reported as 0,
so a stream that sent its completion count alone
came back with a prompt count of 0 that nobody measured,
and one that sent its prompt count alone
came back with a completion count of 0.
The same file's case for a stream reporting no count at all
says why that matters:
a reader must tell a call that cost nothing from one whose cost went unreported.
`readUsage` in `completion-shape.ts`,
the OpenAI-compatible reader,
reports usage only when both counts are numbers.
Now `usageOf` reports usage only when the stream reported both its `input_tokens` and its `output_tokens`,
and logs which one was missing when it reported one alone;
a cache series still reads as 0 when absent,
since a stream reporting no cache field used no cache this reader can count.

Reach:
`spend-line.ts` prints `prompt=` and `completion=` as unreported when usage is absent,
so a stream reporting one count alone now prints both as unreported
where it printed the one count and a 0.
The two captured streams the unit suite reads
(Charm Hyper on 2026-08-24,
OpenRouter's Messages endpoint on 2026-09-03)
report both counts,
and their cases pass unchanged.

Calls made here are open to veto:

- a count reported alone is dropped with its missing partner,
  as the OpenAI-compatible reader drops it,
  rather than kept beside an unreported partner:
  `CompletionUsage` in `module-llm-type` requires both counts.

Recurrence:
a count the provider never sent is absent,
never 0;
`mistake-prevention.md`,
"Provider fields read by their documentation".

## B124: a refuser held out for no time ended the call as every provider dry

Red in `804ae3e85`.
Found reading `budget-hold-wait.ts` for the T8 bundle 2 review.
`markRefused` (`provider-budget.ts`) holds a refuser whose meter reads wet for the rate-limit backoff
only when another provider reads wet,
so the calls move there;
with every other provider dry it holds it for no time,
and the transport ladder paces the retries
(XIEPT2,
2026-09-03).
The router then asked again with that refusal folded in,
and `readBudgetsPastHolds` read every provider dry,
found every hold at 0 ms,
and raised `EveryProviderDryError`,
whose message says Synthetic has no five-hour or weekly credit left,
while its meter read wet.
The two rules were each right alone:
the budget layer meant the refuser to stay spendable at once,
which its own case asserts ("HOLDS NOTHING when a wet provider refuses us"),
and the hold wait meant a just-refused provider to read dry.

Measured:
a probe on the built package
(the real `createProviderBudgets` and `createRoutingClient`,
stub meters with Synthetic wet and the others empty,
a Synthetic caller refusing one call with a 429)
asked Synthetic once and threw,
holds all 0 ms,
for a four-provider seat and a Synthetic-only one
(`t8-par/b124/wet-refuser.mjs` in the audit scratch).
Among the 8,511 `.log` files under the agent scratch
(walked three levels deep,
as the P1 measure walked),
166 all-dry errors name a refuser "just refused this call" with every hold at 0 ms,
all in the Uekawakuyuurei pictures log of 2026-09-04,
beside 166 zero-hold refusal lines there;
each was logged "voice lost",
so the seat's voice was lost each time,
not the run
(`stage-call.ts` catches the error per seat).
The huasheng4 log's 34 all-dry errors all waited out a hold first,
so they are not this.

The fix:
`readBudgetsPastHolds` returns the first reading,
the refuser wet in it,
when the provider that just refused reads wet by meter and has no hold left,
and logs that it asks that provider again.
A unit case pins that reading,
and a router case drives the real budget layer:
Synthetic refuses once,
is asked again,
and answers.
The probe on the fixed build asks Synthetic twice and is answered both times.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal".

## B202: a request the provider refused in a stream's error event was retried

Red in `6f61ba5dd`,
fixed in `72f5be638`.

Found on 2026-10-05 (UTC),
as the item ledger B87 left open.
An Anthropic stream's error event naming a type the provider documents for a refused request
was refused as a `MalformedCompletionError` and retried on the ladder,
spending every attempt on a request the provider refuses again,
where the same refusal over plain HTTP is returned unretried,
400 being no retryable status.
A scripted 200 stream carrying an `invalid_request_error` event drew 3 transport calls where 1 was expected.

The fix:
`requireWholeAnthropicMessage` (`anthropic-whole-message.ts`) throws `StreamErrorEventError`,
a marked `SyntheticHttpError` (renamed `ProviderHttpError` on 2026-10-19)
that carries the status the refusal has over plain HTTP and names the type alone,
with nothing of the provider's body;
`attemptExchange` (`transient-retry.ts`) rethrows a thrown `SyntheticHttpError`
(renamed `ProviderHttpError` on 2026-10-19) whose status is not retryable.
The types read as refused,
by the provider's errors page as the fixing agent read it:
`invalid_request_error` (400),
`authentication_error` (401),
`billing_error` (402),
`permission_error` (403),
`not_found_error` (404)
and `request_too_large` (413).
A `billing_error` so carries 402,
which the provider-budget branches read as they read the plain one.
Retried as before:
`rate_limit_error`,
`api_error`,
`timeout_error`,
`overloaded_error`,
a type the package does not know
and an event that names none.
The class joined the inventory of marked classes,
its `errorType` a named part from a closed list this package holds.

Not read by the agent:
the provider's page on streaming,
whose section on error events it could not open in its session;
the frame's shape rests on the errors page,
on ledger B87 and on the cases the file already held.

Open to the owner's veto:
`conflict_error` (409) stays retried,
since its documentation says to resolve the conflict and then retry.

Open when this entry was written,
closed in B251 (`df807a8b5`):
`InStreamProviderError` (`openrouter-stream-error.ts`) carrying a 4xx code still rides the ladder,
and the Bedrock reader names no error event in its stream end.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal":
a refusal means the same on the ladder whichever channel it arrived by.

## B251: a 200 stream whose error chunk carried a 4xx code drew the whole retry ladder

Red in `3d4558d9c`,
fixed in `df807a8b5`.

Found on 2026-10-06 (UTC),
closing the item B202 left open.
A 200 stream whose OpenRouter error chunk carried a 400 or a 402 drew 3 exchanges for one request,
where the same refusal over plain HTTP is returned unretried (the agent's probe on a ladder of two retries).

The fix:
`InStreamRefusalError`,
a marked status failure carrying the code as its status,
is thrown by `requireNoStreamError` for a whole-number 4xx code other than 408 and 429,
and the retry ladder returns it unretried;
every other code still throws `InStreamProviderError`.
`requireBedrockStreamEnd` reads an error chunk before it asks for its terminator,
so an error chunk is named by its cause and not as a stream cut off before its end.
The cases are "SPENDS ONE EXCHANGE on a 200 stream whose error chunk carries a 4xx code,
since the same refusal over plain HTTP is returned unretried,
and a 429 chunk rides the whole ladder" (`openrouter-client.unit.test.ts`),
and "NAMES AN ERROR CHUNK on either route by its cause,
a refusal for a 4xx code and a provider failure for any other,
where both read as a stream cut off before its end" (`bedrock-stream-end.unit.test.ts`).
The lead added the `@throws` lines of the two clients' whole-message checks.

Not read from a captured frame:
the Bedrock reading is the OpenAI-compatible convention.
AWS documents no error frame in a stream,
by the agent's reading of its pages on 2026-10-06,
so a real Bedrock error frame may look different.

Open to the owner's veto:
a 409 chunk is now unretried.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal".

## B804: the stream guards could not fire, and a comment line counted as progress

Status:
fixed on 2026-10-21 (UTC),
red `65b3695fc7b0816f4e3af187ae4c3a27560474cd`,
fix `2866c62ebd7a361d94854fe1c43a1f70288eef7e`,
first-content window revised from 120000 ms to 180000 ms before landing,
with `firstContentMs` logged,
in `e2c0cbbf90534ec838712d76e1f8e77dfa7d12ed` (see "The choice"),
cache versions none.
Found in the read of the corpus pass for `zheermao22` (class K2) and fixed in `w263`.

What the run showed:
eleven `inception/mercury-2.5` streams were ended 120 s after quorum by the straggler grace,
each with 0 answer characters and 0 reasoning characters on 7575 to 13000 raw characters
(21:59:56 UTC to 22:23:44 UTC).
All eleven raw counts are multiples of 25,
the length of the comment line `: OPENROUTER PROCESSING` and its two line feeds.
The log holds counts and not bytes,
so that the bytes were those comments is an inference from eleven of eleven.
Mercury's consolidation stream time was 1701 s against 462 s in `zheermao21`.

What the code did:
two defects,
one hiding the other.
`STREAM_FIRST_BYTE_MS` and `STREAM_IDLE_MS` were 600000 ms beside a per-call deadline of 360000 ms,
so neither window could fire before the deadline did,
and the artifact stamped `perCallTimeoutMs: 360000, streamFirstByteMs: 600000, streamIdleMs: 600000`
without anything reading the pair.
And `drainBody` told the guard about every chunk,
so any body bytes reset the window,
a comment line among them.
Had the windows been shorter,
Mercury's comment lines would still have kept them open.
The straggler grace (`stage-round.ts`) was the only bound that ever ended these calls,
and it ends them only after a quorum.

What was measured,
keyless,
from the progress lines of the pass logs in the agent directory,
`CuspariaKLSY20` to `zheermao22`
(`~/temp/agent/tr-fan/w263-scratch/stream-stats.ts`,
`stream-tails.ts`,
`stream-long.ts`,
`stream-empty.ts` and `stream-cutmercury.ts`,
outside the repository;
the live log `XingZ6017` left out):

- 19542 completed streams carried text.
  Their first byte,
  leaving out OpenRouter streams whose first byte may be a comment line:
  p50 1915 ms,
  p99 11755 ms,
  p99.9 88110 ms,
  largest 115725 ms.
  Their largest silence between two chunks:
  p99.9 59008 ms,
  largest 93736 ms.
- The streams a bound cut after they had delivered text show a first byte of at most 48854 ms and a gap of at most
  61642 ms,
  so the sample is censored from above by the 120 s grace and the 360 s deadline,
  and the censored part does not hide a slower healthy start.
- The logs record the time to first byte and not the time to first text.
  The first byte is a lower bound on the second,
  so the largest first byte (115725 ms) is not a ceiling on the wait for text.
  A stream whose first byte came early and whose first text came late waited longer than the log shows,
  and no log read for this entry can say by how much.

The choice:
180000 ms for the first-content window and 120000 ms for the idle window,
and the basis is the largest observation and not a percentile.
A bound at p99.9 would cut about 20 healthy streams of the 19542,
and each cut costs a paid call and a retry,
so each bound sits above everything seen.
The idle window is 28 percent above the largest gap (93736 ms),
a quantity the logs measure directly.
The first-content window began at 120000 ms,
3.7 percent above the largest first byte (115725 ms).
That margin is over a lower bound on the quantity the window bounds,
so the true margin could have been negative and a healthy slow stream cut.
It is 180000 ms now,
which is 56 percent above the largest first byte.
Both windows are below the 360 s deadline,
so a stalled call ends in half its deadline at most,
and Mercury's silent streams end at 120 s after their last text or at 180 s after arming,
without waiting for a quorum.

Older tail evidence now cut:
a `hf:zai-org/GLM-5.2` first byte of 183755 ms,
a pass-7 completion at 347099 ms and a mid-stream gap of 124992 ms
(`doc/audit/stream-guards-first-production-traffic.md`,
`doc/handover/translation-repair-history.md`)
are longer than the new windows.
The first of them is only 3755 ms over the first-content window,
so the widening does not clear it,
and it is not meant to.
They came from rosters that are gone,
and the 347099 ms completion sits within 4 percent of the old deadline of 360000 ms,
which the bound must remain shorter than.
They are retired as evidence,
not contradicted.

The fix:
`drainBody` hands the guard the running totals of content and reasoning characters
(`generatedChars()` of the stream scanner),
and the guard resets its window only when the total grows.
The raw characters still count for the progress line.
The phase names are `first-content` and `body`.
`requireStreamBoundsInsideCallTimeout` (`stream-bound-config.ts`) refuses,
by `StreamBoundsOutsideCallTimeoutError`,
a bound that is equal to or longer than the deadline,
or that is not a positive finite number,
and so does a deadline that is not.
`runCallConfigOf` in `run-config.ts` calls it,
and `RUN_CALL_CONFIG` is built through it at module load,
so a run with an inconsistent configuration ends before it spends anything.
The key `streamFirstByteMs` of the stamped configuration is now `streamFirstContentMs`.
Older artifacts keep the old key.
The retry ladder (`exchangeWithRetry`) names the provider on the exchange it hands the transport,
and the guard's cut line reads
`<provider> <model>: no content or reasoning for <N>ms during <phase>; <N> raw chars on the wire, <N> content chars and <N> reasoning chars so far; aborting for retry`.

Red and green:
`stream-content-progress.unit.test.ts` cuts a stream of comment lines only,
one of blank lines only,
one of role-only and empty-content events,
and one that falls to comment lines after its first words,
and keeps a stream that sends a content delta or a reasoning delta every 8 ms.
It failed with five assertion failures and its two controls passed.
`stream-bound-config.unit.test.ts` refuses the pair that shipped,
a bound equal to the deadline,
and each of not-a-number,
zero,
a negative and infinity,
and pins the shipped windows above the largest healthy observations,
the first-content window half again over the largest first byte,
and both below the deadline.
`stream-idle-guard.unit.test.ts` pins 180000 ms and that the 183755 ms first byte is still cut,
and times the first text beside the first byte;
`stream-cut.unit.test.ts` reads `firstContent` off the progress line,
and `stream-content-progress.unit.test.ts` reads a negative `firstContentMs` off a stream of comment lines that did send bytes.
`run-call-config-builder.unit.test.ts` hands `runCallConfigOf` the shipped pair.
`provider-named-exchange.unit.test.ts` reads the provider off the exchange of each of the five clients.
Each failed before the fix,
at import or on an assertion,
and passes after it.

Cache versions:
none moves.
The windows and the call configuration are in no cache key,
as `perCallTimeoutMs` is not:
they change how long a voice has to answer,
not what it is asked.

The time to first text:
the progress line of every stream,
completed or cut,
now carries `firstContent <N>ms` beside `firstByte <N>ms`
(`StreamProgress.firstContentMs`,
negative when no content or reasoning delta ever arrived).
The first-content window was set against the first byte because the logs held nothing else;
the next reading of the pass logs sets it on the quantity it bounds.
Logs written before this change carry the first byte alone.
The change is diagnostic only:
no window,
cache key or decision reads the new field.
The straggler grace stays as a second bound after quorum.

Limits:
the first-content window is 56 percent over a lower bound.
A model that thinks for longer than 180 s before its first delta,
with its thinking hidden from the stream,
would be cut where it was not,
and the first text of such a stream is the figure the new field records.
The logs read hold no such stream among the completed ones,
but they do not record whether a model hid its thinking,
so this is unmeasured.

Open to the owner's veto:
180000 ms for the first-content window,
which the parent chose over 120000 ms because the margin over a lower bound could be negative.
Changing it is `STREAM_FIRST_CONTENT_MS`.

Recurrence:
`mistake-prevention-guards-searches-and-claims.md`,
"A bound is read against the deadline it sits under":
a bound is checked,
at the place both numbers are read,
against the deadline that would pre-empt it,
and progress means the output the bound is for.

## B809: rounds waited five minutes on one stream that sent only comment lines

Status:
fixed in part on 2026-10-22 (UTC),
red `731c541fa2`,
fix `9614ff8336`,
docs `cd456d450e`,
count correction `6eb6109ed7`,
cache versions none.
Found in the read of the corpus pass `XingZ6017`
(23:05 UTC on 2026-10-21 to 03:34 UTC on 2026-10-22,
build `2a336603bf`)
and fixed in `w271`.
The per-card window of this entry is open to the owner's veto.

What the run showed:
27 rounds took 250 s or more.
26 of them waited on an `inception/mercury-2.5` stream with one signature.
The first byte came at 735 to 975 ms,
the largest silence between chunks was 530 to 779 ms,
and the stream held 18181 raw characters
(727 lines of `: OPENROUTER PROCESSING` and their blank lines;
every raw count is a multiple of 25),
0 answer characters and 0 reasoning characters.
The stream "completed" at about 301.3 s with an HTTP 502 inside a 200 stream
(`code 502, type provider_unavailable`),
the ladder retried it at once,
and the retry answered in 5 to 9 s.
The 301 s is the gateway's bound,
which is an inference;
the call deadline is 360 s.
Those 26 rounds held 28 such streams,
because the coverage round at line 1199 and the review round at line 31111 each held two in a row.
Line 31111 was 267 s,
then 92 s until the 360 s deadline,
and it ended with no quorum.
The 27th round,
line 37385,
was a slow live `hf:Qwen/Qwen3.8-27B` stream that answered at 303.6 s with its gaps under 68 s;
it is correct to wait for it.
No round of the 27 was a stream with no first byte or one that delivered text and then went quiet.
The rounds that waited on a comment-only stream,
by log line:
1199,
1683,
8015,
13102,
13298,
13881,
15503,
16005,
16732,
17268,
17524,
27349,
28776,
30503,
31065,
31111,
31348,
31730,
32531,
33466,
34300,
35153,
35511,
35778,
35959 and 36388.
The per-round table is `classification.md` in `~/temp/agent/tr-fan/tr-w271-notes/`,
outside the repository.

Why the grace never applied:
a consolidation bench of 7 with 3 seats unreachable needs 4 voices,
which is every live seat
(`quorumNeeded=4 quorumOver=7 unreachable=3`,
line 28777),
and a repair checker or probe bench of 3 with Kimi-K3 unreachable needs 2 of 2 live seats.
`reachableQuorum` (`stage-reachable-quorum.ts`) returns the bench quorum
whenever the reachable seats are not fewer than it,
and the straggler grace opens only once the quorum stands,
so it cannot open before the one hung voice.
This is the owner's rule of 2026-09-09 (a quorum sized on the reachable bench) working as written,
and it is not changed.

What the code already did:
the first-content window of 180000 ms (B804,
`2866c62ebd` and `e2c0cbbf90`) lives in the transport,
resets only on a content or reasoning delta,
and its abort is retried by `exchangeWithRetry`;
it does not depend on quorum or grace.
Two of the guards this entry first looked for,
a first-content cut whether or not a quorum stands
and a stall bound on a stream whose text stopped,
were therefore present at the worktree's base:
the first-content window cuts comment-only streams at 180 s,
and the idle window of 120000 ms bounds a stream whose text stopped.
Neither had a log to act on in the run,
which predates both.
Under that code each of the 26 rounds would have ended its bad stream at 180 s instead of 301 s,
about 186 s per round instead of about 306 s.

What changed:
the shared window is set against providers that send no comment lines
and says nothing of an OpenRouter model whose gateway writes them.
A card can now name a window of its own,
as the Bedrock card names its stream bound:
`OpenRouterCard.firstContentMs` (`shared-contract`),
`TransportExchange.firstContentMs`,
which `fetchTransport` hands to `armIdleGuard`,
and the OpenRouter client copies the card's window onto each exchange
(`openRouterFirstContentWindowFor`,
`openrouter-catalog.ts`).
Only the Mercury card names one,
90000 ms.

The evidence for 90000 ms:
`seat-stream-stats.ts` over the 24 pass logs on disk (`CuspariaKLSY16` to `XingZ6017`) found
4636 Mercury streams that delivered text,
with elapsed p50 6680 ms,
p99 20250 ms and longest 56772 ms.
The first text comes no later than the end of the stream,
so 56772 ms bounds it from above.
No Mercury stream that ran 60 s or more without text ever delivered any
(130 such streams,
96 with raw counts that are multiples of 25).
The window is 59 percent over the longest observation,
the margin B804 gave the shared window (56 percent).
The other six OpenRouter models in the logs
(`deepseek/deepseek-v4.1-flash`,
`deepseek/deepseek-v4.1-flash-1017`,
`google/gemma-4-26b-a4b-it`,
`minimax/minimax-m3`,
`xiaomi/mimo-v2.6-flash`
and `z-ai/glm-5.3-flash`)
show no stream silent for 60 s or more,
so none has the evidence and none gets a window.
With the card window each of the 26 rounds ends its bad stream at 90 s and the retry answers in about 6 s,
about 96 s per round instead of about 306 s;
the round of line 31111 takes about 190 s instead of 360 s.

Red and green:
`transport-first-content-window.unit.test.ts` in the provider-stream package
cuts a stream of comment lines at a window named on the exchange
and reads the same stream whole on an exchange that names none.
`openrouter-first-content-window.unit.test.ts` in the provider package reads the window off the Mercury exchange,
none off the exchange of a card that names none,
and pins the card's window at least half again over 56772 ms and under the shared window.
The red commit failed on both assertions that name the window,
and its control passed.
The fix commit also moves the exchange-keeping test transport into `recording-transport.test-fixture.ts`
(`provider-named-exchange.unit.test.ts` used its own copy)
and lists the new test bounds in `real-clock-bounds.unit.test.ts`.
The count correction `6eb6109ed7` made the card comment,
the guard note and both test headers say 26 rounds,
where they had said 28:
28 is the count of such streams.

Fits the decisions:
`doc/decision/translation-repair-straggler-grace.md` says
"No grace change is authorized before active runs finish and matched evidence exists",
and this change touches no grace window and no round-ending rule.
The first-content window is a stream guard set in
`doc/decision/translation-repair-runaway-call-termination-stall-window-premise-and-straggler-cuts-remeasured.md`,
which says it stays open for the owner to loosen the window if the next runs show healthy streams cut,
or to tighten it once `firstContent` figures show how long healthy first text takes.
`XingZ6017` predates the `firstContent` field,
so the figure used is the elapsed time of a stream that delivered text,
an upper bound on the first text,
which can only make the window safer than the true figure would.
That is tightening for one model on a bound from above,
not the measurement the decision names,
and it is flagged here and in that decision's section "The windows armed on 2026-10-21".

Not verified:
the whole path (guard cut,
ladder retry,
second attempt returned to the caller) with the real transport has no test.
Each leg has one:
`stream-content-progress.unit.test.ts` and the new transport test for the cut,
and `transient-retry.unit.test.ts` for the ladder retrying a `StreamCutShortError`.
A test over the real clock at 90 s was not written.
The gateway's 301 s bound is inferred.
Whether Mercury's hang rate (about 5 percent of its calls in `XingZ6017`,
no hour without one) continues is unknown.

Open to the owner's veto:
90000 ms for the Mercury card (`firstContentMs` in `model-cards.ts`);
whether to read `firstContent` figures from the next pass log before keeping it;
and,
as a policy question the evidence cannot settle,
whether a short bench that needs every live seat should wait on its last seat at all
(the rule of 2026-09-09;
no change made).

Update of 2026-10-22 (UTC):
the owner answered the policy question with "Let the grace open".
A short bench whose reachable seats equal its quorum,
above two voices,
now closes one voice below every live seat,
so the grace can cut the last of them,
and the bench of 7 with 3 seats unreachable of this entry needs 3 of its 4 live seats.
The text of this entry stands as the record of 2026-10-22 before the answer.
The change,
the shape it does not reach (a bench of 3 with 1 unreachable)
and the cache versions it moved are in B816
(`audit-ledger-short-benches-and-the-last-live-seat.md`).

## B810: a seat abandoned again and again is asked every round; the evidence rejects a hold

Status:
examined on 2026-10-22 (UTC) in `w271`;
no code change,
because a hold on a seat abandoned again and again was tested against the log and rejected.
Found in the read of `XingZ6017`.

What the run showed:
98 `abandoned 120000ms after quorum` lines:
`inception/mercury-2.5` 52,
`hf:Qwen/Qwen3.8-27B` 43,
`deepseek-v4.1-flash` 2 and
`minimax-m3` 1.
The tally counts an abandoned call as `abandoned` and in neither `asked` nor `threw` (B421,
revised 2026-10-19),
so no seat is withheld for it.
After Synthetic read wet at 02:39:44 UTC on 2026-10-22,
Qwen was seated again.
Of its 95 streams from then on,
40 were cut with 0 answer characters (28 to 49 thousand reasoning characters) and 50 answered
(`qwen-era-out.txt`).
The mean consolidation slice cost rose from 149 s to 315 s and 346 s,
and fell to 104 s when Synthetic went dry at 03:26:35 UTC.

The two seats are different.
Mercury's cuts are the comment-only hangs of B809,
about 5 percent of its 1535 streams,
independent of each other:
the longest run of consecutive silent cuts in `XingZ6017` was 2.
Qwen's cuts are a working seat's slow tail.
The streams that answered carry 15 to 50 thousand reasoning characters,
the same range as the cut ones,
and took,
over the whole run,
p50 82 s,
p90 160 s and longest 304 s,
while the cut ones land at 125 to 193 s
because quorum arrives in seconds and the grace is 120 s.
The grace is the owner's choice of 2026-09-03,
taken with the 60 s window's cuts all read as reasoning streams still working,
and 7 cut streams at 120 s.

The hold,
simulated (`hold-sim.ts`,
`run-census.ts`,
in `~/temp/agent/tr-fan/tr-w271-notes/`):
a seat held for H minutes after N consecutive silent abandonments with no answer between,
over Qwen's 95 streams.

- N=3 and a hold of 10 min:
  20 cuts avoided,
  25 answers forgone.
- N=4 and a hold of 10 min:
  18 avoided,
  24 forgone.
- N=4 and a hold of 30 min:
  25 avoided,
  34 forgone.
- N=5 and a hold of 10 min:
  8 avoided,
  14 forgone.
- N=6:
  never fires.

At every setting tried the hold forgoes more answers than the cuts it avoids,
because Qwen answers about 55 percent of the time.
Over 17 pass logs the longest run of consecutive silent cuts by Qwen was 5 (`XingZ6017`)
and by Mercury 9 (`CuspariaKLSY22`,
a day Mercury hung on 33 of 69 streams).
A hold also turns a bench of five reachable seats into four,
which is the shape of B809 again:
the quorum becomes every live seat,
so the grace cannot open,
and the wall-clock saved is smaller than the cuts avoided suggest.
Quality comes before speed,
and the protection of B421 is doing what it was built for,
so no abandonment-counting hold was built.

Update of 2026-10-22 (UTC):
the sentence "the quorum becomes every live seat,
so the grace cannot open" described the rule of 2026-09-09.
The owner's answer "Let the grace open" (B816) ended it for a bench whose reachable seats equal its quorum above two voices:
that bench now closes one voice below every live seat and the grace can open on the last one.
The sentence stands for a bench of 3 with 1 unreachable,
which keeps the floor of two voices.

Not built:
a dead-seat hold,
N=6 consecutive silent cuts and then ten minutes,
would have fired only on Mercury in `CuspariaKLSY22`
and on no Qwen run in 17 logs.
One log is thin evidence,
and the first-content window of B809 now ends such a stream before the grace does.

What stays open,
for the owner:
whether `hf:Qwen/Qwen3.8-27B` should sit on the consolidation and late-judge benches at all
while Synthetic serves it at 2 percent,
the question `HYPER_SLOW_JUDGES` and `HYPER_SLOW_SELECT_JUDGES` (`run-seats.ts`)
answered for Hyper with the owner's authorization.
The figures of this entry are the evidence to put beside it.

Update of 2026-10-22 (UTC):
the owner answered the question left open here with "Remove from both benches".
`hf:Qwen/Qwen3.8-27B` left the lane-contest bench and the consolidation bench
(the slate judges and the gate),
which are seated from the late judges,
and keeps its wide seats where it is still seated per provider.
The reading of "both benches" is open to the owner's veto.
The change,
its red and fix commits and the follow-on in the reseat tests are in B815
(`audit-ledger-providers-and-seats-a-run-holds-out.md`).
