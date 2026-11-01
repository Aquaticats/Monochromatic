# Translation repair history 2026-08-24 to 2026-08-25: naming the two-lane artifact family for its shape, and rehearsing the landing of the parked work on a throwaway worktree

The naming of the two-lane artifact family for its shape rather than a version,
with the two measurements that changed the answer,
the renames that followed and the landing note,
and the landing rehearsed on a throwaway worktree,
with the order the landing procedure must follow.
The rehearsal is filed here because it tests the landing of the renames this file decides.

Part of [Translation repair history: 2026-08-24 to 2026-08-25](translation-repair-history-2026-08-24-to-25.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## `#205`: the two-lane artifact family is named for its shape, not a version

The owner delegated the naming ("you decide,
this isn't a design decision")
and chose to leave the version-1 family where it is.
Measuring first changed the answer twice,
so both measurements are recorded.

### The task's own candidate was refuted

`#205` proposed `artifact-lanes-*` and `SettledArtifactLanes`,
noting neither had been checked.
Checking them killed both:
`ArtifactLaneRelationV2` and `ArtifactLaneSelectionV2` already use `Lane`
for PER-LANE concepts,
so a family-wide `Lanes` would name the whole thing
with the word its own parts already use for one part.

### The plain names belong to the older shape

Dropping the suffix outright collides on exactly six names,
and all six live in `artifact-v1-read.ts`:
`parseSettledArtifact`,
`ParsedArtifact`,
`buildSettledArtifact`,
`judgeSlice`,
`compareDecisions`,
`collectShippedRegions`.

That is worse than `#205` recorded.
The plainest names point at the OLDEST shape,
and the shape the pipeline actually writes wears a version number
that has been wrong since generation 3.
The v1 arm is still reachable,
so this is a naming problem rather than dead code:
`artifact-read.ts` routes unversioned and version-1 artifacts to it.
The owner chose to leave it,
and it is filed rather than fixed here.

### The rule, which the measurement chose rather than taste

A marker belongs exactly where two shapes are distinguished.

-   Six symbols have a version-1 counterpart,
    so those six say `TwoLane`:
    `parseSettledTwoLaneArtifact`,
    `ParsedTwoLaneArtifact`,
    `buildSettledTwoLaneArtifact`,
    `judgeTwoLaneSlice`,
    `compareTwoLaneDecisions`,
    `collectTwoLaneShippedRegions`.
-   Fifty-six have no counterpart at all,
    so their suffix asserted a version they do not carry and they simply lose it.
-   `ARTIFACT_SCHEMA_VERSION_V2` is untouched.
    It denotes the integer 2,
    and a version constant should carry a version number.
-   Forty files move from `artifact-v2-*` to `artifact-two-lane-*`.

### The sweep missed three names, and only the built artifact showed it

The first pass matched `[A-Za-z0-9_]*V2\b`,
which requires a word boundary after the digit.
`DamageRegionV2Error`,
`ArtifactComparisonV2Error` and
`verifyArtifactV2AgainstPreparation` carry `V2` in the MIDDLE,
so the scan never saw them,
the rewrite never touched them,
and a residue check built on the same assumption reported the work complete.

Reading `dist/final/node/index.d.mts` is what found them.
That is the rule this pays for:
a rename is checked at the artifact,
never only in the source it was applied to.

A method note on the check itself.
The first probe of the built types returned zero for every name including ones
that certainly exist,
because it read `dist/final/types/index.d.mts`,
which is not where the declarations land.
A positive control on a name known to be present is what caught it,
before the zero could be read as "the rename dropped everything".

The two error classes carry their own name as a string as well,
and both halves moved together
so a `name` assertion cannot pass against a class that no longer answers to it.

### Two test labels were lying

`artifact-change-sets.unit.test.ts` wrapped two assertions in
`caught(function parseTwoLaneArtifact() {...})` and
`caught(function readTwoLaneArtifact() {...})`,
and both bodies call the SINGLE-lane side.
The labels were chosen to dodge self-shadowing,
since a named function expression binds its own name inside its body,
and the dodge picked a name that says the opposite of what the code does.
They now name the assertion:
`singleLaneParseOfVersionTwo` and `changeSetReadOfVersionTwo`.

### Verification

```text
build          clean
lint           0 warnings, 0 errors
suite          676 PASS, 0 FAIL, exit 0
```

676 is the same count as before the rename,
which is what a pure rename must produce.
The shipped `index.d.mts` carries every renamed export
and `ARTIFACT_SCHEMA_VERSION_V2` as the only surviving `V2`,
and the six version-1 names are still present and untouched.

### Landing note

The 40 file renames mean the parked tarball is no longer sufficient on its own:
extracting new paths would leave the old ones in place.
`~/temp/agent/parked-deletions-20260825.txt` lists the 40 paths to delete,
and `~/temp/agent/parked-status-20260825.txt` holds the full status this park was cut from.

## The landing was rehearsed on a throwaway, and it works

The parked work had never been tested as a LANDING,
only as a working tree.
That gap mattered more after `#205`,
because 40 file renames mean the tarball
alone is no longer sufficient:
extracting new paths leaves the old ones in place.

A worktree was cut from the current main HEAD,
`9569f9d79`,
dependencies installed off the shared store,
and the park applied exactly the way a real landing would apply it:

```text
tar --extract   124 files
delete          40 superseded paths from parked-deletions-20260825.txt
status          67 new, 40 deleted, 57 modified
artifact-v2-*   0 files remain
```

The counts reconcile with the fork's own status,
where the same change reads as 27 new plus 40 renames plus 57 modified:
a rename lands as one new file and one deletion.

Then the whole gate,
on that fresh tree:

```text
build       clean
lint        0 warnings, 0 errors
suite       676 PASS, 0 FAIL, exit 0
```

And the new CLI through its own task,
which is the user boundary rather than a
node invocation of a bundle:

```text
mise run //package/module/translation-repair:run-timing-report -- <log>
  rounds                 2, 1.72min in total
    waiting after quorum 30.01s, 29.0% of round time
    voices never heard   1
  calls in flight        mean 1.05, peak 2
    busy against span    21.00s of calls across 20.00s of run
```

Moved on 2026-10-18 from `//package/module/translation-repair:run-timing-report`
to `//package/translation-repair/run:run-timing-report`.

The worktree was removed afterwards.
What this buys is that the landing,
when the calibration exits,
is a rehearsed
procedure rather than a first attempt on the main worktree.

### The landing procedure, in the order it must happen

1.  Confirm the calibration has exited,
    and collect its standing first.
2.  In the main worktree,
    extract `~/temp/agent/parked-combined-20260825.tar.gz`.
3.  Delete every path in `~/temp/agent/parked-deletions-20260825.txt`.
4.  Build,
    lint,
    and run the suite.
    Expect 676 PASS and 0 FAIL.
5.  Commit with scoped pathspecs,
    naming every new file (CPN),
    and remember the 40 deletions are part of the same change.
