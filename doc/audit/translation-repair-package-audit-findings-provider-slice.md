# Package audit: findings of the provider slice

The findings of the provider slice:
routing,
clients,
locks,
meters and the catalog.

Part of [Translation-repair package audit](translation-repair-package-audit.md),
which lists every file of this record.

## provider-1, BLOCKER, verified: the `#235` path, confirmed independently at source

Same defect as `A-1`;
the reviewer reached it from the code alone and added one fact:
`src/synthetic-client.ts:264-265` builds the wire body as `model: request.modelId` with no
catalog or `hf:` check anywhere in `synthetic-client.ts`,
`synthetic-transport.ts`,
or `run-config.ts`,
while the routing client cannot misroute this way (`provider-router.unit.test.ts:210` pins it).
The TSDoc at `run-config.ts:564-568` ("THE SECOND KEY IS OPTIONAL AND ITS ABSENCE IS LOUD")
describes a refusal the code does not make.

FIXED 2026-08-26 with A-1 (`e0010019f`,
`8b289c3ab`):
the Synthetic client refuses any id outside its catalog
before the wire on both surfaces,
both keys are required,
and the TSDoc that promised a loud absence is gone
(`SECOND KEY IS OPTIONAL` has no match in `run-config.ts`,
measured 2026-08-26).
Recorded here because this entry
carried no marker of its own until the register's closing tally was measured.

## provider-2, MAJOR, verified: the second-opinion re-ask releases a Synthetic slot it never took

`src/provider-router.ts:320-324` increments `inFlightOnSynthetic` only when `chooseProvider` decides
Synthetic;
`:389` releases one slot (`using slot = heldSlot(...)`) on EVERY Synthetic call in `callOn`;
the `chatJson` re-ask at `:597` calls `callOn({ provider: elsewhere })` without `chooseProvider`.
Whenever the preferred answer came from Hyper and was unusable,
`elsewhere` is Synthetic and the
release decrements a count nothing incremented,
so the count drifts negative and the saturation
test at `:305` (`>= syntheticSlotsPerModel`) needs extra concurrent calls before overflow to Hyper
resumes.
The comment at `:384-385` asserts a pairing this path breaks;
`provider-router.unit.test.ts:519` covers only the Synthetic-to-Hyper direction.
Tracked as `#240`.

FIXED 2026-08-26 in `fd8ac6c7e`:
the re-ask takes a Synthetic slot right after the budget read,
with no
`await` between,
so the release in `callOn` pairs with a take in both directions.
Guard:
a Hyper-to-Synthetic re-ask followed by two concurrent calls still splits one slot between the providers;
with the take removed both calls land on Synthetic (2 failing lines);
restored,
passes.

## provider-3, MAJOR, verified: the roster-to-catalog proof is a type nothing instantiates

`src/hyper-catalog.ts:329` exports `HyperOnlyNamesAreServed` and no other line in `src` names it,
so adding a label to `HYPER_ONLY_ROSTER_IDS` without a `HYPER_MODELS` row type-checks and surfaces
at run time as one `NoProviderForModelError` per call,
converted by `attemptStageCall` into a lost
voice:
the `#235` class again,
one roster edit away.
Tracked as `#241`.

FIXED 2026-08-26 in `bbbd5f2c5` and `995b76f30`:
the proof is a value (`HYPER_ONLY_NAMES_ARE_SERVED`) that fails
the type check when the conditional resolves to `never`,
`hyperServesLabel` reads the Hyper catalog under a
label's exact spelling,
and three cases open `roster-reach.unit.test.ts` pinning the roster against both catalogs.
Guard:
a bogus Hyper-only roster label makes `lint:types` fail at the value (TS2322) and the two roster cases
fail naming the label;
restored,
passes.

## provider-4, MAJOR (mechanism verified, frequency unknown): text and tool deltas fold into one answer

`src/anthropic-completion.ts:249-262` pushes both `text_delta` and `input_json_delta` fragments
into `fold.answerParts` with no block attribution.
`hyper-catalog.ts:180` runs `qwen3.8-max` with `toolChoice: 'auto'`,
under which a leading text
block is legal;
such a stream yields prose plus JSON,
fails the schema,
and loses the voice,
and a
Hyper-only seat has no other stack for a second opinion.
No mixed-stream case exists in `anthropic-completion.unit.test.ts`.
How often the provider does this is unmeasured;
a wire capture decides the size.
Tracked as `#242`.

FIXED 2026-08-26 in `3bc37b4f1`:
text and tool fragments accumulate apart;
the tool arguments are the answer
whenever there are any,
prose is the answer only when no tool was called,
and set-aside prose is logged by size.
Guard:
a stream with a text block before the tool block yields the JSON alone;
with the channels glued again the
case fails on the prose prefix (2 lines);
restored,
passes.
Frequency in the wild stays unmeasured,
as the finding said;
the reader is now correct whatever it is.

## provider-5, MAJOR, verified: stale-lock takeover is `rm` then `claim`, which two starters can interleave

`src/corpus-run/runs-lock.ts:353-375`:
`await rm(path, { force: true })` then one `claim`.
The comment says "exactly one create succeeds",
which holds only when both removes precede both claims;
B-rm,
B-claim,
C-rm (removes B's live lock),
C-claim lets both passes proceed,
and the dispose at
`:380-383` removes whichever file is present,
so B's exit deletes C's lock.
Needs two starts against one stale lock,
which a relauncher can produce.
No concurrent-takeover case in `runs-lock.unit.test.ts`.
Tracked as `#243`.

FIXED 2026-08-26 in `caee057fd`:
the eviction is a rename to a name only that call knows,
so exactly one of any
number of concurrent starters evicts (`evictStaleLock`),
every acquisition carries a random token,
and the release
removes the file only while it still carries that token (`releaseIfOwned`),
saying so when it keeps one.
Guards:
two concurrent evictions report `evicted` and `gone` (fails as `evicted`,
`evicted` with a remove),
a release of a lock another holder took over keeps it (fails with the owner check removed),
and of two concurrent takeovers exactly one acquires and the other is refused with the winner's lock intact.

## provider-6, MAJOR by contract, verified: a marked class inherits a provider body excerpt

`src/request-size-refusal.ts:59` declares `messageNamesOnly: true` on `SyntheticRequestTooLargeError`,
and its `super({ status, bodyText, summary })` at `:103-117` ends `Gateway said:`,
after which the
parent (`src/completion-shape.ts:82-89`) appends `bodyText.slice(0, BODY_EXCERPT_LIMIT)`.
The marker's contract (`stated-refusal.ts`,
`refusal-text.ts`) excludes anything read from a provider body.
The observed gateway wording carries only a byte position,
so nothing has leaked;
a gateway that echoes request bytes would print corpus text at the boundary.
Tracked as `#244`.

FIXED 2026-08-26 in `0fb6a8ad8`:
`SyntheticHttpError` (renamed `ProviderHttpError` on 2026-10-19) takes an excerpt policy,
the marked subclass withholds the
gateway's words from its message and keeps them on `bodyExcerpt` for the log.
Guard:
a gateway body echoing a sentinel leaves the message clean and the excerpt carrying it;
with the excerpt quoted again two cases fail;
restored,
passes.

## provider-7, MINOR, verified: a budget refusal during the re-ask is not caught

`src/provider-router.ts:590-604` calls `callOn` bare for the second opinion;
a 429 or 402 there
becomes a throw instead of the first outcome,
and `markRefused` starts that provider's cooldown one
call late.
Folded into `#240`.

FIXED 2026-08-26 in `2376b7d14`:
the re-ask goes through `replyOrBudgetRefusal`,
which reads a 429 or a 402
as the re-ask not happening:
the first answer is returned and `markRefused` runs on the call the refusal arrived on.
`#240` closed the slot half of this site;
this is the refusal half the entry named.
Guard:
with the helper rethrowing,
the case
`KEEPS the first answer and starts the other provider's cooldown when the re-ask is refused on budget` fails;
restored,
passes.

## provider-8, note: the run log carries provider body excerpts and 80-character openings

`src/stage-call.ts:247` logs `String(error)` of a `SyntheticHttpError` (renamed `ProviderHttpError` on 2026-10-19)
(up to 600 body characters);
`src/stream-cut.ts:40` sets `OPENING_CHARS = 80`.
Both are recorded decisions (`#75`),
and the log is treated as corpus-bearing;
no change.

## provider-9, MINOR, verified: the catalog-drift instrument compares against a stale copy

`src/corpus-run/model-catalog-compare.ts:26-33`:
`CATALOG_MODEL_IDS` still lists
`hf:zai-org/GLM-4.7-Flash` and omits every Hyper-only id,
so the tool built to catch catalog
removals reports a departed model as expected.
Fix:
derive from `Object.keys(SYNTHETIC_MODELS)` and delete the copy.

FIXED 2026-08-26 in `2376b7d14`:
`CATALOG_MODEL_IDS` is `Object.keys(SYNTHETIC_MODELS)`;
the Hyper-only ids are
absent on purpose,
since the fetch compares Synthetic's own endpoint.
Guard:
with the departed id appended to the derived list,
the case `IS the compiled Synthetic catalog` fails;
restored,
passes.

## provider-10, MINOR, verified: TSDoc describes a roster that no longer exists

`src/chat-contract.ts:174` (example uses the departed `hf:zai-org/GLM-4.7-Flash`),
`src/synthetic-catalog.ts:134` ("two of the six"),
`src/corpus-run/run-config.ts:111,152,173` (GLM-4.7-Flash as the third editor) and `:330-335`
("Exactly two models read images").
The `#235` diagnosis started from this prose.

FIXED 2026-08-26 in `2376b7d14`:
the seat history in `RUN_MODELS`'s TSDoc opens with the current seating (ten seats
across two providers,
the measured editors,
refiners and checkers) and keeps each earlier rule in the past tense with
its date;
the reader paragraph counts six readers of ten,
measured as `readsImages: true` on 2 Synthetic and 4 Hyper
rows;
the chat contract's example names `hf:zai-org/GLM-5.2`.
Prose only,
no guard.
The "two of the six" the entry placed at `synthetic-catalog.ts:134` is not in that file (`rg` over the whole of `src`
finds the phrase nowhere),
so nothing was changed there;
the file's remaining mentions of the departed model are
dated history.

## provider-11, note: a shared meter read aborted by its first caller resolves wet for every sharer

`src/provider-budget.ts:318-322`,
documented and pinned by `provider-budget.unit.test.ts:268`;
the 429/402 re-route corrects the one decision it can mislead.
Accepted.

## provider-12, MINOR, verified: raw `console.log` in two library modules, and a handle outside `using`

`src/corpus-run/git-command.ts:54`,
`src/corpus-run/runs-lock.ts:173,222,344,351` (`TLG`);
`runs-lock.ts:258-263` opens with `'wx'` and closes by hand,
so a write failure leaves an empty
lock the next pass reports unreadable and takes over (`PP3`).

FIXED 2026-08-26 in `2376b7d14`:
`runs-lock.ts` logs through `tagged({ tag: 'runs-lock' })`,
warn for a lock held
by another process,
an unreadable file and a lock no longer this acquisition's,
info for the two takeovers;
`git-command.ts` logs its fallback through `tagged({ tag: 'git-command' })`;
and `claim` holds its `wx` handle in
`await using`,
so a write that fails still closes it,
the empty file such a failure leaves being what the next pass
reads as unreadable and evicts.
The sink suite `sink-names-only.unit.test.ts` reads the unreadable-lock line and caught the move:
the whole-suite
run after `2376b7d14` failed its LOCK case (782 PASS,
2 FAIL,
exit 1),
and `c7f2a5075` makes the suite's collector
capture the logger's console methods and drops the `LOCK ` prefix from the expectation.
An earlier draft of this paragraph said no test read those lines;
the search behind it was anchored on the old
prefix with a leading quote and missed the case.
The disposal itself has no guard.

## provider-13, MINOR, verified: a malformed round line parses to `NaN` silently

`src/corpus-run/run-timing-parse.ts:297-298`:
`heard: Number(counts[0])` with no check that the
ratio split into two integers;
only the duration fields refuse.

FIXED 2026-08-26 in `2376b7d14`:
`countIn` refuses an empty or non-digit field and the ratio must split into exactly
two fields;
both throw inside the existing `round line unreadable` wrapper.
Guard:
with `Number` restored and the two-field check removed,
the case
`THROWS ON A RATIO THAT IS NOT TWO WHOLE NUMBERS` fails;
restored,
passes.

## provider-14, MINOR, verified: each of provider-1 to provider-5 lacks the test that would have caught it

No re-ask landing on Synthetic,
no mixed text-then-tool stream,
no roster-subset assertion,
no bare-client refusal of a non-`hf:` id,
no `RunConfigError` marker check at the boundary,
no concurrent stale-lock takeover.
Each fix carries its guard,
committed first per `GFP`.

CLOSED 2026-08-26 by the MAJOR landings,
each recorded under its own entry with its guard:
the re-ask landing on
Synthetic (`#240`,
provider-2),
the mixed text-then-tool stream (`#242`,
provider-4),
the roster-subset proof
(`#241`,
provider-3),
the bare client's refusal of a non-`hf:` id and the `RunConfigError` marker at the boundary
(`#235`,
A-1 and provider-1),
and the concurrent stale-lock takeover (`#243`,
provider-5).
Nothing further to land.
