# OpenRouter and the corpus passes: 2026-09-22, 04:30 to 13:40 UTC

What was launched,
read and decided on 2026-09-22 04:30 to 13:40 UTC.
Entries:
XingZ622,
hulicaijia10,
XingZ621,
mikaela7,
shi_Yumiaoya13,
shi_Yumiaoya12,
shi_Yumiaoya11.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## XingZ622 read, 2026-09-22, 13:40 UTC: SETTLED in 3h39m, class eighty-one live on the page ("They once said"), class eighty-two not exercised; the undeclared-handle question open; hulicaijia10 launched

`XingZ622` (frozen `6c701485d`,
pid 1996041,
overlap 8,
launched 09:30 UTC on Bedrock and OpenRouter,
Synthetic dry after 33 calls with its five-hour window spent,
Hyper dry)
SETTLED at 13:09 UTC on one attempt,
3h39m by the tally against `XingZ620`'s 3h59m and the seven-hour deadline.
Seats asked 8,504,
297 retry rounds,
7,778 calls,
88 abandoned streams (0.36 USD),
Jev 249 of 249,
glm-5.3 dark (582 asked,
582 thrown),
Qwen3.8-27B unusable on the naturalness bench.
Meters 112.15 to 107.68 USD on Bedrock and 165.84 to 157.77 on OpenRouter,
12.54 USD together against `XingZ620`'s 10.24.
Phases:
repair lane 09:44 to 10:22 UTC (38 min against `XingZ620`'s 78),
translate lane 10:40 to 11:28 UTC (48 against 41),
contest to 11:34 UTC,
consolidation 11:34 to 13:06 UTC (92 against 69,
110 contested slices at a median of 6.4 minutes a chunk against 4.0),
page written 13:09 UTC.

### Why the consolidation ran slower than XingZ620's

The consolidation is bound by the two slow write-path streams and this run had no Synthetic or Hyper to spread them onto.
`deepseek-v4.1-flash` streams took 48.8 s mean in the consolidation window against 31.9 on `XingZ620`
(458 streams,
372 stream-minutes;
all 1,270 of its calls through OpenRouter where `XingZ620` sent 177 to Hyper),
`GLM-5.3-Flash` 96.7 s against 88.2
(133 streams,
214 stream-minutes;
414 calls through OpenRouter and 33 through Synthetic where `XingZ620` sent 132 to Synthetic).
Rounds waited on them:
produce rounds 108 s mean against 70,
naturalness review 53 s against 27 (grace 9.9 s against 3.5),
slate judging 41.5 s against 33.4.
Not a stall and not a class:
the same roster shape on the same providers,
with the fast providers dry.

### Class eighty-one on the page

Slice 14 (the tombstone paragraph,
TA 曾说) settled at 11:39 UTC on its first slate round:
the contest settled on neither,
the archive's TA text stood and was withheld (`ineligible-standing-withheld`),
the translate lane text was offered beside five valid proposals,
and the slate chose deepseek's "They once said that when a person dies,
all their right and wrong..." at weight 2.5
(gpt-oss,
gemma-26b and deepseek on it,
deepseek's ballot "correctly renders the written TA as singular they";
minimax abstained reading the identity line's three forms as a fault of the candidate),
the gate 3 of 4 for the consolidated text over the standing
("Standing is disqualified by the house rule against leaving 'TA' untranslated").
The identity line reached the sheets as
`ORIGINAL refers to this person as 她 (51 times), and also writes 他 (19 times) and TA (35 times)`,
quoted in a judge's ballot on the inheritance paragraph.
On the page:
she/her 150 to 99 and they/them/their 74 to 111 against `XingZ620`,
65 lines newly carrying they,
among them "They once said that when a person dies",
"They said they had at least three personalities",
"no personal pronoun seems quite right for addressing them",
"This is also reflected in their last note";
the 她 passages keep she ("after her death her friends were all deep in grief").
No `run-off tied again` line and no `translate-runoff` finding:
no slate tied this run,
so class eighty-two stays unit-covered only.

### The seven steps and three checks

- Fourteen headings (Introduction,
  Engagement in Trans Aid,
  Memories by Friends,
  Part One to Part Ten,
  the closing section),
  both links,
  nine definitions with nine markers,
  three TextRing,
  all eleven details blocks,
  front matter equal to the archive,
  no TA,
  no error finish,
  `attempts.json` reads 1.
- The closing section "To the Shooting Star That Once Streaked Across the Night Sky" in the second person as the original;
  the poem quotes with their attributions inside the details blocks;
  the Caged Bird attribution with its link ("—From *Caged Bird*,
  by 洁澄天奏Official").
- Classes sixty-seven to sixty-nine and seventy-six holding:
  Ann,
  HiYku,
  Baimao Suki restored;
  ten headings unified as Part One to Part Ten;
  the memorial's four paragraphs on the page;
  the life in the past tense.
- Three standings shipped without contest endorsement (slices 6,
  42,
  68),
  no gate-kept,
  208 abstention lines with reasons,
  no repeat finding,
  one `archive-revision-refused` (minimax's paragraph-for-paragraph revision refused on shape at the block review).

### Open: an undeclared handle has no rule and the bench renders it three ways on one page

The page carries handles the archive never rendered
(the archive's tenth section is a placeholder and the poem tail is untranslated there):
锦心 ships as `锦心` in the Part Ten heading and signature
(the consolidation slate 2.5 to 1 for "keeps '锦心' as written ... there is no declared English form",
over the translate lane's "Jinxin";
the contributor restore then rewrote the heading's "Brocade Heart" to the signature's 锦心),
where `XingZ619` wrote "Jinxin",
`XingZ620` "Jin Xin" and `XingZ616` "Brocade Heart";
洁澄天奏 ships as "Jiecheng Tianzou" on its signature and "洁澄天奏Official" on the Caged Bird attribution;
雨狸 ships as "Yu Li" on one song attribution and "雨狸" on the next;
《零重祈愿》 as "Ling Chong Qi Yuan" where `XingZ620` kept the Han.
`XingZ620` had the same shape (螐,
洁澄天奏,
妄想症,
零重祈愿 in Han;
"Yu Li" beside "Yuli").
The house policy has no sentence on a handle with no declared or archive form,
and the judges fill the gap with "keep the original form".
The archive's own practice is split:
it romanises 白毛 suki as "Baimao suki" on this page,
and keeps 方方 and 铃木真依 in Han as Twitter link text on `Anilovr`
(11 of 92 English pages carry Han outside comments,
mostly forms under discussion and quoted verse).
One-way rendering per page is settled (class sixty-seven);
which form an undeclared handle takes is the owner's call,
asked 2026-09-22 with pinyin recommended.

### hulicaijia10 launched

`hulicaijia10` (frozen `6c701485d`,
pid 2018636,
overlap 8)
launched 13:14 UTC on Bedrock and OpenRouter
(meters 107.68 USD Bedrock,
157.75 OpenRouter at launch),
waiter `bo0aplgcp`.
Read by the seven steps and three checks against `hulicaijia9`
(13 headings,
17 tag markers,
18 footnote markers,
9 definitions,
2 links,
3 Sakura,
"Conflict" for 左右,
front matter equal,
no TA,
no error finish),
then `CuspariaKLSY`,
`zheermao`,
`yingying` one at a time.

## XingZ621 read, 2026-09-22, 09:35 UTC: INCOMPLETE at slice 14 on a pronoun rule that contradicts itself (class eighty-one) and a run-off that tied again after narrowing (class eighty-two), both fixed in 6c701485d; XingZ622 launched

`XingZ621` (frozen `93b11aef9`,
pid 1949843,
overlap 8,
launched 06:11 UTC on Bedrock and OpenRouter,
Synthetic wet for part of the run and Hyper dry)
stopped INCOMPLETE at 09:14 UTC on one attempt,
3h02m by the tally against `XingZ620`'s 3h59m.
Seats asked 7,229,
311 retry rounds,
6,043 calls,
23 abandoned streams (0.03 USD),
Qwen3.8-27B and glm-5.3 dark on the OpenRouter side.
Meters 115.74 to 112.16 USD on Bedrock and 171.10 to 165.86 on OpenRouter,
8.82 USD together.
Repair lane from 06:12 UTC to about 07:24 UTC,
contest from 08:45 UTC,
consolidation from 08:52 UTC (113 contested slices),
stop at 09:05 UTC on slice 14 with the tally written at 09:14 UTC.
No page was written,
so the seven steps and three checks against `XingZ620` and the archive wait for `XingZ622`.

### The stop

Slice 14 is the tombstone paragraph of Part One
(唯武则天立乾陵无字碑 ... TA 曾说，人死了 ... 在 TA 的遗书里 ... TA 邀请了).
The contest settled on neither (4 of 5 usable),
so the archive's own paragraph stood,
and the deterministic floor withheld it from the slate for its untranslated "TA".
The slate read eight valid candidates,
six proposals and both lane texts
(the repair lane's "They once said",
the translate lane's "She once said"),
and split one ballot each four ways:
Mercury on candidate 2 ("respects the house rule on pronouns where the original uses the neutral form TA"),
gpt-oss on candidate 3 ("the correct pronoun (she) as required by the declared gender"),
minimax on candidate 6 ("singular they ... the correct rendering of TA per the house rules"),
gemma-26b on candidate 5 ("Candidates 3,
4,
7,
and 8 use she/her,
which violates the rule").
The run-off over the four leaders (class sixty-three) split again:
gemma-26b at 0.5 on the first,
gemma-e2b on the fourth,
Mercury on the third ("correctly resolves the passage's neutral pronoun to 'she',
consistent with the declared identity"),
gpt-oss declining all
("Candidates 1,
2,
and 4 use the plural pronoun 'They' instead of the declared feminine pronoun 'she'").
Tied at weight 1,
`no-candidate-backed`,
`slate-declined-standing`,
and since the standing was the incumbent the entry stopped.

### Class eighty-one: the pronoun rule contradicted itself on a page that writes both 她 and TA

The house rule said "Only where the whole ORIGINAL uses a neutral pronoun or avoids pronouns entirely does the TRANSLATION preserve that choice rather than resolving it to he or she"
two sentences before "Never 'correct' a neutral pronoun into a gendered one",
and the identity line said "ORIGINAL refers to this person as 她 (51 times)" with no mention of the forty TA the original also writes.
XingZ60 is a ten-part page by several hands;
Part One writes TA throughout and discusses the pronoun question itself
(至少有三个人格 ... 一个人格的性别认同为男性，另一个人格的性别认同偏女性),
and the other parts write 她.
A judge reading the count and the "whole ORIGINAL" clause resolved TA to she;
a judge reading the TA sentence rendered it they;
the bench never agreed and the same split ran through every pronoun slice of Part One
(the judges' reasons at 08:57 UTC,
08:58 UTC,
09:01 UTC,
09:03 and 09:05 UTC all name one rule or the other).
`XingZ616`,
`619` and `620` shipped "She once said" for this paragraph,
the same wrong choice by the settled rule,
unnoticed because the "no TA" check looks for the untranslated word only.

Fixed in 6c701485d:

- `house-policy.ts`:
  a pronoun the ORIGINAL writes is rendered as written where it stands
  (她 is she,
  他 is he,
  TA is singular they),
  on every page including one whose other passages write 她,
  "since these pages are written by several hands and a writer who chose TA beside a page of 她 chose it";
  the declared-identity line settles unstated subjects only,
  never a pronoun the ORIGINAL wrote;
  the "whole ORIGINAL" clause now governs the neutral rendering of an UNSTATED subject alone.
- `JUDGE_POLICY_TAIL`:
  "A TA THE ORIGINAL WRITES IS NOT AN UNSTATED SUBJECT:
  it is rendered singular they where it stands,
  whatever pronoun the identity line counts for the rest of the page",
  a candidate resolving a written TA to she or he has made the WRONG choice,
  as has one writing they where the ORIGINAL wrote 她 or 他.
- `sourcePronounLines` (`identity-context.ts`):
  where the original writes more than one form the line names every form with its count and carries the as-written sentence
  (`- pronoun: ORIGINAL refers to this person as "她" (51 times), and also writes "TA" (40 times); each form is rendered as written where it stands (她 is she, 他 is he, TA is singular they), and "她" supplies the pronoun only where the ORIGINAL leaves the subject unstated`);
  a page writing one form keeps the old line.

Guards red first at `4cf95e4e6`
(`house-policy-reaches-the-judges.unit.test.ts`,
`identity-context.unit.test.ts` with the three mixed-form expectations of the dominant-form test updated).

### Class eighty-two: a run-off that tied again stopped the entry although it had narrowed the finalists

The class sixty-three run-off offers the tied leaders once;
a second tie settles the decline.
Here the first tie ranked four of eight,
the run-off ranked two of those four (weight 1 each,
one at 0.5,
one abstention),
and the stage settled `no-candidate-backed` over a question it had just narrowed to two valid renderings.
Fixed in 6c701485d:
`judgeSlateWithRetry` (`translate-retry.ts`) asks the same panel again while each run-off strictly narrows the finalists,
and settles only on a round that decides or a tie that narrows nothing
(the whole slate backed,
fewer than two backed,
or a rejection);
each round shrinks the finalists and a narrowed run-off keeps at least two,
so the slate's width bounds the rounds.
Every round's findings are carried with `translate-declined-retried` and `translate-runoff (finalists N of M)` between them.
Guard red first at `4cf95e4e6`
(`translate-retry.unit.test.ts`:
four renderings,
three disinterested judges,
a tie of three of four,
then two of three,
then a decision at weight 2 on the third round;
the rig now takes its translators,
judges and renderings as parameters).
Lint 0 and 0,
types clean,
full suite green (`suite-class81-82.log`).

### Checks that could be made without a page

- `attempts.json` reads 1;
  no `finish_reason=error`;
  no `PublishedPageDisagrees` or `weight-off`.
- 10 standings shipped without contest endorsement,
  1 gate-kept,
  155 `declined every candidate` lines with reasons,
  no `translate-runoff` finding in the log (the run-off is named in the artifact,
  which was never written).
- The `CORPUS NAMES` line named nobody,
  as the census predicted.
- The admission:
  slice 70 admitted inside the details of slices 67 to 71 on the container deficit (class sixty-one holding),
  no placement unresolved.

### XingZ622 launched

`XingZ622` (frozen `6c701485d`,
pid 1996041,
overlap 8)
launched 09:30 UTC on Bedrock and OpenRouter
(meters 112.15 USD Bedrock,
165.84 OpenRouter at launch),
the `CORPUS NAMES` line naming nobody.
Read slice 14 and the other Part One pronoun slices first
(the identity line's mixed-form sentence in the prompt payloads,
"They once said" on the page,
ballots citing the as-written rule,
any `run-off tied again` line and the `translate-runoff` findings),
then the closing poem (slice 91),
classes sixty-five to sixty-nine and seventy-six,
then the seven steps and three checks against `XingZ620` and the archive
(fourteen headings,
both links,
nine definitions,
3 TextRing,
eleven details blocks,
front matter equal,
no TA,
no error finish);
`XingZ620`'s 3h59m and the seven-hour deadline are the timing marks.

## mikaela7 read, 2026-09-22, 06:15 UTC: SETTLED, classes seventy to seventy-seven holding on the newest build, the pair bound silent on a slice with no pair; XingZ621 launched

`mikaela7` (frozen `93b11aef9`,
pid 1944780,
overlap 8,
launched 05:24 UTC on Bedrock and OpenRouter,
Synthetic and Hyper dry)
SETTLED at 06:10 UTC on one attempt,
47 minutes by the artifact against `mikaela6`'s 40.
Seats asked 2,210,
68 retry rounds,
1,947 calls,
7 abandoned streams (0.0064 USD),
Qwen3.8-27B and glm-5.3 dark (294 and 141 asked,
all threw).
Meters 116.35 to 115.75 USD on Bedrock and 171.95 to 171.12 on OpenRouter,
1.43 USD together.

### Checks

- Front matter equal to the archive and to `mikaela6`.
- 4 of 4 headings,
  4 of 4 HTML comments,
  4 of 4 footnote markers,
  𝓠𝓾𝓪𝓷 seven times as the archive (six on `mikaela6`,
  none as "Quan"),
  304 lines against the previous page's 293;
  no `TA`,
  no `finish_reason=error`,
  no double blank line (the archive itself carries one),
  `attempts.json` reads 1.
- No `PublishedPageDisagrees` or `weight-off`,
  two unendorsed standings that are also slate-declined (slices 2 and 30,
  the archive's wording standing),
  no gate-kept standing,
  no `repeats the line`,
  no run-off,
  no `archive-revision-refused`;
  67 `declined every candidate` lines with reasons.
- 5 em-dashes,
  left per the owner's answer of 2026-09-21.
- Classes seventy to seventy-seven holding:
  the Epilogue's excerpt line curled (`[𝓠𝓾𝓪𝓷](...)'s Chronicle Issue No.33, 2023`),
  "Mika was outed" on the page,
  the Epilogue (slice 28) with its source lines one to one
  (the five quoted lines,
  "This is the origin of XYN (Tianjin) Technology.",
  the excerpt line),
  no repeat finding,
  no double blank line,
  the life in the past.
- The pair bound silent as it should be:
  no "pair is ONE line" finding in `artifacts/mikaela_khara.json`,
  since no slice of this original carries a Han line with its own English beside it.
- The `CORPUS NAMES` line names nobody
  ("the original names nobody another entry declares"),
  as the census predicted for this entry.

No class found.

### XingZ621 launched

`XingZ621` (frozen `93b11aef9`,
pid 1949843,
overlap 8)
launched 06:11 UTC on Bedrock and OpenRouter
(meters 115.74 USD Bedrock,
171.10 OpenRouter at launch;
Synthetic and Hyper dry),
the `CORPUS NAMES` line naming nobody.
Read the verse chunks and the closing poem (slice 91) first
(the attribution inside the quote,
no pair finding in the artifact,
the line floor's shortfall check unchanged),
then classes sixty-five to sixty-nine and seventy-six,
then the seven steps and three checks against `XingZ620` and the archive;
the seven-hour deadline and `XingZ620`'s 3h59m are the timing marks.

## shi_Yumiaoya13 read, 2026-09-22, 05:30 UTC: SETTLED, class eighty live (the block bound refused ten candidates and the slate chose the two-line quote on the clause); mikaela7 launched

`shi_Yumiaoya13` (frozen `93b11aef9`,
pid 1928249,
overlap 8,
launched 05:01 UTC on Bedrock and OpenRouter,
Synthetic and Hyper dry)
SETTLED at 05:22 UTC on one attempt,
20 minutes by the artifact.
Seats asked 873,
42 retry rounds,
773 calls,
4 abandoned streams (0.0017 USD),
Qwen3.8-27B and glm-5.3 dark (113 and 79 asked,
all threw).
Meters 116.65 to 116.35 USD on Bedrock and 172.45 to 171.97 on OpenRouter,
0.78 USD together.

### Checks

- Front matter equal to the archive and to `shi_Yumiaoya12`.
- 4 of 4 headings,
  1 photoscroll,
  1 footnote definition,
  15 blockquote lines,
  309 lines against the previous page's 319;
  no `TA`,
  no `finish_reason=error`,
  no double blank line,
  `attempts.json` reads 1.
- No `PublishedPageDisagrees` or `weight-off`,
  no unendorsed standing,
  no gate-kept or slate-declined standing,
  no `repeats the line`,
  no run-off,
  no `archive-revision-refused`;
  9 `declined every candidate` lines with reasons.
- 6 em-dashes,
  left per the owner's answer of 2026-09-21.
- Class seventy-nine holds
  ("she took a large amount of medication in the early hours of January 16 in an attempt to end her life",
  the ICU waking two lines on);
  class seventy-eight holds ("Magic Chewing Gum and Danpian").

### Class eighty live

The closing blockquote ships two lines,
"And in case I don't see you……good afternoon,
good evening,
and good night."
and "From *The Truman Show*",
and this time by the bench's choice,
not by an unendorsed standing:
the closing line is freshly written
("Well,
Yumiao,
*Girls' Last Tour* is over now,
and you can finally get a good night's sleep."),
so the slate's winner carried the quote.
The run artifact (`artifacts/shi_Yumiaoya.json`) carries the block finding ten times,
"gives the line `And in case I don't see you……good afternoon, good evening, and good night.` twice,
once in Chinese and once in English directly beside it;
that pair is ONE line whose English is already its rendering,
and the EXISTING TRANSLATION carries the block holding it as 2 lines",
one per candidate that kept the Chinese line raw or rendered it again,
each withheld from the slate at the floor
(the log carries no line for a floor refusal;
the artifact does).
The slate then read two candidates,
both with the quote as two lines,
and its ballots cite the clause:
deepseek-v4.1-flash "the quote block (Chinese line correctly dropped as it is paired with its English rendering,
per line-structure rule)",
minimax-m3 "dropping the explicit Chinese quotation of the Truman Show line (already present in English on the same page as a bilingual quote)",
the consolidation's deepseek ballot "each pair being one line whose English is already its rendering";
gpt-oss-120b abstained on "Both candidates omit the first Chinese line",
the one seat still reading the older rule,
outvoted.

### Observations, not classes

- The film link ships `[Josee, the Tiger and the Fish](...)` again
  (two ballots citing the cited reference,
  one calling *The Truman Show* "a distractor in the surrounding metadata");
  "the trio's car shows";
  "gaokao" kept;
  "in an attempt to end her life" carries the medication where `shi_Yumiaoya10` and `12` had "tried to end her life" alone,
  the substance still unnamed,
  which the reader-protection rule leaves to the judges.

### mikaela7 launched

The `shi_Yumiaoya` queue is done for class eighty;
the queue resumes one entry at a time on the newest build.
`mikaela7` (frozen `93b11aef9`,
pid 1944780,
overlap 8)
launched 05:24 UTC on Bedrock and OpenRouter
(meters 116.35 USD Bedrock,
171.95 OpenRouter at launch;
Synthetic and Hyper dry),
the `CORPUS NAMES` line naming nobody
("the original names nobody another entry declares"),
chosen as the cheapest line-structured carrier
(the Epilogue,
slice 28,
nine lines)
to read the pair bound and the line-rule clause as regression on a slice with no pair,
then classes seventy to seventy-seven and the survived-attempt clause,
then the seven steps and three checks against `mikaela6` and the archive.
`XingZ60` (the verse chunks and the closing poem) follows,
then `hulicaijia`,
`CuspariaKLSY`,
`zheermao`,
`yingying`.

## shi_Yumiaoya12 read, 2026-09-22, 05:05 UTC: SETTLED with the closing quote as the archive, but by an unendorsed standing, not by the bound; the pair bound rebuilt to compare the block holding the English line, fixed in 93b11aef9; shi_Yumiaoya13 launched

`shi_Yumiaoya12` (frozen `873104d8f`,
pid 1903793,
overlap 8,
launched 04:24 UTC on Bedrock and OpenRouter,
Synthetic and Hyper dry)
SETTLED at 04:53 UTC on one attempt,
29 minutes by the artifact.
Seats asked as the SEAT lines,
769 calls,
5 abandoned streams (0.0045 USD),
Qwen3.8-27B and glm-5.3 dark (115 and 73 asked,
all threw).
Meters 116.96 to 116.65 USD on Bedrock and 172.92 to 172.47 on OpenRouter,
0.76 USD together.

### Checks

- Front matter equal to the archive and to `shi_Yumiaoya11`.
- 4 of 4 headings,
  1 photoscroll,
  1 footnote definition,
  15 blockquote lines against the previous page's 12,
  319 lines against 328;
  no `TA`,
  no `finish_reason=error`,
  no double blank line,
  `attempts.json` reads 1.
- No `PublishedPageDisagrees` or `weight-off`,
  one unendorsed standing (slice 16,
  the closing quote and its lines,
  the archive's own wording standing without contest endorsement),
  no gate-kept or slate-declined standing,
  no `repeats the line`,
  no run-off,
  no `archive-revision-refused`;
  16 `declined every candidate` lines with reasons.
- 11 em-dashes,
  left per the owner's answer of 2026-09-21.
- Class seventy-nine holds ("she tried to end her life in the early hours of January 16");
  class seventy-eight holds ("Magic Chewing Gum and Danpian").

### The closing quote as the archive, by luck

The page carries the quote as two lines,
the film's English and "From *The Truman Show*",
and the closing line in the archive's two-line form
("Well,
Yumiao,
girl's last tour ended.
/ And you can finally get a good sleep.").
That is slice 16 standing unendorsed:
the translate slate split between candidate 1,
which kept both Chinese lines raw
(gpt-oss-120b and minimax-m3:
"preserves every original line (including the Chinese quote and citation)"),
and candidate 3,
which rendered the Chinese line again in English
(deepseek-v4.1-flash and gemma-4-26b:
"the only one that provides a translation for the first line");
the repair lane's editors kept the archive
(three ballots against candidates that "awkwardly retain" the Chinese lines or "duplicate the final line");
the consolidation slate split again with minimax abstaining on "all candidates fail the line-structure criterion",
and the standing stood.
No `translate-candidate-refused` finding and no `LINE-STRUCTURED` pair finding in the log:
the bound of `873104d8f` never fired,
because it compared the whole slice's line count with the lines owed,
and the archive writes the closing line as two lines where the original has one,
so the page carried one line more than owed and the bound read the page as keeping the pair apart.

Rebuilt in `93b11aef9`
(guard red first at `e06ac2945`,
two cases added to `bilingual-pair-bound.unit.test.ts`:
the page splitting another line of the slice,
and a rendering that reworded the English line).
`bilingualPairs` in `line-structure-guard.ts` now returns the pairs themselves,
and `pairBoundFindings` finds each pair's English line in the page and in the rendering
(quote markers off,
whitespace collapsed,
curled quotes straightened)
and compares the block holding it,
the run of quoted lines or the paragraph:
a rendering whose block is wider than the page's has rendered the Chinese half again and is refused at the floor.
Silent without page text,
where the page does not carry the English line as written,
where the page carries the Han line too,
or where the rendering reworded every English line of the block.
Lint 0 and 0,
types clean,
full suite green (`suite-93b11aef9.log`).

### Observations, not classes

- The film link back to `[movie](...)` where `shi_Yumiaoya11` had "Josee,
  the Tiger and the Fish" from the cited reference;
  the reference is on the sheet and the bench chose otherwise this run.
- "little yao-niang" again,
  "Three Musketeers" in quotation marks for 三剑客,
  "gaokao" kept,
  "correctional institution" again.

### shi_Yumiaoya13 launched

`shi_Yumiaoya13` (frozen `93b11aef9`,
pid 1928249,
overlap 8)
launched 05:01 UTC on Bedrock and OpenRouter
(meters 116.65 USD Bedrock,
172.45 OpenRouter at launch;
Synthetic and Hyper dry),
the `CORPUS NAMES` line naming the same five.
Read the log for the block finding first
(a `LINE-STRUCTURED` finding quoting "once in Chinese and once in English directly beside it;
that pair is ONE line" on the translate floor,
and `translate-candidate-refused` on the four-line and three-line candidates),
then the closing quote on the page,
then classes seventy-nine and seventy-eight,
then the seven steps and three checks against `shi_Yumiaoya12` and the archive.

## shi_Yumiaoya11 read, 2026-09-22, 04:30 UTC: SETTLED, the class eighty clause on every sheet and the judges still chose the second English wording; the page's own line count now bounds the pair, fixed in 873104d8f; shi_Yumiaoya12 launched

`shi_Yumiaoya11` (frozen `8f5e28ad4`,
pid 1878666,
overlap 8,
launched 03:47 UTC on Bedrock and OpenRouter,
Synthetic and Hyper dry)
SETTLED at 04:17 UTC on one attempt,
30 minutes by the artifact.
Seats asked 926,
43 retry rounds,
830 calls,
5 abandoned streams (0.0030 USD),
Qwen3.8-27B and glm-5.3 dark (114 and 82 asked,
all threw),
every other seat whole or within five.
Meters 117.28 to 116.97 USD on Bedrock and 173.44 to 172.93 on OpenRouter,
0.82 USD together.

### Checks

- Front matter equal to the archive and to `shi_Yumiaoya10`.
- 4 of 4 headings,
  1 photoscroll,
  1 footnote definition,
  12 blockquote lines against the previous page's 16,
  328 lines against 323;
  no `TA`,
  no `finish_reason=error`,
  no double blank line,
  `attempts.json` reads 1.
- No `PublishedPageDisagrees` or `weight-off`,
  one unendorsed standing (slice 1,
  the caution blockquote,
  the archive's own wording standing without contest endorsement),
  no gate-kept or slate-declined standing,
  no `repeats the line`,
  no run-off,
  no `archive-revision-refused`;
  14 `declined every candidate` lines with reasons.
- 6 em-dashes,
  left per the owner's answer of 2026-09-21.
- Class seventy-nine holds ("she attempted to end her life in the early hours of January 16",
  the ICU waking two lines on);
  class seventy-eight holds ("Chewing Gum and Danpian",
  "her cuddling plans").

### Class eighty, second arm: the clause reached the judges and they overruled it

The closing blockquote shipped three lines again:
"If I should never see you again,
I wish you good morning,
good afternoon,
and good night."
above the film's own line,
then "From *The Truman Show*".
The frozen build carries the clause
(`bilingual-line-clause-DLHhNPE9.mjs` in the frozen dist),
so every translator,
judge and editor on that slice read that the pair is one line whose English is already its rendering.
The translate slate's ballots on the slice:
deepseek-v4.1-flash chose the three-line candidate at weight 1,
"Candidate 2 drops the original Chinese line of the bilingual Truman Show quotation ... leaving only the English line;
under criterion 4 this is a dropped line and makes it ineligible";
gemma-4-26b at weight 0.5,
"The Chinese line ... is a literal translation of the sentiment,
which is distinct from the actual English quote provided in the next line";
minimax-m3 abstained on both grounds at once,
"Candidate 1 adds an English line ... that does not correspond to any line in the ORIGINAL ... This invents a line.
Candidate 2 drops the Chinese quotation line entirely".
In the repair lane the editors' candidate that added the line lost 4 to 1
("Candidate 2 adds an English translation of the preceding Truman Show quote",
"incorrectly duplicates the Truman Show quote that already appears in the surrounding text"),
so the archive's two lines stood there;
the contest then preferred the translate lane's three.
A judge without the film cannot see that the two lines are one,
and one that can still reads "drops a line" as the nearer rule.

The evidence the judges lack is on the sheet:
the EXISTING TRANSLATION,
a human's,
carried the pair as one line.
Fixed in `873104d8f`
(guard red first at `c630bfb7d`,
`bilingual-pair-bound.unit.test.ts`,
five tests:
refused where the page carries the pair once,
accepted where the rendering does too,
silent without page text,
silent where the page keeps the adjacent lines apart,
silent on a slice with no pair however long the rendering runs).
`bilingual-pair-bound.ts` names a rendering carrying more lines than owed on a slice whose original carries a Han line with its own English beside it,
where the page carries exactly the lines owed;
`compareLineCounts` calls it with the page text `translate-validate.ts` now threads through,
so the candidate is refused at the floor and withheld from the slate as class sixty-six withholds a rule-refused candidate.
Without page text,
or where the page itself keeps the two adjacent lines apart
(the ten entries of the census under the 03:50 UTC heading),
nothing is refused and the judges decide on the clause.
The shortfall check's own blind spot,
a longer rendering on a slice with no pair,
is untouched and pinned by the fifth test.
Lint 0 and 0,
types clean,
full suite green (`suite-873104d8f.log`).

### Observations, not classes

- The caution blockquote ships the archive's own wording this run
  ("Tips",
  "doesn't correspond to fact,
  while other parts still may require fact check"),
  the unendorsed standing of slice 1,
  where `shi_Yumiaoya9` and `shi_Yumiaoya10` re-rendered it;
  the `<Banner>` text is re-rendered again.
- The film link ships `[Josee, the Tiger and the Fish](...)`,
  the title the cited Wikipedia reference establishes,
  where `shi_Yumiaoya10` had "[movie]" and `shi_Yumiaoya8` "[name of a film]";
  two ballots cite the web lookup,
  one of them confusing itself mid-reason and recovering.
- "xiaoyao-niang" with the footnote where `shi_Yumiaoya10` had "little yao-niang";
  "the trio's car shows" for the source's 三剑客;
  "*Girls' Last Tour* has ended" italicised,
  where `shi_Yumiaoya10` had the archive's "girl's last tour ended".

### shi_Yumiaoya12 launched

`shi_Yumiaoya12` (frozen `873104d8f`,
pid 1903793,
overlap 8)
launched 04:24 UTC on Bedrock and OpenRouter
(meters 116.96 USD Bedrock,
172.92 OpenRouter at launch;
Synthetic and Hyper dry),
the `CORPUS NAMES` line naming the same five.
Read the closing blockquote first
(two lines,
the film's English and its attribution,
and a `LINE-STRUCTURED` finding naming the pair on the translate floor),
then class seventy-nine,
then class seventy-eight,
then the seven steps and three checks against `shi_Yumiaoya11` and the archive.
