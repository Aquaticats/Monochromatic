# Translation repair: one model card per roster model

## Status

Decided and landed 2026-09-16.
The owner asked for the change
("Adding / removing / changing models should never be this difficult.
And given how fast the AI space is moving,
a reusable workflow or mise task or something should be built")
and,
asked which shape,
delegated the choice
("Please make the decisions yourself as these aren't really design decisions").
Commits:
`ea0024e3c` (the cards and every derivation),
`527681ba4` (the `roster-card` task),
`e53f4c295` (Synthetic's dollar-prefixed prices),
`099b69787` (the role-named fixture seats).

## What it replaces

Before this,
seating one model meant writing it into eight source files
(`roster-id.ts`,
the provider catalog,
`completion-cap.ts`,
`openrouter-abandoned-spend.ts`,
`corpus-run/hyper-price.ts`,
the hold sets in `corpus-run/run-config.ts`,
`corpus-run/run-seats.ts`
and a test fixture),
and unseating one meant finding them all again.
The DeepSeek V4 unseat of the same morning
(`a88f52d5b`)
touched those eight files and fifty-eight unit-test files
with 129 quoted ids,
plus a golden key literal and a roster count.
That measurement is the case for the change.

## The three decisions

### One card per model, from which every table derives

`model-cards.ts` holds one record per roster model,
keyed by the roster id,
carrying every provider's side
(served spelling,
image input,
output ceiling,
prices,
ignored endpoints,
raw-chars ratio,
Bedrock route and stream end),
the measured completion cap or the pooled percentile it falls back to,
and the seat holds
(`judge-unmeasured`,
`writer-unmeasured`,
`reader-unmeasured`,
`translator-dropped`,
`wide-seat-dropped`,
`late-judge-dropped`,
`openrouter-withheld`,
`openrouter-dropped`,
`hyper-slow-select`).
`model-card-derive.ts` projects the cards into
`SYNTHETIC_MODELS`,
`HYPER_MODELS`,
`OPENROUTER_MODELS`,
`BEDROCK_MODELS`,
`COMPLETION_CAP`,
the abandoned-spend ratio table,
`OPENROUTER_WITHHELD`,
`OPENROUTER_DROPPED_SEATS`,
the seated-judge sets and every hold set in `run-config.ts` and `run-seats.ts`.
Seat decisions
(which model holds the third editor seat)
stay as id lists in `run-config.ts`:
they are measurements,
not facts about a model.

Rejected:
a drift-audit task over the eight files
(reports the fan-out instead of removing it)
and a data file with a generator
(a codegen step and generated files,
against the repo's preference for TypeScript once logic is involved).

### The served-id unions stay as lists, beside the cards

Declaration emit under `isolatedDeclarations` cannot infer a nested object literal,
measured on the first build
(`TS9010`,
`TS9013` and `TS9017` on every card),
so the literal unions the catalogs key on
(`RosterModelId`,
`SyntheticServedId`,
`HyperServedId`,
`OpenRouterServedId`,
`BedrockServedId`)
come from `as const` string lists in `roster-id.ts`,
which imports nothing.
The cards are a `Record` over the roster list,
so a roster id without a card and a card without a roster id are type errors;
a card naming a served spelling missing from its provider's list is one too,
since each card side is typed by that list's union;
and `model-cards.unit.test.ts` holds each list to the cards
(every listed spelling on exactly one card,
every derived catalog keyed by its list,
every roster id spelled the first serving provider's way:
Synthetic,
else Hyper,
else Bedrock,
else OpenRouter).
The origin buckets
(`HYPER_ORIGIN_ROSTER_IDS`,
`BEDROCK_ONLY_ROSTER_IDS`,
`OPENROUTER_ONLY_ROSTER_IDS`)
derive from that naming rule.

The derived records are built by `Object.fromEntries`,
whose keys the type system widens to `string`;
`keyedBy` is a runtime-checked type predicate over exactly the list's keys,
so no cast is needed and a card-list disagreement throws at load with both sides named.

### Tests name seats by role

`roster-fixture.ts` exports one seat per roster model,
named by the reach,
image input and holds a test leans on
(`SEAT_HYPER_ONLY`,
`SEAT_SYNTHETIC_VISION_WITHHELD`,
`SEAT_BEDROCK_ONLY_VISION_UNSEATED`,
and so on),
and `roster-fixture.unit.test.ts` holds every name's claim against the cards and the router's reach.
The 172 test files and the archive fixture read the seats instead of 1,361 quoted ids.
Unseating a model now changes the mapping line and the tests that assert on that model itself.
The seats are `as const` literals so a fixture object built from one keeps the literal type,
and each is its own export because declaration emit infers a string literal's type and not an object literal's.

Rejected:
migrating on touch,
which would have left the next removal editing tens of files.

## The task

`mise run roster-card -- <synthetic|hyper|openrouter|bedrock> <served id>`
reads that provider's live listing with the mise-injected key
(never printed),
finds the row,
and prints the provider side of the card in `model-cards.ts` field names,
with a question in place of each field the listing does not carry,
followed by the three steps that seat the model.
Verified live on 2026-09-16:
the OpenRouter listing reproduces Mercury 2.5's card field for field,
Hyper's row for DeepSeek V4.1 Flash prints,
Synthetic's row for Kimi-K3 prints its dollar-prefixed prices as dollars per token,
and an unknown id is refused by name.
Hyper's listing that day reported `max_output_tokens` 32768 for V4.1 Flash
where the card carries the 26214 verified on 2026-09-11;
the card is left as measured and the drift is noted for the next reading.

Rejected:
a command that rewrites the source for add and remove
(a program editing TypeScript,
saving one paste),
and no task at all
(the listings' field names are the part a person gets wrong).

The entry module exports nothing because an exported entry is bundled as a shared chunk,
in which `import.meta.main` is false and the task prints nothing;
the command-line reader lives in `roster-card-ask.ts` for that reason.

## Consequences

- Adding a model is one card,
  its spellings on the lists in `roster-id.ts`,
  one seat in `roster-fixture.ts` with a truthful name,
  and the package checks;
  the steps are `doc/runbook/translation-repair-roster-change.md`.
- Removing one is deleting the card,
  its spellings and its seat,
  adding its spellings to `roster-blocklist.ts` with the owner's words,
  and re-pointing the tests that assert on that model.
- Every catalog comment that recorded per-entry evidence now sits on the card or in the seating decision;
  the catalog files keep the method notes.
- The guard that a Hyper-origin roster name is a Hyper spelling
  (`HYPER_ORIGIN_NAMES_ARE_SERVED`)
  still holds by type,
  now over derived unions.
