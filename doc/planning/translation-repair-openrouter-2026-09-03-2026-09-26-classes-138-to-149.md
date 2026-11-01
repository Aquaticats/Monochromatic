# OpenRouter and the corpus passes: 2026-09-26, classes 138 to 149

What was launched,
read and decided on 2026-09-26 classes 138 to 149.
Entries:
hulicaijia31,
hulicaijia30,
hulicaijia29,
XingZ6013,
hulicaijia28,
XingZ6012,
XingZ6011.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## hulicaijia31 read, 2026-09-26: class one hundred forty-nine not exercised, and the en_CA scan corrected

hulicaijia31 (frozen `eac0b67a8`) SETTLED at 09:45 UTC in 19.3 min,
about 2.81 USD by the meters (Bedrock 21.98 to 20.86,
OpenRouter 51.16 to 49.47).
Hyper read dry from the first reading (balance 0,
spent by hulicaijia30) and Synthetic dry throughout,
so no call reached Hyper,
its window never filled,
and the log carries no `window full` or `request window is full` line:
class one hundred forty-nine was NOT exercised live;
its unit guards stand.
The two Synthetic- and Hyper-only seats threw every call (Qwen3.8-27B 610,
glm-5.3 274),
as on every run with both dry.
Lanes:
repair 3.3 min,
translate 3.3 min,
consolidation 6.9 min.
The 19.3 minutes is the run on Bedrock and OpenRouter alone,
not a measurement of the fix.

Every hulicaijia30 check held:
13 headings,
9 markers,
9 definitions,
2 links,
3 Sakura,
front matter equal,
no TA,
no error finish,
no double blank line,
no straight double quote outside tags,
"douhua" seven times,
"licorice" twice,
register clean,
no Han on the page (hulicaijia30 had two characters).
"Jiefangbei" five times against six:
the walking passage now reads "through the streets of Jiefangbei,
along the pedestrian street around eight or nine in the evening",
which renders every clause of 在晚上八九点钟左右的解放碑步行街上走着 without repeating the name.

The en_CA scan in the scratch `check-hulicaijia.mjs` had reported `center`,
`realize` and `mom` on hulicaijia29,
30 and 31.
All three were false positives:
`center` stood only in `text-align: center` style attributes,
`mom` matched the start of "moment",
and `realize` is Canadian spelling,
which takes -ize.
The scan now strips tags,
matches at a word start,
and lists only spellings Canadian usage writes otherwise
(`color`,
`favorite`,
`center`,
`gray`,
`behavior`,
`neighbor`,
`honor`,
`labor`);
a positive control on "the color of the neighborhood" beside a styled `center` reports `color` and `neighbor` only,
and all three pages read clean.

## hulicaijia30 read, 2026-09-26: class one hundred forty-eight quiet, and class one hundred forty-nine

### What the run did

hulicaijia30 (frozen `1f3d85c29`) SETTLED at 09:12 UTC in 100.6 min against hulicaijia29's 85.0,
about 2.38 USD by the meters (Bedrock 23.14 to 21.98,
OpenRouter 52.40 to 51.18)
plus Hyper's whole balance (208 to 0).
Synthetic read dry throughout;
Hyper read wet from the launch at 07:31 UTC and dry by 09:07 UTC.

The lanes,
by the `SLICE-START` and `SLICE-COST` lines (scratch `lane-times.mjs`):
repair 15.7 min (hulicaijia29 13.7),
translate 43.6 min (25.7),
consolidation 22.1 min (34.7).
The translate lane's p50 slice was 68 s,
but eight slices took 1,590 to 2,067 s.

### The checks

Class one hundred forty-eight never fired:
no `ran past the card's stream bound` line,
and Bedrock's Gemma streams stayed healthy
(`google.gemma-4-e2b` 704 streams at 1.5 s mean,
4 s max;
`google.gemma-4-26b-a4b` 670 at 1.6 s mean,
5 s max).
Class one hundred forty-four was NOT exercised again:
no accepted `accuracy/addition` claim on chunk 69.
The en_CA scan reads the same three US spellings as hulicaijia29 (`center`,
`realize`,
`mom`).

### Class one hundred forty-nine: a full Hyper request window stalled every Hyper call behind one sleep

Hyper allows 1,000 request starts in a rolling hour (`HYPER_REQUESTS_PER_HOUR`,
`request-pace.ts`).
The pass spent them in its first 33 minutes:
919 Hyper completions plus refusals and retries by 08:04:20 UTC,
when the pacer logged `window full (1000 starts in 3600000ms); waiting 1640012ms`.
That sleep was one chained promise every later Hyper caller queued behind,
and the deadline signal could not cut it,
so the log carries eleven `CallTimeoutError` warnings,
all at 08:31:40.8 UTC,
0.1 s after the 1,640,012 ms ran out,
and the pacer then logged waits of 1 ms to 36 s as the window's oldest starts aged out.

Meanwhile the router kept sending Hyper the models OpenRouter also serves,
because the meter read wet and nothing else counted as Hyper's capacity.
On the wire (scratch `stream-times.mjs`),
minimax-m3 averaged 21.6 s a stream on Hyper against 6.0 s on OpenRouter,
and DeepSeek V4.1 Flash 15.1 s against 13.9 s.
The eight translate slices that stalled 27 to 34 minutes are the calls parked behind the window.

Fixed in `eac0b67a8`,
guards red first `02c064371`:

- `request-pace.ts`:
  `take` reserves each start at once,
  in arrival order,
  and waits on its own abortable timer (`setTimeout` from `node:timers/promises` with the caller's signal),
  so an abandoned caller releases its place at once and no caller waits behind another's sleep.
  `waitMs()` says how long a take would wait now.
- `pace-saturation.ts`:
  a provider whose window would make the call wait reads saturated,
  the same state as a per-model slot limit taken.
  `routeProviderFor` already overflows a saturated provider to the next usable one
  and queues on it only when no usable provider stands behind,
  so a model Hyper alone serves (glm-5.3) still waits its turn there.
  Any wait above zero counts;
  no threshold is chosen.
  Each overflow logs `<model>: hyper's request window is full for another <ms>ms; the call goes to <provider>`.
- `provider-router.ts` takes `paces`,
  `hyper-client.ts` exposes `pace.waitMs`,
  and `corpus-run/run-config.ts` wires Hyper's pacer into the routing client.

Guards:
`request-pace.unit.test.ts` (a caller that gives up while another take waits is released at once with its own reason;
`waitMs` reads zero,
the remaining window,
then zero),
`provider-router-pace.unit.test.ts` (minimax-m3 goes to OpenRouter while Hyper's window is full,
stays on Hyper while it has room,
and glm-5.3 queues on Hyper either way).
Lint 0/0,
types clean,
full suite green (`suite-class149.log`,
1151 PASS,
0 FAIL).
hulicaijia31 launched 09:25 UTC on `.frozen-dist-eac0b67a8` (pid 669977,
scope `pass-hulicaijia31`).

## hulicaijia29 read, 2026-09-26: class one hundred forty-four not exercised, and class one hundred forty-eight

### What the run did

hulicaijia29 (frozen `f31fd2ec5`) SETTLED at 07:09 UTC in 85.0 min against hulicaijia28's 24.4,
about 3.06 USD by the meters (Bedrock 24.39 to 23.14,
OpenRouter 54.22 to 52.41).
Synthetic ran dry at 05:56 UTC and Hyper read dry until 06:55 UTC,
so glm-5.3 threw 284 of 337 calls and Qwen3.8-27B 465 of 601 as unreachable.
72 slices,
115 repair issues,
29 accepted,
56 slices where the lanes differed,
41 slices changed on the page.

### The checks

Every hulicaijia28 check held:
13 headings,
9 footnote markers,
9 definitions,
2 links,
3 Sakura,
front matter equal,
no TA,
no error finish,
no double blank line,
"douhua" seven times,
"Jiefangbei" six times,
"licorice" twice.
The only straight single quotes on the page are the seven JSX photo paths,
as the archive has them,
so class one hundred forty-seven has nothing to curl here.
Footnote 7's wordplay note is on the page with 烷 corrected to wán.

Class one hundred forty-four was NOT exercised:
no issue was filed on chunk 69 this run,
so nothing disputed the archive and no dispute block reached the slice 69 slate.
The slate kept the archive's note 3 to 2
(minimax-m3,
google.gemma-4-e2b and Mercury 2.5 citing the page's apparatus;
deepseek-v4.1-flash and gemma-4-26b-a4b-it calling it an import from the ARCHIVE note).

### Class one hundred forty-eight: a Bedrock Gemma stream held in the provider's queue holds its round

Between 06:05 and 06:55 UTC Bedrock's `google.gemma-4-e2b` went from a 1 s mean stream to 235,
263 and 219 s
in three of the five ten-minute windows,
while `google.gemma-4-26b-a4b` on the same provider stayed at 1 s
and every other model kept its own pace.
The streams carry the same signature every time:
a first frame at about 15 s and a frame every 15 s after it,
then content trickling in (70 to 590 content characters over 70 to 305 s),
or nothing at all.
24 calls ran to the 360 s deadline.

With Synthetic and Hyper dry,
every bench leaned on the reachable seats,
and a round of five seats with two unreachable needed Gemma E2B's ballot for its quorum of three:
the slowest rounds read "3/5 heard,
363791ms to quorum".
Across the whole pipeline the ten-minute stream count fell from 220 a model to 7 to 21 while it lasted.
Hyper read wet at 06:55 UTC,
the same minute the episode ends in this log,
and quorum stopped needing the seat.

The same signature appears in other runs,
rarely:
24 of 68,235 Bedrock Gemma streams across the 200 newest pass logs,
against 79 in hulicaijia29 alone.
Without it,
no Bedrock Gemma completion in those logs took longer than 27,340 ms
(over every completion,
signature or not:
p50 1.4 to 1.5 s,
p99 4.2 to 4.5 s);
with it,
completions ran 25 to 304 s.
No other model separates this way:
DeepSeek V4.1 Flash,
GLM-5.3-Flash,
Kimi-K3 and Qwen3.8-27B complete
healthy streams at 200 to 350 s,
so a bound of this kind is per model,
set from the model's own measurement.

## XingZ6013 read, 2026-09-26: classes 139 to 142 live, and classes 145 to 147

### What the run did

XingZ6013 (frozen `e387be2cf`) SETTLED at 04:32 UTC in 56.3 min,
Bedrock and OpenRouter wet,
Synthetic and Hyper dry,
so glm-5.3 (602 asked) and Qwen3.8-27B (1,138 asked) threw every call.
120 slices,
245 repair issues,
149 accepted,
110 slices where the lanes differed,
111 lane contest verdicts,
100 slices changed on the page.

### The four checks the run was launched for

- Class one hundred thirty-nine:
  both song credits keep the album,
  "— Yuli 【妄想症Paranoia】“Nonuple Reality”" and "—— Yuli 【Paranoia】 “Ling Chong Qi Yuan”".
- Class one hundred forty:
  the first credit reads "by Jiecheng Tianzou Official",
  no ",
  author".
- Class one hundred forty-one:
  no English title stands in 《》 anywhere on the page.
- Class one hundred forty-two:
  every straight double quote on the page is inside a tag or the JSX photo path,
  and "take it slow" is curled;
  the four `quote-style-unified` lines curled slices 20,
  39,
  42 and 65.

### Class one hundred forty-five: the first credit's title in two English renderings

The page reads "——from “Gilded Cage” ([Bird in a Cage](https://www.bilibili.com/video/BV17Q4y1s7j1))[^9]".
The original credits the song once,
as the link's text:
`来自《[笼中之鸟](url)》`.
The consolidated candidate wrote "Gilded Cage",
a title the web lookup offered for another work
(minimax-m3 said so on the slate and the gate,
inception/mercury-2.5 on the gate),
and kept 笼中之鸟 as the link's text in parentheses.
The class ninety-eight floor read the Han in parentheses as a gloss beside its English and let the candidate through;
the slate chose it 2.5 to 0.5,
and the gate confirmed it with 2 of 4 ballots,
one for the standing and one for neither.
The class one hundred title pass then rewrote the Han link text into the heading's "Bird in a Cage",
so one song reached the page under two titles.
Fixed in `9157004d2` (guard red first `dd88fd5b6`):
a Han title standing as a link's whole text is never a gloss,
since the link's text is what the reader sees as the work's name.
Replayed over the run's shipped slice 95:
the floor now refuses it.

### Class one hundred forty-six: the second credit left in pinyin

The page reads "—— Yuli 【Paranoia】 “Ling Chong Qi Yuan”" under the heading "Zero-Layer Prayer".
The title pass logged `title-reference-ambiguous (slice 101 ... the slice offers more than one span to read)`:
the slice carries the summary's quoted poem title,
the quoted closing line and the quoted credit.
Fixed in `9157004d2` with `d8d86b531` (same guard commit):
where the original writes the title on a line opening with a tag,
the search is held to the page line opening with the same tag in the same place among such lines,
as it already was for a footnote definition's label.
Replayed over the run's artifact:
slice 101's credit unifies to “Zero-Layer Prayer”.

### Class one hundred forty-seven: a nested quotation left straight

The butterfly speech quotes a law inside its own quotation,
「醒着就要活下去」 inside 「…」,
and the page shipped "the law that says 'to be awake is to keep living.'
They can fly" straight inside curly doubles.
`restoreTypography` curled a straight single quote only as an apostrophe.
The archive pages write nested quotations curly (13 ‘ marks over 6 pages) and never straight.
Fixed in `f4adc4c9f` and `f31fd2ec5` (guard red first `09758edd6`):
`nestedSinglePairs` in `nested-single-quotes.ts` pairs an opening-shaped straight single quote with the next closing-shaped one on its line,
and on a page whose convention is the curly apostrophe the pair is written ‘…’;
the neighbour helpers moved to `quote-neighbours.ts`.
Two older assertions that left a quoted phrase straight,
so it was never half-curled,
now expect the whole pair curled.
Replayed over the whole XingZ6013 page:
that one line is the only line the restoration changes.

### Observations not built

- The two credits write the album two ways,
  【妄想症Paranoia】 and 【Paranoia】,
  and the dash two ways,
  "—" and "——" (em-dashes left per the owner).
- "Yuli" carries no literal gloss at its first appearance;
  the bench wrote none,
  so `placeHandleGlosses` had nothing to move.

### Next

hulicaijia29 on the frozen build carrying classes one hundred forty-four to forty-seven,
read first for the chunk 69 claim's category:
only an accepted `accuracy/addition` claim disputes the archive.

## hulicaijia28 read, 2026-09-26: the note kept, class one hundred forty-four not exercised

### What the run did

hulicaijia28 (frozen `e387be2cf`) SETTLED at 03:33 UTC in 24.4 min,
Bedrock 28.23 to 27.12 and OpenRouter 60.07 to 58.35 by the meters (about 2.83 USD),
Qwen3.8-27B and glm-5.3 dark with Synthetic and Hyper dry.
Every hulicaijia27 check held:
13 headings,
9 markers,
9 definitions,
2 links,
3 Sakura,
front matter equal,
"douhua" eight times,
"Jiefangbei" six times,
"April 29",
"May 4",
"licorice" twice,
no "pharmaceutical sales representative",
no TA,
no double blank line,
no error finish,
the fourteen straight double quotes the JSX photo paths,
the "center" hits the captions' `text-align` values,
no register hit (OD,
jk,
Gaokao).
The page's only Han is inside footnote 7,
the note's own 晚安 and 金刚烷胺.

### Footnote 7

The page ships the archive's note:
"In Chinese,
“goodnight” (晚安,
wǎn’ān) is a near-homophone of amantadine (金刚烷胺,
jīn gāng wán’àn),
the drug mentioned earlier.
The word here carries both meanings."
The pinyin of 烷 reads wán where the archive and both lane texts read wǎn,
so `correctPinyinPage` (class one hundred thirty-seven) acted on it.

### Why this does not show class one hundred forty-four live

gemma-4-26b-a4b-it filed the note as `accuracy/mistranslation major`,
"The translation provides an expansion regarding homophones and linguistic meanings that is not present in the original text."
The panel accepted it 3 to 1:
minimax-m3,
gemma-4-26b-a4b-it and google.gemma-4-e2b supported,
deepseek-v4.1-flash did not.
`archiveDisputesOf` disputes the archive only on an accepted `accuracy/addition` claim,
so no dispute was logged for slice 69 and no sheet carried the dispute block the class one hundred forty-four fix bounds.
The repair lane kept the note unchanged despite the accepted claim,
the translate slate chose the archive's note 3 to 2 without the dispute block
(deepseek-v4.1-flash,
google.gemma-4-e2b and inception/mercury-2.5 for it,
two ballots citing the apparatus rule;
gemma-4-26b-a4b-it for its own short note,
calling the archive's "a restatement of the 'ARCHIVE note'";
typesafe/jev-1.13 for the short note at 0.79),
and the lanes agreed,
so slice 69 had no contest.
The page is right,
but the dispute path is untested live;
the unit guard `7869828fb` still stands for it.

### The panel on the note, three runs

The same three panelists have accepted a claim against the note on each run where it was filed:
minimax-m3 and gemma-4-26b-a4b-it on hulicaijia26,
27 and 28,
google.gemma-4-e2b on 27 and 28,
with the critic and panel sheets naming `TRANSLATOR_NOTE_KIND` as apparatus since class one hundred forty-three.
It is one item asked three times with the same sheet,
not three items,
so no model is reseated on it;
it is the evidence to weigh if a panel measurement across items ever shows the same seats below the standard.

### Lane contest attribution

The artifact's `laneSelection` carries 223 `modelId` keys,
one per "lane contest ballot by" line in the log (223),
so every contest ballot now names the model that cast it.

## Class one hundred forty-four, 2026-09-26: one panel's acceptance made final on every later sheet

### What hulicaijia27 showed

hulicaijia27 (frozen `2472ec48e`) SETTLED at 03:02 UTC in 45.3 min,
Bedrock 29.49 to 28.26 and OpenRouter 62.01 to 60.14 by the meters (about 3.10 USD),
glm-5.3 dark.
Every hulicaijia26 check held:
13 headings,
9 markers,
9 definitions,
2 links,
3 Sakura,
"douhua" eight times and no "tofu pudding",
"Jiefangbei" six times,
no TA,
no double blank line,
no Han,
the six "center" hits the photo captions' `text-align` values,
the fourteen straight double quotes the seven JSX photo paths.
Class one hundred forty-three was NOT live:
footnote 7 again reads "That is,
the amantadine mentioned earlier."
The archive's note ("In Chinese,
“goodnight” (晚安,
wǎn’ān) is a near-homophone of amantadine …") is lost a second time.

### Who judged it, by name

The ballot attribution built for class one hundred forty-three answered the owner's question on this run.
There was no lane contest on slice 69:
both lanes shipped the short note,
so the lanes agreed and no ballot was cast.
The note fell twice,
once per lane.

The repair lane's panel on chunk 69 accepted the addition claim 3 to 2.
deepseek-v4.1-flash filed "The translation of footnote 7 adds the full homophone explanation from the ARCHIVE note",
and gemma-4-26b-a4b-it filed,
in the same issue,
"The translation replaces a simple reference to amantadine
with an explanation of a homophone that does not exist in the source text" (accuracy/mistranslation).
minimax-m3,
gemma-4-26b-a4b-it and google.gemma-4-e2b voted supported;
deepseek-v4.1-flash (against its own claim) and inception/mercury-2.5 voted unsupported.
Every one of the three had the class one hundred forty-three wording on its sheet.

The translate lane's slate on slice 69 chose the short note 4 to 0:
google.gemma-4-e2b (self-vote),
inception/mercury-2.5,
typesafe/jev-1.13 and minimax-m3 (self-vote).
minimax-m3's reason cites the repair panel:
"the full homophone explanation from the ARCHIVE note,
which the repair lane's adjudicators accepted as an addition";
mercury-2.5 wrote "an addition not supported by the ORIGINAL footnote alone".
The class one hundred eight ARCHIVE RENDERING DISPUTED block had told them exactly that:
"A detail those claims name is not page content and not the page's apparatus … Judge such a detail against the ORIGINAL alone."

### Why no model was reseated

No measurement on record puts any of the three supporting panelists below a seated band as a panelist,
and the translate slate followed the instruction its sheet gave.
One wrong vote on one note is not the evidence the owner's authorization asks for.

### What was built

The fault is structural:
the dispute block made one panel's verdict final on every later sheet,
whatever the claim named,
so a 3 to 2 vote on a pun note decided both lanes.
Class one hundred eight built that block for an invented medication event,
which is narrative detail,
and class one hundred forty-three has since stated that bound on its own
(`NARRATIVE_DETAIL_IS_NOT_APPARATUS`).
`archiveDisputeNote` in `archive-dispute.ts` now states the narrative bound,
keeps its any-wording rule for a detail that says what happened,
and sends a claim naming only a gloss or a translator's note (`TRANSLATOR_NOTE_KIND`) back to the page-apparatus rule,
"as if no claim named it".
The dispute itself stands (the owner's "Not eligible;
fall back to the repair text"),
so the stand-in is still the repair text;
what changes is that every later judge may keep the note.
Guard red first `7869828fb` (`translator-note-apparatus.unit.test.ts`),
fix `e387be2cf`,
lint 0/0,
types clean,
full suite `suite-class144.log` 1144 PASS,
no FAIL.

## Class one hundred forty-three, 2026-09-26: a translator note on wordplay voted out as an addition, and who voted

### What hulicaijia26 showed

hulicaijia26 (frozen `0e0b05d0f`) SETTLED at 02:00 UTC in about 15 min,
Bedrock 30.63 to 29.51 and OpenRouter 63.86 to 62.16 by the meters (about 2.82 USD),
Synthetic and Hyper dry,
Qwen3.8-27B and glm-5.3 dark for want of a provider.
Every hulicaijia20 check held:
front matter equal,
13 headings,
9 markers,
9 definitions,
2 links,
3 Sakura,
no TA,
no double blank line,
"douhua" seven times and no "tofu pudding" (class one hundred thirty-five),
"Jiefangbei" on both lines,
register and en_CA clean
(the six "center" hits are CSS `text-align` values in the photo captions),
no English title left in 《》,
no Han outside markup,
the only straight quotes the seven JSX photo paths.
The one loss:
footnote 7 shipped as "the amantadine mentioned earlier" alone.
The archive's note had explained the wordplay between 晚安 (goodnight) and 金刚烷胺 (amantadine),
which the English reader cannot see.

### Who judged it

The owner asked who the judges were.
The artifact's repair chunk 69 records the panel:
deepseek-v4.1-flash filed the note as accuracy/addition,
minimax-m3 and gemma-4-26b-a4b-it voted supported,
Mercury 2.5 unsupported,
so it was accepted.
That acceptance disputed the archive under class one hundred seven,
and the class one hundred eight ARCHIVE RENDERING DISPUTED block told the contest the note was an accepted addition;
four of five contest ballots then called it "an addition the ORIGINAL never states".
The contest ballots carried no model id,
so those four judges cannot be named for this run.

### What was built

The ballots now name their judges (owner:
"that's an our problem").
`LaneContestBallot.modelId` is recorded by the stage from each round outcome,
read back by the artifact reader and the contest cache (optional,
so older artifacts still read),
and the stage logs one line per ballot,
"lane contest ballot by <model>:
chose …,
archive …,
unsupported […],
dropped […]: <reason>",
under a logger tagged with the slice.
Guards red first `ef0c6befd`,
fix `3fe12ce50`.
The adjudicated-issue log line names the panel beside the filers,
"<claim> filed by X,
panel A supported,
B unsupported",
or "panel not on record" (`claim-panel-voters.ts`);
guard red first `d994a7377`,
fix `e46f8ca70`,
full suite 1143 PASS.

Class one hundred forty-three (`page-apparatus-clause.ts`):
`TRANSLATOR_NOTE_KIND`,
a translator's note explaining a pun,
wordplay,
an allusion or a term the English cannot carry by itself,
now sits on the critic sheet's and the panel's lists of accurate translator detail,
in `CONTEST_POLICY`'s exemption and in `PAGE_APPARATUS_IS_KEPT`.
The four sheets had carried four lists,
and none named such a note.
`NARRATIVE_DETAIL_IS_NOT_APPARATUS` ("WHAT HAPPENED IS NEVER APPARATUS") replaces the apparatus clause's
"Only what the ORIGINAL contradicts is an addition" and follows the contest exemption,
since that wording is what a class one hundred eight gate ballot used to keep an invented overdose detail.
Guard red first `4520367f0` (`translator-note-apparatus.unit.test.ts`),
fix `2472ec48e`,
lint 0/0,
types clean,
full suite `suite-class143.log` 1144 PASS,
no FAIL.
A corpus census found this note on one entry,
so no deterministic floor was built;
the wording carries it.

## Class one hundred forty-two, 2026-09-26: straight quotes on archive paragraphs of a page written curly

### What the page showed

XingZ6012 wrote its quotes curly (134 curly double marks and 108 curly apostrophes),
yet two lines kept straight double quotes and two kept straight apostrophes:
`"When a person dies, it is like water disappearing into water."`,
`"take it slow"`,
`it's the standard narrative` and `my body doesn't want to die.`
They stood on archive paragraphs no lane rewrote,
and the typography restoration reads only the text a lane replaced.
Of the other latest read pages only XingZ60 mixed the two forms;
shi_Yumiaoya's straight quotes are JSX string literals and must stay straight.

### What was built

`corpus-run/quote-style-unify.ts` `unifyQuoteStyle` is a page-assembly pass after the pinyin pass
and before the footnote guard.
It counts the prose quote marks on every slice outside the front matter,
through the same `proseMask` the restoration uses (tags and code spans masked,
read by UTF-16 unit),
and where the curly form of a mark outnumbers its straight form it curls that mark on every slice
through `restoreTypography`,
so tags,
code,
unbalanced doubles and the ellipsis stay as they are.
Doubles and apostrophes are decided apart.
Sealed English-original spans and the front matter stand aside,
as for every whole-page pass;
each slice changed prints a `quote-style-unified` finding with the count of marks curled.
Guard red first `c5a03ea2c` (`quote-style-unify.unit.test.ts`:
curls an archive slice on a curly page,
leaves a straight-majority page,
leaves tag attributes and code spans straight),
fix `0e0b05d0f`,
lint 0/0,
types clean,
full suite `suite-class142.log` 1142 PASS with one FAIL,
the load-sensitive grace-window case of `lane-contest-stage.unit.test.ts` (seen before under class one hundred ten),
which passes alone.
Replayed over seven read pages and every archive page:
XingZ6012 curled on four blocks (the four lines "What the page showed" names),
the lin10104 and shihai4h archives one apostrophe each ("it’s",
"doesn’t"),
nothing else changed,
no JSX or code touched.

### The pass the build killed

hulicaijia25 (frozen `8c8c7007b`,
pid 369265) was killed under always-kill-and-relaunch.
hulicaijia26 runs on `.frozen-dist-0e0b05d0f` (pid 524784,
scope `pass-hulicaijia26`).

## Class one hundred forty-one, 2026-09-26: an English title left in the Chinese title marks 《》

### What the page showed

XingZ6012's second song credit shipped
`<p style="text-align: end;">— Yuli【妄想症Paranoia】《Zero-Layer Prayer》</p>`:
the title rendered,
the Chinese title marks kept around it.
《》 mean nothing in English prose,
where a work's title stands in quotation marks,
the way the page's other credit reads (——from “Bird in a Cage”).
A census of the read pages found 《Maybe All Transgender People Are Really Vampires》 twenty times,
《Nonuple Reality》 four times,
《Zero-Layer Prayer》 three times and 《Ling Chong Qi Yuan》 once.
Of the pinned archive pages,
only XingZ60 sets an English-only title in 《》 in English prose
(the Vampires line,
as the original writes it);
the other two hits (AmbeR_the_anpa,
shihai4h) are untranslated Chinese lines whose title carries Han.

### What was built

`translate-latin-title.ts` `latinTitleFindings` refuses before any judge a candidate that sets a title
carrying Latin letters and no Han in 《》,
the link text read where the marks hold a link,
unless the page the candidate would replace writes the same bracketed title.
It runs in `sourceCarryFindings` right after the Han title floor of class ninety-eight.
A title that keeps Han (《舞萌DX》) is left to that floor and the judges;
the bilingual album 【妄想症Paranoia】 stays with the judges as class ninety-eight decided.
The title bullet of the house policy now says a title stands in quotation marks in English prose,
never in 《》.
Guard red first `2999b2a45` (`translate-latin-title.unit.test.ts`,
five cases),
fix `8c8c7007b`,
lint 0/0,
types clean,
full suite `suite-class141b.log` 1142 PASS with no FAIL.
Replayed over the real credit line:
the shipped line is refused with the finding,
the same line with “Zero-Layer Prayer” passes every floor.

### The pass the build killed

hulicaijia24 (frozen `5cb874a36`,
pid 232490) was killed about ten minutes in under always-kill-and-relaunch,
since the policy text reaches every sheet.
hulicaijia25 runs on `.frozen-dist-8c8c7007b` (pid 369265,
scope `pass-hulicaijia25`).

## XingZ6012 read, 2026-09-26: a bracketed album read into the signer's name, and a credit's 作者 as "author" (classes one hundred thirty-nine and one hundred forty)

### What ran

XingZ6012 on frozen `c49ff687e`:
SETTLED at 01:04 UTC in 32.0 min,
one attempt,
120 slices,
93 changed on the page.
Qwen3.8-27B (1,166 asked) and glm-5.3 (567 asked) sat dark all run,
every call refused by the router before it went out
("every provider serving this model is out of budget"):
both seats are served only by Synthetic or Hyper,
and both were dry,
which is the seating's design.

### What the page shows

Class one hundred thirty-eight is live:
the second song credit keeps its bilibili link and `[^9]`,
no `contributor-name-restored` line names "Laizi",
and the page carries both inline links and nine definitions.
The en_CA pass rewrote "center" and "behavior" three times (`canadian-form-rewritten` on slices 13,
20 and 21).
Five `contributor-name-restored` lines,
eight `heading-series-unified`,
one `title-reference-unified`;
165 abstention lines.

Two things read wrong,
each now a class.

-   Slice 98's credit shipped as "—— Yuli,
    “Nonuple Reality”".
    The bench had written "—— Yuli [妄想症Paranoia],
    “Nonuple Reality”" for 「—— 雨狸【妄想症Paranoia】《九重现实》」,
    the album in ASCII brackets.
    `readSignature` ended the page-side name at the comma,
    so the name read "Yuli [妄想症Paranoia]",
    and the contributor-name restore replaced name and album with the bare reading,
    deleting the album (class one hundred thirty-nine).
-   The first song credit shipped as "——from “Bird in a Cage” (…)[^9],
    author Jiecheng Tianzou Official":
    the 作者 of 「，作者 洁澄天奏Official」 rendered word for word,
    where XingZ6011 and XingZ623 wrote "by" (class one hundred forty).
    The lowercase "from" after the dash is an ordinary English attribution style and was left.

### What was built

Class one hundred thirty-nine (`7292556b7`,
guard red first in the class one hundred thirty-eight test file):
`attribution-line.ts` ends a name at an ASCII `[` found past the name's first character
(at the first character the bracket opens a linked name).
Types clean,
suite `suite-class139.log` 1140 PASS,
no FAIL
(the new case joins an existing describe block,
so the count holds);
replayed over the real slice 98 line:
no finding,
the line unchanged.

Class one hundred forty (`rendering-glossary-credit.unit.test.ts` red first,
fix in `rendering-glossary-wording.ts`):
the rendering glossary seeds the credit form `，作者` with "by" and refuses ",
author ".
A census of the pinned originals found 作者 on three pages:
this credit,
the poem credit 「作者：Chrysanthemum Tran」 on the same page (no comma,
not read),
the LCG_Akiball footnote 「原文如此，作者可能想指…」 (subject "the author",
which never renders ",
author "),
and 社会工作者 on GLaDOSister,
which the comma keeps out.

### The pass the build killed

hulicaijia21 (frozen `7292556b7`) died silently at 01:19 UTC,
five minutes in,
with no line in its log.
systemd-oomd killed the whole `app.slice/claude-code-bash` cgroup at 01:19:47 UTC
(21:19:47 EDT in the journal,
"memory pressure for …/app.slice being 86.54% > 80.00% for > 20s"),
the cgroup every Bash command runs in,
while `mise run build` ran beside the pass
and 33 GiB of swap was already in use.
A pass launched from a Bash command lives in that cgroup and dies with it.
hulicaijia23 was launched through `systemd-run --user --scope --unit=pass-<run>`,
which puts the pass in its own scope (`/proc/<pid>/cgroup` reads `app.slice/pass-hulicaijia23.scope`);
launch every pass that way.
hulicaijia23 was killed a few minutes in under always-kill-and-relaunch once class one hundred forty landed,
and hulicaijia24 runs on `.frozen-dist-5cb874a36` (pid 232490,
`pass-hulicaijia24.scope`).

## XingZ6011 read, 2026-09-26: a source credit read as a signature (class one hundred thirty-eight)

### What ran

XingZ6011 on frozen `2d5113b7d` (pid 3762102,
launched 22:50 UTC 2026-09-25):
INCOMPLETE at 00:26 UTC in 96.3 min,
one attempt,
no page or artifact:
"entry XingZ60 would drop 1 source destination(s)".

### What went wrong

The original credits the second song as
「——来自《[笼中之鸟](https://www.bilibili.com/video/BV17Q4y1s7j1)》[^9]，作者 洁澄天奏Official」.
`readSignature` in `corpus-run/attribution-line.ts` ends a signer's name at 《 (class eighty-three),
so it read 来自 ("from") as the signer.
With no archive rendering for the line,
the pinyin reading "Laizi" stood as the authority,
and the contributor-name restore rewrote the page's whole credit:

`——From “A Bird in a Cage” ([笼中之鸟](…))[^9], by Jiecheng Tianzou Official` to `——Laizi, by Jiecheng Tianzou Official`.

The link and the `[^9]` marker went with it,
the page-assembly guard reverted definition 9 as an orphan (`assembly-footnote-reverted orphan-definition gfm 9`),
and the publish check found the bilibili destination missing.
The bench's own credit was right.

### What was built

Class one hundred thirty-eight (`c49ff687e`,
guard red first `bffae3bce`):
`readSignature` refuses a name that is a source-credit lead (来自,
and 摘自,
出自,
选自,
引自,
转自,
节选自,
which say the same).
A census of the originals found one em-dash line naming a work after something other than a signer's name:
this one.
Lint 0/0,
types clean,
suite `suite-class138.log` 1140 PASS,
no FAIL;
replayed over the real slice 95 credit:
no rewrite.
A second guard,
a name carrying link or marker syntax,
could not be made to fail on the current build
(a page signature with no Han takes itself as the authority),
so it was not built.
