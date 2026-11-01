# Translation repair history 2026-08-23 to 2026-08-24: the settled artifact's index vocabulary across generations 3 and 4

The rename of the shipped and withdrawn index arrays and the critics record as generation 3,
the rename of the stamped index as generation 4 and the name collision it exposed,
how both generations are read and verified at the boundary,
and the read-any-generation dispatch that never learned generation 3.

Part of [Translation repair history: 2026-08-23 to 2026-08-24](translation-repair-history-2026-08-23-to-24.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The settled artifact speaks one vocabulary, as generation 3 (`#94`, 2026-08-24)

`withdrawnChunkIndices` sat beside `withdrawnSliceCount` in the same record,
about the same things.
`#99` is what that class of confusion costs.

Renamed,
at 351 sites:

-   `shippedChunkIndices` becomes `changedSliceIndices`.
    Not `shippedSliceIndices`:
    the incumbent ships whenever the archive wording stands,
    so "shipped" reads ambiguously,
    while both arrays name slices whose returned text DIFFERS from the archive.
-   `withdrawnChunkIndices` becomes `withdrawnSliceIndices`.
-   `chunkCritics` becomes `sliceCritics`,
    with `ChunkCriticRecord`,
    `ChunkCriticView`,
    `buildChunkCriticRecords` and `decodeChunkCritics`.

`chunkIndex` is NOT in this change.
It gets its own change and its own verification,
which landed as `#204` below.
The claim recorded here that it reaches cache keys was WRONG,
and is corrected in that section:
it reaches none of the six cache-key builders.

### Why the version moved, and why the symbols did not

A key rename is a shape change,
and `artifact-schema-version.ts` states that a version which does not move on one
is the failure that field exists to end.
So the pass writes generation 3.

`artifact-v2-build.ts` passes `lanes.repair` and `lanes.translate` through whole as `result`,
so the internal field names ARE the wire keys.
There was no internal-only step to land first:
holding the bytes still would have meant a mapping layer built only to be deleted by the next change.

The 63 `V2` symbols at 808 sites across 63 files did NOT follow the integer.
`V2` there names the TWO-LANE shape,
not the number,
and the contract file now says so.
Renaming that family to something version-free is worth doing and is tracked separately.

### How both generations are read

`artifact-key-vocabulary.ts` holds three key names per generation and nothing else.
`parseSettledArtifactV2` reads the recorded version,
selects one table,
and threads it two hops to `parseLanesV2` and the two evidence parsers.
`attribution-read.ts` does the same for `sliceCritics`,
treating an unversioned artifact as generation 1.

No artifact is ever tried under two spellings.
A generation 3 stamp over generation 2 keys is refused,
which is correct:
it is not a generation 3 file.

Generation 1 keeps its own spelling on disk,
read and written.
Nothing writes that generation any more,
and re-spelling it would strand the files it left behind in exchange for nothing.

`assertResumableSchemaGeneration` moved with the writer,
so a run directory of generation 2 artifacts is now foreign to a resuming pass.
That is what the guard is for.

### Verified

Suite exit 0,
lint 0 warnings and 0 errors,
types clean,
build clean.

GFP on the new spelling guard:
making the writer emit `shippedChunkIndices` under a generation 3 stamp
built cleanly and failed the suite at the `hasOwn` check,
which is the failure a fixture-only suite could never produce.

At the boundary,
through the shipped bundle,
over the six real generation 2 artifacts in `~/temp/agent/vub-run1-20260821`:
all six read,
all six generation 3 twins read,
and the two parses are identical on every interpreted field.
They differ only inside `lanes.*.raw`,
which is the file's own record passed through unread
and so carries the file's own spelling by design.
All six mislabelled files,
generation 3 stamp over generation 2 keys,
were refused.

### And live, end to end

One entry through the real pipeline,
`gaoyanger` into `~/temp/agent/gen3-vub-2026-08-24`,
exit 0 in 975 seconds:

```text
TALLY gaoyanger status=SETTLED slices=2 repairStatus=repaired repairIssues=4 repairAccepted=3
repairResolved=3 repairChanged=1 translateChanged=1 documentsDiffer=1 selection=contested
```

Read back off disk:
stamped generation 3,
all three current keys present,
zero older keys,
and its own reader accepts it.
`verify-published` then agreed:
`1 of 1 pages carry every wording their artifact promised, at the length it implies`.

Charm Hyper was dry for this too,
so five of the ten seats lost their voice at every stage
and the entry settled anyway on the five that answered.
That is the resilience the owner asked for,
observed rather than asserted.

## The stamped index is `sliceIndex`, as generation 4 (`#204`, 2026-08-24)

Pass two of the vocabulary rename,
and it was not the mechanical sed the plan described.
Two recorded premises were wrong,
and each was corrected by measuring before acting.

### It reaches no cache key

`chunkIndex` appears in zero of the six cache-key builders.
The keys hash positional arrays,
and `repair-slice-key.ts` records
that the slice index was removed from the key at version 26.
The earlier note in the `#94` section is corrected above.

### The name was already taken, by a different concept

`sliceIndex` already existed:
122 occurrences in 19 files,
meaning POSITION in `prepared.slices`,
with `neighbouringSource`,
`neighbouringIncumbent` and `slicePictures` throwing on a non-position.

A blanket rename collapses two concepts into one name
and recreates exactly the defect `#99` was opened on.
The attempt surfaced as two `TS2451` redeclarations in `translate-document.ts`
and would have been SILENT everywhere else.
It was reverted whole with `git checkout -- package/module/translation-repair/src` (moved on 2026-10-18 to
`package/translation-repair/*/src`) and split in two:

1.  `sliceIndex` to `slicePosition`,
    122 sites in 19 files,
    freeing the name.
    Commit `6a3b24533`.
2.  `chunkIndex` to `sliceIndex`,
    1734 sites in 194 files.
    Commit `49e5a41cd`.

Reading what broke,
rather than trusting the count,
is what caught this.

### The wire moved too, and generation 3 is a mixture

`artifact-v2-project.ts` maps the index explicitly,
so holding the wire still was available here
in a way it was not for the arrays.
It was rejected:
a file spelling one half the new way and the other half the old way
is the defect this work exists to end.

So `artifact-key-vocabulary.ts` gains a fourth field and a fourth row,
and generation 3 becomes what it always was on disk:
a MIXTURE.
Confirmed against the one real generation 3 artifact:
its `result` spells `sliceCritics` and `changedSliceIndices`,
and every one of its twenty-odd index keys spells `chunkIndex`.

`parseSettledArtifactV2` now names the generations it reads in a list
rather than a chain of comparisons,
and five parsers plus one lane envelope
take the vocabulary rather than naming a key.

### Verified

Suite exit 0,
lint 0 warnings and 0 errors,
types clean,
build clean.

Three guards GFP-proven,
each shown to fail with the guard removed and then restored:

-   Collapsing the generation 3 mixture into the current table
    fails the vocabulary dispatch cases and the cross-generation equality,
    and makes the one real generation 3 artifact unreadable.
-   Making the writer spell the ledger index the old way while stamping generation 4
    fails the end-to-end settle in `pass-entry.unit.test.ts`.
-   Letting a ledger row tolerate both spellings
    fails the case pinning that a relabelled body is refused.

At the boundary,
through the shipped bundle,
all 42 real two-lane artifacts under the agent scratch root read with no refusals
and no blank indices:
41 of generation 2 over 492 ledger rows,
and 1 of generation 3 over 4.

That null result has a positive control.
Each file read under a generation it does not carry is REFUSED,
and each refusal names exactly the key the wrong table asked for:
generation 3 read as 4 refuses at `lanes.repair.delivery[0].chunkIndex`,
read as 2 refuses at `lanes.repair.result.shippedChunkIndices`,
generation 2 read as 4 refuses at the same delivery key,
and read as 3 refuses at `lanes.repair.result.changedSliceIndices`.

### And live, end to end

One entry through the real pipeline,
`gaoyanger` into `~/temp/agent/gen4-vub-2026-08-24`,
exit 0:

```text
TALLY gaoyanger status=SETTLED slices=2 repairStatus=repaired repairIssues=9 repairAccepted=7
repairResolved=7 repairFindings=55 repairChanged=1 translateStatus=complete translateChanged=2
documentsDiffer=2 pageChanged=1 pageSilent=0
```

Read back off disk:
stamped generation 4,
`sliceIndex` the only index spelling anywhere in the file,
zero of the three older array keys,
and its own reader accepts it with all four ledger rows carrying a numeric index.
`verify-published` then agreed:
`1 of 1 pages carry every wording their artifact promised, at the length it implies`.

Charm Hyper was dry for this too,
so five of the ten seats had no voice at any stage
and the entry settled anyway on the five that answered.

## The read-any-generation dispatch never learned generation 3 (`#206`, 2026-08-24)

Found sweeping for stragglers of the rename above,
and it is the more interesting find.

`readSettledArtifact` in `artifact-read.ts` is the barrel's entry point for reading an artifact
of any generation.
It compared the recorded version against version 1 and version 2 and nothing else,
so a generation 3 or 4 body fell through to the final throw
and was reported as a generation nothing reads.

THE COMMENT ABOVE THAT THROW CALLED ITSELF UNREACHABLE and named this exact drift:
"the day a generation is added to that list and forgotten here".
`#94` added generation 3 to `KNOWN_ARTIFACT_SCHEMA_VERSIONS` and did not add it here,
so every generation 3 artifact has been refused by this path for as long as the generation existed.

### What it did not cost

Nothing inside the package calls it.
The pass reads through `parseSettledArtifactV2` directly,
and `verify-published`,
`assertResumableSchemaGeneration` and the attribution reader all bypass it.
So no run was affected and no artifact was misread.
A consumer of the barrel would have been.

### The fix, and why the list moved

`TWO_LANE_GENERATIONS` now lives in `artifact-v2-contract.ts` and is exported,
because TWO places decide something about the family:
the reader that accepts a body,
and the dispatch that chooses that reader for a file.
Those two lists drifting apart is not a refusal but a WRONG ANSWER.
The final throw is re-commented as reachable rather than as unreachable.

GFP-proven:
restoring the drifted form,
naming the first generation alone,
fails the new case,
which reads the same list and requires the two-lane reader to have ANSWERED
for every generation in it.

### The straggler class

Four names the bare-word rename could not reach,
because each carries a prefix or suffix:
`_chunkIndex`,
`earlierChunkIndex`,
`laterChunkIndex` and `byChunkIndex`.
None reaches the wire,
and no artifact on disk carries any of them.
A rename measured by counting a bare word will always leave this class behind;
the sweep that finds it is a case-insensitive search for the token as a SUBSTRING.
