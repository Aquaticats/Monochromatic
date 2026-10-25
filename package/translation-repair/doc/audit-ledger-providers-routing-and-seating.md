# Audit ledger: providers, routing and seating

Findings in the provider clients,
the router and the seating of models.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## Providers, routing and seating

Probes and full report:
`~/temp/agent/audit-providers/report.txt`.

### P1: the Bedrock ledger records completed calls only

Status:
fixed 2026-09-28,
each part guarded red first and mutation-checked.
The ledger is the only guard on the owner's card,
and it noted completed calls with usage only;
`bedrockIsDry` read dry at zero while the meter is read once a 60 s freshness window.

- Every billed attempt:
  `2a108dfb3` (guard `e33396294`).
    The retry ladder tells its caller of every attempt that delivered something and failed,
    retried ones included,
  before any rethrow (`onAbandonedAttempt`,
  `transient-retry.ts`):
    a reply the whole-message check refused counts its whole body,
    a stream that ended early what its error says it read.
    Bedrock writes each at its bound
    (no more prompt tokens than body bytes,
  no more completion tokens than `max_tokens`),
    marked `abandoned-bound`,
  on the `SPEND` line and in the ledger (`bedrock-bound-ledger.ts`),
    since whether Bedrock bills output past a cancel is unmeasured and an under-read is the failure.
    OpenRouter writes its reckoned line per attempt;
  it wrapped the whole ladder,
    so an attempt refused and retried inside it left no line.
- No usage:
  `107763dbb` (guard `2770e9ce7`).
    A whole call whose stream carried no usage block is written at its bound,
  marked `unreported-bound`.
    Latent:
  none of 190,009 logged Bedrock calls lacked usage.
- Margin:
  `7cfd5ae4b` (guard `73e70bfab`).
    Bedrock reads dry with 1.33 USD left:
  the most spend any span of one freshness window
    plus the longest Bedrock stream on record (363,790 ms) held over every pass-run log
    (`p1-window-measure2-runs.mjs`) is 1.3245 USD over 822 calls (XingZ628).
    Per-call reservations were considered and not taken:
    the cached reading,
  not the calls in flight,
  is what lets a run spend past zero.
- Reckoned share:
  `a88a87b11` (guard `4364caf88`).
    The reading carries `reckonedUsd` and the `METERS` line `bedrockReckonedUsd`,
    so a reader sees how much of what is left rests on reckoning.
- One-time correction of the live ledger,
  2026-09-28 (`p1-ledger-correction.ts`).
    1,055 Bedrock streams in pass-run logs never reached it
    (344 on gemma-4-26b-a4b,
  693 on gemma-4-e2b,
  14 on gpt-oss-120b,
  4 on gemma-4-31b):
    cut,
  overrun,
  or completed with no `SPEND` line after,
  an attempt the whole-message check refused.
    Each is reckoned at its model's 99th-percentile prompt over paired calls,
    and its generated characters at the model's 10th-percentile characters per token.
    1.3184 USD,
  four lines marked `abandoned`;
  remaining 6.3413 to 5.0229 USD.
    The file before is kept beside it (`bedrock-spend.jsonl.before-p1-correction-2026-09-28T141153.313Z`).
    A conservative reckoning,
  not a bound:
  request bodies are not stored,
  so no byte count exists for them.

Mutation check (`p1-mutants.json`,
`p1b-mutants.json`):
the ladder never telling,
ignoring a refused body,
telling only after the self-ended rethrow,
OpenRouter not reporting,
Bedrock not ledgering an abandoned attempt or a call without usage,
the bound's prompt at a quarter of the bytes or its completion at zero,
the ledger not summing the reckoned share or accepting any mark,
and dryness at zero are each caught;
the control survives.
The first run's verdict on the no-usage mutant was a false survival (see the mutation harness in the process mistakes).
Cache:
the margin moves a Gemma call to OpenRouter sooner as Bedrock nears its credit,
the class P9 accounted;
rides inside all six versions with an account in each;
same slice-cache check,
same result.

### P2: the recovery round never re-asks a seat that answered unreadably before the last round

Status:
fixed in `005692e11` (guard `3549b74be`;
mutation checked with a control).
A seat that answered unreadably now waits for the nudged recovery round,
which re-asks every seat still unreadable
whichever round it came in;
the retry rounds no longer re-send it the same prompt,
which the prompt-uniqueness client answers from memory or disk with the same bytes.
Two older cases scripted unreadable answers that cleared on a same-prompt re-ask,
which no run can do,
and now script the retry rounds' weather as transport failures.
Rides inside all six cache versions.
`stage-quorum.ts` overwrites the unreadable list each round and returns the seat to `pending`,
where the prompt-uniqueness cache answers it with the same bytes;
TianqiChen66620 slice 15's gate settled on neither 2 to 2 with one such voice lost.

### P3: seats the phase knows are unreachable fill the round-0 window

Status:
fixed with guard `9466786dc`,
fix `84a6caa02`.
Every retry round 1 had one or two refused seats in round 0.
The first reading of this entry said rounds then ran to the 360 s deadline;
that was wrong:
no round in TianqiChen66619 or TianqiChen66620 ran past 181 s.
The real cost:
a seat refused in the same millisecond spent the round's spare place,
so the round waited for its slowest reachable voice with no grace or fell through to a retry round.
TianqiChen66620 closed 269 of 421 rounds a seat short with no grace,
2,852 s of the 4,575 s its rounds took.
A refused window seat now hands its place to the next pending seat within the round
(`runGatherRound` `reserve`),
and the seat stays on the bench as the 2026-09-09 rule has it.

### P4: the recall benchmark still seats gpt-oss-120b

Status:
fixed 2026-09-28,
each part guarded red first and mutation-checked.
The owner's cull of 2026-09-24 held on every derived bench,
and nothing stopped a path that named the seat itself.

- Recall judges:
  `460ceb8f4` (guard `b522c3e83`).
    `DEFAULT_JUDGE_MODEL_IDS` listed three judges by hand and still named `hf:openai/gpt-oss-120b`.
    The benchmark now takes its judges from its caller (`judgeModelIds` is required),
    and the recall benchmark passes `RECALL_JUDGE_MODEL_IDS`,
  the wide seats the run derives,
    which every seating rule and the cull reach.
- The run client:
  `9fd80fb28` (guard `6e935644b`).
    `refusingCulledSeats` (`culled-seat-guard.ts`) wraps the routed client
    and throws the `NoProviderForModelError` a round reads as an unreachable seat,
    before any provider is asked,
  on `chatText`,
  `chatJson` and `decide` alike.
    Catalog reach is unchanged:
  the card stays for the catalogs and fixtures,
  and `reachOf` reports what serves a model,
    while the refusal says the owner does not seat it.

Mutation check (`p4-mutants.json`):
the guard never refusing,
the run client unguarded,
and the recall bench naming the culled model are each caught;
the control survives.
No cache moves:
no derived bench seated the culled model since the cull,
so no cached answer came from it.

### P5: the archive-block-review guard rejects a shape its prompt never forbids

Status:
fixed in `8e994bc77` (guard `930096761`;
mutation checked with a control).
All 11 guard rejections in five runs are editorial-context with a non-empty `sourceQuote`.
The prompt asks for "exact source support or empty" and never ties the empty value to that disposition,
and nothing reads an editorial-context quote:
the stage checks the block itself.
`dc51b02d9` fixed the same slip for `revise` on 2026-09-09 and left this one,
the guard-stricter-than-its-prompt family.
Only source-supported retention now needs an anchor.
Archive-block reviews are not cached,
so no version moved.

### P6: the seat tally cannot see an unusable reply

Status:
fixed in `27bc9f951` (guard `a30b43761`;
mutation checked with a control).
`SEAT inception/mercury-2.5 asked=1007 usable=1007 unusable=0` beside 40 schema losses.
The tally sat inside `promptUniqueClient`,
which buys every JSON reply through `chatText` and reads it itself,
so the tally saw text arrive and counted it usable.
It now wraps the client callers hold and settles each JSON call as the outcome its caller gets:
the nudged re-ask elsewhere (P9) is inside that one call,
and a replayed payload counts as an ask,
since its answer is the run's evidence all the same.
The red guard read `asked 2, usable 1, unusable 0` for one unreadable answer:
the second ask was the nudged re-ask,
which the second provider refused,
counted as a seat that threw.
Mutation check (`p6-mutants.json`):
the tally back inside the wrapper,
and a tally reading every JSON outcome as usable,
are each caught.
No cache moves:
the tally reads outcomes and writes nothing a stage reads.

### P7: abandoned-spend estimates mix units

Status:
fixed 2026-09-28.
`7a6635976` (guard `bd8e8ef86`):
overrun and degenerate endings carry `rawChars`,
the raw wire characters delivered,
which `deliveredCharsOf` now returns for every error;
they returned one channel's decoded count,
about a hundredth of it (`completion=5` for 1,633 content characters).
The reckoning never passes the `max_tokens` the call sent.
Card ratios re-measured (`1f5f5e47c`,
kept to pass-run logs in `63455c32b`,
`p7-openrouter-measure-runs.mjs`):
mercury carried 137 where it measures 0.9 over 37,120 streams (it bills far more tokens than it streams);
deepseek-v4.1-flash and mimo carried none (228 and 93);
kimi 140,
minimax 130,
glm-5.3-flash 302,
gemma-4-26b-a4b-it 292,
gpt-oss-120b 286;
the unmeasured default is their median, 184.
The OpenRouter meter stays authoritative;
the run spend meter now reads truer,
so the per-run spend ceiling (`corpus-run/pass-stop-before-next.ts`) can stop new entries sooner.
Mutation check (`p1-mutants.json`,
`p1b-mutants.json`):
a channel count in place of the raw one on either error,
the drain dropping the raw count,
and an uncapped reckoning are each caught.
No cache moves:
nothing a cached answer depends on reads the reckoning.

### P8: a complete JSON value followed by more text is lost

Status:
fixed in `cac097368` (guard `e86cd9f44`;
mutation checked with a control).
36 in five runs,
760 across all logs,
mostly mercury.
The stored TianqiChen666 replies of that shape trail a hyphen line,
a sentence or a stray fence,
every one with the stop reason.
`parseAnswerJson` now reads the whole answer,
then past a false start,
then the value the answer opens with.
Replayed over 587,102 stored replies:
919 recovered and none read differently.
A first ordering put the leading value before the false start and read 18 replies differently:
each held two whole objects,
the first empty or missing a field the second carries,
so the later object is the answer and the false start stays first.
Rides inside all six cache versions.

### P9: the router's cross-provider re-ask never runs in production

Status:
fixed in `7011d72cc` (guard `4d16f4318`);
owner ruled 2026-09-28,
"Enable with the nudge" (`design-commitments.md`).
`promptUniqueClient` buys every JSON reply through `chatText` and reads it itself,
so the router's `chatJson`,
where the re-ask lived,
was never called in a run.
The router now tags each reply with the provider that served it (`servedBy`)
and serves a request carrying `otherThan` on another wet provider serving the model
(`routedTextElsewhere`,
reusing the second-opinion routing,
the slot take and the budget-refusal handling),
throwing `NoProviderForModelError` where none can take it.
The uniqueness wrapper re-asks a reply that could not be used through its own claim path (`nudged-reask.ts`),
so the nudged exchange is claimed and stored like the first and a resumed run replays both;
the payload store replays the tag,
and a payload stored before it existed is not re-asked.
The first answer stands when the re-ask cannot happen or fails,
and the nudged prompt's claim is released.
Decided for quality,
recorded under the commitment:
the re-ask carries `CROSS_PROVIDER_NUDGE`,
worded apart from the recovery round's `RECOVERY_NUDGE`,
since a shared wording would make the round's prompt
this re-ask's digest,
answered from the claims with the reply that already failed;
and it fires on a refusal-shaped reply as on a schema mismatch,
as the router's never-run re-ask did,
with a nudge neutral on why the reply could not be used.
`NoProviderForModelError` moved to its own module so the re-ask can raise it without an import cycle.
Reach (`p9-reach.mjs`,
every log under the agent directory,
unit-test logs included):
157,945 unusable-reply lines,
122,593 on models another provider serves today;
the TianqiChen666 logs hold 134,
so the re-ask adds at most a few exchanges per run.
Mutation check (`p9-mutants.json`):
the re-ask bypassing the claims,
never re-asking,
sharing the recovery nudge,
dropping the hint,
the router ignoring the hint or dropping the tag,
and the store dropping the tag are each caught;
the comment-wording control survives.
Cache:
rides inside all six versions,
with an account in each.

### P10: the deepseek-v4.1-flash card is stale against its own measurement rule

Status:
fixed 2026-09-28:
the card,
the nudge and the census,
each guarded and mutation-checked.
The card named the pooled 99th percentile as "no completed-call distribution of its own yet" after 67,353 calls,
and the recovery round told a model whose reply the cap cut that its shape was wrong.

- The cap:
  `4943d74d5`.
    Re-read over pass-run logs alone (`p10-cap-measure.mjs`,
  `p10-cap-cuts.mjs`),
  499,820 completed calls.
    By the rule (the highest provider p99 with at least 100 calls,
  floored at the pooled 90th),
    deepseek-v4.1-flash reads 13,082 on Hyper,
  the cap itself:
  75 of 4,062 Hyper calls ran to it,
  74 with no content.
    OpenRouter reads 7,531 over 63,291;
  of its 237 calls at the cap,
  the 70 that pair with a stream line are
    69 runaways and one answer.
    The card now carries `13_082` as its own number;
  the runtime value does not move.
    Mimo keeps the pool with its 78 calls named,
  under the 100 the rule reads.
    Every other card's cap comment now says it is the 2026-09-09 reading.
- Why the caps stand (`completion-cap.ts`,
  re-read paragraph).
    Every call since 2026-09-09 carries its cap,
  so a re-read can confirm or lower a cap and never shows a longer answer.
    Seven seat and provider pairs run to their cap on more than one percent of calls since then,
    against the table's "under one percent",
  and where a cut call pairs with its stream nearly every one streamed no content.
    The re-read's pool (p90 2,607,
  p99 10,822) is mostly capped calls,
  nearly a third of them Bedrock Gemma,
    so the pooled constants stay.
    The re-read's scope differs from the 2026-09-09 table's (pass-run logs only,
  where the table read every log),
    which is why Kimi-K3 on Hyper reads 8,496 against the table's 10,921 and minimax-m3 on Hyper 12,207
    (pre-cap calls) against 10,822;
  neither is a defect,
  and no cap moves on them.
- The nudge:
  `ce0ef7b51` (guard `0be7b4437`),
  decided for quality (`design-commitments.md`).
    A lost voice that answered carries its cause (`cut-short` for `truncated-completion` or `truncated-thinking`,
    `off-shape` otherwise),
  and the recovery round asks each cause with its own wording (`recovery-nudge.ts`),
    both groups in one window.
    The off-shape wording is unchanged,
  so a stored payload for it still replays.
    The red guard's probe read both seats re-asked and heard with the same nudge.
    The cross-provider re-ask keeps its neutral wording,
  which is true of a cut reply too.

Mutation check (`p10-mutants.json`):
one wording for both causes,
a cause that never reads cut,
one blind to cut thinking,
the stage call dropping the cause,
the round ignoring it or dropping the cut group,
the deepseek card back on the placeholder,
and the all-dry refusal without its split sentence are each caught;
the control survives.
Cache:
the cut-short nudge changes what such a seat is asked,
so it rides inside all six versions
with an account in each (`dd2d454ab`);
same slice-cache check,
same result.
The census:
`89f642614`,
since a unit test cannot read the logs and a dated test would be a time bomb.
`mise run cap-census -- <logs or directories>` re-reads the rule over pass-run logs per seat and provider,
counts the capped calls that ran to the cap by whether their stream carried content,
and flags a card on the pooled 99th that the rule can now read,
a rule reading off the card,
and cuts over one percent on a provider with enough calls to say.
Its first run matched the scratch measurement (499,820 calls;
deepseek on Hyper 75 of 4,062 at the cap,
74 with no content),
and showed two flag defects before commit:
the placeholder flag fired on the pooled 90th,
which is a measured floor,
and the cut flag fired on Kimi-K3's 20 OpenRouter calls;
both fixed with cases.
Setup step 6 of the corpus-pass runbook runs it,
and its filter was run on the real output.
Mutation check (`p14-mutants.json`):
a wide pairing window,
pairing any stream outcome,
keeping reckoned lines,
the rule ignoring the call minimum or the floor,
cuts counted from before the caps,
the placeholder read off any pool,
and the cut flag on a thin sample are each caught;
the control survives.

### P11: owner-rule enforcement relies on absence rather than a guard

Status:
fixed in `0baa4cb41`;
green on arrival,
since every body already kept the rules,
so the mutation check is its proof.
Qwen3.8-27B and glm-5.3 stayed off OpenRouter only because their cards lacked an OpenRouter block;
the no-thinking,
no-budget,
always-`max_tokens` rules were read only on the Hyper and OpenRouter bodies.
`owner-body-rules.unit.test.ts` reads every request body all four clients send,
text and schema calls,
for `max_tokens` present and no `thinking`,
`budget_tokens`,
`reasoning_effort`,
`reasoning` or `include_reasoning` key at any depth,
and checks that every seat in `OPENROUTER_DROPPED_SEATS` has no OpenRouter block and no OpenRouter reach.
`reachOf` and the OpenRouter picture check now read `OPENROUTER_DROPPED_SEATS` too.
Mutation check (`p4-mutants.json`):
thinking on the Synthetic body,
a budget on the Hyper body,
a reasoning effort on the OpenRouter body,
and no `max_tokens` on the Bedrock body are each caught.
The mutant dropping the new reach condition survives by construction:
no card carries both an OpenRouter block and a drop,
and the card case fails first if one ever does.
No cache moves:
no dropped seat had OpenRouter reach,
so no route changed.

### P12: log lines that cannot be attributed

Status:
fixed 2026-09-28,
each code part guarded red first and mutation-checked;
A11 is its own entry.

- Retry lines never named the model:
  `dc156524b` (guard `db3dae27d`).
    The retry line and the stated-wait line open with the exchange label.
- "ms to quorum" printed when quorum never stood:
  `6d27dab11` (guard `f03900948`).
    Such a round now says `no quorum (heard of needed needed), every ask settled`,
    since its "quorum" mark was only when the last ask settled,
  and the grace after it was nothing.
- `EveryProviderDryError` omitted Bedrock:
  `dc6bd8321` (guard `0857bf052`).
    It lists Bedrock among the providers out of budget,
  says its credit is never topped up,
    and cites the meters the router decided on where it said "no reading cited".
    The first guard checked the whole message for "Bedrock",
  which the refill sentence also carries,
    and the mutant dropping Bedrock from the list survived (`p6-mutants.json`);
    `5cf2889ea` reads every provider in the clause before "Nothing further can be bought",
    and `p12b-mutants.json` catches that mutant and one dropping Hyper.
- `run-config.ts` said Qwen is withheld whenever Synthetic is dry:
  `ab923b266`,
  a comment.
    `run-seats.ts` withholds it while Hyper would serve it,
  Synthetic dry and Hyper wet;
    with both dry no provider serves it and a round reads it as unreachable.

Mutation check (`p6-mutants.json`,
`p12b-mutants.json`):
a round reading as quorate whatever it heard,
the list without Bedrock or Hyper,
the refill sentence without Bedrock,
the refusal without its reading,
and the reading with dry and wet swapped are each caught;
the controls survive.
No cache moves:
log lines and a refusal message only.

### P13: low items

Status:
closed 2026-09-28;
five sub-items fixed,
one refuted with evidence.

#### Retry backoff and caller abort

Fixed in `169b51e8a` (guard `b0cd02da1`;
mutation checked with a control).
The ladder slept its backoff on a timer no signal could end and read the abort after it,
so an aborted or timed-out call held its seat for up to the ladder's 16 s reach.
The sleep now takes the exchange signal and returns at once on its abort.
While there,
`9c10b0a6c` corrected the ladder's reach wording:
`longestBackoffMs` returns `baseMs * 2 ** limit` (16 s),
which its TSDoc,
call site,
test policy
and `31e67a100` all called the last retry's widest window (8 s).
The code value is what the suite pins;
277 of 57,803 logged backoffs slept a stated wait between the two,
and with no model on retry lines (P12) their outcome is unmeasured,
so the wording moved and the behavior stayed.

#### Payment refusal clearing on a downward meter move

Refuted,
recorded at the rule in `de673c843`.
Of 230 payment refusals in the run logs,
198 came from OpenRouter at a wet meter (0.01 to 1.94 USD),
refusing what the balance leaves after in-flight reservations:
8 bodies name `in_flight_budget_exhausted`,
the rest ask for up to 131,072 tokens and are told 1,844 to 84,651 are affordable.
A settling call lowers the balance while freeing its larger reservation,
so clearing only on a rise would hold OpenRouter dry for the rest of a run;
the rule's measured cost is 28 refusals after a downward move.

#### Bedrock stream bound across the retry ladder

Fixed in `b1a4f4b9e` (guard `684df9e91`,
with the `streamBoundMsOverride` test seam;
mutation checked with a control).
The bound is a one-stream measurement (class 148) but was armed once around `exchangeWithRetry`.
It now wraps each attempt inside the transport,
and the ladder rethrows a bound cut,
found through the cause chain,
so the router still holds Bedrock out for the model.
Latent:
all five run cuts ran a single attempt to 60 s.

#### Card prices against the endpoint bought

Fixed in `7ceffe055`,
with no guard (no offline oracle holds the live listing).
GLM-5.3-Flash carried DeepInfra's 0.075 and 0.25 while all 253 of its calls on the three latest TianqiChen666 runs went to Wafer,
listed 2026-09-28 at 0.9 and 0.5;
DeepSeek V4.1 Flash carried the catalog's 0.3 and 1.2 while 812 of 817 went to Morph,
listed at 0.12 and 0.468 and charged at 0.662 to 0.674 of that.
Only the abandoned-spend estimate reads these;
`configuration.md` now says a preferred-endpoint card carries that endpoint's price.

#### Decision seats past reach

Fixed in `a991ef1e1` (guard `cabfa82f4`;
mutation checked with a control,
five mutants caught):
decision-seat structural losses marked reachable,
and decision-seat prompts reaching 30,203 of a 32,000-token context
after `ce824d933` added the house rules with no size check.
The decisions reference documents a 413 for an oversized payload and nothing for a state past the context,
so the endpoint was probed on 2026-09-28 with cat-themed states (under a cent):
a 45,046-character state came back HTTP 400 `max_tokens_exceeded` with the evidence first and with it last,
never a decision on a truncated state.
The same probes measured the house rules at 4,085 tokens (17,493 characters),
Han text at one token a character and English at 4.2 characters a token;
4 of the 13,878 Jev prompts in the run logs would now pass the context.
Not a launch gate:
the largest TianqiChen666 Jev prompt was 4,779 tokens,
from a build before `ce824d933`.
The endpoint is the tokenizer,
so nothing estimates a state's size:
that refusal,
a stage with no typed question and a client with no decisions transport
now read as out of reach,
so the gather sizes its quorum without the seat and never re-asks it
(the 2026-09-09 short-bench rule and the owner's "Count as out of reach").
Rides inside the translate (15),
repair (33) and refine (5) cache versions with written accounts.

### P14: the spend report reads reckoned lines as measured calls

Status:
fixed in `44b392cb1` (guard `be030df79`;
mutation checked with a control),
found 2026-09-28 while scoping the P10 census.
`spend-read.ts` never reads the `estimated=` field,
so `spend-report` counts every reckoned `SPEND` line
(an abandoned OpenRouter attempt,
and since P1 a Bedrock attempt at its bound) as a call the provider reported,
with the reckoned tokens and cost summed in.
2,178 such lines sit in 138 top-level logs under the agent directory.
The writer changed under P1 and P7;
the reader did not,
though the writer's own TSDoc said the mark was there
"so a reader can total such lines beside the others or apart".
`readSpendLine` now reads the mark into `reckoning` (an unknown mark is a damaged record),
the tally counts `reckonedCalls` per seat,
and `spend-report` names them per seat and in a run line;
on shihai4h2 two of GLM-5.3-Flash's 207 calls were reckoned.
The marks live once in `spend-line.ts` (`SPEND_RECKONINGS`,
`isSpendReckoning`),
which the Bedrock ledger reads too,
so a mark added to the writer cannot go unread by either.
The red guard's probe read the reckoned line with no mark,
the seat with no reckoned count,
and a mark the package never writes as a clean record.
Mutation check (`p14-mutants.json`):
the reader ignoring the mark or accepting an unknown one,
and the tally ignoring it,
are each caught.
The shared list missing a mark survived,
since every case iterated the list or used one mark
and only the type check would have refused it;
`6b683e98e` lists the marks the writers pass apart from the list,
and the rerun (`p14b-mutants.json`) catches it.
No cache moves:
nothing a stage reads depends on the spend reader.

### P15: the router's attempt loop was bounded by the provider order, not by the providers that serve the call

Status:
fixed in `5b124dbaf6` (red in `e11823a963`),
found 2026-10-20 while adding NeuralWatt as the fifth provider
(`doc/decision/translation-repair-neuralwatt-provider.md`).
`createRoutingClient` in `provider-router.ts` ran `for (const attempt of PROVIDER_ORDER.keys())`,
one pass per provider in the order,
while its exits counted `serving`,
the providers that serve the model.
With four providers and models served by four,
the two counts agreed and nothing showed.
With a fifth provider in the order,
a model four providers serve got a fifth decision after its fourth attempt,
and when every attempt had been cut at its stream bound
that decision named a budget refusal instead of the stream bound.
The loop now runs `Math.max(serving, 1)` passes,
so a model nobody serves still reaches the decision that says so,
and the comment after the loop no longer says the case takes a model all four providers serve.
The rule for next time:
a loop whose exit reads one count is bounded by that same count,
never by a constant that happens to equal it today.

The same commit keeps a refusal sized to one call from holding a provider out.
`NeuralWattAllowanceError` (the day's energy cannot take this call's estimate) is a budget refusal,
so the router moves the call on;
`isCallSizedRefusal` makes both the router and the re-ask path skip `markRefused` for it,
so the small calls behind a refused large one still reach the free energy.
The guard was shown to fail by mutation:
without the `isCallSizedRefusal` check,
"HOLDS NEURALWATT OUT FOR NOTHING WHEN IT REFUSES ONE LARGE CALL" fails.
The matching check in `provider-router-reask.ts` had no case of its own when the branch was handed back;
`bd92100fa1` adds "HOLDS NEURALWATT OUT FOR NOTHING WHEN IT REFUSES ONE LARGE RE-ASK",
and with the check replaced by `if (true)` that case alone fails
(`['neuralwatt','hyper']` where `['neuralwatt','neuralwatt']` was expected).
No cache moves:
no cache key names a provider or reads the router's attempt count.

### P16: a reported cost was read as plan spend, and the first NeuralWatt call ended the day

Status:
found 2026-10-20 at 22:23 UTC by the first keyed call
(run `zheermao18`,
build `1dde8c5cd6`);
fixed on the owner's ruling
(red `265cb0d224`,
fix `05330e45c4`,
docs `1c4c10b4d1`),
landed as `5b5dca49a9` after gate g132.
The NeuralWatt design read a `: cost` comment with `request_cost_usd` above zero as a request billed to the plan,
marked the UTC day billed in the ledger
and read the provider dry until 00:00 UTC.
The first call,
DeepSeek V4.1 Flash measured at 0.0167 Wh,
reported 0.000107 USD,
so NeuralWatt went dry after one call and the run spent Hyper credit instead,
21 Hyper calls before it was stopped at 22:30 UTC.
The owner read the dashboard:
"Your call only used 60.3J,
which isn't even 0.0%",
which is the measured 0.01675 Wh counted against the free energy.
The cost is nominal on a free request.

The design rested on a reading the documentation did not state.
The energy methodology page names zero for "trial and free requests",
and the free energy page says covered requests cost nothing,
but neither names the free-energy case in the field's description;
the brief and the decision record turned that gap into a rule,
and the decision record listed it under "Not verifiable without a key" while the code acted on it anyway.
A stop built on an unverified reading of an external field failed safe here
(one call of the day's energy and some prepaid credit),
but it disabled the provider the owner put "front and center".

The rule for next time:
a provider-reported field that gates routing is verified against one keyed call and the provider's own dashboard
before a run depends on it,
and until then it is logged and not acted on.
The owner's ruling,
2026-10-20:
"From here on just use NeuralWatt normally,
I can tolerate some potential losses.
If it works wrongly I'll tell you."
The billed stop goes;
the energy ledger stays,
settled by measured energy.
Record:
`doc/decision/translation-repair-neuralwatt-provider.md`,
section "The first keyed call".

### P17: the NeuralWatt stop counted only the pipeline's own calls, by the wrong day

Status:
found 2026-10-20 at about 23:15 UTC by the owner
(`The stop logic should obviously count total free energy used per day. Every neuralwatt day starts at 15:15`);
fixed by red `6ec47d7dd0` and fix `bc429ded8d`
(the account's free energy over the day that starts at the quota's anchor),
`1567eae36e` and `538a151a44`,
the tracking-alias rows by red `3d5081a77f` and fix `5f09a2f4fb`,
docs `7a8883aefe`,
and the anchor retry by red `8f666261b4` and fix `2b94188c26`
(an unknown anchor is learned on a later reading instead of refusing NeuralWatt for the run),
with tests `dd6e8141ce` and `4dfa4ba1b3` and docs `a0fdb0aa19`.
Gate g136 on `a0fdb0aa19` ran 2680 pass and 0 fail,
and the work landed in `31ce5369e1` on 2026-10-21 (UTC).
Two defects in one meter.
The ledger counted only the calls this pipeline sent,
although the free energy is the account's,
and the design record listed the owner's other use as something "the ledger does not know",
when `GET /v1/usage/by-model` reports every covered request on the account,
current to the request
(4.095 Wh on the account against the ledger's 4.094 Wh in the same minute).
And the day ran 00:00 to 00:00 UTC on the documentation's word,
while the account's day starts at its subscription anchor,
`current_period_start` `2026-10-20T19:15:08Z` in `GET /v1/quota`,
15:15 EDT in the owner's zone.
At 00:00 UTC the meter would have handed back a full day
that NeuralWatt was still counting down.

The cause is a family with P16:
the design read the documentation and stopped there.
The usage endpoints sat in the same documentation index as the pages it cited,
and the quota reply it already parsed carried the anchor.
The rule for next time:
a provider meter is checked against the provider's own account-wide figure,
read through every usage endpoint the provider lists,
before it is called blind;
a "the meter cannot see" limit names the endpoints read and what each lacked.
Record:
`doc/decision/translation-repair-neuralwatt-provider.md`,
section "The day boundary and the account-wide count".

### P18: a refused credential withheld the seat when another provider served it

Status:
found 2026-10-20 in the log of `zheermao19`;
fixed in two rounds
(red `5e07a8bb6d`,
fix `786013e4b7`,
guards `de44d19ea7`,
docs `7863b003b1`;
then red `987b4ef553`,
fix `e9d0c5cd6f`,
docs `d3c8727aad` and `f1f7c15d94`),
landed as `624c5ff583` after gate g135
(2657 pass and 0 fail,
every step exit 0).
The Amazon Bedrock key had expired,
and every Bedrock call answered HTTP 401 "The security token included in the request has expired."
The seat tally counted those as failed asks of the seat
and withheld `gemma-4-26b-a4b-it` and `google.gemma-4-31b` for the whole run after 4 each,
though both cards carry OpenRouter sides with no holds and OpenRouter was wet all run:
no `SPEND provider=openrouter model=google/gemma...` line appears in the log.
The consolidation then judged on three or four voices of seven.
A credential belongs to a provider,
not to a seat:
one key serves every model on it.

Withholding the seat was a default and not a ruling.
B685 says "there is no rule for a 401",
and the generic tally withheld any seat that threw on every one of its asks;
the owner's rulings about Bedrock were not to top it up or renew it,
and on 2026-10-20 to leave Bedrock off rather than issue a new key.

Now one function that every provider call passes (`callHoldingRefusedCredential` in `provider/src/credential-hold.ts`)
holds a provider out of every model for the run on a 401,
the first ask and the cross-provider re-ask alike,
and the seat goes to its next serving provider.
The `SEAT CREDENTIAL REFUSED <model> on <provider>` line is written once per provider
and names only the providers that can take the call now;
a seat is withheld only when no provider serves it.
The dryness view every reach reader takes reads a credential-held provider as dry,
so the judge seats,
short benches and quorum denominators stop counting a seat none of whose providers can be asked.
`corpus-pass` builds one router per run (`corpus-pass-run.ts`),
so a 401 is learned once per run;
commands that build a client per arm or slice learn it once per client,
a limit the provider README records.
`google.gemma-4-e2b` has no provider but Bedrock and stays out.

The cause is the family of B21 five,
quorum denominators counting unreachable seats,
reached from the provider side:
a failure scoped to one provider's key was scoped to the seat.
The rule for next time:
a failure is scoped to the narrowest thing it names
(a key to its provider,
a model's refusal to that model on that provider,
a request's size to that request),
and a test pins the scope with a second provider that serves the same seat.

### P19: the NeuralWatt hold took a published mean for this pipeline's use and doubled it on an argument nothing measured

Status:
found 2026-10-21 reading the settled calls of `zheermao19`;
fixed in w250
(red `d07508fd21`,
fix `8f238ff4f4`,
docs `35e5434b80` and `a665d2d028`),
merged with the P17 work and landed in `31ce5369e1` on 2026-10-21 (UTC) after gate g136
(2680 pass,
0 fail).
The meter held each call at the energy matrix's mean for its prompt-size bucket times a margin of 2.
The run's 197 settled calls,
nearly all in the 4,096 to 16,384 token bucket,
measured DeepSeek V4.1 Flash at a mean of 0.0758 Wh against the matrix's 0.0696
and GLM-5.3-Flash at 0.3725 Wh against 0.1015:
GLM was held at about half of its use,
and DeepSeek at 1.8 times its.
The margin's argument was prefill
(a mean over a cached share h understates a fresh prompt by at most 1/(1-h)),
and the energy was in decoding:
a call's energy tracked its completion tokens with a correlation of 0.87 for both models,
which no prompt-size bucket can see.

The figures were a published mean over other people's traffic
and a margin derived from a model of the work that the first measurement did not support.
The design said to recalibrate from the first run,
and that was right;
what it lacked was a way to see how far off it was before then,
and the margin was the guess standing in for it.
The rule for next time:
a published per-request figure that sizes a hold is a placeholder until the pipeline's own settled calls replace it,
and the replacement reads the settled lines for what drives the spread
(here completion tokens,
not prompt size)
before choosing any multiplier.
Record:
`doc/decision/translation-repair-neuralwatt-provider.md`,
section "The margin".

Update of 2026-10-22 (UTC):
the figures this entry names,
0.0758 and 0.3725 Wh,
were the table's for the 4,096 to 16,384 token bucket until `a55cb37c91`.
Ledger B808 replaced them with the means of the rotation `zheermao22`,
`CuspariaKLSY22` and `XingZ6017`:
0.1958 Wh for DeepSeek V4.1 Flash and 0.9577 Wh for GLM-5.3-Flash in that bucket,
and 0.3205 and 1.2239 Wh in the 16,384 to 65,536 token bucket,
which had held the matrix's 0.1395 and 0.2009 Wh.
The rule of this entry held again:
one run of one entry was a placeholder for a workload that is a rotation of entries,
and the rotation measured 2.6 times that run's means.

Update of 2026-10-23 (UTC):
ledger B836 replaced those four means in turn with the means of `XingZ6020`:
DeepSeek 0.3044 and 0.3237 Wh,
GLM-5.3-Flash 0.0714 and 0.0556 Wh.
The GLM seat had been sent at `low` effort since B807,
so its text calls cost a fourteenth of what the default effort had measured,
and a mean taken on one effort stands for no other.

### P20: a failed first quota read stayed the run's starting reading for good

Status:
named in the lead's brief of 2026-10-21 (UTC),
from `createNeuralWattQuotaSession` read against the day anchor keeper;
the behaviour is the lead's ruling of that day;
fixed by red `da0ec11dd6` and fix `d8c2b5451d`,
with the docs commit that carries this entry.
On the red commit nine cases of the provider package fail:
two reworded cases of the quota session suite,
six new ones there,
and one new case for the end-of-run sentence;
on the fix commit the package's whole unit run,
its lint,
its type check and the 49 source scans pass.
`createNeuralWattQuotaSession` kept its one baseline attempt with `state.attempt ??= tryBaseline(...)`,
so a first quota read that failed stored `{ taken: false }` for the whole run.
`planSpend` then answered "the starting reading could not be taken" at the end,
though the day anchor keeper had read the same endpoint successfully a minute or more after the failure
(it tries its own read again through `readNow`,
no sooner than 60 seconds after a failure),
and the warning on the failed read said the end report "will have no starting reading" as if nothing could change that.
A run whose quota endpoint recovered within minutes still ended with no plan-energy line.

The cause is two readers of one endpoint keeping two failure rules:
the keeper stored a failure with the time it happened and the spacing before the next attempt,
and the session stored the failure as the answer.
A once-only memo of a read that can fail is a decision never to look again.

The fix:
`ensureBaseline` tries again no sooner than 60 seconds after the last failure on the injected clock,
with one attempt in flight serving every caller,
and takes the spacing from `NEURALWATT_QUOTA_RETRY_SECONDS` in `neuralwatt-day.ts`,
which the keeper now shares.
Tracing the callers showed that nothing asks `ensureBaseline` a second time in a run:
the keeper's retry and the key allowance's refresh both read through `readNow`.
So a `readNow` that succeeds after a failed baseline takes the baseline's place as well,
with no extra read,
and the cases that go through a client are red without it.
A baseline taken after a failure is not the run's start:
the measured spend carries `laterStartAt`,
and `describePlanSpend` says the comparison starts at that later reading,
gives its time and says that plan energy spent before it is not counted.
A baseline taken on the first try reads as before.
A run whose every attempt fails says how many attempts failed and when the last was.
The failed read is warned about once,
with wording that no longer promises an empty report.
The end report of the run package (`run/src/corpus-run/run-providers.ts`) only prints `describePlanSpend`
and needed no change;
a run-level case cannot move the clock,
so the label is pinned in the provider package's cases and not in the run package's.

The rule for next time:
an attempt at a read that can fail is stored as a result only when it succeeded;
a failure is stored with its time and the spacing before the next attempt,
and a figure measured from a start that moved carries that start wherever it is printed.
Record:
`doc/decision/translation-repair-neuralwatt-provider.md`,
section "The spend report",
and `mistake-prevention-stored-files-and-cached-decisions.md`,
"Cached decisions".

### B920: every DeepSeek call was booked at peak rates, so an off-peak run booked twice what the balance fell

Status:
fixed on 2026-10-25 (UTC), found by the read of the XingZ6021 run (its defect N8).
The id is the lead's allocation and may be renumbered at merge time.
The behaviour is the lead's ruling of that day.

The defect:
`deepSeekCostOf` (`provider/src/direct-cost.ts`) priced every call at the card's peak rates,
"because a run cannot know the holidays".
The XingZ6021 run went from Saturday 22:49 UTC to Sunday 00:55 UTC, off peak throughout,
and booked 0.147093 USD on its 15 DeepSeek lines while the account's balance fell 0.07.
The page (`https://api-docs.deepseek.com/quick_start/pricing`, read 2026-10-25 by the lead) puts peak at
01:00 to 04:00 UTC and 06:00 to 10:00 UTC, Monday through Friday, excluding Chinese public holidays,
and halves every rate at all other hours.
A holiday only turns a weekday window off peak,
so the safe upper bound needs the peak rate only inside the weekday windows.

The fix:
a call is booked at the card's peak rates when its request started inside a weekday peak window in UTC,
and at half of them otherwise (`DEEPSEEK_OFF_PEAK_MULTIPLIER`, `isDeepSeekPeakAt`).
Each window is half-open: 01:00:00 and 06:00:00 are peak, 04:00:00 and 10:00:00 are off peak,
because the page does not say whether a window's last instant belongs to it.
The weekday is the UTC weekday, which is the Beijing weekday at every instant a window covers.
An instant that is not a time is peak.
The direct client had no clock,
so `createDirectClient` and `createDeepSeekClient` take `now` (the system clock by default) and read it once,
before the first attempt of a call,
and hand it to the route as `startedAt`;
Xiaomi's route ignores it.

The red cases:
`1ebf4ca7e0` (seven cases fail: a Saturday call, a call at 23:59 on a Sunday and the edges of both windows and of the week
in `direct-cost.unit.test.ts`,
and in `deepseek-client.unit.test.ts` a Saturday call booked at half and the clock read once before the exchange),
fixed by `57c772b8ab`.
The failing lines are in the red run's log,
and the new predicate has cases of its own in the fix commit.

The readers of the booked cost:
the run spend meter (`noteRunSpend`, fed from `reportSpend`),
the per-run spend ceiling (`pass-stop-before-next.ts`, `spend-ceiling.ts`),
the spend report (`spend-read.ts`, `spend-cost.ts`),
and the SPEND lines themselves add up the figure as written and assumed no peak pricing.
The dry floor of the DeepSeek meter (`DEEPSEEK_DRY_FLOOR_USD`) and the decision record reason from peak as the worst case,
which stays true.
A ceiling now stops a run later in real time on an off-peak night,
at the same real dollars.

Recomputed from the XingZ6021 log, the 15 lines come to 0.073547 USD under the new rule against the balance fall of 0.07
(the balance is printed to two places).

The rule for next time:
a price read from a page that varies by the hour is priced by the hour of the request from an injected clock,
and the page's wording at an edge is stated where the code pins it.

Cache versions:
none moved.
The booked cost is written to the SPEND line and the run meter only,
and no cache key reads either.
