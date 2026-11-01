# Translation repair history 2026-08-12 to 2026-08-15: poisoned cache records, the assembly contract and guards shown to fail

The sabotage test showing that a poisoned cache record reaches the lane,
the refuted claim that no checker runs after refinement,
the net-zero assembly canonicalization,
a contradictory cached slice costing one slice instead of the entry,
the lint debt found,
and the review of which guards were shown to fail without themselves.

Part of [Translation repair history: 2026-08-12 to 2026-08-15](translation-repair-history-2026-08-12-to-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Session 2026-08-15: a poisoned cache record proved reachable, and #97 refuted

### The sabotage test #95 was missing

The unit tests proved `assertReplacementsChange` throws.
Nothing proved the LANE routes a poisoned cache record into it,
and the cache is
the only reachable way in:
a resumed record is trusted on its slice index alone.

`translate-document.unit.test.ts` now poisons every persisted record to claim its
change while carrying the wording it claims to have replaced,
resumes it,
and
expects the refusal.
Two things make it a proof rather than a hope.
It carries a positive control on the poisoning itself,
so it cannot pass because
nothing was sabotaged.
And it was shown to fail:
removing the guard call from `translate-document.ts`,
rebuilding,
and running produced
`Expected promise to reject, but it resolved` at
`translate-document.unit.test.ts:522`.
The poisoned document settles silently without the guard,
which is exactly the
defect.
Guard restored,
suite green at `test_exit=0`.

### `#97` is refuted: a checker DOES run after refinement

The claim was that `resolved` may describe pre-refinement text because no
checker runs after the naturalness lane.
One does.
`refine-phase.ts:199` calls `retainsResolvedIssues` for every refinement that
changed anything,
and that runs a full checker stage over the REFINED text at
`refine-phase.ts:360`,
over exactly the issues `resolvedIssueIds` named.
Any issue it does not re-confirm rolls back the WHOLE slice at
`refine-phase.ts:211`,
so `refined: true` at `refine-phase.ts:272` is stamped
only past that gate.

Measured over the 56 settled artifacts with
`~/temp/agent/refined-resolved-census.mjs`:

```text
issue records                    4098
resolved records                 2586
refined records                   323
resolved AND refined              181
distinct resolved+refined slices   32
refine-recheck-passed findings     32
prediction violations               0
```

The prediction was that resolved-and-refined slices can never outnumber the
re-check findings covering them.
It holds with EXACT equality,
which additionally says no refined slice with
confirmed issues was withdrawn at assembly in this corpus.
The probe was validated first:
stripping one entry's seven findings on a
throwaway copy made it report `bothSlices: 7, rechecks: 0`.

### The gap the weakened assertion opened, and the fix for it

Dropping the second direction of `assertDocumentChangeAgrees` was right,
and it
opened a hole worth naming.
A run whose replacements cancel at a join now returns `shippedChunkIndices`
non-empty beside a byte-identical document,
while both TSDoc blocks say those
indices name slices the document CARRIES a change for.

Both reviewers independently reached the same fix and the same location:
canonicalize inside `guardFootnoteAssembly`.
When the surviving replacements assemble to the incumbent exactly,
return no
survivors,
move them all into the withdrawn set under a reason of their own,
and
leave their accepted wording in `sliceTexts` where it still belongs.
The sol review adds what neither the advisor nor I had:
after canonicalization,
`(assembledText !== targetText) === (replacements.length > 0)` becomes a true
guard postcondition,
so the SECOND ASSERTION DIRECTION CAN BE RESTORED,
which is
strictly better than where this started.
Its other four findings are recorded in `#103`.

### The canonicalization landed, and it bought back the assertion

`guardFootnoteAssembly` now withdraws every surviving replacement whose assembly
reassembles to the archive text,
under `assembly-net-zero-canonicalized`,
and
returns no survivors.
Nobody did anything wrong in that case,
so it is canonicalization rather than a
refusal:
each lane still holds every wording it decided in `sliceTexts`,
and only
the document-level claim changes,
to the true one.

THE NET-ZERO IS REACHABLE,
which was worth establishing before building for it.
Adjacent paragraph slices separated by exactly a blank line cannot produce one:
solving `u + "\n\n" + v == a + "\n\n" + b` needs a second `"\n\n"` inside a
slice,
and one paragraph per slice has none.
But subdivision GROUPS small paragraphs,
measured on a fixture of 30 short
paragraphs:
three slices,
gaps of exactly `"\n\n"`,
and 11,
10 and 6 internal
blank lines.
Moving the first paragraph of the later slice into the earlier one then changes
both slices and no byte of the document.
The guard test builds exactly that and was shown to fail without the
canonicalization:
`expected [ { chunkIndex: +0 }, ... ] to deeply equal []`,
with the assembled text already equal to the archive.

Because the guard now guarantees it,
`(assembledText !== targetText) === (shipped.length > 0)` is a postcondition,
so
`assertDocumentChangeAgrees` CHECKS BOTH DIRECTIONS AGAIN.
That is strictly better than where this started,
and it came from the sol review
rather than from either of my own readings.

### A contradictory cached slice now costs one slice, not the entry

`resumed-slice.ts` is new.
Both lanes check `changed === (decidedText !== incumbentText)` where the record
is ACCEPTED,
discard a record that disagrees,
and buy that slice again,
naming
each discard in the findings so a recomputed slice is distinguishable from one
that was never cached.
Both directions are checked.
The quieter one is a record DENYING a change it made:
only `changed` records
become replacements,
so its wording was previously dropped at assembly with
nothing said about it.

MEASURED FIRST,
over the two surviving repair slice caches re-prepared from the
pinned corpus at zero quota (`~/temp/agent/changed-invariant-census.mjs`):

```text
cached repair outcomes             150
written for an EARLIER slicing      29
attributable to this preparation   121
CLAIMS A CHANGE IT DID NOT MAKE      0
```

The 29 are not a contract violation.
`repairChunk` returns `selection.winner.text` and the unchanged candidate's text
IS the slice incumbent (`repair-chunk.ts:361`),
so `changed: false` beside
differing text means the file was written for an earlier slicing,
which the cache
key correctly makes miss.
The limit on the zero is recorded in `#95`:
a stale file whose `changed` is true
cannot be told from a current one,
and staleness can only hide a positive.

### Two extractions, both forced by `max-lines` and both worth doing anyway

`repair-slice-key.ts` holds the repair cache key,
its run shape,
and the version
constant with the longest comment in the package.
It mirrors `translateRunShape` and `translateSliceKey`,
and the key is testable
for the first time.
`repair-blocked-exit.ts` holds the dominance-blocked result:
the one exit that
never reaches assembly,
and so the one that states by hand every fact assembly
would otherwise have derived.

### The lint debt this session found

The package was carrying 4 lint errors and 13 warnings plus one type error from
the previous session's commits,
none of which the previous verification caught.
All are fixed,
and one was a design rule rather than a style nit:
`acceptedText: string | null` violated this repo's absence rule,
and is now an
optional property.
Verification now runs all three of `buildAndTest`,
`lint` and `lint:types` and
reads their exit codes,
rather than one of them.

### Session close, 2026-08-15: what landed after the invariants work

Everything below is committed and pushed on `translation-repair-rebased`.
Final verification:
`test_exit=0` with 272 PASS lines and zero FAIL,
`lint_exit=0` at "Found 0 warnings and 0 errors",
`types_exit=0`.
All three were run,
and all three exit codes read,
on every commit after the
first:
the previous session's "all green" was stale,
and the package was
carrying 4 lint errors,
13 warnings and a type error nobody had seen.

WHAT THE ASSEMBLY CONTRACT LOOKS LIKE NOW,
since it changed in four steps and
the intermediate states are not worth reconstructing:

-   A resumed record is checked where it is ACCEPTED,
    both directions,
    and
    discarded if it contradicts its own text.
    One bad cache file costs one
    slice.
-   A fresh record cannot contradict itself on either lane:
    translate always
    derived `changed` from its own text,
    and repair now does too.
-   An assembly that changes no byte ships nothing,
    whatever its slices decided.
-   The shipped set is DERIVED from the surviving replacements,
    and the returned
    document is re-spliced from those same replacements and compared.
    The two
    can no longer disagree.
-   The comparison validates each lane's shipped set against that lane's own
    rows before joining anything.

WHAT `withdrawn` MEANS NOW,
worth repeating because a count reader will get it
wrong otherwise:
three causes rather than one,
and only the findings say which.
Reading `withdrawnSliceCount` as footnote damage over-counts.

ALSO LANDED,
unrelated to the invariants:

-   `#93`'s guard for every role except critics.
    A lane configured with nobody
    in a required role refuses before buying anything,
    at all three depths a
    caller can enter at.
    Critics stay unguarded because Question 3 may make an
    empty critic roster the intended configuration.
-   `#97` refuted,
    with the measurement.
-   The repair cache key is pinned by a golden hash.
    It had no other witness:
    persist and resume both call one function,
    so a change to the derivation
    fails nothing and only shows up as quota.

WHAT IS QUEUED AND WHY IT WAS NOT STARTED.
`#99` and `#100` force a translate
cache version bump and rewire slice identity across five files,
and a half-done
state there breaks resumability rather than merely being unfinished.
That is the
one item in the queue where stopping midway costs something,
so it wants a run
of hours rather than the tail of one.
`#103` items 6 and 7 belong with it,
and
so does `#94`'s rename.

### Every guard with a reachable failure was shown to fail without itself

READ THE HEADING LITERALLY.
It is not a claim that every guard added on
2026-08-15 turns a test red when removed:
three do not,
and each is named as
such where it belongs.
The claim is that no guard capable of failing was left
untested,
and that the ones incapable of it were shown to fail NOTHING rather
than assumed to be covered.
That is the finding,
not a proof.

Two of the day's guards landed without a removal proof,
which is how a test that
asserts nothing gets mistaken for a test that passes.
Both were proven afterwards,
by removing the guard,
rebuilding,
running the
suite,
and restoring from `git checkout`.
The guards were already committed,
so restoring could not lose work.

Removing `resumedSliceAgrees` from `translateDocument` fails the poisoned-cache
test,
and the failure arrives from one layer down as
`AssemblyContractError: slice 0 claims a change and carries the archive wording`.
That is the whole argument for the discard in one line:
without it,
a single bad
cache file aborts the document after every other slice has been paid for.

Removing `assertRostersConfigured` from `translateDocument` fails the
empty-roster test.
Without the check the throwing client's failures become lost voices,
the run
settles,
and nothing raises,
which is the exact silence `#93` exists to refuse.

ONE GUARD FROM THIS SESSION IS PINNED BY NOTHING,
on purpose,
and this is the
record so the next reader does not mistake it for coverage.
`winnerChangedText`'s wiring in `repair-chunk.ts` has no test:
there is no repair-chunk harness,
so reverting `changed` to the selection fact
turns no test red.
The only thing that catches that regression is `assertReplacementsChange` at
assembly,
which converts it from a red suite into a rare abort on a legitimate
run.
Building the harness is the fix,
and it is recorded in `#103`.
