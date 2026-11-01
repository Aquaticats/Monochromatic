# Audit ledger: text that shows nothing

Text a reader sees as blank that passed for wording,
because the check asked `trim()` or asked nothing.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B40: text that shows nothing passed every check that asked `trim()`

Status:
fixed 2026-09-30 (UTC),
found while checking whether the B39 fix reached the consolidation lane,
whose judge call shares the changed exit.
The checks deciding whether a model's wording or reason said anything asked `trim()`,
which removes ECMAScript whitespace and keeps every invisible character that is not whitespace.
The intake fold (`invisible-variants.ts`) removes the soft hyphen (`U+00AD`),
the zero-width space (`U+200B`),
the word joiner (`U+2060`)
and the byte order mark (`U+FEFF`),
and `trim()` keeps all of them but `U+FEFF`.
Measured on the build from before the fix
(`t8b6-blank-slate-probe.mjs` in the audit's scratch folder):
a translator reply of `U+200B`,
`U+2060`
or `U+00AD` alone passed `isTranslateReportWire`,
and `buildTranslateCandidates` put it on the slate as a candidate of no characters.
The slate filter's TSDoc said nothing blank reaches it,
and `BlankSelectionError`'s said its exit was unreachable;
both were false.
The deterministic floor passes a proposal of `U+3000`,
`U+00A0`,
`U+200B`,
`U+2060`,
`U+3164`
or `U+115F` alone over an original that says something
(`t8b6-b39-blank-probe2.mjs`),
and the Hangul fillers are not folded at all.
The same reading let a zero-width reason meet "Reason before vote" in the panel ballot,
reach the polish as a gate or slate objection,
print as a judge's reason in the consolidation brief,
anchor a source-supported archive block,
stand as a coverage quote,
pass as a short image reading,
join the consolidation slate as a lane text,
and write an anchor the splice refuses blank text for.
Whether any run met it is not measured.

Fix:
`rendersAsNothing` (`renders-as-nothing.ts`) reads a text as nothing when every code point is Unicode White_Space,
Default_Ignorable_Code_Point
(`DerivedCoreProperties.txt`,
Unicode 18.0.0)
or a control,
one code point at a time,
and every one of those checks asks it:
the translator and repair reply guards,
the slate builder,
`blankAgainst` on both sides,
the absent slice's record check in `translate-slice.ts`,
the splice's anchor check on both sides,
the lane offer,
the panel ballot's missing reason,
the gate and slate objections,
the consolidation brief's findings and reason,
the archive-block review's anchor quote,
the coverage quote,
and the image reading's "nothing at all".
Every character the fold removes is default-ignorable
and every one it turns into a space is White_Space,
so the fold cannot change the answer;
the slate builder folds first and asks it of the folded bytes all the same.
The floor is left as it is:
every route to it passes one of the reply guards first,
or the lane offer's check,
and a reply guard's refusal is the designed re-ask.
The census of the package's 154 lines calling `trim()`
(`t8b6-b40-trim-census.txt`)
left the rest,
which read parsed values,
settings,
corpus text
or one character's spacing.
The editor and refiner wires fold too,
but their guards take any string as a replacement,
the empty one included
(`isEditorEditWire`,
`isRefineRewriteWire`),
so a replacement the fold empties passes no check it would otherwise fail.
Every case is pinned beside its check's siblings,
and `renders-as-nothing.unit.test.ts` pins the reading itself,
past the first plane included.

The B39 exit does not reach the consolidation lane after this (an inference from the code):
that lane counts floor survivors as heard,
a survivor passed the reply guard,
and the fold cannot empty what the guard passed,
so its slate is empty only where nobody survived,
which still raises.
The records of the translate,
consolidation,
repair,
coverage,
archive-block review
and image reading stages can change,
and so can page assembly's,
so the cache-account audit before a launch takes this change beside B36 to B39.

## B128: a claim's quoted side chosen again, by its raw text

Red in `0938d6a21`,
fixed in `9ff566830`.
Found on 2026-10-05 (UTC) by an independent reviewer
reading the cases the MiMo trial added,
one of which asserted the wrong result as the intended one;
it was reproduced on the built package before anything changed.
The trial changed neither production file
(`git diff 28303c42a 7f069adc9 --stat` printed nothing for them):
the defect is older than the trial.

A prober's claim of introduced damage quotes one of two sides,
wording the edit added (`evidence`)
or wording it dropped (`omittedText`),
and `screenEvidence` (`introduced-defect-screen.ts`) decides which by the screened text of each.
Two readers decided again,
by whether the raw `omittedText` was the empty string:

- `toClaim`,
  in the same module,
  chose the quote that `restatesPriorIssue` compares with the accepted issues.
  A claim corroborated on its evidence,
  its omitted side three spaces,
  was compared on the spaces,
  restated nothing,
  and stayed `corroborated`:
  counted as damage the edit introduced,
  though its evidence restated an accepted issue.
  The same claim with an empty omitted side was `pre-existing`
  (probe of the unfixed build).
  The wire guard admits any string in `omittedText`.
- `renderClaim` (`corpus-run/probe-verify-sheet.ts`) chose the quote and the direction a human grader is shown:
  the same claim was printed as "wording the edit DROPPED" over a quote of three spaces
  (same probe).

`screenEvidence` itself asked whether a side was blank of trimmed text,
which keeps every invisible character that is not whitespace.
A one-sided claim with a zero-width space (`U+200B`) or a Hangul filler (`U+3164`) on its other side
read as anchored both ways and was called `unanchored`,
so a claim of real damage was set aside as a wire fault
(the red run printed `unanchored` for both).
That is the question "Text that shows nothing" gives to `rendersAsNothing` (ledger B40),
asked here of an empty-string comparison.
The red run of the two test files printed 3 [PASS] and 6 [FAIL] lines for the screen
and 3 [PASS] and 4 [FAIL] lines for the sheet.

The fix:
`screenEvidence` asks `rendersAsNothing` of each side.
`toClaim` takes the side from the verdict it holds
and compares the screened text the differential read,
with the prior quotes through the same screen;
`renderClaim` takes the side from the claim's recorded admissibility.
`restatesPriorIssue` returned false for a blank quote;
no corroborated claim reaches it blank now,
and a blank quote that ran on would sit inside every prior quote and dismiss the claim,
so the return is an `unreachable:` throw.
`renderClaim` throws likewise for a claim the screen did not corroborate:
`probe-verify.ts` hands a sheet only corroborated claims,
and the damage sample strips claims before formatting.

The trial's case,
"KEEPS a corroborated claim whose quoted side flattens to nothing out of the prior-issue dismissal",
pinned the defect,
and could not have failed on the guard it named:
its issue carried an id the region did not serve,
so no prior quote was ever compared.
It is replaced.
The sheet test's fixture marked every claim `corroborated`,
one of them carrying both sides,
which the screen calls `unanchored`;
it takes each claim's admissibility now.

Whether a corpus run ever met such a claim was not measured:
no stored probe artifact was read for a padded or invisible omitted side.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal" and "Text that shows nothing".

## B189: quotes and spans that show a reader nothing were anchored

Red in `806039cb1`,
fixed in `2b789a4ae`.

Found on 2026-10-05 (UTC)
by a census of every blankness test in the package's production source,
after B128.
The fixing agent read the 25 wire modules field by field,
swept the source for the comparisons,
and ran a syntax-tree pass that found 397 tests of blankness at its base.
It fixed 20 sites,
recorded here and in B190 to B194,
and held the rest,
each with why its question is another one (B195).
Each fixed site tested a text a model wrote by a comparison with the empty string,
a trimmed comparison or a length,
where the package's one question is `rendersAsNothing`.

`locateQuote` (`locate-quote.ts`) refused a quote only when it was the empty string,
so a critic's quote of one ideographic space anchored on the one such space a source held.
`anchorLocatedSpan` (`rendering-audit-anchor.ts`) compared the canonical locator and focus with the empty string,
so a focus of one space anchored inside its locator.
`readSide` (`rendering-audit-screen.ts`) decided again whether a side quoted anything,
by the same comparison on the raw text,
so an omission whose unused candidate side held `U+200B` was dropped as `forbidden-side-quote (candidate)`.
`quoteIsIn`,
`mergedAttestations` and `attestedDetailsOverlapping` (`reference-attest-match.ts`)
searched for a quote without asking whether it showed anything,
so an item quoting `U+200B` on both sides verified against texts holding that character.

The fix:
each asks `rendersAsNothing` of the text the model sent.
Every case of this entry and of B190 to B194 hands its field spaces,
`U+200B` and `U+3164`,
written as escapes,
and asserts the whole result an empty field gets.
The red run printed a failure for each of 27 cases in 23 files.

The whole suite ran on `2b789a4ae` in a side checkout:
1,605 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 36 source scans passing.

No cache version moved,
for this entry or for B190 to B194.
The cache account audit of 2026-10-05 (UTC) found 13,714 slice-cache records,
the newest written 2026-09-27T04:26Z,
with every cache version set since then,
so no stored record exists under a current version.
Whether any stored reply ever carried such text was not measured.

Open to the owner's veto:
padding on a rendering-audit claim's unused side is read as empty and the claim kept,
where the other way goes on dropping the claim.

Recurrence:
`mistake-prevention.md`,
"Text that shows nothing";
`readSide` beside the anchor is also a second reader of one emptiness,
under "Two layers reading one refusal".

## B190: a reading, a finding, a title, a category and a replacement that show nothing counted as said

Red in `806039cb1`,
fixed in `2b789a4ae`.

Found on 2026-10-05 (UTC) by the same census.
`readingMakesSense` (`image-reading-sense.ts`) asked whether a picture reading was blank only through its length floor,
so sixteen Hangul fillers,
or forty zero-width spaces,
read `usable`.
The naturalness review's wire guard (`absolute-naturalness-review-wire.ts`) took a `problem` of `U+200B` as a finding.
`mostGiven` (`page-title-lexicon-stage.ts`) counted a seat's rendering of `U+200B` as a voice,
so two such seats outvoted one seat rendering the title in words.
`repair-damage-evidence.ts` named a prober's category of `U+3164` as itself,
where an empty one is named `unspecified`,
and `repair-sheet.ts` showed an editor's replacement of `U+200B` as text,
where an empty one is said to be a deletion.

The fix:
each asks `rendersAsNothing`.
The reading is refused `too-short` before its characters are counted.
That file was merged by hand,
since B186 had changed the same lines after the agent's base:
the screen asks whether the reading shows anything and then asks `solidCharacters`.

Open to the owner's veto:
a long reading of invisible characters is refused under the existing `too-short` clause,
where the other way gives it a clause of its own;
and the repair sheet says a replacement that shows nothing was a deletion.

Recurrence:
`mistake-prevention.md`,
"Text that shows nothing".

## B191: an archive block revision that shows nothing shipped as wording

Red in `806039cb1`,
fixed in `2b789a4ae`.

Found on 2026-10-05 (UTC) by the same census.
The archive block review's `replacementText` spells a removal as the empty string.
`runArchiveBlockReviewStage` (`archive-block-review-stage.ts`) never asked whether a revision showed anything,
and `revisionShapeFindings` (`archive-revision-shape.ts`) asked only whether it was empty,
so a revision of `U+200B` or `U+3164` passed the shape floor and reached the selectors as revised text,
while one of three spaces was withheld as holding no block
(the fixing agent's probe).

The fix:
the stage reads a revision that shows a reader nothing as the empty removal,
once,
and every later reader
(`archive-replacement-candidates.ts`,
`corpus-run/archive-block-repair.ts`,
`archive-block-selection-evidence.ts`)
reads what was decided.
The shape floor throws `unreachable:` on a revision that is blank and not empty.
One assertion went with it:
that a revision of three spaces is withheld,
which `f9babaec4` had pinned as the behaviour it found.

Open to the owner's veto:
a revision that shows nothing is the removal an empty one spells,
where the other way withholds it as the shape floor withheld three spaces.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal":
the blankness scan holds the three raw readers,
each on the reason that it reads the decided value.

## B192: transport fields that show nothing counted as present

Red in `806039cb1`,
fixed in `2b789a4ae`.

Found on 2026-10-05 (UTC) by the same census.
`extractAnthropicCompletion` (`anthropic-completion.ts`) returned tool arguments of spaces and `U+200B`
in place of the prose answer beside them.
`extractStreamedCompletion` (`stream-completion.ts`) kept a refusal channel of `U+3164` as a refusal,
and `readJsonOutcome` (`chat-json-outcome.ts`) chose the raw text by whether the reply's text was the empty string,
two readers of one emptiness.

The fix:
all three ask `rendersAsNothing`.
A voice formerly lost to such a reply is heard now,
which no cache key reads.

Open to the owner's veto:
a refusal channel or tool arguments that show nothing count as absent.

Recurrence:
`mistake-prevention.md`,
"Text that shows nothing".

## B193: pieces of a candidate or a record that show nothing read as worded

Red in `806039cb1`,
fixed in `2b789a4ae`.

Found on 2026-10-05 (UTC) by the same census.
`isWorded` (`translate-unwrapped-link.ts`) took a link whose words were `U+200B` as worded.
`fencedLabel` (`translate-sheet-leak.ts`) took `U+3164` between two fence runs as a label.
`composeInsertion` (`insertion-separator.ts`) opened a paragraph for a fragment of `U+200B`.
`assertWordingCoherent` (`wording-coherence.ts`) passed a decided text of `U+200B`
at a place the archive never translated,
though `assertAbsentSliceFilled` refuses it where the record is built.
The coverage control (`corpus-run/coverage-control.ts`) compared a roster's spans,
and the text its cut left,
with the empty string,
and asked a second round about a page the cut had left as one line break.

The fix:
each asks `rendersAsNothing`,
the coverage control at all three of its tests.

Two of these are floors of cached stages:
the link floor refuses more and the fence floor accepts more,
for a candidate carrying invisible-only text in that one place.
No version moved,
on the audit named in B189.

Open to the owner's veto:
`assertWordingCoherent` refuses a record of an older build
that holds an invisible decided text at a place the archive never translated,
where it passed one.

Recurrence:
`mistake-prevention.md`,
"Text that shows nothing".

## B194: the page assembly took invisible text for a rendering

Red in `806039cb1`,
fixed in `2b789a4ae`.

Found on 2026-10-05 (UTC) by the same census,
on its second reading.
The page text of a slice is the replacement a lane wrote where there is one,
and the archive's own text otherwise (`corpus-run/assembly-page-text.ts`),
so the assembly passes read model wording;
the census first classed page text as parsed text and missed them.
`renderedHeadings` (`corpus-run/title-reference-unify.ts`) took a heading of `U+200B` or `U+3164`
as the title's English,
and every reference to the title was rewritten to that character,
with a `title-reference-unified` finding.
`nameAuthorities` (`corpus-run/contributor-name-authorities.ts`) took a signature of the same
as the signer's rendering,
and the restore wrote it over the name in the heading naming the signer.
`appearancesIn` (`corpus-run/handle-gloss-place.ts`) took a parenthesis holding a space or `U+200B` as a gloss.

The fix:
each asks `rendersAsNothing`.
A signer whose page rendering shows nothing falls to the pinyin reading,
which the restore writes into the heading and the signature.

Open to the owner's veto:
`readSignature` (`corpus-run/attribution-line.ts`) still reads a name span of invisible characters as a signature,
so the restore has a place to write the signer;
read as unsigned,
the line would stay nameless.

Open when this entry was written,
closed in B260 (`68df23b05`),
seen by the agent and outside this family:
a candidate that signs with an invisible name passes the signer floor (`translate-signer-handle.ts`),
which the restore now repairs at assembly;
invisible characters count toward the four code points `archive-block-evidence.ts` asks of a source quote;
and `nameAuthorities` takes the archive's signature name with no blank test,
which is parsed text and was left.

Recurrence:
`mistake-prevention.md`,
"Text that shows nothing":
a census that classes a text as parsed first reads where the text comes from.

## B195: no scan held the rule that blankness is asked of `rendersAsNothing`

Added in `2b789a4ae`.

Found on 2026-10-05 (UTC) by the same census:
"Text that shows nothing" rested on habit and review,
which let B128 and the twenty sites of B189 to B194 through.

`src/blank-text-tests.unit.test.ts` (moved on 2026-10-18 to
`package/translation-repair/source-scan/src/blank-text-tests.unit.test.ts`),
which `source-scans` runs,
reads every production file's syntax tree for a comparison with the empty string,
written as a string or an empty template,
either way round;
the length of a trimmed text,
or of a name its function declares from one,
compared with anything;
the length of a name annotated as a string compared with zero or asked whether it is less than one;
and any of those tested for truth.
Sites are keyed by file and nearest enclosing named function or class,
with a count,
and compared whole against a held list,
so a new function,
a second test in a listed function
and a stale entry each fail;
every held key carries why its question is another one.
The list held 297 keys at the agent's base and 298 at `2b789a4ae`,
where the lead corrected it for code that had landed since.

That it can fail
(the fixing agent's runs):
on a scratch copy seeded three ways it named each seed,
and on the source without the fixes it named 17 of the 20 fixed sites.
The three it did not name:
the reading's length floor,
where the fix puts a question ahead of a comparison that stays;
the review stage,
which never asked;
and the gloss reader,
where an offset comparison stood for an empty span.
Out of its reach,
as its module doc says:
the length or truth of a string no annotation names,
a constant holding the empty string,
a blank needle handed to a search,
a length floor on text that was never trimmed,
and a trimming helper whose result another function compares.

Open when this entry was written,
closed in B261 (`68df23b05`):
one held key is a known gap,
`introduced-defect-screen.ts#isUsable`,
which keeps a prior quote of one invisible character;
the file was another change's,
and fixing it removes the key.

Open to the owner's veto:
the scan is broad,
with a long held list,
where a narrower one would hold fewer keys and see less.

Recurrence:
`mistake-prevention.md`,
"Text that shows nothing",
whose enforcement paragraph names the scan.

## B260: the signer floor, the source quote floor and the archive authority took invisible text for a name or a quote

Red in `5abbb9c66`,
fixed in `68df23b05`.

Found on 2026-10-05 (UTC),
closing the sites B194 left open.
`signerFinding` (`translate-signer-handle.ts`) passed a candidate that signed with `U+200B` or `U+3164`,
since a name with no Latin letters matches no reading and the function answered no finding,
and `signerHandleFindings` let a page signer of the same kind excuse the candidate.
`isArchiveSourceQuoteAnchored` (`archive-block-evidence.ts`) counted invisible characters toward its four code points,
so two ideographs among zero-width spaces anchored a claim.
`nameAuthorities` (`corpus-run/contributor-name-authorities.ts`) took the archive's aligned signature name
with no blank test,
and the restore wrote a zero-width space over the heading naming the signer.

The fix:
each asks `rendersAsNothing`.
The floor refuses a signer that shows nothing,
with the instruction it gives a signer left in Han;
a page signer that shows nothing governs nothing;
the quote counts only what shows;
and an archive signature that shows nothing falls to the page's rendering and then to the pinyin reading.
`archive-block-evidence.ts` had no test file of its own;
with one added,
its line left the allow-list of `own-unit-tests.unit.test.ts` (the lead's edit,
in the red commit so that scan stayed green).

Open to the owner's veto:
the quote floor counts no space either,
so a quote of three ideographs with spaces between them,
which anchored,
no longer does;
the signer floor refuses an invisible signer where assembly would have repaired it to the bare reading.

Recurrence:
`mistake-prevention.md`,
"Text that shows nothing".

## B261: a prior quote of one invisible character dismissed every claim that carried it

Red in `5abbb9c66`,
fixed in `68df23b05`.

Found on 2026-10-05 (UTC),
closing the gap B195 named when it held this key.
`isUsable` of `collectPriorQuotes` (`introduced-defect-screen.ts`)
kept a prior quote that the screen's fold left non-empty,
so a stored issue quoting one `U+200B` or `U+3164` stayed among the prior quotes,
and an added-wording claim whose evidence held that character was counted as restating the issue:
it read `pre-existing` where it was `corroborated`.

The fix:
`isUsable` asks `rendersAsNothing`.
The lead removed the held key `introduced-defect-screen.ts#isUsable` from `blank-text-tests.unit.test.ts`,
its gap closed,
and corrected the returns line of `collectPriorQuotes`,
which still said "blank ones dropped".

Recurrence:
`mistake-prevention.md`,
"Text that shows nothing":
a filter that drops blank text asks the question that names it.

## B271: an entry note under a heading with no words named an empty heading

Red in `f18d0b9b1`,
fixed in `b8e30dce2`.

Found on 2026-10-06 (UTC) by the coverage cluster over the library files from a to l (T8).
`commentAnchor` (`entry-notes.ts`) names the heading an editor's comment sits under,
so a sheet working one slice knows which heading "this title" points at.
For a heading that is nothing but its opening marks the name was empty,
and the sheet line read `- ARCHIVE editor comment under heading : <note>`,
as if a word had been dropped.
The agent pinned that line as current behaviour and said it did not endorse it;
the lead rewrote the case to expect that the line says the heading has no words.

The fix:
the line reads `under a heading that has no words`,
asked through `rendersAsNothing`,
so a heading of invisible characters reads the same way.

Measured by the lead:
of 42 named test files on the red tree 41 pass,
and `entry-notes.unit.test.ts` fails by the one assertion meant to fail;
on the fix tree 8 named test files pass,
the note's own among them,
with the 39 source scans and a clean type check.
Not measured:
no page of the pinned corpus was searched for such a heading.

Recurrence:
`mistake-prevention.md`,
"Text that shows nothing".
