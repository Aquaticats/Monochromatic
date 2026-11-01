# Translation repair: adding, removing or changing a roster model

The roster is one card per model in
`package/translation-repair/shared-contract/src/model-cards.ts`,
keyed by the lists in `src/roster-id.ts`;
every catalog,
cap table and seat hold derives from the cards
(`doc/decision/translation-repair-model-cards.md`).
Run everything from the package directory
`package/translation-repair`.

## Add a model

1.  For each provider that serves it,
    print the provider side of its card off the live listing:

    ```sh
    mise run roster-card -- openrouter <served id>
    mise run roster-card -- hyper <served id>
    mise run roster-card -- synthetic <served id>
    mise run roster-card -- bedrock <served id>
    mise run roster-card -- neuralwatt <served id>
    ```

    The output is the fragment to paste,
    with a `/* not in the listing: ... */` question wherever the listing does not say
    (Bedrock lists nothing a card needs;
    Hyper lists no price;
    the cap and the raw-chars ratio are measured off completed calls,
    never listed).
2.  In `src/roster-id.ts`,
    add each served spelling to that provider's list
    (`SYNTHETIC_SERVED_IDS`,
    `HYPER_SERVED_IDS`,
    `OPENROUTER_SERVED_IDS`,
    `BEDROCK_SERVED_IDS`,
    `NEURALWATT_SERVED_IDS`)
    and add the roster id to `ROSTER_MODEL_IDS`.
    The roster id is the first serving provider's spelling:
    Synthetic,
    else Hyper,
    else Bedrock,
    else OpenRouter.
3.  In `src/model-cards.ts`,
    write the card under that roster id:
    the pasted provider sides,
    `completionCap: 'pooled-p99'`,
    and the holds the seating rule requires for a new candidate:
    `judge-unmeasured` for a single-provider candidate
    (it holds no seat until the judge fidelity probe seats it),
    `writer-unmeasured` until a producer calibration measures its writing,
    `reader-unmeasured` until a transcription is measured.
4.  In `src/roster-fixture.ts`,
    add one `SEAT_...` constant named by the model's reach,
    image input and holds,
    and add its claim to `src/roster-fixture.unit.test.ts`.
5.  Run the package checks:

    ```sh
    mise run build
    mise run lint:oxlint
    mise run lint:types
    mise run test:unit
    ```

    `model-cards.unit.test.ts` holds the lists to the cards and
    `roster-fixture.unit.test.ts` holds the seat names to the cards;
    a type error names a list entry without a card or a card without a list entry.
6.  Measure the model into its roles
    (`judge-fidelity-probe`,
    `producer-calibrate`,
    a transcription measurement)
    and remove each hold on the card as the measurement seats it,
    recording the reading in
    `doc/decision/translation-repair-roster-seating-2026-09-01.md`.

## Add a decision-only model

A model OpenRouter serves only through its decisions endpoint
(`POST /api/alpha/decisions`,
typed answers to `choice`,
`noul` and `score` questions over a state,
no chat completion)
gets a card with a `decisions` side instead of provider sides.

1.  In `src/roster-id.ts`,
    add the served spelling to `OPENROUTER_DECISION_IDS`
    and the roster id to `ROSTER_MODEL_IDS`;
    the roster id is the decisions spelling.
    No chat catalog lists it,
    so `roster-card` prints nothing for it;
    read the context length and prices off its OpenRouter model page.
2.  In `src/model-cards.ts`,
    write the card with
    `decisions: { id, contextLength, promptUsdPerMillion, completionUsdPerMillion }`,
    `completionCap: 'pooled-p90'`
    (typed answers carry no completion text)
    and the three holds.
3.  In `src/roster-fixture.ts`,
    add its `SEAT_...` constant with an empty reach and no image input,
    and its claim in `src/roster-fixture.unit.test.ts`.
4.  `DECISION_ONLY_ROSTER_IDS` (`src/model-card-derive.ts`) then lists it,
    `RUN_ROSTER` leaves it out,
    `RUN_DECISION_JUDGES` seats it once `judge-unmeasured` is gone,
    and `RUN_SELECT_JUDGES` adds it to every candidate-select bench
    (`src/decision-seat.unit.test.ts` holds these).
    The router's `decide` refuses it while OpenRouter is dry.
5.  Only the candidate-select stage has a typed question for it
    (`selectDecision` in `src/candidate-select-decision.ts`);
    every other stage a judge sits on
    (consolidation slates,
    lane contests,
    panels,
    gates)
    calls chat and reads a decision seat as out of reach,
    so a decision seat votes only where a `StageDecision` is threaded
    and no quorum waits on it anywhere else.
    A state past the seat's context is out of reach for that ballot too:
    the endpoint refuses it with HTTP 400 `max_tokens_exceeded`
    (probed 2026-09-28;
    ledger P13 in `package/translation-repair/doc/audit-ledger.md`).
6.  Measure it with `judge-fidelity-probe -- --candidates <id>`
    as for any judge;
    its ballots carry the reason prefix `typed decision`
    with the probabilities the endpoint returned.

## Remove a model

1.  Delete its card from `src/model-cards.ts`,
    its spellings from the lists in `src/roster-id.ts`,
    and its seat from `src/roster-fixture.ts`
    (re-point the seat to another model only where a model with the same reach,
    image input and holds exists;
    otherwise delete the seat and its claim,
    and the type check names every test that leaned on it).
2.  Add every spelling both providers serve it under to `src/roster-blocklist.ts`
    with the owner's words as the reason,
    so a catalog refresh labels it BLOCKED instead of proposing it again.
3.  If it held an editor,
    refiner or checker seat in `src/corpus-run/run-config.ts`,
    pass the seat on the evidence recorded there.
4.  Run the package checks;
    the remaining failures are tests that assert on that model itself,
    and each names what it measured.
5.  Record the removal as an addendum of the seating decision.

## Cull a model from every role while its card stays

When the owner culls a model that the unit fixture seats
(`src/roster-fixture.ts` names one identity per reach shape,
and a four-provider identity has no stand-in),
keep the card and empty the seats instead of deleting it:

1.  Add `owner-culled` to the card's `holds` with the owner's words and the evidence beside it.
2.  `RUN_ROSTER` and `RUN_READER_MODELS` already filter `OWNER_CULLED`,
    so every derived bench loses the seat;
    pass any static seat it held (editor,
    refiner,
    checker) in `src/corpus-run/run-config.ts` on the evidence recorded there.
3.  Leave the blocklist alone:
    it labels ids the catalogs do not carry,
    and its guard refuses a compiled catalog that seats a blocked id.
4.  Run the package checks;
    `src/corpus-run/owner-cull.unit.test.ts` holds the culled seat out of every bench on every reading.
5.  Record the cull as an addendum of the seating decision.

## Change a model's facts

Edit the card.
A price,
an ignored endpoint,
a preferred endpoint,
a measured cap or a hold is one line on the card,
and the package checks say whether anything else disagrees.
A card that names a preferred endpoint carries that endpoint's listed price,
not the model's cheapest listing that `roster-card` prints,
since that endpoint is the one the seat buys
(`GET /api/v1/models/<author>/<slug>/endpoints` lists each endpoint's price;
ledger P13 in `package/translation-repair/doc/audit-ledger.md`).

## While a pass is running

Kill the pass by pid,
build,
freeze the new dist
(`cp -r dist/final/node node_modules/.frozen-dist-<commit>`)
and relaunch on it,
as the handover's always-kill-and-relaunch rule has it.
