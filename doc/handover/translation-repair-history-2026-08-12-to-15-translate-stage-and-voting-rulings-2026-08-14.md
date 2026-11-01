# Translation repair history 2026-08-12 to 2026-08-15: the translate stage and three voting rules the user changed

The stage that renders a slice instead of repairing it,
the user's rulings that no stage waits for its whole roster
and that producers may judge with self-votes counted at half weight,
the cache version bump,
and the review findings tracked but not fixed.

Part of [Translation repair history: 2026-08-12 to 2026-08-15](translation-repair-history-2026-08-12-to-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Session 2026-08-14: the translate stage, and three voting rules the user changed

The stage that renders a slice rather than repairing it exists and is tested.
Three user decisions landed with it,
and every one of them reaches wider than
the new lane.

### The lane itself

`runTranslateStage` in `package/module/translation-repair/src/translate-stage.ts` (moved on 2026-10-18 to
`package/translation-repair/translate-stage/src/translate-stage.ts`):

-   fans `buildTranslateMessages` to the translator roster through
    `gatherStageVoices`,
    one whole-slice rendering per model
-   assembles the slate in `translate-candidates.ts`,
    with the archive's own
    translation standing among the fresh renderings as one more candidate
-   ROTATES the slate by a hash of the source before judges see it,
    so the
    incumbent does not sit in one ballot position.
    Rotation rather than a
    shuffle,
    and keyed on the slice rather than on a draw,
    because a resumed
    slice has to ask the judges the same question a fresh one did
-   ships the incumbent on every failure path,
    and records WHY separately from
    WHAT:
    `decision` distinguishes `judged` from `declined-indecision`,
    `declined-rejection`,
    `sole-candidate` and `no-candidate`,
    so a tie that
    keeps the incumbent is never counted as the incumbent winning
-   judges a sole FRESH candidate rather than shipping it unexamined,
    and skips
    the round only when the sole survivor IS the incumbent,
    where nothing could
    change

`CandidateProducer` gained an `incumbent` variant carrying `matched`,
the models
that independently produced identical text.
A stand-in model id was rejected on
both counts:
it would discount a model that never saw the text and inflate the
producer count the roster guard is arithmetic over.
Incumbency survives a
duplicate collapse,
so "the human translation was kept" cannot be reported as
"a model rewrote it identically" on the slices where the two are the same bytes.

The translator prompt now carries the line-structure fact,
which the editor
addendum carried and the prototype did not.
`Toka_ls` is the case:
21 source
blocks at median 22 characters against 18 target blocks at median 101.
A
translator shown only the merged translation reproduces the merge.

### Decision: no stage waits for its whole roster

User,
2026-08-14:
"full roster should never be a retry target for anything,
because that will block everything even if only one or two model of the
provider is degraded for the day."

`retryTarget` is gone from `gatherStageVoices` entirely rather than left unused,
and the editor and refiner stages that passed `full-roster` no longer do.
This
REVERSES the 2026-08-12 choice recorded under "Every fan-out stage now has a
quorum one voice cannot meet",
which was made believing Kimi-K3 was dead.
The property that choice protected survives without it:
editors,
refiners and
checkers all sit at three with a quorum of two,
so no stage is decided by one
model either way.

`stage-roster-incomplete` is now emitted whenever a roster ends short rather
than only under the retired target,
because the ratio is what the per-model
`stage-voice-lost` findings cannot carry.

### Decision: producers judge, and self-votes count half

User,
2026-08-14:
"A model can both be a translator and a judge... its own
judgement would still be somewhat valuable",
then "Self-judge and self-vote
should always be allowed,
just given less weight."

`selectBestCandidate` no longer removes producers from the judge roster.
`SELF_VOTE_WEIGHT` is `1 / 2`,
`MIN_SELECTION_WEIGHT` is 2,
and the arithmetic
carries the old property:
a single-model candidate draws at most half a vote
from its own author and a three-contributor composite at most three halves,
so
self-votes alone can never select anything.
A model can add to a case
disinterested judges already made and cannot make one.

The user also corrected the framing twice,
and both corrections are load-bearing:

-   a model backing its own work is NOT the ordinary case.
    The judge sheet is
    anonymized and says so,
    so a producer cannot see which candidate is its own;
    the discount corrects a tilt rather than a declared preference.
-   asking why a model would ever abstain:
    it abstains because the sheet offers
    `0` for "no candidate is acceptable" and asks for it by name.
    That is what
    makes a `rejection` disposition different from a tie.

Anonymity was verified rather than assumed:
`candidate-select-wire.ts` renders
candidates as `CANDIDATE 1..N` with fenced text and tells the judge it cannot
know who wrote what.
The incumbent rides the ballot unlabelled,
and its text is
deliberately NOT repeated as evidence,
which would have identified it.

Every ballot now leaves the selector with its model,
its choice,
its reason and
its weight.
Reasons reached a log line and nothing durable before,
and one lost
pipe on 2026-08-13 already erased twenty minutes of them.

### Slice cache version 25

Both decisions change who was heard and who decided while touching no prompt,
which is precisely the class the structural guard cannot catch.

### What the sol review found that is NOT fixed

Relayed by the user and tracked rather than acted on,
because each is its own
piece of work:

-   `#88` a whole-candidate validator.
    The apply gate's preservation,
    footnote
    and line-structure policies have no envelope to bound them on a whole-slice
    replacement,
    and faking one envelope fails both ways.
    The pipeline-shape
    decision doc claimed those checks survive unchanged;
    that claim is wrong and
    is corrected there.
-   `#89` the driver,
    the translate-shaped slice outcome,
    its own cache guard,
    and a per-slice `sliceSelections` field.
    The artifact cannot currently
    record which slice kept its incumbent.
-   `#90` slicing sizes source runs by the incumbent's length,
    so the worse the
    coverage the larger the call,
    and one-sided sections are not sliced at all.
-   `#91` checker independence never reads `refinerModelIds`.
-   `#92` footnotes cross slices,
    so the reassembled document needs its own
    check,
    plus the token,
    latency and truncation measurements to take before
    any long run.

### Verification

Types,
oxlint (0 warnings,
0 errors) and the full unit suite pass.
Six new cases cover the lane,
five of them about something absent:
a slice with
no translation,
a reply wrapped in prose,
a whitespace reply,
judges declining,
and a slate where every translator reproduced the incumbent.
Judges in those tests are scripted by the TEXT they see rather than by candidate
number,
since pinning index 1 would assert the rotation instead of the decision.
