# Audit ledger: the repair lane's smaller items

The smaller repair-lane findings collected as L14,
each under its own letter.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## L14: smaller items

Status:
(a) to (d) fixed;
(e) closed:
seven sites a census artifact,
two faithful refrains.

### L14(a): the resolution checker sheet carried none of the panel's evidence

Status:
fixed in `fe0fc1f13`,
guarded red first in `01a2dc31f`.
The sheet showed the ORIGINAL,
the REVISED TRANSLATION,
and each claim's category,
severity and summary.
The panel that accepted each issue also read the DECLARED NAMES block with its rules,
the cited references,
and the claim's own quotes.
Checker `worse` ballots strip an edit (L3) and roll a rewrite back (L11),
so a checker that could not see a declaration could count a declared handle kept as written as damage.
The M32 step came first:
`buildResolutionMessages` is the whole sheet
(`runCheckerStage` sends `plan.messages` unchanged),
and it took no identity or reference parameter,
so no other path could have carried either.

The fix (`resolution-sheet-evidence.ts`):
the declared names before the documents,
with `DECLARED_IDENTITY_RULES` and one checker line
(a revision moving a declared rendering away from its declared value is worse);
the cited references after the revised translation,
with a checker line that a referenced detail is never a reason for worse
and that the references never reopen an accepted issue,
since the panel judged with them and the reference screen already voided attested additions;
and each claim's quotes,
JSON-encoded,
the TRANSLATION side labelled as the text before this revision.
The encoding is measured need:
349 of 45,860 claim quotes over 266 artifacts span more than one line
(`~/temp/agent/audit-glossary-fix/l14-summary-newlines.mjs`).
Rules ride with their blocks,
as on the introduced-defect probe (H8),
so a page declaring nothing and linking nowhere reads the rules it always did.
The stage,
the repair proof and its worse-vote recheck,
and the refinement recheck all forward both contexts.

Guards:
`resolution-sheet-evidence.unit.test.ts` (the builder)
and `checker-evidence-threading.unit.test.ts` (each caller,
both directions).
Two guard defects surfaced while greening them.
The threading control's fixture had a heading,
and preparation writes the archive's rendering of a heading into the identity context,
so the control page declared a name and its checker sheet was rightly shown the block
(`ab7ad54f0`;
a `prepareDocumentPair` probe confirmed the heading-less fixture yields neither context).
The mutation check (24 mutants,
control surviving) left one survivor:
a fence chosen without the declared names,
because the fence case gave the references the longer run of equals signs,
and a fence clearing that run cleared the identity's too;
one sheet per context now carries the run alone (`b507cbf28`),
and both fence mutants are caught.
Cache:
rides inside repair 34 and refine 5,
checked on 2026-09-28:
the only slice-cache file not older than 04:26 UTC on 2026-09-27 is the consolidation entry written that minute.

Scope kept out,
as a candidate rather than bundled:
the probe's sheet also carries the neighbouring window and community renderings,
and the checker sheet carries neither;
and the audit's finding also named the text before the revision,
which the claim quotes now carry only where a claim quoted it.
The panel's measured regression from adding sheet content (`NEARBY_RULE` in `adjudicate-prompt.ts`)
says sheet additions need evidence first.

### L14(b): editors write neighbouring text into an envelope

Status:
fixed in `49d3c285b`,
guarded red first in `8408e8864`.
The audit reported editors writing neighbouring text into a region and the chunk judges choosing it
(CuspariaKLSY11 slice 3,
shihai4h2 slice 28),
and that neither shipped.

What was measured (`~/temp/agent/audit-glossary-fix/l14b-census.mjs` and its siblings):
56 of 6,222 chunk candidates over 266 artifacts carried a sentence of 12 characters or more
more often than the slice's archive English did,
in 43 of 2,476 rounds;
the judges selected such a candidate 3 times;
and Carena0442 slice 14 on carena-rerun-20260902 (a schema 9 build) shipped two such sentences
through a repair-lane win,
so "neither shipped" does not hold for the family.
Some of the 56 are noise:
URL and link fragments the sentence splitter cut at a period,
and one blockquote line accounts for 9.
The editor sheet shows the whole TRANSLATION and the whole ORIGINAL (rendered 2026-09-28),
so a rule about text outside the region asks for nothing the editor cannot check.

What was not measured:
the class the audit described,
neighbouring ORIGINAL content re-translated into a region in new words.
A verbatim census against the adjacent slices' archive English found none,
and its positive control failed:
at CuspariaKLSY11 slice 3 the selected candidate repeats no sentence and no 4-word phrase,
and shares none with the neighbouring slice,
so string matching cannot see that class,
and the zero is not evidence of its absence.
The fix rests on the audit's reading of that class and on the measured verbatim family.

The fix:
the editor sheet says a region carries only what the ORIGINAL says at that place,
and that a sentence the TRANSLATION already carries outside it,
or content a NEARBY ORIGINAL passage says,
goes in only where the ORIGINAL says it again,
as a refrain the TRANSLATION rendered once;
the faithfulness test both editor selections read says the same of candidates.
Conditioned on the ORIGINAL because the omission rules say to translate ALL of the missing content,
and a refrain the archive rendered once is such content.
Mutation check:
the unwired editor rule,
the rule without its condition,
and the dropped judge line are each caught;
the control survived.
Cache:
rides inside repair 34,
same check,
same result.

Candidate kept out:
prefer,
among indecision fallbacks,
a patch that repeats no sentence.
Its one measured case is XingZ631 slice 37,
where the indecision fallback carried a repeat,
the repair was lost,
and the slate endorsed the standing text.

### L14(c): `selectChunkPatch` wording on declines

Status:
fixed in `e7e530564`,
guarded red first in `ebe1c8ffd`.
The chunk selection used the shared promise
"the caller keeps text it already trusts when you decline".
A decline names no candidate and counts as no vote:
the existing English is kept only when every judge declines (`rejection`),
and a round the naming judges cannot decide (`indecision`) sends the editor patch that landed the most edits
on to the checkers (`pickFallbackCandidate`).
The audit called this latent,
with no chunk round holding a majority of declines.
Measured,
that was wrong:
over 2,476 chunk rounds in 266 artifacts,
656 had at least one decline,
14 had declines outnumbering the ballots naming a candidate,
10 of those were read as indecision (TianqiChen66621 chunk 17:
4 declined,
1 named),
4 selected a candidate the minority named,
and none had every judge decline
(`~/temp/agent/audit-glossary-fix/l14c-census.mjs`).

The vote rule stays.
A decline as an abstention that does not count is the measured policy for the consolidation gate's `neither`
(`doc/planning/the-third-rendering.md`),
and the fallback repairs because the panel ruled its issues real (`editor-candidates.ts`).
The chunk sheet now states that rule (`CHUNK_DECLINE_CONSEQUENCE`),
and both behaviours it states are pinned (`chunk-decline-consequence.unit.test.ts`).
The shared promise stays for the envelope selection,
where any declined round leaves the envelope unedited,
so a round that declines keeps the trusted text there.
Mutation check:
the unwired constant,
the dropped unanimity,
the unstated fallback,
and an indecision that keeps the existing English are each caught;
the control survived.
Cache:
rides inside repair 34,
same check,
same result.

### L14(e): a repaired text repeating a sentence no recorded stage wrote

Status:
closed at all 9 sites,
7 a census artifact and 2 faithful refrains (classified 2026-09-28);
found while measuring L14(b).
In 9 slices the repaired text carries a sentence more often than the archive English
while no chunk candidate of the slice does and no repair region's replacement carries it:
6 whose rounds were chunk and envelope selection
(Mio12-20260910,
Mio16-20260916 and Mio21-20260916 slice 14,
Mio20-20260916 slice 15,
TianqiChen66613 slice 12,
XingZ626 slice 44),
2 with a refinement round (XingZ628 slice 53,
XingZ631 slice 54),
and 1 refined without an accuracy patch (XingZ6010 slice 38).
At Mio12-20260910 slice 14 the repeated sentence is 12 characters,
absent from the archive English and from both candidates,
and the repaired text is 36 characters longer than the selected candidate,
so a step after selection wrote it (`~/temp/agent/audit-glossary-fix/l14b-origin.mjs`,
`l14b-mio.mjs`).

What that step did (`l14e-lines.mjs`,
`l14e-all.mjs`,
`l14e-substr.mjs`):
at all 9 sites the repaired text holds more lines than the archive English and the selected candidate
(1 or 3 lines becoming 11 to 17,
and 76 becoming 91 or 92),
so a later step set the text on more lines
(first written here as the ORIGINAL's lines;
the ORIGINALs of the two sites this entry names hold 3 lines and 1,
and the step is the semantic wrap,
`wrapReplacementText`:
both repaired texts equal their own wrap,
and wrapping TianqiChen66613's selected candidate gives its 13 lines,
`l14e-original-wrap.mjs`),
and the "repeated sentences" are short verse fragments (a single word and a comma,
a four-word line)
that the census's sentence rule counts only once they stand on their own lines.
Counted as substrings,
7 of the 9 carry the fragment exactly as often as the archive English or the candidate:
nothing was said twice,
and the census,
not a stage,
made the repeat.
Two carry a fragment more often:
TianqiChen66613 slice 12 (archive 1,
candidate and repaired text 2)
and XingZ6010 slice 38,
refined without an accuracy patch (archive and candidate 0,
refined text 4).
Neither ORIGINAL repeats a line of text (`l14e-source.mjs`;
only markup lines repeat),
which left a refrain within a line open while the slice's own ORIGINAL was not in the artifact.
Rebuilt from each run's own preparation
(`rebuildPreparation`;
each rebuilt slice's archive text equals the run's recorded row,
`l14e-original-structure.mjs`),
both ORIGINALs repeat a phrase within a line,
counted as the most repeated Han runs (`l14e-original-ngrams.mjs`):
at TianqiChen66613 slice 12 a four-character phrase stands twice,
matching the candidate's and the repaired text's two where the archive has one,
and at XingZ6010 slice 38 a three-character adverb opens a clause four times,
matching the refined text's four where the archive and the candidate have none.
Both are the ORIGINAL's own refrain,
rendered as often as the ORIGINAL says it;
how the archive English renders each refrain was not read.
No rule follows:
the refine sheet needs none about saying something twice,
since its only case turned out faithful.
Lesson for any later census of repetition:
count substrings as well as split sentences,
since setting a text on its lines changes what a sentence splitter sees.

### L14(d): model-written and quoted text rendered raw on line-based sheets

Status:
fixed in `3be658509`,
guarded red first in `25d549aea`,
with every line end pinned in `bfc458ad3`;
found while fixing L14(a).
The panel sheet renders each evidence quote raw after `- evidence (SIDE): `,
and both the panel and checker sheets render each claim summary raw.
349 quotes and 3 of 23,714 summaries over every artifact carry a line break,
so their later lines stand as unlabelled lines between claims.
None opened a `CLAIM`,
`GROUP`,
`ISSUE` or `REGION` line in the artifacts measured,
but a quote or summary may,
and a forged opening line renumbers every ballot after it.
The same raw summary split the filing log lines (`claim-filers.ts`),
so every line after the first lost its chunk tag,
and the grading sheet rendered quotes raw where the repair sheet folded them.

The fix (`sheet-line-text.ts`):
a summary is one sentence of prose and folds onto one line with `flattenSpace`
(moved there from `introduced-defect-screen.ts`,
since the probe wire now needs it and the screen imports the wire);
a panel quote is evidence and keeps its lines,
each later one indented under its evidence item by `indentContinuation`,
so 99 percent of panel quotes stay byte-identical;
the grading sheet folds quotes as the repair sheet did;
the checker JSON also escapes NEL,
LS and PS,
which `JSON.stringify` leaves raw.
The helper treats LF,
VT,
FF,
CR,
NEL,
LS and PS as line ends and CR LF as one;
none of the 45,860 quotes carried VT,
FF,
NEL,
LS or PS
(`~/temp/agent/audit-glossary-fix/l14-unicode-breaks.mjs`),
so those are covered for the class,
not for a seen case.
Mutation check:
20 mutants over each fold,
the indent,
CR LF and each line end,
all caught,
with the control surviving in a separate run
(the first spec's control pattern opened and closed the region,
so the harness skipped it).
Cache:
rides inside repair 34 and refine 5,
same check,
same result.
