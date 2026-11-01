# OpenRouter and the corpus passes: 2026-10-21

What was read and decided on 2026-10-21.
Entries:
zheermao20,
CuspariaKLSY20.

The passes after the rotation restarted on build `3637592bd4`
(`zheermao21`,
`CuspariaKLSY21` and `XingZ6016`)
are recorded in
[the rotation restart](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## zheermao20 read, 2026-10-21, 03:19 UTC: SETTLED in 33.0 min of pass; no class traced to code; the letter fell back toward the archive

`zheermao20` (entry `zheermao101`) ran on tip `31ce5369e1…` and pipeline `sha256-tree-v1:e9eda9fe032032f4896f29b0c82099ae5b04084f209ce37144bb83d78eadcfcd`,
the build that carries the NeuralWatt day anchor,
the credential hold and the energy recalibration,
and its `START` line carried both.
The `START` line carries no time;
the first timestamp of the log is 02:15:32 UTC and the last is 02:48:31 UTC,
on 2026-10-21.
The corpus input is the pin `a41fc607ea`.
The `DONE` line reads `elapsed=1978016ms`,
which is 33.0 min:
`TALLY zheermao101 status=SETTLED slices=15 repairStatus=repaired repairIssues=16 repairAccepted=13
repairResolved=13 repairFindings=220 repairChanged=7 translateStatus=complete translateChanged=10
documentsDiffer=10 pageChanged=8 pageSilent=0 alignmentFindings=13 selection=contested`.
`DESTINATIONS zheermao101 source=2 page=3 dropped=0`,
and the publish wrote 15 slices into a page of 6007 characters.
The front matter was the archive's (`FRONT MATTER entry=zheermao101 authority=archive`,
02:15:34 UTC).

The log holds 424 `[warn]` lines (zheermao19:
395),
no `[error]` line,
no `DEFECTS` line,
and none of `REATTEMPT`,
`STALLED`,
`mdx-downgraded` or `StreamOverrunError`.
Other counts over the log (6 decision lines read `keeping the fallback` or `keeping the repaired text`,
against zheermao19's 12):

- `seat unreachable` on 245 lines (zheermao19:
  166),
  and `HTTP 429` on 12 lines (10),
  all from 02:37:25 UTC to 02:38:46 UTC.
- `closed on a short bench`:
  none (zheermao19:
  30).
  The naturalness reviews of consolidation read `4/7 usable, ... quorumNeeded=4 quorumOver=7 unreachable=3`,
  where zheermao19's needed 2 and closed on `3/7` or `2/7`.
- Four warn kinds that zheermao19's log does not have,
  each traced and none a page fault:
  `refused by the page grammar` (3 lines) and `MdxParseError` (2) are the refusal of one model's broken picture tag (the section on two warns that look like a code fault);
  `introduced repetition` (1) describes the translate lane's text before consolidation,
  not the shipped page;
  `archive-revision-refused` (1) is a guard working (the same section).

The full line-by-line read is kept at `/var/home/user/temp/agent/x15-fix/reads/zheermao20-read.md`;
this section keeps every departure it found.

### Seats and meters

Usable answers of asked,
from the `SEAT` lines after `DONE`:
`deepseek-v4.1-flash` 160 of 160,
`minimax-m3` 160 of 161,
`inception/mercury-2.5` 159 of 162,
`gemma-4-26b-a4b-it` 142 of 147,
`hf:zai-org/GLM-5.3-Flash` 57 of 57,
`typesafe/jev-1.13` 36 of 36,
`xiaomi/mimo-v2.6-flash` 26 of 26,
`hf:moonshotai/Kimi-K3` 24 of 43 (19 threw),
`hf:Qwen/Qwen3.8-27B` 21 of 158 (137 threw,
3 abandoned),
`glm-5.3` 0 of 4,
`google.gemma-4-e2b` 0 of 4 and `google.gemma-4-31b` 0 of 3.
The run ended with `SEATS DARK: 3 of 12 seats asked produced nothing usable this run`,
naming `glm-5.3`,
`google.gemma-4-e2b` and `google.gemma-4-31b`.

- The Bedrock key had expired again (`SEAT CREDENTIAL REFUSED gemma-4-26b-a4b-it on bedrock`,
  HTTP 401,
  02:15:36 UTC).
  The log says `bedrock is held out for every model for the rest of the run, because one credential serves them
  all. gemma-4-26b-a4b-it moves to openrouter`.
  That is the hold of w249 at work:
  `gemma-4-26b-a4b-it` was served by `google/gemma-4-26b-a4b-it` on OpenRouter (`served by "DeepInfra"`) and gave 142 usable answers,
  where zheermao19 withheld it after 4 asks.
  Its 5 throws are three picture reads (`every provider serving this model had its credential refused this run (bedrock)`) and two replies cut at the content bound of 1024 characters.
  `google.gemma-4-e2b` and `google.gemma-4-31b` have no other provider and stayed dark.
  The Bedrock meter never moved (5.02 USD).
- Hyper was dry from the first `METERS` line (02:15:34 UTC,
  `hyperBalance=0`) to the last,
  so `glm-5.3`,
  whose only provider is Hyper,
  threw its 4 asks with `every provider serving this model is out of budget, seat unreachable`.
- Synthetic's weekly meter read 2% at 02:15:34 UTC and 0.30% at 02:36:42 UTC;
  `Qwen3.8-27B` and `Kimi-K3` first answered `HTTP 429` at 02:37:25 UTC,
  `Kimi-K3` answered `HTTP 402`,
  and the first `METERS` line with `synthetic=dry` was taken at 02:37:26 UTC.
  That is 22 min into the run (zheermao19:
  21 min).
- OpenRouter went from 14.99 USD to 14.50 USD;
  its `SPEND` lines sum to 0.498 USD (`minimax/minimax-m3` 0.269,
  `google/gemma-4-26b-a4b-it` 0.101,
  `inception/mercury-2.5` 0.097,
  `typesafe/jev-1.13` 0.016,
  `xiaomi/mimo-v2.6-flash` 0.015,
  `z-ai/glm-5.3-flash` 0.001).
- NeuralWatt went from 521.298 Wh to 490.017 Wh on the day that starts at 19:15:08 UTC (the anchor learned from the quota's `current_period_start`).
  The `settleNeuralWattCall` lines count 215 calls and 31.332 Wh measured against 30.728 Wh estimated (`deepseek-v4.1-flash` 160 calls,
  13.094 Wh against 11.918 Wh;
  `glm-5.3-flash` 55 calls,
  18.238 Wh against 18.810 Wh).
  The 214 calls settled between the first and the last `METERS` line sum to 31.281 Wh,
  exactly the meter's fall,
  which zheermao19 could not reconcile.
  The run-end line reads `NeuralWatt plan energy: no plan energy was spent between the start and the end of the run; the free
  daily energy is not in this figure`,
  and the start line `NeuralWatt quota at the start of the run: 3.1687 kWh charged this period`.
  The first reading is 0.826 Wh under zheermao19's last,
  a gap this read did not explain.
  The bucket means are in [the handover](../handover/translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20-zheermao20-to-zheermao21-reads-and-the-w253-to-w257-fix-rounds.md#next).

The condition that is new against zheermao19 is that the bench was not short:
seven seats are rostered for consolidation,
three were unreachable,
and four voted (`minimax-m3`,
`gemma-4-26b-a4b-it`,
`deepseek-v4.1-flash`,
`inception/mercury-2.5`) against `quorumNeeded=4`.
The run lost the thinking voices:
Qwen from 02:37:33 UTC (budget) and,
in the letter block,
by the grace clock,
and Kimi from 02:37:26 UTC.
Any one of the four voters failing a naturalness review would have left it short of its quorum.

### The seating of glm-5.3 and Qwen

Two questions went to a read-only investigation:
why `glm-5.3` was seated at preparation though its every provider was out of budget,
and why `hf:Qwen/Qwen3.8-27B` stayed seatable,
and was re-seated as a checker under a hold,
when `hf:moonshotai/Kimi-K3` was withheld.
The seating is as documented:

- `doc/decision/translation-repair-short-bench-share.md` keeps a seat that no wet provider serves on the bench,
  to be lost at call time;
  the checker bench pads with unserved seats up to three.
- Kimi-K3 was withheld by the owner's OpenRouter cost rule (`OPENROUTER_WITHHELD`),
  which does not name Qwen;
  Qwen has no OpenRouter side,
  so with Synthetic and Hyper dry no provider serves it and it stays seated,
  unserved.
- One defect stands:
  `recordThrow` in `package/translation-repair/stage/src/seat-tally.ts` counts the router's own refusal (`NoProviderForModelError`) as an ask that threw,
  where the decision says a refusal is not an ask.
  So `glm-5.3` reads `asked=4 threw=4` though it was never sent a request,
  and Qwen's 137 throws are almost all such refusals.
  The fix is queued in keyless worktree w254;
  its account is in [the handover](../handover/translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20-zheermao20-to-zheermao21-reads-and-the-w253-to-w257-fix-rounds.md#next).

### The page

The lead's reader read `fixed/people/zheermao101/page.en.md` line by line beside the original (`page.md`),
the archive (`page.en.md`,
pinned at `a41fc607ea`),
the three photos (`photo1.webp` and `photo2.webp` are Arts's letter,
`photo3.webp` is the email exchange) and `info.yml`,
and compared it with zheermao19's page by `diff`.
The Markdown was not rendered.
Her life is in the past tense throughout,
apart from "But now,
everything can only be turned into a long sigh..."
(如今),
which is right.
The page has no em-dash or en-dash character (zheermao19's had three).

Three classes of departure are named,
and the ids D01 to D30 run in page order:

- `C`:
  traced to code.
- `W`:
  a model's wording choice with no code cause.
- `WS`:
  a model's wording choice where the single-round,
  whole-block selection of the archive block review (a design,
  not a defect) decided which partial fix shipped.

#### Which stage wrote what

Arts's letter (lines 52 to 87) and the email (lines 105 to 128) are archive blocks with no Chinese counterpart:
`alignment target-unclaimed (pair 2: 5 translation blocks no original claims, 3519 characters:
block/13, block/14, block/19, block/20, block/21)`.
They are not part of any of the 15 slices the lanes write.
The archive block review decides them before the lanes start,
in one round (`unclaimed archive blocks remain after the single correction round: 5`),
and whatever wins there is the page.
The other 15 slices went through the repair lane,
the translate lane,
the lane contest and consolidation.

#### The letter, and why it fell back

The archive review of the letter ran from 02:26:36 UTC to its select round at 02:34:19 UTC.
Of the four voices heard,
two answered `source-supported` with no change (`8cddb6a0`,
`38241cf7`) and two answered `revise`:
`37163f78` (the cat,
罪恶 as "unforgivable" and "guilty",
the remorse line;
2076 characters) and `693db0a7` (the MtF,
我们 and 我们的,
and 苟活于世;
2130 characters).
The round closed at 02:30:41 UTC (`4/5 heard, 245286ms total, 125266ms to quorum, 120020ms in grace`),
and the fifth voice,
`hf:Qwen/Qwen3.8-27B`,
was `abandoned 120000ms after quorum (cut-mid-reply after 0 answer chars and 41975 reasoning chars
(1150144 chars on the wire), first byte at 2888ms)`.
It was cut a second time in the select round (`after 174 answer chars and 51173 reasoning chars`).
The selection compared `2 admissible revisions in 3 candidates` and `candidate 2 from hf:moonshotai/Kimi-K3 won weight 3 (ballots: 4)`.
The shipped letter is `693db0a7`'s replacement exactly (a trailing newline differs),
and `37163f78`'s fixes are not in it.
Neither revision touched the QQ sentence,
the three "For example" lines or the closing.

The same round in zheermao19 went the other way:
`archive-block-review round: 4/5 heard, 464672ms total, 464672ms to quorum, 0ms in grace` (23:16:17 UTC),
`comparing 3 admissible revisions in 4 candidates`,
and `candidate 3 from hf:Qwen/Qwen3.8-27B won weight 3 (ballots: 5)`.
Its shipped letter equals reply `4dc0e2d6`,
apart from curly against straight apostrophes,
and that reply fixes every point D09 to D20 lists.
The payload files store no model name,
so the tie of a reply to a model is by the log line and the content match.
The rosters differ between the two runs (`gemma-4-26b-a4b-it` is on this run's bench and `glm-5.3` is not),
and which voice closed quorum in either run is not known.

So the letter regressed against zheermao19,
and the cause has two parts that are design and not fault:

- `runArchiveBlockReviewStage` in `package/translation-repair/archive/src/archive-block-review-stage.ts` hands the candidates to `decideBestCandidate`,
  which selects one whole candidate;
  a revision is a reviewer's complete rewrite of the block,
  and no step merges two revisions.
  The review runs once (`repairArchiveBlocks` in `pass-prepare.ts`).
  Only two narrow revisions reached it,
  and the two partial fixes do not overlap,
  so they could not both ship.
- The round waits a grace after quorum and then abandons the stragglers,
  120000ms here.
  The `WRITER GRACE built in` line gives writer rounds 180000ms;
  the archive review asks a reviewer for a whole rewrite but is not in that list.
  Qwen's reply was cut by the 120 s grace after quorum,
  with 0 answer characters after about 245 s.
  Whether 180000ms would have let it finish is not known.

A first draft of this read called it the most likely reading
that Qwen's cut reply would have been the winning whole-letter rewrite of zheermao19.
A read-only investigation contradicts that:
in zheermao19 the letter's Synthetic Qwen call also ended empty,
and the winning text came from Qwen's re-ask on Hyper,
which was dry for all of zheermao20.
Synthetic was still wet during the letter's review.
The two zheermao20 revisions changed disjoint lines,
and both  replies were dropped as unanchored for a cause not yet traced.
The investigation and the three gaps it found in the source are in
[the archive block review and the zheermao20 letter](translation-repair-archive-block-review-zheermao20-letter.md).

#### Better than zheermao19

- The quote that opens the page:
  有一天 and 你 are back ("If one day I meet you in heaven,"),
  这么多 is "so much",
  就好了呢 is "would be nice" (not the comparison "would be better"),
  the question keeps the name as its subject ("What flavour does Zheermao like?"),
  and 对不起了 is a plain "I’m sorry".
- 极度 is "extremely" in line 21,
  the unneeded "(romance visual novels)" gloss is gone from the game line,
  the full game title holds (the repair panel supported the subtitle 4 to 0 on web lookups),
  and 也 and 数码产品 are carried in the Zhihu sentence.
- Lines 37 to 39:
  孩子 is "child" (zheermao19:
  "girl"),
  其实 is "in fact",
  大家 is "everyone’s" and 依赖 is "depended on".
  邮箱 is "mailbox" in line 95.
- The email:
  尽力 is "tried my best",
  很抱歉 is "so sorry",
  你们 is "Thank you all",
  好嘛 is "okay?",
  动态 and 静态 are "dynamic" and "static",
  and 顺颂春祉 is "Wishing you spring blessings."

#### What departs from the Chinese

Each is a single wording in one sentence.
Code comes first.

- `C` D01,
  line 6:
  `desc: Treat you to some beverages.` is the archive's,
  and the Chinese `desc` is 如果有一天在天国遇到你，请你喝奶茶。 The English `desc` has lost the conditional and the milk tea,
  and it disagrees with the quote it comes from.
  The cause is policy:
  `FRONT MATTER entry=zheermao101 authority=archive: the archive translated it, so it ships as it
  stands and no lane writes it`,
  from `package/translation-repair/corpus-pass/src/corpus-run/pass-prepare.ts:190`,
  under `doc/decision/translation-repair-front-matter-guard.md`.
  zheermao19's page has the same `desc`.
- `C` D26,
  line 130:
  the note `{/* 可以避开黑暗的锋芒，到那些有光的地方躲一躲。 这句不知道如何翻译 */}` stands as a paragraph of its own after the email's blockquote,
  where the archive had it as a quoted line inside the email.
  `withBlockNotes` in `package/translation-repair/page-check/src/corpus-run/archive-block-notes.ts` puts a kept note on a line of its own after the replacement (`replaced with 1 note between translators kept`).
  The note still says the sentence could not be translated though it now is;
  the owner's rule of 2026-10-20 keeps notes as written.
  zheermao19's page has the same placement and text.

The wording choices in the lanes' slices:

- `W` D02,
  line 16:
  "I can’t wait that long."
  for 我等不到了;
  "that long" is an addition,
  unchanged since the archive.
- `W` D03,
  line 21:
  小 (little) is lost again;
  zheermao19 had "a little trans girl".
  The repair critics' two style claims against "extremely lacking in love" were rejected 0 to 4,
  and in the translate lane four translators returned the archive's sentence (`every proposal was the incumbent ... shipping it unjudged`),
  so no candidate carried 小.
  The translators see the page as it stands,
  so a copy is what the design invites;
  no code fault was found.
- `W` D04,
  line 29:
  喜欢 is "loved" and 遭遇 (what she met with,
  often hardship) is the mild "experiences";
  both are the archive's.
- `W` D05,
  line 31:
  热情地给出了许多回答 became "shared many enthusiastic answers",
  which moves the enthusiasm from her manner to the answers.
  The `deepseek-v4.1-flash` ballot said exactly this (`shifting the enthusiasm from her manner of answering to the answers themselves`) and lost 3 to 1.
  "on related topics on Zhihu" doubles "on".
- `W` D06,
  line 33:
  也 is dropped from the library sentence and 看看 is "read";
  the loss moved from zheermao19's sentence to the next.
  No critic filed it.
- `W` D07,
  lines 37 to 39:
  极度 is "deeply",
  a notch softer than "extremely".
- `W` D08,
  line 43:
  "Arts’s letters" is plural for 信件,
  which leaves number open,
  while the page also says "a caring letter" and the label says "Arts’s letter".
  It came from a critic claim the panel left as `needs-human` 2 to 2,
  then from the consolidation slate and its gate (2 for `consolidated`,
  1 `standing`,
  1 `neither`).
  zheermao19 had "Arts’s letter" throughout.
- `WS` D09,
  lines 54 to 55:
  the archive's QQ sentence is back with four departures.
  "in your forum profile" and "instant messaging" are additions;
  禁止通过QQ号查找 is misrendered as "disabled initiating conversations using QQ number";
  我搜索之后发现 and 因而我无法加你好友 (so I could not add you as a friend) are gone,
  so "Therefore" points at a reason that is not on the page;
  and 真的很抱歉 is "I’m sorry!".
  zheermao19 had all of it right.
- `WS` D10,
  lines 57 to 59:
  猫猫 is "cat-like person",
  three times,
  where the writer says she is a cat.
  zheermao19 had "a cat too".
- `WS` D11:
  罪恶 is "unforgivable" in line 58 and "guilty" in line 60,
  one word as two.
  zheermao19 had "evil".
- `WS` D12,
  lines 60 to 62:
  更不会 (much less) is flattened away,
  只会 (do nothing but) is "keep",
  and 无尽的痛苦 is "suffer more and more";
  "truly guilty won’t feel guilty" is a thin rendering of 罪恶的人，不会自责.
- `WS` D13,
  line 65:
  不知道 is "cannot comprehend",
  一定 (must be) is lost,
  and 善良美好 is only "good".
  zheermao19 had "a kind and beautiful girl" and "must be".
- `WS` D14,
  line 67:
  "it’s not easy to live" for 人生将会是怎样的 is the archive's invention;
  zheermao19 had "what life is like".
- `WS` D15,
  line 68:
  在遭遇失败时 is "when in trouble",
  愿意 is gone,
  and 也 and 迷茫 are not carried.
- `WS` D16,
  lines 70 to 75:
  并不 is "doesn’t really",
  所以 is gone,
  the first 也 is gone,
  就只有痛苦 loses "only",
  劝说 (urge) is "flatly tell",
  and 但 is gone.
  zheermao19 had "urge",
  which is exact.
- `WS` D17,
  line 76:
  "I know that" is an addition;
  the photo has no 我知道.
  The shipped half of `693db0a7` is right here:
  "as us" and "for our future" are 和我们 and 为了我们的未来,
  which the archive had wrong.
- `WS` D18,
  lines 77 to 79:
  three "For example" lines where the photo has two 比如,
  with the second holding two groups;
  因种种原因 and 比比皆是 are gone and 沉迷于自虐 is the milder "have issues with self-harm".
  The quoted structure (`> > You are not alone.` then a line starting `> For example`) is the archive's and zheermao19's;
  in the photo the sentence runs on in one paragraph,
  and the Markdown was not rendered.
- `WS` D19,
  lines 81 to 82:
  一定 is dropped twice,
  世界上 and 很多 once each.
  "like me,
  Arts" for 比如Arts is a fair reading.
- `WS` D20,
  line 84:
  最后 (Lastly) is lost again,
  and "If you like,
  I’d like to hug you" is a conditional the Chinese does not have,
  saying the wish twice.
- `W` D21,
  line 97:
  连夜 is "overnight",
  where zheermao19 had "that same night".
  The photo stamps the mail 11:55 PM and the reply 12:10 AM
  (the zone it was observed in was never established),
  so "overnight" is true but can read as "took all night".
  The gate shipped the consolidated candidate 3 to 0 with 1 `neither`.
- `W` D22,
  line 110:
  成功 (successfully) is dropped from "If I die,
  please put my entry up,
  okay?";
  zheermao19 dropped 死去 and kept "succeed".
  The reader-protection rule at `package/translation-repair/house-rule/src/house-policy.ts:58` reaches the method of a death,
  and 死去 names none,
  so it does not require this loss;
  two review voices read it in opposite ways.
  The email block's winner was `candidate 1 from hf:Qwen/Qwen3.8-27B` (weight 5.5,
  6 ballots).
- `W` D23,
  line 114:
  the salutation line 玖月折耳猫： is lost,
  leaving "Good evening,";
  zheermao19 had "Good evening,
  Zheermao,".
  A reviewer (`76807c82`) had flagged it and the winning candidate did not restore it.
- `W` D24,
  line 114:
  夜安 as "Good evening" is a register choice for a night greeting stamped 12:10 AM
  (the zone it was observed in was never established),
  unchanged since the archive.
- `W` D25,
  line 125:
  爱你 is "I love you" where zheermao19 had "Love you.";
  the register is firmer and 呀 is not carried.
  "I want to hug you" in line 117 is the same kind of mild register shift.
- `W` D27,
  lines 132 to 137:
  联系上了她 is "reached out to her",
  which says the members made the first move more than 联系上 does,
  and "with them" is a small addition.
  The slice went through the translate lane and consolidation (`incumbent won weight 2`),
  and `translate-archive-disputed (slice 11)` means the repair lane's text stood in for the archive's.
- `W` D28,
  line 139:
  the second 仍然 is not carried ("ultimately"),
  unchanged since the archive.
  The lane contest settled on `neither` (2 `neither`,
  1 `repair`,
  1 `translate`;
  mercury alone named the missing "still"),
  and the polish "in the end she still chose to leave" lost 3 to 1.
- `W` D29,
  line 141:
  离世 is "departure" though line 92 says "death" for the friend,
  and 悼文 (a memorial piece) is "epitaph" (an inscription);
  both are the archive's.
  The slice 13 judges kept the incumbent on a declared-names reading;
  `package/translation-repair/house-rule/src/candidate-judge-rules.ts` says declared names settle how to spell a person or a place,
  which 离世 is not.
  The sheet text is not in the run directory,
  so whether the sheet licenses it is open,
  and nothing in the run backs or refutes "epitaph" (`REFERENCE 1 https://t.me/Lianyi520/565: success, 25 chars, cached`,
  `ATTESTED heard=6 answered=0 verified=0 needed=3 details=0`).
- `W` D30,
  line 145:
  the passive "be turned into" for 化作 stands for the second run in a row;
  the translate lane chose the active at weight 4,
  the lane contest settled on `neither`,
  and the gates kept the archive against "dissolve into" and "fade into".
  也 is not carried in 一切也只能.

The labels at lines 50 and 103 were rewritten by the archive review from "English translation of the preceding letter:"
and "English translation of the preceding email conversation:"
under the house rule at `package/translation-repair/house-rule/src/house-policy.ts:111` (a passage is referred to by naming it,
never by its position).
That is apparatus with no Chinese counterpart,
and not a departure of meaning.

Counts by class,
over the 30 departures:
`C` 2 (D01,
D26),
`WS` 12 (D09 to D20) and `W` 16 (D02 to D08,
D21 to D25,
D27 to D30).
Against the archive,
20 of the 30 repeat its wording,
8 are wording new to this run (D05,
D07,
D08,
D17,
D21,
D22,
D25,
D27) and 2 are the code effects.
Of the 20 that repeat the archive,
zheermao19 had fixed,
wholly or in part,
D03,
D06,
D09 to D16,
D18 to D20 and D23.

The most serious,
by what a reader of the letter is told,
are D09 ("Therefore,
I chose to contact you in this way" points at a reason that is not on the page),
D10 and D11 ("How can a cat-like person be unforgivable?",
where the writer says she is a cat and that a cat is not 罪恶) and D13 to D15 ("I believe you’re a good girl at heart" for 一定是一个善良美好的女孩子,
and "it’s not easy to live" for 人生将会是怎样的),
each of which softens or invents what the writer says to a girl in danger.

#### Two warns that look like a code fault and are not

- Slice 7 is the one picture tag,
  `<PhotoScroll photos={[ ... ]} />`.
  `hf:zai-org/GLM-5.3-Flash`'s candidate was refused (`MdxParseError: MDX body refused to parse at 4:1`) and GLM answered `as-intended`,
  claiming its output `reproduces that tag byte-for-byte`.
  Its reply file reads `<PhotoScroll photos={` with no `[` after the brace,
  so the candidate dropped a bracket,
  GLM's defence is false,
  and the refusal was right.
  Three other translators returned the tag unchanged,
  and the page ships it as the source has it (`delivery: incumbent-retained`).
  The three `refused by the page grammar` warns are notes on that refused candidate,
  not on the page.
- `archive-revision-refused (gemma-4-26b-a4b-it): the block carries 3 quoted passages and the revision
  carries 1; a revision keeps every quoted passage the block carries ...` (02:22:53 UTC) is a guard working:
  gemma's email revision kept one quoted passage where the block has three,
  and was refused.
  No page text came from it.

#### Class verdict

No class.
Every departure is a wording in one sentence;
none touches who died,
when,
how she is named,
or what the writers tell her.
The log shows none of the five markers that could name a class,
and no `[error]`.
The 12 `WS` departures share one cause,
the whole-block,
single-round selection of the archive block review,
which is a design with a cited source,
so they are recorded as a condition of this run and not as a class with a fix.

### Against zheermao19

The `TALLY` beside zheermao19's (same entry,
tip `5b5dca49a9`,
35.6 min):
`repairIssues` 16 against 24;
`repairAccepted` 13 on both;
`repairResolved` 13 against 12;
`repairFindings` 220 against 88 (this read did not trace why the count rose);
`repairChanged` 7 on both;
`translateChanged` 10 against 11;
`documentsDiffer` 10 against 9;
`pageChanged` 8 against 9;
`alignmentFindings` 13 against 14;
`selection=contested` on both;
`ms` 1977563 against 2133786.

Improved:
the items of the section on what is better than zheermao19.

Worse:
the letter (D09 to D20),
where zheermao19 had each point right or closer,
its flattened 更不会 excepted,
and this page gained nothing over it;
and 小 (D03),
也 in the library sentence (D06),
the plural "letters" (D08),
"overnight" (D21),
the lost salutation (D23),
the firmer "I love you" (D25) and the dropped 成功 (D22).

Stayed:
the account of her death (lines 89 to 92),
the despite line,
the epitaph line,
the departure line,
the sigh line,
the credit,
the note after the blockquote with its stale text,
the front matter `desc`,
and "that long".

By slice,
in short:
slices 0,
3,
5 and the email are better;
slices 1,
4,
6 and 9 are mixed;
the letter is clearly worse;
the rest are unchanged.
One sample each cannot separate a regression from run-to-run spread,
and this read did not measure that spread.

Of zheermao19's observations,
the milk-tea quote,
the game title,
the `desc` and the stale note hold;
the letter's QQ search,
"a cat" and "evil" (only "us",
"our" and "trans woman" survive),
the three em-dashes,
and the `closed on a short bench` lines with 12 fallback decisions (none;
6) no longer do.

### What follows

- The read finds no class traced to code.
  The letter's regression against zheermao19 is recorded with its cause:
  the archive block review picks one whole revision,
  only two narrow revisions reached it,
  and Qwen's reply was cut by the 120 s grace after quorum.
  Fixes for the three gaps the investigation found are queued
  in [the handover](../handover/translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20-zheermao20-to-zheermao21-reads-and-the-w253-to-w257-fix-rounds.md#next):
  a composite candidate of disjoint revisions,
  the writer window for the review round,
  the stage's findings kept in the artifact,
  and a trace of why both `source-supported` replies were dropped as unanchored.
- The seating investigation's one defect is queued in w254,
  and the two defects of the NeuralWatt quota session and the rotation sweep in w253;
  both are in [the handover](../handover/translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20-zheermao20-to-zheermao21-reads-and-the-w253-to-w257-fix-rounds.md#next).
- The next run on this build meets the same bench:
  Synthetic 0%,
  Hyper 0,
  OpenRouter 14.50 USD,
  Bedrock 5.02 USD with an expired token,
  and NeuralWatt 490.017 Wh at the end of this run.
- Not done:
  the declared-names sheet and the prompts were not read (the run directory holds replies,
  not prompts),
  so D29 stays open;
  no spread measurement was run;
  the page was not rendered.

## CuspariaKLSY20 read, 2026-10-21, 03:19 UTC: SETTLED in 11.5 min of pass; no class; two departures traced to code, both by design; a better page than CuspariaKLSY19

`CuspariaKLSY20` (entry `CuspariaKLSY`) ran on the same tip `31ce5369e1…` and pipeline `sha256-tree-v1:e9eda9fe032032f4896f29b0c82099ae5b04084f209ce37144bb83d78eadcfcd`,
and its `START` line carried both.
The brief gave the start as 02:52:16 UTC on 2026-10-21;
the log's first timestamped line is 02:52:42 UTC and its last is 03:04:13 UTC.
The corpus input is the pin `a41fc607ea`.
The pinned directory holds no `.git`,
so the read checked the source and archive by size and bytes against the artifact instead.
The `DONE` line reads `elapsed=690419ms`,
which is 11.5 min (CuspariaKLSY19:
16.0 min;
CuspariaKLSY18:
49.4 min):
`TALLY CuspariaKLSY status=SETTLED slices=4 repairStatus=repaired repairIssues=21 repairAccepted=21
repairResolved=20 repairFindings=66 repairChanged=4 translateStatus=complete translateChanged=4
documentsDiffer=4 pageChanged=4 pageSilent=0 alignmentFindings=1 selection=contested`.
The publish wrote 4 slices into a page of 1929 characters (CuspariaKLSY19:
1923;
CuspariaKLSY18:
1898).

The log holds 134 `[warn]` lines (CuspariaKLSY19:
160;
CuspariaKLSY18:
154),
no `[error]` line,
no `DEFECTS` line,
and none of `REATTEMPT`,
`STALLED`,
`mdx-downgraded`,
`StreamOverrunError` or `closed on a short bench`.
Of the 134,
57 carry `seat unreachable`,
2 carry `HTTP 401`,
2 carry `HTTP 429` (`minimax-m3` on OpenRouter,
retried and answered),
7 carry `schema-mismatch` and 4 carry `voice lost`.
Three decision lines read `keeping the fallback`.
The full read is kept at `/var/home/user/temp/agent/x15-fix/reads/CuspariaKLSY20-read.md`;
this section keeps every departure it found.

The repair lane ran 02:52:54 UTC to 02:58:19 UTC,
the translate lane to 03:00:18 UTC,
and consolidation to 03:04:12 UTC.
The repair lane's slice 3 waited on one seat:
`hf:zai-org/GLM-5.3-Flash` as editor was `abandoned 180000ms after quorum`,
which is the writer grace the `WRITER GRACE` line announces.

### Seats and meters

Usable answers of asked:
`deepseek-v4.1-flash` 63 of 63,
`minimax-m3` 53 of 54,
`inception/mercury-2.5` 61 of 62,
`gemma-4-26b-a4b-it` 56 of 56,
`hf:zai-org/GLM-5.3-Flash` 16 of 16,
`typesafe/jev-1.13` 15 of 15,
`xiaomi/mimo-v2.6-flash` 10 of 10,
`glm-5.3` 0 of 4,
`hf:Qwen/Qwen3.8-27B` 0 of 5 and `google.gemma-4-e2b` 0 of 5.
The run ended with `SEATS DARK: 3 of 10 seats asked produced nothing usable this run`.
`hf:moonshotai/Kimi-K3` is in the `withheld=` list of all six `JUDGE SEATS` lines and was never asked.

- `glm-5.3` and `hf:Qwen/Qwen3.8-27B` threw every ask with `every provider serving this model is out of budget, seat unreachable`:
  Synthetic (`syntheticWeekly=0%`) and Hyper (`hyperBalance=0`) read dry on the first `METERS` line and on every later one,
  so these two seats had no provider from the first ask.
- `google.gemma-4-e2b` met `SEAT CREDENTIAL REFUSED` with HTTP 401 ("The security token included in the request has expired.")
  at 02:52:47 UTC,
  after which Bedrock was held out for every model.
- What differs from CuspariaKLSY18 and CuspariaKLSY19:
  `gemma-4-26b-a4b-it` answered all 56 asks,
  where it threw all 4 in both.
  It was served by OpenRouter (`served by "DeepInfra"`),
  so the held Bedrock credential no longer takes the seat down.
  By commit subject the range holds `786013e4b7` and `e9d0c5cd6f`;
  the read did not read those diffs.
  The new voice matters to the page:
  it is the ballot that filed the nickname gloss as an addition.
- OpenRouter went from 14.49 USD to 14.36 USD (its `SPEND` lines sum to 0.1415 USD) and Bedrock stayed at 5.02 USD.
- NeuralWatt went from 489.965 Wh to 468.773 Wh,
  which is 21.192 Wh,
  and the 80 `settleNeuralWattCall` lines sum to the same 21.192 Wh measured against 10.722 Wh estimated (`deepseek-v4.1-flash` 63 calls,
  12.381 Wh against 4.723 Wh;
  `glm-5.3-flash` 17 calls,
  8.811 Wh against 5.999 Wh).
  The closing line reads `NeuralWatt plan energy: no plan energy was spent between the start and the end of the run`.
  The bucket means,
  and what two runs can and cannot say about the estimate,
  are in [the handover](../handover/translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20-zheermao20-to-zheermao21-reads-and-the-w253-to-w257-fix-rounds.md#next).

`DESTINATIONS CuspariaKLSY source=0 page=0 dropped=0` is the same as in CuspariaKLSY18 and CuspariaKLSY19.
The entry holds no link,
image or reference definition in the source,
the archive or the shipped page (a count for `https?://`,
`](`,
`![` and reference definitions finds none in any of the three),
so `dropped=0` is true but could not have been otherwise here.

### The page

The lead's reader read `fixed/people/CuspariaKLSY/page.en.md` line by line beside the source and the archive,
with the artifact's per-slice records beside it.
The entry holds one photo (`profile.jpg`,
the profile picture,
not opened) and no photo of text,
so no page line rests on a photo.
The Markdown was not rendered.
Her life is in the past tense throughout;
the only present-tense verb in the body is `means` in the nickname gloss.
The page has no footnote,
no note between translators and no link,
no em-dash or en-dash character,
and no straight apostrophe.

#### Against CuspariaKLSY19

The page is better than CuspariaKLSY19's in three slices and the same in one,
with one new regression.

- Slice 0:
  the same,
  with one regression and two gains.
  The regression is "loved" for 喜欢 ("liked" in CuspariaKLSY19).
  The gains are 很深 and 她和向日葵 carried in "she and the sunflowers shared a deep,
  fated bond",
  and 很 carried in "deeply devoted".
  "devoted" for 执着 is a side step from "persistent".
- Slice 1:
  better.
  义无反顾 and 自己死 are carried literally ("her death",
  "without looking back"),
  the added "though" is gone from the line on God,
  and the gloss is the archive's own line.
- Slice 2:
  better.
  "Problems" for 问题 (CuspariaKLSY19:
  "Pressure"),
  二次元 carried,
  and 一直 exact;
  lost are 加上 ("as well") and the waiting of 观望.
- Slice 3:
  better.
  "where doctors fought to save her" (doctors are not in the Chinese) is "for emergency treatment",
  and "they learned",
  with a "they" that has no antecedent,
  is "she had already been confirmed brain dead".
  New is "trying to console" (CuspariaKLSY19:
  "consoling").

Against CuspariaKLSY18 the page is better in slices 2 and 3;
"loved" is new to this run.

#### Departures traced to code

Both have a cited design source,
so this read finds no class.

- The gloss sentence in line 38,
  "“Ling Shui Yu Yu Zi” means fish in clear water.",
  has no counterpart in the Chinese.
  `restoreNameGlossLines` in `package/translation-repair/page-check/src/corpus-run/name-gloss-restore.ts` put the archive's line back in the page assembly (`name-gloss-restored (slice 1: "“Ling Shui Yu Yu Zi” means fish in clear water.")`,
  `page assembly rewrote 1 slice and withdrew 0`).
  Three stages had removed or voted against it:
  the repair panel accepted the claim that it is an addition 2 to 1 and the editor's `editorAfter` was empty;
  the translate lane's text stood in for the archive (`translate-archive-disputed (slice 1)`);
  and all three ballots of the consolidation slate chose the gloss-less candidate.
  The last step then put it back.
  The line is page apparatus by design:
  `package/translation-repair/shared-contract/src/page-apparatus-clause.ts` names this very line and lists "a gloss of a name" in `APPARATUS_KINDS`.
  The clause says "its tense follows the passage",
  and the restore does not touch the archive's present-tense "means";
  CuspariaKLSY18 ("it means") and CuspariaKLSY19 ("The name means") had the same tense,
  and a name keeps its meaning,
  so it is recorded and not counted.
  In CuspariaKLSY18 and CuspariaKLSY19 the models themselves carried the gloss,
  so the restore never fired (`name-gloss-restored` appears once in the three logs,
  in this one).
  Whether the new `gemma-4-26b-a4b-it` voice changed the panel was not tested.
- The front matter `desc`,
  "Be more similar to “herself”.",
  against the source's 「泠水鱼鱼子」是在喂鱼时起的哦。 It is the front matter authority (`FRONT MATTER ... authority=archive`) and `doc/decision/translation-repair-front-matter-guard.md`,
  not counted,
  as in the CuspariaKLSY18 read.

Also traced to code and correct,
not a departure from the source:
the archive's "She swallowed lots of her meds that night" and "by August 1st" are gone (`package/translation-repair/house-rule/src/house-policy.ts`,
line 87,
which says a survived attempt is told "never by what means";
repair issues 17 and 18).

#### Wording choices with no code cause

Eleven,
in page order,
each checked against the lane records:

- Line 11:
  "She loved sunflowers."
  for 喜欢;
  the strength is the archive's "loves",
  carried into the past by the translate lane,
  and line 15 renders the same verb as "liked".
- Line 13:
  "She was deeply devoted to her art."
  for 她对艺术很执着;
  执着 is tenacity or fixation,
  "devoted" is dedication.
  The polish's choice,
  taken on 3 of 4 gate ballots;
  `minimax-m3` voted for the base saying it rewords the Chinese "beyond what the Chinese states".
- Line 17:
  "She often told me she couldn’t get enough to eat."
  for 吃不上饭,
  which turns a want of food into a quantity.
  The archive's "cannot have a good meal" was weaker still,
  so this is a gain over the archive and level with CuspariaKLSY19.
- Line 19:
  "She felt she didn’t have many friends."
  for 很少;
  the intensifier is flattened.
- Line 29:
  "make other people’s lives better" for 让其他人好 ("lives" added) and "She believed that" for 觉得.
- Line 42:
  感觉 given a subject ("She felt"),
  and one sentence split into two.
- Line 44:
  加上 not rendered,
  观望 as "watching" only,
  and "the idea" for 萌生想法.
- Line 46:
  "wanted" for 希望,
  and a singular "character" for 笔下的角色.
- Line 48:
  苦于 not rendered,
  and "the means" for 能力.
- Line 50:
  "trying to console" for 劝慰.
- Line 50:
  得知 not rendered ("When word came again,
  she had already been confirmed brain dead").
  This is new in kind:
  CuspariaKLSY19 rendered it as "they learned" with a "they" that has no antecedent.

The three that shift meaning most are the want of food (line 17),
执着 (line 13) and "loved" (line 11).
Lines 44 and 48 carry CuspariaKLSY19's wording through unchanged.

#### Behaviours of the run that are not departures on the page

- The one `repairResolved` miss (20 of 21) is issue 0,
  "Sunflowers" capitalized mid-sentence in the repair lane's text;
  the shipped page has lowercase,
  from the translate lane.
- Slice 2:
  `minimax-m3` answered `as-intended` to a structural finding with the claim that the original "is four prose sentences with no list formatting" (the source is a four-item ordered list);
  the slate withheld its proposals as failing the structural guard and the translate lane's text stood (`slate-declined-standing`).
  Slice 3 settled on its base (`slate-unjudged-standing`).

### What follows

- No class:
  the two departures traced to code are by design,
  and the eleven wording choices share no cause.
  The page is better than CuspariaKLSY19's and level with the archive where the archive was right.
- The chain of the gloss is the point for the lead:
  a critic panel and the translate judges decide the line is an addition,
  and the last step puts it back.
  It is recorded and not counted,
  since the apparatus clause names the line.
- Not done:
  the pinned commit was not read from disk;
  the diffs of `786013e4b7` and `e9d0c5cd6f` were not read,
  so the account of why `gemma-4-26b-a4b-it` answered rests on the stream line and the commit subjects;
  `prompt-payloads` and the live sheets were not opened,
  so whether the apparatus clause was on the critic panel's sheet is unknown;
  the Markdown was not rendered;
  and the CuspariaKLSY19 comparison uses that run's shipped page and log counts only.
- Both reads ran on one build and meet the same bench.
  The readiness rotation does not go on to `XingZ60` yet:
  the fixes in w253 and w254 change the build when they land,
  and the rotation then restarts at `zheermao101` (see [the readiness signal](translation-repair-readiness-signal.md)).
