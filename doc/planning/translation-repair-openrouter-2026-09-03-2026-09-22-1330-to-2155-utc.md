# OpenRouter and the corpus passes: 2026-09-22, 13:30 to 21:55 UTC

What was launched,
read and decided on 2026-09-22 13:30 to 21:55 UTC.
Entries:
XingZ623,
da02857b1,
XingZ624,
yingying4,
yingying3,
zheermao4,
CuspariaKLSY7,
CuspariaKLSY6,
hulicaijia13,
hulicaijia12,
hulicaijia11,
hulicaijia10,
fa014fc55.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## XingZ623 read, 2026-09-22, 21:55 UTC: INCOMPLETE at publish after 3h01m on the bare TextRing slice, nothing standing behind a declined contest at an absent archive (class eighty-seven); fixed in da02857b1, XingZ624 launched

`XingZ623` (frozen `40679244d`,
pid 2184330) ended INCOMPLETE at 21:32 UTC,
3h01m after its 18:31 UTC launch (TALLY `ms=10862998`),
one attempt,
8,163 seats,
257 retry rounds,
7,378 calls,
69 abandoned streams (0.32 USD),
10.41 USD by the meters (Bedrock 102.73 to 98.04,
OpenRouter 151.09 to 145.37).
Synthetic dry throughout,
glm-5.3 dark (asked 585,
threw 585).
No page and no artifact were written;
this reading comes from the run log and the slice cache.

The stop:
`TALLY ... error=slice 89 did not meet absolute naturalness floor`,
thrown by `assertFinalNaturalnessComplete` at publish,
after the page assembly had already run (its contributor-name-restored lines are timestamped 21:32:46 UTC,
the TALLY 21:32:46 UTC).
Slice 89 this run is the bare `<TextRing text="⊕⊕⊕⊕" fontSize="1.25rem"/>` line alone
(one mdxJsxFlowElement;
`XingZ622` had sliced it together with the `<h3>` heading above it),
an admitted tail slice with no archive text.
The repair lane had nothing to repair (`repair-not-applicable chunk 89`);
the translate lane wrote the tag verbatim and its judges accepted it at weight 4.5 of 5
("a JSX/HTML component line rather than source-language prose,
so preserving it unchanged is faithful");
the lane contest then declined both lanes 4 of 4
("both candidates are empty placeholders",
"non-linguistic markup ... translation or repair impossible",
"the candidates are identical (one is blank,
the other reproduces the tag verbatim),
so neither is more faithful").
`standingTextFor` turns a declined contest into the incumbent,
which is the empty string at an absent archive;
`readStandingVerdict` refused it (0 blocks against 1) and `laneTextsForSlate` offered the translate text (class forty);
but `buyConsolidationAttempt` and `settleConsolidation` both exit on `standingText === ''` before the offer is read
("no standing text to consolidate against,
so no slate is bought";
terminal `no-standing-text`),
so the one valid rendering was never put to a judge,
the slice shipped nothing,
and the final naturalness check refused the record.
The 3h01m and 10.41 USD bought no page.

Class eighty-seven,
fixed in da02857b1:
the empty-standing exits in `consolidate-slice-buy.ts` and `consolidate-settle.ts` apply only when no lane text is on offer;
with one,
the producers are still not asked (they write from the standing,
and there is none)
and the settlement runs the class forty path over the lane texts alone:
the empty standing withheld as an ineligible one is,
the slate judged through `judgeSlateWithRetry`,
the winner gated and polished,
a decline failing the slice at once under the owner's rule of 2026-09-04.
The buy logs "no standing text to consolidate against;
the lane texts alone go to the slate judges" on that path.
Guard red first at daf73f3dc (`consolidate-settle.unit.test.ts`:
an empty standing with one lane text is judged and gated,
and stays `no-standing-text` without one),
lint 0 and 0,
types clean,
full suite green (`suite-class87.log`).
Open on the same path:
an empty standing with no valid lane text still ends `no-standing-text` and fails the entry at publish rather than at once;
no run has reached it (a declined contest at an absent archive always has a lane text,
and the only way both fail the rule is both being refused).

Class eighty-three live in the lane texts,
though no page carried them:
the Part Ten heading rendered "### No. 10:
Jinxin (Brocade Heart)" and its signature "——Jinxin,
February 10,
2025" (both lanes' cached texts),
the song credit "— Yuli (Rain Raccoon) [Paranoia]",
the Bird in the Cage attribution "by Jiecheng Tianzou Official (...)";
the assembly's restore lines this run were HiYku (heading and signature,
slice 48),
Ann (slice 49),
Baimao Suki (slice 79) and the memorial ceremony line (slice 88).
Whether the page carries them one way each is `XingZ624`'s read.

Before the stop,
the log carried
181 abstention lines,
eight unendorsed standings (5,
11,
15,
38,
52,
54,
63,
82),
two gate-kept (11,
52),
five slate-declined (15,
38,
54,
63,
82),
no repeat,
run-off,
archive-revision-refused or error-finish line.
Timing as `XingZ622`:
admission 18:50 UTC,
repair lane 18:50 to 19:22 UTC,
translate to 20:14 UTC,
contest to 20:36 UTC,
consolidation 20:36 to 21:32 UTC.

`XingZ624` (frozen `da02857b1`,
pid 2237613,
overlap 8)
launched 21:43 UTC on Bedrock and OpenRouter
(meters 98.03 USD Bedrock,
145.35 OpenRouter at launch),
waiter bmu8s7wec.
Read for class eighty-seven live (slice 89 or wherever the bare TextRing slices this run:
the "lane texts alone go to the slate judges" line,
the tag on the page,
no `no-standing-text` stop)
and class eighty-three live on the page (Part Ten heading and signature "Jinxin",
the song credits "Yuli" one way,
"by Jiecheng Tianzou Official",
the parenthesised gloss at first appearance),
beside every check of `XingZ622`.

## yingying4 read, 2026-09-22, 18:40 UTC: SETTLED in 23 min, class eighty-six live, the blog title names "Yingying"; XingZ623 launched for class eighty-three

`yingying4` (frozen `40679244d`,
pid 2180429) SETTLED at 18:30 UTC,
23 minutes after its 18:07 UTC launch (TALLY `ms=1384777`),
one attempt,
326 seats,
15 retry rounds,
279 calls,
2 abandoned streams (0.01 USD),
0.23 USD by the meters (Bedrock 102.81 to 102.73,
OpenRouter 151.24 to 151.09).
Synthetic dry,
Qwen3.8-27B and glm-5.3 dark.

Class eighty-six live:
the second definition reads "[^2]:
[Farewell. I miss you, Yingying.](https://leohearts.com/archives/farewell-yingying.html)".
Both lanes wrote "Yingying" into the title this time (slice 3,
repair and translate agreed,
no contest),
and a translate slate ballot cites the glossary line by its words:
"the names glossary states that 樱樱 is declared 'Yingying' and that the declared form is used inside the title,
so Candidate 2 is the faithful rendering and Candidate 1 alters the name form".

The page against `yingying3`:
1 of 1 heading,
4 of 4 markers,
2 of 2 definitions,
front matter equal,
the References line kept,
no TA,
no error finish,
no double blank line,
no unendorsed,
gate-kept or slate-declined standing,
3 abstention lines (gpt-oss declining every candidate for changing the archive's wording),
no repeat or run-off finding;
the farewell in the second person as the original.
Wording moved where the bench chose:
"undergone" for "completed",
"played with digital gadgets and chemistry",
"burst out laughing",
the Bilibili sentence reordered into the past perfect,
"motorcycle riding" again (as `yingying2`;
`yingying3` had "bicycle";
the source's 骑车 settles neither),
the accident sentence joined into one,
"what you wanted to do" as the archive's tense.
The `[^2]` marker stands on its own line after "next world!":
the archive writes "world!
[^2]" with a space,
so the rendered page shows the same space before the superscript;
`yingying3` had attached it.

`XingZ623` (frozen `40679244d`,
pid 2184330,
overlap 8)
launched 18:31 UTC on Bedrock and OpenRouter
(meters 102.73 USD Bedrock,
151.09 OpenRouter at launch),
waiter bpl8gb3gq.
Read for class eighty-three live
(Part Ten heading and signature "Jinxin",
the song credits "Yuli" one way,
"by Jiecheng Tianzou Official" on the Caged Bird attribution,
the literal gloss in parentheses at each handle's first appearance)
beside every check of `XingZ622`
(fourteen headings,
both links,
nine definitions,
3 TextRing,
eleven details blocks,
the closing poem's attribution inside the quote,
"They once said" on the tombstone paragraph,
front matter equal,
no TA,
no error finish).

## yingying3 read, 2026-09-22, 18:15 UTC: SETTLED in 14 min, but the blog title names the person "Sakura" against the declared "Yingying" (class eighty-six); fixed in 40679244d, yingying4 launched

`yingying3` (frozen `7ad1b8ec7`,
pid 2155878) SETTLED at 17:56 UTC,
14 minutes after its 17:42 UTC launch (TALLY `ms=854118`),
one attempt,
268 seats,
8 retry rounds,
234 calls,
2 abandoned streams (0.00 USD),
0.16 USD by the meters (Bedrock 102.89 to 102.81,
OpenRouter 151.32 to 151.24).
Synthetic dry,
Qwen3.8-27B and glm-5.3 dark.

The page against `yingying2`:
1 of 1 heading,
4 of 4 markers,
2 of 2 definitions,
front matter equal,
the References line kept,
no TA,
no error finish,
no double blank line,
no unendorsed,
gate-kept or slate-declined standing,
2 abstention lines,
no repeat or run-off finding;
the farewell in the second person as the original ("May you still have the same cheerfulness") where `yingying2` had "May she";
"riding her bicycle" as the archive where `yingying2` inferred "motorcycle" from the next paragraph's 摩托骑行 (the sentence itself says 骑车).

But the second definition reads "[^2]:
[Farewell. I miss you, Sakura.](https://leohearts.com/archives/farewell-yingying.html)",
the archive's rendering of the blog title 永别了。我想你了，樱樱。,
on a page that names the person "Yingying" twice and whose front matter declares 樱樱 as Yingying;
`yingying1` and `yingying2` had written "Yingying" into the title.
Both lanes copied the archive's title this run,
the slice was never contested,
and the definition shipped as the archive wrote it.
A slice 0 select ballot on the same run shows the judges know the declared form
("Candidate 5 uses the name 'Sakura' instead of the declared name '樱樱' (Yingying)"),
so the title was not a judgement but an instruction:
the page-name glossary (class seventy-one) offers every same-href link title as the archive renders it,
under a heading that says to render the same person or title the same way everywhere,
and a title that names the entry's person carries the archive's other rendering of that name into the instruction.

Class eighty-six,
fixed in `40679244d`:
`linked-title-declared-name.ts` (new) pairs the front matter's declared forms
(name and aliases,
source to target by position,
a count mismatch pairing nothing)
and,
for a link pair whose source title carries a declared Han name the archive's title renders otherwise,
appends to the glossary line
`; names 樱樱, declared "Yingying": the declared form inside the title, the archive's words for the rest`;
the heading says "a declared name inside a title takes its declared form";
`document-preparation.ts` passes the declared pairs.
Probed off the built dist on the real `yingying` pair:
the line carries the note.
Guards red first at `751504ac8`
(`page-name-glossary.unit.test.ts`:
the note on a title the archive renders otherwise,
none where the archive already carries the declared form or the title names nobody declared),
lint 0 and 0,
types clean,
full suite green (`suite-class86.log`).
Mechanically the fix is a sheet instruction at the exact line,
not a floor:
the page's link text cannot be corrected by rule since "Sakura" is not derivable from 樱樱 by anything the pipeline holds.

`yingying4` (frozen `40679244d`,
pid 2180429,
overlap 8)
launched 18:07 UTC on Bedrock and OpenRouter
(meters 102.81 USD Bedrock,
151.24 OpenRouter at launch),
waiter binj7npur.
Read for the definition "[Farewell.
I miss you,
Yingying.]"
(class eighty-six live) beside every check of `yingying3`;
then `XingZ60` again for the handles.

## zheermao4 read, 2026-09-22, 17:50 UTC: SETTLED in 32 min, no class; the reply email revised from the transcript, the Arts letter kept as the archive; yingying3 launched

`zheermao4` (frozen `7ad1b8ec7`,
pid 2151756,
entry `zheermao101`) SETTLED at 17:40 UTC,
32 minutes after its 17:08 UTC launch (TALLY `ms=1898177`),
one attempt,
902 seats,
31 retry rounds,
786 calls,
16 abandoned streams (0.04 USD),
0.70 USD by the meters (Bedrock 103.17 to 102.89,
OpenRouter 151.74 to 151.32).
Synthetic dry throughout,
Qwen3.8-27B and glm-5.3 dark.

The page against `zheermao3`:
15 slices,
2 of 2 headings,
front matter equal,
no TA,
no error finish,
no double blank line,
publish check silent,
one em-dash (bench-written this time,
in the revised reply:
"Please don’t worry—we will";
the archive's own em-dash on the Arts letter is gone with the block the review replaced on `zheermao3`),
one unendorsed standing,
gate-kept (slice 12),
no slate-declined,
11 abstention lines,
no repeat or run-off finding.

The two picture blocks went the other way from `zheermao3`.
The Arts letter stands as the archive wrote it ("Actually,
Arts is also a cat-like person."
and the twelve lines that follow,
the "Good night,
ArtsEpiphany" close)
where `zheermao3` had revised it from the transcript;
the One Among Us reply is revised from the transcript
(six longer lines and a "Sincerely,
wishing you a happy spring."
close where the archive has fifteen short lines,
the archive's untranslated line and its JSX translator comment "这句不知道如何翻译" rendered as "you can avoid the edge of the darkness and hide for a while somewhere with light")
where `zheermao3` had kept it.
Both are the archive block review's call on a picture block,
one transcript per picture (the owner's 2026-09-16 decision),
and each run's bench drew the line differently.
The email label reads "English translation of this email conversation:"
(class seventy-seven holding).

Slice 12 ("Despite this,
she insisted on her decision... and ultimately chose to leave."):
the contest settled on neither (two ballots naming the dropped 仍然,
one each way);
the consolidation carried "still" and "own" and the gate tied 2 to 2
(two ballots calling "still" and "own" embellishments the Chinese lacks,
against 仍然 and 自己的 in the ORIGINAL),
so the standing shipped by the tie rule.
A wrong ballot,
not a class:
the gate's rule is stated,
the tie rule is the owner's,
and the standing is a faithful sentence short of one adverb.

`yingying3` (frozen `7ad1b8ec7`,
pid 2155878,
overlap 8)
launched 17:42 UTC on Bedrock and OpenRouter
(meters 102.89 USD Bedrock,
151.32 OpenRouter at launch),
waiter `b07ha4fr0`.
Read against `yingying2` (1 heading,
4 markers,
2 definitions,
front matter equal,
the References line kept,
"Yingying" for the archive's "Sakura");
then `XingZ60` again for the handles.

## CuspariaKLSY7 read, 2026-09-22, 17:15 UTC: SETTLED in 22 min, class eighty-five live (the gloss on the page and every line of the life in the past); zheermao4 launched

`CuspariaKLSY7` (frozen `7ad1b8ec7`,
pid 2147965) SETTLED at 17:08 UTC,
22 minutes after its 16:46 UTC launch (TALLY `ms=1303584`),
one attempt,
405 seats,
5 retry rounds,
419 calls,
no abandoned stream,
0.12 USD by the meters (Bedrock 103.20 to 103.17,
OpenRouter 151.83 to 151.74).
Synthetic wet at launch and dry again by 17:06 UTC,
glm-5.3 dark.

Class eighty-five live.
Slice 1 settled with no gate-kept and no unendorsed standing on the page (no finding line of either kind on any slice),
the three lines `CuspariaKLSY6` shipped in the present now read
"She really valued her friends.",
"She was an Eastern Orthodox Christian.",
"She didn’t believe in God all that much.",
and the archive's gloss stands beside the nickname line:
"She coined her nickname “Ling Shui Yu Yu Zi” while feeding fish.
“Ling Shui Yu Yu Zi” means fish in clear water."
(the archive breaks the gloss onto its own line inside the paragraph;
the candidate ran it on;
a single newline inside a paragraph is a soft break,
so the rendered page is the same,
per the owner's 2026-09-21 answer on line breaks).
The first settled CuspariaKLSY page carrying both the gloss and the past tense.

The page against `CuspariaKLSY5`:
1 of 1 heading,
front matter equal,
no TA,
no error finish,
no double blank line,
publish check silent,
no unendorsed,
gate-kept or slate-declined standing,
16 abstention lines,
no repeat or run-off finding;
wording differs on most lines (a fresh bench each run),
every line of the life in the past.

`zheermao4` (frozen `7ad1b8ec7`,
pid 2151756,
overlap 8,
entry `zheermao101`)
launched 17:08 UTC on Bedrock and OpenRouter
(meters 103.17 USD Bedrock,
151.74 OpenRouter at launch),
waiter `bktsyyo1o`.
A first launch under the run name `zheermao4` at 17:08 UTC named the entry `zheermao` and exited at once with `pending=0` (the corpus directory is `zheermao101`);
its empty run directory and log were removed before the relaunch.
Read against `zheermao3` (14 slices,
the email label kept,
the Arts letter,
past tense,
one em-dash left per owner,
no double blank line,
no unendorsed);
then `yingying`,
then `XingZ60` again for the handles.

## CuspariaKLSY6 read, 2026-09-22, 16:50 UTC: SETTLED in 16 min, but slice 1 flipped back to the archive's present tense to keep a translator's gloss line (class eighty-five); fixed in 7ad1b8ec7, CuspariaKLSY7 launched

`CuspariaKLSY6` (frozen `59fe52b51`,
pid 2121546) SETTLED at 16:33 UTC,
16 minutes after its 16:22 UTC launch (TALLY `ms=934677`),
one attempt,
413 seats,
14 retry rounds,
361 calls,
no abandoned stream,
0.22 USD by the meters (Bedrock 103.31 to 103.20,
OpenRouter 151.94 to 151.83).
Synthetic wet again from 16:27 UTC (five-hour window reset),
glm-5.3 dark.

The page against `CuspariaKLSY5`:
1 of 1 heading,
front matter equal,
no TA,
no error finish,
no double blank line,
publish check silent,
7 abstention lines,
no repeat or run-off finding,
the archive's translator gloss line "“Ling Shui Yu Yu Zi” means fish in clear water."
back on the page (dropped on `CuspariaKLSY5`).

But three lines of the life in the present tense
("She cares about her friends a lot.",
"She’s an Eastern Orthodox Christian.",
"She doesn’t trust God 100%,
though."):
slice 1 shipped gate-kept and unendorsed,
the archive's own text.
The lane contest settled on neither (5 of 8 usable;
the translate lane text carried the gloss AND the past tense,
and one ballot called the gloss "unsupported explanations",
another called the past tense "inconsistent with the present-tense life narrative the house rule holds",
which no sheet says);
the consolidation slate chose gpt-oss's seven-line candidate 4.5 of 6,
every ballot praising "nothing added or dropped" and the past tense;
the consolidate gate refused it 3 to 2 (Kimi,
deepseek,
gemma-e2b for the standing:
"consolidated omits the archive line glossing the nickname,
which the Chinese is silent about and which the page therefore keeps").
`CuspariaKLSY5` had gone the other way on the same slice (the gate accepted the consolidation 4 of 5,
the gloss lost,
recorded then as an observation).
Neither page carried both.

Class eighty-five:
what the page carries and the Chinese is silent about had four rules on four sheets.
`CONTEST_POLICY` (the lane contest and the gate):
"DROPPED ALSO COVERS WHAT THE ARCHIVE CARRIES AND THE CHINESE DOES NOT SAY ... keeping it is correct";
the slate criteria (`translate-selection-sheet.ts`):
"Faithfulness:
nothing added" with only the declared-name carve-out;
the consolidation writer (`consolidate-wire.ts`):
accurate detail the archive adds is kept,
listing a name,
a referent,
a credit and a citation's translator,
not a gloss;
the translate writer (`translate-wire.ts`):
its own list,
which did include "a gloss identifying someone".
A writer told to strip and a judge told to reward the stripping produce a candidate the gate must refuse,
and the refusal keeps the archive with whatever else the house rules would have changed.
The consolidation writer's tense rule also still named the archive as the authority
("KEEP THE TENSE OF THE PAGE ... the ARCHIVE RENDERING is what settles"),
class seventy-six's wording on a sheet that fix did not reach.
Corpus census:
the same shape stands on `Futajuhuacha` ("Huacha means flower-scented tea.",
archive-only,
inside the first paragraph)
and `Kotori` (the Latin motto's gloss).

Fix (`7ad1b8ec7`):
`page-apparatus-clause.ts` (new) carries one wording,
"WHAT THE EXISTING TRANSLATION CARRIES AND THE ORIGINAL IS SILENT ABOUT IS KEPT IN PLACE:
a gloss of a name or a term ...,
a candidate carrying it has added nothing,
and a candidate leaving it out has dropped page content",
interpolated into the slate's faithfulness criterion beside the declared-name carve-out,
into the consolidation writer's rules in place of its list,
into the translate writer's rules in place of its list,
and a gloss named among `CONTEST_POLICY`'s examples of what is not unsupported;
the consolidation writer's tense rule now reads "ONE TENSE,
IN THIS ORDER:
first the past tense the house rule sets for the life of a person who has died ...,
then the tense the page holds for what is still true".
Guards red first at `7e433ff86`
(`translate-selection-sheet.unit.test.ts`:
the clause inside the criterion that says "nothing added";
`consolidate-wire.unit.test.ts`:
the gloss kept and the house rule's past named,
the archive no longer the tense authority),
`tense-authority-reaches-every-sheet.unit.test.ts` moved to the new wording in the fix,
lint 0 and 0,
types clean,
full suite green (`suite-class85.log`).

Observation,
not shipped:
the repair lane's editors deleted the gloss line and left its blank behind,
so the repair text carried a run of three newlines inside the slice (`\n\n\n` between "while feeding fish."
and "She began ...");
the repair lane lost the contest,
and no floor reads a blank run inside a candidate.
A candidate's internal blank run collapses to one blank line in the rendered page,
so this is recorded,
not fixed.

`CuspariaKLSY7` (frozen `7ad1b8ec7`,
pid 2147965,
overlap 8)
launched 16:46 UTC on Bedrock,
OpenRouter and Synthetic
(meters 103.20 USD Bedrock,
151.83 OpenRouter at launch),
waiter `br3cmmd0a`;
read for the gloss line on the page AND every line of the life in the past (slice 1 first),
no gate-kept standing on slice 1,
then `zheermao`,
`yingying` one at a time,
then `XingZ60` again for the handles.

## hulicaijia13 read, 2026-09-22, 16:25 UTC: SETTLED in 68 min, class eighty-four live (two removals of the Nayuki note withheld, the note on the page with its marker); CuspariaKLSY6 launched

`hulicaijia13` (frozen `59fe52b51`,
pid 2112448) SETTLED at 16:16 UTC,
68 minutes after its 15:08 UTC launch (TALLY `ms=4068612`),
one attempt,
4,497 seats,
147 retry rounds,
3,775 calls,
10 abandoned streams (0.01 USD),
4.90 USD by the meters (Bedrock 105.39 to 103.31,
OpenRouter 154.76 to 151.94).
glm-5.3 and Qwen3.8-27B dark (Synthetic dry).

Class eighty-four live.
The archive block review put the relabeled `[^10]` note to its reviewers again;
deepseek-v4.1-flash and gpt-oss-120b proposed the removal,
and both were withheld at 15:19:36 UTC with the new finding
(`archive-revision-refused (...): the revision leaves the page with a footnote defect the archive did not carry (unresolved-reference gfm 10)`).
The one admissible revision,
"Formerly Nayuki,
renamed Naixue in 2025."
(the archive's "Naìsnow" spelled as the body spells the café),
was chosen over the incumbent by the selection round
(gemma-4-e2b and Mercury citing page-wide consistency with the body's "Naixue"),
so the page carries `Naixue[^10]` on the order line and the definition among the notes.
Two reviewer voices on that block were lost to the schema guard as the guard intends
(gemma-4-e2b answered `editorial-context` quoting the English archive,
minimax-m3 answered `source-supported` with no source quote);
23 schema-mismatch lines on the run,
the same count as `hulicaijia12` and fewer than `hulicaijia9`'s 30.

The page against `hulicaijia9`:
13 of 13 headings,
17 of 17 tag markers,
18 of 18 footnote markers,
9 of 9 definitions,
2 of 2 links,
3 of 3 Sakura,
"Conflict" for 左右,
front matter equal,
no TA,
no error finish,
no `contributor-name-restored` line,
two unendorsed standings (3,
58),
both gate-kept,
no slate-declined,
129 abstention lines,
no repeat or run-off finding.

`CuspariaKLSY6` (frozen `59fe52b51`,
pid 2121546,
overlap 8)
launched 16:22 UTC on Bedrock and OpenRouter
(meters 103.31 USD Bedrock,
151.94 OpenRouter at launch),
waiter `btbbvt2r4`.
Read by the seven steps and three checks against `CuspariaKLSY5`
(the life in the past tense,
1 heading,
1 marker,
front matter equal,
no double blank line,
publish check silent);
then `zheermao`,
`yingying` one at a time,
then `XingZ60` again for the handles.

## hulicaijia12 read, 2026-09-22, 15:10 UTC: SETTLED in 79 min, but a translator's note removed and its marker left dangling (class eighty-four); fixed in 59fe52b51, hulicaijia13 launched

`hulicaijia12` (frozen `4e5cf65e0`,
pid 2066905) SETTLED at 14:57 UTC,
79 minutes after its 13:38 UTC launch (TALLY `ms=4721721`),
one attempt,
4,577 seats,
149 retry rounds,
3,869 calls,
15 abandoned streams (0.02 USD),
4.96 USD by the meters (Bedrock 107.56 to 105.39,
OpenRouter 157.55 to 154.76).
glm-5.3 and Qwen3.8-27B dark (Synthetic dry).

The page against `hulicaijia9`:
13 of 13 headings,
17 of 17 tag markers,
2 of 2 links,
3 of 3 Sakura,
"Conflict" for 左右,
front matter equal,
no TA,
no error finish,
no `contributor-name-restored` line (the page's signers are archive-rendered,
so class eighty-three had nothing to restore here),
six unendorsed standings (3,
12,
18,
34,
50,
55),
four gate-kept (3,
18,
50,
55),
two slate-declined (12,
34),
129 abstention lines,
no repeat or run-off finding,
no `archive-revision-refused` line.

But 17 footnote markers and 8 definitions against `hulicaijia9`'s 18 and 9:
the archive's translator note `[^7]: Formerly Nayuki, renamed Naìsnow in 2025.`,
relabeled `[^10]` by the class nineteen relabel (the original carries 8 notes,
this one is the translator's),
went to the archive block review as an archive-only block;
five of five reviewers called the rename a factual claim the original does not carry
("criterion 1 requires such claims to be removed"),
the selection chose the removal candidate at weight 4 across 5 ballots,
and the definition left the archive text.
The marker `Naixue[^10]` stood in the body,
where no reviewer was asked about it,
so the page ships a reference to a note that is not there.
`hulicaijia7` had dropped both translator notes the same way (recorded then as an observation);
`hulicaijia8` and `hulicaijia9` kept them because the reviewers happened to retain the block.

Class eighty-four:
the archive block review's revision is spliced into the archive text (`archive-block-repair.ts`)
every later stage reads as the incumbent,
including the page footnote guard (`page-footnote-integrity.ts`),
which reads its defects AGAINST the archive so a defect the archive already carried is never blamed on the page;
a defect the review itself introduces is therefore inherited and blamed on nobody.
A footnote is a relation between blocks and a revision of one block is judged against the whole page it would leave:
`archive-revision-footnotes.ts` (new) replaces the block in the archive text,
runs `introducedFootnoteFindings` against the unrevised archive,
and withholds a revision that introduces an unresolved reference,
an orphan definition or a duplicate,
under the `archive-revision-refused` prefix with the defect spelled the way the page guard spells it;
`archive-replacement-candidates.ts` applies it after the shape floor (class seventy-seven),
so a removal of a referenced definition empties the slate and the note ships as the archive wrote it.
Guard red first at `27845568a`
(`archive-block-review-stage.unit.test.ts`:
a definition block removed while its marker stands,
retained with the finding,
no selection round bought),
fixed in `59fe52b51`,
lint 0 and 0,
types clean,
stage suite green,
full suite green (`suite-class84.log`,
1095 PASS,
exit 0).

`hulicaijia13` (frozen `59fe52b51`,
pid 2112448,
overlap 8)
launched 15:08 UTC on Bedrock and OpenRouter
(meters 105.39 USD Bedrock,
154.76 OpenRouter at launch),
waiter `b6vsy9ha7`.
Read by the seven steps and three checks against `hulicaijia9`
(18 markers and 9 definitions again,
the `[^10]` note on the page),
plus the `archive-revision-refused` line naming `unresolved-reference` for the removal;
then `CuspariaKLSY`,
`zheermao`,
`yingying` one at a time,
then `XingZ60` again for the handles.

## Class eighty-three amended, 2026-09-22, 13:40 UTC: syllable groups, never one joined word (owner's veto); hulicaijia11 killed, hulicaijia12 launched on 4e5cf65e0

The owner vetoed the one-word joining at 13:35 UTC:
"I want syllable groups.
Jie Cheng Tian Zou is fine.
Jiecheng Tianzou is fine.
Jiechengtianzou is not fine."
The reading now pairs the syllables,
each pair capitalised,
a space between pairs,
a lone trailing syllable standing by itself
(锦心 Jinxin,
雨狸 Yuli,
洁澄天奏 Jiecheng Tianzou,
洁澄天奏Official "Jiecheng Tianzou Official"),
pairs because the corpus's own romanisations pair them ("Baimao suki");
the house-policy bullet says "in capitalised groups of two syllables ...,
never one joined word" with the same examples.
Guards red first at `790719ea3`
(`handle-reading.unit.test.ts`,
new:
two syllables one group,
four syllables two groups,
a lone trailing syllable,
Latin letters kept a space apart,
the gloss tolerance;
the house-policy guard's phrase;
`handleReading`,
`withoutGloss` and `carriesRendering` exported through the readiness barrel for the guard),
fixed in `4e5cf65e0`,
lint 0 and 0,
types clean,
the three guard suites green,
full suite in `suite-class83c.log`.

`hulicaijia11` (frozen `fa014fc55`,
pid 2057750) was killed at 13:38 UTC,
ten minutes in,
still in preparation,
under always-kill-and-relaunch
(meters 107.57 to 107.56 USD Bedrock and 157.58 to 157.55 OpenRouter,
0.04 USD).
`hulicaijia12` (frozen `4e5cf65e0`,
pid 2066905,
overlap 8)
launched 13:38 UTC on Bedrock and OpenRouter
(meters 107.56 USD Bedrock,
157.55 OpenRouter at launch),
waiter `brc9j43qv`.
Read by the seven steps and three checks against `hulicaijia9`,
plus any `contributor-name-restored` line naming the pinyin reading;
then `CuspariaKLSY`,
`zheermao`,
`yingying` one at a time,
then `XingZ60` again for the handles.

## Class eighty-three built, 2026-09-22, 13:30 UTC: an undeclared handle is pinyin one way across the page, with its literal meaning in parentheses (owner's answer); hulicaijia10 killed, hulicaijia11 launched on fa014fc55

The owner answered the undeclared-handle question of the XingZ622 read at 13:25 UTC:
"A,
and put the literal translation in parens".
Option A as put to the owner was pinyin as one word,
capitalised;
its example line wrote 洁澄天奏 as "Jiecheng Tianzou",
which contradicts the label,
and the rule follows the label
(one word:
Jiechengtianzou),
since the mechanical restore needs one deterministic form;
the owner can veto the joining.

### The rule

`house-policy.ts` gained a bullet after the community-vocabulary bullet:
a person's handle with no declared,
archive or corpus rendering
(a signer,
a cited author,
an account name the ORIGINAL writes in Han)
is romanised as the handle is read,
pinyin for a Chinese handle,
as one capitalised word (锦心 is Jinxin,
雨狸 is Yuli,
洁澄天奏 is Jiechengtianzou),
with its literal meaning in parentheses the first time it appears on the page
(Jinxin (Brocade Heart))
and the romanisation alone after that;
never left in Han,
never rendered by its meaning alone,
the same way everywhere on the page;
a handle the person writes in Latin letters keeps their spelling (HiYku,
wing,
Mikä),
and Latin letters the ORIGINAL writes against a Han handle stay after the romanisation
(洁澄天奏Official is Jiechengtianzou Official).
The bullet reaches every producing and judging sheet through `HOUSE_POLICY_BLOCK`.

### The mechanical half

- `handle-reading.ts` (new):
  `handleReading` reads a handle through `pinyin-pro` (already a dependency,
  used by the directory-id check),
  every run of Han as one capitalised toneless word,
  everything else as written,
  a space between a romanised run and Latin letters or digits against it;
  probed:
  锦心 Jinxin,
  雨狸 Yuli,
  洁澄天奏Official "Jiechengtianzou Official",
  白毛 suki "Baimao suki",
  空白/引号 "Kongbai/Yinhao",
  Mikä unchanged.
  `withoutGloss` strips a trailing parenthetical;
  `carriesRendering` accepts a written name equal to the rendering or the rendering with its gloss.
- `contributor-name-authorities.ts` (new,
  `nameAuthorities` moved out of the restore for the line budget):
  every signer takes an authority,
  not only the headed ones (class sixty-seven's restriction):
  the archive's aligned signature,
  else the page's aligned signature where it carries no Han (its gloss stripped),
  else the handle's pinyin reading,
  the finding naming which
  (`contributor-name-restored (... ; the pinyin reading of the original's handle)`).
- `contributor-name-restore.ts`:
  a heading or signature already carrying the rendering with its gloss is left alone,
  so the gloss survives at the first appearance and is not copied to the rest.
- `attribution-line.ts`:
  a signature's name ends at 【 or 《 as well as at a comma,
  so a song credit (—— 雨狸【妄想症Paranoia】《零重祈愿》) is a signature the restore reaches;
  `nameEndAt` replaces `commaAt`.

Guards red first at `3474e062f`
(`contributor-name-restore.unit.test.ts`:
a Han signature romanised in heading and signature with the pinyin finding,
a glossed heading left alone,
a twice-signed unheaded name rendered one way;
`house-policy-reaches-the-judges.unit.test.ts`:
the rule's phrases),
fixed in `fa014fc55`,
lint 0 and 0,
types clean,
full suite green twice (`suite-class83.log` before the split,
`suite-class83b.log` after,
1094 PASS lines,
exit 0).

### What this does not cover

A handle in running prose or in a link's text ("by 洁澄天奏Official" on the Caged Bird attribution,
which is no signature line)
is covered by the rule on the sheets alone;
the mechanical restore reaches headings and signature lines.
The next XingZ60 read checks "by Jiechengtianzou Official" and the song credits.

### hulicaijia10 killed, hulicaijia11 launched

`hulicaijia10` (frozen `6c701485d`,
pid 2018636) was killed at 13:28 UTC,
14 minutes in,
at chunk 28 of 50 of the repair lane,
under always-kill-and-relaunch once `fa014fc55` landed
(meters 107.68 to 107.57 USD Bedrock and 157.75 to 157.58 OpenRouter,
0.28 USD).
`hulicaijia11` (frozen `fa014fc55`,
pid 2057750,
overlap 8)
launched 13:28 UTC on Bedrock and OpenRouter
(meters 107.57 USD Bedrock,
157.58 OpenRouter at launch),
waiter `bfokdo0kk`.
Read by the seven steps and three checks against `hulicaijia9`,
plus any `contributor-name-restored` line naming the pinyin reading;
then `CuspariaKLSY`,
`zheermao`,
`yingying` one at a time,
then `XingZ60` again for the handles.
