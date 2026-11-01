# Reading the Toka_ls page the pipeline published on 2026-09-02 (second rerun)

The second Toka_ls rerun,
on `aeb4181b9` at overlap 3,
14:13 to 15:52 UTC:
`TALLY Toka_ls
status=SETTLED slices=16 ms=5972259`,
100 minutes against 117 (and INCOMPLETE) at overlap 2 in the
morning.
Page and artifact (generation ten) under `~/temp/agent/toka-rerun2-20260902`;
`verify-published` clean (4,639 characters,
16 wordings,
0 silent,
0 missing);
0 refusals,
0 holds,
no
unendorsed standing,
no escaped quote,
no recovery nudge.
Seats:
Qwen3.8-27B threw 67 of 193 and Kimi-K3
57 of 192 at the window;
every other seat 5 or fewer.
The earlier cache reading
(`translation-repair-toka-ls-reading-2026-09-02.md`) named four things to look for;
this reads them
first,
then the rest of the diff.

## The four findings of the cache reading

- Slice 9's pronoun:
  FIXED.
  偶尔灵感迸发，左右推敲，留下工整的格律 now reads "Occasionally inspiration
  strikes,
  and after weighing each word carefully,
  she leaves behind verse in a neat meter."
  The archive
  had "I leave a neat rhythm" (wrong person);
  the morning's run had "they".
  The house rule and the
  `pronoun` line did their work:
  they/them/their on the page went from 1 to 2 (the friends,
  correctly),
  she/her from 32 to 33.
- Slice 10's title:
  HALF FIXED,
  AND THE HALF THAT MOVED IS A NEW DEFECT.
  《奇妙漂流》（或称「Aiden 的奇幻漂流」）
  became “Flow” (also known as “Life of Aiden”).
  "Life of Aiden" survives,
  but "Flow" is the English
  title of the 2024 film 喵的奇幻漂流,
  one of the five neighbour results the web lookup returned for
  奇妙漂流 (none of the five named the work).
  The judges read a neighbour as the official title.
  Fixed
  after the run:
  a result that never names the work now says so on its line and sorts last
  (`2a94c278a`,
  test shown to fail without it);
  the house rule already says a lookup is evidence to
  weigh.
- Slice 12's heading:
  "## The final chapter" became "## Final words" for 绝笔,
  which is right.
- Whitespace churn:
  no `> ` blank quote line on either page;
  the fold of `4cdc85f69` held,
  and the
  archive's three lines carrying only U+FEFF were replaced by real blank lines.

## The rest of the diff

- The opening poem (slices 1 to 3):
  tense settled to the present throughout ("brings",
  "lights",
  "leave",
  "caresses",
  "sees"),
  where the archive mixed past and present;
  "brighten and fade the day's
  scenery" for 明灭 recovers what "paints" lost;
  sentence-final periods added where the archive had none.
- "Toka Sakyo,
  also known as Tonghua and Nonamev" for 左橋瞳華（亦称「瞳华」、「Nonamev」）:
  the archive had
  dropped 瞳华;
  the page transliterates it.
  Front matter unchanged (`name: Toka Sakyo`,
  `alias:
  Nonamev`),
  the repair lane winning slice 0 with 6 ballots.
- "Her prose carries a light,
  delicate air,
  and her style is ornate and gorgeous ... she often draws
  on the classics and writes long letters,
  wishing the recipients a bright future":
  the archive had
  "wishing her a bright future",
  a wrong referent;
  fixed.
- "AI image generation" for AI 生图 (archive:
  "AI graphics"),
  as the cache reading predicted.
- The farewell letter (slice 13,
  the blockquote):
  re-rendered clause per line;
  "And from here on,
  everyone will be living through a time I cannot know" for 接下来大家要度过一段我不知道的时光 is right where
  the archive's "spend a time that I don't know" was not;
  "I am proud of myself,
  too,
  for being able to
  work with everyone for so long" recovers 也 and 这么久.
  Signature "> Nonamev / > Last words" for
  瞳华 / 绝笔 (the archive signed "Toka Sakyo").
  The generation-ten review saw the blockquote as one
  paragraph;
  the run's 13 absolute reviews were 11 unacceptable and 2 acceptable,
  all with 6 to 8
  usable ballots and no refused finding,
  and the gated text shipped with those verdicts recorded,
  as the
  no-loop design says.
- A paragraph the archive had omitted is now on the page:
  "On the afternoon of October 9,
  2024,
  Nonamev wrote this farewell note.
  At 4 p.m.,
  after resuscitation failed to reverse hemorrhagic shock,
  Nonamev sadly passed away in Shanghai at the age of 26."
  The source says exactly that
  (2024 年 10 月 9 日下午 ... 因失血性休克抢救无效，于上海不幸逝世，终年26岁).
  Whether "hemorrhagic
  shock" is the kind of detail the reader-protection rule keeps vague is the owner's call:
  the rule
  names the method of a suicide and medication;
  a medical cause of death that implies the method sits at
  its edge,
  and the archive had left the whole paragraph out.
  Flagged,
  not changed.
- Regression to note:
  "The meandering stream transforms into dynamic musical notes;"
  now ends in a
  semicolon (the source line ends the couplet);
  harmless.

## What the run measured

- Overlap 3 saved 17 minutes over overlap 2 on the same entry with no refusals;
  the hour rule is still
  missed by 40 minutes.
  The two seats throwing at the window (Qwen 67 of 193,
  Kimi 57 of 192) are the
  same two as on Carena (113 of 298,
  59 of 298) and keyword233 (12 of 37,
  8 of 37).
- The Exa lookup fired once (《奇妙漂流》,
  5 lines) and is cached;
  the neighbour warning applies on the
  next run without a new purchase.
