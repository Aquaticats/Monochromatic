# OpenRouter and the corpus passes: the page of zheermao21 on build 3637592bd4 (2026-10-21)

What the shipped page of `zheermao21` (entry `zheermao101`) says against the Chinese,
line by line,
with the departures from it and the comparison with the page of `zheermao20`.
The run's log,
meters,
seats and the archive block review are in
[the run of zheermao21](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-zheermao21-run.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the three passes of the restart share are in
[the rotation restart overview](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart.md).

## zheermao21 page read, 2026-10-21, 06:28 UTC: no class named by the read; 24 departures, two traced to code

The page read is part of the read of `zheermao21` (SETTLED in 40.7 min of pass),
made at 06:28 UTC on 2026-10-21 on tip `3637592bd43652fe2c6ff3cf2ec31e5a27c39c7d`.

### The page

The reader read the shipped page line by line beside `page.md`,
the archive `page.en.md`,
the three photos,
`info.yml`
(`born: '2008-04-21'`,
`died: '2024-02-26'`) and the `zheermao20` page by `diff`.
Numbers after "line" are lines of the shipped page.
The Markdown was not rendered,
and the lookup of the game's title was not repeated.

Her life is in the past tense throughout:
"Zheermao was a little trans girl",
"Her MBTI personality was",
"She also enjoyed",
"She played",
"She also loved",
"Zheermao knew quite a bit",
"she would visit",
"Zheermao was in fact a child",
"She relied on",
"Zheermao kept",
"she made up her mind".
The one present tense in the narration is "But now,
everything can only turn into a long sigh..." (line 142),
which renders 如今 and is correct.
Present tense inside the letter and the email is the writers' own and is correct there.
Canadian spelling holds where it shows:
"flavour" (line 14),
"favourite" (line 34);
the read found no American spelling.

The page has two `U+2014` characters (em-dashes),
at lines 130 and 132 (`rg` for the character),
where `zheermao20` had none.
They sit in the one sentence "They talked with her for a long time ... about experiences related to being transgender,
... and about the meaning of going on living ... trying to hold on to her a little longer."
Both lanes' texts for that slice already carried them (the slice-text extract for slice 11),
`gemma-4-26b-a4b-it`'s select ballot even called them stylised (05:56:01 UTC),
and the consolidation gate kept them.
The read found no coded rule against them:
`git grep` for `em dash`,
`em-dash`,
`2014` and the character in `house-rule/src` and `typography/src`
finds only quote-handling code,
so the `zheermao20` page having none looks like chance,
not a guard (inference from the absence of a rule).
This is a house-style break either way.

Classes the read used for the departures:

- `C`:
  traced to code or policy.
- `WD`:
  model wording where a decision rule (a minimum vote weight,
  a tie,
  a lane contest split) decided which text shipped.
- `WB`:
  model wording where the whole-block archive review decided which text shipped (the letter and the email).
- `W`:
  model wording with no code cause found by the read.

These are the read's labels.
The tracing of the findings to causes is a separate reading and its results are recorded when it finishes;
where this record says "cause not yet traced",
that tracing has not reported.
Ids (D01 and on) run in page order through the departures.

#### Which stage wrote what

The letter (lines 53 to 83),
the email (lines 101 to 124) and the two labels (lines 51 and 99)
are archive blocks with no Chinese counterpart
(`alignment target-unclaimed (pair 2: 5 translation blocks no original claims, 3373 characters: block/13, block/14, block/19, block/20, block/21)`).
`block/21`,
the fifth,
the read did not identify.
The other four are decided by the archive block review before the lanes start,
in one round,
and whatever wins there is the page.
The other slices went through the repair lane,
the translate lane,
the lane contest and consolidation.

#### Front matter, lines 1 to 7

D01 (`C`,
read's label):
`desc: Treat you to some beverages.` is the archive's;
the Chinese `desc` is 如果有一天在天国遇到你，请你喝奶茶。
The English `desc` has lost the conditional and the milk tea,
and disagrees with the quote it comes from (line 11).
The read's cause is policy
(`FRONT MATTER entry=zheermao101 authority=archive ... ships as it stands and no lane writes it`,
05:21:51 UTC),
the same as in the `zheermao20` read.
`name: Zheermao September`,
the alias and `location: Xi’an, Shaanxi` are the archive's and are right.

#### The quote, lines 9 to 17

The slice settled on the repair lane's text because the slate declined:
`lane contest: 4/7 usable, settled on neither` (05:58:15 UTC),
then a consolidation slate of 4 candidates on which the best-ranked text drew
`winner drew only weight 1.5 (ballots: 4, abstentions: 0) against a minimum of 2.00; keeping the fallback` (05:59:43 UTC),
a later tie at 1.5 (05:59:53 UTC),
and `slate-declined-standing` in the artifact.
The text that shipped is the repair lane's.
The critics' claim about the missing 有一天 (filed by `Qwen3.8-27B`) was rejected by all four panelists (0 to 4),
and the `就好了呢` claim (filed by `inception/mercury-2.5`) by 1 to 3.
The translate lane's text,
which kept both,
had the most weight on the slate (candidate 2:
1.5) and still shipped nothing.

- 「谢谢你和我聊了这么多，如果有一天在天国遇到你，请你喝奶茶。」:
  "Thank you for talking with me so much.
  / If we meet in heaven,
  / I’ll treat you to milk tea."
  Better than the archive on 奶茶 and 这么多 ("so much").
  D02 (`WD`):
  有一天 (one day) and 你 (you) are lost,
  as in the archive;
  `zheermao20` had "If one day I meet you in heaven".
- 「如果能在这个世界上一起喝就好了呢。折耳猫喜欢什么口味的呀。」:
  "It would be better if we could drink it together in this world.
  / What flavour does Zheermao like?"
  Better than the archive:
  the question keeps the name as its subject and the invented "In addition" is gone.
  D03 (`WD`):
  就好了呢 is "would be better",
  a comparison the Chinese does not make;
  the archive had it,
  `zheermao20` had "would be nice".
- 「我等不到了，对不起了……」:
  "I can’t wait that long.
  / I’m so sorry..."
  D04 (`WD`,
  kept from the archive):
  "that long" is an addition;
  我等不到了 does not say how long.
  D05 (`WD`):
  对不起了 is a plain apology;
  "so" is an intensifier the archive added and `zheermao20` removed.

#### Description, lines 19 to 34

- Line 21,
  折耳猫是一个极度缺爱，需要关心的跨性别小女孩。:
  "Zheermao was a little trans girl who was starved of love and needed care."
  Better than `zheermao20`:
  小 (little) is back.
  极度 is now carried only by "starved of",
  a fair intensifier,
  where the archive and the translate lane had "extremely lacking in love".
  The polish gate shipped this text
  (`consolidation polish gate ... settled on polished`,
  05:59:46 UTC).
- Line 23:
  "Her MBTI personality was INFJ-A/INFJ-T."
  Equal to the archive and to `zheermao20`.
- Line 25,
  她也喜欢着东方 Project 的歌曲。:
  "She also enjoyed songs from *Touhou Project*."
  也 kept (the archive dropped it).
- Line 27:
  the three games are right and in order.
- Lines 29 to 30,
  她也喜欢 Galgame，在推《恋爱绮谭·不存在的真相》的时候，她想到了许多自己的遭遇……:
  "She also loved galgames.
  / While playing *Mystery Lover 2:
  Forgotten Truth*,
  she thought about many of her own experiences..."
  The full title is in (the archive had `*Mystery Lover 2*`);
  the repair lane accepted three `terminology/wrong-term` claims for slice 3.
  D06 (`W`,
  from the archive):
  喜欢 is "loved" (plain "liked"),
  and 遭遇 (what she met with,
  often hardship) is the mild "experiences".
- Line 32,
  折耳猫对数码产品有不少了解，在知乎平台的相关话题下，她也热情地给出了许多回答。:
  "Zheermao knew quite a bit about digital products,
  and on Zhihu she also enthusiastically answered many questions on related topics."
  Better than `zheermao20`:
  热情地 is back on her manner of answering (`zheermao20` moved it to "enthusiastic answers"),
  也 is carried,
  and the archive's "electronics" and "shared her insights" are gone.
  "answered many questions" is a natural rendering of 给出了许多回答 and not an error.
- Line 34,
  偶尔，她也会去到西安当地的图书馆看看自己喜欢的书。:
  "Occasionally,
  she would visit the local library in Xi’an to read her favourite books."
  D07 (`W`):
  也 is dropped here,
  as in the archive and in `zheermao20`;
  看看 (have a look at) is "read".

#### Departure, first part, lines 38 to 44

- Lines 38 to 40,
  正如简介中所述的那样，折耳猫其实是个极度缺乏关爱的孩子，依赖着身边的人，常常需要大家的陪伴。:
  "As mentioned in her Description,
  / Zheermao was in fact a child starved of love and care.
  /
  She relied on the people around her and often needed everyone’s companionship."
  Right on 孩子 (child),
  其实 (in fact),
  依赖 (relied on),
  大家 (everyone) and 陪伴 (companionship,
  which `zheermao20` had as "company").
  Slice 5 shipped the consolidated text
  (`consolidate gate: ... settled on consolidated`,
  06:00:21 UTC).
  "her Description" for 简介 is the archive's small addition.
- Lines 42 to 44,
  [Arts] 也曾经给折耳猫发送过关心的信，她们关系很好，折耳猫一直留着她的信件。:
  "`[Arts](...)` once sent Zheermao a caring letter as well.
  / The two of them were close,
  / and Zheermao kept Arts’s letter all along."
  Better than `zheermao20`:
  "Arts’s letter" is singular,
  matching the label (line 51) and the sentence that introduces the letter
  (`zheermao20` had the plural "Arts’s letters",
  D08 in that read).
  The consolidation gate kept the standing text
  (`consolidate gate: ... settled on neither, ships standing`,
  06:00:23 UTC),
  and the polish gate then shipped the polished one
  (`settled on polished`,
  06:00:55 UTC),
  which is where "The two of them" comes from.

#### Arts's letter, lines 51 to 83

This block is where the page recovered.
Its review is in the section "The letter block" of the run file;
the shipped letter is the `deepseek-v4.1-flash` revision.
Line by line against `photo1.webp` and `photo2.webp`:

- Line 51,
  "English translation of Arts’s letter:"
  Apparatus rewritten from "the preceding letter" under the house rule against positional pointers
  (`house-policy.ts`,
  as in the `zheermao20` read);
  the page is better for it.
- Lines 53 to 54,
  我是Transland管理员ArtsEpiphany，你可以叫我Arts。:
  "I am ArtsEpiphany,
  an admin of Transland.
  / You can call me Arts."
  Better than `zheermao20` and the archive:
  "the online forum" (not in the photo) is gone.
- Line 55,
  我看到你在跨岛留下了自己的QQ号，但是我搜索之后发现你似乎设置了禁止通过QQ号查找，因而我无法加你好友，才选用这种方式和你联系，真的很抱歉！:
  "I saw the QQ number you left on Transland,
  but after searching I found that you seem to have disabled being found by your QQ number,
  so I couldn’t add you as a friend and chose to contact you in this way.
  I’m really sorry!"
  Right in every clause,
  including 我搜索之后发现,
  因而我无法加你好友 and 真的很抱歉.
  This repairs what `zheermao20` shipped wrong
  ("disabled initiating conversations using QQ number ... Therefore").
- Lines 57 to 59,
  其实，Arts也是一只猫猫。猫猫怎么会是罪恶的呢？猫猫是世界上最可爱的存在哦。:
  "Actually,
  Arts is also a cat-like person.
  / How can a cat-like person be unforgivable?
  / Cat-like people are the cutest beings in the world."
  D08 (`WB`,
  from the archive):
  猫猫 is "cat" (the writer says she is a cat),
  and "cat-like person" appears three times.
  D09 (`WB`):
  罪恶 is "unforgivable" at line 58 and "wicked" at line 60,
  one word rendered two ways in neighbouring sentences.
  `zheermao19` (its read) had "a cat too ... evil".
- Lines 60 to 63,
  真正罪恶的人，不会自责，更不会想要了结自己。他们只会去伤害其他的人，给他人带来无尽的痛苦。他们唯利是图，贪生怕死，得意时想着怎样获取利益，失意时想着怎样苟活于世。:
  "Those who are truly wicked don’t reproach themselves,
  and would never think of ending their own lives.
  /
  They do nothing but hurt other people,
  bringing them endless pain.
  /
  They think only of profit,
  clinging to life and dreading death.
  /
  When things go their way,
  they think of how to gain more;
  when things go badly,
  they think of how to scrape by."
  Right,
  and better than both the archive ("truly guilty won't feel guilty") and `zheermao20`:
  更不会 is carried by "would never",
  只会 by "do nothing but",
  无尽 by "endless",
  贪生怕死 by "clinging to life and dreading death".
  "gain more" for 获取利益 is a small addition,
  and "scrape by" leaves 于世 out;
  neither changes the sense.
- Line 64,
  虽然不知道你经历了什么，但Arts想，你内心中一定是一个善良美好的女孩子。:
  "Although I don’t know what you’ve been through,
  I believe you’re a kind and lovely girl at heart."
  Better than `zheermao20` ("cannot comprehend",
  "good girl"):
  善良美好 is "kind and lovely".
  D10 (`WB`,
  in part from the archive):
  the particles that carry the writer's certainty and rhythm are thin in the comfort passage:
  一定 (must) is dropped at line 64 and twice at line 78;
  the first 也 of 你也可能会困惑迷茫 (line 67) and of Arts也不理解 (line 71) is dropped;
  所以其实 is "Actually" (line 69;
  所以 lost) and 但Arts想告诉你 loses 但 (line 73);
  并不知道 is "doesn’t really know" (line 69);
  Arts想 is "I believe" where the letter says "Arts".
- Lines 66 to 67:
  "Arts understands what life is like without the support of a family.
  / You may be lonely,
  because you have no one to turn to when in pain;
  you may be helpless,
  because no one is willing to lend a hand when you fail;
  you may be lost,
  because you don’t know what you live for."
  Right and better than `zheermao20`:
  人生将会是怎样的 is "what life is like" (not the archive's invented "not easy to live"),
  愿意 is "willing",
  遭遇失败 is "when you fail",
  迷茫 is "lost".
- Lines 69 to 73,
  所以其实，Arts并不知道该怎样安慰你。……也正因如此，Arts不会劝说你活下去。:
  "Actually,
  Arts doesn’t really know how to console you.
  / Why are some people’s lives full of happiness and joy,
  while others’ hold only suffering and not a hint of hope?
  /
  Arts doesn’t understand,
  and Arts hates this unfair world as well.
  / That is exactly why Arts won’t try to talk you into living on.
  /
  What Arts wants to tell you is this:"
  Right and better than `zheermao20`:
  幸福与欢乐 is "happiness and joy",
  就只有痛苦 keeps "only",
  也正因如此 is "exactly why",
  劝说 is "try to talk you into" (`zheermao20` had "flatly tell",
  which the archive made up).
- Lines 74 to 76:
  "> You are not alone."
  and the two "For example" lines:
  "For example,
  there are trans women in every corner of the world who share our feelings and wishes,
  all working hard for our future.
  /
  For example,
  plenty of people have cut ties with their parents for one reason or another,
  and plenty are addicted to hurting themselves."
  Two 比如,
  as in the photo (`zheermao20` had three "For example" lines).
  我们 and 我们的未来 are kept as "our";
  因种种原因 is "for one reason or another";
  沉迷于自虐 is "addicted to hurting themselves";
  比比皆是 is "plenty".
  D11 (`WB`):
  作为MtF (as an MtF) no longer says that Arts is one;
  the sentence now reads that there are trans women who share "our" feelings,
  where `zheermao20` had "as a trans woman,
  I know that" (the archive had "as a trans person").
  The nested `> > You are not alone.` followed by a `> For example` line is the archive's structure;
  in the photo the sentence runs on in one paragraph.
  The read did not render it.
- Line 78,
  所以，请相信，世界上一定有能够愿意关心你，理解你，陪伴你的人，也一定有很多你知道或不知道的人希望你能够幸福，比如Arts。:
  "So please believe that there are people who would care for you,
  understand you and stay by your side,
  and that there are many people,
  whether you know them or not,
  who wish you happiness,
  like Arts."
  Faithful on 关心,
  理解,
  陪伴 and the closing 比如Arts;
  世界上 and the two 一定 are not carried (D10).
- Line 80,
  最后，如果你愿意的话，抱抱你呢。:
  "And finally,
  if you’d like,
  a hug for you."
  Right:
  最后 is "finally" and 抱抱你 is a hug given (the archive's "If you like,
  I’d like to hug you" and its conditional are gone).
- Lines 82 to 83:
  "Good night,
  / ArtsEpiphany".
  Right.

The whole letter against `zheermao20`:
the QQ sentence,
"dreading death",
"scrape by",
"our feelings ... our future",
"kind and lovely",
"talk you into living on",
"finally ... a hug for you" and the two 比如 are right here and were wrong or missing there.
What `zheermao20` had that this page lost or never regained:
"a cat too" and "evil" (D08,
D09),
the explicit "as a trans woman,
I know" (D11),
"must be" (D10).
Against `zheermao19` (as the `zheermao20` read quotes it):
the QQ sentence,
the comfort line and "finally" are now at its level,
and the cat and evil lines are behind it;
the read did not re-read the `zheermao19` page.

#### The account of her death, lines 85 to 93

- Line 85:
  "Zheermao attempted to leave this world several times but survived with difficulty through rescues."
  Unchanged since the archive,
  faithful.
- Lines 87 to 88:
  "However,
  in the early spring of 2024,
  / following the death of a friend,
  she made up her mind again to leave."
  Faithful.
- Lines 90 to 93,
  在离开之前几天，她给 One Among Us 的邮箱发送了一封邮件。那些秋叶的成员在看到后连夜给她发送了一份回信：:
  "A few days before her departure,
  / she sent an email to One Among Us’s mailbox.
  / When the One Among Us members saw it,
  / they sent her a reply that same night:"
  Better than `zheermao20` and the archive:
  邮箱 is "mailbox",
  连夜 is "that same night" (`zheermao20`:
  "overnight"),
  and 那些秋叶的成员 is the members,
  plural (the archive had "one member").
  Slice 9 settled on the translate lane's text at the lane contest and the slate declined
  (`winner drew only weight 1.5 ... keeping the fallback`,
  06:00:13 UTC);
  the text that shipped is the translate lane's.
  `introduced repetition in slices 9, 11 (archive 0, shipped 2): "One Among Us members"` is warned (05:56:31 UTC).
  The source says 那些秋叶的成员 in both sentences,
  so the repetition is faithful,
  not a defect.

#### The email, lines 99 to 124

Against `photo3.webp`
(请准备一下啦 / 是玖月折耳猫，我。。尽力活着了但是。。很抱歉，谢谢 / 如果成功死去了 / 请把我挂在上面，好嘛 / 谢谢你们):

- Line 99:
  "English translation of the email conversation:"
  Apparatus,
  rewritten under the same house rule.
- Line 102:
  "Please prepare a little."
  Equal to the archive and to `zheermao20`.
- Line 103:
  "It’s Zheermao."
  Right (the name is shortened as the page does everywhere);
  我 is not carried.
- Lines 104 to 105:
  "I tried to live,
  but... / I’m sorry.
  Thank you."
  D12 (`WB`,
  from the archive):
  尽力 (did my best) and 很 of 很抱歉 are lost;
  `zheermao20` had "I tried my best to live" and "I’m so sorry".
- Line 106:
  "If I succeed,
  please put up my entry."
  D13 (`WB`,
  from the archive):
  如果成功死去了 becomes "If I succeed",
  which leaves out what she hoped to succeed at,
  and 好嘛 (okay?)
  is lost;
  `zheermao20` had "If I die,
  please put my entry up,
  okay?"
  (it dropped 成功).
  The reader-protection rule reaches the method of a death;
  死去 names none,
  so the rule does not require the vagueness.
- Line 107:
  "Thank you."
  D14 (`WB`,
  from the archive):
  谢谢你们 is plural;
  `zheermao20` had "Thank you all".
- Line 110:
  "Good evening,"
  and no salutation line.
  D15 (`WB`,
  from the archive):
  玖月折耳猫： (the salutation) is lost,
  as in `zheermao20`;
  夜安 is a night greeting and "Good evening" is a register choice.
- Line 112,
  在刚给朋友写完邮件的时候刷新了收件箱，看到这封信有一点点难过，抱抱你。:
  "I just refreshed my inbox after writing an email to my friend,
  and seeing your letter made me a little sad.
  Hugs for you."
  Right (the archive's "I’m sorry.
  I’d like to hug you" is fixed):
  刚,
  有一点点难过 and 抱抱你 are all carried.
- Line 113,
  但也请你相信，无尽的黑暗之中也有细小的光芒，可以避开黑暗的锋芒，到那些有光的地方躲一躲。:
  "But please believe me,
  there are sparks of brightness in the endless nights as well,
  and you can dodge the darkness’s edge and shelter where the light is."
  Right on the sentence the translators' note says nobody could translate (it is now translated).
  "endless nights" for 无尽的黑暗 shifts darkness to nights;
  "believe me" for 请你相信 adds "me";
  both are small.
- Lines 114 to 115:
  "All from One Among Us will always be with you.
  / If you need help,
  please contact us or someone you trust."
  D16 (`WB`,
  from the archive):
  "All from One Among Us" is stilted for 「那些秋叶」的大家;
  还有身边你信任的人 loses 身边 (around you);
  `zheermao20` had "Everyone from One Among Us is always with you" and "the people around you whom you trust".
- Lines 116 to 117:
  "Please do not worry,
  we’ll try to write down what we remember.
  / But to keep living is the best way to rebel against the hatred in the world."
  D17 (`WB`,
  from the archive):
  故事 (stories) and 尽力 are lost,
  and 最大反抗 (the greatest resistance) against 恶意 (malice) becomes "the best way to rebel against the hatred";
  `zheermao20` had both right.
- Lines 118 to 119:
  "The dynamic stories about you or me,
  the stories that are living,
  are much more alive than finished stories.
  / That’s what we can never write down."
  D18 (`WB`,
  from the archive):
  属于你我 (belong to you and me) is "about you or me";
  一直能续写下去 (can keep being written) is "that are living";
  静态 (static) is "finished";
  我们的笔 (our pen) is gone.
  `zheermao20` had "dynamic ... static" and "our pen can never reach".
- Line 120:
  "Love you.
  Do not give up easily."
  Right in register (`zheermao20`:
  "I love you.
  Please don’t give up easily",
  firmer than 爱你).
- Lines 121 to 122:
  "It must still be cold in the north.
  I’d hug you again.
  / Spring is coming.
  Ice will melt,
  no matter how hard it is."
  D19 (`WB`,
  from the archive):
  再给你一个温暖的抱抱 is a gift of a hug,
  not a conditional "I’d hug you again",
  and 温暖 is lost;
  慢慢 (slowly) is lost.
  `zheermao20` had "I’ll give you another warm hug" and "slowly thaw".
- Line 124:
  "With best wishes for spring."
  A fair rendering of 顺颂春祉 (restored by this revision;
  the archive had dropped it).
- Line 126:
  the note `{/* 可以避开黑暗的锋芒，到那些有光的地方躲一躲。 这句不知道如何翻译 */}` stands on a line of its own after the block.
  D20 (`C`):
  the archive had it inside the email;
  `withBlockNotes` puts a kept note after the replacement.
  The note says the sentence could not be translated,
  though it now is at line 113;
  the owner's rule of 2026-10-20 keeps notes as written.
  The same as `zheermao20`.

The email against `zheermao20`:
it fixes the three things the archive got wrong (the sadness line,
the shelter-in-the-light sentence,
the sign-off),
and `zheermao20` had fixed those same three,
so this block holds the archive's other weaknesses that `zheermao20` had also repaired (D12 to D19).
The block is worse than `zheermao20`'s and better than the archive's.

#### The close, lines 128 to 145

- Lines 128 to 132,
  后来，那些秋叶的成员在 Telegram 上联系上了她，试着安慰了她，与她聊了许久。包括跨性别相关的经历，爱与被爱，以及活下去的意义，试图多挽留她一会儿。:
  "Later,
  / One Among Us members reached her on Telegram and tried to comfort her.
  /
  They talked with her for a long time U+2014 about experiences related to being transgender,
  /
  about loving and being loved,
  / and about the meaning of going on living U+2014 trying to hold on to her a little longer."
  (the two `U+2014` are the characters themselves on the page,
  written here by code point.)
  Faithful in sense,
  and better than the archive ("that member",
  "keep her for a little longer" run in one sentence).
  联系上了 is "reached",
  which is closer than `zheermao20`'s "reached out to".
  D21 (`W`;
  WITHDRAWN on 2026-10-21 (UTC):
  the source has a full stop and a comma at the two places,
  so by the owner's dash ruling the dashes are not a defect,
  see [the causes](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-causes.md),
  "Finding 7";
  original text kept):
  the two `U+2014` characters,
  a house-style break (see the paragraph on em-dashes in "The page");
  `zheermao20` split the same content with commas and had none.
  The text is the consolidation's
  (`consolidate gate ... settled on consolidated`,
  06:00:37 UTC);
  a polish that won 2.5 of 5 ballots was then declined at the polish gate
  (`settled on neither, ships base`,
  06:01:28 UTC).
- Lines 134 to 135,
  尽管如此，她仍旧坚持着自己的决定……最后仍然选择了离开。:
  "Despite this,
  / she still held to her decision... and ultimately chose to leave."
  Better than the archive and `zheermao20` ("insisted"):
  仍旧 is carried by "still held".
  D22 (`W`):
  the second 仍然 is not carried ("ultimately");
  a consolidated candidate that carried it split the gate 2 to 2
  (deepseek and mercury for it,
  minimax and gemma against) and the standing text shipped (06:01:45 UTC).
- Line 137:
  "After her departure,
  Han Lianyi wrote an [epitaph](https://t.me/Lianyi520/565) for her."
  D23 (`W`,
  from the archive):
  离世 is death and the page says "departure",
  though line 88 says "death" for the friend;
  悼文 is a memorial piece and "epitaph" is an inscription.
  The `zheermao20` read could not tell whether declared names licensed it;
  the reader of this page did not read the declared-names sheet either,
  so D23 stays open.
  `REFERENCE 1 https://t.me/Lianyi520/565: success, 25 chars, cached`,
  and the reference attestation lost two voices
  (`stage-voice-lost (reference-attest glm-5.3)`,
  `google.gemma-4-e2b`).
- Line 139:
  "Perhaps Zheermao’s departure was not an unexpected ending."
  Equal to the archive (the repair lane's "was no accident" was not adopted).
- Lines 141 to 142,
  但如今，一切也只能化作一声长叹……:
  "But now,
  / everything can only turn into a long sigh..."
  Better than the archive and `zheermao20`:
  the active "turn into" reads 化作 correctly (they had the passive "be turned into").
  D24 (`W`):
  也 (also) is not carried in 一切也只能.
- Lines 144 to 145:
  "Contributor for this entry:
  / Echo of One Among Us".
  Equal to the archive;
  the line break after the colon is a soft break in Markdown.

### What changed between zheermao20 and zheermao21

Improved:

- The letter:
  the QQ sentence,
  "dreading death" and "scrape by",
  "our feelings ... our future",
  "kind and lovely",
  "talk you into living on",
  "finally ... a hug for you",
  two 比如,
  "admin of Transland" (lines 53 to 83).
- The email:
  the sadness-and-hug line,
  the shelter-in-the-light sentence and the sign-off hold,
  as they did in `zheermao20` (lines 112 to 124).
- 小 is back (line 21).
- 热情地 is back on her manner (line 32).
- "Arts’s letter" is singular,
  matching its label (line 44).
- 连夜 is "that same night",
  邮箱 is "mailbox" (lines 92 to 93).
- The second 仍旧 is carried (line 135).
- "turn into" is active (line 142).

Worse:

- The quote:
  有一天 and 你 are lost,
  就好了呢 is "would be better",
  对不起了 is "so sorry" (D02,
  D03,
  D05).
- The email:
  D12 to D19 (尽力,
  很抱歉,
  谢谢你们,
  好嘛 and 成功,
  the greeting and the "people around you",
  故事,
  最大反抗,
  你我,
  动态 and 静态,
  the warm hug).
- Two `U+2014` characters (lines 130 and 132).
- The letter's `罪恶` is two words (D09);
  作为MtF is gone (D11);
  "cat-like person" stays (D08).

Stayed:

- The account of her death (lines 85 to 88),
  the epitaph line (line 137),
  the departure line (line 139),
  the credit (lines 144 to 145),
  the front matter `desc` (line 6),
  "that long" (line 16).
- The note after the blockquote and its stale text (line 126).
- 也 lost from the library sentence (line 34).

By slice,
in short:
slices 1,
4,
6,
9,
12,
14 and the letter are better;
the quote (0) and the email are worse;
slice 11 is the same in sense and worse in style;
2,
3,
5,
7,
8,
10 and 13 are unchanged.
Net:
the page is better than `zheermao20`'s,
because the letter is the longest block and is now mostly right,
but not uniformly:
the first thing a reader meets,
the quote,
and the email are weaker,
and the page breaks the no-em-dash style.
One sample each cannot separate a regression from run to run spread,
and the read did not measure that spread.

Slices that improved on the archive:
0 (milk tea,
the name as subject,
no "In addition"),
1 (也,
小),
3 (the full title),
4 (digital products,
也,
热情地),
5,
6,
9,
11 (members,
联系上了),
12 (仍旧),
14 (the active verb),
the letter and its label,
and the email and its label.
Slices that read worse than the archive anywhere:
slice 11 for the two `U+2014` characters,
and nothing else the read found.

Claims of the `zheermao20` read checked against this run:

- The quote offers milk tea and names Zheermao:
  holds.
- The game is "Mystery Lover 2:
  Forgotten Truth":
  holds.
- The `desc` keeps the archive's wording while the quote reads milk tea:
  holds (D01).
- The note after the blockquote with its stale text:
  holds (D20).
- Em-dash count 0:
  no longer holds (2).
- The letter is the archive's text with partial fixes,
  because the review chose a narrow revision:
  does not hold this run;
  the review chose a comprehensive revision.
- "Qwen was cut at the 120 s grace and was the cause":
  the cut recurs (three archive review rounds,
  three later rounds) and did not decide the letter.
- Hyper dry the whole run:
  holds.
- Synthetic dry partway:
  holds (31.4 of 40.7 min against 22 of 33.0 min).

### Class verdict

No class named by the read.
Every departure listed is a wording in one place;
none touches who died,
when,
how she is named,
or what the writers tell her.
D13 leaves vague what she hoped to succeed at,
as the archive did.
The log shows none of the five markers that could name a class and no `[error]`.

Counts by the read's labels,
over the 24 departures D01 to D24:

- `C` (traced to code or policy):
  2 (D01,
  D20).
- `WD` (a decision rule chose what shipped):
  4 (D02 to D05).
- `WB` (the whole-block archive review chose what shipped):
  12 (D08 to D19).
- `W` (model wording,
  no code cause found by the read):
  6 (D06,
  D07,
  D21 to D24).

Against the archive,
19 repeat its wording (D02 to D08,
D10,
D12 to D19,
D22 to D24),
3 are new to this run (D09,
D11,
D21),
and 2 are code effects (D01,
D20).
Of the 19,
`zheermao20` had fixed,
wholly or in part:
D02,
D03,
D05,
D12,
D13,
D14,
D16,
D17,
D18 and D19.

The most serious departures,
by what a reader is told:

- D12 to D19 together:
  Zheermao's last email and the members' reply read thinner,
  in places stilted
  ("If I succeed,
  please put up my entry";
  "All from One Among Us will always be with you";
  "I’d hug you again").
  The reply is the message to a girl in danger,
  and `zheermao20` had these lines right.
- D02,
  D03 and D05:
  the opening quote of the page,
  her own last words,
  lose "one day" and say "better" and "so sorry".
- D21:
  two em-dash characters,
  in the one place the page breaks its house style.

The best three improvements are the letter's QQ sentence and comfort lines,
the email's restored sadness,
light and sign-off lines,
and the small ones in slices 1,
6,
9 and 14 (小,
the singular letter,
"that same night",
the active verb).

Whether any departure is a class,
and its cause where the read gave none:
cause not yet traced for every departure labelled `WD`,
`WB` and `W`;
the separate tracing reports in a later record.

Traced on 2026-10-21 (UTC),
original text kept ([the causes](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-causes.md)):
the opening quote (D02,
D03 and D05) is the minimum selection weight working as written (finding 5,
no class);
the email (D12 to D19) is class A2,
the whole-revision pick of the archive block review,
which the composite candidate of ledger B712 addresses in the code that followed this build (finding 6);
D21 is withdrawn (finding 7).
The other departures labelled `WD`,
`WB` and `W` stay untraced.


### What the page read did not do

- It did not read the declared-names sheet or any prompt
  (the run directory holds replies,
  not prompts),
  so D23 stays open.
- It did not run a spread measurement.
- It did not render the page.
