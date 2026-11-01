# OpenRouter and the corpus passes: 2026-09-25, 00:22 to 06:00 UTC

What was launched,
read and decided on 2026-09-25 00:22 to 06:00 UTC.
Entries:
shi_Yumiaoya27,
shi_Yumiaoya26,
shi_Yumiaoya25,
shi_Yumiaoya24,
shi_Yumiaoya23,
zheermao13,
yingying11,
mikaela17.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## shi_Yumiaoya27 read, 2026-09-25, 06:00 UTC: classes one hundred twenty-two to twenty-five live, five more renderings (class one hundred twenty-six)

shi_Yumiaoya27 ran on frozen `13b633bad`,
launched 05:40 UTC,
SETTLED at 05:57:50 UTC in 26.3 min (`ms=1577313`),
one attempt,
851 calls,
928 seats,
5 retry rounds,
about 0.41 USD by the meters (Bedrock 40.84 to 40.70,
OpenRouter 83.69 to 83.42).
Front matter equal,
4 headings,
no TA,
no error finish,
no double blank line,
19 abstention lines;
glm-5.3 asked 77 times and usable 0 (Hyper-only,
Hyper dry).

Every targeted sentence reads right on the correct build:
"In the ninth grade she suddenly became a top student" (class one hundred twenty-three),
"she attempted to end her life in the early hours of January 16."
with no means (class one hundred twenty-four),
"She missed her chosen schools and ended up at a second-tier teachers' college" (classes one hundred twenty-three and twenty-five),
"this was her way of telling everyone not to overdose or misuse drugs."
(class one hundred twenty-five).
"trans girl" on both 小药娘 lines,
"Magic Chewing Gum and Danpian" (the name entry `a2581911655` declares),
the closing attribution once.

CLASS ONE HUNDRED TWENTY-SIX,
under the owner's standing instruction of 2026-09-25.
A read of the whole page found five more renderings an English reader stumbles on:
年级组长 as "the grade leader",
未成年药娘 as "a minor trans girl" (which reads as "an unimportant trans girl"),
骨灰骰子 as "ash dice",
同居者 as "her cohabitants" (a romantic partner,
and plural for the one housemate the page names two paragraphs before),
and 精神霸凌 as plain "bullied" (the adjective dropped).
The rendering glossary now seeds 年级组长 ("head of year",
"grade director";
refused "grade leader"),
未成年 ("underage",
"a minor";
refused "minor trans",
"minor girl",
"minor boy"),
骨灰骰子 ("dice made from the ashes";
refused "ash dice",
"ashes dice"),
同居者 ("housemate",
"roommate";
refused "cohabitant",
"cohabiter")
and 精神霸凌 ("psychologically bullied",
"emotionally bullied";
refused "spiritual bull",
"spiritually bull").
No archive English page carries a refused form;
saurikissa's "cohabitation" is not refused,
since its original carries no 同居者.
Fixed in `0468bf5bc`,
guard red first `a2db36962` (`rendering-glossary-school-life.unit.test.ts`),
lint 0/0,
types clean,
full suite `suite-class126.log` 1127 PASS,
no FAIL.
The read left three renderings as written:
"ODing",
"jk skirt" and "Gaokao".
shi_Yumiaoya28 launched 06:03 UTC on `.frozen-dist-0468bf5bc` (pid 2861379).

OWNER ANSWER,
2026-09-25,
on those three:
"overdosing",
"sailor uniform",
"National College Entrance Examination".
CLASS ONE HUNDRED TWENTY-SEVEN builds it.
The rendering glossary seeds " OD" ("overdosing",
"overdose",
"overdosed"),
"jk 裙" ("sailor uniform",
"sailor-uniform skirt";
refused "jk skirt",
"jk-skirt",
"jk uniform",
"jk-style",
"jk dress")
and 高考 ("National College Entrance Examination",
"college entrance examination";
refused "gaokao").
The OD term carries its leading space:
every OD in the pinned corpus stands after one (aiyysk,
Susiethegamer,
interrgned,
hulicaijia,
shi_Yumiaoya),
and a bare OD would match the "MOD" s5ehfr9 writes.
A candidate keeping the term is refused by the floor's untranslated-term check,
whose finding now says "untranslated" instead of "in Han" and names the term trimmed.
No archive English page writes "gaokao" or a bare OD;
spike0qy's archive "JK skirts" is now a refused form there too,
by the owner's answer.
Guard red first `352062144` (`rendering-glossary-owner-forms.unit.test.ts`,
which also holds that MOD passes),
fixed in `811d908a9`,
lint 0/0,
types clean,
full suite `suite-class127b.log` 1128 PASS,
no FAIL.
The commit message of `811d908a9` names the guard wrongly as `0e0bd1a8c`;
a commit comment on GitHub corrects it.
shi_Yumiaoya28 was killed at 06:14 UTC eleven minutes in under always-kill-and-relaunch;
shi_Yumiaoya29 launched 06:17 UTC on `.frozen-dist-811d908a9` (pid 2915263,
waiter wait-s29).

## shi_Yumiaoya26 void, 2026-09-25, 05:30 UTC: the freeze nested the build, so the pass ran a stale one

shi_Yumiaoya26 SETTLED at 05:27 UTC in 24.7 min (894 calls,
about 0.42 USD by the meters),
but not on the class one hundred twenty-five build.
Its page carried "awakened her top-student trait" (class one hundred twenty-three),
"attempted to end her life ... by taking a large amount of medication" (class one hundred twenty-four)
and "slipped to a second-tier university" and "using this method to tell" (class one hundred twenty-five).
The frozen floor replayed "valid" on slice 6's shipped text,
and the frozen top level carried no "normal college" at all:
the freeze was first written to a misnamed `.frozen-dist-` that already held an older build,
`cp -r` nested the new build inside it as `node/`,
and the rename to `.frozen-dist-67da525a2` kept the stale top level the launcher runs.
The mistake was in the freeze,
not in the code:
the correct freeze of the same commit refuses slices 6 and 7 when replayed.
The handover's freeze paragraph now says to test that the directory is absent before the copy
and to grep the frozen top level for the newest fix afterwards.

The run still showed two things worth using.
Slice 10's "using this method to tell" is the refused calque with another noun,
and slice 7's "slipped to a second-tier university" does not say that she missed her chosen schools.
Both forms are now refused (`13b633bad`;
guards red first `e569be075` and `889811657`).
The consolidation polish on slice 4 had three review seats calling "awakened her top-student trait" a calque,
yet the refiner's proposal only reordered a clause and the polish gate kept the base.
The glossary floor,
which the stale build lacked,
is what closes that path.

shi_Yumiaoya27 was launched at 05:40 UTC on `.frozen-dist-13b633bad`
after a fresh copy and a grep for "using this method to".

## shi_Yumiaoya25 read, 2026-09-25, 03:30 UTC: class one hundred twenty-four live, two calques on the page (class one hundred twenty-five)

shi_Yumiaoya25 ran on frozen `2fe4def37` (class one hundred twenty-four),
launched 03:03 UTC,
SETTLED at 03:26:59 UTC in 23.5 min (`ms=1411317`),
one attempt,
905 calls,
978 seats,
1 retry round,
about 0.43 USD by the meters (Bedrock 41.12 to 40.98,
OpenRouter 84.32 to 84.03).
Front matter equal,
4 headings,
no TA,
no error finish,
no double blank line,
20 abstention lines;
glm-5.3 asked 73 times and usable 0 (Hyper-only,
Hyper dry).

Class one hundred twenty-four is live:
the January 16 sentence reads "she attempted suicide in the early hours of January 16.",
no means.
Classes one hundred twenty-two and twenty-three hold:
"Although she had taken medication before her passing",
"ended her short and tragic life" with no place,
"a teachers' college",
"suddenly blossomed into a top student".
"trans girl" on both 小药娘 lines,
"Chewing Gum and Danpian" this run,
the closing attribution once.

OWNER ANSWER,
2026-09-25,
on the question from the shi_Yumiaoya24 read:
"Leave it".
每日大量服用精神药物 ("took large amounts of psychiatric medication every day",
then the ICU)
stays as the original writes it;
the reader-protection rule reaches a death and an attempt only.
Read under the same answer (my reading,
not the owner's words):
the last weeks' "began abusing medication heavily" and "take medication and rest",
which the original does not call an attempt either.

CLASS ONE HUNDRED TWENTY-FIVE,
under the owner's standing instruction of 2026-09-25.
Two passages the archive never translated shipped as calques:
滑档二本 as "slid down into a second-tier admission slot"
(the English is that she missed her chosen schools and ended up at a second-tier university),
and 用这种方式告诉大家 as "she was only using this way to tell everyone"
("use this way to" is not English).
Each form appears once in the pinned corpus,
on this page.
Fixed in `67da525a2`:
`RENDERING_GLOSSARY` seeds 二本 ("second-tier university";
refused "admission slot",
"second batch",
"erben"),
滑档 ("missed her chosen schools",
"ended up at";
refused "slid down",
"slid into",
"slipped a file",
"sliding file",
"huadang")
and 用这种方式 ("this was her way of",
"in this way";
refused "using this way to",
"use this way to",
"used this way to").
Guard red first `052f6cffe` (`rendering-glossary-calques.unit.test.ts`),
lint 0/0,
types clean,
full suite `suite-class125.log` 1126 PASS lines,
no FAIL.
shi_Yumiaoya26 launched 05:02 UTC on `.frozen-dist-67da525a2` (pid 2731796,
waiter wait-s26).

## shi_Yumiaoya24 read, 2026-09-25, 03:05 UTC: classes one hundred twenty-two and twenty-three live, a survived attempt's method on the page (class one hundred twenty-four)

shi_Yumiaoya24 ran on frozen `7e20bbee2` (classes one hundred twenty-two and twenty-three),
launched 02:37 UTC,
SETTLED at 02:57:47 UTC in 24.5 min (`ms=1472366`),
one attempt,
885 calls,
about 0.44 USD by the meters (Bedrock 41.25 to 41.12,
OpenRouter 84.63 to 84.32).
Every shi_Yumiaoya23 check holds:
4 headings,
front matter equal,
no TA,
no error finish,
no double blank line,
the closing quote's two lines with the attribution once,
"trans girl" and "trans woman" on the two 小药娘 lines,
Han only inside the archive's comments;
16 abstention lines,
2 retry rounds.

Class one hundred twenty-two is live:
the death paragraph reads "Although she had taken medication before she died"
and "ended her brief and tragic life",
with no drug class,
no count and no rooftop.
Class one hundred twenty-three is live:
"teachers' college" and "she suddenly became a top student".

CLASS ONE HUNDRED TWENTY-FOUR.
The January 16 sentence,
two sentences before the ICU waking,
reads
"she swallowed a large amount of medication in the early hours of January 16 and attempted to end her life".
The method (吞下大量药物) is on the page for a survived attempt.
The winning ballots show why:
one wrote that the candidate "adheres to the 'Reader protection' rule ... appropriate way to handle the specific method",
another "(survived the attempt,
so no method named)".
The bullet opened "When a death was by suicide,
the specific method is deliberately kept vague",
so the judges read an attempt as outside it.
Fixed in `2fe4def37`:
the bullet opens "When a death or an attempt was by suicide,
the specific method is deliberately kept vague,
whether she died or survived",
and after the 自杀 sentence it states
"A survived attempt is no exception:
the page says that she attempted suicide or tried to end her life,
never by what means".
Guard red first `fe230891b` (`attempt-method-policy.unit.test.ts`:
the house policy and the select sheet carry both clauses,
the old opening gone),
lint 0/0,
types clean,
full suite `suite-class124.log` 1125 PASS lines,
no FAIL.
shi_Yumiaoya25 launched 03:03 UTC on `.frozen-dist-2fe4def37` (pid 2596683,
waiter wait-s25).

Not a class:
the ICU paragraph reads "Magic Chewing Gum and interrgned"
where shi_Yumiaoya9 to 23 wrote "Chewing Gum and Danpian".
Both are the entries' own renderings:
interrgned's original declares 单片 as name and alias,
its English page declares `name: interrgned` (the handle on every site in its `info.yml`) and `alias: Danpian`,
the same shape as a2581911655's `name: Magic Chewing Gum` and `alias: Chewing Gum`;
the page names both people by their entries' English names,
once each.
Line 237 ("she took large amounts of psychiatric medication every day",
before the second ICU stay)
renders 每日大量服用精神药物 as the original writes it:
daily overuse whose harm to her body led to the ICU,
which the original does not call an attempt.
It stands under the rule as written (a death or an attempt);
whether reader protection also reaches medication overuse the original does not call an attempt is left to the owner.

## Classes one hundred twenty-two and one hundred twenty-three, 2026-09-25, 02:40 UTC: the owner's reading of shi_Yumiaoya23

The owner read the shi_Yumiaoya23 note and wrote on 2026-09-25:
"these should obviously be translated better.
From now on whenever you see anything that can be translated better do it."
and
"dropping "swallowing a large amount of medication" is fine.
It's documented somewhere that not showing suicide methods therefore promoting replication is expected."
Both are standing instructions:
a rendering that reads badly is a class to fix,
not an observation to record,
and the reader-protection rule (`src/house-policy.ts`) is the authority on method detail.

CORRECTION of "shi_Yumiaoya23 read,
2026-09-25,
01:49 UTC":
it called "Although before her passing she took three bottles of some kind of sedative-hypnotic" closer to the original.
Under the reader-protection rule it is a violation:
the rule keeps "a method,
a substance,
a dose" vague,
and a drug class with a count is both.
Every shi_Yumiaoya run from 14 to 23 except 22 shipped the class and the count
(shi_Yumiaoya14 "three bottles of a certain sedative",
16 "three bottles of a certain sedative",
17 and 19 "a large amount of a ... sedative-hypnotic medication",
18 "three bottles of a certain sedative-hypnotic"),
and every run shipped "the rooftop of a tall building" for 走上了高楼的天台.
The artifact shows why:
the translate slate judges read the drug class as vague because it named no drug
("the drug name/dosage is deliberately kept vague"),
one read the rule as reaching only the medication that was the means
("violates the rule against softening/generalizing unless it's a suicide method"),
and the one candidate writing "a large amount of medication" was marked down as over-vague.
The rule never said that a class is a substance,
that it reaches medication beside the means,
or that a place can itself be the means.

CLASS ONE HUNDRED TWENTY-TWO:
the house policy now says that a class of drug is a substance and a count is a dose,
named or not;
that this holds for any medication the page ties to a death or an attempt,
not only the means,
the page saying at most that she had taken medication;
and that a place which is itself the means (a rooftop,
a high floor,
a bridge,
a railway line,
deep water) is a method,
the page keeping a place only where it tells a reader nothing to copy (a city,
a hotel,
a hospital).
The place clause is my reading of the rule's own replicability test against its "the date,
the place and the age" stay;
the owner's words ("not showing suicide methods") settle it the same way.

CLASS ONE HUNDRED TWENTY-THREE:
a rendering glossary (`src/rendering-glossary.ts`) beside the community glossary,
sharing its machinery:
the ordinary words an entry's source carries reach every sheet's identity context under a RENDERINGS heading,
and the source-carry floor refuses a candidate that keeps one in Han or writes a refused calque before any judge reads it.
Seeded from the corpus census (`rg` over every `page.md` at the pin):
师范学院 (shi_Yumiaoya) as "teachers' college",
refusing "normal college";
师范学校 (Xu_Yushu) as "teacher-training school",
refusing "normal school";
学霸 (shi_Yumiaoya,
mikaela_khara) as "top student",
refusing "top-student trait",
"academic tyrant",
"study tyrant" and "xueba".
The official names 北京师范大学 and 辽宁师范大学 keep "Normal University" and are not entered.
Each future rendering that can be better is a new entry.

Guards red first `87a50966f` (`protected-substance-policy.unit.test.ts`,
`rendering-glossary.unit.test.ts`),
built `7e20bbee2`,
lint 0/0 on the touched files,
types clean,
full suite green (`suite-class122-123.log`,
1124 PASS,
no FAIL).
`document-preparation.ts` went one line over its budget with two glossary calls,
so `glossaryTermLines` in `rendering-glossary.ts` carries both.
`hulicaijia18` (frozen `e57a2589c`) was killed at 02:36 UTC under always kill and relaunch,
and `shi_Yumiaoya24` launched on `.frozen-dist-7e20bbee2` to read both classes live.

## shi_Yumiaoya23 read, 2026-09-25, 01:49 UTC: SETTLED in 20.9 min; the sedative detail and two calques on the page (classes one hundred twenty-two and twenty-three, found by the owner)

`shi_Yumiaoya23` ran on frozen `e57a2589c` (class one hundred twenty-one),
launched 01:28 UTC,
and settled at 01:49 UTC in 1,252 s (`ms=1252566`) against shi_Yumiaoya22's 31.9 min,
one attempt,
917 seats,
841 calls,
5 retry rounds,
about 0.35 USD by the meters (Bedrock 41.94 to 41.82,
OpenRouter 85.85 to 85.62).
Synthetic wet,
Hyper dry;
glm-5.3 dark (72 seats,
Hyper-only by design).
Jev 29 of 29;
9 abstention lines;
no disputed slice;
longest gap 33 s.

Every shi_Yumiaoya22 check holds:
4 of 4 headings (Description,
Experience,
Departure,
Memorial),
front matter equal,
no TA,
no error finish,
no double blank line,
the death paragraphs on the page,
the closing quote as the archive's two lines with "From *The Truman Show*" once,
小药娘 as "trans girl" in the body and "trans women" in the `[^1]` definition,
with Han only inside the archive's own comments.

The death paragraph names a drug class and a count
(虽然在离世前她吃了三瓶某种镇静催眠药 as "Although before her passing she took three bottles of some kind of sedative-hypnotic"),
where shi_Yumiaoya22 wrote "She had taken medication before she died";
this note first called that closer to the original,
which the reader-protection rule contradicts
(corrected under "Classes one hundred twenty-two and one hundred twenty-three,
2026-09-25").
The ICU paragraph reads "Chewing Gum and Danpian" for 口香糖和单片,
the original's short form,
where shi_Yumiaoya22 wrote the declared long form (class seventy-eight holds either way).
Observation,
not a class:
the January 16 sentence drops 吞下大量药物 ("she attempted suicide in the early hours of January 16"),
the same rendering shi_Yumiaoya14 shipped;
class seventy-nine holds (an attempt,
not a death).
Other observations:
师范学院 as "a normal college",
觉醒了学霸属性 as "awakened her top-student trait",
three bench-written em-dashes left per the owner.
The Banner's title and text differ from the archive's in both runs;
the Banner is a JSX component in the body,
not front matter,
so the rule publishing the archive's front matter does not reach it.
:
SETTLED in 17.2 min,
the death paragraph whole;
no class

`CuspariaKLSY15` ran on frozen `e57a2589c` (class one hundred twenty-one),
launched 01:10 UTC,
and settled at 01:27 UTC in 1,030 s (TALLY `ms=1030329`) against CuspariaKLSY14's 1,299,
one attempt,
380 seats,
361 calls,
one retry round,
about 0.09 USD by the meters
(Bedrock 41.97 to 41.94,
OpenRouter 85.91 to 85.85).
Synthetic wet,
Hyper dry,
so glm-5.3 dark all run (19 seats refused at call time,
no wait).
Jev 15 of 15;
9 abstention lines.
The longest log gap is 51 s.

Every CuspariaKLSY14 check holds:
front matter equal,
1 heading,
no TA,
no error finish,
no double blank line,
the life in the past on every line.
The gloss line “Ling Shui Yu Yu Zi” means fish in clear water stands after the nickname line,
put back by the class one hundred five restore (`name-gloss-restored`,
one line) after the bench dropped it again.
Class one hundred seven live on slices 1 and 3 (`translate-archive-disputed`,
2 and 4 accepted claims).
The death paragraph carries every clause of the original with no medication detail:
双方情绪爆发 ("their emotions boiled over"),
抢救 ("where doctors tried to revive her"),
再次得到消息 ("When word came again"),
脑死亡 and 于 ("confirmed brain dead ... cremated on August 1st,
2023");
又 before 和其父亲发生争吵 is dropped again ("After waking up,
she argued with her father"),
the CuspariaKLSY14 observation,
not a class.
The life list carries `1.  `,
`3.  ` and `4.  ` with two spaces and `2. ` with one:
that is the archive's own spacing,
kept where the bench kept the archive's words;
it renders the same,
so no class.
The casing pass stayed silent by design (the archive's one capital word,
ACG,
has three letters).
No class.
`shi_Yumiaoya23` launched at 01:28 UTC on `.frozen-dist-e57a2589c` (pid 2246652),
waiter wait-s23.

## zheermao13 read, 2026-09-25, 01:08 UTC: SETTLED in 21.0 min; no class

zheermao13 (entry `zheermao101`) ran on frozen `e57a2589c` (class one hundred twenty-one),
launched 00:47 UTC,
and settled at 01:08 UTC in 1,259 s against zheermao12's 1,298,
one attempt,
877 seats,
829 calls,
0.11 USD Bedrock (42.09 to 41.98) and 0.19 OpenRouter (86.11 to 85.92) by the meters.
Synthetic was wet (weekly 86 percent at the end) and Hyper dry,
so glm-5.3,
a Hyper-only seat,
was dark all run (47 seats refused at call time by `NoProviderForModelError`,
no wait).
Jev 23 of 23;
13 abstention lines;
one retry round.

Every zheermao12 check holds:
front matter equal,
2 headings,
no TA,
no error finish,
no double blank line
(the one `>   > ` line with a trailing space is the archive's own).
Both labels one line each:
"English translation of Arts’s letter:"
and "English translation of the email conversation with One Among Us:"
(class seventy-seven holding).
The reply label reads "One Among Us members sent her a reply that very night:",
plural and 连夜 carried,
as zheermao12.
One `StreamOverrunError` in the log,
minimax-m3 on `produceTranslateSlate`,
1,035 characters against 1,024:
the translate lane's picture transcripts stay within the volume bound (class one hundred sixteen holding).
Class one hundred seven live on slice 5:
the repair lane's adjudicators accepted three minor accuracy claims against the archive ("always" added;
身边的人 narrowed to "friends"),
so the repair text stood in (`translate-archive-disputed (slice 5)`),
and the page reads "She leaned on the people around her and often needed their company."
The casing pass stayed silent:
the archive's one capital form is INFJ (twice,
in "INFJ-A/INFJ-T"),
and the page writes it in capitals.
No Han on the page.
No class.
`CuspariaKLSY15` launched at 01:10 UTC on `.frozen-dist-e57a2589c` (pid 2217593),
waiter wait-c15.

## yingying11 read, 2026-09-25, 00:46 UTC: SETTLED in 15.4 min; no class

yingying11 ran on frozen `e57a2589c` (class one hundred twenty-one),
launched 00:30 UTC,
and settled at 00:46 UTC in 923 s,
one attempt,
333 calls,
0.03 USD Bedrock (42.12 to 42.09) and 0.06 OpenRouter (86.18 to 86.12) by the meters.
Synthetic was wet (weekly 89 percent) and Hyper dry.

Every yingying10 check holds:
front matter equal,
1 heading,
2 markers and 2 definitions as the archive,
"Yingying" in the [^2] definition where the archive writes "Sakura" (classes eighty-six and one hundred fourteen),
"May you still have" as the original's 你 (class ninety-seven),
References kept,
"Geek Lab" with 硬核视频 rendered as "the hardcore video Bilibili account",
no Han,
no TA,
no error finish,
no double blank line.
The casing pass stayed silent,
as expected:
the archive's only all-capitals word is BUAA,
and it already reads that way in the contributor line.
Slice 0 shipped gate-kept (`gate-kept-standing`),
with 8 abstention lines.

Observation,
not a class:
one translate-slate abstention on the farewell says both candidates "violate the corpus house rule by using second person instead of third person".
No house rule says that.
The class ninety-seven floor refused the Kimi and minimax candidates that dropped the second person,
and the page reads "May you" as the original does.

## mikaela17 read, 2026-09-25, 00:22 UTC: SETTLED in 40.8 min; class one hundred twenty-one

mikaela17 ran on frozen `9ec838a01`
(launched 23:41 UTC 2026-09-24,
`TALLY mikaela_khara status=SETTLED slices=32`,
2,445,213 ms),
one attempt,
2,394 calls
(Bedrock 641,
OpenRouter 1,086,
Synthetic 667),
Bedrock 42.43 to 42.12 and OpenRouter 86.64 to 86.18 USD by the meters;
Hyper read dry.
The time against mikaela16's 20.6 min is Synthetic's,
wet again after the owner's reset and serving 667 of the calls:
the standing latency observation,
not a class.

Every mikaela16 check holds:
front matter byte-equal,
4 headings,
4 comments,
2 markers and 2 definitions as the archive,
𝓠𝓾𝓪𝓷 six times with no straight apostrophe after it,
"outed" once,
the HRT passage once ("It should have been an ordinary matter:
a family conflict,
her parents not wanting her to undergo hormone replacement therapy (HRT).")
where the archive has the "tried coming out" reading,
no TA,
no error finish,
the one double blank line the archive's own.
71 abstention lines,
one unendorsed standing (slice 2).
Observations,
not classes:
the Huai An parenthetical "(𝓠𝓾𝓪𝓷’s classmate in high school)" dropped again
(a Qwen3.8-27B ballot:
"does not add an unsupported relationship for Huai An";
mikaela6 recorded the same);
the excerpt line now renders the magazine's full title
"Tea Dwelling · Star-Lingering Pavilion · Star-Trace Chronicle,
2023,
Issue No. 33"
where the archive writes "Chronicle Issue No.33".

Class one hundred twenty-one:
the body shipped "Huai An and I decided to get off at Xiawafang Station[^2]"
beside the footnote's "XIAWAFANG Station is an interchange station",
where mikaela11 to 16 all wrote the archive's capitals.
The archive's translator comment gives the reason:
"下瓦房站：XIAWAFANG Station（备注这个是因为天津用大写拼音…）",
Tianjin writes its station names in capitals.
Mercury 2.5 chose the candidate "using standard capitalization for the station name ('Xiawafang') instead of all-caps",
and no sheet saw the body and the footnote together.
Built as a page-assembly pass after the list spread
(`restoreArchiveCasing`,
`corpus-run/archive-casing-restore.ts`):
a Latin word of four or more letters the archive writes in capitals at least twice in prose and never in another casing there
takes that form wherever a shipped slice's prose writes it title case;
headings are read past on both sides,
since the archive's own "## From Xiawafang to Tianjin Eye" writes the name title case by the heading's style;
a lower-case spelling,
a link destination and a word after `/`,
`.`,
`#`,
`=`,
`_` or `-` stand;
finding `archive-casing-restored`.
Guard red first `196a4fd6c` then `990f5f68f` (the heading case,
red against the first fix),
fixed in `d12aa2ed7` and `e57a2589c`,
lint 0/0,
types clean,
replayed over the mikaela17 artifact:
`archive-casing-restored (slice 16: "Xiawafang" to "XIAWAFANG")`,
no other row.
A census of every archive in the corpus finds the rule fires on eleven more capital forms across thirteen entries
(BUAA on four,
COOH,
LGBT,
USTC,
COHAE,
GAHT,
COVID,
ISBN,
HANLIANYI,
TODO,
INFJ),
all acronyms or names the archive writes in capitals only.
