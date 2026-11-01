# The settled artifact stores the roster's block pairing

## Decision

A version 2 artifact stores the pairing its slicing was built on,
as an OPTIONAL `blockPairing` key under `preparation`,
parsed to a named absence rather than to an empty list.
This is not a version 3.

Decided 2026-08-21 by the agent under the owner's standing instruction of 2026-08-18:
"I saw your open decisions and I think they're not worth asking to me,
again,
under the principle of 'always pick whatever yields the best quality,
if you don't know which will,
prototype and measure'."

OPEN TO VETO.
Recorded rather than asked because the precedent below determines the answer.
If the owner wants a version 3,
say so and it becomes one.

## Why it is stored at all

The pairing decides which original each slice is judged against.
No later stage can repair a wrong one,
and `pass-entry.ts` calls `discardSliceCache` after the artifact write,
by design,
so the cache holding it is gone the moment an entry settles.

Recovering it meant racing a live run before it settled,
which is what had to be done on 2026-08-20 to attribute `lintong`'s deleted paragraph,
and why the `pairing2` run's attribution was unrecoverable at all.

## Why version 2 rather than version 3

`archiveText` already answered this question in the same record,
one field earlier,
and `#135` names `#96` as the same decision class.
It was added to `SettledPreparationV2` as an optional key,
listed in the reader's `allowed` set,
and parsed to `{ kind: 'unrecorded' }` against `{ kind: 'stored', text }`,
with the reasoning written into `artifact-v2-read.ts`:
"ABSENT AND EMPTY ARE DIFFERENT ANSWERS."

The version contract licenses this explicitly.
`artifact-schema-version.ts` states the rule as
"a version that does NOT move on a shape change is the failure this field exists to end,
so say so here when a field is added compatibly",
which asks for the history entry,
not for a bump.
Both compatible additions are now written there;
`archiveText` had been added without one,
so the rule was stated and then not followed.

The population settles what a bump would cost either way.
Measured 2026-08-21 over both run directories:

```text
translation-repair-runs-20260817           6 artifacts   all version 2
translation-repair-runs-flagged-20260818   5 artifacts   all version 2
```

Every one of the 11 carries neither `archiveText` nor `blockPairing`,
and none carries a `block-pairing` finding,
so the whole stored population predates both fields.
A version 3 would orphan nothing,
and an optional field strands nothing.
With no population to protect,
the tie breaks on which shape a reader is better off with,
and version dispatch for a field a reader can simply find missing is the worse one.

## What the absence means, stated rather than implied

Three distinguishable facts,
and the field does not carry all three:

-   ABSENT:
    no roster was asked.
    For every artifact that exists today this also means
    the file predates the field,
    and those two do collapse.
    They can be told apart only by
    the fact that every production entry runs through `prepareDocumentPairWithRoster`,
    which always supplies a map.
    The TSDoc says so rather than claiming a distinction the
    bytes do not carry.
-   PRESENT AND EMPTY:
    the roster was asked and committed to nothing anywhere.
-   A SECTION MISSING FROM A PRESENT LIST:
    no pairing was consumed for that section.
    Which of the several reasons applies is legible from `alignmentFindings` and not from
    this field:
    the section may have been too small to be worth a question,
    its round may have fallen back to scoring,
    or no voice may have answered.

## The voice counts go on the findings channel, not into this field

`#135` also asked for the counts already logged,
"paired N of X original and Y translation blocks,
from U usable voices of H heard".
Those were `pl.info` only and reached no artifact:
confirmed by reading `pair-blocks-stage.ts`,
and confirmed again over all 11 stored artifacts,
none of which contains the string.

They are emitted as a finding instead of stored in `blockPairing`,
from `prepare-with-pairing.ts` rather than from the stage.
The stage is asked one section at a time and cannot name which,
so counts filed there arrive as a run of identical-shaped lines attributing nothing.
Filing them as findings also leaves the pairing cache payload alone,
which stores `JSON.stringify(pairs)` and would otherwise have to grow a format.

A resumed section carries its pairs and no count line,
which is accurate:
no round was bought this run.

## What argues the other way, kept rather than buried

AN OLDER READER MEETING A NEWER ARTIFACT refuses it,
because `requireExactKeys` rejects a key its `allowed` list does not name.
That is the real cost of a compatible addition here,
and it is only tolerable because reader and writer ship together in one package
with no deployed consumers.
If an artifact ever has to be read by a build that predates it,
this reverses.

## What is still owed

Nothing in this change has run against a live pass.
The field is written by `buildSettledArtifactV2` and read by `parseSettledArtifactV2`,
both covered by unit tests with every guard shown to fail when removed,
but no settled artifact on disk exercises the stored branch yet.
The next corpus pass is what turns the population census above from
"neither key appears" into a real test of the reader.
