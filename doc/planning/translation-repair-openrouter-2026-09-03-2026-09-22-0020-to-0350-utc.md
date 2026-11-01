# OpenRouter and the corpus passes: 2026-09-22, 00:20 to 03:50 UTC

What was launched,
read and decided on 2026-09-22 00:20 to 03:50 UTC.
Entries:
shi_Yumiaoya10,
shi_Yumiaoya11,
shi_Yumiaoya9,
d29feecd5,
shi_Yumiaoya8,
mikaela6,
zheermao3,
zheermao2.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## shi_Yumiaoya10 read, 2026-09-22, 03:50 UTC: SETTLED, class seventy-nine live, class eighty found (the bilingual film quote rendered twice in English because every line-structure wording said never drop a line), fixed in 8f5e28ad4; shi_Yumiaoya11 launched

`shi_Yumiaoya10` (frozen `d29feecd5`,
pid 1846495,
overlap 8,
launched 02:51 UTC on Bedrock and OpenRouter with Synthetic wet again at launch,
Hyper dry)
SETTLED at 03:33 UTC on one attempt,
45 minutes by the artifact against `shi_Yumiaoya9`'s 28:
repair lane 02:54 to 03:00 UTC,
translate lane 03:00 to 03:20 UTC (against 8 minutes on `shi_Yumiaoya9`),
consolidation 03:20 to 03:33 UTC.
Seats asked 964,
39 retry rounds,
888 calls,
no abandoned stream,
glm-5.3 dark (75 asked,
75 threw),
Qwen3.8-27B 56 of 124 and Kimi-K3 42 of 61 through Synthetic until its five-hour window read 2750 of 2750 at 03:30 UTC,
after which the 19 Kimi and 68 Qwen refusals are `NoProviderForModelError` seat holds inside the consolidation,
not provider errors.
Meters 117.48 to 117.28 USD on Bedrock and 173.89 to 173.46 on OpenRouter,
0.63 USD together.

### Checks

- Front matter equal to the archive and to `shi_Yumiaoya9`.
- 4 of 4 headings,
  1 photoscroll,
  1 footnote definition,
  16 blockquote lines against the previous page's 18,
  323 lines against 320;
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
  23 `declined every candidate` lines with reasons.
- 7 em-dashes,
  left per the owner's answer of 2026-09-21.

### Class seventy-nine live

The January 16 sentence ships
"After celebrating her current partner's birthday with them,
in the early hours of January 16,
she tried to end her life."
followed by
"after six days of rescue efforts in the ICU,
Yumiao — her body gravely damaged — finally woke up".
The log carries "tried to end her life" on three ballots
("as in the original and house rules";
one abstention faulting a candidate for "tried to end her life" without the temporal detail,
which the winner carries)
and "ended her life" only on the rooftop sentence,
where she did die,
and in the rule's own text.
Class seventy-eight holds:
the ICU paragraph ships "Magic Chewing Gum and Danpian",
the entry's declared name,
where `shi_Yumiaoya9` had the alias "Chewing Gum";
贴贴计划 ships "her cuddling plan".

### Class eighty: the bilingual film quote rendered twice in English

The closing blockquote's original is four lines:
如果再也不能见到你，祝你早安，午安，晚安。 then
"And in case I don't see you……good afternoon,
good evening,
and good night."
then
出自《楚门的世界》 then
"From *The Truman Show*".
The archive carries two lines,
the film's English line and "From *The Truman Show*".
`shi_Yumiaoya10` shipped three:
"If I can never see you again,
I wish you good morning,
good afternoon,
and good night."
above the film's own line,
then the attribution once.
`shi_Yumiaoya4` to `shi_Yumiaoya9` shipped four,
the two Chinese lines kept in Chinese on the English page
(the `shi_Yumiaoya9` read under the heading of 02:55 UTC called that "as the archive has it";
it is not,
the archive carries the English alone,
and that note is corrected here).

The bench had no rule for the pair.
The line floor counts a Han line beside its own English as one line owed
(class forty-seven,
`bilingualPairCount`),
and the class seventy-four guard catches a wording carried twice,
but every wording of the line rule on the sheets said the opposite:
`TRANSLATE_LINE_STRUCTURE_RULE` to the translators and consolidation producers
("never invent or drop a line"),
`TRANSLATE_LINE_STRUCTURE_CRITERION` to the slate judges
("a candidate that ... drops or invents a line is FAULTY"),
`LINE_STRUCTURE_RULE` to the editors
("never invent a line,
drop a line").
So every translate candidate rendered the Chinese line
(deepseek-v4.1-flash:
"original has a 4-line blockquote (Chinese quote,
English quote,
Chinese attribution,
English attribution) but the Chinese quote line is absent from every candidate"),
and the judges split on the ground the rule left open:
gpt-oss-120b abstained on "All candidates omit the original line 出自《楚门的世界》",
Qwen3.8-27B on "Candidates 1 and 2 omit the first quoted line",
gpt-oss-120b in the repair lane on "Both candidates keep the original English line after adding a new translation".

Fixed in `8f5e28ad4`
(guard red first at `49327e735`,
`bilingual-line-policy.unit.test.ts`,
four tests:
the translators' rule,
the judges' criterion and the editors' addendum carry the clause,
an ungoverned addendum carries none).
`bilingual-line-clause.ts` holds one wording,
interpolated into all three:
"A line the ORIGINAL gives twice,
once in Chinese and once in English directly beside it (a quotation in both languages,
and its attribution the same way),
is ONE line whose English is already its rendering:
carry that English line once,
as the ORIGINAL has it;
a rendering carrying the English line alone for such a pair has dropped nothing,
and one carrying the Chinese line,
or a second English wording of it,
beside the English has invented a line."
Stated for the models rather than enforced as a surplus bound:
measured over the pinned corpus
(`scratchpad/bilingual-census.mjs`,
every Han line followed by a Latin-only line in the 92 originals),
`shi_Yumiaoya`'s two pairs are the only Chinese lines with their English beside them,
while Arita,
NIGHT81473140,
gqt,
zhangyubaka,
Y1Ran,
luxuanwen3,
cheonwoomaeng,
hakureico,
Susiethegamer and XIEPT2 carry a Chinese line followed by an English line of different content
(an introduction then a quoted poem,
a letter then its signature),
which `bilingualPairCount` cannot tell apart,
so a mechanical bound would refuse faithful renderings there.
Lint 0 and 0,
types clean,
full suite green (`suite-8f5e28ad4.log`).

### Observations, not classes

- The closing line ships "girl's last tour ended",
  the archive's own wording,
  where `shi_Yumiaoya9` had "Girls' Last Tour has ended";
  the consolidation slate split 2/1/1 over candidates praising the anime title,
  and the standing stood.
- "correctional institution" for the source's quoted “矫正”机构 where `shi_Yumiaoya9` kept the quotation marks;
  "college entrance exam" for gaokao;
  "gave up on her schoolwork" for the 摆烂 idiom where `shi_Yumiaoya9` had "let it rot".
- "Yumiao — her body gravely damaged — finally woke up":
  one more em-dash pair,
  left per the owner.
- The `<Banner>` block's title and text and the caution blockquote are re-rendered again
  ("You may be following a hot event that is currently happening"),
  differently on every run;
  the front matter itself is equal.
  Measured over the pinned corpus,
  two originals carry a Banner,
  each with its own text,
  so it is no shared boilerplate to restore;
  the judges own the wording.

### shi_Yumiaoya11 launched

`shi_Yumiaoya11` (frozen `8f5e28ad4`,
pid 1878666,
overlap 8)
launched 03:47 UTC on Bedrock and OpenRouter
(meters 117.28 USD Bedrock,
173.44 OpenRouter at launch;
Synthetic and Hyper dry),
the `CORPUS NAMES` line naming the same five.
Read the closing blockquote first
(the film's English line once,
"From *The Truman Show*" once,
no Chinese line and no second English wording),
then the January 16 sentence,
then class seventy-eight,
then the seven steps and three checks against `shi_Yumiaoya10` and the archive.

## shi_Yumiaoya9 read, 2026-09-22, 02:55 UTC: SETTLED, class seventy-eight live, class seventy-nine found (a survived suicide attempt rendered as a death by the reader-protection rule's own sample sentence), fixed in d29feecd5; shi_Yumiaoya10 launched

`shi_Yumiaoya9` (frozen `545bff7b9`,
pid 1820803,
overlap 8,
launched 02:14 UTC on Bedrock and OpenRouter,
Synthetic and Hyper dry)
SETTLED at 02:42 UTC on one attempt,
28 minutes by the artifact,
the same as `shi_Yumiaoya8`.
Seats asked 923,
54 retry rounds,
819 calls,
6 abandoned streams (0.0029 USD),
Qwen3.8-27B and glm-5.3 dark all run (110 and 94 asked,
all threw,
the `SEATS DARK` line at the tally),
GLM-5.3-Flash 48 of 54,
gemma-4-e2b 134 of 137,
Jev 22 of 22,
every other seat whole.
Meters 117.81 to 117.50 USD on Bedrock and 174.39 to 173.93 on OpenRouter,
0.77 USD together.

### Checks

- Front matter equal to the archive and to `shi_Yumiaoya8`.
- 4 of 4 headings,
  1 photoscroll,
  1 footnote definition,
  18 blockquote lines,
  320 lines against the previous page's 324;
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
  14 `declined every candidate` lines with reasons.
- 6 em-dashes,
  left per the owner's answer of 2026-09-21.

### Class seventy-eight live

The `CORPUS NAMES` line named five people
(贴贴 for `CutOceanHeyFis1` as "Cuddly",
"Affectionate";
盐田 for `SS3B_0016` as "Yantian",
"Metro Line 8";
紫壑 for `Weideriche_` as "Weideriche_",
"Zihe";
口香糖 for `a2581911655` as "Magic Chewing Gum",
"Chewing Gum",
"jjlin1219";
单片 for `interrgned` as "interrgned",
"Danpian").
The ICU paragraph ships
"Yumiao heard the devastating news that Chewing Gum and Danpian had also passed away,
one after the other"
where `shi_Yumiaoya8` had "Xiaoguantang and Danpian".
Three of the five translate-slate ballots on that slice name the declared names in their reasons
(gpt-oss-120b:
"uses the declared names (Yumiao,
Yantian,
Chewing Gum,
Danpian)";
deepseek-v4.1-flash:
"All three candidates ... use the declared names (Yumiao,
Yantian,
Chewing Gum,
Danpian)";
minimax-m3:
"all three use the declared spellings Yumiao,
Yantian,
Chewing Gum,
Danpian correctly").
The hedge held:
贴贴计划 ships "her cuddling plans",
the ordinary word,
not "Cuddly's plans".
紫壑 ships as "Weideriche_",
the entry's declared name,
where `shi_Yumiaoya8` had guessed "Zihe";
the line offers both forms and the bench chose the handle,
which is what that entry's own page carries.

### Class seventy-nine: a survived attempt rendered as a death

The January 16 sentence shipped
"in the early hours of January 16 she ended her life by taking a large amount of medication",
followed two sentences later by
"after six days of emergency treatment in the ICU,
Yumiao,
whose body had been severely damaged,
finally woke up".
The original reads 吞下大量药物自杀。幸运的是，在 ICU 抢救了六天后……终于醒来了:
an attempt she survived.
"Ended her life" asserts a death;
the page contradicts itself,
and `shi_Yumiaoya8` shipped the same sentence
("she ended her life by taking medication in the early hours of January 16"),
where `shi_Yumiaoya7` had "tried to end her life".
The log carries "ended her life" 12 times and no candidate wrote "attempted" or "tried".

Every ballot approved it as reader protection.
minimax-m3:
"It handles reader protection correctly for the suicide method:
'ended her life by taking a large amount of medication' keeps the general method vague while preserving the act of suicide".
deepseek-v4.1-flash:
"All three candidates ... use the vague method wording required by reader protection.
Candidate 1 renders 自杀 with the explicit phrase 'ended her life' ... which is compliant".
gemma-4-e2b:
"'took medication and ended her life' which is too specific per the house rules".
The rule prescribed the words:
the reader-protection line in `house-policy.ts` opened "When a death was by suicide"
and closed its method paragraph with "the page says that she ended her life,
and keeps the night,
the room and the dress,
not the means",
so 自杀 was written as a death whatever the original went on to say.

Fixed in `d29feecd5`
(guard red first at `a478ad64f`,
`survived-attempt-policy.unit.test.ts`,
three tests:
the house policy carries the clause,
the translate sheet carries it,
the select sheet carries it;
`HOUSE_POLICY_BLOCK` exported through the translate barrel for the built-artifact test).
The rule now continues:
"自杀 names the act,
not its outcome:
where the ORIGINAL goes on to a rescue,
a waking or a life that continues (抢救,
醒来,
幸存),
the person survived the attempt and the page says that she tried to end her life or attempted suicide;
'ended her life' and 'took her own life' assert a death,
and on an attempt survived they are a factual error,
not vagueness."
Lint 0 and 0,
types clean;
the full suite's result is recorded under the next heading.

### Observations, not classes

- The second `[^1]` marker of `shi_Yumiaoya8` is gone:
  the page carries one marker and one definition,
  as the source does.
- The Han residue of `shi_Yumiaoya8` is gone:
  补佳乐 ships "Progynova",
  贴贴 "cuddling",
  「逆子」 "unfilial child".
  The Han that remains is the source's own
  (the Truman Show quote's Chinese line in the bilingual blockquote,
  which the archive does NOT carry (the archive has the English line and its attribution alone;
  corrected under the shi_Yumiaoya10 heading of 03:50 UTC,
  class eighty),
  and the camera list inside the source's HTML comment).
- "three bottles of a certain sedative-hypnotic medication" for 三瓶某种镇静催眠药 in the final-tweet paragraph,
  where `shi_Yumiaoya8` had "a large amount of a sedative-hypnotic";
  the substance stays unnamed,
  the count is the original's,
  and the archive never carried this paragraph,
  so the judges own it under the replicability test.
- `a [movie](...) that they both loved` for the film link where `shi_Yumiaoya8` had "[name of a film]";
  "let it rot" for 摆烂;
  the heading "Departure" again.
- "little yao-niang" with the footnote where `shi_Yumiaoya7` glossed inline;
  the bench chose the footnote this run and last.

### shi_Yumiaoya10 launched

`shi_Yumiaoya10` (frozen `d29feecd5`,
pid 1846495,
overlap 8)
launched 02:51 UTC on Bedrock and OpenRouter
(meters 117.48 USD Bedrock,
173.89 OpenRouter at launch),
the `CORPUS NAMES` line naming the same five.
Read the January 16 sentence first
(an attempt,
never a death),
then class seventy-eight again,
then the seven steps and three checks against `shi_Yumiaoya9` and the archive.

## shi_Yumiaoya8 read, 2026-09-22, 02:20 UTC: SETTLED, class seventy-eight found (a handle the corpus declares in another entry rendered as a pinyin of nothing), fixed in 545bff7b9; shi_Yumiaoya9 launched

`shi_Yumiaoya8` (frozen `0f9cd44da`,
pid 1772617,
overlap 8,
launched 01:33 UTC on Bedrock and OpenRouter,
Synthetic and Hyper dry)
SETTLED at 02:01 UTC on one attempt,
28 minutes by the artifact,
against `shi_Yumiaoya7`'s 62.
Seats asked 921,
52 retry rounds,
812 calls,
3 abandoned streams (0.0012 USD),
Qwen3.8-27B and glm-5.3 dark all run (118 and 78 asked,
all threw,
Synthetic dry),
Jev 24 of 24.
Meters:
Bedrock 118.13 to 117.81,
OpenRouter 174.96 to 174.39,
0.89 USD;
the spend lines say 0.86 USD on OpenRouter.

The checks.
Front matter equal,
4 of 4 headings,
the photo component as the source,
the death paragraphs (slice 13) on the page,
the closing quote's attribution once (`From *The Truman Show*`,
class seventy-four holding),
no `TA`,
no `finish_reason=error`,
no double blank line,
no `PublishedPageDisagrees` or weight-off line,
attempts 1,
no unendorsed,
gate-kept or slate-declined standing,
13 abstention lines with reasons,
no repeat or run-off finding,
7 em-dashes on a page whose archive has none
(`shi_Yumiaoya7` 6),
left as the owner said on 2026-09-21.
Eighteen slices,
14 changed by the translate lane,
2 by the repair lane.

Class seventy-eight.
The paragraph after the ICU names two people who died in the same month,
口香糖 and 单片,
and the page ships "the successive deaths of Xiaoguantang and Danpian".
"Xiaoguantang" is the pinyin of nothing
(口香糖 reads kouxiangtang;
`shi_Yumiaoya7` wrote "Kouxiangtang").
The corpus itself declares both people:
entry `a2581911655` carries `name: 奇妙的口香糖`,
`alias: 口香糖` and its English page `name: Magic Chewing Gum`,
`alias: Chewing Gum, jjlin1219`;
entry `interrgned` carries `name: 单片` and `alias: Danpian`.
The class seventy-one glossary reads names off the page's own archive,
which never carried this paragraph,
so nothing told the bench.
Three of five consolidation candidates transliterated,
one wrote "Chewing Gum",
one left the Han,
and the judges weighed "a phonetic transliteration" against "a translation of a form"
(minimax:
"The declared names section does not specify how 口香糖 and 单片";
gemma-4-26b:
"there is no declared name for 口香糖,
all candidates are somewhat guessing")
and chose candidate 5,
the transliteration.
Fixed in `1cdbeea75` and `545bff7b9`,
guard red first at `e284a6ebc`:
`corpus-name-index.ts` reads every entry's `page.md` and `page.en.md` front matter at the pin
(1.6 s over 92 entries,
148 names),
pairs each Han name or alias with every form the entry's translation declares,
and `pass-prepare.ts` joins the names this original carries to the identity context beside the page's own renderings,
with a `CORPUS NAMES entry=<id> named=<n>` line.
Measured over the pinned corpus before shipping:
猫猫 is declared by two entries and carried by 26 of 92 pages as the ordinary word,
so a form two entries declare names nobody and is dropped;
岁月,
不存在 and 贴贴 are declared handles other pages carry inside ordinary words
(灰暗岁月里,
不存在的真相,
贴贴图),
and Chinese prose marks no word boundary,
so the heading says a handle that is an ordinary word here stays a word and the judges decide.
After the drop 22 of 92 pages carry lines.
On `shi_Yumiaoya` the line names 贴贴 (the cuddle plan,
an ordinary word here),
盐田,
紫壑,
口香糖 and 单片;
`shi_Yumiaoya9` reads whether the bench renders the two handles as their entries do and leaves 贴贴计划 a plan.
Lint 0 and 0,
types clean,
full suite green on both commits.

Observations,
not classes.
The January 16 sentence ships "she ended her life by taking medication in the early hours of January 16"
for 吞下大量药物自杀,
two sentences before she wakes in the ICU;
`shi_Yumiaoya7` wrote "she tried to end her life".
The source says 自杀 and the ballots chose the "safe 'taking medication' phrasing" on the reader-protection rule;
English asserts the death the source's next sentence denies.
The judges keep deciding wording,
as the owner said of headings on 2026-09-21,
and the read of `shi_Yumiaoya9` watches the sentence.
The source's one `[^1]` on 小药娘 ships as a footnote this run
(`shi_Yumiaoya6` kept it,
`shi_Yumiaoya7` glossed it inline),
and the bench added a second `[^1]` on the 药娘 line the source leaves bare
(2 markers against the source's 1,
one definition,
which itself says the term applies throughout),
with the term in italics;
a judge praised "'minor yaoniang [1]' exactly as the house rules prescribe".
Han residue in the English:
"estradiol valerate tablet (补佳乐)",
"贴贴 (cuddling) plans",
"insulted as a 「逆子」";
`shi_Yumiaoya7` rendered all three.
The film link ships `[name of a film](...)` for the source's 一部电影 where `shi_Yumiaoya7` named the film;
the third heading ships "Departure" for 关于鱼喵的离世 where `shi_Yumiaoya7` wrote "About Yumiao's Departure".

`shi_Yumiaoya9` (frozen `545bff7b9`,
pid 1820803)
launched 02:14 UTC;
its log carries the `CORPUS NAMES` line with the five names.

## mikaela6 read, 2026-09-22, 01:35 UTC: SETTLED, classes seventy to seventy-seven holding on the mikaela page, three translator additions dropped by the bench as observations; shi_Yumiaoya8 launched

`mikaela6` (frozen `0f9cd44da`,
pid 1765655,
overlap 8,
launched 00:58 UTC on Bedrock and OpenRouter,
Synthetic and Hyper dry all run)
SETTLED at 01:29 UTC on one attempt,
40.0 minutes by the artifact,
against `mikaela5`'s 27 with Synthetic wet
(Qwen3.8-27B and glm-5.3 threw all 411 of their calls this run,
68 retry rounds against 54).

The classes on the page.
Class seventy holds:
the closing excerpt line curls its possessive after the link.
Class seventy-one holds:
the dialogue line reads 𝓠𝓾𝓪𝓷 and the comment block ships as the archive has it.
Class seventy-two holds:
炸柜 ships "Mika was outed" from the translate lane
(`mikaela5` wrote "outed against her will");
two abstentions on that round read the glossary meaning as forcible and declined "outed" as short of it,
gpt-oss read the term as running away,
and the lane settled on the plain passive.
Class seventy-three holds:
slice 28 (the Epilogue) consolidated on its first slate round,
and the Epilogue ships the source's fourteen lines one to one
(four prose lines,
"One night",
"During a phone call with Mika in a group chat",
seven quoted lines,
the XYN origin line,
the excerpt).
Class seventy-four:
no `repeats the line` finding and no doubled attribution.
Class seventy-five:
one triple newline on the page,
the archive's own after "Those were Mika’s last words",
and the publish weight check silent.
Class seventy-six:
the life in the past on every line
("was a 16-year-old from Beijing",
"She was a top student",
"She enjoyed playing").
Class seventy-seven:
not exercised,
the page has no archive-only block
(no archive block review line in the log).

Three translator additions the bench dropped,
observations and not classes
(the same reading as hulicaijia7's two notes and CuspariaKLSY5's gloss line):
"trans girl" in the opening line,
where the source line names only her age and Beijing and the repair lane won the slice;
the dollar conversion gloss after ¥20,000,
where the source names 两万块钱 alone;
and the parenthetical after Huai An naming her as 𝓠𝓾𝓪𝓷's high-school classmate,
which the archive took from its own translator comment
(the comment ships unchanged among the page's four).
The 老师 honorific on 怀安老师 is dropped with it.
The excerpt's magazine title ships in pinyin
("Mingju,
Xingliuge,
Xinghenzhi")
where the archive wrote "Chronicle" and `mikaela5` a word-for-word rendering;
bench variance on a title with no glossary form.
One spaced em-dash enters on the "Love never really goes away" line
against the archive's ellipsis
(page 4 em-dashes to the archive's 5),
left as the owner said on 2026-09-21.
One abstention cited the reader-protection rule against "Progynon Depot"
(the HRT purchase on slice 14,
not the death),
deepseek-v4.1-flash alone on one repair round;
the name ships as the archive has it.

The page.
Thirty-three slices,
25 changed by the translate lane,
16 by the repair lane,
21 changed on the page,
seven standing as the archive.
Front matter equal,
4 of 4 headings,
2 PhotoScroll,
2 footnote definitions,
37 blockquote lines as the archive,
4 translator comments as the archive,
no `TA`,
no `finish_reason=error`,
no `PublishedPageDisagrees` or weight-off line,
attempts 1,
two unendorsed standings as findings
(slices 0 and 29,
both settled-neither with the archive declined),
gate-kept and slate-declined standings as findings,
52 abstention lines with reasons,
no repeat or run-off finding.

Seats asked 2062,
68 retry rounds,
1822 calls,
7 abandoned streams (0.0043 USD),
Qwen3.8-27B dark
(284 asked,
284 threw)
and glm-5.3 dark
(127 asked,
127 threw)
on Synthetic dry,
GLM-5.3-Flash 7 of 110 threw,
Jev 59 asked,
59 usable.
Meters:
Bedrock 118.70 to 118.13,
OpenRouter 175.83 to 174.96,
1.44 USD;
the spend lines say 1.41 USD.

Nothing to fix from this read.
`shi_Yumiaoya8` (frozen `0f9cd44da`,
pid 1772617,
overlap 8)
launched 01:33 UTC as the stalest entry in the queue
(last read on `364b83557`),
meters at launch Bedrock 118.13 and OpenRouter 174.96.

## zheermao3 read, 2026-09-22, 00:55 UTC: SETTLED, class seventy-seven live (the email label revised as a label, a bulleted-list revision refused on shape); the Arts letter revised this time from the photo transcripts; queue continues

`zheermao3` (frozen `0f9cd44da`,
pid 1734549,
overlap 8,
launched 00:10 UTC on Bedrock and OpenRouter,
Synthetic wet at launch and dry from 00:14 UTC,
Hyper dry)
SETTLED at 00:43 UTC on one attempt,
33.5 minutes by the artifact.

Class seventy-seven live.
The label block ("English translation of the preceding email conversation:")
was reviewed as its own block again.
Six reviewers heard,
two eligible assessments:
gemma-4-e2b offered a bulleted list for the one-paragraph label,
and the shape floor withheld it with the warn line
"archive-revision-refused (google.gemma-4-e2b):
the block is paragraph and the revision is list (bulleted);
a revision keeps the block's own shape".
The slate then held one admissible revision beside the incumbent,
both labels;
Mercury's "English translation of the email conversation:"
won at weight 3.5 across five ballots,
four of them citing the positional-reference rule against "preceding"
(gemma-4-26b,
deepseek-v4.1-flash,
Mercury,
GLM-5.3-Flash),
gemma-4-e2b for the incumbent.
The label ships as a label;
the nested blockquote carrying the whole exchange stands behind it as the archive has it.
No letter was offered this run,
so the refusal the guard covers
(a four-block letter for a one-paragraph label)
was not exercised live;
the list refusal is the same floor on the same block.

The other three archive blocks.
The reply blockquote
(One Among Us's reply inside the exchange):
five revisions in six candidates,
GLM-5.3-Flash's won at weight 2.5 across five ballots
(deepseek-v4.1-flash,
Mercury and GLM-5.3-Flash for it,
gemma-4-e2b for a fourth candidate,
gpt-oss declined all),
and its one change against the archive is the line the archive left as a JSX translator comment
(`{/* 可以避开黑暗的锋芒，到那些有光的地方躲一躲。 这句不知道如何翻译 */}`),
now rendered
"You can dodge the darkness's sharpest edge and take shelter for a while in the places that hold light."
`zheermao2` had rewritten the whole reply from the transcript;
this run the review kept the archive's wording and filled the gap.
The Arts letter label ("English translation of the preceding letter:"):
five heard,
GLM-5.3-Flash lost on a schema mismatch
(it answered a disposition the sheet does not offer,
"editorial-context"),
no reviewer revised,
so the block stands as the archive wrote it,
"preceding" and all,
where `zheermao2` had revised it to "letter from Arts" on five ballots.
Bench variance on the same rule,
not a class:
the review is single-round by design.
The Arts letter itself
(the English of the two photographed pages):
two revisions in three candidates,
deepseek-v4.1-flash's won at weight 2.5 across six ballots
against the incumbent's 2
(gemma-4-26b,
deepseek at 0.5 and Mercury for it;
gemma-4-e2b and minimax for the archive's;
gpt-oss at 0.5 for the third),
the ballots citing the photo transcripts
("the corroborated letter text from photo1/photo2")
and the renderings of 罪恶 and 不会自责.
`zheermao2` had retained the archive's letter 4 of 6.
The revision renders 猫猫 as "cat" where the archive wrote "cat-like person";
the corpus renders 猫猫 as "cat" on every other page that carries it
(Everythings99,
Proselyte093,
Considerate_cat,
lin10104,
measured with `rg` over `page.md` and `page.en.md`),
so no glossary seed follows.
It carries one em-dash
(the em-dash ahead of the closing "for example" clause)
on a page whose archive has none,
left as the owner said on 2026-09-21.

The page.
Fourteen slices,
six changed by the translate lane,
two by the repair lane,
eight changed on the page,
six standing as the archive.
The life stays in the past on every line.
Front matter equal,
2 of 2 headings,
photo components and the contributor comment as the archive,
no `TA`,
no `finish_reason=error`,
no double blank line,
no `PublishedPageDisagrees` or weight-off line,
attempts 1,
no unendorsed standing,
four gate-kept standings (slices 1,
5,
6,
9),
two slate-declined standings (11,
12),
14 abstention lines with reasons,
no repeat or run-off finding.
The body diffs against `zheermao2` on the ordinary slices
(the milk-tea line,
the Zhihu line,
the Telegram paragraph)
are the bench choosing differently between valid renderings;
none crosses a rule.

Seats asked 850,
39 retry rounds,
731 calls,
15 abandoned streams (0.03 USD),
glm-5.3 dark all run (53 asked,
53 threw,
Synthetic dry),
Kimi-K3 7 of 13 threw,
Jev 23 asked,
23 usable.
Meters:
Bedrock 118.96 to 118.70,
OpenRouter 176.17 to 175.83,
0.60 USD;
the spend lines say 0.59 USD on OpenRouter.

Nothing to fix from this read.
The queue continues one entry at a time.

## zheermao2 read, 2026-09-22, 00:20 UTC: SETTLED, but the archive block review replaced the email intro label with a four-block letter cut mid-sentence (class seventy-seven, fixed in 0f9cd44da); zheermao3 launched

`zheermao2` (frozen `d70087757`,
pid 1717905,
overlap 8,
launched 23:27 UTC on Bedrock and OpenRouter,
Synthetic dry at launch and wet at the end,
Hyper dry)
SETTLED at 23:59 UTC on one attempt,
46 minutes.

The page.
Sixteen slices,
eleven changed by the translate lane,
six by the repair lane,
ten changed on the page.
The life was already told in the past by the archive,
and every line stays in the past.
One em-dash on the page against the archive's none
(the reply letter's "worry—we'll",
written by the archive block review's revision of the reply blockquote),
left as the owner said on 2026-09-21.
No double blank line,
no `PublishedPageDisagrees` or weight-off line,
front matter equal,
no `TA`,
no `finish_reason=error`,
attempts 1,
headings,
list items,
photo components and the contributor comment as the archive,
no unendorsed standing,
two gate-kept standings (slices 8 and 13),
two slate-declined standings (4 and 10),
22 abstention lines with reasons,
no repeat or run-off finding.
Seats asked 979,
18 retry rounds,
912 calls,
10 abandoned streams (0.03 USD),
glm-5.3 dark all run (70 asked,
70 threw),
Kimi-K3 5 of 72 threw;
Jev 23 asked,
23 usable.
Meters:
Bedrock 119.13 to 118.96,
OpenRouter 176.51 to 176.17,
0.51 USD;
the spend lines say 0.50 USD on OpenRouter.

Class seventy-seven.
The archive carries,
under the third photo,
a label ("English translation of the preceding email conversation:")
and then the English rendering of the exchange as a nested blockquote;
neither is in the original,
whose text for that exchange is the picture.
The archive block review reviewed the label as its own block.
Six reviewers were heard;
one revision was admissible,
and the selection log says what it was:
"Candidate 1 provides a complete and accurate translation of the third photo's text (the email conversation).
Candidate 2 is an empty repeat of the header"
(gemma-4-26b),
"Candidate 1 provides the actual email translation content required for the block ...
Candidate 2 contains only the header and omits the translation entirely"
(Mercury),
against deepseek-v4.1-flash,
which read the block for what it is
("the current archive block,
which consists of the line 'English translation of the preceding email conversation:'
and nothing else.
Candidate 2 reproduces exactly that line"),
and gpt-oss,
which declined both.
Candidate 1 won at weight 2.5 across five ballots.
It is a four-block plain-text letter
(a greeting,
two paragraphs,
a sign-off "Warm wishes / Chun Zhi"),
its third paragraph ending mid-sentence
("the stories that are living,
are much more"),
Zheermao's side of the exchange absent,
and it shipped where the label stood,
ahead of the blockquote that already carries the whole exchange.
`zheermao1` had shipped a revised label
("English translation of the email exchange with One Among Us:").

The class is the review's admission.
`replacementCandidates` admitted any revision that kept the contributor identities;
nothing held a revision to the block's own shape,
while the lanes hold every candidate to the page's block shape through `translate-validate.ts`.
Fixed in `0f9cd44da`:
`archive-revision-shape.ts` reads the block and the revision through `readSliceSkeleton`
and withholds a revision whose blocks differ in count,
kind or detail
(finding `archive-revision-refused (<model>): the block is paragraph and the revision is paragraph, paragraph, paragraph, paragraph; a revision keeps the block's own shape`);
a removal (an empty revision) stays admissible;
a block the grammar cannot read holds nothing.
`replacementCandidates` moved to `archive-replacement-candidates.ts`
so the stage stays under its line budget,
and returns the withheld findings,
which the stage logs and carries into the block's findings.
Guard red first at `42a6ab2de`
(the label revised into the letter shipped the letter),
green at `0f9cd44da` with no selection round bought over the emptied slate;
lint and types clean.
The full suite was red at `0f9cd44da` on one older guard,
`archive-selection-boundary.unit.test.ts`,
whose competing partial proposal was a bare paragraph offered for a blockquote block,
exactly what the floor refuses;
the fixture now keeps the block's shape (`8c2dc7e74`),
and the full suite is green at `8c2dc7e74` (exit 0,
no FAIL line).
The other three archive blocks on this page went as they should:
the letter from Arts retained (the incumbent won weight 4 of 6),
its label revised from "preceding letter" to "letter from Arts"
(five ballots citing the rule against positional references),
and the reply blockquote revised from the picture transcript
(deepseek's candidate at weight 2 across 5).

`zheermao3` (frozen `0f9cd44da`,
pid 1734549)
launched at 00:12 UTC to read the label standing,
the same three archive blocks,
and the seven steps and three checks again.
