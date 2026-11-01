# Translation repair history 2026-08-21 to 2026-08-23: the gateway's real cap and the verse rule reaching a judge

The session of 2026-08-22:
the picture ceiling re-measured against the gateway rather than a model,
the verse rule that reached both producer sheets and no judge,
the lesson to test the derivation rather than the parameter,
and the work left open at the end of that session.

Part of [Translation repair history: 2026-08-21 to 2026-08-23](translation-repair-history-2026-08-21-to-23.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Session 2026-08-22: the gateway's real cap, and the verse rule finally reaching a judge

Three tasks landed.
Two of them share a shape worth naming before the details:
A FACT THE PIPELINE COMPUTES CAN REACH HALF THE ROUND AND STOP THERE,
and nothing fails,
because both halves keep answering.

### `#168`: the picture ceiling was guarding against the wrong authority

The ceiling on a picture asked what a MODEL would read.
The thing that actually refuses is the GATEWAY,
and it refuses by describing our JSON rather than its own limit:
a body over the cap comes back as `400` naming a parse failure at a byte offset,
so a request refused for being too big reads as a request that was malformed.

-   The ceiling is now 7 MiB,
    measured against what the gateway carries rather than derived from what a model reads.
    The eight mebibytes standing until today mapped onto a body of 11185335 bytes,
    which is 699575 ABOVE the only size measured to pass:
    the number guarding the request permitted requests the gateway rejects.
-   The unavailable reason is renamed `too-large-for-transport`.
    A reader who saw the old spelling would look for a model that refused,
    and no model ever did.
-   `failureForReply` re-raises that refusal as `SyntheticRequestTooLargeError`,
    on three signals together:
    status `400`,
    the gateway's parse-failure wording,
    and a body larger than anything measured to arrive.
    Never as a pre-flight refusal,
    because only the PASSING size is exact
    and a client-side guard would reject gray-zone bodies the gateway may well carry.
-   The trap in measuring it:
    the corpus is Chinese,
    so `.length` reads about a third of the wire size.
    `Buffer.byteLength` is required,
    and there is a test that fails on exactly that substitution.

A coverage gap was found by inspection rather than by a failure:
nothing pinned the ceiling.
The oversize fixture cleared both the old number and the new one,
so a revert would have passed the whole suite.
`PAST_THE_NEW_CEILING_ONLY` sits between them and is the only thing that can tell them apart.

### `#177`: the verse rule reached both producer sheets and no judge at all

`#150` made the verse rule outrank the page rule for PRODUCERS,
and said so in the rule text rather than by ordering,
because a model resolves a contradiction however it likes when neither side defers.
`#176` gave the same rule to the consolidation's producers.
Neither reached a judge,
and `judgeTranslateSlate` had no line-structure parameter at all,
so there was nothing to forward and nothing to notice missing.

WORSE THAN SILENCE.
Criterion four tells every judge that a shape the ORIGINAL does not have is not a fault,
because this archive's pages split,
merge and quote passages of their own accord.
On a verse slice that is the right rule pointed the wrong way:
there the ORIGINAL is what carries the line structure and the page is what merged it.
A producer obeying the rule unmerges,
and its judge had been handed a reason to prefer the merged rival.
Measured under `#162`:
211 slices across 34 entries of the 92 pairs are governed.

What landed:

-   `TRANSLATE_LINE_STRUCTURE_CRITERION`,
    phrased for SELECTION rather than production.
    A judge builds nothing and chooses between candidates already written,
    so the same fact has to arrive as a test it can apply to each one.
-   It is inserted BY IDENTITY immediately ahead of the rule it overrides,
    and names that rule outright rather than relying on list order.
    The clause both criteria state is ONE SHARED CONSTANT,
    so the override cannot come to quote a sentence no judge is given.
-   `lineStructured` is REQUIRED on `judgeTranslateSlate`.
    `#176` carried the same fact as an optional field for a day,
    no caller set it,
    and every verse passage was quietly told it was prose.
    Required,
    the compiler named all seven call sites at once.
-   Both cache versions moved,
    `TRANSLATE_SLICE_CACHE_VERSION` to 6 and `CONSOLIDATE_CACHE_VERSION` to 2.
    Both keys ALREADY carried `lineStructured`,
    so this change is invisible to the material:
    without the bumps a governed slice settled yesterday resumes
    with a judgment made under a sheet that never mentioned lines.
    The owner may veto the discard;
    it is recorded rather than asked,
    since quality decides and quota is not a constraint.

### The lesson that generalises: test the derivation, not the parameter

Every case that writes a flag out by hand passes whether or not any caller ever computes it.
`#177` carries a governed and an ungoverned round at four levels,
and the GFP proves the levels are not redundant:
breaking the STAGE's handover fails the stage and whole-document cases
and leaves the two that call the judging half directly passing.

`document-verse-rule-reaches-the-wire.unit.test.ts` is the one that drives a real
document and lets the pipeline derive the verdict itself.
It also asks the SHIPPED predicate about its own fixture,
so a fixture that quietly stopped being line-structured is reported as a broken fixture
rather than passing as a clean null.

### Open, in the order they matter

-   `#178`,
    split out of `#162` today:
    the consolidation's judges are picture-blind and window-blind.
    The picture half is a LIVE REGRESSION rather than a longstanding gap,
    because `#176` gave the producers pictures this morning,
    so the judges now weigh proposals written against evidence they cannot see.
    Its window half puts the window into the consolidation key,
    which unlike `#176`'s picture append discards nearly every settled consolidation:
    nearly every slice of a multi-slice document has a window.
-   `#162` keeps the pre-slate wrap alone,
    now blocked on `#177` and `#178`,
    and its `!lineStructured` gate removes the degradation arm by construction.
    Its savings claim still owes the measured band-pair fork run.
-   `#163`,
    `#175`,
    and the five older pending tasks are unchanged.
