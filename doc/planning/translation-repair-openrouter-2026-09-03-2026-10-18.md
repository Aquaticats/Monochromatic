# OpenRouter and the corpus passes: 2026-10-18

What was launched,
read and decided on 2026-10-18.
Entries:
TianqiChen66626.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## TianqiChen66626 read, 2026-10-18: SETTLED, no run-off, two Bedrock seats dead, the Synthetic meter unreadable

TianqiChen66626 launched on 2026-10-18 at 20:58 UTC at `7c21a56fc2`
(the unit's journal gives the start as 20:58:38 UTC)
through the runbook's unit form into a fresh runs directory,
with `--only TianqiChen666`,
and settled in 2 h 53 min 38 s of pass,
the unit ending at 23:52:32 UTC
(`TALLY TianqiChen666 status=SETTLED slices=24 ... selection=contested`).
The corpus head it read,
`a41fc607ea`,
is the `corpusSha` of the artifact.
The pass wrote only into the runs directory's `fixed/` tree,
and the corpus checkout is untouched.
`publish` wrote 24 slices into a page of 8,525 characters.
`verify-published` read the page back:
1 of 1 page carries every wording its artifact promised,
at the length it implies
(`wordings=24 silent=0 chars=8525=expected missing=0 consistency-groups=2 read-at-generation=18`).
`DESTINATIONS` dropped none,
no `DEFECTS` line printed,
the log holds no `[error]` line,
and it holds 687 `warn` lines,
where TianqiChen66624 held 626 and TianqiChen66625 held 611.
The unit has no failure line in the journal,
and the log ends on `DONE processed=1 of pending=1; artifacts=1/1 declined=0`,
so the clean exit is inferred from the absent failure line and not read as a code.

Three things are said plainly here,
because the verdict does not show them:

- No consistency run-off happened,
  so the finalist extension the owner kept on 2026-10-18 was not exercised by this page.
- The two Bedrock Gemma seats were dead for the whole run,
  because their security token had expired,
  and the meters did not say so.
- The Synthetic meter was unreadable in every one of its 126 readings.

The fix round `w191` for three pipeline defects of this reading landed at `e9d720b816` after gate g106.
A seat whose credential gets HTTP 401 now says so once,
as `SEAT CREDENTIAL REFUSED`.
The JUDGE SEATS line prints `unreadable-treated-as-spendable` for a meter that said nothing,
not `wet`.
The SEAT line counts replies cut at the straggler deadline in an `abandoned` column;
before,
the census counted them in no column,
not as usable.
Quorum over the full bench of 8 follows the recorded reachable-quorum rule and is unchanged.

### Seats

Both Bedrock Gemma seats,
`gemma-4-26b-a4b-it` and `google.gemma-4-e2b`,
answered every ask with HTTP 401 and the text
"The security token included in the request has expired."
(first at log lines 211 to 215,
last at 286 to 298,
five asks each).
After five throws each seat was withheld
(from log line 605),
and the stages printed 283 "not asking ... withheld for throwing on every ask" lines
and 131 "no provider can take ... seat unreachable" lines.
The closing census reads `SEATS DARK: 2 of 11 seats ... threw 5` (log line 9367).
Roughly 434 of the 687 `warn` lines (63%) come from these two seats.

The meters never showed it.
`bedrock=wet` stands in all 126 `METERS` readings with the spend frozen at 5.02 USD,
and the first `JUDGE SEATS` line reads `bedrock=wet ... roster=9 withheld=none` (log line 206).
The refusal lines name the error class `SyntheticHttpError` (renamed `ProviderHttpError` on 2026-10-19)
for a Bedrock model.
The class is shared:
the Synthetic and OpenRouter clients throw it,
and `package/translation-repair/provider/src/bedrock-client.ts` documents it as thrown at line 255.
The name misleads a reader and is not wrong.
The absolute-naturalness reviews print `quorumNeeded=4 quorumOver=8 unreachable=2` or `3`
(log lines 8580,
9206 and 9213),
so their quorum is a majority of 8 although 2 or 3 of the 8 cannot answer.
The parts for 2026-10-12 and 2026-10-15 record the same two seats throwing on every ask.

The Synthetic meter read `unreadable` in 126 of 126 readings
(`Synthetic /quotas body violated expectations: rollingFiveHourLimit is not an object`,
log line 204).
The call is treated as spendable,
so 651 subscription calls (Qwen3.8-27B 298,
Kimi-K3 353) ran with no reading of the allowance,
and `JUDGE SEATS` prints `synthetic=wet` beside `METERS` `synthetic=unreadable`
(log lines 206 and 205).
`meter-report` calls it "spendable on NO MEASURABLE FRACTION".

Replies that did not finish or did not fit their shape:

- 58 Qwen3.8-27B replies were cut at the straggler deadline
  ("cut-mid-reply after 0 answer chars and 30k to 63k reasoning chars"),
  in 11 stages:
  select 14,
  absolute-naturalness-review 10,
  panel 8,
  lane-contest 7,
  critic 6,
  translate 3,
  produceConsolidations 3,
  consolidate-gate 3,
  consolidation-polish-gate 2,
  introduced-defect-probe 1,
  and checker 1.
  Two deepseek-v4.1-flash voices and one Kimi-K3 voice were abandoned the same way,
  61 in all.
  The closing census still reads `SEAT hf:Qwen/Qwen3.8-27B asked=299 usable=299 unusable=0 threw=0` (log line 9363),
  so a half-dark seat cannot show in `SEATS DARK`.
  Whether the census counts abandoned replies elsewhere was not traced.
- Mercury 2.5 lost 4 voices to schema mismatches,
  had 4 panel ballots logged as irregular (`group-index-out-of-range`),
  and the census reads `unusable=4` (log line 9361).
- Three rounds ended short of quorum
  (`block-pairing round: 4/6 heard ... no quorum (4 of 5 needed)` twice,
  and `critic round: 3/5 heard ... no quorum (3 of 4 needed)`),
  each retried and recovered;
  no stage shipped on fewer voices than its quorum.

One log line reads against the artifact.
For slice 14 of the consolidation,
log line 8573 says `consolidate gate: 5/6 usable, settled on neither, ships standing`,
and log line 8574 (a `warn`) says the proposal the slate chose ships
because a majority of the contest ballots called the standing text flawed.
The artifact has `terminal: "consolidated"` and the page reads "inside the headpiece" (page line 157),
which is the proposal.
The gate's own verdict is the first line and the rule that shipped the proposal is the second,
so the first read alone misleads.

### Time and spend

10,418,202 ms against TianqiChen66624's 10,629,332 ms and TianqiChen66625's 11,874,915 ms,
the shortest of the three;
the providers' shared capacity makes this a comparison of one reading each and not a matched one.
Like for like across TianqiChen66624,
TianqiChen66625 and TianqiChen66626 (`run-timing-report`,
`spend-report`,
`ledger-report`):

- rounds 513,
  540 and 511;
- rounds without quorum 10,
  3 and 3;
- waiting after quorum 57.3%,
  57.2% and 61.0% of round time;
- voices never heard 220,
  230 and 206;
- calls in flight on average 5.79,
  5.64 and 5.86;
- credits priced 148.11,
  153.33 and 149.51;
- OpenRouter 0.2681,
  0.2886 and 0.2947 USD;
- contests read 135,
  148 and 142;
- ballots that named nothing 28,
  23 and 14.

`run-timing-report` for this pass reads 511 rounds in 10.17 h,
with 6.21 h of waiting after quorum,
206 voices never heard,
calls in flight averaging 5.86 with a peak of 20,
and 16.97 h of calls across 2.89 h of the run.
`slice-cost-report` was not run on this pass,
so no time by lane is recorded.

`spend-report` priced 149.51 credits at the rates read on 2026-10-11
(glm-5.3 51.09,
minimax-m3 48.92,
deepseek-v4.1-flash 46.18,
glm-5.3-flash 3.12,
qwen3.8-27b 0.19)
and 0.2947 USD on OpenRouter
(mercury-2.5 0.1576,
mimo-v2.6-flash 0.0544,
jev-1.13 0.0339,
minimax-m3 0.0276,
deepseek-v4.1-flash 0.0212),
and reports 2 calls as "RECKONED,
NOT REPORTED" (the OpenRouter deepseek lines).
The providers' own balances,
first reading to last
(126 readings from 20:58:54 UTC to 23:50:50 UTC;
`meter-report`):
Hyper 250 to 136,
OpenRouter 22.41 to 22.12 USD,
Bedrock unchanged at 5.02 USD,
Synthetic unreadable throughout.
The `cost=` fields of the `SPEND` lines sum to 0.2947 USD,
which matches the OpenRouter balance drop of 0.29.
Hyper began at 250 where TianqiChen66625 ended at 27,
so it was topped up between the passes.
`ledger-report` read 142 contests;
14 ballots named nothing and 3 named a candidate their slate did not have.
Shares of disinterested ballots:
GLM-5.3-Flash 38.8% (49 chosen of 129),
Qwen3.8-27B 37.3%,
deepseek-v4.1-flash 36.3%,
Kimi-K3 28.8%,
glm-5.3 28.2%,
mercury-2.5 20.0%,
and minimax-m3 14.1%.

### Tense: her last message keeps her tense

Source lines 71 to 75 are the message she wrote before she died,
in her present.
Page lines 203 to 215 give it in her present,
as TianqiChen66625 did:
"I feel like I might not make it to that day"
and "There are still so many kids who need help".
The narration around it holds the past
(the "final message she left us" and "she was gone forever").
Her other quoted words keep her tense
(page lines 130 to 145,
154 to 157,
171 to 177 and 195 to 197).
TianqiChen66624 had moved the message into the past
(its lines 217 to 229),
so the quoted-words rule holds on this pass too.
The narration of her life is in the past throughout.
One passage is a judgement item,
page lines 225 to 227:
"the memes left behind still flicker ... a smile that seems familiar" is in the present.
Its subject is the memes,
which persist (仍然,
似曾见),
so it is defensible under the rule,
and its neighbours are past.

### The consistency pass carried both groups, with no run-off

The log prints rounds,
group totals and `settled groups=2 changed slices=4` (log line 9347),
and no ballots or labels;
the votes are read from the artifact's `consistency.groups`.
Consistency changed slices 4,
7,
9 and 15.

Group 1 of 2,
the refrain 所以她是个……的女孩吧 (source lines 17,
25,
33 and 37;
slices 1,
4,
7 and 9;
four places,
none unplaced and none unlocated):

- Before the pass the four places read "So she must have been a girl who loved life",
  "So,
  she must have been a girl of incredible perseverance",
  "So,
  she was a girl who could be trusted,
  wasn't she"
  and "So she was a gentle,
  sensitive girl,
  I suppose".
- The first ask had quorum 3 and 3 usable ballots of a bench of 4,
  because deepseek-v4.1-flash was abandoned 120,000 ms after quorum
  (log lines 9250 and 9253).
  minimax-m3 and glm-5.3 chose OCCURRENCE 1 and inception/mercury-2.5 chose OCCURRENCE 4,
  counts 2,
  0,
  0 and 1,
  a strict majority,
  and OCCURRENCE 1 won.
- The carries to the other three places each had quorum 3 and 4 usable ballots,
  and each was unanimous.
  Slice 4 took CANDIDATE 3,
  "So she must have been a girl who had incredible perseverance.",
  by Qwen3.8-27B,
  Kimi-K3,
  minimax-m3 and mercury-2.5.
  Slice 7 took CANDIDATE 1,
  "So she must have been a girl who could be trusted.",
  by Kimi-K3,
  minimax-m3,
  glm-5.3 and deepseek-v4.1-flash.
  Slice 9 took CANDIDATE 1,
  "So she must have been a girl who was gentle and sensitive.",
  by the same four,
  who all called CANDIDATE 2,
  a splice reading "a girl who gentle,
  sensitive.",
  ungrammatical.
- The result line reads `placed=4 unplaced=0 replaced=3 ... unlocated=0 chosen=1 kept-current=0 untouched=0`
  (log line 9345),
  and the group's findings are `stage-voice-lost (page-consistency deepseek-v4.1-flash)`
  and `stage-roster-incomplete (page-consistency 3/4)`.

Group 2 of 2,
the maximal repeat 她想以可爱的样子陪伴在更多人的身边，为更多人带来阳光。 (source lines 35 and 55;
slices 8 and 15):

- The first ask had quorum 3 and 4 of 4 usable ballots.
  Kimi-K3,
  glm-5.3 and deepseek-v4.1-flash chose OCCURRENCE 1 and minimax-m3 chose OCCURRENCE 2,
  counts 3 and 1,
  a strict majority.
- The carry for slice 15 took CANDIDATE 3 by Qwen3.8-27B,
  Kimi-K3 and minimax-m3,
  3 of 4:
  "She wanted to stay by more people's side as her cute self,
  bringing sunshine to more people."
  mercury-2.5 chose CANDIDATE 2,
  a lowercase splice fragment that would have opened the sentence without a subject.
- The result line reads `placed=2 unplaced=0 replaced=1 ... unlocated=0 chosen=1` (log line 9346).

There was no consistency run-off.
The first asks of both groups and all four carry choices had a strict majority,
the artifact has no `runOff` field on any group or carry,
and the reader's shape for both groups is `strict-majority` with `hasRunOff` false.
So the finalist extension (tied leaders,
or a lone leader without a majority,
run off with second-count labels) was not exercised on this page,
and the page gives no evidence on the run-off sheet's wording,
its rounds,
a second split or a replayed first-round reply.
Group 1 came near it:
it had exactly quorum,
and had the abandoned deepseek voice voted OCCURRENCE 4,
the count would have been 2 to 2 and the extension would have run.
That the only dissenting ballot (mercury-2.5) argued that OCCURRENCE 1 and 2 omit 吧,
which it then said OCCURRENCE 4 captures best,
and that its reasoning contradicted itself,
did not change the result;
a different lost voice would have.
The consolidation stage's translate-slate run-offs are a separate mechanism
(`translate-runoff (finalists 2 of 5)` and `2 of 3` in the slice findings;
log lines 6808 and 8434 for the tied-slate rule and 6867,
7584,
7957,
8103 and 8989 for `declined-indecision ... asked again over the 2 tied finalists`),
and their votes were not extracted.

After consistency,
the page assembly unwrapped the curly quotes that the consolidation had put inside the blockquotes of slices 16,
18 and 20,
to match the archive's bare blockquote style
(`blockquote-quotes-unwrapped`,
log lines 9348 to 9351).
The consistency stage still logs no ballots,
candidates or labels,
as the part for 2026-10-15 recorded for TianqiChen66625.

The reader check ran twice,
once on the `dist` built at pass launch and once on the `dist` of another worktree built at 23:59 UTC,
over the artifact:
both read it as accepted,
`kind: "version-2"`,
generation 18,
with a consistency section and 2 groups,
group 1 with quorum 3,
usable 3 and counts 2,
0,
0 and 1,
group 2 with quorum 3,
usable 4 and counts 3 and 1,
both `strict-majority` with no run-off.
The consistency package's source changed between the launch build and the branch head,
so the first reading is of the launch build and not of the head.

### Other readings against the source

Page line numbers are those of this pass's page and source line numbers are those of `page.md`.
Severity:
high is a changed meaning or lost content;
medium is a visible fault a reader of the page would meet;
low is polish or a judgement item.
There is no high-severity defect.

Checks that came out clean:

- No source sentence is missing and no paragraph was added.
- The glossary terms hold:
  Arona and Atri (page line 117),
  "headpiece" for 头壳 (111,
  122,
  140 and 157),
  "healed" for 治愈 (127),
  "putting on the kigurumi" for 变娃 (146,
  238),
  "outed" for 柜门炸开 (84),
  "high-performance robot" for 高性能机器人 (147),
  "cuddle" for 贴贴 (135),
  "transgender girl" for 跨性别女孩 (11),
  and "China West Normal University" for 西华师大 (68).
- The refrain is one frame four times
  (page lines 25,
  57,
  91 and 109,
  "So she must have been a girl who ..."),
  and the maximal repeat reads the same at lines 106 to 107 and 168 to 169 apart from the connective "And she also".
  "kigurumi" is lower case everywhere,
  and "headpiece" and "Menma" are uniform.
- en_CA:
  "travelling" (32) and "savour" (101),
  and a case-insensitive whole-word search for realize,
  recognize,
  organize,
  analyze,
  color,
  honor,
  center,
  favorite,
  traveled,
  gray,
  defense,
  fulfill,
  skillful,
  enroll,
  catalog,
  labor,
  neighbor,
  behavior,
  humor,
  flavor,
  harbor,
  rumor,
  plow,
  mold,
  gotten and any `-ize` or `-ization` word found nothing.
- Rendered with the repository's `marked@18.0.11`,
  the page has 3 `<h2>`,
  5 blockquotes,
  2 `<em>`
  (`Anohana: The Flower We Saw That Day` across a line break and `ATRI -My Dear Moments-`)
  and 1 link,
  `https://github.com/dongguacute`,
  which matches the source,
  with no stray asterisks or underscores.
  All 16 consolidated slice texts,
  the 4 consistency slice texts and the 3 page-assembly replacement texts are present in the page.

Medium:

1. Page lines 241 to 245,
   source line 87:
   再见咯面码，这次的 uno，真的加了很多呢。
   The page reads "Goodbye,
   Menma.
   This time,
   the game of Uno",
   then an em-dash,
   then "truly,
   so many draw cards stacked on the next player."
   "The next player" is not in the source and the doer of the adding is dropped;
   the source says only that this time the Uno game "really added a lot".
   TianqiChen66625 had "you stacked a lot of draw cards on everyone" and TianqiChen66624 "we really stacked a lot".
   The Uno reading is uncertain on all three pages;
   this one adds an addressee and leaves a fragment.
2. Front matter,
   page lines 2 to 6 (known,
   unchanged on TianqiChen66624 and TianqiChen66625).
   The archive authority ships it as it stands.
   The source has `name: めんま`,
   `alias: 面码, Menma` and `desc: 这次的捉迷藏，真的找不到你了。`;
   the page has `name: Menma`,
   `alias: めんま, Menma` and `desc: This time, we won’t find you anymore.`
   The `desc` loses 捉迷藏 (hide-and-seek),
   which the body carries at page line 232,
   and the alias drops 面码.

Low:

3. Page lines 238 to 239,
   source line 85:
   "Sleep well,
   Menma.
   This time,
   putting on the kigurumi",
   then an em-dash,
   then "truly,
   you’re really about to become one with it."
   真的 is rendered twice,
   as "truly" and "really".
   TianqiChen66625 had "you’re about to become one with it".
4. Page line 146,
   source line 47:
   "She had been putting on the kigurumi since high school" for 她从高中开始变娃.
   The past perfect progressive points back from a moment the narration has already passed;
   TianqiChen66624 and TianqiChen66625 wrote "She began putting on the kigurumi in high school",
   which is closer.
5. Page lines 247 to 252,
   source line 89:
   the source repeats 也会继续 three times and ends 就像说过的那样吧.
   The page keeps the "too" on the third clause only,
   adds "And",
   and drops the hedge 吧 ("just as she said").
   TianqiChen66624 and TianqiChen66625 had "also" on all three clauses.
6. Page lines 137 to 138,
   source line 43:
   "Want to come?
   Want to come?"
   for 要来吗要来吗！
   The closing exclamation is lost on the second;
   TianqiChen66624 had "Wanna come!
   " at that place.
7. Page lines 212 to 213,
   source line 75:
   "I still can’t hold on any longer" for 我还是坚持不下去了.
   还是 here is "in the end,
   after all",
   and TianqiChen66625's "I just can’t hold on anymore" reads closer.
   A judgement item.
8. Page line 86,
   source line 29:
   吾心安处，并非吾乡... is unquoted in the source,
   and the page puts it in quotation marks and ends it with the single character U+2026,
   where every other ellipsis on the page is three full stops
   (page lines 157,
   171,
   177,
   212 and 251) and the source uses three full stops.
   TianqiChen66624 and TianqiChen66625 also quoted it,
   and the part for 2026-10-15 records the marks as intended.
   The mixed ellipsis forms are new on this page.
9. Page lines 11 to 13:
   "(an anime)" is added after the title,
   which the source does not have.
   The archive's `page.en.md` carries it
   (its line 11) and the pass kept it.
10. Small additions:
    page line 16 "left everyone behind" for 离开了大家 (no "behind";
    inherited from the archive's line 13,
    and on TianqiChen66624);
    page line 50 "washing over everyone" (the object "everyone" is supplied for 扑面而来);
    and page line 152 "fellow fans" for 同好 (like-minded hobbyists,
    narrower as "fans").
11. Small omissions:
    page line 40 "using her fluent English and Japanese" drops 喜欢 ("liked to") and softens 高水平 to "fluent";
    page line 106 "And she also wanted" doubles the connective for 也是.

Information:

12. The page carries five em-dashes
    (page lines 37,
    38,
    232,
    238 and 244) where the source has none;
    the owner's ruling of 2026-09-21 leaves em-dashes as they are.

Read and found not to be defects:
吧 as "must have been" in the refrain,
"Under the lamplight" for 灯火下,
"In the plainest of lives",
理想 as "aspirations",
the nocturne,
mist and moonlight sentence,
"the N2 Japanese exam",
"Old Man Chen" for 陈大爷,
and "Contributor for this entry" for 条目贡献.

Against TianqiChen66625 this page is better in these places:

- The refrain is carried to all four places;
  TianqiChen66625 carried three and kept "So she must have been a gentle,
  sensitive girl."
- The maximal repeat of source lines 35 and 55 is carried;
  TianqiChen66625 left it in different wording at its lines 112 to 113 and 178.
- The university is "China West Normal University";
  TianqiChen66625 had "Xihua Normal University".
- "kigurumi" is lower case throughout;
  TianqiChen66625 mixed "Kigurumi" (its lines 131,
  136,
  161 and 163) with "kigurumi".
- The Atri gloss names the character and the work,
  as on TianqiChen66624;
  TianqiChen66625 wrote "(Atri’s own way of describing herself)".
- Uno is no longer set in italics as a title
  (TianqiChen66625 had *UNO* and TianqiChen66624 *Uno*).
- No "I suppose" or "wasn’t she?"
  variants of the refrain are left.
- "Want to come?"
  is closer to 要来吗 than TianqiChen66625's "Will you come?"

It is worse than TianqiChen66625 in these:
the Uno sentence (defect 1),
"truly ... really" (3),
"had been putting on" (4),
the 也 and 吧 losses in the last paragraph (5),
and "I still can’t hold on any longer" (7,
a judgement item).
The front matter (2) and the quotation marks around 吾心安处 (8) are unchanged across all three pages.

Against TianqiChen66624:

- Her last message is in her present,
  as on TianqiChen66625;
  TianqiChen66624 had moved it into the past.
- The refrain and the maximal repeat are both carried;
  TianqiChen66624 carried only the maximal repeat and left the refrain's four frames different.
- TianqiChen66624's spaced em-dashes are now unspaced.
- Uno regresses against TianqiChen66624's neutral "we really stacked a lot" only in that it adds "the next player".

Net:
this is the most consistent of the three pages,
with no high-severity defect,
and its remaining faults are small and mostly in the closing paragraphs.
