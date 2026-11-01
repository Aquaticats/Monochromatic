# Translation repair history 2026-08-23 to 2026-08-24: branch tests for the modules the suite reaches only through a caller

The ranking of the indirectly reached modules by branch density,
what each landing proved by mutation,
the runner behaviour that limits reading a mutation result,
the two barrels split on the way,
and the final landing on the join rule that hides a seeded deletion.

Part of [Translation repair history: 2026-08-23 to 2026-08-24](translation-repair-history-2026-08-23-to-24.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The 53 indirectly-reached modules, branch by branch (`#209`, 2026-08-24)

`#208` closed the tier no test reached at all.
This is the next one:
the 53 modules the suite reaches only through a caller.
`TC2`'s question there is not whether the module is reached
but whether each implementation branch has a test,
or only the happy path does.

The shape of the gap is the same every time.
`parseSettledArtifactV2` calls `assertIndexSetsMatchLedger` on every valid artifact fixture in the suite,
thousands of times,
always down the arm where nothing was wrong.
That is coverage of the caller.
The refusal branches are what the module exists for,
and no valid fixture can reach one.

Ranked by branch density,
measured with a brace-and-keyword count over each module.
The measurement lives in the session scratchpad;
the ranking is reproduced here because it is what the remaining work is ordered by:

```text
branches  code lines  exports  module
      31         239        1  align-headings-optimal.ts              DONE
      20         164        3  corpus-run/artifact-v2-read-set-relations.ts  DONE
      16         138        2  preservation-tokens.ts                 DONE
      14         131        1  lane-slice-sets.ts                     DONE
      12         127        2  translate-skeleton.ts                  DONE
      12         121        2  corpus-run/artifact-placement.ts       DONE
      12         118        5  corpus-run/artifact-v2-project.ts      DONE
      11         237        3  corpus-run/artifact-v2-read-consolidate-parts.ts  DONE
       9          66        1  stream-recurrence-watch.ts             DONE
       9         250        1  refine-slice-settle.ts                 DONE
       8          91        1  translate-retry.ts                     PROBED, already defended
       8         122        1  fidelity-splice.ts                     DONE
```

### What each landing proved, by mutation

Every module gets a mutation applied to the SOURCE,
rebuilt,
run,
and restored.
What matters is not that the suite went red;
it is whether the new cases were the ONLY thing that noticed.

`align-headings-optimal.ts`,
commit `5774615b8`.
Mutation G judged a pairing by its prefix alone,
replacing the backward-table term in `scanOptimalPaths` with a zero score.
16 failures,
including every new aligner case.

`corpus-run/artifact-v2-read-set-relations.ts`,
commit `299e2e6f3`.
Mutation A removed the position-by-position comparison from `assertListMatches`,
leaving a length check.
TWO failures,
both new,
and NOTHING ELSE in the suite noticed:
before this,
a lane recording the right indices in the wrong order was accepted,
and the ordering both contracts state was untested.
Mutation B made the whole-document refusal count as a guard withdrawal,
which two new cases and two existing `parseSettledArtifactV2` cases caught.

`preservation-tokens.ts`,
commit `c367d77a0`.
Mutation C added a colon to `SENTENCE_ENDS`,
the exact regression the module's comment says made a deleted contributor name invisible.
Caught by the new colon case and by one existing `applyPatchOperations` case.
Mutation D stopped dropping one-character Latin words.
ONE failure,
the new case,
and nothing else.

`lane-slice-sets.ts`,
commit `0c078ad36`.
Mutation E ran the archive rules BEFORE the disjointness pass.
Caught by the new ordering case and by one existing `buildLaneSliceTexts` case.
The ordering is contract:
a slice named by two lists disagrees with itself first,
so reporting which archive rule it breaks answers a question neither list has earned the right to ask.
Mutation F dropped the decided-at-once refusal,
caught by two new cases and two existing ones.

`validateNamedSets` and `NamedSliceSet` reach the barrel as `@internal` so the test can exercise the shipped bundle,
which is the standing ruling on internals rather than a new exception.

`translate-skeleton.ts`,
commit `e6d57da2c`.
Mutation G stopped a list saying whether it is ordered,
caught by three new cases.
Mutation H stopped a footnote DEFINITION contributing an atom,
so only the marker survived.
ONE failure,
the new case,
and nothing else:
a translation dropping the definition and keeping the marker passed every structural guard in the suite.
`blockDetail` reaches the barrel as `@internal` so a case can ask what an absent `ordered` field means,
which no Markdown input produces.

`corpus-run/artifact-placement.ts`,
commit `9244e8120`.
Mutation I made an unreadable digest unplaceable rather than legacy,
and mutation J let an artifact carrying no id skip the identity check;
the census already covered both,
which is the honest result and the reason the third was run.
Mutation K accepted uppercase hex as an object id.
ONE failure,
the new case,
and nothing else:
git writes lowercase,
so two spellings of one commit would have counted as two generations.

`corpus-run/artifact-v2-project.ts`,
commit `6aebccf27`.
Mutation L spread the live ledger record instead of rebuilding it through a literal,
caught by the new key-list case and by one existing `buildSettledArtifactV2` case.
Mutation M aliased `undecidedLanes` instead of copying it.
ONE failure,
the new case,
and nothing else:
the artifact outlives the run,
and a reader mutating what it read would have reached back into the builder's own comparison.

`corpus-run/artifact-v2-read-consolidate-parts.ts`,
commit `caf4ca0f2`.
Mutation N let text ship from a slice that settled on no change,
and mutation O read the ballot evidence lists as prose rather than as rendering names;
`parseConsolidationV2` already covered both,
which is the honest result.
Mutation P let an unchanged slice carry text it does not ship.
ONE failure,
the new case,
and nothing else.

`stream-recurrence-watch.ts`,
commit `670a552d6`.
THE STRONGEST RESULT IN THIS TIER,
and the one that came from measuring rather than reading.
The runaway watch's own tests already prove this detector's headline claims,
so a duplicate file would have restated them.
Before writing anything,
two diagnostic mutations asked what the suite actually defends:
setting the consecutive-hit threshold to 1 was caught,
and REMOVING THE TRAILING-BUFFER TRIM LEFT THE WHOLE SUITE GREEN.

That trim is a correctness rule,
not an optimisation.
It decides how far apart two copies of a passage may be before the earlier one stops counting.
A reasoning trace in this pipeline restates whole candidates verbatim,
so one quoted near the start and again near the end is ordinary work,
and without the trim the early copy stays findable forever
and the second quotation reads as a loop that kills a healthy voice.
The new case is exactly that pair of distant quotations,
and re-running the same mutation after it landed produced one failure:
the new case,
and nothing else.

### A runner behaviour worth knowing before reading any of this

A failing `await describe(...)` REJECTS,
and a rejected top-level await ends the module.
Later `describe` blocks in the same file never run.
So a mutation report naming two failures out of five suites is not evidence the other three passed:
they may not have executed.
Read a GFP result as "these cases fired",
never as "only these cases were affected",
unless the failing suite is the last one in its file.

### Two barrels split on the way

Adding `@internal` exports so the tests could reach the shipped bundle
pushed two files past the 300-line budget,
and both were split by audience rather than shortened.
`corpus-barrel.ts` gave up the version 2 READER family to `artifact-read-barrel.ts`,
leaving 252 lines against 52.
`index.ts` gave up the stream-watching family to `stream-barrel.ts`,
leaving 265 against 41.
`index.ts` composes both,
so no importer sees either seam.

### The eleventh landing: the join rule that hides a seeded deletion

`fidelity-splice.ts` holds one export,
`spliceOutSentence`,
and one importer,
`fidelity-damage.ts`,
whose own cases ask whether a seeded sentence disappeared.
Every join rule answers that identically,
including no join rule at all,
so the whitespace decision the module exists for was invisible to the suite.

PROBE W1 confirmed it before a line of test was written.
Removing the line-break precedence from the private `survivingRun`,

```ts
  if (lineBreaksIn({ run: before, },) !== lineBreaksIn({ run: after, },)) {
    return (lineBreaksIn({ run: before, },) > lineBreaksIn({ run: after, },))
      ? before
      : after;
  }
```

left only the two boundary rules and the length tiebreak,
and the whole suite stayed green at 653 verdicts,
exit 0.

WHAT THAT BRANCH DECIDES.
A paragraph cut from the middle of a page
sits between a paragraph break and whatever follows.
Without the rule the longer run wins,
so a two-character `\n\n`
loses to a three-space gap and two paragraphs collapse onto one line
with a triple space at the join.
That is precisely the typographic edit-mark the module note says it exists to prevent,
and it would let a damaged candidate lose a fidelity trial on tidiness
rather than on the coverage the trial means to test.

Sixteen cases landed in `src/fidelity-splice.unit.test.ts`,
one rule each:
both arms of the line-break comparison,
both boundary rules and the order they are asked in
(the only observable case is a cut that reaches both ends at once),
the length tiebreak and its equal-weight fallback,
the ideographic and no-break spaces the Chinese half of the corpus separates with,
first-occurrence-only removal,
and the two runs of length zero.

GFP:
re-applying probe W1 fails exactly the two line-break cases
and the suite aborts at 654 PASS / 3 FAIL,
exit 1.
Restored,
rebuilt,
back to 654 / 0.

### `translate-retry.ts`: probed, already defended, no file written

Probes V1 and V2 were both caught by existing `runTranslateStage` cases.
Recorded here rather than left as a gap:
the module is reached indirectly,
and its branches are covered through its caller.

### Suite size across the eleven landings

636 suite verdicts before `#209`,
654 after the eleventh,
with exit code 0 each time and the FAIL count read off the runner's own `] FAIL ` prefix.

Eight mutations were caught by the NEW CASES ALONE,
which is the part that says a gap existed rather than a rule being restated:
the index ordering comparison,
the one-character-token rule,
the footnote definition atom,
the lowercase-only object id,
the undecided-lanes copy,
the unchanged slice carrying text,
the trailing-buffer trim,
and the line-break precedence in the splice join.
