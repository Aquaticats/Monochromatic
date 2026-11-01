# OpenRouter and the corpus passes: the page of CuspariaKLSY21 on build 3637592bd4 (2026-10-21)

What the shipped page of `CuspariaKLSY21` (entry `CuspariaKLSY`) says against the Chinese,
line by line,
with the departures from it and the comparison with the page of `CuspariaKLSY20`.
The run's log,
meters and seats are in
[the run of CuspariaKLSY21](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-cuspariaklsy21-run.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the three passes of the restart share are in
[the rotation restart overview](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart.md).

## CuspariaKLSY21 page read, 2026-10-21, 06:31 UTC: worse than CuspariaKLSY20's, most in slices 3 and 1; three departures traced to code by design

The page read is part of the read of `CuspariaKLSY21` (SETTLED in 11.8 min of pass),
made at 06:31 UTC on 2026-10-21 on tip `3637592bd43652fe2c6ff3cf2ec31e5a27c39c7d`.

### The page

The shipped page is `fixed/people/CuspariaKLSY/page.en.md` of the run directory.
The reader read it line by line beside the source `page.md` and the archive `page.en.md`,
with the artifact's `comparison`,
`laneSelection`,
`consolidation` and `pageAssembly` records and the issue list.
Numbers after "line" are lines of the shipped page,
the same numbers as in `CuspariaKLSY20`'s page.
The Markdown was not rendered.
The entry has one photo (`profile.jpg`) and no photo of text,
so no page line rests on a photo.

Structure and style:

- The front matter (lines 1 to 7) equals the archive's (a check of the text between the fences),
  as the log says:
  `FRONT MATTER entry=CuspariaKLSY authority=archive` (06:04:42.599 UTC).
  `desc: Be more similar to “herself”.` does not render the source's `desc: 「泠水鱼鱼子」是在喂鱼时起的哦。`;
  as in the `CuspariaKLSY18` and `CuspariaKLSY20` reads this is recorded and not counted.
- Past tense:
  a scan for `is`,
  `are`,
  `loves`,
  `likes`,
  `thinks`,
  `knows`,
  `tells`,
  `feels`,
  `cares`,
  `has`,
  `have`,
  `does`,
  `can`,
  `wants`,
  `means`,
  `says` and `cannot`
  finds two lines:
  line 19 `didn’t have` (past) and line 38 the present `means` of the name gloss.
  Her life is in the past tense throughout.
- No em-dash,
  no en-dash,
  no straight apostrophe or quotation mark (4 curly apostrophes and 6 curly double quotation marks),
  no footnote and no link.
- The trailing comment (line 52) is the archive's line with its full-width colon,
  and the log reads `ARCHIVE NOTE entry=CuspariaKLSY reading=advisory: Contributor for this entry：jiushigeshazi via email`;
  the credit name is intact.
- Canadian English:
  the page contains no word whose spelling differs between en_CA and en_US.
  "August 1,
  2023" is the date form `CuspariaKLSY20` used as well.

#### Slice 0, the first eight traits, lines 9 to 25

- Line 9:
  简介 as "## Description".
  Equal to the archive and `CuspariaKLSY20`.
- Line 11,
  她喜欢向日葵，她跟我说过她和向日葵有很深的缘分。:
  "She loved sunflowers.
  She once told me she felt a deep,
  fated bond with sunflowers."
  "loved" for 喜欢 is a strength the source lacks,
  as in `CuspariaKLSY20` (and line 15 says "liked" for the same verb).
  "felt" is added:
  the source reports what she told me about her and the flowers,
  not a feeling.
  `CuspariaKLSY20`'s "she and the sunflowers shared a deep,
  fated bond" was nearer.
  很深 and 缘分 are carried by "deep,
  fated bond";
  lowercase "sunflowers" is right (the archive had "Sunflowers").
  A small step back from `CuspariaKLSY20` on "felt".
- Line 13,
  她对艺术很执着。:
  "She was persistent when it came to her art."
  The archive's wording and the better reading of 执着 (persistence,
  with a note of fixation).
  The repair lane's issue 1 (that "persistent" is a dictionary gloss and stiff) was rejected 0 to 4.
  `CuspariaKLSY20`'s "deeply devoted to her art" shifted 执着 to dedication;
  that is gone.
  Lost:
  很 (very),
  which `CuspariaKLSY20` carried through "deeply".
  Better than `CuspariaKLSY20`.
- Line 15,
  她喜欢喝魔爪。:
  "She liked to drink Monster Energy."
  Equal.
- Line 17,
  她经常和我说吃不上饭。:
  "She often told me she couldn’t get a proper meal."
  吃不上饭 is to have no food to eat,
  hunger from want.
  "a proper meal" turns it into a matter of quality,
  the archive's softening ("a good meal") kept with another adjective.
  The repair lane's panel accepted this exact complaint twice at 4 to 0
  (issue 2:
  "softens 吃不上饭 ... into merely not eating well"),
  and its editor shipped "couldn’t get enough to eat",
  which two checkers marked fixed.
  The lane contest then chose the translate lane's text for the whole slice on tense (3 of 4 ballots),
  and that text had "could not have a good meal".
  The consolidation polish gate took "couldn’t get a proper meal" on 3 ballots to 1
  (`gemma-4-26b-a4b-it` voted base,
  calling it an "interpretive change").
  So an accepted accuracy fix was lost at the slice contest and the polish re-softened it.
  Level with `CuspariaKLSY20`'s "enough to eat" (a quantity reading);
  both are off the Chinese in different directions.
- Line 19,
  她觉得自己朋友很少。:
  "She thought she didn’t have many friends."
  觉得 is carried by "thought";
  很少 (very few) is flattened to "many",
  a lost intensifier.
  `CuspariaKLSY20` had "felt" and the same flattening.
- Line 21:
  "Her first language was Russian."
  Line 23:
  "She knew how to play guitar."
  Line 25:
  "Her parents fought a lot."
  Equal to `CuspariaKLSY20`.
  The repair lane's issue 4 ("fought" suggests physical conflict for 吵架) was rejected 0 to 4 and the word stands.

#### Slice 1, the character, belief and name, lines 27 to 40

- Line 27,
  她不怎么看重自己的生命。:
  "She didn’t value her own life very much."
  Carried.
  The archive's invented comparison ("more valuable than others’") is gone
  (issue 5,
  accepted,
  resolved).
  Equal to `CuspariaKLSY20`.
- Line 29,
  她觉得要是自己死能让其他人好，她会义无反顾。:
  "She felt that if her death could make things better for others,
  she would do it without hesitation."
  觉得 is "felt" (`CuspariaKLSY20` had "believed that",
  too firm);
  自己死 is "her death";
  义无反顾 is "without hesitation",
  the usual rendering of the idiom,
  where `CuspariaKLSY20`'s "without looking back" was the image.
  "things" is added to 让其他人好 (`CuspariaKLSY20` added "lives").
  Natural and about level in meaning;
  slightly better than `CuspariaKLSY20` on English.
- Line 31,
  她真的很珍重朋友。:
  "She truly cherished her friends."
  Equal (真的 by "truly",
  珍重 by "cherished").
- Line 33:
  "She was an Eastern Orthodox Christian."
  Line 35:
  "She didn’t believe in God all that much."
  Equal to `CuspariaKLSY20`,
  with 不是太 carried and no added "though".
- Line 37,
  她圈名「泠水鱼鱼子」是在喂鱼的时候起的。:
  "Her circle name,
  “Ling Shui Yu Yu Zi,”
  was given to her while she was feeding fish."
  The read calls this the worst regression of the page.
  起的 (was named,
  was coined) names no giver,
  and the context,
  a handle she chose,
  reads as her own act;
  "was given to her" supplies a giver the source does not have.
  `CuspariaKLSY20`'s "She came up with her nickname ... while feeding the fish."
  and the archive's "She coined her nickname" were faithful.
  The pipeline saw it and lost the vote:
  the lane contest ballot of `deepseek-v4.1-flash` called it out
  ("turns her own coining of the name into 'was given to her'"),
  the consolidation polish proposed "She came up with her circle name,
  ... while she was feeding fish",
  and the gate split 2 to 2
  (`deepseek-v4.1-flash` and `inception/mercury-2.5` for polished,
  `minimax-m3` and `gemma-4-26b-a4b-it` for base),
  which settled on `neither` and shipped the base.
  "circle name" is a literal 圈名 where the archive said "nickname";
  `deepseek-v4.1-flash` filed it as a calque in the naturalness review
  ("use a natural term such as 'handle,'
  'username,'
  or 'online name'").
  It is faithful and stiff.
- Line 38:
  "“Ling Shui Yu Yu Zi” means fish in clear water."
  has no counterpart in the Chinese.
  It is the archive's line,
  restored by code (see "Departures traced to code").
  It is the one present-tense verb in the body.
- Line 40,
  服用激素，成为跨性别的原因：:
  "The reasons she took hormones and became transgender:"
  Both 服用激素 and 成为跨性别 are carried
  (the archive's "She began transitioning because:"
  had lost 服用激素;
  issue 8,
  accepted,
  resolved).
  Equal to `CuspariaKLSY20`.

#### Slice 2, the list of reasons, lines 42 to 48

- Line 42,
  父母和她说过其实想要个女儿，感觉自己生错了性别。:
  "1.
  Her parents told her they actually wanted a daughter,
  and she felt she was born with the wrong sex."
  其实 is "actually",
  说过 is "told",
  感觉 is "felt".
  One sentence joined by "and",
  where the source has a comma.
  The naturalness review's finding that "born with the wrong sex" is not an idiomatic collocation is recorded and not acted on.
  Equal to `CuspariaKLSY20`.
- Line 44,
  一直在观望，接触二次元 ACG 后为自己立了设定，加上接触了 futa 萌生想法。:
  "2.
  She had been observing for a while,
  and after encountering ACG (Animation,
  Comics,
  and Games),
  she established a persona for herself,
  and her ideas were further sparked by her exposure to futa."
  Gains:
  立了设定 as "established a persona for herself";
  加上 as "further";
  萌生 as "sparked" (`CuspariaKLSY20` dropped 加上 and said "gave her the idea").
  Losses:
  一直 (all along) becomes "for a while";
  观望 (to watch and wait) is "observing",
  which drops the waiting
  (`deepseek-v4.1-flash` and `inception/mercury-2.5` filed this);
  二次元 is gone,
  which `CuspariaKLSY20` carried as "two-dimensional world";
  "her ideas" is plural for 想法,
  and "her exposure to futa" hands the thought to her ideas,
  not to her.
  The expansion "Animation,
  Comics,
  and Games" is added apparatus;
  the repair lane's issue 10 (terminology,
  accepted) required it by house rule.
  Three of four naturalness voices called the sentence unacceptable
  (`minimax-m3`,
  `deepseek-v4.1-flash`,
  `inception/mercury-2.5`:
  16 unique findings,
  among them "a broken coordination:
  three independent clauses strung together with 'and ... and'").
  Worse than `CuspariaKLSY20`,
  whose sentence was two clean sentences with 二次元 and 一直 carried and 加上 and 萌生 lost.
- Line 46,
  她笔下的角色越来越接近她自己，希望自己取代笔下角色。:
  "3.
  The characters she wrote became more and more similar to herself,
  and she hoped to replace the characters she created."
  希望 is "hoped" (exact;
  `CuspariaKLSY20` had "wanted").
  The ending is ambiguous:
  the source has her take the characters' place (自己取代),
  and "replace the characters she created" does not say who or what replaces them.
  The two nouns repeat,
  "wrote" then "created".
  `CuspariaKLSY20`'s "She wanted to take that character’s place."
  was clear.
  The plural "characters" is a guess at an unmarked number,
  as `CuspariaKLSY20`'s singular was.
  Worse than `CuspariaKLSY20`.
- Line 48,
  家庭问题，学校问题日益突出，希望逃离这个身体，但苦于没有能力。:
  "4.
  Family problems and school problems became increasingly prominent;
  she hoped to escape this body,
  but struggled because she lacked the ability to do so."
  Gains over `CuspariaKLSY20`:
  日益突出 is "became increasingly prominent" (`CuspariaKLSY20` "serious");
  苦于 is carried by "struggled because" (`CuspariaKLSY20` dropped it);
  能力 is "the ability" (`CuspariaKLSY20` "the means",
  which suggests resources);
  希望 is "hoped".
  Cost:
  "Family problems and school problems" repeats the noun (`deepseek-v4.1-flash` filed it)
  and "lacked the ability to do so" trails off
  (`minimax-m3`,
  `deepseek-v4.1-flash` and `inception/mercury-2.5` each filed it).
  Closer to the Chinese than `CuspariaKLSY20`,
  stiffer in English.
  Better on fidelity.
- The list markers are `1.` to `4.` with one space and blank lines between,
  as in the source.
  Net for slice 2:
  a fidelity gain in line 48,
  a loss in lines 44 and 46;
  slightly worse than `CuspariaKLSY20`.
  The consolidation could not polish it:
  `refine-skipped (0 eligible paragraphs)`,
  because the lane rewrites only `paragraph` blocks (`ELIGIBLE_KIND` in `refine-eligibility.ts`)
  and the slice is one ordered list;
  the review's findings were recorded and the gated text shipped.

#### Slice 3, the account of her death, line 50

Line 50,
在劝慰朋友时发生矛盾，双方情绪爆发，醒来后又和其父亲发生争吵，后因尝试自杀送往医院抢救，再次得到消息得知其已经确认脑死亡，于　2023 年 8 月 1 日火化。:
"While she was trying to console a friend,
a conflict broke out,
and emotions boiled over on both sides.
After waking up,
she argued with her father again.
She then attempted suicide and was taken to the hospital,
where doctors fought to save her.
Word came again that she had been confirmed brain dead,
and she was cremated on August 1,
2023."

- Carried:
  双方情绪爆发,
  又 ("again"),
  脑死亡 ("brain dead"),
  再次 ("again" on "Word came again"),
  the exact date,
  and no method for the attempt.
  The archive's "She swallowed lots of her meds that night" and "by August 1st" are gone
  (issues 12,
  14,
  16 and 19,
  all accepted and resolved);
  `house-policy.ts` line 87 says a survived attempt is told "never by what means".
  "a friend" follows 朋友 better than `CuspariaKLSY20`'s "her friend".
- Defects:
  "where doctors fought to save her" adds doctors and a fight to 抢救 (emergency treatment);
  the Chinese has neither.
  已经 (already) is lost,
  which `CuspariaKLSY20` carried ("she had already been confirmed brain dead"),
  and the cremation is folded into the same sentence as the report of death.
  得知 (learned) is not rendered,
  as in `CuspariaKLSY20`.
  "trying to console" adds "trying" to 劝慰,
  as in the archive and `CuspariaKLSY20`.
- Worse than `CuspariaKLSY20`:
  the page is back at `CuspariaKLSY19`'s "where doctors fought to save her"
  (a search of the shipped page of `CuspariaKLSY19` for `doctors` finds it on line 49).

### Comparison with CuspariaKLSY20's page

A line-by-line `diff` of the two shipped pages shows 13 changed lines of 52:
lines 11,
13,
17,
19,
29,
31,
37,
40,
42,
44,
46,
48 and 50.
The other lines are identical:
the front matter,
the heading,
and lines 15,
21,
23,
25,
27,
33,
35,
38 and 52 among them.

- Slice 0:
  level overall.
  Better:
  line 13 (the 执着 reading back to "persistent").
  Worse:
  line 11 ("felt" added,
  "shared" lost).
  Sideways:
  lines 17 and 19.
- Slice 1:
  worse.
  Better:
  line 29 (the verb and the idiom).
  Worse:
  line 37 ("was given to her" adds a giver;
  `CuspariaKLSY20` was faithful).
- Slice 2:
  slightly worse.
  Better:
  line 48 (苦于 and 能力 carried,
  日益突出 literal).
  Worse:
  line 44 (一直,
  二次元 and 观望 lost,
  the run-on) and line 46 (ambiguous ending).
- Slice 3:
  worse.
  Better:
  "a friend".
  Worse:
  "doctors fought to save her" and the lost 已经.
- Net:
  worse than `CuspariaKLSY20`,
  by about one clear meaning change in line 37,
  the invented doctors in line 50,
  and a rougher slice 2.
  It is still better than the archive in every slice where the archive was wrong
  (the method,
  the date,
  the comparison in line 27,
  the lost hormones in line 40).

### Departures, classified

The labels in this section are the read's.
Whether any departure is a class is being decided by a separate tracing;
where the read found no code cause,
the cause is not yet traced.

#### Departures traced to code by the read (3, all by design, none a class)

First,
the gloss sentence (line 38),
as in `CuspariaKLSY20`.
`pageAssembly.trimmed[0]` is slice 1 with a `replacementText` that ends the nickname paragraph with the gloss on its own line,
and `pageAssembly.findings` reads `name-gloss-restored (slice 1: "“Ling Shui Yu Yu Zi” means fish in clear water.")`;
the log has it at 06:16:25.909 UTC with `page assembly rewrote 1 slice and withdrew 0`.
The chain,
from the artifact and the log:

- The repair lane's panel did not settle the gloss:
  issue 7 (accuracy/addition) ended `needs-human` on a 2 to 2 vote
  (`minimax-m3` and `gemma-4-26b-a4b-it` supported;
  `deepseek-v4.1-flash` and `inception/mercury-2.5` did not).
  In `CuspariaKLSY20` the same issue was accepted 2 to 1.
  The log then says `translate-archive-disputed (slice 1)` at 06:08:41.510 UTC.
- The lane contest chose the translate lane's gloss-less text for slice 1 (3 ballots to 1).
- The consolidation `select` chose candidate 1,
  the gloss-less incumbent,
  with `minimax-m3`,
  `gemma-4-26b-a4b-it` and `inception/mercury-2.5` each calling the gloss an addition;
  `deepseek-v4.1-flash` voted for a candidate with the gloss.
- `restoreNameGlossLines` in `package/translation-repair/page-check/src/corpus-run/name-gloss-restore.ts`
  put the archive's line back at assembly.
  `page-apparatus-clause.ts` names this very line as page apparatus,
  so it is by design;
  the tense stays the archive's,
  as in `CuspariaKLSY18` to `CuspariaKLSY20`.
  The read records this and does not count it.
  The chain in this run is the same as in `CuspariaKLSY20`:
  the critics and the judges remove the line and the last step restores it.

Second,
the front matter `desc`,
traced to `FRONT MATTER ... authority=archive` and `doc/decision/translation-repair-front-matter-guard.md`;
not counted.

Third,
"where doctors fought to save her" (line 50).
The glossary lists this rendering for 抢救:
`rendering-glossary-wording.ts` holds `fought to save` and `tried to save` beside `emergency treatment`,
and its `why` says "English says emergency treatment or that doctors fought to save her".
The repair lane's issue 13 (omission of 抢救,
accepted 4 to 0) made the editor choose a glossary rendering,
and the judges in the repair lane's and the translate lane's selections cite the glossary
(`deepseek-v4.1-flash`:
"the approved warm rendering";
`inception/mercury-2.5` in one translate ballot objected that "doctors" is a specific detail).
Slice 3 had no consolidation,
because both lanes wrote the same text.
So it is by design,
and the Chinese still has no doctors;
`CuspariaKLSY20`'s "emergency treatment" was in the same list and was nearer.
The glossary at the previous tip was not read,
so the read cannot say whether this entry's choice changed with the build or only with the draw.

Also traced to code,
and correct (not a departure):
the archive's "She swallowed lots of her meds that night" and "by August 1st" are gone
(`house-policy.ts` line 87;
repair issues 12,
14,
16 and 19).

#### Model wording choices with no code cause found by the read (12)

The three that shift meaning most,
ranked by the read:

1. Line 37:
   "Her circle name,
   “Ling Shui Yu Yu Zi,”
   was given to her while she was feeding fish."
   adds a giver to 起的.
   The correction was proposed and lost on a 2 to 2 gate.
2. Line 17:
   "She often told me she couldn’t get a proper meal."
   keeps the softening of 吃不上饭 that the repair panel accepted 4 to 0 as an error,
   and the slice contest lost the repair lane's fix.
3. Line 44:
   "She had been observing for a while,
   and after encountering ACG (Animation,
   Comics,
   and Games),
   ... her ideas were further sparked ..."
   loses 一直,
   观望 and 二次元 and strings three clauses;
   three of four naturalness voices called it unacceptable.

The full list in page order,
each checked against the lane records:

1. Line 11:
   "loved" for 喜欢 and "felt" added.
2. Line 17:
   "a proper meal" for 吃不上饭.
3. Line 19:
   很少 flattened to "many".
4. Line 29:
   "things" added to 让其他人好.
5. Line 37:
   "was given to her" for 起的,
   and "circle name" for 圈名.
6. Line 42:
   one sentence joined for two clauses.
7. Line 44:
   the run-on with 一直,
   观望 and 二次元 lost.
8. Line 46:
   the ambiguous ending and the plural.
9. Line 48:
   "Family problems and school problems" and "lacked the ability".
10. Line 50:
    已经 not rendered.
11. Line 50:
    得知 not rendered.
12. Line 50:
    "trying to console" for 劝慰.

The read found no code cause for any of these.
Items 3,
11 and 12 are carried from `CuspariaKLSY20` unchanged,
and each was re-checked against its page.
Cause not yet traced for all twelve.

Traced on 2026-10-21 (UTC),
original text kept ([the causes](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-causes.md)):
item 10 (已经) is noise,
because no stage held the word and both lanes and the archive lack it (finding 3).
The lost 一直,
观望 and 二次元 of slice 2 are the translate lane's wording picked whole by the lane contest (finding 4);
"was given to her" is the translate lane's wording (finding 1);
and the softened 吃不上饭 of slice 0 is class A1 at the polish gate,
fixed by ledger B731 (finding 2).
The other items stay untraced.

#### Behaviours of the run that are not departures on the page

- The repair lane's `repairResolved=14` of 20 counts:
  issue 0 (the capitalized "Sunflowers",
  accepted 4 to 0,
  `resolved` false) and issue 3
  (the present tense of slice 0,
  accepted 3 to 1,
  two checkers voted `not-fixed`) are the two accepted issues left unresolved;
  issues 1 and 4 were rejected and issues 6 and 7 are `needs-human`,
  which with those two make the six that are not resolved.
  The page has lowercase "sunflowers" and the past tense,
  both from the translate lane.
  `CuspariaKLSY20` marked its tense issues `resolved: true` while the text kept the present tense
  (the read's claim about that run);
  this run marks the tense issue `not-fixed`,
  which the read calls the honest reading.
- Issue 6 (the tense switch in slice 1) ended `needs-human` at 2 to 2:
  `gemma-4-26b-a4b-it` held that present tense is right for standing facts,
  and `minimax-m3` said the quoted lines did not show the switch.
  The lane contest's tense scan (`tenseCandidates`) found present-tense lines in the repair lane's text for slices 0 and 1,
  and the judges chose the translate lane.
- Slice 2:
  `minimax-m3` returned an invalid candidate (four paragraphs against the page's one list)
  and a later `unable` answer to its findings;
  the slate stood on the incumbent (`slate-endorsed-standing`).
- The consolidation `select` of slice 0 tied at weight 1 and again at weight 2 (`keeping the fallback`);
  the polish then shipped (3 ballots to 1).
- The absolute naturalness reviews each had 4 usable voices of 7 (`unreachable=3`):
  slice 0 acceptable (twice),
  slice 1 unacceptable on one voice
  (`deepseek-v4.1-flash`,
  the "circle name" calque),
  slice 2 unacceptable on three voices;
  every rejection reads `recorded as evidence; the gated text ships`.
  `hf:Qwen/Qwen3.8-27B`,
  `glm-5.3` and `google.gemma-4-e2b` are `unusable` in every review,
  so each review ran at its minimum.

### What the page read did not do

- It did not open the stored prompt payloads or the live sheets,
  so it cannot say what the critics and the judges saw of the glossary beyond what their reasons quote.
- It did not render the Markdown,
  and looked up no external source for the page (it has none).
- The `CuspariaKLSY19` comparison is the single sentence of slice 3,
  from its shipped page only.
- `translateChanged=4` and `pageChanged=4` are tallies;
  they were not recomputed beyond the four per-slice `comparison` records,
  which show `both-differ` on slices 0 to 2,
  `both-agree` on slice 3,
  and `replacement-shipped` on all four.
