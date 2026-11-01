# Audit ledger: block pairings that do not fit their text

Pairings of source and target blocks that a later step applied to text they did not fit.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B138: a pairing naming a block its section lacks was grouped as though it named a neighbour

Red in `3a7a72502`,
fixed in `e71249a76`.
Found on 2026-10-05 (UTC) reading the MiMo trial's production diff
(ledger M113 had recorded the empty result as found and not yet decided).
`groupNodesSealed` (`group-aligned.ts`) takes a walk of steps,
each naming a block by its index,
and `walkIntoRuns` read a step's block as `[nodes[index]].filter(isPresent)`:
a step naming a block its side lacks became a step carrying no block there.
`renderedSpan` (`group-source-anchor.ts`) documented the same tolerance.
What followed depended on the walk,
by a delegated agent's probes of its build of `80e129926`:
one paired step naming translation block 3 of 1 returned no runs at all,
since `mergeOneSidedRuns` held the one-sided run for a neighbour and found none to host it;
with a paired neighbour the original joined that neighbour's run instead;
a translation-only step past the last block threw `Expected non-nullish value, got undefined`;
and an original-only step past the last original returned an insertion of no blocks.

A walk is built by `blockPairingToSteps` (`pair-blocks-steps.ts`),
which checked nothing.
`readBlockPairing` refuses such a pair in a model's reply,
but a pairing also arrives from a settled artifact's recipe,
which `parseBlockPairing` checks for shape,
order and section,
never against the blocks of the text the rebuild carves.
On the agent's build,
an artifact recording a pair with translation block 3,
rebuilt over a section of 3 blocks,
came back from `rebuildPreparation` as a moved carve with the pair dropped and the rest regrouped,
and nothing named the pairing.

No settled artifact was misread,
measured by the agent over the 466 artifact files under 477 artifacts directories on this machine:
268 read as version 2,
238 of them record a block pairing,
none of 9,536 recorded pairs names a block the section it is rebuilt over lacks,
and every rebuild answers the same before and after the fix
(249 reproduced,
17 moved).

The fix:
`blockPairingToSteps` refuses a pair that names no block of its side with a `BlockPairingError`,
in `readBlockPairing`'s words,
`pairing names translation block 3, and there are 1`.
Every walk the package builds then names only blocks it is grouped over,
and the readers say so:
`blockAtStep` (`group-source-anchor.ts`) reads the block a step names
and throws an `unreachable:` error naming the step,
the side,
the index and the count where there is none;
`walkIntoRuns` reads every step through it before anything else reads the walk,
so each shape gets the one message,
and `renderedSpan` reads through it too.
The guard M113 restored in `anchorOffsets` stays,
its closing clause reworded,
since a walk pairing past the translation blocks no longer reaches it.
The red run printed three [FAIL] cases in `pair-blocks-steps.unit.test.ts`
and two in `group-aligned.unit.test.ts`,
each because the call returned where the case expected a throw.

Open to the owner's veto:
the walk reader throws `unreachable:` although `groupNodesSealed` is exported and its signature admits any walk,
where B127's reading would give a named `RangeError`;
and on the rebuild path a recipe that does not fit its text now throws where it came back as a moved carve.
`rebuildPage` catches it;
`carveSettled`,
`rendering-audit-settled-input.ts` and `slice-census-entry.ts` do not,
so one such artifact would stop those instruments.
B170 has the rebuild answer `moved` again;
`slice-census-entry.ts` converts the pairings itself and still stops.

Open when this entry was written,
closed in B170,
B171 and B172:
a catch of `BlockPairingError` in `rebuildPreparation` answering `moved` in the refusal's words,
so those instruments go on;
`slice-cache-store.ts` says a cached pairing is re-read through `readBlockPairing`,
and it is not unless media adjacency widens it,
with the same claim leaned on in `pair-media-adjacency.ts` and `pair-definition-order.ts`,
so on that warm path an index outside the lists is refused only by `nonNullishOrThrow`;
`readBlockPairing` and `assertPairsNameBlocks` word one refusal in two files;
`groupNodesSealed` does not check that a supplied walk covers every block,
and a walk of one original-only step over two originals and one translation returns no runs
(the agent's probe of the fixed build;
every walk the package builds covers both sides,
and `prepareDocumentPair`'s coverage assertion catches the rest);
and the comment closing `mergeOneSidedRuns` names a caller's one-sided fallback that no longer exists.

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone".

## B170: a recorded pairing that did not fit its text stopped the rebuild instruments

Red in `5df296c5f`,
fixed in `25c33df4f`.

Found on 2026-10-05 (UTC),
as the item ledger B138 left open.
Since `e71249a76` `blockPairingToSteps` refuses a pair naming a block its section lacks,
and `rebuildPreparation` (`corpus-run/artifact-two-lane-rebuild.ts`) let that `BlockPairingError` out.
`rebuildPage` (`page-republish.ts`) caught it and left the page with the class name alone,
`because=BlockPairingError`;
`carveSettled` and the rendering audit's settled input caught nothing,
so one such artifact would stop the displacement probe and the audit.
On a build of `d233f882f`,
an artifact recording pairs up to translation block 3,
written before the archive was stored,
rebuilt over a corpus copy whose section holds 3 blocks,
threw `BlockPairingError: pairing names translation block 3, and there are 3`
(the fixing agent's probe;
the red case printed the same error on the branch).
The same refusal was worded in two files,
`pair-blocks-wire.ts` and `pair-blocks-steps.ts`.

The fix:
`rebuildPreparation` catches the refusal through `requireBlockPairingRefusal`,
carves the artifact again as one that records no block pairing,
and answers `moved`.
Its detail says that the recorded block pairing does not fit the text carved,
gives the refusal in parentheses (`pairing names translation block 3, and there are 3`),
and says every section's blocks were carved by the deterministic aligner.
A republish then leaves the page and says that sentence;
`carveSettled` returns a carve marked moved,
which the displacement probe warns about and still measures;
the rendering audit answers `refused`,
or `unverifiable` where the section half is unrecorded.
The wording lives once,
in `assertPairsNameBlocks` (`pair-blocks-wire.ts`),
which `readBlockPairing`,
`blockPairingToSteps` and the cache's warm path call.
`parseBlockPairing` gained a comment saying why it bounds no block index:
the record carries no block counts.
The red run printed the `BlockPairingError` for the case
"ANSWERS MOVED IN THE REFUSAL'S WORDS where the recorded block pairing names a translation block the carved text lacks,
and carves every section's blocks by the deterministic aligner instead".

No settled artifact answers differently
(the fixing agent's measurement,
over 466 artifact files under 477 artifacts directories on the audit machine):
268 read as version 2,
249 rebuild as reproduced and 17 as moved on both builds,
and none differs in reproduction,
defaulted halves or carve identity.

Open to the owner's veto:
every section's blocks are carved by the deterministic aligner once one pairing is refused,
since the refusal names no section
and a pairing that misses one section says the text parsed otherwise than the run's.

Open when this entry was written,
closed in B196:
`slice-census-entry.ts` converts a recipe's pairings itself and catches nothing,
so the slice census still stops on such an artifact.

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone".

## B171: a cached block pairing naming a block its section lacks ended the run with a missing-value error

Red in `5df296c5f`,
fixed in `25c33df4f`.

Found on 2026-10-05 (UTC),
as an item ledger B138 left open.
`prepareBlockPairing` (`prepare-block-pairing.ts`) handed a cached record's pairs to `claimMediaAdjacentTargets` as stored.
The cache reader checks a record's shape only (`isPairList`),
and three comments said the record was re-read through `readBlockPairing`,
which happened only where media adjacency widened it.
A record naming original block 9 of 2 reached `pairSharesMedia`,
whose `nonNullishOrThrow` ended the section with `Expected non-nullish value, got undefined`,
naming no section,
key or block.
The key hashes the blocks' text,
so such a record is a damaged or hand-edited file,
which is an input all the same.

The fix:
the warm path reads the cached pairs through `assertPairsNameBlocks`,
the check `readBlockPairing` reads a reply's indices through.
A record that does not fit is a miss:
the warning names the section and the key,
gives the refusal (`pairing names original block 9, and there are 2`)
and says the roster is asked again,
and the section is bought and stored over the same key.
The comments in `corpus-run/slice-cache-store.ts`,
`pair-media-adjacency.ts` and `pair-definition-order.ts` now say that.
The red run printed the missing-value error for the case
"MISSES ON A CACHED PAIRING THAT NAMES A BLOCK ITS SECTION LACKS,
warns in the refusal's words,
and buys the section again".

No cache version moves:
the lane's generation is the digest of the built files,
which this change moves by itself
(the fixing agent's reading of `corpus-run/pipeline-digest.ts`,
with the two builds' digests measured to differ).

Open when this entry was written,
closed in B218:
the warm path checks block indices only,
not the order or the repeats `readBlockPairing` refuses;
and a cached section pairing is handed on unchecked by `buySectionPairing`.

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone".

## B172: a walk that named no step for a block grouped the section without it

Red in `5df296c5f`,
fixed in `25c33df4f`.

Found on 2026-10-05 (UTC),
as an item ledger B138 left open.
`groupNodesSealed` (`group-aligned.ts`) read each step's blocks and never asked whether every block had a step.
A walk of one original-only step over two originals and one translation returned no runs,
and so did an empty walk,
with nothing said.

The fix:
after every step's blocks are read,
`assertWalkNamesSide` refuses a walk that leaves a block of either side unnamed,
in a message opening `unreachable: no walk step names original block 1, and there are 2`
and naming the two builders that give every block a step.
It is an invariant throw:
`alignBlocks` walks its cursor from the lists' lengths to the origin,
`bareBlockPairingSteps` gives every original a step and every translation block one,
and `mergeGap` reuses the indices it was given.
The red run printed `Expected groupsTheWalk to throw, but it returned` for the case
"REFUSES a supplied walk that names no step for a block of either side,
in the words of the refusal for a block the section lacks,
an empty walk among them".
The comment closing `mergeOneSidedRuns` named a caller's one-sided fallback that went in T8's seventeenth batch;
it now says the held blocks leave the runs unsaid and which assertion refuses that.

No carve changed:
268 version 2 artifacts rebuild to the same carve identity on both builds
(the fixing agent's measurement).

Open when this entry was written,
closed in B219,
which refuses such a block by name and reached the state through `prepareDocumentPair` with a seeded generator:
`groupNodesSealed` called alone still drops a translation-only block behind a sealed run with no paired run after it;
six shapes through `prepareDocumentPair` did not reach that state (the fixing agent's probes).

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone".

## B196: the slice census stopped on a recorded pairing its text did not fit

Red in `6f61ba5dd`,
fixed in `72f5be638`.

Found on 2026-10-05 (UTC),
as the item ledger B170 left open.
`censusEntry` (`corpus-run/slice-census-entry.ts`) converted a settled recipe's block pairings itself
and caught nothing,
so an artifact recording a pairing that names a translation block past its section
ended the census with `BlockPairingError: pairing names translation block 5, and there are 4`,
where `rebuildPreparation` answers the same artifact as `moved`.

The fix:
`censusEntry` measures through `measureCarve`,
which catches the refusal through `requireBlockPairingRefusal`,
warns,
and measures every section by the deterministic aligner,
as the rebuild carves the same artifact.
The row reads `carve: 'settled-moved'`,
its `pairingRefusal` holds the sentence the rebuild gives,
and the `CENSUS carve:` line of `corpus-run/slice-census.ts` counts such rows.
The blankness scan holds one more key,
for the empty `pairingRefusal` a row carries when nothing was refused.

The whole suite ran on `72f5be638` in a side checkout:
1,605 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 36 source scans passing.
No cache version moved,
for this entry or for B197 to B202,
on the audit named in B189.

Open to the owner's veto:
every section's pairing is set aside once one is refused,
as the rebuild does;
and the row keeps `carve` as one word beside a separate `pairingRefusal`,
two fields that must agree,
where the other way makes the carve an object carrying its refusal.

Open when this entry was written,
closed in `df807a8b5`,
as B253 notes:
the sentence is worded in two places,
here and in `rebuildPreparation`,
until one shares it with the other.

Recurrence:
`mistake-prevention.md`,
"Refusals inside a composed operation".

## B218: a cached pairing was used without the reader a fresh reply goes through

Red in `63567b4f5`,
fixed in `843bd6ce0`.

Found on 2026-10-05 (UTC),
as the item ledger B171 left open.
`prepareBlockPairing` checked a cached block pairing's indices alone,
and `buySectionPairing` (`prepare-section-round.ts`) handed a cached section pairing on unread.
The fixing agent's probes of damaged records:
a section record naming a translation section the document lacks
stopped the alignment with a missing-value error naming nothing;
one naming an original section it lacks left the document with no slice,
without a word;
a repeated or reversed pair ended in `PlacementLayoutError` far from the cache;
and one original paired with two translations aligned the wrong sections,
without a word.
A block record that moves backwards ended in `SliceCoverageError`,
and one that repeats a correspondence or pairs a definition with a body block was sliced as stored.

The fix:
both warm paths read the record through the whole reader a fresh reply goes through,
`readBlockPairing` with the question's own free order,
so definitions whose labels cross still resume,
and `readSectionPairing`.
A record the reader refuses is a miss said on a warning in the refusal's words,
and the round is bought again and stored under the same key.
Three comments that leaned on the index check now name the reader
(`corpus-run/slice-cache-store.ts`,
`pair-media-adjacency.ts`,
`pair-definition-order.ts`).

Eight cases,
three in `prepare-block-pairing.unit.test.ts` and five in `prepare-section-round.unit.test.ts`,
each hold one kind of damage,
the warning and the round bought again.
All eight failed on the lead's red tree.

A risk the agent names and probed on two shapes:
a record the cold path stores after media widening could fail the stricter reader,
had media claimed a definition into a body pair;
neither shape stored such a record,
and a false miss would cost one round bought twice,
never a wrong page.

Cache:
no version moved;
no record's shape changes,
and a record the reader refuses is replaced.

Recurrence:
`mistake-prevention.md`,
"Files a later run reads back":
the check a fresh answer goes through is its whole reader,
not that reader's first test.

## B219: the grouping dropped a translation block ahead of a sealing note without a word

Fixed in `843bd6ce0`,
with its cases,
which name a class the fix adds;
the fixing agent's red run failed both.

Found on 2026-10-05 (UTC),
as the item ledger B172 left open.
A translation block that stands ahead of the archive's note,
with no paired run before it,
has no run to join:
nothing folds into a sealed run,
and every block after the note is sealed with it.
`mergeOneSidedRuns` (`group-merge.ts`) returned its runs without the block and said nothing.
B172 had six shapes that did not reach the state through `prepareDocumentPair`.
The fixing agent's seeded generator did:
of 3,000 sections reaching the merge,
1,446 of them sealed,
115 lost a block this way,
77 under the scorer's walk and 38 under a supplied one,
and two more seeds gave 182 of 4,462 and 176 of 4,493.
`prepareDocumentPair` refused every such page,
a layer up,
as a block that reached no slice,
so none shipped;
a caller of `groupNodesSealed` alone was handed the runs.

The fix:
where a sealed run stands and translation blocks are still held,
`mergeOneSidedRuns` throws `UnplacedTranslationBlocksError`,
naming the blocks by their positional ids.
The agent's first version threw a plain `Error`.
The lead sent it back:
a log line through `refusalText` would have read `refused by Error`,
where the coverage refusal it pre-empts is marked and prints its sentence.
The class is marked,
its message a count word,
the ids and sentences written in the module,
and the message inventory names the class and its parts.
`mergeOneSidedRuns`,
`groupNodesSealed` and `prepareDocumentPair` each say so in a `@throws`.
After the fix the first seed's 115 sections are refused in the merge and none is dropped,
by the agent's run.

Cases in `group-aligned.unit.test.ts` hold the refusal from the grouping and through `prepareDocumentPair`,
each by its class,
its whole text and the line `refusalText` gives,
beside a control page whose greeting has an original of its own.

Measured by the lead with the package's slice census on `cf50117c5`,
before this fix:
92 complete pairs and 1,259 slices over the pinned corpus,
and no page refused.
So no page of the corpus reaches the refusal under the deterministic aligner;
a roster's pairing still can,
as the supplied walks of the probe did.

Open to the owner's veto,
and open:
the page is refused,
where the other way carries the block with the archive's bytes untouched so that the entry proceeds.
A follow-up was launched for that and had not handed back when this was written.
It has since:
B247 (`df807a8b5`) carries such a block as the archive has it,
and removes the refusal class this entry added.
Seen by the agent and not looked into:
two generated inputs of the second seed ended in `PlacementLayoutError`,
from supplied pairings that repeat a pair.

Recurrence:
`mistake-prevention.md`,
"Refusals inside a composed operation":
a step that cannot place something refuses it there,
by name,
in a class a log line may print.

## B247: a translation block ahead of a sealing note that no paired run could take stopped the whole page

Red in `3d4558d9c`,
fixed in `df807a8b5`.

Found on 2026-10-06 (UTC),
as the item B219 left open:
the commit that added `UnplacedTranslationBlocksError` made the drop loud and left the page refused.
`mergeOneSidedRuns` (`group-merge.ts`) threw for a held translation block when a sealed run stood in the section,
where the archive's own bytes could be kept.
The agent's seeded generator of 3,000 sections reached the refusal for 176 on one seed
(its measurement,
not rerun by the lead).

The fix:
`mergeOneSidedRuns` hands such blocks back as `unplacedTargetIds`,
`groupNodesSealed` and `subdivideSealedChunkPair` carry them,
and `prepareDocumentPair` keeps the blocks in no slice,
lists them in `unclaimedTargetBlocks`,
and names them by positional id in an `alignment target-unplaced` finding.
The refusal class is removed with its barrel line and its inventory name,
since no translation block remains that cannot be carried.
The cases are in `group-aligned.unit.test.ts`:
"CARRIES a translation block ahead of the sealing note that no paired run can take as unplaced,
with no run for it and the letter's original sealed",
and "PREPARES the same page through prepareDocumentPair with the greeting kept as the archive has it,
handed to the block correction round and named in a finding by its id".

The agent's measurements,
not rerun by the lead:
after the change all 9,000 sections of three seeds prepare,
and every translation block lands in exactly one of slice,
sealed,
declined and unplaced.
Measured by the lead:
83 named test files pass,
with the 37 source scans and a clean type check;
the slice census over the pinned corpus prints the same sixteen summary lines as before the change
(92 complete pairs,
1 incomplete,
1,259 slices).
No page of the corpus met the old refusal,
so that census does not exercise the new path.
The whole suite ran on `df807a8b5` in a side checkout:
1,629 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 37 source scans passing.
No cache version moved for B247 to B253:
the stores key on the pipeline digest,
which this build changes (the agent's note).

Open to the owner's veto:
carrying an unplaced block where refusing the page is the alternative;
the unclaimed record not telling unplaced from declined,
which only the finding's label does.

Recurrence:
`mistake-prevention.md`,
"Refusals inside a composed operation":
a step that cannot place a part keeps the part as it stands where that is sound,
and refuses the whole only where nothing can be kept.

## B248: a translation block held at a seal crossed it and was written out of order

Fixed in `df807a8b5`,
with its case,
which the agent ran red in its worktree.

Found by the generator of B247,
on 2 of 3,000 sections of another seed,
which threw a layout fault (`PlacementLayoutError`,
`overlaps-previous`).
`mergeOneSidedRuns` left a held translation block,
an unpaired heading ahead of the note,
held past the seal,
and it joined the original after the note,
so a slice at offset 0 came after an insertion at the seal's end.

The fix:
a translation block still held at a seal becomes unplaced there.
The case is "PREPARES a page whose heading ahead of the sealing note is unpaired
and whose later originals follow the seal,
keeping the heading out of every run that stands after the seal".

Recurrence:
`mistake-prevention.md`,
"Refusals inside a composed operation".
