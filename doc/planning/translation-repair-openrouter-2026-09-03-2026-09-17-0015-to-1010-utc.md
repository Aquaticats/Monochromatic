# OpenRouter and the corpus passes: 2026-09-17, 00:15 to 10:10 UTC

What was launched,
read and decided on 2026-09-17 00:15 to 10:10 UTC.
Entries:
hulicaijia7,
hulicaijia6,
Mio27,
Mio26,
Mio25,
Mio24.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## hulicaijia7 launches on class forty-five, 2026-09-17, 10:10 UTC

Frozen `978bd7d20` (classes forty-one to forty-five),
pid 3491560,
runs `~/temp/agent/hulicaijia7`,
log `~/temp/agent/hulicaijia7.log`,
Bedrock and OpenRouter wet,
Synthetic and Hyper dry at launch
(the launch log names glm-5.3 and Qwen3.8-27B with no provider on the attestation).
Read first the class forty-five finding in the page assembly's findings
(`repeats another section's heading` in the log or the artifact's `pageAssembly`)
and the thirteen headings on the page,
then the three shapes no read page has met
(the inline ruby annotation,
the Sakura component and the footnote),
then any `carries the sheet's own` refusal,
then the `ATTESTED` lines,
then the seven steps and the three checks.

## Class forty-five, 2026-09-17, 10:05 UTC: a lane renders one section under another section's heading and the page dies at publish

Found by the hulicaijia6 read.
The entry has thirteen level-2 headings,
and the archive renders every one distinctly
(初识 as `## Meeting`,
相遇 as `## Soulmate`,
among them).
The repair lane rendered slice 8's 相遇 as `## Meeting`,
which is a fair rendering of the word and the archive's heading for 初识 two sections earlier;
every slice floor passed it,
since a slice holds one heading and nothing in a slice's evidence names another section's;
the consolidation wrote from it;
and the publish guard
(`assertHeadingsStayDistinct`,
built for the twenty-third hakureico read)
refused the page after 205 minutes:
"would render 13 distinct source heading(s) as 12 distinct page heading(s)".
The launcher's re-attempt resumed the same cached slice texts and would have died the same way,
so it was killed at 09:54 UTC.
The fault is the place of the check,
not the check:
the one stage that sees every heading the page would carry sat after the last decision.
Fixed in `978bd7d20`
(guard red first at `b6bfd2b96`,
the commit before it):
`restoreCollidingHeadings` (`corpus-run/heading-collision-restore.ts`)
runs inside `guardPageAssembly`,
which now takes the original's text.
It reads the headings the page would carry slice by slice
(the replacement where one ships,
the archive's text where none does),
and where the original and the archive have the same number of headings,
a page heading repeats another section's,
the original's two headings differ,
and the page's rendering is not the archive's at that position,
it puts the archive's heading back into that slice's text.
The restored slice is recorded as a page assembly override
(the same `trimmed` row a footnote trim writes;
`would-ship-text.ts` reads it as the page's wording,
`decidedBy: page-assembly`)
with a finding naming both headings.
Measured at pin `a41fc607` no archive renders two distinct source headings identically,
so the archive's rendering is always a distinct fallback;
the publish guard keeps the last word for a page the restoration cannot mend
(a page carrying fewer headings than the original,
which the block floor owns).
A restoration that returns a slice to the archive's exact wording is left out of the footnote guard's input,
whose invariant refuses an unchanged replacement,
and still rides the override.

## hulicaijia6 read, 2026-09-17, 09:54 UTC: ERROR at publish, thirteen headings rendered as twelve

Frozen `0986c871f`,
205 minutes to the TALLY
(`ms=12288953`),
`status=ERROR`,
no page written
(`fixed/` empty;
`attempts.json` reads 2,
the re-attempt killed by pid at 09:54 UTC before it could resume the same texts).
Spend by `pass-spend.mjs`:
3.71 USD logged,
5,312 seats asked,
137 retry rounds,
19 abandoned OpenRouter streams (0.14 USD);
Bedrock 1.92 (gemma-26b 650 calls,
gemma-e2b 610,
gpt-oss-120b 524),
OpenRouter 1.64 (deepseek-v4.1-flash 463 calls with 1.7 M cached prompt tokens,
mercury 569,
minimax 449),
Hyper 680 calls at no meter,
Synthetic dry by the end.
Meters at the kill:
Bedrock 180.83 USD,
OpenRouter 258.20 USD.
What the log says before the error:
the attestation bought the one cited reference
(cq.gov.cn,
2,525 characters)
and answered nothing on it twice
(`ATTESTED heard=5 answered=0 verified=0 needed=3`),
which is the healthy reading of a government page cited for a date;
72 slices translated with no `carries the sheet's own` refusal
(class forty-four's first live measurement:
zero of the pass's candidates carried a label);
the 长夜 heading was argued at length by the slate judges
("Long Night" against "The Long Night")
and stayed distinct.
The ruby annotation,
the Sakura component and the footnote were never read,
since no page was written;
hulicaijia7 reads them.
The heading collision is class forty-five,
recorded under its own heading.

## hulicaijia6 launches on class forty-four, 2026-09-17, 06:15 UTC

Frozen `0986c871f` (classes forty-one to forty-four),
pid 3369011,
runs `~/temp/agent/hulicaijia6-20260917`,
log `~/temp/agent/hulicaijia6-20260917.log`,
Synthetic back wet since 05:30 UTC,
Hyper dry,
Bedrock and OpenRouter wet.
The queue resumes here on the Mio27 page reading clean.
Read first the three shapes no read page has met:
the inline ruby annotation,
the Sakura component and the footnote;
then any `carries the sheet's own` refusal in the translate log
(class forty-four's first live measurement),
then the `ATTESTED` lines
(hulicaijia cites pages or it does not;
`REFERENCE` lines in the preparation say which),
then the seven steps and the three checks.

## Class forty-four, 2026-09-17, 06:10 UTC: the writers copy the sheet's picture transcript into their candidates

Found by the Mio27 read.
Of the pass's 68 translate slate candidates,
four carried the writer sheet's own `WHAT THE PICTURES HERE SAY` block,
copied under the picture component with the readers' transcript inside,
and both slices that had one chose it:
slice 11 (photo4 and the rule) three of four candidates carried it and the judges chose one three of four,
gpt-oss-120b for "the full PhotoScroll component,
the heading,
and all transcribed text exactly as in the ORIGINAL",
deepseek and gemma-26b alike,
minimax abstaining because a candidate "changes the original heading";
slice 17 (the poem,
unpaired this time) one of four carried it,
the judges chose it,
the contest chose translate five of five without a ballot naming the block,
and the consolidation replaced it three of five because the standing was "10.9x" the original,
not because anyone named the block.
The judge sheet labels the transcript
"WHAT THE PICTURES HERE SAY,
transcribed by two readers that agreed",
which the judges read as a heading of the original.
The deterministic rule had no opinion:
a candidate carrying the sheet's evidence was as valid as one without.
Fixed in `0986c871f`
(guard red first at `d138f456b`):
`sheetLeakFindings` refuses a candidate carrying any of the sheets' fence labels
(`WHAT THE PICTURES HERE SAY`,
`EXISTING TRANSLATION`,
`ATTESTED DETAILS`,
`DECLARED NAMES`,
`CITED REFERENCES`,
`LATEST REJECTION`,
`ARCHIVE RENDERING`) that neither the original nor the page carries,
before any shape is read,
so such a candidate never reaches a judge;
run on Mio27's two chosen lane texts it refuses both.
The page was clean because slice 11's contest settled neither with the archive endorsed
and slice 17's consolidation wrote its own text.

## Mio27 read, 2026-09-17, 06:05 UTC: the sister clause ships end to end, the poem keeps its five breaks

`TALLY Mio status=SETTLED slices=18` in 116.9 minutes,
one attempt
(`attempts.json` reads 1).
The pairing gave 18 slices this time and left the poem unpaired
(slice 17,
incumbent empty),
where Mio25's pairing had set the farewell paragraph against it.

The attestation
(class forty-three's first pass):
`ATTESTED heard=5 answered=7 verified=6 needed=3 details=1` after
`ATTESTED CONFIRM heard=5 candidates=2 needed=3 confirmed=1`.
Four voices extracted the sister
(minimax-m3,
gemma-4-26b-a4b,
gpt-oss-120b,
mercury-2.5),
the confirmation ratified it five of five,
and minimax's other candidate,
pairing "she became a gentle big sister to many people" with the reference's sentence on Mio's sister,
was confirmed by one voice and dropped.
So the confirmation ratified rather than rescued;
its rescue path is still unmeasured on a live entry.

Slice 2.
The repair lane kept the sister clause and wrote
"With her brilliant mind and clever hands,
she was studying at a local 985 (one of China's elite universities)"
with an em dash before the gloss;
the translate lane dropped the clause.
The contest chose repair five of six,
four ballots naming the attestation
("keeps the attested archive detail that Mio's older sister is also trans"),
one ballot for translate on the class thirty-seven reading
("while attested in the references,
is not in the provided Chinese original").
The consolidation gate split two consolidated,
two standing,
one neither
("both candidates add the unsupported detail"),
so the standing repair text stood
(`gate-kept-standing`).
The polish
(class forty-one's first pass):
deepseek-v4.1-flash's proposal changed "handles everything at home" to "homemaker" and "clever hands" to
"deft hands",
kept the clause,
and the polish gate chose it five of five,
two ballots saying so in words
("retain the attested sister detail").
The page's line 20 reads "She has an older sister who is also trans.",
the first Mio page to carry the clause through every stage since class thirty-five was found.
The naturalness review's one finding on the clause
("'also' lacks an antecedent") is recorded as evidence.

Slice 17,
the poem:
the translate lane wrote it with five `<br/>` breaks,
the title as "Lamenting Autumn",
the credit as `——[Xiao Chui](https://space.bilibili.com/246513889)`;
the contest chose translate five of five
(the repair lane was empty,
the poem being unpaired);
the consolidation wrote its own rendering
("a final sigh of regret",
"created for Mio") and the gate chose it three of five;
the page ends `May you rest in peace, Mio.`,
a blank line,
then the poem as a blockquote with five `<br/>` and the link.
Class forty-two's substitute floor was not exercised
(no page block stood against the poem);
the source-only floor held as before.
No `repeats the ORIGINAL untranslated` refusal fired.
Class forty fired once:
slice 4's contest tied three to three and settled neither,
the standing was the archive's paragraph,
the repair lane's text was offered on the slate,
the slate declined it,
and the archive's wording shipped.

The first chat block ships with the archive's translation,
its `Translation:` label and its speakers;
"masculine" corrected;
0 straight double quotes,
31 curly apostrophes against the archive's 29.
The opening quote is the archive's own line
("Does my name mean 澪,
you ask?"),
which Mio25 had rewritten.
The three checks:
front matter equals the archive's,
one `json false start` kept,
no `finish_reason=error`,
no `InStreamProviderError`.
The seven checks against Mio12:
澪 kept in the archive's own sentence;
the coming-out paragraph and its three bullets as the archive has them
(no prose duplication this time);
the university as "a local 985 (one of China's elite universities)" with the dash;
SRS unexpanded;
the `Translation:` label;
"masculine";
the poem with five breaks and the credit line under the farewell.
Wording notes,
not fixed:
"## Found a Best Friend" in title case;
"Beiyan Yunyi" where Mio25 wrote "Bei Yan Yun Yi";
the dash in the 985 gloss.

Spend:
1,314 seats asked,
46 retry rounds,
1,202 SPEND lines,
26 abandoned streams at 0.07 USD;
Bedrock 183.06 to 182.74,
OpenRouter 260.63 to 259.92 against 0.99 logged
(the meter moved less than the log),
1.03 USD in all.
Synthetic came back wet at 05:30 UTC
(weekly 0.14 percent),
so the Kimi,
Qwen and GLM seats were reachable for the lanes and the contest.

## Mio27 launches on class forty-three, 2026-09-17, 04:07 UTC

Frozen `f7aa7c675` (classes forty-one,
forty-two and forty-three),
pid 3011126,
runs `~/temp/agent/Mio27-20260917`,
log `~/temp/agent/Mio27-20260917.log`,
Synthetic and Hyper dry,
Bedrock and OpenRouter wet.
Read first the `ATTESTED CONFIRM heard=H candidates=C needed=N confirmed=K` line
and the `ATTESTED` line after it
(the sister should be attested whenever one voice extracts it and the bench confirms it),
then the Mio26 reading order:
slice 2's polish,
the page's sentence on the sister,
slice 16's hard breaks,
any untranslated refusal,
`attempts.json`,
then the Mio25 reading.

## Mio26 killed at 04:07 UTC in its pictures phase

Under ALWAYS KILL AND RELAUNCH once `f7aa7c675` landed;
42 minutes in,
still reading pictures,
no contest run.
Its attestation is class forty-three's evidence and is recorded there;
nothing else of it was read.

## Class forty-three, 2026-09-17, 04:00 UTC: the attestation's open question is answered empty by voices that never looked

Mio26's attestation printed `ATTESTED heard=5 answered=2 verified=1 needed=3 details=0`.
The replies,
read back from the payload cache:
minimax-m3 `{"attested":[]}` in 6 completion tokens,
gemma-4-e2b the same in 6,
gpt-oss-120b the same in 666,
mercury-2.5 the same in 2,416,
gemma-4-26b-a4b the sister with both quotes verifying
(plus one item on the QQ group that did not verify),
deepseek-v4.1-flash cut by its completion cap at 13,082 tokens with no content
(`finish_reason=length`,
voice lost),
GLM-5.3-Flash abandoned after 866,605 delivered characters,
Qwen and glm-5.3 unreachable with Synthetic dry.
One verified voice against a quorum of three,
on the same bench that attested the sister five of six on Mio25,
four of five on Mio24 and five of six on Mio23:
the open question ("list every detail...") is one a cheap voice answers in six tokens without reading,
and an empty list is not a vote that the detail is false.
Fixed in `f7aa7c675`
(guard red first at `b41ff966c`):
extraction proposes and confirmation disposes.
Every verified detail,
one voice or more,
goes back to the same bench as a numbered candidate carrying its archive quote and its reference quote
(`buildReferenceAttestConfirmMessages` in `reference-attest-confirm-wire.ts`),
each voice answers `{"confirmed":[n,...]}`,
and a candidate confirmed by at least half the confirming voices is attested with the confirmation's counts
(`confirmedDetails`).
No candidate means no second round;
a confirmation nobody answers leaves the extraction quorum standing.
The log prints `ATTESTED CONFIRM heard=H candidates=C needed=N confirmed=K` before the `ATTESTED` line,
and the findings carry
`reference attestation confirmed K of C candidates by H voices`.
Not fixed here:
deepseek's cap cut and GLM-5.3-Flash's runaway on a 5,000-token sheet,
which the confirmation's shorter question may or may not repeat;
Mio27 measures it.

## Mio26 launches on classes forty-one and forty-two, 2026-09-17, 03:25 UTC

Frozen `052f8eea0` (class forty-one,
the references on the refine sheets;
class forty-two,
the break floor under a substitute block and the untranslated refusal),
pid 2838543,
runs `~/temp/agent/Mio26-20260917`,
log `~/temp/agent/Mio26-20260917.log`,
Synthetic and Hyper dry,
Bedrock and OpenRouter wet.
Read first slice 2's polish
(`consolidation.slices` by `sliceIndex`,
`polish.baseText` against `polish.text`,
and the polish gate ballots,
which should now cite the attested line rather than call the clause unsupported),
then the page's sentence on the sister,
then slice 16's rendering
(`cat -A` on the page's last lines:
the poem's quote lines end in two spaces when the breaks survived,
and the translate log carries
`ORIGINAL blockquote block(s) carry 5 explicit line break(s)` on any flat candidate),
then any `repeats the ORIGINAL untranslated` finding in the log,
then `attempts.json`
(Mio25 needed two),
then the Mio25 reading.

## Classes forty-one and forty-two, 2026-09-17, 03:20 UTC: the polish strips the attested clause, the verse loses its breaks under a substitute paragraph

Class forty-one.
On Mio25's slice 2 the consolidation slate gate kept the consolidated text with
the sister clause four ballots to one
(three of the four name the attestation),
and the consolidation polish then removed the clause:
three refiners heard,
three proposing,
minimax-m3's proposal selected at weight 2 of 5 ballots
(the record notes a deepseek-v4.1-flash self-vote),
and the polish gate chose the polished text four ballots of six
("removes the unsupported 'also trans' detail",
"the original says only 有一个姐姐,
so base's unsupported addition must go"),
one ballot for the base
("drops the information about the sister being trans,
which is present in the base and the archive rendering"),
one neither.
The page shipped "She has an older sister."
The mechanism is the same blindness class thirty-six closed for the contest and the slate:
`runRefineStage`,
`buildRefineMessages`,
`buildRefineSelectionContext` and `ConsolidationPolishGateSubject` carried no `referenceContext`
(the only files naming the reference block were the critic,
the adjudication,
the lane contest,
the slate and its gate,
and the attestation),
so the repair lane's refiners and their judges,
and the consolidation polish and its gate,
had never seen the references or the attested lines.
Fixed in `836f90687`
(guards red first at `16f7639a1`):
the reference lines and their rule go on the refiner sheet after the declared names
(`citedReferenceCandidateLines`),
the refine judges read them as labelled evidence in both modes
(`citedReferenceEvidence`),
the polish gate sheet carries the same block after the two candidates,
the repair driver hands `prepared.referenceContext` down through `refineSettledSlices`,
`runRefinePhase`,
`settleRefinePhaseSlice` and `settleRefinedSlice`,
the polish reads it off the consolidation subject,
and `refineRunShape` folds it into the refine cache key only where it exists,
so the 33 entries that cite nothing key exactly as before.

Class forty-two.
Mio25's page carries the poem as a blockquote with the bilibili link and none of its five hard breaks:
the translate lane's rendering wrote each verse line as two soft-broken lines,
Markdown renders soft breaks in a quote as spaces,
and the poem reads as one paragraph.
Mio23's rendering had kept the breaks by the writer's choice.
The rule that owes explicit breaks
(`sourceOnlyBreakFindings`) returns nothing whenever the slice's page text is non-empty,
and this slice's page text is the archive's farewell paragraph paired with the poem quote
(class thirty-two's substitute),
so the verse's breaks were never owed.
Fixed in `052f8eea0`
(same guards):
`substituteBreakFindings` owes,
per block kind the page never rendered,
the original's break count for that kind,
counted per kind because the candidate carries the page's paragraph beside the original's quote and
indexes do not align;
run on Mio25's own rendering it refuses with
"ORIGINAL blockquote block(s) carry 5 explicit line break(s),
but your blockquote block(s) carry 0",
and it passes Mio23's.
The same commit refuses a candidate that repeats a Han-carrying original character for character
(`untranslatedFindings`,
a bare link or component returned as it stands still passes):
on Mio25's first attempt two of the four poem candidates were the Chinese returned as it stood,
one judge chose one of them as "the most faithful rendering",
and the select missed its weight floor twice.

## Mio25 read, 2026-09-17, 03:00 UTC: settled on the second attempt, the sister clause lost at the polish, the poem without its breaks

`TALLY Mio status=ERROR` at 73.3 minutes,
then `REATTEMPT Mio queued: 61 more cache records than it had`,
then `TALLY Mio status=SETTLED slices=17` at 65.2 minutes on the second attempt:
136 minutes wall,
two attempts,
`attempts.json` reads 2.

The first attempt died on slice 16,
the poem,
in the translate lane
(`SLICE-COST lane=translate chunk=16 sourceChars=132 ms=689571`).
The select's first round heard four of five judges,
three abstained,
and the one vote drew weight 1 against the 2.00 floor
("keeping the fallback"),
so the stage ran its one follow-up round:
three of five writers heard
(Qwen3.8-27B unreachable with Synthetic dry,
GLM-5.3-Flash cut at the 360 s deadline after 1.9 MB of raw stream,
59,468 reasoning characters and no content),
a retry round asked two more,
and the follow-up select split:
gpt-oss-120b candidate 1,
gemma-4-26b candidate 1 at weight 0.5,
deepseek-v4.1-flash candidate 3,
gemma-4-e2b candidate 4
("preserves the original Chinese text exactly,
including the title '惋秋'"),
with three ballots naming candidates 2 and 4 as the Chinese left untranslated;
weight 1.5 against 2.00,
and the slice has no archive translation to fall back on,
so `TranslateAbsenceError` ended the attempt.
The launcher's re-attempt resumed from the cache and the second select settled the poem.
Class forty-two's untranslated refusal takes such candidates off the slate before any judge reads them;
the absence error itself stands as designed
(nothing honest to ship),
and the re-attempt is the recovery.

Class forty was not exercised:
no `lane texts offered on the slate` line,
because the second attempt's contest on slice 16 chose translate six of six with the archive flawed on
every ballot,
the consolidation terminal is `slate-endorsed-standing` and the translate text shipped unchanged.
The page ends on `May you rest in peace, Mio.`,
then the poem as a blockquote,
`Autumn Lament` as its title,
the credit line with `https://space.bilibili.com/246513889`,
and no hard break on any line
(`cat -A` shows no `  $`;
Mio23's page had them on every verse line):
class forty-two.

`ATTESTED heard=6 answered=13 verified=12 needed=3 details=1`
on the second attempt
(heard 5,
verified 11 on the first):
the sister by five of six voices
(minimax-m3,
gemma-4-26b-a4b,
deepseek-v4.1-flash,
mercury-2.5,
GLM-5.3-Flash),
minimax's graduation-certificate and surgery-money items and deepseek's five other items verified but short of the quorum.

Slice 2.
The repair lane kept the sister clause and wrote
"she attended a local 985 (a top-tier university) at the time of her death"
for 生前;
the translate lane wrote "and she has an older sister" and "during her lifetime".
The contest chose translate four of six with two `neither`,
every ballot weighing the repair lane's unsupported "at the time of her death" against the translate
lane's dropped attested detail
("although it drops the archive's attested detail that Mio's older sister is also trans").
The consolidation gate chose the consolidated text four of five,
with the clause,
three ballots naming the attestation and one calling the clause fabricated.
The polish removed it and its gate confirmed the removal four of six:
class forty-one.
The independent naturalness review then read the polished text unacceptable four of six
(the "Guangdong,
Foshan" order,
"during her lifetime" for 生前,
"a local 985" as jargon),
recorded as evidence with the gated text shipping,
as designed.
The page reads "Her father is a high school teacher,
and her mother is a housewife.
She has an older sister.
Brilliant and dexterous,
she attended a local 985 (a top-tier university) during her lifetime."

The first chat block ships with the archive's wording and speakers,
the `Translation:` label,
"masculine" and "WeChat" corrected,
curly quotes throughout
(`read-page.mjs`:
30 curly apostrophes against the archive's 29,
0 straight double quotes,
the 20 straight apostrophes all inside the JSX photo paths).

The three checks:
front matter equals the archive's;
one `json false start` kept;
46 `schema-mismatch` lines,
none naming `finish_reason=error`,
and four `InStreamProviderError` lines with `code unnamed`,
retried.
The seven checks against Mio12:
澪 kept;
the coming-out sentence ships as prose
("She once served as an administrator of the 春の芽工作室 (Harunome Studio) QQ group,
and she also:")
above the archive's three bullets,
the redundancy as before;
the university as "a local 985 (a top-tier university) during her lifetime";
SRS unexpanded,
as the archive has it;
the `Translation:` label restored;
"masculine";
the poem with its credit line under the farewell but without its five breaks
(class forty-two).

Spend:
2,148 seats asked,
77 retry rounds,
1,737 SPEND lines,
39 abandoned streams at 0.07 USD;
Bedrock 183.59 to 183.09,
OpenRouter 261.51 to 260.77 against 0.73 logged,
1.23 USD for the two attempts.
Synthetic served 222 calls at no cost while its meter read dry.

## Mio25 launches on class forty, 2026-09-17, 00:40 UTC

Frozen `1c38e4f34` (class forty,
the lane texts on the consolidation slate),
pid 2590736,
runs `~/temp/agent/Mio25-20260917`,
log `~/temp/agent/Mio25-20260917.log`,
Synthetic and Hyper dry,
Bedrock and OpenRouter wet.
Read first any `lane texts offered on the slate` line
(one per slice whose standing was neither contest-endorsed nor eligible),
then slice 17's settlement:
which candidate won,
whether the page carries the poem as a blockquote with its line breaks and
`https://space.bilibili.com/246513889`,
then slice 2's sister clause and the contest,
then the first chat block,
then the Mio21 reading and the spend.

## Class forty, 2026-09-17, 00:35 UTC: a declined contest leaves the slate blind to the lane texts

Mio24 stopped INCOMPLETE at slice 17,
the closing poem.
The archive never translated the poem;
its page ends on the farewell line the Mio pages close with,
which the pairing set against the poem's blockquote,
so the slice's incumbent is that one line and the original carries a link the line does not.
The contest heard four ballots,
every one judging the archive flawed,
and settled `neither` three to one:
two ballots called both lanes unsupported for "gendered pronouns" and for the translate lane's
"May you rest in peace" line,
one said the lanes differ only in wording,
one chose translate.
A declined contest leaves the archive's rendering standing,
the deterministic publication rule refused it
(the log line names the bilibili link the original carries and the translation does not),
and the consolidation withheld it from the slate under the owner's decision of 2026-09-04.
The slate then carried only the writers' fresh proposals,
which flattened the poem's blockquote and its line breaks,
and the slate judges said so:
three of five abstained
("none of the candidates reproduce the block quote structure",
"the passage being replaced is a single line"),
the two who voted tied one to one,
the fallback was the withheld standing,
and the entry stopped after 76.7 minutes with `slate-declined-standing`.
The repair lane's text
(`> Lament for Autumn`,
five lines,
the attribution with the link)
and the translate lane's
(the same with the farewell line kept above it)
both passed the rule,
both carried the blockquote,
the line breaks and the link,
and neither was on any slate.
Mio23 never met this because its contest chose translate on slice 17.

The fix,
`1c38e4f34`:
the consolidation slate offers each lane text that passes the deterministic rule and is not the standing,
whenever the standing is neither contest-endorsed nor eligible
(`laneTextsForSlate` in `consolidate-lane-offer.ts`).
A lane text joins the slate under a new `lane` producer kind,
between the incumbent and the fresh proposals,
so the incumbent's bytes win a collapse against a lane that reproduces them and a lane's bytes win one
against a proposal that reproduces the lane,
with the reproducing model discounted as it is for the incumbent.
The settle key carries the offered lane texts only when any were,
so every other slice keys as before;
a floor that refuses every proposal no longer ends the settlement when lane texts remain.
A silent producer round still interrupts the entry as before,
lane texts or none.
Guards red first at `341a540bf`
(lane candidates,
the offer rule,
the settle key,
the artifact producer reader),
all four checks green at `1c38e4f34`.
Not changed:
the contest's `neither` still leaves the archive standing,
which is the consolidation's retry by design;
the slate judges' reading of the one-line standing as the scope of the passage
("the task is to replace only the current English passage")
is noted,
not fixed.

## Mio24 read, 2026-09-17, 00:15 UTC: the attestation and the repair lane hold, the entry stops at slice 17

Mio24 (frozen `27109ca8b`,
launched 22:58 UTC,
Synthetic and Hyper dry) ended INCOMPLETE after 76.7 minutes at slice 17,
class forty,
so no page was written and the seven checks,
the three checks and the first chat block could not be read.
What could be read:

- `ATTESTED heard=5 answered=6 verified=6 needed=3 details=1`;
  the sister detail by four of five voices
  (gemma-4-26b-a4b,
  deepseek-v4.1-flash,
  mercury-2.5,
  Kimi-K3),
  mercury's two extra items
  (the depression hospitalisation,
  the graduation certificate)
  verified but fell to quorum.
- Slice 2,
  the repair lane:
  both addition claims on the sister
  ("Translation adds that the sister is trans",
  "adds that the older sister is also trans")
  rejected before the panel with the reference-attested finding,
  the mistranslation claim on "handles everything at home" rejected by the panel,
  the omission of 当地 accepted,
  and the lane kept "She has an older sister who is also trans."
- Slice 2,
  the translate lane,
  the first pass with the ATTESTED DETAILS block on the writer sheet:
  one candidate of the slate carried the clause
  (candidate 3,
  which mercury voted for,
  "incorporates the cited-reference detail about the sister"),
  and gemma-4-26b-a4b's candidate 4,
  "and she has an older sister",
  won three ballots of five
  (minimax,
  deepseek-v4.1-flash,
  gemma-4-e2b,
  each praising it for adding nothing).
  The writers now hear the rule and one followed it;
  the translate judges have the candidate-sheet rule and three of five still preferred the dropper.
  Recorded,
  not fixed:
  the repair lane carries the clause and the contest chose it.
- Slice 2,
  the contest:
  repair 3 to 2,
  the translate ballots citing "before she passed away" as the qualifier repair lacks,
  one repair ballot noting that repair
  "does drop the archive's accurate attested detail" in its reason while choosing repair.
  Had the page assembled,
  it would have carried the clause.
- Slice 17:
  class forty,
  read in full under its own heading.
- The other contests:
  fourteen settled,
  ten repair,
  three translate,
  one neither
  (slice 17);
  the archive judged flawed on most ballots of most slices.
- Spend:
  select 107 rounds,
  474 seats asked,
  392 heard;
  translate 43 rounds,
  157 asked,
  75 heard;
  1,450 seats asked in all with 66 retry rounds.
  METERS at exit:
  Bedrock 183.59 USD,
  OpenRouter 261.52 USD,
  Synthetic and Hyper dry.
  Read the meter deltas at the Mio25 read together with Mio25's own.
