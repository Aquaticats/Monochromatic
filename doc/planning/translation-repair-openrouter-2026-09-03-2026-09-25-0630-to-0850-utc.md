# OpenRouter and the corpus passes: 2026-09-25, 06:30 to 08:50 UTC

What was launched,
read and decided on 2026-09-25 06:30 to 08:50 UTC.
Entries:
hulicaijia20,
shi_Yumiaoya35,
hulicaijia19,
shi_Yumiaoya34,
shi_Yumiaoya32,
shi_Yumiaoya31,
shi_Yumiaoya30.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## hulicaijia20 read, 2026-09-25: class one hundred thirty-four live; douhua, a name's casing and a pinyin tone (classes one hundred thirty-five to one hundred thirty-seven)

### What ran

hulicaijia20 on frozen `4641ff1d2` (pid 3632174):
SETTLED at 22:28 UTC in 81.9 min,
one attempt,
72 slices,
41 changed on the page.
glm-5.3 sat dark all run:
its seat is Hyper-only and Hyper was dry,
which is the seating's design.

### What the page shows

Class one hundred thirty-four is live.
The log carries `canadian-form-rewritten (slice 62: "liquorice" to "licorice", ...)`,
the page reads "April 29" and "May 4" with no "29th April" or "4 May",
"compound licorice tablets" twice,
no "pharmaceutical sales representative",
and the en_CA scan is clean ("favourite",
"city centre";
the only "center" is the archive's CSS `text-align`).

Three things read wrong,
each now a class.

-   The closing quote wrote "she was something like tofu pudding."
    where the front matter,
    the archive and every other slice write "douhua" for 豆花 (class one hundred thirty-five).
-   One line wrote "the Jiefangbei pedestrian street"
    where the archive writes "Jiefangbei Pedestrian Street" mid-sentence in the body and a caption and never otherwise
    (class one hundred thirty-six).
-   The archive's translator note glossed 金刚烷胺 as "jīn gāng wǎn’àn" to show the pun on 晚安 (wǎn’ān),
    and the page carried it;
    烷 has one reading,
    wán (class one hundred thirty-seven).

### What was built

-   Class one hundred thirty-five (`46731b86e`,
    guard red first `c18e78a6e`):
    豆花 seeded in the wording glossary as "douhua",
    refusing "tofu pudding",
    "tofu flower",
    "bean curd pudding",
    "soybean pudding" and "tofu custard".
-   Class one hundred thirty-six (`76341f8e4`,
    guard red first `3d71c35a0`):
    `restoreArchiveNameCasing` in `corpus-run/archive-name-casing.ts`,
    after the class one hundred twenty-one casing pass.
    A run of two or more title-case words the archive writes mid-sentence at least twice,
    outside headings,
    and never in another casing is restored where a shipped slice keeps the first word and lowers a later one.
    Replayed over 36 shipped pages:
    the Jiefangbei line on hulicaijia19's page and nothing else.
-   Class one hundred thirty-seven (`2d5113b7d`,
    guard red first `db88480e5`):
    `correctPinyinPage` in `corpus-run/pinyin-tone.ts`,
    after the Canadian forms pass,
    on every slice the page carries.
    Where a parenthesis pairs a Han run with tone-marked pinyin,
    one syllable to a character,
    a syllable whose single-reading character differs from it in tone alone takes the character's tone (pinyin-pro).
    A polyphonic character,
    a count mismatch,
    another syllable and toneless pinyin stand aside.
    The slice loop it shares with `canadianizePage` moved to `rewriteEverySlice` in `corpus-run/page-slice-rewrite.ts`.
    Replayed over hulicaijia20's page:
    "wǎn" to "wán" for 烷 and nothing else,
    晚安's "wǎn’ān" untouched;
    over 36 shipped pages:
    no other rewrite.
    Lint 0/0,
    types clean,
    suite `suite-class137.log` 1139 PASS,
    no FAIL.

A `--fix` lint pass rewrote the new file's code-point spreads of `String.prototype.slice` into plain strings
(`unicorn/no-useless-spread` reads `.slice` as an array method);
the file now uses index scans,
and the autofix is filed as issue 563.

## shi_Yumiaoya35 and hulicaijia19 read, 2026-09-25: class one hundred thirty-three live, Canadian dates and spellings on untouched slices (class one hundred thirty-four)

### What ran

shi_Yumiaoya35 on frozen `6a24362ff` (pid 3272330,
launched 08:48 UTC):
SETTLED at 09:23 UTC in 33.5 min,
one attempt,
18 slices,
14 changed.
hulicaijia19 on frozen `6a24362ff`:
SETTLED at about 10:03 UTC in 72.0 min,
one attempt,
72 slices,
39 changed.

### What the pages show

Class one hundred thirty-three is live on shi_Yumiaoya35:
"Yumiao took her very first Progynova" and "it had already left her deeply traumatized";
no "of her life" beside the pill and no "great trauma".
The en_CA scan of shi_Yumiaoya35 is clean.

hulicaijia19 carried three forms no lane had rewritten,
each on a slice whose archive text stood:
"29th April was my birthday",
"On 4 May,"
and "compound liquorice tablets" (twice),
beside the bench's own "March 13".
And 药代 shipped as "In her role as a pharmaceutical sales representative,"
where the page means she resold medication to others.
The drug names the bench dropped on the death passage are left as they are:
the house reader-protection rule covers drugs tied to a death.

### What was wrong (class one hundred thirty-four)

The en_CA rule of class one hundred thirty-two reaches only the sheets,
so it reaches only slices a lane rewrites.
A slice whose archive rendering stands untouched ships the archive's own date order and spelling.
The policy also named no date order,
and 药代 had no glossary entry.

### What changed

`canadianizePage` (`src/corpus-run/canadian-forms.ts`) runs in `guardPageAssembly` after the casing pass,
over every slice as the page will carry it,
the untouched ones included:

-   `monthFirstDates` (`canadian-date.ts`) rewrites a day number (1 to 31,
    ordinal suffix or not)
    before a full month name month first,
    a four-digit year after it taking a comma ("13 March 2024" to "March 13,
    2024");
    a day that closes a range ("1st to 3rd June") stands aside.
-   `canadianSpellings` (`canadian-spelling.ts`) respells a closed list of lower-case words
    (colour,
    centre,
    grey,
    licorice,
    travelled,
    catalogue,
    defence,
    realize,
    analyze and their forms);
    a capitalised word is a name or a title and stands,
    and words with a second sense (meter,
    check,
    tire,
    mum) are left to the judges.
-   `protectedRanges` (`prose-ranges.ts`) keeps both out of front matter,
    tags and their attributes,
    JSX expressions,
    link destinations,
    bare URLs,
    inline and fenced code and HTML comments;
    a dot counts as a path neighbour only with a letter or digit on its far side.
-   Front-matter slices and slices overlapping a span sealed as the English original stand aside.
    A changed slice no lane replaced gets a restored row,
    and each change prints `canadian-form-rewritten (slice N: …)`.

`HOUSE_POLICY_BLOCK` now says dates are written month first (April 29,
May 4,
March 13,
2024),
never day first,
and lists licorice among the spellings.
`WORDING_GLOSSARY` seeds 药代 ("sold medication",
"selling medication",
"medication seller";
refusing "pharmaceutical sales representative" and its shorter forms).

Guards red first `acb5a9ccb` (`canadian-forms.unit.test.ts`,
`canadian-forms-policy.unit.test.ts`),
fixed `4641ff1d2`;
lint 0/0,
types clean,
suite `suite-class134.log` 1136 PASS,
no FAIL.
Replayed over shipped pages with `canadianizeText`:
hulicaijia19 four rewrites (the two dates,
"liquorice" twice);
XingZ6010 nine ("center of the mourning hall",
"behavior" five times,
"gray portraits",
"recognise",
"honored"),
the four `align="center"` and `text-align: center` attributes untouched;
mikaela17 "humor";
zheermao13 "favorite flavor";
shi_Yumiaoya35,
CuspariaKLSY15 and yingying11 none.
Frozen `.frozen-dist-4641ff1d2`;
hulicaijia20 launched (pid 3632174).

### What the owner said

Asked whether the en_CA convention should also cover the repo's own docs,
comments and prose through `AGENTS.md`:
"Translated pages only".
The rule stays in the package (the house policy,
the glossaries,
this pass and the README) and is not an `AGENTS.md` rule.

## shi_Yumiaoya34 read, 2026-09-25, 08:50 UTC: classes one hundred thirty-one and thirty-two live, two life-event calques (class one hundred thirty-three)

### What ran

Frozen `9bdeae9c6`,
corpus-pass pid 3235678,
launched 08:16 UTC.
SETTLED at 08:45 UTC in 28.3 min,
one attempt,
948 calls,
2 retry rounds,
10 abstention lines,
about 0.23 USD by the meters (Bedrock 40.13 to 39.97,
OpenRouter 82.77 to 82.70).
glm-5.3 answered 77 of 77;
gemma-4-e2b 117 of 118,
deepseek 126 of 129,
Qwen 119 of 120.

### What the page shows

Every shi_Yumiaoya32 structural check holds.
Class one hundred thirty-one on the page:
no "counselor",
"correctional facility",
"took the initiative to propose",
"rescue plan",
"days of resuscitation",
"replace Yumiao's perspective" or "of such a good nature".
Class one hundred thirty-two on the page:
a whole-word scan of `page.en.md` finds no OD,
jk,
Gaokao,
color,
center,
behavior,
favorite,
counselor,
adviser,
petrol,
maths,
mum,
realise,
recognise or romanised.

### What was wrong (class one hundred thirty-three)

Under the owner's standing instruction of 2026-09-25 ("whenever you see anything that can be translated better do it"),
two sentences read word for word:
人生中的第一颗补佳乐 shipped as "the first Progynova of her life",
and 精神已留下了巨大的创伤 as "her mind had already been left with great trauma".
Each term stands in one pinned entry (this one),
so the refused forms reach only its paragraphs.

### What changed

`WORDING_GLOSSARY` gains 人生中的第一颗 ("her very first",
"her first-ever";
refusing "of her life",
"of his life",
"of its life")
and 留下了巨大的创伤 ("left her deeply traumatized",
"deeply traumatized",
"left deep scars";
refusing "left with great trauma",
"left with a huge trauma",
"great trauma",
"huge trauma").
Guard red first `47fbf2253` (`rendering-glossary-life-event.unit.test.ts`),
fixed `6a24362ff`;
lint 0/0,
types clean,
suite `suite-class133.log` 1134 PASS,
no FAIL.
Frozen `.frozen-dist-6a24362ff`;
shi_Yumiaoya35 launched 08:48 UTC (pid 3272330).

## Canadian English, 2026-09-25, 08:17 UTC: class one hundred thirty-two

### What the owner said

Asked "adviser - is this en_CA?"
about the class one hundred thirty-one rendering for 辅导员,
then:
"The convention is and should be en_CA.
It should have been documented and in the prompts."

### What was wrong

No sheet named the English the page is written in.
The package said nothing about it;
the corpus archive is mixed
(across `page.en.md`:
center 22 against centre 5,
behavior 10 against behaviour 1,
realize 8 against realise 0,
favourite 14 against favorite 4,
travelled 3 against traveled 0).
So the bench mixed varieties:
shi_Yumiaoya32 shipped "Her counselor",
"the head of year" and "Like a petrol car",
and the glossaries seeded British and American forms as the renderings to use
("head of year",
"year head",
"flatmate",
"petrol car",
"behavior-correction center",
"student adviser").
The house policy itself wrote "romanised",
"neutralise" and "localisation".

### Which forms

McGill's style guide,
which takes the Canadian Press Stylebook and the Canadian Oxford as its references,
writes -our,
-re and -ize and says "advisor,
not adviser",
so 辅导员 is "student advisor".
Canadian schools say Grade 12,
not Year 12 (the breathinglanguage.com Canadian school vocabulary notes),
so 年级组长 is "grade coordinator" and "head of year" is refused.
燃油车 is "a gas-powered car" and "petrol car" is refused.

### What changed (class one hundred thirty-two)

`HOUSE_POLICY_BLOCK` gains a bullet naming Canadian English (en_CA) with its spelling and vocabulary,
and says English already on the page in another variety's spelling is respelled as a correction of form,
never a change of meaning,
while names,
titles and quoted English keep their own spelling.
The policy's own -ise forms are respelled -ize.
The glossaries seed Canadian renderings only;
`flatmate` is dropped from 同居者.
The package README gains an "English variety" section.
Guard red first `53ddd36de` (`canadian-english-policy.unit.test.ts`:
the rule on the house policy and on the translate and select sheets,
no British -ise form in the policy,
no British or American rendering in either glossary,
"petrol car" and "head of year" refused,
the Canadian words accepted),
fixed `9bdeae9c6`;
the class one hundred twenty-six,
thirty and thirty-one tests and the house-policy test
updated to the Canadian forms;
lint 0/0,
types clean,
suite `suite-class132b.log` 1133 PASS,
no FAIL.

shi_Yumiaoya33 (frozen `fe56bffed`,
launched 08:07 UTC) was killed at 08:09 UTC under always-kill-and-relaunch
before it reached the lanes.
Frozen `.frozen-dist-9bdeae9c6`;
shi_Yumiaoya34 launched 08:16 UTC (pid 3235678).

## shi_Yumiaoya32 read, 2026-09-25, 08:05 UTC: class one hundred thirty live, seven wording calques (class one hundred thirty-one)

### What ran

Frozen `6637865d6`,
corpus-pass pid 3135802,
launched 07:36 UTC.
SETTLED at 08:02 UTC in 26.7 min,
one attempt,
965 seats,
965 calls,
one retry round,
6 abstentions,
about 0.16 USD on Bedrock and 0.07 USD on OpenRouter by the meters
(40.31 to 40.15,
82.86 to 82.79).
Every provider wet;
every seat answered every call it was asked (glm-5.3 68 of 68).

### Checks

Every shi_Yumiaoya31 check holds:
front matter equal,
4 headings as the archive,
no TA,
no double blank line,
no error finish,
the register check clean,
classes one hundred twenty-six to twenty-nine on the page.
Class one hundred thirty on the page,
none of its refused forms present
(the count read zero;
the same count over "top gear trio",
"petrol car" and "wheels of fate" read 3):
"the Top Gear trio's car shows",
"Like a petrol car",
"the crushing academic pressure left her barely able to breathe",
"both of her arms were covered in scars",
"a huge mistake on her university applications",
"the wheels of fate began to turn",
"she also tried hard to get hospital treatment",
"Yumiao's parents also played a large part in her death",
"at her cuddle meetups".

### Seven wording calques (class one hundred thirty-one)

Under the owner's standing instruction of 2026-09-25,
seven more renderings join the rendering glossary:

- 辅导员 shipped "Her counselor",
  which tells an English reader she was seeing a therapist;
  the page says "student adviser",
  a university's student affairs officer.
- 矫正机构 shipped "a correctional facility",
  which in English is a prison;
  the page says "behaviour-correction centre".
- 主动提出了与他分手 shipped "took the initiative to propose breaking up with him";
  the page says she "was the one who suggested" it.
- 营救计划 shipped "a rescue plan" for work to free an arrested housemate;
  the page says "efforts to free" her.
- ICU 抢救了六天 shipped "after six days of resuscitation in the ICU";
  resuscitation takes minutes,
  so the page says six days in intensive care.
- 代替鱼喵的视角 shipped "will replace Yumiao's perspective";
  the page says the camera looks on the world "in her place".
- 性格非常好的人 shipped "a person of such a good nature";
  the page says she "was very good-natured".

抢救 alone stands in thirteen pinned entries (xixi_yuexi's "我还能抢救一下" means "salvage"),
so only `ICU 抢救` is seeded;
代替 stands in two (hakureico's "in place of a cold obituary" is right),
so only the perspective calques are refused.
The other five terms stand in this entry alone,
and no entry carries both 辅导员 and a counsellor (心理咨询).
The entries sit in `rendering-glossary-wording.ts`,
spread into `RENDERING_GLOSSARY`.
Guard red first `74322469d` (`rendering-glossary-wording.unit.test.ts`:
seeds,
refuses the seven calques,
accepts the English),
fixed `fe56bffed`,
lint 0/0,
types clean,
suite `suite-class131.log` 1132 PASS,
no FAIL.
Frozen `.frozen-dist-fe56bffed`;
shi_Yumiaoya33 launched 08:07 UTC (pid 3186607).

## shi_Yumiaoya31 read, 2026-09-25, 07:35 UTC: class one hundred twenty-nine live, nine idiom calques (class one hundred thirty)

### What ran

Frozen `a719b81f6`,
corpus-pass pid 3059542,
launched 06:58 UTC.
SETTLED at about 07:29 UTC in 30.6 min,
one attempt,
983 seats,
981 calls,
no retry rounds,
10 abstentions,
about 0.16 USD on Bedrock and 0.07 USD on OpenRouter by the meters
(40.47 to 40.31,
82.93 to 82.86).
Every provider wet;
glm-5.3 answered 80 of 80 with Hyper back.

### Checks

Every shi_Yumiaoya30 check holds:
front matter equal,
4 headings as the archive,
no TA,
no double blank line,
no error finish,
"year head",
"psychologically bullied by an underage trans girl",
"memorial dice",
"housemate",
"overdosing" and "Don't overdose",
"sailor uniform",
"National College Entrance Examination",
"teachers' college",
"became a top student".
Register check clean:
none of OD,
ODing,
jk,
TGT,
MtF,
gearhead or slacked off;
MtX spelled out where it first appears.
Class one hundred twenty-nine on the page:
"Yumiao's death had many causes",
"so Yumiao,
as an exception,
received no punishment",
"she stopped trying in her studies" and "she stopped trying completely",
"This drove Yumiao to the edge of madness",
"go on observing this vast world".

### Nine idiom calques (class one hundred thirty)

Under the owner's standing instruction of 2026-09-25,
nine more renderings that read badly join the rendering glossary,
each term carried by this entry alone in the pinned corpus:

- 三剑客汽车节目 shipped "the Three Musketeers car show";
  English viewers say "the Top Gear trio".
- 燃油车 shipped "like a fuel-powered car";
  the page says "a petrol car".
- 喘不过气 shipped "the intense academic pressure left her breathless";
  "breathless" reads as excited or winded,
  so the page says the pressure was crushing or that she could barely breathe.
- 密密麻麻的伤痕 shipped "covered in densely packed scars";
  the page says her arms were covered in scars.
- 志愿填写 shipped "a major mistake in filling out her application preferences";
  the page says her university applications.
- 命运的齿轮 shipped "the gears of fate began to turn";
  the English idiom is "the wheels of fate".
- 相关医院 shipped "sought treatment at the relevant hospitals";
  the page says she sought hospital treatment.
- 巨大的影响 shipped "Yumiao's parents also had an enormous influence on her death";
  the page says they played a large part in her death.
- 贴贴计划 shipped "through her cuddling plan she met many companions";
  the page names the cuddle meetups.

Left as written,
each read against its source line:
"She seriously hurt herself" is 其进行了较为严重自伤行为,
right;
"bipolar disorder with other psychotic features" keeps the original's 其他;
"negative energy" for 负能量 is ordinary English;
"did not want to disturb him" for 不想打扰他 reads as English.

`rendering-glossary.ts` crossed the 300-line budget with the new entries,
so class one hundred twenty-nine's entries moved to `rendering-glossary-phrasing.ts`
and class one hundred thirty's sit in `rendering-glossary-idiom.ts`,
both spread into `RENDERING_GLOSSARY`.
Guard red first `3774ee79a` (`rendering-glossary-idiom.unit.test.ts`:
seeds,
refuses the nine calques,
accepts the English),
fixed `6637865d6`,
lint 0/0,
types clean,
suite `suite-class130.log` 1131 PASS,
no FAIL.
Frozen `.frozen-dist-6637865d6`;
shi_Yumiaoya32 launched 07:36 UTC (pid 3135802).

## shi_Yumiaoya30 read, 2026-09-25, 06:55 UTC: classes one hundred twenty-six to twenty-eight live, five phrasing calques (class one hundred twenty-nine)

### What ran

Frozen `c432737d5`,
corpus-pass pid 2972665,
launched 06:29 UTC.
SETTLED at 06:54 UTC in 25.1 min,
one attempt,
965 seats,
881 calls,
6 retry rounds,
7 abstention lines,
about 0.13 USD on Bedrock and 0.32 USD on OpenRouter by the meters
(40.61 to 40.48,
83.25 to 82.93).
Hyper dry all run:
glm-5.3,
a Hyper-only seat,
was asked 82 times and refused at call time each time
(`NoProviderForModelError`,
out of budget),
which costs nothing and leaves quorum unaffected (class twenty-six).

### Checks

Front matter equal,
4 headings as the archive,
no TA,
no double blank line,
no error finish.
Class one hundred twenty-six on the page:
"The head of year",
"psychologically bullied by an underage trans girl",
"ashes will be made into memorial dice",
"housemate" on both 同居者 lines.
Class one hundred twenty-seven:
"overdosing day and night" and "Don't overdose ... not to overdose or misuse medication" for both OD lines,
"sailor uniform",
"college entrance examination" (an entry rendering;
no "Gaokao").
Register check (class one hundred twenty-eight) clean:
no OD,
ODing,
jk,
TGT,
MtF,
gearhead or slacked off on the page;
"The Grand Tour" for TGT;
HRT and ICU kept;
"trans girl" for 小药娘.
The rooftop in the departure paragraph is not on the page,
as the reader-protection rule asks.

### Five phrasing calques (class one hundred twenty-nine)

Under the owner's standing instruction of 2026-09-25,
five renderings that read badly join the rendering glossary:

- 原因是多方面的 shipped "There were many sides to the cause of Yumiao's death";
  the page says the death "had many causes".
- 鱼喵这个特例 shipped "so this exception,
  Yumiao,
  received no punishment";
  the page says "Yumiao,
  as an exception,".
- 摆烂 shipped "Yumiao's attitude toward studying turned to one of giving up";
  the page writes the plain verb ("stopped trying"),
  and "slacked off" is refused as slang (class one hundred twenty-eight).
  lxy writes 偶尔摆烂 for taking it easy now and then,
  so "took it easy" is an entry rendering.
- 陷入癫狂 shipped "This pushed her mental state into madness";
  the page says it drove her to the edge of madness.
  The entry keys on the verb because XingZ60 writes 癫狂 alone for an outlook others call mad.
- 万千世界 shipped "continue to observe this myriad world";
  the page says "this vast world".

Guard red first `a5be55ae8` (`rendering-glossary-phrasing.unit.test.ts`:
seeds,
refuses the five calques and "slacked off",
accepts the English),
fixed `a719b81f6`,
lint 0/0,
types clean,
suite `suite-class129.log` 1130 PASS,
no FAIL.
Frozen `.frozen-dist-a719b81f6`;
shi_Yumiaoya31 launched 06:58 UTC (pid 3059542).

"Her mathematics grades looked very interesting" for 数学成绩看上去十分有趣 stays:
the original is wry,
and the English keeps the wryness in plain words.

## The page's register, 2026-09-25, 06:30 UTC: plain written English on every sheet (class one hundred twenty-eight)

The owner answered the three renderings left as the original wrote them
("overdosing",
"sailor uniform",
"National College Entrance Examination";
class one hundred twenty-seven),
then wrote:
"I'm not sure why you allowed the very informal terms like OD to go through."

### Why OD passed

The reads treated the original's register as the authority:
a page that writes OD was read as faithful when the English wrote OD or ODing.
Nothing on any sheet said otherwise.
The house policy's community-vocabulary bullet keeps
"an abbreviation local to the person's world,
such as 大证 or WER"
and glosses it,
which a judge could read as licence for OD,
jk and TGT,
and no bullet named the register of the English itself.
So shi_Yumiaoya27 shipped "ODing",
"jk skirt" and "the TGT series",
and no ballot held them against a candidate.

### What the corpus writes

A sweep of the pinned originals found the shorthand the rule has to reach:
MtF,
MTF and mtf
(also inside links such as `mtf.wiki` and handles such as `siyan_MTF`),
SRS,
GAHT,
POV,
OST,
ACG,
RPG,
DID,
OD,
jk and MtX.
No archive writes "gaokao" or a bare OD.
Mechanical floors for each were rejected:
a substring floor on MtF would refuse links and handles,
and spelling a term out once and abbreviating it after is ordinary formal practice
that a floor cannot tell from carried shorthand.

### The rule

`src/house-policy.ts` gains a bullet after the community-vocabulary bullet,
so it reaches every writer and judge sheet through `HOUSE_POLICY_BLOCK`:
the page is plain written English for a general reader whatever the ORIGINAL's register;
chat shorthand and internet slang are rendered by the plain words
(OD is overdose,
jk 裙 a sailor uniform,
MtF trans woman,
TGT The Grand Tour);
the English adds no slang of its own (gearhead,
slacked off);
an abbreviation a general reader would not know is spelled out where it first appears;
known abbreviations (HRT,
ICU) and names in Latin letters (QQ,
a handle,
a link) stay;
an identity label with no everyday English equivalent (MtX) is kept and glossed;
quoted speech keeps its warmth but not its shorthand;
and carried shorthand is a defect,
never faithfulness to the ORIGINAL's tone.
Guard `src/register-policy.unit.test.ts`,
red first at `194eca498`,
asserts the rule on the house policy,
the translate sheet and the select sheet.
Built in `c432737d5`;
lint 0/0,
types clean,
suite `suite-class128.log` 1129 PASS,
no FAIL.

### The test the rule broke

The first suite run failed one case in `src/archive-block-review-stage.unit.test.ts`:
"archive review left the block unresolved" read 0 of 3 where the case expects 1 of 3.
The case scripted one anchoring reply by roster position (`ROSTER[0]`),
but `gatherStageVoices` asks seats in the order `rotatedBench` gives,
a rotation keyed on the prompt,
and the quorum of three on a four-seat roster closes without the last seat asked.
The longer policy changed the rotation and put `ROSTER[0]` fourth.
The run with the policy of `HEAD` passed and five runs with the new policy all failed,
so the cause was deterministic,
not load.
The case now anchors the first seat asked,
which keeps its intent (one heard reply anchors,
the rest do not) under any sheet wording.

### Runs

shi_Yumiaoya29 (frozen `811d908a9`,
pid 2915263) was killed under always-kill-and-relaunch.
shi_Yumiaoya30 launched at 06:29 UTC on `.frozen-dist-c432737d5` (pid 2972665),
to be read by every shi_Yumiaoya27 check,
the class one hundred twenty-six and twenty-seven renderings,
and the register check the handover's step 1 lists.
