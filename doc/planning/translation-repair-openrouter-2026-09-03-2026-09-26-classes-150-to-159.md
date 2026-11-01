# OpenRouter and the corpus passes: 2026-09-26, classes 150 to 159

What was launched,
read and decided on 2026-09-26 classes 150 to 159.
Entries:
Xu_Yushu1,
aiyysk4,
aiyysk2,
aiyysk1,
shihai4h2,
shihai4h1,
shi_Yumiaoya39,
shi_Yumiaoya38,
XingZ6014,
yingying12,
shi_Yumiaoya37,
shi_Yumiaoya36.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## Xu_Yushu1 read, 2026-09-26: INCOMPLETE at the pictures phase, the padded-reading log fixed

Xu_Yushu1 ran on `.frozen-dist-8f2b07c37` and stopped INCOMPLETE in 3.3 min (199,440 ms),
"visual evidence incomplete for 1 referenced assets":
`composition.webp`,
a handwritten essay.
The seat line read Synthetic and Hyper dry,
`readers=5`,
Kimi-K3 withheld
(OpenRouter withholding of 2026-09-03),
and Qwen3.8-27B failed outright with no provider,
since the owner's no-bleed directive of 2026-09-09 keeps it off OpenRouter.
GLM-5.3-Flash read the essay (1,216 characters).
minimax-m3 (1,149 characters),
gemma-4-26b (370,
after one refusal) and google.gemma-4-31b returned garbage;
no two of four readings agreed,
the closest GLM-5.3-Flash and minimax-m3 at overlap 0.206.
The stop is the bench's reach while Synthetic is dry,
not a code defect,
and corroboration stays as it is:
one model's reading is not evidence.

One logging defect:
gemma-4-31b's reply was 27 characters followed by about 2,700 ideographic spaces.
The sense verdict judged the trimmed 28 characters,
but the stage logged "2753 characters"
and handed the padded text on.
The usable branch of `readImageAsset` now logs and returns the trimmed reading,
as the short branch did
(guard red first `a6ce93330`,
fix `141598127`).
TianqiChen6661,
launched on `.frozen-dist-8f2b07c37`,
was killed a few minutes in under always-kill-and-relaunch;
TianqiChen6662 runs on `.frozen-dist-141598127`.

## aiyysk4 read, 2026-09-26: classes one hundred fifty-eight and fifty-nine live

aiyysk4 ran on `.frozen-dist-8f2b07c37` and SETTLED at 16:01 UTC in 44.9 min,
one attempt,
79 slices,
4,110 calls,
4,967 seats,
257 retry rounds,
2.66 USD of OpenRouter cost,
2 abandoned streams (0.01 USD).
All three new glossary terms reached the page as the glossary seeds them:
"I'd never actually spent time with an MTF person before" (交往,
class one hundred fifty-eight),
"I don't really care about my surroundings" (环境的问题) and "prototype" on all three 工程机 lines
(class one hundred fifty-nine).
Class one hundred fifty-seven held in its third seeded form,
"Sakurana used her own life as leverage:".
Front matter equal,
13 headings,
3 footnotes,
no TA,
no error finish,
no double blank line,
no unendorsed standing,
100 abstention lines,
one run-off;
11 em-dashes,
left as the owner said.
The two "doubles" log hits are judge prose ("Candidate 2 doubles …"),
not findings.
The time over aiyysk2's 26.7 min is the providers':
Synthetic read wet on 12 meter readings,
so Qwen (70 usable) and Kimi-K3 (35 usable) ran there at its latency,
and Mercury drew 156 upstream rate-limit lines
against aiyysk2's 24 (94 of its 582 seats threw).
No class.

## aiyysk2 read, 2026-09-26: class one hundred fifty-seven live, class one hundred fifty-eight

### What the run did

aiyysk2 ran on `.frozen-dist-2f5875dcf` and SETTLED at 15:07 UTC in 26.7 min,
one attempt,
79 slices,
4,103 calls,
2.57 USD of OpenRouter cost.
Front matter equal,
13 headings,
3 footnotes,
no TA,
no double blank line,
no error finish;
110 abstention lines,
three unendorsed standings;
13 em-dashes,
left as the owner said.
Class one hundred fifty-seven is live:
the threat line reads "Sakurana threatened to take her own life:".

### Class one hundred fifty-eight: 交往 as dating

我之前也没和 MTF 交往过…然后我也不敢去交往 shipped "I'd never dated a trans woman before either" on aiyysk2
and "I'd never actually dated a trans woman before" on aiyysk1,
a romance the original never states.
The narrator had simply never spent time with trans women;
the next paragraph says
"It was my first time ever talking with a trans woman",
and the archive wrote "I'd never actually talked to an MTF person before".
The pinned corpus carries 交往 four times,
all on aiyysk (lines 157,
159 twice and 185),
all ordinary social contact
(corrected 2026-09-28,
ledger R16:
this line once said three times).
交往 joins the phrasing glossary with "talked to" first.
The floor matches refused forms as lower-cased substrings (`refusedFormIn`,
`translate-community-term.ts`),
so the refused forms are multi-word ("never dated",
"dated a trans",
"dating them" and the like)
and never bare "dated" or "dating",
which "updated" and "validating" hold;
the guard carries an "updated" case.
Guard `rendering-glossary-contact.unit.test.ts`,
red first;
fix `bc20adb69`;
suite 1,157 PASS,
0 FAIL (`suite-class158.log`).
aiyysk3 launched 15:10 UTC on `.frozen-dist-bc20adb69` (pid 830253,
scope `pass-aiyysk3`)
and was killed about four minutes in under always-kill-and-relaunch once class one hundred fifty-nine landed.

### Class one hundred fifty-nine: 环境的问题 and 工程机

Reading the rest of aiyysk2 turned up two more calques,
both carried only by aiyysk in the pinned corpus.
她对我说：「我没有怎么在乎环境的问题…」 shipped "It's not that I cared all that much about the environment",
which an English reader takes for nature and pollution;
she meant the crowded place she lived.
The archive wrote "the environment itself" too.
工程机 (three times,
the Qualcomm Reference Design unit she spent two months reviving) shipped "engineering phone",
as the archive had it;
the phone world calls such a unit a "prototype" or "engineering sample".
Both join the phrasing glossary:
环境的问题 with "my surroundings" first
and "about the environment",
"the environment itself" and the "environmental" forms refused;
工程机 with "prototype phone" first and "engineering phone",
"engineering machine" and "engineering device" refused.
Other readings stayed as the bench wrote them:
「忧」 as "Worried."
is the original's own one-character reply,
and 刷机 as "flashing" is the phone world's word.
Guard `rendering-glossary-circumstance.unit.test.ts`,
red first;
fix `8f2b07c37`;
suite 1,158 PASS,
0 FAIL
(`suite-class159.log`).
aiyysk4 launched 15:17 UTC on `.frozen-dist-8f2b07c37` (pid 923957,
scope `pass-aiyysk4`).

## aiyysk1 read, 2026-09-26: class one hundred fifty-six against a correct archive, class one hundred fifty-seven

### What the run did

aiyysk1,
the first run of aiyysk,
ran on `.frozen-dist-6cf583ca3-class156` (the `0899e69b1` tree)
and SETTLED at 14:37 UTC in 37.6 min,
one attempt,
78 slices,
3,913 calls,
4,620 seats,
184 retry rounds,
2.70 USD of OpenRouter cost,
5 abandoned streams (0.03 USD);
Synthetic and Hyper dry.
Front matter equal,
13 headings,
3 footnotes,
no TA,
no double blank line,
no error finish;
145 abstention lines,
one run-off,
four unendorsed standings.
All three 初中 read "junior high" ("a junior high school in Yunfu",
"Her junior high chemistry teacher",
"After finishing junior high school"),
the archive's own rendering,
so class one hundred fifty-six held against an archive already right.
The page reads as fluent English throughout,
in Canadian spelling ("mould",
"meagre").

### Class one hundred fifty-seven: 以生命相逼 reversed

樱奈以生命相逼：「要是被我发现（你伤害自己），我就自杀。」 shipped "Sakurana threatened her life:",
which tells an English reader Sakurana threatened Shikai;
she staked her own life.
The archive had "threatened her in return".
The pinned corpus carries the idiom once,
here.
Under the owner's standing instruction to translate better wherever possible,
以生命相逼 joins the idiom glossary
with "threatened to take her own life" first and "threatened her life" and its variants refused
by the source-carry floor before any judge reads the candidate.
Guard `rendering-glossary-leverage.unit.test.ts`,
red first;
fix `2f5875dcf`;
suite 1,156 PASS,
0 FAIL (`suite-class157.log`).
aiyysk2 launched 14:43 UTC on `.frozen-dist-2f5875dcf` (pid 257150,
scope `pass-aiyysk2`).

## shihai4h2 read, 2026-09-26: class one hundred fifty-six live

shihai4h2 ran on `.frozen-dist-6cf583ca3-class156`,
built from the tree later committed as `0899e69b1`;
it was frozen and launched before the commit because every cli-git call failed on another session's abandoned worktree-copy journal.
It SETTLED at 13:12 UTC in 24.7 min,
one attempt,
3,122 calls,
3,647 seats,
119 retry rounds,
3.19 USD of OpenRouter cost,
2 abandoned streams (0.0016 USD);
Synthetic and Hyper dry,
so Qwen3.8-27B threw all 525 calls and glm-5.3 all 234.
Every shihai4h1 check held:
front matter equal,
5 headings,
10 footnotes,
2 PhotoScroll,
no TA,
no double blank line,
no error finish;
40 abstention lines,
one run-off,
five archive-revision-refused lines,
two unendorsed standings (17,
33).

- Class one hundred fifty-six live:
  "junior high school" on both of the page's 初中 lines (73,
  169),
  "to take self-study examinations" (76),
  "was frequently pressured" (78).
- Class one hundred fifty-four held:
  "She and the Trans Community",
  both body occurrences "the trans community".
- Both calques shihai4h1 left as observations are gone by the bench's own choice:
  "was never able to meet her parents’ expectations afterward" (73)
  and "the deepest core of a person’s spirit and will is always alone" (342).
- Canadian spelling held ("centre" twice).

## shihai4h1 read, 2026-09-26: class one hundred fifty-four against a correct archive, class one hundred fifty-six

### What the run did

shihai4h1 (frozen `6cf583ca3`,
overlap 8),
the first run of shihai4h,
SETTLED at 12:34 UTC in 27.5 min,
one attempt,
54 slices,
41 changed,
3,175 calls,
3.14 USD of OpenRouter cost;
the meters read Bedrock about 16.09 and OpenRouter 39.56 at the end.
Synthetic and Hyper dry:
Qwen3.8-27B threw 461 of 514 calls and glm-5.3 all 240.
Front matter equal,
5 headings,
10 footnotes,
2 PhotoScroll,
no TA,
no double blank line,
as the archive;
211 blockquote lines against the archive's 168 because the page is semantically wrapped
(676 lines against 458).
The log carries 37 abstention lines,
one translate run-off ("the finalists were valid,
so the ballots decide"),
two archive-revision-refused lines and five archive review lines.

- Class one hundred fifty-four held against an archive that was already right:
  all three 跨圈 read "the trans community",
  the heading "She and the Trans Community" as the archive.
  "crossdressing hobby" on line 139 renders 异装癖,
  another word,
  and is right.
- The page corrected two archive errors:
  the year 2013 where the archive wrote 2023,
  and "her older cousin" where the archive wrote "her sister".
- The translator's doubt comment was dropped because the sentence it doubted was retranslated correctly.
- Canadian spelling and vocabulary clean.

### Class one hundred fifty-six: three schooling calques

Under the owner's standing instruction to translate better wherever possible:
初中 shipped "junior middle school" (line 76),
自考 "self-taught exams" (line 79)
and 压力话 "frequently subjected to pressured remarks" (line 82).
初中 stands nine times in five pinned entries;
the archives write "junior high school" (Jennife80677612 once,
aiyysk twice),
"middle school" (yulianNyanner twice) and "junior middle school" (shihai4h alone),
and XIEPT2's archive renders it nowhere
(corrected 2026-09-28,
ledger R16:
this line once counted a 初中 in a Xu_Yushu reader comment as an archive passage).
自考 (高等教育自学考试,
the self-study examinations) and 压力话 stand on shihai4h alone,
with no archive English.
The three join the wording glossary with "junior high school",
"self-study examinations" and "pressured" first,
and "junior middle school",
"self-taught exam" and "pressured remarks" with their variants refused
by the source-carry floor before any judge reads the candidate.
Guard `rendering-glossary-study.unit.test.ts` (`6dea1d435`),
red first on the `6cf583ca3` build;
fix `0899e69b1`.
Suite 1,155 PASS,
0 FAIL (`suite-class156.log`).
Both commits waited about an hour:
another session's `worktree add` died partway through its ignored-state copy,
and every cli-git call failed on the abandoned copy journal until that session recovered it.

### Observations not built

- "continued to fail to meet her parents’ satisfaction" (line 76) and
  "a person’s most core spiritual will is always alone" (line 360) read as calques,
  but neither is a term a glossary entry can hold;
  watched.

## shi_Yumiaoya39 read, 2026-09-26: classes one hundred fifty-four and fifty-five live

shi_Yumiaoya39 (frozen `6cf583ca3`,
overlap 8) SETTLED at 12:05 UTC in 7.6 min,
one attempt,
746 calls,
874 seats,
44 retry rounds,
0.58 USD of OpenRouter cost,
Bedrock 17.45 to 17.24 by the meters;
Synthetic and Hyper dry,
so Qwen3.8-27B and glm-5.3 threw every call.
Every shi_Yumiaoya38 check held:
front matter equal,
4 headings,
1 footnote,
no TA,
no double blank line,
no unendorsed standing,
no error finish,
11 abstention lines.

- Class one hundred fifty-four live:
  all four 跨圈 read "the Trans Community" (lines 33,
  36,
  159,
  177);
  no crossdressing form on the page.
- Class one hundred fifty-five live:
  the insult reads "insulted as an “unfilial son” — a disgrace,
  good-for-nothing."
- Class one hundred fifty-three held on a prose page:
  22 polish-gate ballots,
  none naming line breaks or wrapping.
- 同类 on source line 83 now reads "a housemate from the local trans community",
  no gender added (the shi_Yumiaoya38 watch closed).
- The caution callout's label 提示 reads "Tips",
  as the archive writes it (runs 30 to 37 wrote Note,
  Notice or Tips);
  the archive's rendering stands,
  so nothing is built.
- One Mercury select ballot rejected a candidate repeating "And you can finally get a good sleep."
  twice;
  the page carries it once.

## shi_Yumiaoya38 read, 2026-09-26: class one hundred fifty-one live, classes one hundred fifty-four and fifty-five

### What the run did

shi_Yumiaoya38 (frozen `12b581811`,
overlap 8) SETTLED at 11:49 UTC in 6.4 min,
Bedrock 17.64 to 17.45 and OpenRouter 42.24 to 41.91 by the meters,
727 calls.
Front matter equal,
4 headings,
1 footnote,
no TA,
no double blank line,
no unendorsed standing,
7 abstention lines.
Class one hundred fifty-one live:
the father's insult reads "insulted as an 'unfilial son'—a disgrace,
someone with no capability"
where shi_Yumiaoya37 wrote "rebellious child".
初三 now reads "In Grade 9".

### Class one hundred fifty-four: 跨圈 read as the crossdressing community

跨圈 (short for 跨性别圈子) appears four times on the page.
Three shipped "the trans community";
source line 77,
在跨圈内结识了很多朋友,
shipped
"made many friends in the crossdressing community",
where shi_Yumiaoya37 had "across different communities".
Crossdressing is another word (女装) for another thing.
The corpus carries 跨圈 on shihai4h (3),
XingZ60 (1) and shi_Yumiaoya (4);
shihai4h's archive renders it "the Trans Community" (its heading "She and the Trans Community").
Fixed in `42319ff3a` (guard red first `5c51bd440`):
跨圈 joins the community glossary
with "trans community" first and "crossdressing community",
"cross-dressing community" and the "circle" forms refused
by the source-carry floor before any judge reads the candidate.
Suite 1,153 PASS,
0 FAIL (`suite-class154.log`).

### Class one hundred fifty-five: 没本事 word for word

The same insult rendered 没本事 as "someone with no capability";
shi_Yumiaoya37 had "a failure".
The pinned corpus carries 没本事 once,
in that insult,
with no archive English.
Under the owner's standing instruction to translate better wherever possible,
fixed in `6cf583ca3` (guard red first `c160b2006`):
没本事 joins the wording glossary
with "good-for-nothing" first and "no capability" and its variants refused.
Suite 1,154 PASS,
0 FAIL (`suite-class155.log`).
shi_Yumiaoya39 launched 11:57 UTC on `.frozen-dist-6cf583ca3` (pid 2592631,
scope `pass-shi_Yumiaoya39`).

### Observation not built

同类 shipped "a trans woman from a local community",
narrowing a gender the original leaves open;
watched.

## XingZ6014 read, 2026-09-26: class one hundred fifty-two live, class one hundred fifty-three

### What the run did

XingZ6014 (frozen `1dd8ebc52`,
overlap 8) SETTLED at 11:33 UTC in 54.9 min,
Bedrock 20.39 to 17.65 and OpenRouter 47.98 to 42.31 by the meters (about 8.41 USD),
Synthetic and Hyper dry,
so glm-5.3 and Qwen3.8-27B threw every call.
121 slices,
207 repair issues,
139 accepted,
114 slices where the lanes differed,
99 slices changed on the page.
Every XingZ6013 check held:
14 headings,
2 links,
9 definitions,
3 TextRing,
11 details blocks,
front matter equal,
no TA,
no double blank line,
no error finish,
the placeholder kept.
The heading and footnote 6 read "Layer-Zero Prayer" (XingZ6013 wrote "Zero-Layer Prayer"),
one rendering on the page;
the second credit carries the album as "[Paranoia]".
Two unendorsed standings (2,
9),
31 gate-kept,
25 slate-declined,
143 abstention lines.

### Class one hundred fifty-two live

The polish gate ran 59 times with 264 ballots and shipped 17 polishes (XingZ6013 shipped 9).
On slice 30,
where the polish differs from the wrapped base by one comma and the line break after it,
three ballots set the breaks aside as the sheet told them
("the only difference is line-break wrapping,
which the rules say not to weigh").

### Class one hundred fifty-three: the base the page's own one-line paragraph

The class one hundred fifty-two sentence said both candidates were wrapped by one rule.
That is false where the base stands as the archive's own wording:
on slices 36,
61 and 64 the base was one line a paragraph and the polish was wrapped,
and three ballots weighed the polish's added breaks
(minimax-m3 "adds line breaks inside a paragraph",
gemma-4-26b-a4b-it "introduces unnecessary line breaks"
and "its line breaks are not requested by the source").
Slice 36 settled on base after a 2 to 2 tie with the one line-break ballot abstaining;
slice 61 lost its polish ("I did what I'd planned from the start ... She was sad too that she never got to meet you") 3 to 1.
Fixed in `12b581811` (guard red first `b2f5f373c`):
`foldSoftBreaks` in `soft-break-fold.ts` folds each top-level paragraph's soft breaks into spaces,
and the gate sheet shows both candidates folded on a prose slice,
keeping hard breaks,
blockquotes,
lists and a line-structured slice as written;
what ships is unchanged.
The policy sentence now says the paragraphs are shown one line each.
Replayed over the run's real slices 16,
23,
30,
36,
61 and 64:
every candidate reaches the sheet one line a paragraph.

### Observations not built

- The absolute naturalness review still names soft breaks on prose although its sheet says they render as spaces:
  slice 23 "excessive line breaks",
  slice 50 "poem-like line breaks ... choppy verse formatting"
  and a capitalisation "error" at a wrapped line's start,
  slice 84 "the line break creates an unnaturally abrupt fragment".
  Slice 50's polish shipped anyway on its other findings;
  no outcome turned on these,
  so it is watched,
  not built.
- gemma-4-e2b cast 59 gate ballots;
  one reason is garbled ("changes 'it seems' to 'it seems'"),
  the rest name the wording they weigh.
  Not enough to drop it.
- Slice 50 shipped a bench-written em-dash ("yourself — but that chance is truly gone now"),
  left per the owner.

## yingying12 read, 2026-09-26: class one hundred fifty-two

### What the run did

yingying12 (frozen `.frozen-dist-0de47d546`,
classes one hundred fifty and fifty-one)
SETTLED at 10:35 UTC in 10.4 min,
one attempt,
Bedrock 20.43 to 20.39 and OpenRouter 48.17 to 48.01 by the meters.
Synthetic and Hyper read dry throughout.

Every yingying11 check held:
front matter equal,
1 heading,
2 markers and 2 definitions as the archive,
the References line kept,
"Yingying" in [^2] where the archive has "Sakura",
"May you" in the second person,
no TA,
no Han,
no double blank line,
no unendorsed or gate-kept standing,
2 abstention lines.
Neither the class one hundred fifty nor the class one hundred fifty-one floor applies to this page.

The polish gate's ballot lines (class one hundred fifty's logging) ran three times,
12 ballots:
the first slice shipped the polish 3 to 1
("riding her bike",
"a toothpaste box marked with the Intel logo;
she said it made everyone who went into her washroom burst out laughing").

### Class one hundred fifty-two: the polish gate weighed a prose slice's wrap

The third gate run tied 2 to 2 and the base shipped
"It is a pity that all this stopped abruptly;
she lost her life forever in a car accident."
deepseek-v4.1-flash chose the polish for replacing "the stiff calque";
inception/mercury-2.5 kept the base because the polish "changes line structure",
and on the second gate run gemma-4-26b and Mercury both cited the polish's line breaks.
The comparative policy named "line structure" among what the polish may not change,
yet on a prose slice both candidates are wrapped at their semantic boundaries by one rule before the gate
(`consolidation-polish-round.ts`),
so where either breaks a line follows from its wording and is nobody's choice.

`comparativePolishPolicy` (`consolidation-polish-gate-wire.ts`) now keeps "line structure" only where the line rule governs the slice
and otherwise tells the gate that line breaks inside a paragraph are the page's own wrap,
never to be weighed;
`ConsolidationPolishGateSubject` carries the slice's `lineStructured`,
threaded from the polish round.
The required-correction policy never named line structure and is unchanged.
Guard `7d5d3fe75` (the gate test's prose and line-structured sheets) red on the class one hundred fifty-one build,
fix `1dd8ebc52`,
lint 0/0,
types clean.

### Observations not built

- google.gemma-4-e2b's first-gate ballot reason is incoherent ("changing 'riding her bike' to 'riding her bike'");
  one ballot is not the evidence a cull needs,
  so its gate ballots are watched on the next runs.
- "hardcore video Bilibili account" and "items related to electronic devices" stay close to the Chinese;
  the gate judged them faithful and the polish that would change them lost only on the wrap.

## shi_Yumiaoya37 read, 2026-09-26: the attempt said, and class one hundred fifty-one

### What the run did

shi_Yumiaoya37 (the uncommitted class one hundred fifty build,
frozen as `.frozen-dist-class150-20260926`)
SETTLED at 10:18 UTC in 15.8 min,
one attempt,
901 seats,
51 retry rounds,
771 calls,
about 1.03 USD by the OpenRouter spend lines plus Bedrock 20.65 to 20.43.
Synthetic and Hyper read dry throughout.

The January 16 sentence reads "she attempted to end her life in the early hours of January 16."
The bench wrote the attempt itself:
no candidate drew the class one hundred fifty refusal (`drops the suicide` appears nowhere in the log),
so the floor was silent,
not exercised.
The polish gate's ballot lines are live:
28 ballots over 7 gate runs,
each with model,
choice and reason.

Every shi_Yumiaoya36 check held:
front matter equal,
4 headings,
no TA,
no error finish,
no double blank line,
no unendorsed standing,
no repeat or run-off finding,
11 abstention lines.
The film link reads "Josee,
the Tiger and the Fish" again,
where shi_Yumiaoya36 had "film".

### Class one hundred fifty-one: the father's insult lost its "son"

The original reports that her father insulted her as 「逆子」,
"unfilial son".
On a trans woman's memorial,
the insult is her father calling her his son.
shi_Yumiaoya36 and 37 both shipped "rebellious child",
which takes the misgendering out of the insult the page reports,
and shi_Yumiaoya8 left it in Han.
No archive English covers the passage (the archive is partial),
so nothing on any sheet said which.

`逆子` joins the community glossary (`community-glossary.ts`)
with the renderings "unfilial son",
"undutiful son",
"disobedient son",
"ungrateful son" and "rebellious son",
and a why naming the misgendering.
So the identity context carries the term,
the judges' sheets name any candidate lacking every rendering,
and the source-carry floor refuses the term left in Han.
It refuses no English form;
"child" is named as evidence,
not barred.
The corpus carries 逆子 only on shi_Yumiaoya (the Simplified and Traditional originals).

The departure line on the judges' sheets ended "the archive renders it so",
which was already untrue for 药娘 (the archive keeps it in Han,
and the owner's ruling overrides it)
and would be untrue for 逆子.
It now ends "the community glossary renders it so".
The glossary's header says which entries have no archive rendering.

Guard:
`community-glossary.unit.test.ts` seeds five terms and names a "rebellious child" candidate while leaving "unfilial son" unnamed.
It was red against the class one hundred fifty build (2 FAIL;
`aea5c3d91`) and passes on the fix (`0de47d546`);
types clean,
full suite 1,152 PASS,
0 FAIL (scratch `suite-class151.log`).

### A note on the unit runner's output

A throwaway probe with pass,
fail,
pass in one `describe` showed the runner prints one PASS line per `describe`
naming every passing test in it,
and one FAIL line per failing test.
Suite PASS counts are therefore counts of `describe` blocks,
not tests;
FAIL counts are per test and complete.

### Not built

- 提示 in the caution callout still ships as "Tips".
- "to do suicide intervention" reads as a calque.

## shi_Yumiaoya36 read, 2026-09-26: class one hundred fifty

### What the run did

shi_Yumiaoya36 (frozen `eac0b67a8`) SETTLED at 09:53 UTC in 5.5 min,
one attempt,
848 seats,
51 retry rounds,
729 calls,
about 0.50 USD by the meters (Bedrock 20.83 to 20.65,
OpenRouter 49.36 to 49.04).
Synthetic and Hyper read dry throughout,
so the two seats only they serve threw every call (Qwen3.8-27B 117,
glm-5.3 81)
and class one hundred forty-nine was not exercised again.

Every shi_Yumiaoya35 check held:
front matter equal,
4 headings,
no TA,
no error finish,
no double blank line,
no unendorsed standing,
no repeat or run-off finding,
18 abstention lines.

### Class one hundred fifty: the page dropped a suicide attempt the original names

The original's 于 1 月 16 日凌晨吞下大量药物自杀 is an attempt she survived:
the paragraph goes on to six days in intensive care and her waking.
The page shipped "she overdosed in the early hours of January 16."
The attempt the original states was gone,
and the means,
which the house rule keeps vague,
stood in its place.
shi_Yumiaoya14 and 35 had shipped "attempted suicide" for the same sentence.

The bench had the right text in hand.
The consolidation polish proposed "attempted suicide by overdosing" and won the polish round 2.5 to 2,
with two ballots citing the survived-attempt rule of class seventy-nine.
The polish gate then settled on neither,
and the base shipped.
The log carried only the gate's tally,
not its ballots' reasons,
so why the gate refused cannot be read from this run.

Fixed in two parts:

- `droppedSuicideFindings` in `translate-suicide-drop.ts` refuses,
  before any judge reads it,
  a candidate whose original names a suicide (自杀,
  自尽,
  轻生,
  outside comments)
  and which carries no wording for suicide at all.
  A word on the stem `suicid`,
  a kill verb with a reflexive ("killed herself"),
  or an end or take verb closing on a life ("tried to end her life",
  "took her own life") all count.
  The floor asks only that the suicide be said,
  never how;
  whether the means are too specific stays with the judges.
  It runs in `sourceCarryFindings` after the address floor,
  so it covers translate candidates,
  consolidation proposals and standings alike.
- The consolidation polish gate now logs every ballot with its model and reason,
  as the consolidate gate has since class fifty-six.

The corpus census of 2026-09-26 (scratch `suicide-census.mjs`) found 44 entries whose original names a suicide.
The archive English of 40 says it.
Two (shi_Yumiaoya,
XIEPT2) are partial archives without those passages.
Two render a quoted line without the word:
自杀痛苦 as "the pain of dying" and 别自杀 as "please don't follow her".
The floor now refuses those two lines as standings,
so those slices go to the lanes instead of shipping as the archive has them.

The guard `suicide-drop.unit.test.ts` was red against the old build (two FAIL,
one PASS)
and passes on the new one.
The full suite passed 1,152 with no FAIL (scratch `suite-class150.log`).
That suite ran before the polish gate's log line was added;
the polish gate's own tests pass on the rebuilt dist.
A replay over the real paragraphs refuses shi_Yumiaoya36's shipped "overdosed" paragraph
and accepts shi_Yumiaoya35's "attempted suicide" paragraph.

### Observations not built

- 逆子 as "rebellious child" loses the gendered "son" in the father's insult.
- The handles 雪秋 and 由之 shipped without the literal gloss class eighty-three asks for.
  No candidate glossed them,
  so there was nothing for a deterministic pass to restore.
- The caution callout's 提示 as "Tips".
- 初三 as "third year of junior high".
- The italics on *Girls' Last Tour* lost.
- The film link's text is just "film".
- The sedative detail stays on the page,
  as on shi_Yumiaoya23.

### Next

shi_Yumiaoya37 was launched at 10:02 UTC.
It runs from a freeze of the class one hundred fifty build,
`node_modules/.frozen-dist-class150-20260926`.
The freeze was taken before the commit because another session's `worktree add` held git's worktree-copy settlement lock.
The lock cleared at about 10:20 UTC;
the guard landed as `c7168c1ec` and the fix as `fb41795ba`,
the same source as the freeze.
Read first whether the January 16 sentence says the attempt.
Then check any `drops the suicide` refusal and the polish gate's ballot lines,
then the standing checks.
