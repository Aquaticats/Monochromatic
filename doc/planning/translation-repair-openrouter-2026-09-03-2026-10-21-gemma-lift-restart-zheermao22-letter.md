# OpenRouter and the corpus passes: the letter of zheermao22 on build 2a336603bf (2026-10-21)

What was read on 2026-10-21 of the archive block review of the letter and the email of `zheermao22`
(entry `zheermao101`),
and of the shipped letter against the Chinese,
line by line.
The run's log,
meters and seats are in
[the run of zheermao22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-run.md),
and the rest of the shipped page is in
[the page of zheermao22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-page.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of the restart share are in
[the Gemma-lift restart overview](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart.md).

## zheermao22 letter read, 2026-10-21, 23:16 UTC: the letter regressed to the archive's text with ten lines changed

The letter read is part of the read of `zheermao22` (SETTLED in 79.0 min of pass),
made at 23:16 UTC on 2026-10-21 on tip `2a336603bf38a6a53aeb4f3e70f0a3447acf5487`.
The ids D01 and on,
and the classes C,
WD,
WB and W,
are defined in
[the page of zheermao22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-page.md#the-page).
The letter holds D10 to D18.

### The letter block

This is the block the investigation of the archive block review was written about,
and the one `zheermao21` recovered.
B712 to B715 are in this build:
the review round waits the writers' window,
a composite of disjoint revisions is attempted,
the review findings are kept,
and a dropped reply is logged with its reason.

The reviews ran email first,
then its label,
the letter,
then its label.
The artifact records `archive block reviewed and revised` for `block/20`,
`block/19`,
`block/14` and `block/13`;
`block/14` is the letter,
`block/13` its label,
`block/20` the email,
`block/19` its label.

The seats of each review round are a rotated window of the writers' bench:
`archive-block-review-stage.ts:103` to `110` says the window is "rotated by the sheet text and a sheet edit
elsewhere moves which seats it holds".
The ruling of B732 edited the sheet of the archive block review (`e5bc27f9f0` lists
`archive-block-review-wire.ts`),
so the seating of this run need not match `zheermao21`'s:
the letter round here seated `inception/mercury-2.5`,
`gemma-4-26b-a4b-it`,
`deepseek-v4.1-flash`,
`minimax-m3`,
`hf:zai-org/GLM-5.3-Flash`,
`glm-5.3` and `google.gemma-4-e2b` (7;
the last unreachable),
with no Kimi and no Qwen;
`zheermao21` seated 8 with Kimi and Qwen and without `minimax-m3`.
That the edit caused the move is an inference from the code comment and the difference in seating;
The rotation was not run.

The letter's review ran 21:15:51.6 to 21:25:09.1 UTC (558 s;
`zheermao21`:
349 s) in three rounds:

- Review round (log lines 489 to 523;
  the replies are in `prompt-payloads/`):
  `5/7 heard, 187012ms total, 142977ms to quorum, 44035ms in grace`.
  - `inception/mercury-2.5`,
    8.4 s:
    `revise` (reply `0395cc78`),
    36 lines,
    keeping the archive's QQ sentence and writing "cat" for "cat-like person".
  - `gemma-4-26b-a4b-it`,
    8.9 s:
    `revise` (`352f45eb`);
    refused for shape (`the block is 1 block (blockquote) and the revision is 2 blocks (paragraph, blockquote)`,
    log line 548).
  - `deepseek-v4.1-flash` on NeuralWatt,
    51.2 s:
    ran to its cap of 13,082 tokens with 0 content characters and 51,791 reasoning characters
    (`finish_reason=length`,
    0.6835 Wh).
    The nudged re-ask on Hyper (5.0 s,
    1,161 tokens) answered `source-supported`,
    quoting 其实，Arts也是一只猫猫。猫猫怎么会是罪恶的呢？猫猫是世界上最可爱的存在哦。
    (reply `3a8b9bb8`).
    In `zheermao21` the same model on the same block wrote a complete 31-line `revise` in 35 s and 8,050 tokens.
  - `minimax-m3` on Hyper,
    57.7 s:
    ran to its cap of 10,822 tokens with 0 content characters (`max_tokens`).
    The re-ask on OpenRouter (Together,
    1.9 s,
    95 tokens) answered `source-supported`,
    quoting the first paragraph (reply `0925fa39`).
  - `hf:zai-org/GLM-5.3-Flash` on NeuralWatt,
    100.7 s:
    ran to 18,316 tokens with 0 content (2.7184 Wh).
    The nudged re-ask on Hyper ran 86.2 s to 18,316 tokens again (`max_tokens`),
    and the log says `hyper answered schema-mismatch too` (line 521).
  - `glm-5.3` on Hyper,
    142.9 s,
    10,187 tokens:
    `revise` (`9767b45d`),
    34 lines.
    Its cap is 22,067 (`model-cards.ts:324`),
    twice the cap of the two models that failed.
- Recovery round (lines 524 to 549):
  one cut-short voice re-asked.
  `hf:zai-org/GLM-5.3-Flash` was asked again on NeuralWatt (5.0034 Wh,
  18,316 tokens,
  0 content,
  21:20:36 UTC),
  then on Hyper,
  and the round ended `0/1 heard, 180002ms total` at 21:21:58 UTC:
  `abandoned 180000ms after quorum (cut-mid-reply after 0 answer chars and 69336 reasoning chars)`.
- Select round (lines 557 to 576):
  `5/6 heard, 190449ms total, 70449ms to quorum, 120000ms in grace`.
  Qwen was cut at the 120 s grace (0 answer characters,
  43,372 reasoning characters) for the second archive select round in a row.

Candidates and assessments:

- `archive composite not built: candidate 1 has 34 lines against the block's 36 lines, and lines can only be matched one for one` (line 546).
- `archive review: comparing 2 admissible revisions in 3 candidates; 5 eligible assessments from 5 heard reviews`:
  the admissible revisions are `glm-5.3`'s and `inception/mercury-2.5`'s;
  the third candidate is the archive.
- Five ballots:
  `hf:moonshotai/Kimi-K3`,
  `minimax-m3`,
  `deepseek-v4.1-flash` and `glm-5.3` chose candidate 1 (`glm-5.3`'s own ballot at weight 0.5,
  the `select-self-vote` finding);
  `gemma-4-26b-a4b-it` chose candidate 3 (the archive).
  `candidate 1 from glm-5.3 won weight 3.5 (ballots: 5)` (line 575).

What shipped:
the letter on the page (lines 52 to 85) is `9767b45d` exactly
(a `diff` of the letter as shipped against the reply's `replacementText` differs only in apostrophe form:
the reply has straight apostrophes,
the page the typographic ones).
`glm-5.3`'s revision changed 10 of the archive's lines in five hunks
(archive working-tree lines 53 to 54,
57,
61 to 63,
73 and 76 to 78,
by the `diff` of the archive against the archive after the review):
the QQ sentence (fixed),
the MtF sentence and the examples (fixed),
"endless suffering",
"fearing death",
"scrape by" and "urge you to keep living" (fixed),
and "unforgivable" to "guilty" (new damage).
It leaves the rest of the archive's text:
"the online forum",
"cat-like person",
"cannot comprehend",
"good girl",
"not easy to live",
"puzzled",
the three "For example" lines and "If you like,
I’d like to hug you".

Against the investigation and `zheermao21`:

- The comprehensive 31-line `deepseek-v4.1-flash` revision of `zheermao21` did not recur.
  Three of the five reviewers that could have written one (`deepseek-v4.1-flash`,
  `minimax-m3`,
  `hf:zai-org/GLM-5.3-Flash`) ran to their completion caps before answering,
  and the two that were re-asked answered `source-supported` on a different provider with far fewer reasoning tokens.
  The log cannot say what the first reply would have been.
- The composite (B712) refused on line counts:
  the one thorough revision removed two lines (the QQ paragraph's two lines became one,
  the three "For example" lines became two),
  and the rule keeps only revisions that keep the block's line count
  (`archive-composite-revision.ts:36` to `42`).
- The depth claim of the investigation ("every thorough revision came from Hyper") holds in this run:
  the only complete revision came from a Hyper call (`glm-5.3`).
- One sample each cannot separate this from run to run spread,
  and this read did not measure that spread.
  What the log does show is two causes that are not model quality:
  the caps,
  and the seating that the sheet edit moved.

The email block (`block/20`) for comparison,
same terms:

- Review round (log lines 360 to 399):
  `6/7 heard, 111868ms total, 105669ms to quorum, 6199ms in grace`.
  - `inception/mercury-2.5` 5.9 s,
    `gemma-4-26b-a4b-it` 7.6 s:
    both short (`source-supported` from `inception/mercury-2.5`,
    `revise` from `gemma-4-26b-a4b-it`).
  - `deepseek-v4.1-flash` on NeuralWatt,
    31.3 s:
    `revise` (8,892 tokens,
    1.0122 Wh),
    19 lines.
  - `minimax-m3` on Hyper:
    ran to its cap (10,822 tokens) at 21:10:14 UTC;
    the OpenRouter re-ask answered usably in 10.1 s (735 tokens,
    2,752 content characters).
  - `glm-5.3` on Hyper 104.4 s and `hf:moonshotai/Kimi-K3` on Synthetic 110.6 s:
    `revise`.
  - `archive composite not built: candidate 2 has 23 lines against the block's 24 lines` (line 398).
  - `comparing 5 admissible revisions in 6 candidates; 6 eligible assessments from 6 heard reviews`.
- Select round:
  `5/6 heard, 193275ms total, 73273ms to quorum, 120002ms in grace`;
  Qwen was cut at the grace (0 answer characters,
  42,338 reasoning characters).
  `inception/mercury-2.5` and `gemma-4-26b-a4b-it` chose candidate 1;
  `hf:zai-org/GLM-5.3-Flash`,
  `hf:moonshotai/Kimi-K3` and `minimax-m3` chose candidate 5.
  `candidate 5 from deepseek-v4.1-flash won weight 3 (ballots: 5)` (line 426).
  The ballot texts name what candidate 5 fixes:
  好嘛 as "okay?",
  谢谢你们 as "thank you all",
  the sadness line,
  the shelter-in-the-light sentence and the closing 顺颂春祉.
- What shipped:
  the email (page lines 108 to 126) is `deepseek-v4.1-flash`'s 19-line revision exactly
  (a `diff` of the email as shipped against that reply:
  the shipped file adds the label and a blank line).
  The note between translators was kept and put after the block
  (`replaced with 1 note between translators kept`,
  21:14:09 UTC).
- The email recovered.
  It came from one whole revision that won 3 of 5 ballots,
  not from the composite.

The two labels:

- `block/19`:
  `English translation of the One Among Us email conversation:` won with weight 3 of 6 ballots
  (candidate 2 from `hf:Qwen/Qwen3.8-27B`,
  line 486).
- `block/13`:
  `English translation of Arts’s letter:` won with weight 5 of 6 ballots
  (`candidate 1 from composite(glm-5.3 + deepseek-v4.1-flash + hf:zai-org/GLM-5.3-Flash)`,
  line 627).
  No `composite built` line exists,
  so the name is read as the producer list of three byte-identical revisions of a one-line block,
  which the composite header calls "the only merge the slate already trusts"
  (`archive-composite-revision.ts:44` to `47`).
- Both labels remove the positional word `preceding` under the house rule (`house-policy.ts:111`),
  as in `zheermao21`.
- B715's logged drops fired:
  six `archive review dropped the editorial-context reply` warns
  (three on each label block,
  `the block of 56 characters is not wholly apparatus` and the like),
  and two `archive review discarded uncorroborated retention claim` findings.

### Arts’s letter on the page [50 to 85]

This block is where the page regressed.
Its review is traced under "The letter block";
the shipped letter is the `glm-5.3` revision.
Line by line against `photo1.webp`,
`photo2.webp` and their transcription:

- [50] "English translation of Arts’s letter:"
  Apparatus rewritten from "the preceding letter" under the house rule against positional pointers;
  the page is better for it.
- [52] 我是Transland管理员ArtsEpiphany，你可以叫我Arts。:
  "I am ArtsEpiphany,
  an admin from the online forum Transland.
  / You can call me Arts."
  D10 (WB,
  archive):
  "the online forum" is not in the photo;
  `zheermao21` had removed it.
- [54] 我看到你在跨岛留下了自己的QQ号，但是我搜索之后发现你似乎设置了禁止通过QQ号查找，因而我无法加你好友，才选用这种方式和你联系，真的很抱歉！:
  "I saw the QQ instant messaging number you left on Transland,
  but when I searched for it,
  I found you seem to have turned off being found through your QQ number,
  so I couldn’t add you as a friend;
  that’s why I chose to contact you this way,
  then a `U+2014`,
  then I’m really sorry!"
  Right in every clause (this repairs what the archive and `zheermao20` shipped wrong);
  `zheermao21`'s sentence was equally right.
  D11 (WB,
  archive):
  "instant messaging" glosses QQ,
  which the house policy keeps as a name (`house-policy.ts:93` names "a product such as QQ").
  D12 (WB):
  the `U+2014` at [54],
  at [77] and at [122] are new to this run.
  [54] sits at the source comma after 才选用这种方式和你联系,
  [77] at the comma after 沉迷于自虐的人,
  and [122] at the comma after 要来得鲜活.
  Under Finding 7 of [the causes document](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-causes.md)
  (a page’s dash is a defect only where its source has no matching break or pause),
  none is a defect;
  They are listed because the brief for the read asked for every one.
  The first two are in the `glm-5.3` revision (`9767b45d`),
  the third in `deepseek-v4.1-flash`'s email revision.
  The pair of `zheermao21` (slice 11) is gone.
- [56 to 58] 其实，Arts也是一只猫猫。猫猫怎么会是罪恶的呢？猫猫是世界上最可爱的存在哦。:
  "Actually,
  Arts is also a cat-like person.
  / How can a cat-like person be guilty?
  / Cat-like people are the cutest beings in the world."
  D13 (WB,
  archive):
  猫猫 is "cat-like person" three times (the writer says she is a cat).
  D14 (WB,
  new in part):
  罪恶 is "guilty" at [57] and [59].
  The archive had "unforgivable" at [57] and "guilty" at [59];
  the `glm-5.3` revision changed [57] to "guilty" (the `diff` of the archive against the archive after the review),
  so the question and the next sentence now agree on a word that does not render 罪恶,
  and "Those who are truly guilty won’t feel guilty."
  [59] is an empty sentence
  (真正罪恶的人，不会自责).
  `zheermao21` had "unforgivable" and "wicked" (its D09).
- [60 to 63] 更不会想要了结自己。他们只会去伤害其他的人，给他人带来无尽的痛苦。他们唯利是图，贪生怕死，得意时想着怎样获取利益，失意时想着怎样苟活于世。:
  "They won’t think of ending themselves.
  / They’ll keep harming other people,
  bringing them endless suffering.
  / They only care about how much benefit they can get for themselves,
  clinging to life and fearing death.
  / When they are lucky,
  they think of how to obtain more;
  when they are unlucky,
  they think of how to scrape by."
  Lines 61 to 63 are `glm-5.3`'s fixes and read well
  ("endless suffering",
  "fearing death",
  "scrape by").
  D15 (WB,
  archive):
  更 ("even less") is lost at [60],
  and 得意/失意 are "lucky/unlucky" at [63] where the words say success and setback.
- [64 to 67] 虽然不知道你经历了什么，但Arts想，你内心中一定是一个善良美好的女孩子。 ... 人生将会是怎样的 ... 你也可能会困惑迷茫:
  "Although I cannot comprehend what you’ve been through,
  I believe you’re a good girl at heart.
  /
  Arts understands that it’s not easy to live without the support of a family.
  / You may be lonely,
  because you have no one to turn to when in pain;
  you may be helpless,
  because no one extends their hand when in trouble;
  you may be puzzled,
  because you don’t know what to live for."
  D16 (WB,
  archive):
  不知道 (don’t know) is "cannot comprehend",
  善良美好 is "good",
  一定 is lost,
  Arts想 is "I believe";
  人生将会是怎样的 (what life will be like) is the invented "it’s not easy to live";
  困惑迷茫 is "puzzled" and the first 也 is lost.
  `zheermao21` had "don’t know",
  "kind and lovely",
  "what life is like" and "lost".
- [69 to 73] 所以其实，Arts并不知道该怎样安慰你。 ... 也正因如此，Arts不会劝说你活下去:
  "Actually,
  Arts doesn’t really know how to console you.
  / Why do some people live in happiness but others live in suffering,
  without a hint of hope?
  / Arts doesn’t understand.
  / Arts hates this unfair world as well.
  / Because of this,
  Arts won’t urge you to keep living."
  Right and close to the source;
  所以 is lost and 幸福与欢乐 is "happiness".
  "urge you to keep living" is `glm-5.3`'s fix of the archive's "flatly tell".
- [74 to 77] "> You are not alone."
  and the examples:
  "For example,
  as a trans woman,
  there are people in every corner of the world who share the same feelings and wishes as us,
  all striving forward for our future.
  / For example,
  there are people who have cut ties with their parents for all sorts of reasons.
  For example,
  there are people who have issues with self-harm,
  then a `U+2014`,
  then such people are everywhere."
  Recovered:
  作为MtF is back as the speaker’s own identity,
  我们 and 我们的未来 are kept,
  因种种原因 is "for all sorts of reasons",
  比比皆是 is "such people are everywhere".
  D17 (WB,
  archive in part):
  the source has two 比如 and the page has three "For example" (the archive’s structure);
  沉迷于自虐 (addicted to self-harm) is the softer "have issues with self-harm".
  The nested `> > You are not alone.` followed by a `> For example` line is the archive’s structure;
  in the photo the sentence runs on in one paragraph.
  It was not rendered.
- [79 to 80] 所以，请相信，世界上一定有能够愿意关心你，理解你，陪伴你的人，也一定有很多你知道或不知道的人希望你能够幸福，比如Arts。:
  "Therefore,
  please believe that there are people who would care,
  understand,
  and accompany you.
  / There are also people who would wish for your happiness,
  whether you know them or not,
  like me,
  Arts."
  Faithful on 关心,
  理解,
  陪伴 and the closing 比如Arts;
  世界上 and the two 一定 are not carried.
- [82] 最后，如果你愿意的话，抱抱你呢。:
  "If you like,
  I’d like to hug you."
  D18 (WB,
  archive):
  最后 (finally) is lost and the hug is an offer where 抱抱你 gives one;
  `zheermao21` had "And finally,
  if you’d like,
  a hug for you."
- [84 to 85] "Good night,
  / ArtsEpiphany".
  Right.

Whole-letter reading against `zheermao21`:
kept from it are the QQ sentence,
"fearing death",
"scrape by",
"our feelings ... our future" and "urge you to keep living".
Lost from it are "kind and lovely",
"finally ... a hug for you",
the two 比如,
"admin of Transland",
"don’t know" and "what life is like".
New damage is "guilty" (D14).
Gained over it is "as a trans woman" as the speaker.
The letter is the archive’s text with ten lines changed (nine for the better,
one for the worse),
and so is close to the shape `zheermao20` shipped.
