# Translation repair history 2026-08-21 to 2026-08-23: the document publisher that writes fixed pages into a new directory tree, 2026-08-22

The publisher that writes fixed pages into a new directory tree,
why it runs inside the pass,
why its write order is the correctness property,
what was measured across the settled pools,
the evidence from breaking it on purpose,
and the check at the user boundary.

Part of [Translation repair history: 2026-08-21 to 2026-08-23](translation-repair-history-2026-08-21-to-23.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The document publisher: the pipeline now writes fixed pages into a new directory tree

### `#175`: the owner answered, and the pipeline now writes documents

The owner's words:
"The pipeline should replicate the directory structure of the corpus in a newly created dir,
and put fixed `*.en.md` files in the new dir."

Until today every decision the pipeline made existed only as JSON.
The contest picked a lane,
the consolidation picked a text,
the gate accepted or refused,
and the only full documents that existed at any moment were the ones the checkers assembled to verify and threw away.

`publishFixedPage` in `src/corpus-run/publish-fixed.ts` closes that,
writing `<runsDir>/fixed/people/<id>/page.en.md` for every entry the pass settles.

### Why the publisher runs inside the pass, and not over an artifact pool

This was the open fork when the owner's answer arrived,
and the artifact schema settles it.
`spliceSlices` resolves each replacement by `slice.target.chunkIndex` and needs the OFFSETS a `ChunkPair` carries.
`ArtifactComparisonRowV2` carries `chunkIndex`,
`incumbentKind`,
`incumbentText` and the lane texts,
and no ranges at all.
Recovering offsets by searching `incumbentText` inside `archiveText` is not merely fragile on repeated wording:
where `incumbentKind` is `absent` the incumbent is the empty string,
which matches at every offset,
so only a stored range can say where an insertion goes.

Extending the schema with ranges was rejected as a standalone-republisher feature nobody asked for.
NAMED EXCLUSION:
artifact pools settled before this publisher existed get no pages without a re-run.

### The write order is the correctness property

`corpus-pass.ts` builds its skip set as `settledEntryIds({ artifactsDir })`,
so "done" means exactly "an artifact JSON exists".
Publishing after the artifact would let a crash between the two writes
strand an entry marked done forever with no page ever written.
Publishing first makes "done implies published" true by construction.

The file already argued for this ordering in its own words:
`settledTallyLine` was moved above the artifact write earlier the same day
because it can raise and would otherwise print `status=ERROR` for an entry whose complete artifact was on disk.
The publisher reads the same would-ship readings and can raise the same error,
so it sits in the same slot.

### What was measured, and the branch measurement cannot reach

Across 14 artifact pools,
40 entries and 249 slices,
every settled artifact on disk:

-   readings shipping nothing:
    0
-   slices whose archive holds no wording:
    0
-   slices whose archive holds wording:
    249

The same reader discriminates four deciders over those slices,
`archive` 228,
`consolidation` 11,
`contest` 9,
`lanes-agreed` 1,
so it is reading real per-slice data rather than returning a constant.

All three silence kinds require an empty archive incumbent,
and no slice in any pool has one,
which is consistent with `#90` and `#100`:
absent incumbents come from one-sided sections the slicer does not yet produce.
So `spliceSlices`' refusal of empty text at an insertion anchor cannot fire today,
and becomes reachable only when section-scale one-sided slicing lands.
It is left to propagate and fail the entry loudly.

WHAT THE MEASUREMENT COULD NOT SHOW is that a silent reading is reachable at all.
Forcing one by blanking a real artifact's row was refused by `parseSettledArtifactV2`,
which cross-derives comparison rows from the ledgers beside them,
so the branch is proved by a fixture artifact in the tests instead.

A SAFETY NOTE from that refusal:
the parse error message quotes the passage it disagrees about.
Scratch probes over artifacts must print error CLASS NAMES only,
never messages.

### The GFP evidence, and the file split it produced

Three type-legal breaks:

-   `order`,
    publishing after the artifact write:
    fails only `settleEntry`'s ordering case,
    with `ENOENT` on the page.
-   `mkdir`,
    creating only the immediate parent:
    fails every case that writes a page,
    in both files.
-   `silent`,
    dropping silent readings instead of handing them the empty string:
    fails the replacement builder and both refusal cases.

The `silent` break is what found a structural defect in the new tests.
It first reported ONE narrow failure while the real blast radius was every page the pass writes:
the runner abandons a whole file once any describe in it fails,
and the two pure describes sitting above the writing ones hid all of them.
Split into `publish-fixed-replacements.unit.test.ts` and `publish-fixed.unit.test.ts`,
the same break reports both files.

This is the second time this runner behaviour has cost a segment's evidence.
The rule it implies:
cheap unit describes and end-to-end describes do not share a file.

### Verified at the user boundary, and one claim of mine corrected

A real `corpus-pass --only gaoyanger` into a throwaway `TRANSLATION_REPAIR_RUNS_DIR`,
settling the entry in 976 seconds and publishing `fixed/people/gaoyanger/page.en.md`.

The check is an INDEPENDENT reconstruction rather than a second call to the assembler.
Re-splicing would compare the publisher against itself and agree with any offset bug it has,
so the expected page is rebuilt from the corpus bytes by string replacement,
which knows nothing about ranges:
start from `git show <pin>:people/gaoyanger/page.en.md`,
and swap each incumbent wording for what the artifact says shipped there.

-   archive 349 characters,
    published 370
-   2 slices:
    1 shipped new wording,
    1 kept the archive wording,
    0 shipped nothing
-   page matches the independent reconstruction:
    true
-   trailing newline preserved exactly,
    on both sides

CORRECTION TO WHAT THIS RECORD FIRST SAID.
It claimed re-runs overwrite the page.
A second pass into the same runs directory reports `pending=0 done=1`,
skips the entry outright,
and leaves the page with an unchanged mtime and an identical digest.
So a resumed pass does not republish at all,
which is what the "done implies published" ordering is FOR:
the page is already there,
and nothing has to re-derive it.
Overwriting is only the path where an entry is genuinely re-settled,
an artifact gone while its page remains,
and that path is covered by a unit case rather than by this run.

The run also carried the first real traffic through `#162`,
`#177` and `#178`,
which dropping `#162`'s fork run had left unexercised.
It settled with `selection=contested`,
`pageChanged=1`,
`pageSilent=0` and no refusal.

### Commits

-   `f982ca3cc`,
    `f6ebbce0b`,
    `3e711d9bf`:
    `#162`,
    the wrap and its tests.
-   `6acb9a86c`:
    the publisher and its wiring.
-   `db52bc892`:
    the publisher's tests.
-   `717bb781c`:
    the file split GFP forced.
