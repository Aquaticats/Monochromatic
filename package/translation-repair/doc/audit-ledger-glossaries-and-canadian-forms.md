# Audit ledger: glossaries and Canadian forms

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

Every glossary entry and Canadian-forms pass measured against the corpus,
with the mutation check and the re-probe of the glossaries.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## Community and fandom glossaries

Probes:
`~/temp/agent/audit-community-glossary/`
(`corpus-floor`,
`overreach`,
`probe-floor`,
`probe-163`,
`probe-more`).
Files:
`src/community-glossary.ts`,
`src/community-glossary-fandom.ts`,
`src/translate-community-term.ts`.

### C1: refused forms match as raw substrings

Status:
fixed in `cc96eca77` (guard `glossary-match.unit.test.ts`),
"head mask" added to 头壳.
`renderingCarries` and `refusedFormIn` in `src/translate-community-term.ts` check no word boundary,
and a rendering excuses a refused form only when it opens inside the occurrence and runs past its end.
The built floor refuses "inside her head mask",
"head-mask",
"headgear",
"headdress",
"Atri waited by the laboratories",
"Atri was dictatorial"
and "Arona came from Badalona".
头壳's own why calls the object a "full head mask".
Fix:
match Latin forms at ASCII letter and digit boundaries,
excuse an occurrence any accepted rendering overlaps at any position,
add "head mask" to 头壳.
Guard:
小猫戴着头壳 passes "inside her head mask" and "inside her headgear" and refuses "inside her head";
亚托莉在猫舍 passes "Atri napped near the laboratories".

### C2: the 药娘 floor refuses a registered company name

Status:
fixed in `f3cd0ef83`,
guarded in `6fbcfa2d8` (owner,
2026-09-27:
"Allow it,
because it's the proper name of an org.").
mikaela_khara names a registered company 小药娘网络科技 (`page.md` lines 194 and 202);
the archive writes "XiaoYaoNiang(XYN)" and its translator's note gives the English name "XYN (Tianjin) Technology".
The floor refuses the archive and every mikaela run that rendered the name.
The class one hundred nineteen ruling refuses the term "even where the existing translation keeps it";
whether a registered name falls under it is the owner's call.

### C3: departures ignore inflection

Status:
fixed in `357f534b7` (guard `8da383b89`,
stems guarded in `36e7a4c30`).
Renderings now match bounded and inflected (`renderingSpans`:
s,
es,
d,
ed,
ing,
a final e dropped,
a final y turned),
so "heal",
"soothe" and "cure" carry every form and "atrium" is not Atri;
multi-word renderings that inflect inside ("becoming the doll") are listed,
and 药娘 takes "transgender girl".
The new rendering end was not replayed over the archives as the refusals were:
it reaches only the COMMUNITY RENDERINGS block,
evidence the judges weigh,
so a looser count ("masked" as 头壳's "mask") costs a departure line,
never a refusal.
`communityRenderingDepartures` names a departure for "Healing views",
"How hard this mental illness is to cure",
"becoming the doll",
"transgender girl"
and the MeowBot233 archive's own "soothing views".
The block text says "A rendering may inflect".
Fix:
add inflected renderings after each lead rendering (the kigurumi guard pins 治愈's lead "healed"),
never a bare "heal" (it sits inside "health"),
and match at boundaries.
Guard:
猫可以治愈人 with "a healing cat" and 猫是药娘 with "a transgender girl" name no departure.

### C4: renderings count as present inside unrelated words

Status:
fixed in `cc96eca77`.
"cured" inside "secured",
"outed" inside "shouted",
"atri" inside "psychiatric" hide real departures.
Guard:
猫被治愈 with "The cat was comforted and secured" names a departure.

### C5: whys promise refusals the refused forms do not carry

Status:
fixed in `357f534b7` (guard `8da383b89`).
Every form was replayed over the 364 archives and settled pages first:
the new refusals fire only on thirteen settled shi_Yumiaoya pages that shipped a "child" for 逆子,
one that shipped "HRT girl",
one that shipped "across different communities",
and none else.
头壳's bare "her head" stays with the judges and the why says so:
a slice that names the head mask can name the wearer's head too,
and on the ten settled TianqiChen666 pages every "her head" was "her headpiece" save one "inside her head",
which the floor refuses.
变娃 passes "doll up",
"dolls up",
"dolled-up";
头壳 passes "Her head was large and heavy" and "their head";
跨圈 passes "crossdresser community" and "across different communities";
药娘 passes "little HRT girl",
"medicine girl"
and tone-marked "Yào Niáng";
逆子 passes "rebellious child",
the form class one hundred fifty-one shipped twice.
Guard:
one cat fixture per gap.

### C6: false or page-specific whys and comments shown to the models

Status:
fixed in `357f534b7` (guard `8da383b89` flips the kigurumi test that pinned 安慰);
the root planning doc and handover,
which repeated the misquotation to the owner,
were corrected on 2026-09-28
in the commit that records this line (the page's only 安慰 is in a reader-comment JSON;
`page.md` line 89 writes 安抚).

- 炸柜's why pairs it with throwing away the medication,
    true of XIEPT2 only and false on mikaela_khara.
- 治愈's comment,
    why and the kigurumi test contrast it with 安慰,
    and the planning doc and handover repeated that contrast to the owner:
    TianqiChen666 writes 安抚 (`page.md` line 89),
    never 安慰.
    The owner's "Healed first" still stands on the fandom "healing" and the archive's "healed";
    the owner is told of the misquotation.
    "soothed" stays accepted,
    since MeowBot233's archive renders 治愈人的风景 as "soothing views".
- 治愈's why names one page's use and prints on shihai4h and MeowBot233.
- The header comment says "No candidate is barred by it",
    false since class one hundred nineteen,
    and "A term the archive got wrong is not entered",
    false for 炸柜,
    阿洛娜,
    亚托莉,
    变娃 and 治愈.
- The block heading "rendered as the archive and the community render them" is false where the archive departs.
- 变娃's "who then stays silent" conflicts with the doll's quoted speech.
- 柜门炸开's "never a literal door" beside the accepted "blown out of the closet".

### C7: sibling spellings missing

Status:
fixed in `357f534b7` (guard `8da383b89`);
爆柜 takes Anilovr's archive "outed",
跨性别圈 GLaDOSister's "trans community".
爆柜 (Anilovr) and 跨性别圈 (GLaDOSister) have no entries.

### C8: comments cut on the floor side only

Status:
fixed in `cc96eca77`.
The floor cuts HTML comments;
`communityRenderingDepartures` and the sheet term lines do not.

### C9: the two floor messages name the term differently

Status:
fixed in `cc96eca77` (the term is now `OD`,
so both messages name it alike).
One uses the trimmed term,
the other the raw one.

### C10: terms match inside other words

Status:
fixed in `357f534b7` (guard `8da383b89`) by an `enclosingWords` field beside `properNameContexts`;
"trans circle" alone stands,
inflecting to "trans circles".
自切 inside 各自切,
亲自切,
独自切.
"trans circle" is redundant beside "trans circles".

## Rendering glossaries

Probes:
`~/temp/agent/audit-rendering-glossary/`.
Files:
`src/rendering-glossary*.ts`.

### R1: " OD" misses lowercase and line-start OD, and SHIPPED

Status:
fixed in `cc96eca77`;
the three false comments corrected there and in this ledger.
hulicaijia opens a paragraph with `OD`;
XingZ60 writes lowercase `od` four times.
XingZ6014 shipped "I hate od,
/ so once z60 started od,"
and XingZ6010 "I hate od,
/ … started od'ing".
The comment claiming every OD stands after a space is false in `src/rendering-glossary.ts`,
`src/rendering-glossary-owner-forms.unit.test.ts`
and the planning doc.
" OD" also matches inside " ODE".
Fix:
case-insensitive Latin-token matching for terms that carry Latin letters,
on the source and on the Han-kept check.
Guard:
小猫讨厌 od and a line-start OD are refused when kept;
"an odd cat",
"recited an ode"
and 写 MOD pass.

### R2: 燃油车 refuses "fossil-fuel car"

Status:
fixed in `357f534b7` (guard `8da383b89`):
"fossil-fuel car" and "fossil-fuel vehicle" are renderings,
which excuse the refused form they overlap.

### R3: 未成年 "minor trans" fires inside transgression, translation, transfer

Status:
fixed in `cc96eca77` (boundaries,
and "minor transgender" added).

### R4: 滑档 refuses ordinary motion verbs

Status:
fixed in `357f534b7` (guard `8da383b89`):
every refused slide lands on a tier;
the replay found nineteen settled shi_Yumiaoya pages that shipped one of them and no ordinary motion verb refused.
"slipped into a low mood",
"tears slid down her face"
and "slid to the floor" are refused.
Fix:
key the refusals on the tier.

### R5: 化作 "turned into in " refuses grammatical English and is a sentence lesson

Status:
fixed in `357f534b7` (guard `8da383b89`);
`rendering-glossary-grammar.ts` and its test are gone.
"That butterfly is what the kitten turned into in spring" is refused.
Fix:
move the lesson to `GRAMMATICAL_ENGLISH_RULE` and correct the comment.

### R6: 摆烂's why prescribes one page's sentence

Status:
fixed in `357f534b7`:
the why covers taking it easy and no longer prescribes one sentence;
"slacked off" stays refused under the house rule that names it slang
(`src/house-policy.ts:79`:
"The English adds no slang of its own either (gearhead,
slacked off)").
On lxy "stopped trying" is a mistranslation of 偶尔摆烂.
"slack off" register is an owner question.

### R7: 矫正中心 and 矫正学校 missing

Status:
fixed in `357f534b7` (guard `8da383b89`);
zhangyubaka's archive "correctional school" is refused,
the prison reading 矫正机构's why already names.

### R8: 师范学院's why promises an official-name exception the floor refuses

Status:
fixed in `357f534b7`:
no promise in the why;
a named institution would take a `properNameContexts` entry,
and the pin carries none.
"normal school" is dated English,
not a calque.

### R9: three whys write "gaokao" while 高考's line refuses it

Status:
fixed in `357f534b7` (guard `8da383b89`);
the guard exempts a form the entry refuses itself,
which its why quotes to refuse.
Guard:
no why carries a form another entry refuses.

### R10: 亲友 "close friends" fires inside "close friendships"

Status:
fixed in `cc96eca77` by boundaries;
the re-probe of 2026-09-28 passes "close friendships".

### R11: 交往 lists "dated" on the one page where it never means dating

Status:
fixed in `357f534b7`;
the pin carries 交往 four times,
all on aiyysk,
never of romance.

### R12: 抢救's renderings carry "intensive care"

Status:
fixed in `357f534b7`;
"resuscitation" and "salvage" join the renderings where the passage means them.

### R13: sentence lessons and page-specific whys in word entries

Status:
fixed in `357f534b7`:
学霸,
未成年,
志愿填写 and 三剑客 carry word-level whys.
学霸 (属性),
未成年,
志愿填写,
三剑客.

### R14: refusal overreach without boundaries

Status:
fixed in `cc96eca77` by boundaries.
"wish form" in "a wish formed",
"threatened her life" in "lifelong",
"head of year" in "ahead of year-end",
"erben" in "verbena".

### R15: refusal gaps

Status:
fixed in `357f534b7` (guard `8da383b89`);
`JK裙` already passed under the class one hundred eighty-six matcher.
"type II diabetic",
"diabetes type II",
"threatens her life",
`JK裙`.

### R16: false or stale comments

Status:
fixed in `357f534b7` and the planning doc:
初中 stands nine times on five entries (a Xu_Yushu reader comment was counted as an archive passage),
one 激素 paragraph writes 药物 (the other 药 is inside 药娘),
交往 stands four times,
and the sheet line reads `- OD:`.
初中 counts,
激素 "two of them also write 药物",
the planning doc's 交往 count,
the double space before " OD" on the sheet.

## Canadian forms passes

Probes:
`~/temp/agent/audit-canadian/`.
Files:
`src/corpus-run/canadian-date.ts`,
`src/corpus-run/canadian-spelling.ts`,
`src/corpus-run/canadian-forms.ts`,
`src/corpus-run/prose-ranges.ts`,
`src/house-policy.ts`.

### K1: the date rewrite drops the comma after the year, and a test asserts it

Status:
fixed in `e9065faef` and `2efc90630` (guards `68d86f733`).
aiyysk1 and aiyysk2 shipped "February 9,
2024 was Lunar New Year’s Eve."
The Language Portal of Canada requires a comma after the year when the sentence continues.
`canadian-forms.unit.test.ts` expects "March 13,
2024 in a box".
Every full date the pass reads now takes the closing comma where a word follows its year
(`closingComma` in `src/corpus-run/canadian-date-parts.ts`),
including a month-first date the page already wrote ("December 21,
2023 the cat").
The replay over the 364 archive and settled pages adds it at both aiyysk pages and at the archive.

### K2: `closesRange` gets ranges wrong

Status:
fixed in `e9065faef` and `2efc90630` (guards `68d86f733`).
Splits a range across a line break,
skips "until 4 May" and a list item "- 4 May:",
and mixes orders when both ends carry a month.
`closesRange` is gone.
`readRange` (`src/corpus-run/canadian-date-read.ts`) reads a range or pair of days sharing one month
("1st to 3rd June",
"4th or 5th May") whole and writes it once,
month first,
keeping the join as written,
line break included;
a range whose ends both carry a month is two dates,
each rewritten.
A day after "until",
"till" or "to" is a date like any other,
and a no-break space,
an en dash,
an em dash,
a `>` and an emoji now open one.
A hyphen opens one only right after the date before it ("2 June-3 July"),
so "COVID-19 May" stays.

### K3: the closed word list leaves non-Canadian forms, including ones the house policy names

Status:
fixed in `1873b23dc` (guards `8ef3bdaec`).
Missing inflections of listed stems,
"judgement" and "summarise" shipped on XingZ6012 to XingZ6014,
"counselor",
"programmes",
"my mum".
The list is now built from -our and -ise stem families,
each crossed only with endings that keep the Canadian letter
(`src/corpus-run/canadian-spelling-stems.ts`:
"honourable" but "honorary",
"humourless" but "humorous"),
plus explicit pairs (`canadian-spelling-pairs.ts`).
Where Canadian sources split,
McGill's language guidelines,
the house policy's source,
decide:
"counsellor,
not counselor",
"enrolment,
not enrollment",
"program,
not programme";
"fulfill" and "skillful",
on which McGill is silent and the Ryerson guide writes with two l's,
stand.
"mum" becomes "mom" after a possessive and as a capitalised form of address.
A replay over the 364 archive and settled pages makes 52 distinct respellings,
every one correct
(counselor 39,
enrollment 25,
harbor 9,
splendor 7,
quarreled 6,
marginalised 4 and the -ise verbs among them),
and leaves only forms Canadian shares or the owner's sources leave open.
A test holds 25 words that must never change (humorous,
humoral,
honorary,
laborious,
coloration,
analyses,
paralyses,
meter,
check,
tire,
license,
practice,
fulfill,
skillful,
the root -ise words and others).

### K4: ordinal and year-first dates left alone

Status:
fixed in `e9065faef` and `2efc90630` (guards `68d86f733`).
`readMonthFirst` and `readYearFirst` (`src/corpus-run/canadian-date-read-leading.ts`)
drop a month-first ordinal's suffix ("December 29th"),
write a year-first date month first ("2023 Feb 25th",
"2023,
31 Mar."),
and spell an abbreviated month out,
with or without a day ("4 Sept 2024",
"Nov 2023").
A year-first date is read only where its day carries a suffix or ends the clause,
so "In 2021 May 4 was a Tuesday" stays;
every year-first date in the pinned archive carries a suffix.
A comma-joined year before a day-first date is joined only where the date ends its clause,
so "In 2020,
4 May was a holiday" becomes "In 2020,
May 4 was a holiday".
The replay on `2efc90630` rewrites 269 dates across the 364 pages and leaves no day-first,
month-first ordinal,
year-first,
unclosed-year or abbreviated-month date in prose.

### K5: withdrawn slices ship raw archive text

Status:
fixed in `3497e0041` (prep `ab4373cd2`,
guards `4014d49c4` and `66fe334ce`).
The page passes and the footnote guard now run again over the rows left once the guard takes a row back,
until a round takes nothing back (`corpus-run/page-assembly-rounds.ts`);
the withdrawal stays recorded,
and a row a pass wrote over the archive text wins over it.
No stored artifact carries an instance:
of 265,
only shi_Yumiaoya7 withdraws a slice,
an anchor with no archive text.
Guard output is byte-identical to the build before on six stored artifacts,
shi_Yumiaoya7 among them.
`restoredOnly` drops the Canadian row of a withdrawn slice and the archive text ships unconverted.

### K6: the house policy writes "capitalised" and "judgement"

Status:
fixed in `b45000747`;
`sheet-canadian-spelling.unit.test.ts` now reads every rendered sheet.
Its spelling test checks four forms.

### K7: `prose-ranges.ts` protects too much

Status:
fixed in `1873b23dc` and `dc325d847` (guards `8ef3bdaec`,
`18c2bc00f`).
A quote mark inside a JSX comment and a stray backtick protect the rest of the text.
A JSX expression now skips its comments,
a quote mark with no partner on its line inside an expression
is a stray apostrophe (a tag's attribute value may still run on across lines),
and a backtick run closes only at a run of the same length inside its paragraph,
else it is literal (CommonMark).
The Han-residue floor reads these ranges too,
so the change rides inside translate 15 and consolidation 20
and moved lane contest to 6 (M28).

### K8: small wrong rewrites

Status:
dates fixed in `e9065faef` and `2efc90630` (guards `68d86f733`);
accented neighbours fixed under H14.
"5 May beetles",
"the 4th May",
"4 May2024",
accented neighbours.
A day-first month followed by a listed compound ("May beetles",
"March hare")
or a capitalised word ("June Carter",
"April Fools’") is no date;
"I" and a capital after an abbreviation's period are exempt.
No day-first date in the 364 pages is followed by a capitalised word,
so the rule costs nothing there.
An ordinal day's article and "of" go with it ("the 4th of May" is "May 4");
a bare day keeps its article ("the 4 May deadline" is "the May 4 deadline").
A month or year running into digits or letters refuses the date.
A mutation check broke each of 15 guards in turn (the tag-bracket opener after `2efc90630`);
the Canadian form tests failed every time.

### K9: quoted lowercase English is respelled though the README says it is kept

Status:
fixed in `1873b23dc` (guard `8ef3bdaec`).
Every page rewrite now receives the slice's original (`page-slice-rewrite.ts`),
and a word the original writes in English in its prose keeps its spelling.
No pinned source carries a listed English word outside markup attributes,
so the corpus holds no instance;
the unit case and its mutant are the evidence.

### K10: underscore and slash edges

Status:
fixed in `1873b23dc` (guard `8ef3bdaec`).
An underscore or a dot stops a word only with a letter or digit on its far side (an identifier,
a file name);
a slash stops it only inside a token that reads as a path or address
(a leading `/`,
`~`,
`./` or `../`,
a dotted name,
a colon,
an equals sign or a backslash).
No corpus instance.

### K11: capitalized listed words at a sentence start are never respelled

Status:
fixed in `1873b23dc` and `70a2a73ca` (guard `8ef3bdaec`,
cases `031275d29`).
A capital opening a sentence,
a paragraph or a marked line,
or styling a title-case heading,
is respelled and keeps its capital
(`canadian-spelling-capital.ts`).
A capital mid-sentence,
after a title or an initial,
before another capital,
in a sentence-case heading,
in emphasis or in capitals throughout still names someone or a work;
"Gray" and "Id" stay capitalised as names wherever they stand.
A line that only continues its paragraph opens no sentence.
No sentence-start instance in the corpus;
see K14 for the heading instance.

### K12: test names claim more than they check

Status:
fixed in `403db3c6e`.
The audit's specifics were not recorded,
so the Canadian test names were read afresh against their checks.
Three overclaimed:
"LEAVES markup,
links,
code,
comments and emphasis untouched" (link text and emphasised lower-case words are rewritten),
"RESPELLS the closed word list in lower case" (capitals now respell too),
and "the house policy writes its own words in Canadian spelling" (it checks four -ise forms;
`sheet-canadian-spelling.unit.test.ts` checks every word).

### K13: the house policy's "-re" line omits "meter" the device

Status:
fixed in `403db3c6e`;
accounted inside translate 15,
consolidation 20,
repair 33 and refine 5.
The line now names metre for the unit and meter for the device that measures.

### K14: a title-case heading is never respelled

Status:
fixed in `1873b23dc` (guard `8ef3bdaec`).
Found by the spelling census of 2026-09-28:
the pinned archive's `Chinatsu_Suzuki/page.en.md:29`,
a section heading translating its original's line 29,
carries an American -or spelling in title case,
and the first pass took the capital for a name.
Title case makes capitals styling,
so a listed word there is respelled with its capital;
a sentence-case heading's mid-line capital still names someone.
The replay respells it.

### Spelling mutation check

26 guards broken one at a time,
with an unchanged-source control that must pass:
24 caught at first,
and the two survivors (the combining-accent branch,
the JSX-comment skip)
were caught once `18c2bc00f` gave them cases only they can pass.
A first run reported every mutant alive because its filter dropped every assertion line (M27).

## Glossary entry-content mutation check

23 fixes of `357f534b7` broken one at a time,
with an unchanged-source control that must pass:
all 23 caught by `glossary-entry-content.unit.test.ts`,
`community-glossary-kigurumi.unit.test.ts` or `rendering-glossary-calques.unit.test.ts`.
The inflected stems and the "d" ending had no case until `36e7a4c30`.
One mutant (the 安慰 contrast restored) first broke the build with an apostrophe inside a string literal,
so its catch came from the previous mutant's stale build;
rerun as a valid mutant beside the control,
it was caught.
A mutant whose build fails proves nothing,
so the harness now reports a failed build as no verdict.

## Glossary re-probe

Probe:
`~/temp/agent/audit-glossary-fix/x363-reprobe.mjs`
(output `x363-reprobe.out`,
glossary data,
ids and counts only)
and `x363-form-sites.mjs` (paths and line numbers only).
Run 2026-09-28 over every current entry (15 community and fandom,
41 rendering) against all 92 pinned originals,
with the real floor (`communityTermFindings`) over all 92 human archives
and the 214 settled pipeline pages of the entries carrying each term.
Since the probes of 2026-09-26,
five entries are new (爆柜,
跨性别圈,
矫正中心,
矫正学校,
螐),
化作 is gone and ` OD` became `OD`;
each new entry has one carrier,
the page it was written from.
The human archives fire on eight entries,
the set of 2026-09-26 but for four changes:
化作 removed,
mikaela_khara's 药娘 now an organization's name (`f3cd0ef83`),
zhangyubaka's "correctional school" refused on purpose (R-series),
and spike0qy's "jk skirt",
since the class one hundred eighty-six matcher reads the page's `JK裙` as the term
and the owner's forms refuse that rendering.
On the settled pages every fire is the term left untranslated
(OD written bare,
滑档 and 炸柜 in Han,
螐 on XingZ60's pages:
the H9 defect)
or a refused form.
Eleven refused forms fire there that the probes of 2026-09-26 never listed,
all on shi_Yumiaoya's pages;
one site of each was read against the original,
and each renders its own term wrongly:
the 药娘 forms render 小药娘 (the owner's "trans woman" or "trans girl"),
"across different communities" renders 在跨圈内,
the 逆子 forms drop the "son" of the father's quoted insult,
and the 滑档 tier slides are R4's own refusals.
No global refusal ignores context in the task's named cases:
交往 is R-series,
and 治愈 refuses no form,
while 治愈率 is on no pinned original (three carry 治愈).
