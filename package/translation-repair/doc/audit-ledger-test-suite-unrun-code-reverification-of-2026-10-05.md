# Audit ledger: code no unit test runs, re-verification of 2026-10-05

The correction of 2026-10-05 of eight groups the trial model had blamed on the census,
the verified facts about the census tools,
the method adopted from then on,
and the re-verification of the sites left open.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## Correction of eight groups blamed on the census's attribution

NOTE (2026-10-25,
pointer by position named):
the CORRECTION paragraph of this section read
"the eight paragraphs above that explain",
which pointed at the paragraphs that stood before it
when the ledger was one text,
ahead of its re-filing into files.
They are the spend cluster (thirty-ninth) in
`audit-ledger-test-suite-unrun-code-trial-clusters-markup-through-editor.md`
and the slice,
model,
lookup,
lane,
nudged,
resolution
and canadian clusters
(forty-third,
fifty-sixth,
fifty-ninth,
sixty-second,
sixty-fourth,
sixty-sixth
and sixty-eighth)
in
`audit-ledger-test-suite-unrun-code-trial-clusters-roster-through-publish.md`.
The claim of the CORRECTION paragraph is unchanged;
only the pointer now names its targets.

CORRECTION (2026-10-05),
self-named:
the eight paragraphs of the trial-cluster files that explain cold lines as
"the census's attribution"
(the spend,
slice-cache-namespace,
model-content,
lookup-cache,
lane-contest-cache-store,
nudged-reask,
resolution-wire,
and canadian groups)
were wrong.
Re-verified against each run's own census byte spans
and V8's own ranges on the coverage build,
every one of the eight traced to my own fixtures or assertions:

- `resolution-wire.ts` 328 was covered all along:
  V8 reports the return's bytes with count 1,
  and the run's cold span `[11812..11892]` begins after them.
  The stretch's line label starts at the boundary line,
  so the words "still cold 328" named a span holding
  lines 329 to 331.

- `lane-contest-cache-store.ts` 59 to 61:
  the guard reads the archive field of each BALLOT
  (line 84 inside `isLaneContestBallot`),
  and my case put `archive` on the outcome.

- `nudged-reask.ts` 147,
  153,
  159:
  my first reply carried no `servedBy`,
  so the function answered from the untagged reply
  before the re-ask ever ran;
  and both assertions read `kind === 'ok'`,
  which every path returns.

- `canadian-date-read-leading.ts` 259:
  the refused year arises where four or more digits
  fail the clean-year read
  (the example shape is `May 3 20233`);
  my "no year" text took the none path.

- `canadian-date-read-leading.ts` 335:
  the runs-on comma belongs to `yearMonthDay`
  (the year-first form,
  such as `2026 May 3rd`)
  and needs an ordinal day;
  my texts went through the month-first and leading readers,
  whose own `closingComma` produced the commas
  my expected strings looked for.

- `canadian-spelling-capital.ts` 463:
  the counter runs inside `insideEmphasis`,
  which the respeller calls only for a capitalized candidate word;
  my lowercase word never reached it.

- `spend-read.ts` 238:
  the negative-count return is the same literal as the returns
  beside it,
  and the assertion could not tell them apart;
  the file's own known-good tail now proves the readable path
  beside the refusals.

- `spend-read.ts` 345's `?? ''` is dead code
  (`String.split` always yields at least one piece)
  and became `nonNullishOrThrow`.
  The remaining sites in these groups
  (`lookup-cache.ts` 193 and 221,
  `slice-cache-namespace.ts` 266 and 378,
  `model-content.ts` 36)
  were covered by their cases as written:
  the runs' own byte spans exclude those statements,
  and the residual spans are neighbouring statements
  other cases cover.

## Verified facts about the census tools

VERIFIED TOOL FACTS
(kept for later readers of these censuses):
the suite prints one info line per suite
listing its fulfilled children's names,
per-case `[PASS]` lines are debug level,
and empty-name suites log at debug
(`package/module/test/src/describe.ts`),
so counting `[PASS]` lines counts named suites,
not cases;
a cold stretch's line label can start at the line
before the cold bytes
(the piece begins at the boundary after an executed statement),
and the run's own census JSON carries the byte spans
(`bundle`,
`start`,
`end`)
whose slice of the coverage chunk's text names the exact statement;
the verdict line's words are derived from baseline stretch boundaries
matched against merged spans and can misattribute;
the coverage build is unminified for exactly this reason
(`package/module/translation-repair/mise.toml` (moved on 2026-10-18 to `package/translation-repair/*/mise.toml`),
`coverage-census:build` says compression folds guards into expressions
V8 gives no range for,
ledger M79);
and `isJsonRecord` is object and non-null and not-array
(`src/json-guard.ts` (moved on 2026-10-18 to `package/translation-repair/document-structure/src/json-guard.ts`)).

## Method adopted after the correction

METHOD FROM HERE:
a case counts as covering a branch only where its assertion
distinguishes that branch from the other paths returning the same value,
and the run's own census bytes exclude the target statement
from the cold set.
A passing suite is not branch evidence.

## Re-verification of the sites left open, by byte span

RE-VERIFICATION COMPLETE (2026-10-05):
the fixture-reasoned left sites are re-verified
by byte span too,
and all but one now carry cases
whose assertions distinguish their branches:

- `photo-reference.ts` 133 and 303 covered:
  the reduce's earlier-mark arm needs one element
  carrying two quote kinds,
  and the asset-directory refusal needs a path
  starting with the entry placeholder
  but not the photos directory
  (my earlier path failed the placeholder check first
  and returned the same value there).

- `dropped-destinations.ts` 187 and 292 to 295 covered:
  the reduce's second arm needs one text carrying
  both scheme kinds
  (mine carried one scheme twice),
  and the empty return is the tree-side shape
  where the destination is all punctuation
  (a scan run always starts with a scheme character).
  Its run-advance fallback (241)
  was provably dead
  (the end scan always consumes the scheme)
  and became the plain advance.

- `active-footnote-markers.ts` 309 to 335 covered:
  the four bounds arms split between a run at its
  parent's ends
  and a run between positioned neighbours.
  The autolink transform rebuilds the surrounding text
  unpositioned too,
  so the far side needs an inline node
  that keeps its positions.

- `claim-panel-voters.ts` 54
  and `claim-filers.ts` 253 and 262 covered:
  the empty-ballots return needed an issue
  whose readings carry the claim,
  and the filers fallback was dead behind its own filter.

- `publish-fixed.ts` 320 and 321 were dead
  behind the shared dropped list
  and the trace type's required fields;
  they became `nonNullishOrThrow`.

- `model-card-derive.ts` 366 covered
  through `recordOver`'s own keys;
  421 to 427 remains UNRESOLVED
  with the cause known:
  both its inputs are the module's own card
  and roster tables,
  so the disagreement it guards is data drift
  no test fixture can stage.

Two of the re-verification edits put TSDoc blocks
before statements;
the attachment scan caught both,
and they are comments now
(fix-forward,
never amend).

Suite at the close:
1,564 [PASS] and no [FAIL];
35 source scans and no [FAIL].

## Re-verification of the published-page sites

PUBLISHED-PAGE RE-VERIFICATION (2026-10-05):
the incumbent-map fallbacks
(`published-page-check.ts` 179,
196,
236)
are dead:
`wouldShipTextPerSlice` walks the same
`artifact.comparison` the map is built from
(an earlier fixture cut the comparison
and the results proved identical),
and they became `nonNullishOrThrow`.
The filled-anchor caveat
(`published-page-disagreement.ts` 99 to 100)
now has its case,
its message pair distinguishing the floor
from the outright comparison.
The unweighable guard (552)
cannot fire after the refutes check
(`pageWeightRefutes` returns false for unweighable)
but narrows the types,
so it became an `unreachable:` throw,
its line cold by design
(ledger M112's shape).
One position word left a comment
and the position-reference scan caught it (WR5),
fixed forward.

## Re-verification of the declined-target sites

DECLINED RE-VERIFICATION (2026-10-05):
the flatMap's node read
(`declined-target-runs.ts` 193 to 196)
cannot miss
(the `isDeclined` filter keeps only target-only steps
and the alignment indexes the same node list),
so the kind ternary and the undefined arm
became one `nonNullishOrThrow`
under a type-guard filter.
The empty-pairing refusal (158)
now has its case
(an empty source side yields no paired step
and no source-only step to stop earlier).
The four spans first left open
with the cause "the aligner's `continuesPairing`
emission rules unmapped"
were then covered anyway,
and that cause was itself unverified:
`declinedTargetBlocks` takes its steps as an argument,
so hand-built steps suffice.
Their fixtures need a `paired` step
(the no-pairing guard returns early)
and the plain and continuation steps
must name different indices
(the claimed set would exclude the plain one otherwise).
All four spans are covered:
the rendering-continues machinery
and the claiming walk's source-only arm.
Two scan slips in the edit
(a TSDoc before a `return`,
a position word in its comment)
were caught and fixed forward.
