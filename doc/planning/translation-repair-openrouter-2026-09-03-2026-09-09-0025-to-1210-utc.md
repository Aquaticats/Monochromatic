# OpenRouter and the corpus passes: 2026-09-09, 00:25 to 12:10 UTC

What was launched,
read and decided on 2026-09-09 00:25 to 12:10 UTC.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## The twentieth class stops the fifth yuki pass at the carried-insertion guard and the Arita page is read, 2026-09-09, 00:25 UTC

The fifth `yuki418330012` pass (`2da6e7f22`,
22:42 UTC) did what the closure was built for:
`FOOTNOTES entry=yuki418330012 relabelled [^2]->[^1], [^1]->[^2] off the definitions the roster paired`
(the roster agreed on one pair,
`7 usable voices of 8 heard`,
the one refusal a definition paired with a body block,
and elimination gave the other),
`FOOTNOTES entry=yuki418330012 definitions moved into the original's order: [^1], [^2]`,
and the re-preparation paired `7 of 8 original and 11 of 12 translation blocks across 11 relations`,
both definitions among them.
It then stopped at 00:06 UTC,
83 minutes in,
`TALLY yuki418330012 status=INCOMPLETE ms=4998139 aborted=false error=translation repair interrupted:
carried-evidence-lost`.
The one unpaired original was the credits line (`条目贡献：真理 三三 娑娜`),
admitted as an insertion the archive already carries
(`slice 8: coverage=carried, missingDestinations=0`,
the roster's quote `Contributors for this entry: Zhenli, Sansan, Suona` anchored on the archive's one line).
The stage caches show what became of that line:
the repair and translate lanes shipped it as the archive had it,
and the consolidation's slate,
incumbent and every fresh candidate alike,
carried `Contributors for this entry:\nZhenli,\nSansan,\nSuona`.
That is the semantic wrap (`consolidate-wrap.ts`,
`wrapConsolidation`,
the site's own line-break convention applied to a slice the lanes changed),
and `assertCarriedInsertionsRemain` compared the anchored quote byte for byte against the wrapped page.
Every word of the credits was on the page.
THE TWENTIETH CLASS:
a guard that reads its evidence by a stricter reading than the one that produced it.
Fixed in `11143f681`:
the guard folds soft line breaks and normalizes punctuation on both sides,
the reading `locateQuote` anchored the quote with,
and its findings name each lost region
(`carried-insertion-evidence-lost slice N: "..."`) instead of a count alone.
The suite failed first
(the new case,
a carried region broken across a soft break),
then passed;
full suite 975 `PASS` and 0 `FAIL`,
oxlint and types clean.
Under the rule the seventeenth `hakureico` was killed at 00:22 UTC,
in consolidation since 23:46 UTC,
and the eighteenth `hakureico` (`~/temp/agent/hakureico18-20260908`,
pid 528810) and the sixth `yuki418330012` (`~/temp/agent/yuki6-20260908`,
pid 528921) launched at 00:22 UTC on `11143f681`,
each watched by the filtered poller.

The second `Arita` pass (`2da6e7f22`,
22:42 to 00:13 UTC,
`~/temp/agent/arita2-20260908`) settled on the build before the guard fix and is read as evidence for it,
since the fix touches only a guard the pass never tripped.
`TALLY Arita status=SETTLED slices=12 repairStatus=repaired repairIssues=44 repairAccepted=25 repairResolved=24
repairFindings=139 repairChanged=9 translateStatus=complete translateChanged=10 documentsDiffer=10 pageChanged=9
pageSilent=0 alignmentFindings=1 selection=contested ms=5459151`:
91.0 minutes on Bedrock and OpenRouter beside two other passes,
Synthetic's week returning for the translate lane.
The three checks passed:
the front matter is the archive's byte for byte (`FRONT MATTER entry=Arita authority=archive`),
8 `json false start` reads kept,
one `schema-mismatch` that was no false start
(deepseek-v4-flash-0731 returned an empty reply on `finish_reason=stop` in a consolidation round,
a voice lost),
and no error finish occurred;
one `InStreamProviderError` rode the retry ladder.
`verify-published` matched 1 of 1 at length (`wordings=12 silent=0 chars=6963=expected missing=0`).
The component this entry was queued for,
the double-quoted path,
shipped verbatim:
`<PhotoScroll photos={["${path}/photos/image0.webp"]} />` at line 156,
the archive's line 37.
Quotes on the archive's conventions (straight 2 and curly 14 on both),
apostrophes curly (24 against the archive's 21,
the three new ones inside rendered text),
one three-dot ellipsis where the archive has one and a second where the source carries one the archive dropped
(`不管从现在起命运的道路将会把我带向何方...只要我保持耐心`,
rendered `No matter where the path of destiny leads me from now on... as long as I stay patient`),
in the archive's three-dot convention.
Rounds:
editor 9,
refiner 17,
translate 12,
naturalness review 10,
lane contest 11,
polish gate 9,
`gate-kept-standing` 0,
`one-reader-only` 0,
`corroborated` 36.
113 voices lost at the straggler window and 15 `HTTP 429` retries,
three passes sharing the two providers.
Meters at the tally:
OpenRouter 147.83 USD,
Bedrock 195.98 USD.
The page is the first read page to carry the component.
The prose read against the source:
the archive's two errors are repaired
(`人间失格` is *No Longer Human*,
where the archive had *Indignation in the World*;
the sentence about dying on her birthday in 2027 and being one day younger,
which the archive dropped,
is back),
the roommate passage carries the direct question the archive softened,
and two nits stand:
`meet — won’t we?` spaces its dash where the archive writes them closed,
and the Yeats line keeps the archive's comma in `breast, or his lips` where the source quotes the poem without
one.
No class.

## The twentieth class's second face where a partial vote's quote is not evidence, 2026-09-09, 01:35 UTC

The sixth `yuki418330012` pass (`11143f681`,
00:22 UTC) relabelled,
reordered and re-prepared as the fifth had,
and stopped at 01:30 UTC,
67 minutes in,
as `carried-evidence-lost` again,
on the folded guard.
The fold was measured right
(the wrapped credits and the anchored one-line quote fold to the same string),
so the region the guard lost was another one.
The coverage round that admitted the credits line as carried had eight anchored voices:
seven said `full` and quoted `Contributors for this entry: Zhenli, Sansan, Suona`,
one said `partial` and quoted `She had attempted suicide many times before.`,
a sentence from another part of the page.
`judgeCoverage` recorded every anchored quote as evidence whatever its degree
(`weighed.filter(isAnchored)`),
against its own contract
("Exact target regions supporting full-coverage votes"),
and a lane rewrote that sentence
(the consolidation ships `defused many of Yuki's suicide crises` where the archive had
`rescued Yuki many times from suicide`),
so the guard stopped the entry over a region no full vote ever named.
The fifth pass had carried the same eighth vote;
its stop had two causes and the fold removed one.
Fixed in `d6db46519`:
the verdict's evidence is the full votes' regions alone,
the verdict suite's carried case now expects three regions of four voices and a new case pins the partial
voter's quote out,
shown to fail first;
and `tallyCaughtEntry` prints each finding of a `TranslationRepairInterruptedError` as
`INTERRUPTED <id>: <finding>` before the tally line,
since the tally carried the reason alone and two passes stopped with nothing in the log naming the region.
Full suite 975 `PASS` and 0 `FAIL`,
oxlint and types clean.
Under the rule the eighteenth `hakureico` was killed at 01:34 UTC,
in consolidation since 01:19 UTC,
and the nineteenth `hakureico` (`~/temp/agent/hakureico19-20260909`,
pid 546904) and the seventh `yuki418330012` (`~/temp/agent/yuki7-20260909`,
pid 547017) launched at 01:35 UTC on `d6db46519`,
each watched by the filtered poller.
The last reading before the kill:
Bedrock 195.29 USD,
OpenRouter 132.20 USD.
What the seventh `yuki418330012` must show:
the relabel and the reorder as before,
no `carried-evidence-lost`,
and a settled page.

## The seventh yuki page settles under the closed relabel and the full-vote evidence, 2026-09-09, 02:55 UTC

`TALLY yuki418330012 status=SETTLED slices=9 repairStatus=repaired repairIssues=33 repairAccepted=20
repairResolved=18 repairFindings=128 repairChanged=5 translateStatus=complete translateChanged=7 documentsDiffer=7
pageChanged=6 pageSilent=1 alignmentFindings=5 selection=contested ms=4563909`
at 02:51 UTC on `d6db46519`,
76.1 minutes on Bedrock and OpenRouter beside the nineteenth `hakureico`,
`~/temp/agent/yuki7-20260909`.
The log in order:
the roster paired the definitions by content
(`7 usable voices of 8 heard`,
the one refusal a definition paired with a body block),
`FOOTNOTES entry=yuki418330012 relabelled [^2]->[^1], [^1]->[^2] off the definitions the roster paired`,
`FOOTNOTES entry=yuki418330012 definitions moved into the original's order: [^1], [^2]`,
the re-preparation paired `7 of 8 original and 11 of 12 translation blocks`,
the credits line was admitted as carried (`slice 8: coverage=carried, missingDestinations=0`),
and no `INTERRUPTED` line:
the guard read the wrapped credits as the full votes' evidence and passed.
The three checks passed:
the front matter is the archive's byte for byte,
1 `json false start` read kept with no `schema-mismatch` beside it,
no error finish.
`verify-published` matched 1 of 1 at length (`wordings=8 silent=1 chars=5225=expected missing=0`);
the silent slice is slice 2,
which both lanes left as the archive has it (`archive-stands`).
Destinations 0,
quorum held (`quorum-not-met` 0,
`one-reader-only` 0,
`corroborated` 22),
73 voices lost at the straggler window and no `HTTP 429`.
Meters at the tally:
OpenRouter 114.14 USD,
Bedrock 194.66 USD.

THE NINETEENTH CLASS IS FIXED ON THE PAGE.
`Zhouzhou[^2] took her in and they lived together;` (line 103) stands above
`[^2]: Yuki’s godmother? They were like mother and daughter. Zhouzhou cared for Yuki for a long time and averted
many of Yuki’s suicide crises.` (line 136),
and `Yuki went to stay with Zhenli[^1] to take a breather.` (line 109) above
`[^1]: She was younger than Yuki, but she treated Yuki like an older sister would. She liked Yuki a lot, bought
her gifts, and cooked her meals.` (line 134):
each marker at its own note,
the original's labels,
the original's definition order,
four markers on each side.
The credits ship as `Contributors for this entry:\nZhenli,\nSansan,\nSuona`,
the semantic wrap's shape,
which the twentieth class's two fixes let through.

THE TWO SLANG REGRESSIONS STAND,
now on a second page:
`自切` ships as `After she began cutting herself, their attitude improved significantly` (line 52) where the archive
had `After she attempted self-surgery`,
and `超天酱` ships as `Chōten-chan` (line 17) where the archive named *Needy Streamer Overload*.
The bench does not know the community's words;
the glossary question stands with the owner.

## The nineteenth hakureico page is read and the twenty-first class takes the sibling note with the orphan, 2026-09-09, 03:25 UTC

`TALLY hakureico status=SETTLED` at 03:03 UTC on `d6db46519`,
88 minutes on Bedrock and OpenRouter,
`~/temp/agent/hakureico19-20260909`,
pid 546904 exited.
The seal held:
`ARCHIVE ORIGINAL entry=hakureico span=0 [3966, 4561)` in the log,
artifact `artifactSchemaVersion: 12` with `archiveOriginalSpans` carrying the one span and its note,
the letter on the page byte for byte the archive's and ending the page,
the CJK counts equal to the archive's own (three Chinese notes in HTML comments,
none of them the pipeline's).
The three checks passed:
the front matter the archive's,
1 `json false start` read kept with no `schema-mismatch` beside it,
no error finish.
`verify-published` matched 1 of 1 at length (`wordings=14 silent=1`),
37 voices lost at the straggler window,
no `HTTP 429`.
Meters at the tally:
Bedrock 194.64 USD,
OpenRouter 113.39 USD.

THE PAGE CARRIES A REFERENCE WITH NO NOTE.
Line 80 reads `she used “Mayday”[^1]` and no `[^1]` definition stands anywhere on the page.
The trace,
in the artifact and the translate lane's cache:
the source's two definitions
(`[^1]: 国际通用的无线电遇难求救讯号…` and `[^2]: 即 Google App Engine`)
sit behind the sealed letter as insertion slice 14 (`coverage=absent`);
the translate lane rendered both as one insertion;
the sealed letter is the archive's,
which has `HOSTED__WITH__GAE____` with no `[^2]` marker,
so on the lane's assembly `[^2]` was an orphan definition;
`guardFootnoteAssembly` blamed slice 14 for it and withdrew the whole insertion,
the `[^1]` note with the orphan
(`replacement-withdrawn reason assembly-integrity`);
the consolidation candidate for the body slice carried `[^1]` and shipped;
and nothing read the composed page as one document,
since the guard runs only inside `repair-assemble.ts` and `translate-assemble.ts`.
The archive itself carries no footnote marker at all,
so a page-level reading against the archive would have refused this page.
The twenty-first class,
in two faces:
a guard that withdraws more than the defect,
and a page nobody parses whole.

THE FIX (`aedee7414`).
`assembly-orphan-trim.ts`:
before the guard blames a slice,
`trimOrphanDefinitions` cuts an orphan definition's own block out of every replacement that is nothing but
definition blocks (`assembly-footnote-trimmed orphan-definition gfm 2 (slice 14)`),
leaves a replacement that carries prose beside its definitions for the whole withdrawal as before
(cutting a block out of judged prose ships a text nobody judged),
and withdraws only a trim that lands on the slice's own incumbent
(`assembly-footnote-trimmed-to-incumbent`);
the guard's round bound widens by the count of definition blocks,
since a trim round withdraws nothing.
`corpus-run/page-footnote-integrity.ts`:
`assertPageFootnotesIntact` splices the shippable replacements the way `publish-fixed` will,
reads `introducedFootnoteFindings` against the archive,
and stops the entry as `TranslationRepairInterruptedError` with the new reason `page-footnote-integrity`,
naming each defect (`page-footnote-unresolved-reference gfm 1`) on `INTERRUPTED` lines;
`corpus-run/pass-page-guards.ts` runs it beside the carried guard from `pass-entry.ts`.
The assembly suite's trim case failed first on the old build
(`expected [ 1 ] to deeply equal []`,
the whole insertion withdrawn);
two new suites cover the trim and the page guard;
full suite 979 `PASS` and 0 `FAIL`.

THE TWENTIETH `hakureico` LAUNCHED at 03:21 UTC on `aedee7414`,
`~/temp/agent/hakureico20-20260909`,
pid 578547,
Bedrock and OpenRouter.
What its page must show:
`[^1]` above its note,
no `[^2]` anywhere,
`assembly-footnote-trimmed` in the translate lane's findings,
no `INTERRUPTED` line,
and the seal as the nineteenth had it.

## Two shapes-first entries launched beside it, 2026-09-09, 03:24 UTC

The shape census re-run over the 92 sources against the shapes read pages have since carried
(footnotes,
bold,
the container,
the inline break,
the math pair,
the double-quoted path;
scratch `unmet-shapes.mjs`)
ranks the sources by shapes still on no read page:
`XingZ60` seven (16,650 characters:
a rule,
a deep heading,
a list and four of the seven components),
`gqt` three (3,802 characters:
`### 时间线`,
a bold-led unordered list,
an italic paragraph),
`Mio` two (1,675 characters,
with photos:
a rule in the body and a bare URL),
`noname` two,
then one each.
`gqt` launched at 03:24 UTC (`~/temp/agent/gqt1-20260909`,
pid 581368) and `Mio` beside it (`~/temp/agent/mio1-20260909`,
pid 581484),
both on `aedee7414`,
Bedrock and OpenRouter,
three passes sharing the two providers as on the Arita night.
`XingZ60` waits for a quieter bench,
since a 16,650-character source beside two passes is the seven-hour shape the owner refused.
A decision about run order,
not design,
open to veto.

## The twenty-first class's second face where the ledger says what was decided and the document carries the trim, 2026-09-09, 04:45 UTC

`TALLY hakureico status=ERROR ms=4081159 aborted=false error=writing the ledger's 13 shipped rows over the archive
produces a different document than the lane returned, so the rows do not say what the document carries`
at 04:28 UTC on `aedee7414`,
68 minutes,
`~/temp/agent/hakureico20-20260909`.
No `withdrew` warning in the log:
the guard trimmed rather than withdrew,
which is the first face working.
What refused was the delivery invariant behind it (`assertDeliveryAgreesWithDocument`):
the ledger's shipped row for slice 14 carried the judges' accepted text,
two definitions,
the document carried one,
and splicing the rows over the archive no longer wrote the document.
The record model keeps two facts apart on purpose,
what the slice CHOSE (`outcome.acceptedText`) and what the document CARRIES (`shippedText`),
and until now the two could differ only by a withdrawal,
where the document carries the incumbent.
A trim is a third relation,
and nothing carried it from the guard to the ledger.

THE FIX (`9abcbee50`).
`guardFootnoteAssembly` reports `trimmed`,
the surviving replacements whose text differs from what they arrived with;
both lane results carry them as `trimmedReplacements`
(`TranslateDocumentResult`,
`RepairTranslationResult`);
`laneDelivery` hands them to `buildSliceDelivery`,
which reads a shipped row's `shippedText` from them before it reads the decision
and refuses a trim naming a slice the document does not ship (`trim-names-unshipped`);
`assertDeliveryCoherent` accepts a shipped row whose text is its decision with definition blocks cut and nothing
else changed (`isDefinitionTrim`,
which recomputes the cut and demands equality),
so the artifact's row shows both texts and the relation between them is checked at write and at read.
The comparison rows already read `shippedText`,
so the contest and the page carry the trimmed text.
Both ledger cases failed first on the old build;
lint and types clean;
full suite 981 `PASS` and 0 `FAIL`.

RELAUNCHED at 04:39 UTC on `9abcbee50`,
three passes:
the second `gqt` (`~/temp/agent/gqt2-20260909`,
pid 593252),
the second `Mio` (`~/temp/agent/mio2-20260909`,
pid 593381)
and the twenty-first `hakureico` (`~/temp/agent/hakureico21-20260909`,
pid 595265),
the first `gqt` and `Mio` killed under the kill-and-relaunch rule after 75 minutes on the superseded build.
What the `hakureico` page must show is unchanged,
plus the artifact's slice 14 row with `shippedText` one definition shorter than `acceptedText`.

## The twenty-first class's third face where two notes one line apart are one block to the trim, 2026-09-09, 06:25 UTC

`TALLY hakureico status=INCOMPLETE ms=5593268 aborted=false error=translation repair interrupted:
page-footnote-integrity` at 06:13 UTC on `9abcbee50`,
93 minutes,
`~/temp/agent/hakureico21-20260909`,
with `INTERRUPTED hakureico: page-footnote-unresolved-reference gfm 1` above it
and `withdrew 2 replacements at assembly; the findings say why` from the translate lane,
the findings nowhere in the log.
The page guard did what it is for:
the page would have shipped `“Mayday”[^1]` with no note again,
and the pass stopped instead.

WHY THE TRIM DID NOT FIRE.
The translate cache holds slice 14's rendering:
`[^1]: The internationally recognized radio distress signal. … immediate rescue.\n[^2]: i.e., Google App Engine`,
the two notes one line apart,
no blank line.
The trim split blocks at blank lines,
saw one block labelled `1`,
found no orphan block to cut,
and left the replacement to the guard,
which withdrew slice 14 for the orphan `[^2]` and then slice 7 for the reference it left unresolved.
GFM ends a definition where the next `[^id]:` line begins,
and the document parser reads it so (the guard's own finding named `[^2]` as a definition);
the trim read a shape the parser does not.

THE FIX (`379122379`).
`assembly-orphan-trim.ts` reads blocks by lines:
a block ends at a blank line or where a definition line begins,
a definition's indented continuation stays in its block,
and every gap between blocks is kept as written,
a kept block after a cut run taking the gap that stood before the run;
`cutDefinitionBlocks` and `isDefinitionTrim` share the reading,
so the ledger's coherence rule reads the same shape the trim cuts.
Both assemblers now log every guard finding after the withdraw warning,
since an entry stopped before its artifact leaves the log as its only trace.
The guard case with the notes one line apart failed first
(`expected [ 1 ] to deeply equal []`);
lint and types clean;
full suite 981 `PASS` and 0 `FAIL` on `e945b1b5f`,
where one withdrawal case now expects the finding line after the count.

RELAUNCHED at 06:20 UTC on `379122379`:
the twenty-second `hakureico` (`~/temp/agent/hakureico22-20260909`,
pid 615099),
the third `gqt` (`~/temp/agent/gqt3-20260909`,
pid 615846),
the third `Mio` (`~/temp/agent/mio3-20260909`,
pid 616606);
the second `gqt` and `Mio` killed under the rule after 100 minutes on `9abcbee50`,
neither at its tally.

## The twenty-first class's fourth face where the consolidation puts the orphan back and the page is a document nobody guarded, 2026-09-09, 08:05 UTC

`TALLY hakureico status=INCOMPLETE ms=5144300 aborted=false error=translation repair interrupted:
page-footnote-integrity` at 07:46 UTC on `379122379`,
86 minutes,
`~/temp/agent/hakureico22-20260909`.
The log now says what happened:
`assembly-footnote-trimmed orphan-definition gfm 2 (slice 14)` from the translate lane,
the third face working,
and then `INTERRUPTED hakureico: page-footnote-orphan-definition gfm 2` from the page guard.
The consolidation cache for slice 14 holds the consolidation's own rendering of the two notes from the source,
`[^1]: The internationally used radio distress signal. …\n\n[^2]: That is, Google App Engine.`,
the orphan back;
the consolidation replaces what the contest left at a slice,
and the page it composes with the polish and the per-slice contest is a document no guard assembled.
The lanes guard their own assemblies;
the page had only the refusal.

THE FIX (`3620004db`,
artifact generation thirteen).
The pass composes the artifact,
runs `guardFootnoteAssembly` over the page that artifact would ship
(`page-assembly-guard.ts`,
over `shippableReplacements` less any write that repeats the archive's own wording),
and records what the guard trimmed,
withdrew and found as the artifact's `pageAssembly` section
(`artifact-two-lane-page-assembly.ts`;
`pass-page-assembly.ts` composes twice,
both compositions pure,
and logs the findings).
`wouldShipTextFor` applies that section FIRST,
ahead of the polish,
the consolidation and the contest,
so every reader composes the page the guard settled:
a trimmed slice ships the guard's text as `page-assembly`,
a withdrawn slice stands as the archive
(`page-assembly-withdrew-and-archive-silent` where the archive has nothing).
A generation-twelve artifact reads with an empty section.
The page footnote guard stays as the backstop after the section is applied.
The would-ship case failed first on the old build;
the parser,
the guard and eleven suites on the path clean;
the schema guard's fixtures and wording moved to thirteen;
full suite 984 `PASS` and 0 `FAIL` on `dc3946757`.

RELAUNCHED at 08:01 UTC on `3620004db`:
the twenty-third `hakureico` (`~/temp/agent/hakureico23-20260909`,
pid 650518),
the fourth `gqt` (`~/temp/agent/gqt4-20260909`,
pid 650645),
the fourth `Mio` (`~/temp/agent/mio4-20260909`,
pid 650761);
the third `gqt` and `Mio` killed under the rule after 100 minutes on `379122379`,
neither at its tally.
What the `hakureico` page must show:
`[^1]` above its note,
no `[^2]`,
`page assembly: assembly-footnote-trimmed orphan-definition gfm 2 (slice 14)` in the log,
the artifact's `pageAssembly.trimmed` naming slice 14,
no `INTERRUPTED` line.

## The twenty-third hakureico page settles under the page assembly and closes the twenty-first class, 2026-09-09, 09:40 UTC

`TALLY hakureico status=SETTLED slices=15 repairStatus=repaired repairIssues=73 repairAccepted=55 repairResolved=52
repairFindings=192 repairChanged=11 translateStatus=complete translateChanged=15 documentsDiffer=15 pageChanged=13
pageSilent=0 alignmentFindings=6 selection=contested ms=5384276`
at 09:31 UTC on `3620004db`,
89.7 minutes on Bedrock and OpenRouter (Synthetic dry this run,
Kimi-K3 withheld),
`~/temp/agent/hakureico23-20260909`.
The log in order:
the seal (`ARCHIVE ORIGINAL entry=hakureico span=0 [3966, 4561)`),
slice 7 left out of the relabel reading as before,
`assembly-footnote-trimmed orphan-definition gfm 2 (slice 14)` from the translate lane,
`page assembly: assembly-footnote-trimmed orphan-definition gfm 2 (slice 14)` and
`page assembly trimmed 1 slices and withdrew 0` from the page-level guard,
and no `INTERRUPTED` line.
The three checks passed:
the front matter the archive's byte for byte,
7 `json false start` reads kept with no `schema-mismatch` beside them,
no error finish.
`verify-published` matched 1 of 1 at length (`wordings=15 silent=0 chars=4721/expected 4718+separators`).
The artifact is generation 13,
carries the sealed span,
`pageAssembly.trimmed` naming slice 14 with the one-note text,
and its slice 14 translate row with `shippedText` one definition shorter than `acceptedText`.
78 voices lost at the straggler window,
10 `HTTP 429` retries,
three passes sharing the two providers.

THE TWENTY-FIRST CLASS IS CLOSED ON THE PAGE.
`she used “Mayday”[^1] to signal that she was approaching her limit.` (line 77) stands above
`[^1]: The internationally recognized radio distress signal. When used on an aircraft, it usually indicates an
extreme emergency requiring immediate rescue.` (line 189),
the last line of the page,
after the sealed letter,
where the source puts its notes;
no `[^2]` anywhere,
which is right,
since the sealed letter carries no marker for it.
The seal holds:
the span is on the page byte for byte,
the CJK count equals the archive's own (127,
the three Chinese HTML-comment notes),
no straight quote,
`corroborated` readings and no `quorum-not-met`.

TWO NITS,
no class:
`Though many of those journeys were ones she made alone...` (line 37) renders the source's `……` as three dots
on an archive that carries no ellipsis of either convention,
so the restoration had no convention to follow;
and the semantic wrap breaks `her *osu!*` from `was still online.` (lines 48 to 49),
reading the exclamation mark inside the emphasis as a sentence end.
Both are wording the site renders as one paragraph.

THE READING:
four faces of one class in one day,
each found by the page guard the first face added,
and the fourth was the cause of the other three:
the page was never one document to any guard.
It is now,
and the guard's outcome is in the artifact,
so a lane's trim,
a consolidation's rewrite and a contest's per-slice choice meet one reading before the tally.

THE NEXT ENTRY,
by the census (scratch `unmet-shapes.mjs`):
`hulicaijia` (6,651 characters,
309 lines,
photos;
`<ruby>コネクト<rt>Connect</rt></ruby>` inline beside a `[^1]` reference,
`<Sakura count="50" />` at the end),
two shapes on no read page and a footnote,
never run,
launched at 09:34 UTC on `3620004db` beside the fourth `gqt` and `Mio`
(`~/temp/agent/hulicaijia1-20260909`,
pid 668600).

## The fourth gqt page is read and the twenty-second class declines an English original, 2026-09-09, 09:55 UTC

`TALLY gqt status=SETTLED slices=24 repairStatus=repaired repairIssues=73 repairAccepted=42 repairResolved=42
repairFindings=199 repairChanged=16 translateStatus=complete translateChanged=17 documentsDiffer=18 pageChanged=17
pageSilent=0 alignmentFindings=5 selection=contested ms=6270496`
at 09:46 UTC on `3620004db`,
104.5 minutes beside two other passes,
`~/temp/agent/gqt4-20260909`.
The three checks passed (front matter the archive's,
7 `json false start` reads kept,
no error finish),
`verify-published` matched 1 of 1 at length (`wordings=24 silent=0`),
the artifact is generation 13 with an empty `pageAssembly`,
28 `HTTP 429` retries and 95 voices lost across three passes.
THE THREE SHAPES RENDERED:
`### Timeline` at level 3 where the archive has it;
the bold-led list items one per line as the archive has them,
the two the lanes rewrote (`20:05`,
`21:10`) wrapped after the bold lead's colon onto an indented continuation line,
one list item still,
which is the semantic wrap reading the colon as a clause end (a nit,
the same family as the `*osu!*` break);
the italic paragraph after `16:06` carried across four wrapped lines inside one pair of asterisks,
which CommonMark reads as one emphasis.
The source's footnote,
which the archive dropped,
ships as `my ideal blunt rotation[^1]` in the quote and its definition at the end,
with the page guard silent.

THE PAGE SHOULD NOT EXIST.
Line 8 of the page,
of the archive and of the source alike is `<!-- (Original Language: Engish) -->`:
Ara wrote in English,
the Chinese page is the translation,
and the owner's rule of 2026-09-08 declines such an entry.
`readNote` knew the two Chinese wordings the 2026-09-08 census found
(`原文即英文`,
`不要动本篇`,
and the span marks) and read this English one as advisory,
so four `gqt` launches repaired Ara's page from its Chinese back-translation.
The twenty-second class:
the archive-original recognizer reads only Chinese.
A census over the 92 sources finds this the only note of its wording.

THE FIX (`7bcea2dc4`).
`ENGLISH_WHOLE_PAGE_MARKS` (`original language: english`,
and the corpus's own `original language: engish`) are matched on the lowercased note and read as whole-page;
the recognizer's case failed first;
lint and types clean;
full suite 984 `PASS` and 0 `FAIL`.
The fifth `gqt` on `7bcea2dc4` declined in 146 ms:
`ARCHIVE ORIGINAL entry=gqt: the archive's note says the whole page is the author's own English
((Original Language: Engish)), so the pipeline declines to repair it and the archive stands`,
`TALLY gqt status=DECLINED reason=archive-original ms=146`.
The fourth `Mio` and the first `hulicaijia` were killed under the rule and relaunched at 09:50 UTC
(`~/temp/agent/mio5-20260909`,
pid 676894;
`~/temp/agent/hulicaijia2-20260909`,
pid 677559),
and `noname` launched beside them (`~/temp/agent/noname1-20260909`,
pid 678195;
47 lines,
`### 武汉江宸天街杀人事件` and bare URLs,
never run),
since `gqt`'s decline leaves the level-3 heading,
the unordered list and the emphasis on no read page:
`noname` carries the heading,
`shi_Yumiaoya` the emphasis (with `Banner`),
`XingZ60` the list (with a rule and three components).

## The first noname page is read and the twenty-third class decides a contested target by votes, 2026-09-09, 12:10 UTC

`TALLY noname status=SETTLED slices=13 repairStatus=repaired repairIssues=72 repairAccepted=56 repairResolved=56
repairFindings=180 repairChanged=12 translateStatus=complete translateChanged=10 documentsDiffer=12 pageChanged=11
pageSilent=1 alignmentFindings=9 selection=contested ms=6922176`
at 11:45 UTC on `7bcea2dc4`,
115.4 minutes beside two other passes,
`~/temp/agent/noname1-20260909`,
Bedrock and OpenRouter wet (0.36 USD over 535 calls and 4.68 USD over 820 calls).
The three checks passed (front matter the archive's,
6 `json false start` reads kept,
no error finish),
`verify-published` matched 1 of 1 at length (`wordings=12 silent=1 chars=4321`),
the artifact is generation 13 with an empty `pageAssembly`,
no `HTTP 429`,
125 voices lost,
most of them critics and panellists abandoned 120 s after quorum with millions of raw characters delivered
and no reply closed,
which the grace handles and which is noted here as a shape to watch.
No forbidden mark:
the archive carried one fullwidth colon and one CJK run (`知乎` as a link label),
and the page carries neither,
the colon `:` and the label `Zhihu`.
THE SHAPES:
three level-3 headings at the source's positions,
`### Wuhan Jiangchen Paradise Walk Murder Incident`,
`### The Transgender Deceased of Coolapk`,
`### Transgender companions who departed between November 2023 and May 2024`,
each a rendering of the source's heading where the archive's heading had added a date and a description the
source heading does not carry
("Feminine-presenting person stripped and murdered in men's bathroom":
the source says stabbed and never stripped,
and the page says stabbed);
the three links verbatim,
one per line after `**More information**:`,
which CommonMark joins with soft breaks;
the `PhotoScroll` line equal to the source's byte for byte,
where the archive had `'${path} /photos/kuan4.webp'` with a space inside the path,
a broken picture the page repairs;
the archive's translator comment kept.
Nits:
"she was dressed in feminine clothing" where the source gives no pronoun (着女性装扮),
and "the number of departed individuals we can see" for 单我们能看见的逝者之数量即已,
which drops the "alone" the source stresses.

THE BARE URL WAS NEVER THERE.
The census regex read the `https://matters.news/...` inside a `web.archive.org` link target as bare,
on `noname` and on `Mio` alike.
A census that strips link targets and autolinks first finds a bare URL in a body on three entries:
`yingying`'s `[^1]: https://leohearts.com/archives/farewell-yingying.html`,
a footnote definition that is nothing but the URL,
and the `icon: "https://one-among.us/favicon-large.png"` prop of the `Banner` on `shi_Yumiaoya` and `Aniloviraw`.
The shape stays on no read page;
`shi_Yumiaoya` carries it with the emphasis and the `Banner`,
and `yingying` carries the prose form.
Scratch `unmet-shapes.mjs` is corrected.

THE HEADING THE PAGE LACKS.
The source opens `## 简介` above its first paragraph;
the archive has the paragraph and no heading,
and so does the page.
84 of the 86 sources with a level-2 heading have it in the archive
(37 `## Description`,
21 `## Introduction`,
2 `## Profile`,
1 `## Thoughts of Her`,
two with a trailing space),
and `noname` and `Hangmster` are the two without.
Section 0 of the pairing:
original blocks [heading,
paragraph],
translation block [paragraph];
`paired 1 of 2 original and 1 of 1 translation blocks across 1 relations, from 8 usable voices of 9 heard`,
the relation `{source: 0, target: 0}`,
the heading to the paragraph,
and the finding `block-pairing non-monotone (1,0 runs back behind 0)`.
`agreePairs` walks the sources in order:
source 0's agreed pair `(0,0)` was kept first,
and source 1's agreed pair `(1,0)` named the same target with no voice naming the two together as a merge,
so it was dropped as non-monotone whatever its votes
(the pairing replies are not among the stored payloads,
so the split is not on record;
the finding proves both pairs reached two voices).
Everything after followed from that:
the source paragraph became slice 1,
an insertion the admission found `coverage=carried` by the archive paragraph in slice 0;
slice 0 set the heading alone against the paragraph,
the repair lane's select could seat no winner (weight 1.5 across 6 ballots,
4 abstentions),
the contest split three ways and settled neither,
the consolidation's nine candidates were all `invalid` under the block floor but one
(two blocks against a one-block page,
or a heading where the page has a paragraph),
the gate kept standing 6 to 2,
and `final-selection-unendorsed (slice 0)` went into the artifact as evidence.
The twenty-third class:
a target two sources claim without a corroborated merge is decided by source order,
not by votes.

THE FIX (`efc9a4f3c`).
`pair-contested-target.ts` decides such a claim by votes,
the mirror of `bestVoted` for one source named against two targets:
the better-voted claim keeps the target and the earlier pair is taken back,
so its source is left unpaired;
a tie keeps neither;
fewer votes lose.
The finding reads `contested target (target 0: source 1 outvotes source 0, 3 to 2)`,
`... loses to ...`,
or `... sources 0 and 1 tie at 2 votes`.
A run-back (a lower target) is still `non-monotone`,
and a corroborated merge is still kept.
Three cases failed first on `7bcea2dc4` (3 `FAIL` in the agreement suite);
the block,
section and steps suites pass;
oxlint 0 and 0;
types clean;
full suite 984 `PASS` and 0 `FAIL`.
What follows for `noname` if the votes favour the paragraph pair:
the heading is a source-only slice,
the shortfall admits it
(the archive is 4522 code points against 5300 expected from 2000 source code points at 2.65,
short by 778,
and the heading wants 13),
and the block floor accepts a heading over the page's paragraph where the page's one block is a kind the
original has.
If the votes tie,
both blocks are unpaired and the section slices as [heading,
paragraph] against [paragraph],
which the floor also accepts.
Measured by the second pass,
not predicted.
The fifth `Mio` and the second `hulicaijia` were killed at 11:58 UTC under the rule and relaunched at 11:59 UTC
beside the second `noname`,
all on `efc9a4f3c`:
`~/temp/agent/noname2-20260909` (pid 711861),
`~/temp/agent/mio6-20260909` (pid 711977),
`~/temp/agent/hulicaijia3-20260909` (pid 712107),
Bedrock and OpenRouter wet.
The second `noname`'s section 0 round read
`paired 1 of 2 original and 1 of 1 translation blocks across 1 relations, from 9 usable voices of 9 heard`
at 11:59:36 UTC;
which pair survived is read from the artifact's findings when it tallies.
At 12:03:39 UTC its lanes answered:
`slice 0: coverage=absent, missingDestinations=0`,
`SLICE-START lane=repair chunk=0 sourceChars=5` and
`chunk 0: no translation to repair; the translate lane owns this passage`,
then `chunk=1 sourceChars=55` under repair:
the heading is a source-only slice the admission admitted,
and the paragraph pair survived the contest.
The page says whether the heading ships.

OPENROUTER DRY AT 12:22 UTC.
All three passes read `HTTP 402` from OpenRouter within two seconds of each other
(`kimi-k3`,
`glm-5.3`,
`deepseek-v4-pro-0813`,
`minimax-m3`),
the meter reading `openrouterUsd=1.51`,
and `markRefused` reads it dry until the meter moves,
as the fourteenth class set it.
The balance was 172.94 USD at 21:22 UTC on 2026-09-08 and 1.51 at 12:22 UTC on 2026-09-09:
the hakureico,
yuki,
Arita,
gqt,
Mio,
hulicaijia and noname launches of the night spent about 171 USD,
the killed passes included.
The three passes run on Bedrock alone (187.91 USD):
the second `noname` warned
`JUDGE SEATS phase=translate lane short of quorum: translators 2 of 8 reachable, quorum 4; select 3 of 8 reachable, quorum 4`
and runs on what is reachable,
which the owner named normal operation.
The pages are read knowing which seats spoke;
a two-translator translate lane is thinner evidence than the seven-translator one the first `noname` had.
The owner is notified;
whether OpenRouter is topped up is theirs.
