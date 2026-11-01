# OpenRouter and the corpus passes: the page of zheermao22 on build 2a336603bf (2026-10-21)

What the shipped page of `zheermao22` (entry `zheermao101`) says against the Chinese,
line by line,
with the departures from it,
the recurrence of the findings of the previous reads,
and the comparison with the page of `zheermao21`.
The run's log,
meters,
seats,
rulings and traces to code are in
[the run of zheermao22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-run.md),
and the letter,
with the archive block review that wrote it,
is in
[the letter of zheermao22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-letter.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of the restart share are in
[the Gemma-lift restart overview](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart.md).

## zheermao22 page read, 2026-10-21, 23:16 UTC: the email and the quote recovered, the letter regressed; 30 departures

The page read is part of the read of `zheermao22` (SETTLED in 79.0 min of pass),
made at 23:16 UTC on 2026-10-21 on tip `2a336603bf38a6a53aeb4f3e70f0a3447acf5487`.

### The page

The reader read the shipped page line by line beside `page.md`,
the archive `page.en.md`,
the three photos (transcribed),
`info.yml` (`born: '2008-04-21'`,
`died: '2024-02-26'`) and the `zheermao21` page by `diff`
(`diff --unified=0` of the two pages).
Numbers in brackets are lines of the shipped page.
The Markdown was not rendered.

Her life is in the past tense throughout:
"Zheermao was a trans girl",
"Her MBTI personality type was",
"She also enjoyed",
"She played",
"She also loved",
"Zheermao was quite knowledgeable",
"she would also go",
"Zheermao was in fact a child",
"She relied on",
"Zheermao kept",
"Zheermao tried to leave",
"she made up her mind",
"Han Lianyi wrote".
The one present tense in the narration is "But now,
everything can only turn into a long sigh..." [144 to 145],
which renders 如今 and is correct.
Present tense inside the letter,
the email and the quote is the writers' own and is correct there.
Names:
"Zheermao" in the body and "Zheermao September" in the front matter (the archive's),
"Arts" and "ArtsEpiphany",
"Han Lianyi" for 寒涟漪,
"One Among Us" for 那些秋叶 in all three places it appears,
"Echo of One Among Us" for 秋叶的回声,
"Transland",
*Touhou Project*,
*Plague Inc.*,
*Overwatch*,
*Needy Streamer Overload*,
*Mystery Lover 2:
Forgotten Truth*.
Quotes:
the opening quote uses typographic quotation marks;
a `rg` for a straight double quote and for a letter-apostrophe-letter pattern finds none.
Canadian spelling holds where it shows:
"flavour" [14];
a `rg` for `color`,
`favorite`,
`flavor`,
`honor`,
`neighbor`,
`center`,
`theater`,
`labor` and `behavior` finds none.
The page has three `U+2014` characters (em-dashes) at [54],
[77] and [122] (`rg` for the character),
and no `U+2013`;
each sits at a comma of the source (see D12 in [the letter of zheermao22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-letter.md)).

Classes used for the departures:

- C:
  traced to code or policy.
- WD:
  model wording where a decision rule (a minimum vote weight,
  a tie,
  a lane contest split,
  a polish gate) decided which text shipped.
- WB:
  model wording where the whole-block archive review decided which text shipped (the letter and the email).
- W:
  model wording with no code cause.

Ids (D01 and on) run in page order through the departures.
A departure that repeats the archive's wording is marked "archive".

#### Which stage wrote what

The letter [52 to 85],
the email [108 to 126] and the two labels [50],
[106] are archive blocks with no Chinese counterpart
(`alignment target-unclaimed`);
they are decided by the archive block review before the lanes start,
and whatever wins there is the page.
The slices went through the repair lane,
the translate lane,
the lane contest and consolidation.
The slice boundaries differ from `zheermao21`'s:
slice 1 is 58 source characters here against 80 there,
and slice 2 is 67 against 45
(`SLICE-COST` lines);
the Touhou sentence moved from slice 1 to slice 2.
`chunk-document.ts` is not among the files `git diff --name-only 3637592bd4 2a336603bf` lists,
so the reason was not traced.

The lane contest settled 7 of the 11 contested slices on the translate lane (slices 0,
4,
6,
9,
11,
12,
14) and 4 on neither (1,
3,
5,
8).
Per slice,
from the log:

- Slice 0 (the quote):
  translate lane's text (`glm-5.3`'s candidate won weight 4 at 21:56:47 UTC after a first-round tie at weight 2);
  contest `4/6 usable, settled on translate` (22:01:13 UTC);
  consolidation slate won weight 2 by `minimax-m3` (4 ballots,
  22:09:13 UTC),
  consolidate gate `4/6 usable, settled on neither, ships standing` (22:09:24 UTC).
  The translate lane's text shipped.
- Slice 1:
  contest `settled on neither` (21:59:56 UTC);
  refinement from `hf:zai-org/GLM-5.3-Flash` won weight 3 (22:07:59 UTC);
  polish gate `5/6 usable, polished 5, base 0, settled on polished by rule polished-lead` (22:08:18 UTC).
  The polished text shipped.
- Slice 2:
  repair and translate lanes agree;
  no contest.
- Slice 3:
  contest `settled on neither` (21:58:05 UTC);
  `candidate 1 from composite(hf:moonshotai/Kimi-K3 + glm-5.3) won weight 3` (22:07:27 UTC);
  gate `4/6 usable, settled on consolidated` (22:11:06 UTC).
- Slice 4:
  contest `settled on translate` (22:00:44 UTC);
  slate `judges tied at weight 1.5` then the run-off,
  `candidate 1 from hf:Qwen/Qwen3.8-27B won weight 3` (22:08:13 UTC);
  gate `5/5 usable, settled on consolidated` (22:09:26 UTC);
  refinement from `hf:zai-org/GLM-5.3-Flash` won weight 5 (22:11:17 UTC);
  polish gate `polished 2, base 3, settled on base by rule base-lead` (22:12:34 UTC).
  The consolidated text shipped.
- Slice 5:
  contest `settled on neither` (21:59:36 UTC);
  `candidate 5 from deepseek-v4.1-flash won weight 2.5` (22:14:42 UTC);
  gate `5/6 usable, settled on consolidated` (22:15:33 UTC).
- Slice 6:
  contest `settled on translate` (22:01:45 UTC);
  slate `deepseek-v4.1-flash won weight 2.5` (22:17:53 UTC);
  gate `4/6 usable, settled on standing` (22:20:02 UTC).
  The translate lane's text shipped.
- Slice 8:
  contest `settled on neither`,
  archive endorsed (22:02:27 UTC);
  refinement from `hf:zai-org/GLM-5.3-Flash` won weight 4 (22:18:19 UTC);
  polish gate `5/6 usable, polished 3, base 2, settled on polished by rule polished-lead` (22:19:03 UTC).
  The polished text shipped.
- Slice 9:
  contest `settled on translate` (22:03:00 UTC);
  consolidation `every proposal was the incumbent` (22:14:20 UTC).
  The translate lane's text shipped.
- Slice 11:
  contest `settled on translate` (22:02:31 UTC);
  `composite(hf:Qwen/Qwen3.8-27B + inception/mercury-2.5) won weight 3.5` (22:16:02 UTC);
  gate `5/6 usable, settled on neither, ships standing` (22:16:41 UTC);
  refinement from `hf:zai-org/GLM-5.3-Flash` won weight 5 (22:17:47 UTC);
  polish gate `polished 1, base 4, settled on base by rule base-lead` (22:18:31 UTC).
  The translate lane's text shipped.
- Slice 12:
  contest `settled on translate` (22:03:30 UTC);
  slate `winner drew only weight 1.5 ... against a minimum of 2.00` (22:19:09 UTC),
  the run-off `judges tied at weight 2` (22:19:18 UTC);
  refinement `judges tied; keeping the repaired text` (22:21:40 UTC).
  The translate lane's text shipped.
- Slice 14:
  contest `settled on translate` (22:02:50 UTC);
  `candidate 2 from hf:zai-org/GLM-5.3-Flash won weight 2` (22:21:35 UTC);
  gate `3/5 usable, settled on neither, ships standing` (22:23:44 UTC);
  refinement from `minimax-m3` won weight 4 (22:24:18 UTC);
  polish gate `polished 5, base 0, settled on polished` (22:24:28 UTC).
  The polished text shipped.
- A script prints the shipped text per slice from `consolidation.slices[k].polish.text`;
  a second script checks that this text is in the page for all 11 consolidated slices
  (the artifact's `shipped.kind` reads `unchanged` for the slices where the polish gate shipped the polished text,
  so it is not the field to read).

#### Front matter [1 to 7]

D01 (C):
`desc: Treat you to some beverages.` is the archive's;
the Chinese `desc` is 如果有一天在天国遇到你，请你喝奶茶。
The English `desc` has lost the conditional and the milk tea,
and disagrees with the quote it comes from [9 to 11].
The cause is policy (`FRONT MATTER entry=zheermao101 authority=archive ... ships as it stands and no lane writes it`,
21:05:46 UTC),
the same as in the `zheermao21` read.
`name: Zheermao September`,
the alias and `location: Xi’an, Shaanxi` are the archive's and are right.

#### The quote [9 to 17]

Slice 0.
The text is the translate lane's (see "Which stage wrote what").

- 「谢谢你和我聊了这么多，如果有一天在天国遇到你，请你喝奶茶。」:
  "Thank you for talking with me so much.
  / If we meet in heaven one day,
  / I’ll treat you to some milk tea."
  Recovered:
  有一天 (one day) is back,
  the object 你 is carried by "we meet",
  and 奶茶 is milk tea (the archive had "beverages").
  The critics' milk-tea claims were accepted by the panel (four filed,
  slice 0 `accepted/minor`),
  and the translate lane carried the fix.
  D03 (W):
  "some" before "milk tea" is an addition (the archive had "some beverages"),
  a small one.
- 「如果能在这个世界上一起喝就好了呢。折耳猫喜欢什么口味的呀。」:
  "It would be lovely if we could drink it together in this world.
  / What flavour does Zheermao like?"
  The question keeps the name as its subject and the archive's invented "In addition" is gone.
  D03 (continued):
  就好了呢 is "would be lovely",
  a lift in register (the archive had "better",
  `zheermao20` "nice",
  `zheermao21` "better").
- 「我等不到了，对不起了……」:
  "I can’t wait that long.
  / I’m so sorry..."
  D02 (W,
  archive):
  "that long" is an addition;
  我等不到了 does not say how long.
  "so" is an intensifier 对不起了 does not carry.
  All three candidate texts (archive,
  repair,
  translate) carry both,
  and the critics filed no claim on either in this run.

#### Description [19 to 33]

- [21] 折耳猫是一个极度缺爱，需要关心的跨性别小女孩。:
  "Zheermao was a trans girl who was starved of love and needed care."
  Slice 1.
  The polished text shipped (5 to 0).
  D04 (W,
  archive):
  小 (little) is lost.
  `zheermao21` had it from the translate lane;
  here the archive,
  the repair text and the translate text all lack it,
  and the two claims filed on slice 1 (the tense claim and "extremely lacking in love") were both rejected.
  No critic filed the missing 小.
  极度 is carried only by "starved of",
  a fair intensifier.
- [23] "Her MBTI personality type was INFJ-A/INFJ-T."
  "type" is added by the polish;
  the Chinese says 人格.
  A small and natural addition.
- [25] 她也喜欢着东方 Project 的歌曲。:
  "She also enjoyed songs from *Touhou Project*."
  也 kept (the archive dropped it;
  the lanes agree on this slice).
- [27] The three games are right and in order.
- [29] 她也喜欢 Galgame，在推《恋爱绮谭·不存在的真相》的时候，她想到了许多自己的遭遇……:
  "She also loved galgames (dating sims);
  while playing *Mystery Lover 2:
  Forgotten Truth*,
  she thought about many of her own experiences..."
  Slice 3,
  the consolidated text.
  The full title is in (the archive had `*Mystery Lover 2*`);
  the repair lane accepted five claims (omission and `terminology/wrong-term`) on the title.
  D05 (C):
  "(dating sims)" is a gloss that is in neither the source nor `zheermao21`'s page.
  The house policy requires a generic term with no English equivalent to be glossed inline the first time it appears
  (`house-policy.ts:92`),
  so this is licensed and it is not counted as a defect.
  D06 (W,
  archive):
  喜欢 is "loved" (plain "liked"),
  and 遭遇 (what she met with,
  often hardship) is the mild "experiences".
  The semicolon joins what the source separates by a comma.
- [31] 折耳猫对数码产品有不少了解，在知乎平台的相关话题下，她也热情地给出了许多回答。:
  "Zheermao was quite knowledgeable about digital products,
  and on Zhihu she also enthusiastically gave many answers under related topics."
  Slice 4.
  热情地,
  也,
  许多 and 数码产品 are all carried (four claims on the slice were accepted and the consolidation carried them).
  D07 (WD):
  "gave many answers under related topics" is a calque of 给出了许多回答 and 相关话题下;
  `zheermao21` had "answered many questions on related topics".
  The polish gate had a more natural text (it dropped 也) and declined it,
  `polished 2, base 3` (22:12:34 UTC):
  `hf:moonshotai/Kimi-K3`'s ballot says the polished text "drops 也 ... a small but real content word".
- [33] 偶尔，她也会去到西安当地的图书馆看看自己喜欢的书。:
  "Occasionally,
  she would also go to the local library in Xi’an to browse the books she liked."
  Recovered:
  也 is back (it was dropped in the archive,
  `zheermao20` and `zheermao21`),
  carried by the translate lane.

#### Departure, first part [35 to 43]

- [37 to 39] 正如简介中所述的那样，折耳猫其实是个极度缺乏关爱的孩子，依赖着身边的人，常常需要大家的陪伴。:
  "As mentioned in the Description,
  / Zheermao was in fact a child deeply starved of love.
  / She relied on the people around her and often needed their company."
  Slice 5,
  the consolidated text.
  Right on 孩子 (child),
  其实 (in fact),
  依赖 and 身边的人 (the archive's "friends" is gone).
  D08 (W,
  archive):
  关爱 is "love" only (`zheermao21` had "love and care"),
  大家 is not rendered,
  and 陪伴 is "company" (`zheermao21`:
  "everyone’s companionship").
  The panel rejected the claims about 其实 and 大家 on slice 5 (`rejected/minor`).
- [41 to 43] [Arts] 也曾经给折耳猫发送过关心的信，她们关系很好，折耳猫一直留着她的信件。:
  "`[Arts](...)` also once sent Zheermao a caring letter.
  / They had a good relationship,
  / and Zheermao kept Arts’s letters all along."
  Slice 6,
  the translate lane's text.
  也 is carried (the panel accepted the omission claim,
  filed three times),
  and 一直 is carried by "all along".
  D09 (WD):
  "Arts’s letters" is plural,
  against the label [50] ("Arts’s letter") and `zheermao21`'s "letter".
  The lane contest chose the translate lane's text with 3 translate ballots to 2 neither (22:01:45 UTC);
  `gemma-4-26b-a4b-it` and `inception/mercury-2.5` argued for the plural as matching 信件,
  and `glm-5.3` and `deepseek-v4.1-flash` called singular and plural equal.
  信件 does not mark number,
  so the plural is defensible;
  it is a mismatch with the label [50] and with the singular "a caring letter" in [41].

#### Arts’s letter [50 to 85]

The label and the letter are read in
[the letter of zheermao22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-letter.md),
with departures D10 to D18
(the archive block review chose the `glm-5.3` revision);
the page here continues with the account of her death.


#### The account of her death [87 to 100]

- [87 to 90] 折耳猫曾好几次尝试离开这个世界，但在救助之中艰难地存活了下来。:
  "Zheermao tried to leave this world several times,
  / but each time she was rescued and,
  / with great difficulty,
  / survived."
  Slice 8,
  the polished text.
  D20 (W):
  "each time she was rescued" and "this world" at [95] are additions
  (but "this world" echoes the source’s sentence;
  "each time" turns 在救助之中 into a per-attempt claim).
  The ballot of `hf:Qwen/Qwen3.8-27B` names both additions and calls them harmless.
- [92 to 95] 然而，在 2024 年的早春时节，因着朋友的离世，她再一次下决心要离开。:
  "However,
  / in the early spring of 2024,
  / following the death of a friend,
  / she made up her mind once again to leave this world."
  D19 (WD):
  因着 (because of) is the temporal "following".
  The panel accepted the claim 4 to 1 (21:35:19 UTC,
  `accepted minor`),
  and the repair text carried "because of";
  the lane contest called the two texts equal (`5/5 usable, settled on neither`),
  and the polished text kept "following" (the archive and `zheermao21` had it too).
  This is the repair correction lost in a new form (see "Recurrence" in this file and K3 in [the traces to code](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-run.md#traces-to-code)).
- [97 to 100] 在离开之前几天，她给 One Among Us 的邮箱发送了一封邮件。那些秋叶的成员在看到后连夜给她发送了一份回信：:
  "A few days before her departure,
  / she sent an email to the One Among Us mailbox.
  / Upon seeing it,
  / One Among Us members stayed up that very night to send her a reply:"
  Slice 9,
  the translate lane’s text.
  Better than the archive:
  邮箱 is "mailbox",
  the members are plural,
  and the site name is in.
  D21 (W):
  连夜 is "stayed up that very night";
  "stayed up" is an inference (`zheermao21`:
  "that same night").
  `introduced repetition in slices 9, 11 (archive 0, shipped 2): "One Among Us members"` is warned
  (21:51:56 UTC and 21:57:43 UTC);
  the source says 那些秋叶的成员 in both sentences,
  so the repetition is faithful.

#### The email [106 to 126]

Against `photo3.webp` (请准备一下啦 / 是玖月折耳猫，我。。尽力活着了但是。。很抱歉，谢谢 / 如果成功死去了 /
请把我挂在上面，好嘛 / 谢谢你们):

- [106] "English translation of the One Among Us email conversation:"
  Apparatus,
  rewritten under the same house rule.
- [109 to 114] "Please prepare a little.
  / It’s Zheermao.
  / I tried my best to live,
  but...
  / I’m sorry.
  Thank you.
  / If I succeed,
  please put up my entry,
  okay?
  / Thank you all."
  Recovered:
  尽力 ("tried my best"),
  好嘛 ("okay?")
  and 谢谢你们 ("Thank you all") are back.
  D22 (WB,
  archive):
  很 of 很抱歉 is lost ("I’m sorry").
  D23 (WB,
  archive):
  如果成功死去了 is "If I succeed",
  which leaves out what she hoped to succeed at (the reader-protection rule reaches the method of a death;
  死去 names none,
  so the rule does not require the vagueness;
  as in the `zheermao21` read).
- [117 to 118] "Good evening,"
  and no salutation line.
  D24 (WB,
  archive):
  玖月折耳猫： (the salutation) is lost,
  and 夜安 is a night greeting where "Good evening" is a register choice.
- [119] 在刚给朋友写完邮件的时候刷新了收件箱，看到这封信有一点点难过，抱抱你。:
  "I had just finished writing an email to a friend and refreshed my inbox.
  Seeing this letter made me a little sad.
  I’d like to give you a hug."
  Right on 刚 and 有一点点难过 (the archive’s "I’m sorry" is fixed).
  D30 (WB,
  archive):
  抱抱你 is an offer ("I’d like to give you a hug"),
  and [120] adds "me" to 请你相信 ("believe me").
- [120] 但也请你相信，无尽的黑暗之中也有细小的光芒，可以避开黑暗的锋芒，到那些有光的地方躲一躲。:
  "But please believe me:
  in the endless darkness there are tiny glimmers of light as well.
  You can avoid the sharp edge of the darkness and take shelter in the places where there is light."
  Right,
  including the sentence the translators’ note says nobody could translate.
- [121] 「那些秋叶」的大家始终与你同在，如果需要帮助，还请联系我们，还有身边你愿信任的人。:
  "Everyone at One Among Us will always be with you.
  If you need help,
  please contact us,
  or someone close to you whom you trust."
  Right on 大家,
  始终,
  身边 and 愿信任
  (`zheermao21` had the stilted "All from One Among Us" and lost 身边).
- [122] 请不要担心，我们会尽力记下我们能记忆的故事。但活下去，就是对这个世界恶意的最大反抗。那些属于你我的动态的故事，一直能续写下去的故事，远远比静态的故事要来得鲜活，这是我们的笔永远无法企及的。:
  "Please don’t worry.
  We will do our best to record the stories we can remember.
  But to keep living is the greatest rebellion against the world’s malice.
  The dynamic stories that belong to you and me,
  the stories that can keep being written,
  are far more vivid than static stories,
  then a `U+2014`,
  then something our pens can never reach."
  Recovered in full:
  故事,
  尽力,
  最大反抗 / 恶意,
  属于你我,
  动态 / 静态,
  一直 ("keep being written") and 我们的笔 ("our pens") are all carried.
  The `U+2014` is at the source comma (D12).
- [123 to 126] 爱你，不要轻易放弃呀。 / 北方还很冷吧，再给你一个温暖的抱抱。春天来了，即使是坚冰也会慢慢解冻的。 / 顺颂春祉:
  "Love you.
  Don’t give up easily.
  / It must still be cold in the north.
  Here’s another warm hug for you.
  Spring is coming;
  even solid ice will slowly thaw.
  / With best wishes for spring".
  Right:
  温暖,
  坚冰 and 慢慢 are carried,
  and the closing formula is rendered (the archive had dropped it).
  The final full stop is missing,
  a trivial departure.
- [128] The note `{/* 可以避开黑暗的锋芒，到那些有光的地方躲一躲。 这句不知道如何翻译 */}`
  stands on a line of its own after the block.
  D25 (C):
  the archive had it inside the email;
  `withBlockNotes` puts a kept note after the replacement
  (`replaced with 1 note between translators kept`).
  The note says the sentence could not be translated,
  though it now is at [120];
  the owner's rule of 2026-10-20 keeps notes as written.
  The same as in `zheermao21`.

The email against `zheermao21` is better throughout:
the eight departures D12 to D19 of that read are repaired
(尽力,
好嘛,
谢谢你们,
the greeting’s "someone close to you",
故事 and 最大反抗,
属于你我,
动态 / 静态,
the warm hug),
except for 很 and the salutation.
It is also better than `zheermao20`'s and the best email of the four reads.

#### The close [130 to 148]

- [130 to 135] 后来，那些秋叶的成员在 Telegram 上联系上了她，试着安慰了她，与她聊了许久。包括跨性别相关的经历，爱与被爱，以及活下去的意义，试图多挽留她一会儿。:
  "Later,
  / One Among Us members contacted her on Telegram and tried to comfort her,
  / talking with her for a long time about transgender-related experiences,
  / love and being loved,
  / and the meaning of living,
  / in an attempt to keep her for a little longer."
  Slice 11,
  the translate lane’s text (the polish gate declined the polished text `polished 1, base 4`).
  Faithful in sense,
  and better than the archive ("that member").
  The `U+2014` pair of `zheermao21` is gone.
  D26 (W,
  archive):
  联系上了 is "contacted",
  which loses the success 上 carries (`zheermao21`:
  "reached").
- [137 to 138] 尽管如此，她仍旧坚持着自己的决定……最后仍然选择了离开。:
  "Despite this,
  / she still insisted on her decision... and ultimately chose to leave."
  Slice 12,
  the translate lane’s text.
  仍旧 is carried by "still".
  D27 (WD):
  the second 仍然 is not carried ("ultimately"),
  and "insisted on" is stiff for 坚持着 (`zheermao21`:
  "held to").
  A consolidated candidate with a plurality of 1.5 missed the minimum 2.00,
  the run-off over the two leaders tied at 2 and the standing text shipped.
- [140] "After her departure,
  Han Lianyi wrote an [epitaph](https://t.me/Lianyi520/565) for her."
  D28 (W,
  archive):
  离世 is death and the page says "departure",
  though [94] says "death" for the friend;
  悼文 is a memorial piece and "epitaph" is an inscription.
  Slice 13 (archive stands).
- [142] "Perhaps Zheermao’s departure was not an unexpected ending."
  Equal to the archive.
- [144 to 145] 但如今，一切也只能化作一声长叹……:
  "But now,
  / everything can only turn into a long sigh..."
  Slice 14,
  the polished text (5 to 0).
  The active "turn into" reads 化作 correctly.
  D29 (W,
  archive):
  也 (also) is not carried in 一切也只能.
- [147 to 148] "Contributor for this entry:
  / Echo of One Among Us".
  Equal to the archive.

### Recurrence of the findings of the previous reads

Each finding of the `zheermao21` read and [the causes document](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-causes.md),
checked against this entry:

- The 起的 polish gate tie (`CuspariaKLSY21` slice 1):
  not applicable to this entry;
  the word is not in the source (`rg` for 起的 in `page.md` finds nothing).
  The polish gate here had no tie (see [the rulings that fired](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-run.md#whether-the-new-rulings-fired)).
- The repair correction lost when the translate lane won (`CuspariaKLSY21` slice 0 and the class A1 narrowed to the
  polish gate):
  recurs in a new form on slice 8.
  The 因着 claim was accepted 4 to 1 and the repair text carried "because of";
  the contest called the texts equal and no candidate on the polish sheet carried the fix.
  The polish gate did see the claim (B731 fired:
  `hf:moonshotai/Kimi-K3`’s ballot at 22:19:03 UTC says "neither fixes the weak 'following' for 因着,
  so they tie on
  that accepted claim",
  and `hf:Qwen/Qwen3.8-27B`’s says "both candidates share the same 'following' for 因着").
  The class is traced in [the traces to code](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-run.md#traces-to-code).
  Slices 0,
  2,
  3,
  4,
  5,
  6,
  9,
  11 and 14 carried their accepted fixes into the shipped text
  (read against the accepted claims of the artifact).
- 已经,
  观望,
  二次元:
  not applicable;
  none is in `page.md` or the photo transcription (`rg` finds nothing).
  Finding 3 of the causes document called 已经 noise.
- 一直:
  present twice and carried both times:
  折耳猫一直留着她的信件 is "kept ... all along" [43],
  and 一直能续写下去 is "can keep being written" [122].
- The opening quote losing 有一天 and 你:
  recovered.
  The translate lane’s text keeps both,
  and the contest read `4/6 usable, settled on translate` (22:01:13 UTC).
  Finding 5 of the causes document (the minimum selection weight of two) did not bind this time.
- The thin email lines:
  recovered (see "The email").
- The two `U+2014` characters:
  three now,
  in different places,
  each at a source comma;
  none is a defect under Finding 7 (see D12 in [the letter of zheermao22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-letter.md)).
- The short probe (`zheermao21` slice 3,
  1 of 3):
  did not recur;
  all 14 probe rounds read `3/3 heard`.
  Ledger B733 (the reworded cache-refusal line) was not exercised,
  since the only shape it words is the short close.
- The meter-to-account gap:
  recurs on 13 of 57 lines,
  each equal to one call in flight (see [the NeuralWatt section](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-run.md#neuralwatt-baseline-stop-and-energy)).
  It is explained by Finding 9.
- The re-seat line printed 23 times:
  prints 0 times in its chunk form (2 slice lines,
  1 translate lane line),
  because Synthetic stayed wet until consolidation (see [the seats section](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-run.md#seats-units-and-meters)).
- The Qwen cut at the 120 s grace:
  recurs,
  10 times (6),
  and now also for `inception/mercury-2.5` (11) and once each for Kimi and `GLM-5.3-Flash`.
- Hyper dry the whole run:
  does not hold;
  Hyper was wet.
- Synthetic dry partway:
  holds (69.1 of 79.0 min).

### What changed between zheermao21 and zheermao22

Improved:

- The email,
  every line:
  [109 to 126] (see "The email").
- The quote:
  有一天 and 你 are back [10 to 11].
- 也 is back in the library sentence [33].
- "(dating sims)" is glossed on first use [29] (policy).
- The letter’s MtF sentence names the speaker [76]
  ("as a trans woman ... as us ... our future").
- The close:
  the `U+2014` pair of slice 11 is gone [130 to 135].
- The local photo read is back to about a second a photo (1.6 s,
  0.9 s and 0.7 s against 98 s,
  120 s and 47 s).
- Five photo readers instead of four (Gemma lifted).
- Slice 8’s first sentence reads naturally ("tried to leave this world several times,
  but each time she was rescued and,
  with great difficulty,
  survived") where the archive and `zheermao21` had the calque "survived with difficulty through rescues"
  (with the additions of D20).

Worse:

- The letter:
  "the online forum" [52],
  "QQ instant messaging number" [54],
  "guilty" [57,
  59],
  "cannot comprehend" and "good girl" [64],
  "it’s not easy to live" [66],
  "puzzled" [67],
  the three "For example" [76 to 77],
  "If you like,
  I’d like to hug you" [82] (D10 to D18).
- 小 is lost [21] (D04).
- "Arts’s letters" is plural [43] (D09).
- "gave many answers under related topics" [31] (D07).
- 大家 and "care" are lost [38 to 39] (D08).
- "contacted" for "reached" [131] (D26).
- "insisted on" for "held to" [138] (D27).
- "stayed up that very night" for "that same night" [100] (D21).
- Three `U+2014` characters [54,
  77,
  122] (not defects under Finding 7).

Stayed:

- The front matter `desc` [6],
  "that long" and "so sorry" [16 to 17].
- 如果成功死去了 as "If I succeed" [113],
  the missing salutation [117],
  "departure" and "epitaph" [140],
  and 也 in 一切也只能 [145].
- The note after the email and its stale text [128].
- 因着 as the temporal "following" [94] (D19):
  the archive and `zheermao21` had it too;
  what is new is that this run’s repair lane fixed it and the fix was lost (see K3 in [the traces to code](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-run.md#traces-to-code)).
- Present-tense handling and names.

By slice,
in short:
slices 0 and 3 and the email are better;
slice 4 is mixed (也 back,
the calque "gave many answers");
slice 8 is better in its first sentence and the same in its second;
slices 1,
5,
6,
9,
11 and 12 are slightly worse (D04,
D08,
D09,
D21,
D26,
D27) and the letter is worse;
2,
7,
10,
13 and 14 are the same.
Net:
the page is mixed.
The email and the opening quote are the best of the four reads,
and the letter is the weakest since `zheermao20`.
The letter is the longest block and `zheermao21` had recovered it,
so by length of text the page is not better than `zheermao21`’s,
while the email is the part a reader in danger would read most closely.
One sample each cannot separate a regression from run to run spread,
and this read did not measure that spread.

Slices that improved on the archive:
0 (milk tea,
the name as subject,
no "In addition"),
1 (the intensifier,
polished),
3 (the full title,
the gloss),
4 (digital products,
也,
热情地),
5 (people around her),
6 (也),
9 (members plural,
mailbox),
11 (members),
12 (仍旧),
14 (the active verb),
the letter’s first paragraph and harm paragraph,
and the email.
Slices that read worse than the archive anywhere:
the letter for D14 ("guilty") and nothing else the read found.

### What the page read did not do

- It did not read the declared-names sheet or any prompt
  (the run directory holds replies,
  not prompts).
- It did not run a spread measurement.
- It did not render the page.
- It did not trace why the boundary between slices 1 and 2 moved.
