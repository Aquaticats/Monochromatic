# OpenRouter and the corpus passes: the page of CuspariaKLSY22 on build 2a336603bf (2026-10-21)

What the shipped page of `CuspariaKLSY22` (entry `CuspariaKLSY`) says against the Chinese,
line by line,
with the departures from it,
the comparison with the page of `CuspariaKLSY21`,
what became of the page findings of the previous read,
and the departures classified.
The run's log,
meters,
seats,
rulings,
classes and open items are in
[the run of CuspariaKLSY22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-cuspariaklsy22-run.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of the restart share are in
[the Gemma-lift restart overview](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart.md).

## CuspariaKLSY22 page read, 2026-10-21, 23:41 UTC: better than CuspariaKLSY21 in slices 0 to 2, worse in slice 3

The page read is part of the read of `CuspariaKLSY22` (SETTLED in 39.3 min),
made at 23:41 UTC on 2026-10-21 on tip `2a336603bf38a6a53aeb4f3e70f0a3447acf5487`.

### The page

The reader read the shipped page,
`fixed/people/CuspariaKLSY/page.en.md` of the run directory,
line by line beside the source `page.md`,
the archive `page.en.md`,
the entry's `info.yml` (`born: 2005-10-21`,
`died: 2023-08-01`),
the artifact's `comparison`,
`laneSelection`,
`consolidation` and `lanes` records,
the issue list and the log.
Page line numbers differ from those of `CuspariaKLSY21` after line 37:
the nickname and its gloss are now one line,
so [40] to [52] of `CuspariaKLSY21` are [39] to [51] here.
The numbers in brackets are lines of this page.
Repair issue numbers are the zero-based positions in `lanes.repair.result.issues` of the artifact
(29 issues,
numbers 0 to 28),
the numbering the read of `CuspariaKLSY21` also used.
The Markdown was not rendered.
The photo `profile.jpg` is a drawn avatar of a girl with black hair holding a Monster Energy can,
with no text,
so no page line rests on a photo.

Structure and style
(a script that checks the page,
and a `diff` against the page of `CuspariaKLSY21`):

- Front matter [1 to 7] equals the archive's,
  as the log says
  (`FRONT MATTER entry=CuspariaKLSY authority=archive`,
  line 211,
  22:25:27.685 UTC).
  `desc: Be more similar to “herself”.` does not render the source's `desc: 「泠水鱼鱼子」是在喂鱼时起的哦。`;
  as in the reads of `CuspariaKLSY20` and `CuspariaKLSY21` it is recorded and not counted
  (`doc/decision/translation-repair-front-matter-guard.md`).
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
  `says` and `cannot` finds one line:
  [37],
  the present `means` of the name gloss.
  Her life is told in the past tense throughout.
- No em-dash and no en-dash (a count over the file gives 0 and 0 for U+2014 and U+2013);
  no straight apostrophe or straight double quotation mark;
  4 curly apostrophes (U+2019),
  6 curly double quotation marks (U+201C and U+201D),
  and one U+FF1A (the full-width colon of the credit comment).
- The trailing comment [51] is the archive's line with its full-width colon;
  the log reads `ARCHIVE NOTE entry=CuspariaKLSY reading=advisory: Contributor for this entry：jiushigeshazi via email`
  (line 200);
  the credit name is intact.
- Canadian English:
  the page has no word whose spelling differs between en_CA and en_US,
  so nothing breaks the house rule;
  "August 1,
  2023" is the date form of `CuspariaKLSY20` and `CuspariaKLSY21`.
- The list markers are `1.  `,
  `2. `,
  `3.  ` and `4.  ` (two spaces after three of the four,
  [41],
  [45],
  [47]),
  where the source has one space throughout.
  This is the archive's irregular spacing carried through
  (the archive has the same four markers,
  the repair lane's slate text has them,
  and nothing normalizes them);
  Markdown reads the same list,
  so the read records it as a cosmetic difference.
  The page of `CuspariaKLSY21` had one space throughout.
- Names:
  `Ling Shui Yu Yu Zi` in the front matter and twice on [37],
  the archive's form;
  no name is invented.
- Paragraph structure:
  seven paragraphs for the second slice as in the source;
  the gloss is no longer a separate line ([37]),
  which matches the source's single line better than the two lines of `CuspariaKLSY21`.

#### Which stage wrote what

The slice each line came from (log lines in brackets):

- Slice 0 [9 to 25]:
  translate lane (`glm-5.3` won weight 3 of 4 in the challenge round,
  line 1584),
  lane contest chose translate (4 of 6 usable,
  line 1738),
  then the consolidation slate chose a composite of `hf:moonshotai/Kimi-K3` and `glm-5.3`
  (weight 2.5 of 4 ballots,
  line 1889),
  the gate chose it (`4/5 usable, settled on consolidated`,
  line 1910),
  and the polish gate rejected the polish (line 2232).
- Slice 1 [27 to 39]:
  translate lane (`deepseek-v4.1-flash` candidate 5 at weight 2,
  against 1.5 for candidate 4,
  line 1570),
  lane contest translate (line 1712),
  consolidation slate candidate 4 from `hf:zai-org/GLM-5.3-Flash` (weight 2,
  line 2002),
  gate `neither` so the standing stood (line 2022),
  then the refinement from `hf:zai-org/GLM-5.3-Flash` (weight 4,
  line 2068) and the polish gate for it (4 to 0,
  line 2100).
- Slice 2 [41 to 47]:
  translate lane (`glm-5.3` candidate 1 after the challenge round,
  line 1602),
  lane contest translate (4 of 5 usable,
  line 1729),
  consolidation slate tied so the standing stood (line 2140),
  no polish round (the artifact's polish record has `roundCount: 0`).
- Slice 3 [49]:
  translate lane composite of `glm-5.3` and `deepseek-v4.1-flash` (weight 2 of 3 ballots,
  line 1644),
  lane contest translate (5 of 6 usable,
  line 1704),
  consolidation slate kept the incumbent (weight 2,
  line 2047),
  the refinement from `hf:zai-org/GLM-5.3-Flash` (line 2193) and a polish gate of 4 to 1 for polished (line 2217).
  The lanes are nearly one text:
  `comparison[3]` repairText and translateText differ by one line break only.

#### Slice 0, the first eight traits [9 to 25]

- [9] 简介 as "## Description".
  Equal to the archive and to `CuspariaKLSY21`.
- [11] 她喜欢向日葵，她跟我说过她和向日葵有很深的缘分。:
  "She loved sunflowers.
  She once told me she and sunflowers shared a deep,
  fated bond."
  "loved" for 喜欢 is a strength the source lacks,
  and [15] says "liked" for the same verb
  (recurs from `CuspariaKLSY20` and `CuspariaKLSY21`).
  The sentence is better than that of `CuspariaKLSY21`:
  "felt" (added there) is gone,
  and 很深 and 缘分 are carried by "deep,
  fated bond" with the joint subject of the Chinese.
  Lowercase "sunflowers" is right
  (the archive had "Sunflowers";
  repair issue 1,
  accepted 4 to 0,
  resolved).
  Slice:
  the repair lane's wording
  ("fated" answers repair issue 0 on 缘分,
  accepted 3 to 1,
  resolved;
  the translate lane had dropped it to "deep bond"),
  carried into the consolidation composite;
  the polish proposal "she had a deep,
  fateful bond with them" lost 3 to 0 (one further ballot was neither).
- [13] 她对艺术很执着。:
  "She was devoted to her art."
  执着 is persistence or tenacity with a note of fixation;
  "devoted" reads it as dedication,
  and 很 is lost.
  `CuspariaKLSY21` had "persistent when it came to her art" (better),
  `CuspariaKLSY20` "deeply devoted".
  Slice:
  the translate lane's wording.
  The repair lane's text for this slice still read "She is persistent when it comes to her art."
  in the present tense
  (repair issue 2,
  major tense,
  accepted 4 to 0,
  `resolved=false`,
  one checker `fixed` and one `not-fixed`),
  and the lane contest settled on the translate lane (4 of 6 usable,
  line 1738),
  so the better word left with the repair text.
  That the contest chose on tense is an inference from the text;
  the contest ballots for this slice were not read.
- [15] 她喜欢喝魔爪。:
  "She liked drinking Monster Energy."
  Equal in sense (魔爪 is Monster Energy).
- [17] 她经常和我说吃不上饭。:
  "She often told me she went hungry."
  吃不上饭 is to have no food to eat.
  "went hungry" is idiomatic and near;
  it moves a little toward the experience of hunger.
  It is closer than the "couldn’t get a proper meal" of `CuspariaKLSY21` (a quality reading)
  and the "enough to eat" of `CuspariaKLSY20` (a quantity reading).
  Slice:
  the repair lane's editor wrote "went hungry" for the accepted claim (issue 4,
  supported 3 to 0,
  resolved);
  the lane contest dropped it with the repair text (the translate lane had "couldn't get a proper meal");
  the consolidation slate restored it through the composite of `hf:moonshotai/Kimi-K3` and `glm-5.3`.
  So the accepted claim reached the page by way of the consolidation draw and not by the polish gate
  (see the accepted-claims section in
  [the run of CuspariaKLSY22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-cuspariaklsy22-run.md)).
- [19] 她觉得自己朋友很少。:
  "She felt she had few friends."
  觉得 is "felt" and 很少 is "few":
  the intensifier that `CuspariaKLSY20` and `CuspariaKLSY21` flattened to "many" is carried.
  Better than both.
- [21] "Her first language was Russian."
  [23] "She could play guitar."
  [25] "Her parents fought a lot."
  [21] and [25] equal those of `CuspariaKLSY21`.
  [23] says "could play" for 会 (knows how to);
  the ability reading is fair,
  and the "knew how to play guitar" of `CuspariaKLSY20` and `CuspariaKLSY21` was nearer.
  Minor.

#### Slice 1, the character, belief and name [27 to 39]

- [27] 她不怎么看重自己的生命。:
  "She didn’t value her own life much."
  Carried,
  and the archive's invented comparison ("more valuable than others’") is gone
  (repair issue 6,
  supported 4 to 0,
  resolved).
- [29] 她觉得要是自己死能让其他人好，她会义无反顾。:
  "She felt that if her death could make things better for others,
  she wouldn’t hesitate."
  觉得 is "felt",
  自己死 is "her death",
  义无反顾 is "wouldn’t hesitate".
  "things" is added to 让其他人好,
  as in `CuspariaKLSY21`.
  Natural and level in meaning.
  Slice:
  the polish from `hf:zai-org/GLM-5.3-Flash`
  (base text:
  "She wouldn’t have a second thought if she believed her death would make things better for others."),
  which the polish gate took 4 to 0 (line 2100);
  "things" is already in the base text.
- [31] 她真的很珍重朋友。:
  "She truly treasured her friends."
  真的 by "truly",
  珍重 by "treasured".
  Equal in sense.
- [33] 她是东正教徒。:
  "She was an Eastern Orthodox Christian."
  Equal.
- [35] 她不是太信上帝。:
  "She didn’t believe in God all that much,
  though."
  不是太 is carried by "all that much" and 信 is now "believe in"
  (the polish moved it from "trust",
  and four of five gate ballots took it,
  line 2100).
  The ",
  though" has no counterpart in the Chinese:
  it is the archive's own ("She doesn’t trust God 100%,
  though.").
  `CuspariaKLSY19` added it,
  `CuspariaKLSY20` and `CuspariaKLSY21` did not,
  and it is back.
  Both lanes carried it,
  and no critic claim,
  panel verdict or judge ballot named it
  (a search of the log for `though` finds no mention of it as an addition).
  Slice:
  the translate lane's wording,
  kept by the consolidation.
- [37] 她圈名「泠水鱼鱼子」是在喂鱼的时候起的。:
  "She coined her nickname “Ling Shui Yu Yu Zi” while feeding fish.
  “Ling Shui Yu Yu Zi” means fish in clear water."
  The first sentence is faithful:
  起的 names no giver and "coined" is the archive's own faithful verb;
  the "was given to her" of `CuspariaKLSY21` (its worst regression) is gone.
  圈名 is a handle used in a community;
  "nickname" is the archive's reading.
  The second sentence has no counterpart in the Chinese:
  it is the archive's gloss of the name,
  in the present tense (the only present-tense verb on the page),
  and it is now on the same line as the first sentence.
  This time models carried it and code did not restore it (see the traces).
  The absolute naturalness review of this slice returned `unacceptable` from one voice of five usable
  (`hf:moonshotai/Kimi-K3`;
  `minimax-m3`,
  `gemma-4-26b-a4b-it`,
  `glm-5.3` and `deepseek-v4.1-flash` said acceptable):
  its finding on this paragraph is that the full handle is repeated verbatim in two consecutive sentences.
  It is recorded as evidence and does not change the text,
  by design (`consolidate/src/consolidation-polish.ts:7` to `12`:
  "no correction loop and no re-ask under any verdict").
- [39] 服用激素，成为跨性别的原因：:
  "Her reasons for taking hormones and becoming transgender:"
  Both 服用激素 and 成为跨性别 are carried
  (the archive's "She began transitioning because:"
  lost 服用激素;
  repair issue 9,
  supported 3 to 1 and 4 to 0,
  resolved).
  "Her reasons" is a fair frame for 的原因.
  "becoming transgender" follows 成为跨性别 closely;
  the same Kimi-K3 review calls it a calque and asks for "transitioning" (recorded as evidence only).

#### Slice 2, the list of reasons [41 to 47]

- [41] 父母和她说过其实想要个女儿，感觉自己生错了性别。:
  List item 1:
  "Her parents had told her they actually wanted a daughter.
  She felt she had been born the wrong sex."
  其实 is "actually",
  说过 is "had told" (a pluperfect the Chinese does not mark),
  感觉 is "felt".
  The source's one sentence with a comma is two sentences,
  as in `CuspariaKLSY20`.
  "born the wrong sex" is better than the "born with the wrong sex" of `CuspariaKLSY21`.
  The repair lane's panel left the omission of 其实 at 2 to 2 (issue 10,
  needs-human);
  the translate lane carries it.
- [43] 一直在观望，接触二次元 ACG 后为自己立了设定，加上接触了 futa 萌生想法。:
  List item 2:
  "She had long been watching from the sidelines.
  After getting into the world of anime,
  comics and games (ACG),
  she set up a persona for herself,
  and then,
  coming across futa (characters with both male and female anatomy),
  the idea began to take root."
  Carried here,
  and lost by `CuspariaKLSY21`:
  一直 ("long"),
  观望 ("watching from the sidelines"),
  二次元 (folded into "the world of anime,
  comics and games"),
  立了设定 ("set up a persona for herself"),
  萌生想法 ("the idea began to take root").
  Departures:
  - "(characters with both male and female anatomy)" is an addition to a bare "futa";
    it is the inline gloss of a community term,
    which the house policy asks for on first appearance (`house-policy.ts:92`),
    so it is by design and not a model whim;
    the explanation itself is the model's.
  - "(ACG)" keeps the abbreviation in parentheses,
    and the policy says the abbreviation follows only if the page uses it again (`house-policy.ts:93`),
    and the page does not;
    a repair-lane select judge said so in the log (line 1097).
    The source does carry "ACG",
    and repair issue 12 (accepted 4 to 0,
    resolved) asked for it to be spelled out,
    so the spelling out is answered and the parenthetical is the part over the rule.
  - 加上 (in addition) became "and then":
    an additive link turned into a sequence.
  - "and then,
    coming across futa (...),
    the idea began to take root" dangles:
    two of the five usable voices of the slice's absolute naturalness review
    (`hf:moonshotai/Kimi-K3` and `deepseek-v4.1-flash`) said so and returned `unacceptable`,
    which is recorded as evidence while the gated text ships (`consolidation-polish.ts:243` to `246`).

  Net:
  much nearer the Chinese than the run-on of `CuspariaKLSY21`,
  with a gloss and two small shifts added.
- [45] 她笔下的角色越来越接近她自己，希望自己取代笔下角色。:
  List item 3:
  "The character she created grew more and more like her.
  She wanted to take the character’s place."
  Clear,
  and 自己取代 is carried;
  "created" for 笔下 (wrote) and "wanted" for 希望 (hoped) as in `CuspariaKLSY20`;
  the singular is a guess at an unmarked number.
  Better than the ambiguous "replace the characters she created" of `CuspariaKLSY21`.
- [47] 家庭问题，学校问题日益突出，希望逃离这个身体，但苦于没有能力。:
  List item 4:
  "Problems at home and at school kept mounting.
  She wanted to escape this body,
  but she had no way to do it."
  日益突出 is "kept mounting" (fair);
  希望 is "wanted".
  苦于 (suffering from) is dropped again,
  as in `CuspariaKLSY20` and the archive;
  `CuspariaKLSY21` carried it.
  能力 (ability) is "no way to do it",
  a lack of means or opportunity and not of ability.
  Slightly worse than `CuspariaKLSY21` on fidelity,
  better on English.
  The repair lane's issue 14 (the "replace her" ambiguity) is met.
- No polish round ran for the slice (`roundCount: 0`,
  `refinersHeard: []`;
  the reason is not in the log),
  and the consolidation slate tied so the standing stood.

#### Slice 3, the account of her death [49]

- [49] 在劝慰朋友时发生矛盾，双方情绪爆发，醒来后又和其父亲发生争吵，后因尝试自杀送往医院抢救，再次得到消息得知其已经确认脑死亡，于　2023 年 8 月 1 日火化。:
  "While she was trying to console her friend,
  a conflict broke out and emotions flared on both sides.
  When she came to,
  she argued with her father.
  She then attempted suicide and was rushed to the hospital,
  where doctors fought to save her.
  Word came again that she had been confirmed brain dead,
  and she was cremated on August 1,
  2023."
  Carried:
  双方情绪爆发,
  脑死亡 ("brain dead"),
  再次 ("again" on "Word came again"),
  the exact date,
  and no method for the attempt
  (the archive's "She swallowed lots of her meds that night" and "by August 1st" are gone:
  repair issues 18,
  22 and 28 for the medication at critical and major,
  and 21 and 23 for the date,
  all accepted 4 to 0 and resolved;
  `house-policy.ts:87` says a survived attempt is told "never by what means").
  Defects,
  each against the Chinese:
  - 醒来后又 ("after waking up ... again") is "When she came to";
    又 is lost (`CuspariaKLSY21`:
    "After waking up,
    she argued with her father again"),
    and "came to" says she regained consciousness,
    which 醒来 does not say.
  - "rushed" adds urgency to 送往 (sent).
  - "doctors fought to save her" adds doctors and a fight to 抢救 (emergency treatment);
    the Chinese has neither
    (recurs from `CuspariaKLSY19` and `CuspariaKLSY21`).
  - 已经 (already) is not rendered
    (as in `CuspariaKLSY21`;
    `CuspariaKLSY20` carried it),
    and 得知 (learned) is not rendered
    (as in `CuspariaKLSY20` and `CuspariaKLSY21`).
  - "her friend" gives 朋友 a possessive (`CuspariaKLSY21` had "a friend");
    "trying to console" adds "trying" to 劝慰
    (as in `CuspariaKLSY20` and `CuspariaKLSY21`).

  Worse than `CuspariaKLSY21` (the 又,
  "came to" and "rushed" are new)
  and than `CuspariaKLSY20` (which had "emergency treatment" and "already").

### Comparison with the page of CuspariaKLSY21

A `diff` of the two shipped pages shows 17 changed hunks.
Unchanged:
the front matter,
[9],
[21],
[25],
[33] and the trailing comment.

- Slice 0:
  better.
  Better:
  [11] ("felt" gone),
  [17] ("went hungry"),
  [19] ("few").
  Worse:
  [13] ("devoted" for "persistent",
  很 lost),
  [23] ("could play").
- Slice 1:
  better.
  Better:
  [37] (起的 is faithful again,
  the single largest fix on the page).
  Level:
  [35] "believe in" (`CuspariaKLSY21` had it too).
  Worse:
  [35] (",
  though" added).
- Slice 2:
  better on the largest items and worse on small ones.
  Better:
  [41] (two sentences),
  [43] (一直,
  观望,
  二次元,
  立了设定,
  萌生 carried;
  the run-on is gone),
  [45] (clear ending).
  Worse:
  [43] (the "futa" gloss,
  the "(ACG)" parenthetical,
  "and then" for 加上,
  a dangling participle),
  [47] (苦于 lost,
  能力 turned into "no way").
- Slice 3:
  worse.
  Worse:
  又 lost,
  "came to",
  "rushed",
  "her friend".
  Unchanged:
  "doctors fought to save her",
  the lost 已经 and 得知,
  "trying to console".
- List markers:
  back to the archive's irregular spacing.
- Net:
  better than `CuspariaKLSY21` in three slices of four and worse in slice 3.
  Against `CuspariaKLSY20` it is level overall
  (better on "few friends",
  the same on 起的,
  a worse slice 3).
  It is better than the archive in every slice where the archive was wrong
  (the method,
  the date,
  the comparison in [27],
  the lost hormones in [39],
  the fabricated medication).

### What happened to the findings of the CuspariaKLSY21 read

From the read of `CuspariaKLSY21` and
[the causes](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-causes.md),
each finding that concerns the page of this entry.
The findings about meters,
seats and behaviours are in
[the run of CuspariaKLSY22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-cuspariaklsy22-run.md).

- The gloss sentence,
  restored by code
  (read:
  traced by design;
  causes document:
  the chain is the same as `CuspariaKLSY20`).
  Changed form:
  `pageAssembly.trimmed`,
  `withdrawn` and `findings` are all empty and the log has no `name-gloss-restored` or `page assembly` line,
  because no stage removed the gloss.
  The repair lane filed no issue against it
  (none of the 29 issue summaries concerns the gloss),
  both lanes' slice 1 texts carry it,
  and the translate slate's judges split on it
  (`mercury-2.5` and `typesafe/jev-1.13` took the gloss candidate;
  `minimax-m3` and `gemma-4-26b-a4b-it` took the one without it,
  saying the gloss is an addition,
  lines 1557 to 1569;
  candidate 5 won weight 2 against 1.5).
  So the sentence is on the page by a 2 to 1.5 vote and not by code.
- Front matter `desc`:
  unchanged,
  by design.
- "where doctors fought to save her":
  recurs.
  The rendering glossary lists it for 抢救
  (`rendering-glossary-wording.ts:99` to `117`:
  `emergency treatment`,
  `fought to save`,
  `tried to save`,
  with a `why` that says English says either),
  and the repair lane's editor wrote "where doctors fought to save her" for the accepted omission claims
  (issues 25 to 27).
  By design;
  the Chinese still has no doctors.
- Finding 1 of the causes document,
  "was given to her" for 起的:
  gone.
  The page says "coined",
  the archive's verb.
  The ruling "Polished text wins ties" it occasioned did not fire (no tie),
  so the page did not need it.
  The read of `CuspariaKLSY21` traced "was given to her" to the translate lane's wording;
  this draw's translate lane did not repeat it.
- Finding 2,
  吃不上饭 (class A1,
  polish gate,
  ledger B731):
  the page now says "went hungry",
  the repair lane's accepted fix,
  back by the consolidation composite and not by the gate;
  whether B731 acted is unverifiable here.
- Finding 3,
  已经:
  recurs (noise;
  neither lane's text has it).
- Finding 4,
  一直,
  观望 and 二次元:
  carried this time.
  The cause named there
  (the translate lane weakened the words and the lane contest picked that lane whole)
  still holds:
  the lane contest picked the translate lane on all four slices again.
  Here the translate lane's wording was the better one for slice 2.
- The twelve model wording choices of the read of `CuspariaKLSY21`,
  each against this page
  (the bracketed numbers in this list are that read's page lines;
  the lines of this page are one lower after [37]):
  1.  [11] "loved" and "felt":
      "loved" recurs;
      "felt" gone.
  2.  [17] "a proper meal":
      gone ("went hungry").
  3.  [19] 很少 as "many":
      gone ("few").
  4.  [29] "things" added:
      recurs.
  5.  [37] "was given to her" and "circle name":
      gone ("coined",
      "nickname").
  6.  [42] one sentence joined:
      changed (two sentences again).
  7.  [44] 一直,
      观望 and 二次元 lost,
      the run-on:
      gone;
      new gloss and "and then".
  8.  [46] the ambiguous ending:
      gone.
  9.  [48] "Family problems and school problems",
      "lacked the ability":
      changed (苦于 lost,
      "no way").
  10. [50] 已经 not rendered:
      recurs.
  11. [50] 得知 not rendered:
      recurs.
  12. [50] "trying to console":
      recurs.

  So 5 of 12 are gone,
  2 changed form and 5 recur.

### Departures, classified

#### Traced to code (5; by design or by the wire)

1.  The gloss sentence [37]:
    `house-policy.ts:92` interpolates `ARCHIVE_NAME_GLOSS_SCOPE` (defined at `house-rule/src/name-form-policy.ts:36`)
    and `page-apparatus-clause.ts` names this very line as apparatus
    (the classes of `CuspariaKLSY20` and `CuspariaKLSY21`).
    In this run the translate lane's vote carried it,
    and the restore function had nothing to do.
    Present tense,
    as in the archive;
    not counted.
2.  The front matter `desc`:
    the archive's front matter stands
    (`FRONT MATTER ... authority=archive`,
    line 211;
    `doc/decision/translation-repair-front-matter-guard.md`);
    not counted.
3.  The "futa" gloss and the spelled-out ACG in [43]:
    `house-policy.ts:92` (a community term is glossed inline on its first appearance) and :93
    (an abbreviation is spelled out,
    and kept in parentheses only if the page uses it again).
    The gloss is by design;
    the retained "(ACG)" is over the rule's second clause;
    the source does have "ACG".
4.  "where doctors fought to save her" [49]:
    `rendering-glossary-wording.ts:99` to `117`,
    by design.
5.  The "came to" of [49]:
    traced to a repair-lane region and its propagation,
    by a mechanism the code documents.
    The repair editor was asked to remove the fabricated sentence "She swallowed lots of her meds that night"
    (issues 18,
    22,
    28)
    and the select round for that region (lines 972 to 1000) offered "She fell asleep",
    "She was still upset" and "She came to";
    `minimax-m3` chose "She fell asleep",
    two voices declined every candidate,
    and the round ended
    `winner drew only weight 1 (ballots: 3, abstentions: 2) against a minimum of 2.00; keeping the fallback`
    (line 1000).
    The log does not say which of the three is the fallback;
    a repair-lane judge reason at line 1337 already quotes the current text as "She came to and argued with her ...",
    so it was in the text by then.
    The archive's slice 3 was then disputed (`translate-archive-disputed (slice 3)`,
    line 1410:
    "the repair lane's text stands in for it"),
    so the repair lane's text,
    with "When she came to",
    was the incumbent the translate lane worked from.
    The translate lane's slice 3 text and the repair lane's differ only by a line break,
    and the lane contest compared two nearly identical texts
    (`gemma-4-26b-a4b-it`,
    line 1700:
    "identical in their rendering").
    By design (classes one hundred seven and one hundred seventy-six,
    as the line says),
    but it means the two lanes of slice 3 were one reading,
    not two.
    Whether "came to" would recur depends on the editor's draw for the gap the deleted sentence leaves;
    the read calls it a one-off in content,
    with the stand-in as the mechanism that carried it.

Also traced to code,
and correct (not a departure):
the archive's medication sentence and "by August 1st" are gone
(`house-policy.ts:87`,
issues 18,
21,
22,
23 and 28).

#### Carried from the archive with no stage naming it (2)

- ",
  though" [35]:
  both lanes' incumbent text had it from the archive;
  no claim,
  verdict or ballot names it.
  The panel filed issues on other lines of the same slice (issues 6 to 9),
  so the slice was read,
  and none of them concerns this word.
- The list markers' two spaces [41],
  [45],
  [47]:
  the archive's text,
  kept by every writer;
  cosmetic.

#### Model wording choices with no code cause (11)

The three that shift meaning most,
ranked:

1.  [49] "When she came to,
    she argued with her father":
    醒来后又 loses 又 and says "came to"
    (see trace 5 under "Traced to code";
    the wording cause is the editor's filler).
2.  [43] the "futa" gloss and "and then":
    an addition (by rule) and an additive link made a sequence.
3.  [13] "devoted" for 执着:
    dedication for tenacity,
    with 很 lost;
    the better word sat in the present-tense repair text
    (the read's reading of why it was lost).

The full list,
in the order of the page:

1.  [11] "loved" for 喜欢.
2.  [13] "devoted" for 执着,
    很 lost.
3.  [23] "could play" for 会.
4.  [29] "things" added to 让其他人好.
5.  [35] ",
    though" added to 不是太信上帝.
6.  [41] "had told" for 说过;
    two sentences for one.
7.  [43] "and then" for 加上;
    "(ACG)" retained;
    the dangling participle.
8.  [47] 苦于 not rendered;
    "no way to do it" for 能力.
9.  [49] "rushed" for 送往;
    "her friend" for 朋友.
10. [49] 已经 and 得知 not rendered.
11. [49] "trying to console" for 劝慰.

The read found no code cause for any of these.
Items 1,
4,
10 and 11 recur from `CuspariaKLSY21`.

### What the page read did not do

- The Markdown was not rendered.
- The lane contest ballots and the translate-lane sheets for slice 0 were not read.
- No external source was looked up for the page (it has none).
