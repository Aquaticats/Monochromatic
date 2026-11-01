# OpenRouter and the corpus passes: 2026-10-20

What was read and decided on 2026-10-20.
Entries:
XingZ6015,
zheermao15,
CuspariaKLSY18,
zheermao16,
CuspariaKLSY19,
zheermao17,
zheermao19.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## XingZ6015 read, 2026-10-20, 00:40 UTC: SETTLED in 351.0 min of pass; five classes, and the rotation on this build ends

XingZ6015 was the third entry of the readiness rotation the owner chose on 2026-10-19,
launched at 18:14 UTC on pipeline `sha256-tree-v1:a92725968adf…`
([the launch](translation-repair-openrouter-2026-09-03-2026-10-19.md#xingz6015-launched-2026-10-19-1814-utc)).
It settled in 21,057,457 ms of pass
(`TALLY XingZ60 status=SETTLED slices=119 repairStatus=repaired ... repairAccepted=331 ... alignmentFindings=29 selection=contested`),
and the unit ended at about 00:05 UTC with `Result=success` and exit status 0.
`verify-published` read the page back:
1 of 1 page carries every wording its artifact promised
(`wordings=119 silent=0 chars=49489/expected 49426+separators missing=0 consistency-groups=1 read-at-generation=19`).
`DESTINATIONS XingZ60 source=2 page=2 dropped=0`,
no `DEFECTS` line,
no `[error]` line,
and 4525 `warn` lines.
No `REATTEMPT`,
`STALLED`,
`mdx-downgraded`,
`StreamOverrunError`
or error finish.

### Seats and meters

Three Gemma seats were dark on `SEAT CREDENTIAL REFUSED ... HTTP 401`,
as in the two earlier reads of the rotation.
Hyper ran dry at 22:18 UTC and Synthetic's weekly allowance at 22:22 UTC,
so from then on `glm-5.3`,
`hf:Qwen/Qwen3.8-27B` and `hf:moonshotai/Kimi-K3` were out of reach:
`glm-5.3` threw on 602 of its 725 asks and Qwen on 832,
with 112 more abandoned.
Across 281 readings Synthetic's weekly meter fell from 71.52% to 0%,
Hyper from 193 to 0,
OpenRouter from 21.92 to 16.13 USD,
and Bedrock stayed at 5.02 USD.
The spend report prices the metered seats at 278.87 credits and OpenRouter at 5.7961 USD.
The contest and consolidation lanes ran on 3 of 7 seats
(`lane-contest: closed on a short bench, 3 of 7 seats within reach, quorum 2 of 4`),
which is what an OpenRouter-only roster gives.

### Where the wall clock went

1919 rounds over 21.96 h of round time,
11.92 h of it (54.3%) waiting after quorum;
calls in flight averaged 5.77 and peaked at 20.
Lanes ran repair 18:39 to 22:32 UTC,
translate 22:32 to 23:05 UTC,
contest 23:05 to 23:09 UTC,
and consolidation 23:09 UTC to 00:05 UTC on 2026-10-20.

### The page

A Sonnet reader read all 1175 lines beside the original and the archive
(notes in `~/temp/agent/x15-read/notes.md`),
and the lead checked every finding this section records against the artifact and the run log
(`~/temp/agent/x15-read/classes.md`).
The page is complete where the archive stops at its own "Under Construction" mark,
defines all nine footnotes the original has (the archive defines none),
renders the original's TA as "they" and wing's 他 as "he",
and corrects several archive errors ("maimai DX",
坐牢 as "like doing time",
亚撒西).
No TA,
no Han outside one HTML comment,
no US spelling,
front matter equal to the archive's.

Five classes,
each fixed before the next launch:

- A consolidation kept an archive wording every judge rejected.
  At slice 45 every contest ballot called the archive flawed and split three ways;
  in the consolidation two judges chose the lane wording with "they" and called the archive's "he" and "sir" ineligible,
  the third declined every candidate,
  and the leader's weight of 1.5 fell short of the minimum of 2.00,
  so `keeping the fallback` shipped the archive.
  Slice 34 reached a deciding run-off on the same opening and is not this class:
  its gate then kept the standing on a 2 to 1 vote.
- The declared-name floor read a swap between two declared names as a lost name.
  At slice 80 both lanes' fixes,
  12 accepted issues among them a major tense one,
  were refused because the archive's "XingZ60" became "Lili",
  her declared alias and the rendering of the original's 璃璃
  (`translate-refused-declared-name (slice 80: archive text carries "XingZ60" and the replacement does not; keeping the archive text)`);
  once more at slice 15.
- The page pass dropped the original's section numbers.
  Both lanes rendered 其一 to 其十 as "Part One" to "Part Ten",
  the contest chose the numbered headings and the repair panel accepted the missing number as an omission,
  and `heading-series-unified` rewrote nine of them to the archive's bare style,
  under the rule class eighty-nine set on 2026-09-23.
- Contest judges chose a lane for keeping a hidden translators' note.
  At slice 6 all three judges chose the repair wording because it kept the archive's
  `<!-- 前面这句中文懂什么意思，先这么翻译，大家看一下 -->`,
  calling it page apparatus,
  over a cleaner translate wording.
  Ten English archives carry Han inside comments,
  `mikaela_khara` and `shi_Yumiaoya` among them.
- The archive's "unfinished" banners shipped on a complete page.
  `**Come back later!**`,
  `[!Warning] This version is NOT the formal version ...` and
  `[!Caution] This is an unfinished version ...` are the archive's,
  not the original's,
  and the archive-block review kept them as a notice about the page's own status.
  XingZ60 is the only archive in the corpus with such a notice.

One question went to the owner,
and the answer changes a house rule.
The page opens with her own words,
所以说我甚至可以去含着氰化钾胶囊做一些事情,
about doing dangerous things with a cyanide capsule on hand;
it is not how she died,
and the archive keeps the capsule.
Two of three contest judges chose the repair wording,
"do something knowing that it could cost me my life",
citing reader protection.
Asked at about 00:36 UTC whether reader protection covers a suicide means mentioned outside the account of the death or an attempt,
the owner chose "Only the death":
"Scope the rule to how she died or attempted,
so her quote keeps the cyanide capsule as the original and the archive have it."

Meaning errors of single renderings,
not classes:
footnote 9 renders the fan work 《罗德岛理工大学开学第一课》 as the "Rhode Island School of Design",
where the title lookup returned only pages about that school and one gate judge took them as support;
the Lu Xun quote at the Blank section takes the wrong subject;
胡蝶 became a person "Hu Die";
翻车 is rendered three ways;
several sentences about her stay in the present tense,
slice 11 among them,
where the contest's tense candidates fired on all three texts and none was in the past.
The pass log carries them so the next read can tell a recurrence.

Two reader findings did not hold.
"One Among Us" for 那些秋叶 is right,
since 那些秋叶 is the site's own Chinese name (seven corpus pages use it so),
and the archive's "autumn leaves" is the error.
And slice 34,
which the reader counted among the archive pastes,
shipped the archive by its gate's vote.

### What follows

The rotation on `a92725968adf…` ends here:
two reads with no class,
zheermao14 and CuspariaKLSY17,
then XingZ6015 with five,
so the record of three no-class reads on one build stands.
`yingying` was not launched.
The five classes and the owner's ruling are being fixed in five worktrees (`w210` to `w214`);
once they land with their cache versions bumped,
the branch worktree moves to the fixed build
and the rotation restarts from its first entry.

## zheermao15 read, 2026-10-20, 13:20 UTC: SETTLED in 46.0 min of pass; one class, and its fix is in progress

The restarted rotation began with the entry `zheermao101`,
launched as `zheermao15` at 02:12 UTC on 2026-10-20,
on tip `0c64519b57…` and pipeline `sha256-tree-v1:c003a383075f…`
(the unit `corpus-pass-corpus-pass-zheermao15`;
the handover section "Where the work stands" names the launch).
It settled in 2,759,820 ms of pass,
and the `DONE` line reads `elapsed=2759940ms`,
which is 46.0 min:
`TALLY zheermao101 status=SETTLED slices=15 repairStatus=unchanged ... translateChanged=11 documentsDiffer=11
pageChanged=9 pageSilent=0 alignmentFindings=13 selection=contested`.
The repair lane changed nothing (`repairChanged=0`,
29 issues,
21 accepted,
none resolved),
the translate lane changed 11 of the 15 slices,
and the page pass changed 9.
`DESTINATIONS zheermao101 source=2 page=3 dropped=0`,
and the publish wrote 15 slices into a page of 6026 characters.
The log holds 569 `warn` lines,
no `[error]` line,
no `DEFECTS` line,
and none of `REATTEMPT`,
`STALLED`,
`mdx-downgraded` or `StreamOverrunError`.
It holds no `verify-published` line either,
so this read did not rest on that read-back.

### Seats and meters

Three Gemma seats were dark on `SEAT CREDENTIAL REFUSED ... HTTP 401` (Bedrock's token had expired),
as in the earlier reads of the rotation:
`gemma-4-26b-a4b-it` and `google.gemma-4-e2b` threw on all 4 asks and `google.gemma-4-31b` on all 3.
`glm-5.3` threw on all 4 of its asks,
since every provider serving it was out of budget (Hyper was dry at launch).
Synthetic's weekly meter read 2% at 02:12 UTC and 0% at 02:40 UTC,
in repair slice 4.
From then on `hf:moonshotai/Kimi-K3` and `hf:Qwen/Qwen3.8-27B` threw
with "no provider can take ...:
every provider serving this model is out of budget,
seat unreachable":
Kimi threw on 56 of its 82 asks (26 usable),
and Qwen on 138 of 162 (24 usable).
The seats that threw nothing were `minimax-m3`,
`inception/mercury-2.5`,
`deepseek-v4.1-flash`,
`hf:zai-org/GLM-5.3-Flash`,
`typesafe/jev-1.13` and `xiaomi/mimo-v2.6-flash`.
From the translate lane at 02:47 UTC the roster was 8 with Kimi withheld,
and the absolute-naturalness review of consolidation slice 12 closed on a short bench
(3 of 7 seats within reach,
quorum 2 of 4).
The run ended with `SEATS DARK: 4 of 12 seats asked produced nothing usable`
(the three Gemma seats and `glm-5.3`).
OpenRouter read 16.11 USD at launch and 15.53 USD at the last meter reading,
02:58 UTC.

### The page

The lead read the shipped page (`fixed/people/zheermao101/page.en.md` of the run) beside the original and the archive,
and checked the letter and the email against the three photos of the page
(photos 1 and 2 are the letter,
photo 3 is the email and its reply).
Both renderings were faithful;
the email's "If I manage to die" is 如果成功死去了 in the photo.
The check is the lead's reading of the photos;
this entry records it and did not re-run it.

Two things in the page read as defects and are not,
and each has a source:

- The front matter `desc`,
  "Treat you to some beverages.",
  is the archive's,
  and `desc` stays frozen.
  `doc/decision/translation-repair-front-matter-guard.md` lets the archive's front matter stand,
  since a rewritten `desc` is an editorial change nobody asked for.
- "Zheermao tried to end her life several times" is the wording the house rule sets for a survived attempt.
  `package/translation-repair/house-rule/src/house-policy.ts` (line 87) says the page
  "says that she attempted suicide or tried to end her life,
  never by what means".

One class,
in the email block.
The archive's rendering of the reply from One Among Us carries a note between translators on the line the page renders as
"You can avoid the darkness’s sharpest edges and take shelter for a while in places where there is light":
`{/* 可以避开黑暗的锋芒，到那些有光的地方躲一躲。 这句不知道如何翻译 */}`.
The pinned archive holds that note once;
the shipped page and the run's artifact do not hold it.
The owner confirmed on 2026-10-20 that the page keeps such notes (B698,
"Keep the notes (Recommended)"),
so the loss is a defect.
The cause is the order of two steps.
The archive block review replaced the archive's email block before the lanes ran,
with a revised rendering (the artifact's `preparation.archiveText` reads "Arts is also a kitty"
where the pinned archive reads "Arts is also a cat-like person"),
and the revised block has no note.
The page assembly carries a note back by comparing the page with `preparation.archiveText`
(`archive-note-carry.ts`,
since `a3a7baeb3b`),
so the carry compared against a text that no longer held the note and had nothing to carry.

### What follows

The fix is in progress,
not landed:
an agent works on it in the worktree `w223` off `dc4982c9cb`,
red test first,
so that the prepared archive text keeps every note between translators the replaced block held.
The next launch is `CuspariaKLSY18` on the build that has the note fix and the seat-window census,
under a new frozen digest;
it is not a repeat of `zheermao101`,
whose read found nothing else on that page.

## CuspariaKLSY18 read, 2026-10-20, 14:55 UTC: SETTLED in 49.4 min of pass; no class

The note fix landed as `a6a0ed7dc0` after gate g121
(2564 `[PASS]` lines,
no `[FAIL]` line).
The rotation went on with the entry `CuspariaKLSY`,
launched as `CuspariaKLSY18` at 14:00 UTC on 2026-10-20,
on tip `a6a0ed7dc0…` and pipeline `sha256-tree-v1:1149dceac4ea…`
(the unit `corpus-pass-corpus-pass-CuspariaKLSY18`),
and the `START` line matched the frozen digest.
It settled in 2,965,786 ms of pass,
and the `DONE` line reads `elapsed=2965899ms`,
which is 49.4 min:
`TALLY CuspariaKLSY status=SETTLED slices=4 repairStatus=repaired repairIssues=48 repairAccepted=41 repairResolved=41
... repairChanged=4 translateStatus=complete translateChanged=4 documentsDiffer=3 pageChanged=4 pageSilent=0
alignmentFindings=1 selection=contested`.
The repair lane,
the translate lane and the page pass each changed all 4 slices.
The one alignment finding records that block pairing matched all 18 original and all 18 translation blocks
(`preparation.alignmentFindings` of the artifact),
not a defect.
`DESTINATIONS CuspariaKLSY source=0 page=0 dropped=0`,
and the publish wrote 4 slices into a page of 1898 characters.
The log holds 154 `warn` lines,
no `[error]` line,
no `DEFECTS` line,
and none of `REATTEMPT`,
`STALLED`,
`mdx-downgraded` or `StreamOverrunError`.
Its two `verify-published` lines are the build's output listing,
so this read did not rest on that read-back either.

### Seats and meters

Two Gemma seats were dark on `SEAT CREDENTIAL REFUSED ... HTTP 401`
("The security token included in the request has expired"):
`google.gemma-4-e2b` and `gemma-4-26b-a4b-it` threw on all 4 asks each and were withheld after them,
as `package/translation-repair/doc/status.md` records for the Bedrock Gemma seats.
`inception/mercury-2.5` threw on 6 of its 54 asks,
and `hf:Qwen/Qwen3.8-27B` had 12 asks abandoned;
every other seat threw nothing.
The run ended with `SEATS DARK: 2 of 11 seats asked produced nothing usable`.
Synthetic's weekly meter read 8% at 14:00 UTC and 2.03% at the last reading,
14:50 UTC,
so the next run starts near the level at which zheermao15 lost Kimi and Qwen.
Hyper's balance went from 250 to 229,
OpenRouter from 15.52 USD to 15.47 USD,
and Bedrock stayed at 5.02 USD.

### The page

The lead read the shipped page (`fixed/people/CuspariaKLSY/page.en.md` of the run) beside the original (`page.md`)
and the archive (`page.en.md`),
line by line.
Every line of the original is rendered,
in the past tense for a life that ended,
and the archive's errors the lanes were there to repair are repaired:
"She didn’t consider her life more valuable than others’" is now "She didn’t value her own life much" (不怎么看重自己的生命),
"She doesn’t trust God 100%" is "She didn’t really believe in God" (不是太信上帝),
and the last paragraph now carries 双方情绪爆发,
抢救 and 脑死亡,
which the archive left out.

Two things in the page read as defects and are not,
and each has a source:

- The front matter `desc`,
  "Be more similar to “herself”.",
  is the archive's and does not render the original's 「泠水鱼鱼子」是在喂鱼时起的哦,
  and `desc` stays frozen.
  `doc/decision/translation-repair-front-matter-guard.md` lets the archive's front matter stand.
- The archive's "She swallowed lots of her meds that night" is gone.
  The original does not hold it,
  and `package/translation-repair/house-rule/src/house-policy.ts` (line 87) says the page tells a survived attempt
  "never by what means".

Three renderings are the judges' choices and read less well than they could,
with no shared cause the lead found,
so they are recorded and not counted as a class:

- "Word came again that she had been confirmed brain-dead" for 再次得到消息得知其已经确认脑死亡:
  "again" renders 再次 word for word,
  where "The next news was that" says what the original means.
- "Contact with futa,
  too,
  planted the seed of an idea in her" for 加上接触了 futa 萌生想法:
  the idea is the one the list is about,
  transitioning,
  and "Contact with" reads as touch.
- "was tormented by having no way to do so" for 苦于没有能力:
  苦于 is troubled by,
  weaker than tormented.

### What follows

Three fixes to the cut of the notes between translators landed after this run's build.
The cut of a note alone on a quoted line now takes the line
(`cca7711de7`,
gate g122),
and the five cache versions whose sheets cut notes moved for it.
The cut of two notes on one line and of a note ending a CRLF line followed
(`a5dc23c5bb`,
gate g123)
and ride inside those versions,
since no pinned archive's sheet text changes
and a settled entry leaves no slice-cache record
(`retireSettledEntryCache` in `package/translation-repair/corpus-pass/src/corpus-run/pass-entry-caches.ts`).
"Ready" rests on a rotation on one frozen build,
and `a5dc23c5bb` is not this run's build,
so the rotation restarts at its first entry:
`zheermao16` launched at 14:58 UTC on 2026-10-20 on pipeline `sha256-tree-v1:16f46cb79c96…`,
and its `START` line matched the frozen digest.
It is also the first run of the note fix on a page that holds a note,
since `CuspariaKLSY`'s archive holds none.

## zheermao16 read, 2026-10-20, 15:50 UTC: SETTLED in 43.2 min of pass; one class, and the rotation on this build ends

`zheermao16` ran as the unit `corpus-pass-corpus-pass-zheermao16`
on tip `a5dc23c5bb…` and pipeline `sha256-tree-v1:16f46cb79c96…`,
and its `START` line matched the frozen digest.
The `DONE` line reads `elapsed=2592966ms`,
which is 43.2 min:
`TALLY zheermao101 status=SETTLED slices=15 repairStatus=repaired repairIssues=34 repairAccepted=28 repairResolved=28
... repairChanged=9 translateStatus=complete translateChanged=10 documentsDiffer=10 pageChanged=11 pageSilent=0
alignmentFindings=14 selection=contested`.
`DESTINATIONS zheermao101 source=2 page=3 dropped=0`,
and the publish wrote 15 slices into a page of 5973 characters.
The log holds 278 `warn` lines,
no `[error]` line,
no `DEFECTS` line,
and none of `REATTEMPT`,
`STALLED`,
`mdx-downgraded` or `StreamOverrunError`.

### Seats and meters

Three Gemma seats were dark on `SEAT CREDENTIAL REFUSED ... on bedrock` with HTTP 401
("The security token included in the request has expired"):
`gemma-4-26b-a4b-it`,
`google.gemma-4-e2b` and `google.gemma-4-31b` threw on all 4 asks each and were withheld after them.
`inception/mercury-2.5` threw on 2 of its 118 asks,
and `hf:Qwen/Qwen3.8-27B` had 4 asks abandoned.
The run ended with `SEATS DARK: 3 of 12 seats asked produced nothing usable`.
Synthetic's weekly meter read 2.03% at 14:58 UTC and went dry at 15:22 UTC.
Hyper's balance went from 229 to 150,
OpenRouter from 15.47 USD to 15.36 USD,
and Bedrock stayed at 5.02 USD.
At this run's draw,
Hyper's balance covers about two more runs of this size,
and Hyper will not be recharged.

### The page

The lead read the shipped page (`fixed/people/zheermao101/page.en.md` of the run) beside the original (`page.md`),
the archive (`page.en.md`) and the three photos,
line by line.
The note between translators is kept once,
as a paragraph of its own after the email's blockquote
(`{/* 可以避开黑暗的锋芒，到那些有光的地方躲一躲。 这句不知道如何翻译 */}`),
with no unclaimed block and no bare quote line,
so the note fix holds on the page it was made for.
The email now renders the sentence that note is about,
"where you can steer clear of the darkness’s sharp edge and take shelter somewhere bright",
and every line of the photo of the email,
the closing 顺颂春祉 ("Wishing you well this spring") among them,
which the archive left out.
Arts's letter now carries 比比皆是 ("Such people are everywhere"),
and its "cat-like person" is "kitty" (猫猫).
The `>   >` line inside the reply keeps a trailing space,
which the archive's line carries too.

One class:
the handle 寒涟漪 ships as "Han Lian Yi (Cold Ripples)",
where the archive renders it "Han Lianyi",
which the link `https://t.me/Lianyi520/565` also spells.
The house rule
(`package/translation-repair/house-rule/src/house-policy.ts`,
line 104)
romanizes and glosses only "A person's handle with no declared,
archive or corpus rendering",
and its own grouping for three syllables gives "Hanlian Yi",
so the shipped form matches neither the archive nor the rule.
In slice 13 a repair-lane critic filed the gloss and the grouping as owed and the panel accepted both;
the lane contest then chose the translate lane's "Han Lianyi" on all four ballots,
naming the archive's established rendering;
the consolidation's select round chose "Han Lian Yi (Cold Ripples)" on three ballots,
one of them saying the handle has "no declared or archive rendering";
and the polish gate shipped it on three of five ballots.
The investigation of which sheets lack the archive's rendering,
and of which guard should keep it,
was started at about 15:49 UTC on 2026-10-20 as its own piece of work.

Separately,
the front matter `desc` keeps the archive's "Treat you to some beverages.",
while the quote it comes from now reads "milk tea" (奶茶).
`doc/decision/translation-repair-front-matter-guard.md` lets the archive's front matter stand,
so this is recorded and not counted as a class.

### What follows

The class ends the rotation on `a5dc23c5bb` with no read that counts,
and its fix will change this page,
so the rotation restarts at `zheermao101` on the build that carries the fix.
`CuspariaKLSY19` was launched at 15:47 UTC on 2026-10-20 on the same frozen build to find more classes meanwhile,
and its read does not count toward readiness.

## CuspariaKLSY19 read, 2026-10-20, 16:10 UTC: SETTLED in 16.0 min of pass; no class, and a hunt that does not count

`CuspariaKLSY19` ran as the unit `corpus-pass-corpus-pass-CuspariaKLSY19`
on tip `a5dc23c5bb…` and pipeline `sha256-tree-v1:16f46cb79c96…`,
and its `START` line matched the frozen digest.
The `DONE` line reads `elapsed=961503ms`,
which is 16.0 min:
`TALLY CuspariaKLSY status=SETTLED slices=4 repairStatus=repaired repairIssues=32 repairAccepted=20 repairResolved=18
... repairChanged=4 translateStatus=complete translateChanged=4 documentsDiffer=4 pageChanged=4 pageSilent=0
alignmentFindings=1 selection=contested`.
`DESTINATIONS CuspariaKLSY source=0 page=0 dropped=0`,
and the publish wrote 4 slices into a page of 1923 characters.
The log holds 160 `warn` lines,
no `[error]` line,
no `DEFECTS` line,
and none of `REATTEMPT`,
`STALLED`,
`mdx-downgraded` or `StreamOverrunError`.

### Seats and meters

The same three Gemma seats were dark on HTTP 401 and withheld after 4 asks each;
`inception/mercury-2.5` threw on 8 of its 56 asks;
the run ended with `SEATS DARK: 3 of 11 seats asked produced nothing usable`.
Synthetic stayed dry.
Hyper's balance went from 150 to 111,
OpenRouter from 15.36 USD to 15.31 USD,
and Bedrock stayed at 5.02 USD.

### The page

The lead read the shipped page beside the original and the archive,
line by line,
and found no class.
The page keeps the archive's sentence on the nickname,
"The name means “fish in clear water.”",
which the archive's own page carries,
so it is not the class of the `zheermao16` read.
Three renderings are the judges' choices and read less well than they could,
with no shared cause found,
so they are recorded and not counted as a class:

- "When word came again,
  they learned that she had been confirmed brain dead" for 再次得到消息得知其已经确认脑死亡:
  the original names no one who learned it,
  and "they" has no antecedent on the page.
- "Pressure from family and school was becoming increasingly heavy" for 家庭问题，学校问题日益突出:
  问题 is problems,
  not pressure;
  the wording is the archive's.
- "didn't have the means to do so" for 苦于没有能力:
  苦于 (troubled by) is dropped;
  the wording is the archive's.

### What follows

No further hunt runs on `a5dc23c5bb`.
Hyper is prepaid and will not be recharged,
and its 111 left at 16:03 UTC covers less than the `zheermao101` and `CuspariaKLSY` runs of the restarted rotation took
on this build (79 and 39),
so the balance is kept for the runs that count.
The rotation restarts at `zheermao101` on the build that carries the fix for the `zheermao16` class.

## zheermao17 read, 2026-10-20, 20:30 UTC: SETTLED in 37.3 min of pass; no class, and the first read that counts on its build

`zheermao17` ran as the unit `corpus-pass-corpus-pass-zheermao17`
on tip `5ae2173d23…` and pipeline `sha256-tree-v1:ee83b531c14f…`,
and its `START` line matched the frozen digest.
A first launch at 18:58 UTC stopped before `START` on a package the branch worktree had not installed,
with no provider called (ledger M285);
the relaunch began at 19:16 UTC.
The `DONE` line reads `elapsed=2237935ms`,
which is 37.3 min:
`TALLY zheermao101 status=SETTLED slices=15 repairStatus=repaired repairIssues=35 repairAccepted=20 repairResolved=20
... repairChanged=9 translateStatus=complete translateChanged=9 documentsDiffer=7 pageChanged=10 pageSilent=0
alignmentFindings=14 selection=contested`.
`DESTINATIONS zheermao101 source=2 page=3 dropped=0`,
and the publish wrote 15 slices into a page of 5977 characters.
The log holds 226 `warn` lines,
no `[error]` line,
no `DEFECTS` line,
and none of `REATTEMPT`,
`STALLED`,
`mdx-downgraded` or `StreamOverrunError`.

### Seats and meters

The three Gemma seats on Bedrock were dark on HTTP 401 again and withheld after 4 asks each;
the run ended with `SEATS DARK: 3 of 12 seats asked produced nothing usable`.
Every other seat answered:
`deepseek-v4.1-flash` 119 of 119,
`minimax-m3` 115,
`hf:moonshotai/Kimi-K3` 109,
`inception/mercury-2.5` 94 of 95,
`glm-5.3` 63,
`hf:zai-org/GLM-5.3-Flash` 50,
`hf:Qwen/Qwen3.8-27B` 34,
`xiaomi/mimo-v2.6-flash` and `typesafe/jev-1.13` 23 each.
Synthetic's weekly meter read 2% at 19:16 UTC and went dry at 19:38 UTC.
Hyper's balance went from 110 to 45,
OpenRouter from 15.31 USD to 15.23 USD,
and Bedrock stayed at 5.02 USD.

### The page

The lead read the shipped page (`fixed/people/zheermao101/page.en.md` of the run) beside the original (`page.md`),
the archive (`page.en.md`) and the three photos,
line by line,
and found no class.

- The quote that opens the page now offers milk tea (奶茶) where the archive gave "some beverages",
  and its second line asks Zheermao by name which flavour she likes,
  as 折耳猫喜欢什么口味的呀 does.
- The game is "Mystery Lover 2:
  Forgotten Truth".
  Two critics filed the subtitle as owed,
  one citing the indienova database.
  The lead checked it:
  Steam lists 恋爱绮谭 不存在的真相 under its Chinese name only,
  while the Nintendo eShop sells it under that English title
  (developer Never Knows Best,
  released 2025-05-29),
  so the title is the published one.
- Arts's letter,
  read against the two photos,
  now carries what the archive had wrong:
  Zheermao had blocked searches by QQ number (禁止通过QQ号查找),
  Arts is "a cat" (一只猫猫),
  and MtF is "a trans woman".
  It still says "the same hopes and dreams as you" and "for themselves",
  where the photo says 和我们 and 为了我们的未来 (us,
  our future);
  the wording is the archive's.
- The email,
  read against the third photo,
  now renders the sentence the note between translators is about,
  "you can dodge the edge of the darkness and take shelter in the bright places",
  and the closing 顺颂春祉 as "Respectfully,
  with the blessings of spring."
  Its salutation 玖月折耳猫： is left out,
  as in the archive.
- The note is kept once,
  as a paragraph of its own after the email's blockquote,
  where `zheermao16`'s page had it too,
  so B709's account of where the carry puts a note a writer dropped holds.
  It still says the sentence could not be translated,
  and the owner's rule of 2026-10-20 keeps the notes as written.

The front matter `desc` keeps the archive's "Treat you to some beverages.",
while the quote it comes from now reads milk tea;
`doc/decision/translation-repair-front-matter-guard.md` lets the archive's front matter stand,
so this is recorded and not counted as a class,
as on `zheermao16`'s page.

### What follows

`CuspariaKLSY` was not launched.
Hyper's 45 left is less than this run spent,
and the NeuralWatt provider the owner added at about 19:45 UTC on 2026-10-20
serves `deepseek-v4.1-flash` and the GLM-5.3-Flash seat on 550 Wh a day of free energy.
Adding it changes the build,
so a read on `5ae2173d23` would stop counting once it lands.
The rotation restarts at `zheermao101` on the build that carries NeuralWatt.

## zheermao19 read, 2026-10-20, 23:53 UTC: SETTLED in 35.6 min of pass; no class traced to code, on a short bench

`zheermao19` (entry zheermao101) ran as the unit `corpus-pass-corpus-pass-zheermao19`
on tip `5b5dca49a9…` and pipeline `sha256-tree-v1:f706c65f04541d1d043d5d39c4597ef034b537c8a73e827ef8c937da7c6d4ee9`,
the first build with NeuralWatt,
and its `START` line carried both.
It ran from 23:00:24 to 23:35:58 UTC.
The `DONE` line reads `elapsed=2134104ms`,
which is 35.6 min:
`TALLY zheermao101 status=SETTLED slices=15 repairStatus=repaired repairIssues=24 repairAccepted=13 repairResolved=12
... repairChanged=7 translateStatus=complete translateChanged=11 documentsDiffer=9 pageChanged=9 pageSilent=0
alignmentFindings=14 selection=contested`.
`DESTINATIONS zheermao101 source=2 page=3 dropped=0`,
and the publish wrote 15 slices into a page of 6016 characters.
The log holds 395 `[warn]` lines
(zheermao17:
226),
no `[error]` line,
no `DEFECTS` line,
and none of `REATTEMPT`,
`STALLED`,
`mdx-downgraded` or `StreamOverrunError`.
The full line-by-line read is kept at `/var/home/user/temp/agent/x15-fix/reads/zheermao19-read.md`;
this section keeps every departure it found.

### Seats and meters

Usable answers of asked,
from the `SEAT` lines after `DONE`:
`deepseek-v4.1-flash` 141 of 143,
`minimax-m3` 140 of 140,
`inception/mercury-2.5` 111 of 116,
`hf:moonshotai/Kimi-K3` 78 of 81,
`hf:zai-org/GLM-5.3-Flash` 56 of 56,
`hf:Qwen/Qwen3.8-27B` 27 of 78,
`glm-5.3` 25 of 73,
`typesafe/jev-1.13` 24 of 24,
`xiaomi/mimo-v2.6-flash` 23 of 23,
and the three Gemma seats 0 of 4 each.

- Synthetic went dry at 23:21:54 UTC (`syntheticWeekly=0%`,
  from 1.40% at the start).
- Hyper went dry at 23:32:11 UTC (`hyperBalance=0`,
  from 39;
  zheermao17 had closed at 45,
  and `zheermao18`'s 21 Hyper calls ran between them).
- From then every provider of `glm-5.3` and `hf:Qwen/Qwen3.8-27B` was out of budget,
  which is their 48 and 51 throws,
  and `hf:moonshotai/Kimi-K3` threw 3 the same way.
- The Bedrock key had expired
  (HTTP 401 "The security token included in the request has expired.");
  the three Gemma seats were withheld after 4 asks each,
  though two of them have OpenRouter sides
  (fixed in keyless worktree w249;
  the owner ruled on 2026-10-20 to leave Bedrock off).
- OpenRouter went from 15.22 USD to 15.02 USD,
  Bedrock stayed at 5.02 USD.
- NeuralWatt served 197 calls for 28.686 Wh measured against 30.487 Wh estimated
  (`deepseek-v4.1-flash` 143 calls,
  10.704 Wh against 19.628;
  `glm-5.3-flash` 54 calls,
  17.982 Wh against 10.859),
  and its meter fell from 549.983 Wh to 522.124 Wh.
  The run-end `NeuralWatt PLAN ENERGY SPENT: 345.2 Wh` is the account,
  not this run:
  `mimo-v2.6-pro` on the owner's other key rose 335 Wh between 23:15 and 23:36 UTC by `/v1/usage/by-model`.

The bench was short.
The log holds 30 `closed on a short bench` lines (zheermao17:
none),
and the consolidation's naturalness reviews read `3/7 usable ... quorumNeeded=2 ... unreachable=4`,
once `2/7` with no quorum,
where zheermao17's read `5/7 usable ... quorumNeeded=4`.
Twelve decisions kept the fallback text (zheermao17:
ten).
This read therefore counts for less than zheermao17's:
the page was judged on three or four voices.

### The page

The lead's reader compared the shipped page with the original,
the archive,
the three photos and zheermao17's page,
line by line.
Her life is in the past tense throughout.

What is now right that zheermao17 had wrong:

- Arts's letter says "the same feelings and hopes as us" and "for our future"
  for 和我们 and 为了我们的未来,
  where zheermao17 still had the archive's "as you" and "for themselves",
  and its two 比如 items are two,
  as in the photo.
- 小女孩 keeps 小 ("a little trans girl").
- 唯利是图，贪生怕死 is carried whole ("only after profit,
  coveting life and fearing death").
- 连夜 is "that same night",
  where zheermao17 said "they stayed up through the night".
- The reply's salutation 玖月折耳猫： is carried ("Good evening,
  Zheermao,").
- 再给你一个温暖的抱抱 is a hug given ("Here's another warm hug").

What departs from the Chinese,
each a single wording in one sentence:

- 尽力活着了 in her own email is "I tried to live";
  尽力 (did my best) is lost,
  where zheermao17 had "I tried my best to live".
  No candidate in the run wrote "tried my best"
  (`rg --fixed-strings` over the run directory finds it nowhere),
  so no seat offered it and no critic flagged the loss.
- 极度缺爱 is "starved of love";
  zheermao17 had "extremely lacking in love".
- 就好了呢 is "It would be better if",
  which reads as a comparison with meeting in heaven that the Chinese does not make;
  zheermao17 had "so nice".
- 顺颂春祉 is "With spring blessings",
  losing the formal 顺颂;
  zheermao17 had "Respectfully,
  with the blessings of spring."
- 邮箱 is dropped ("an email to One Among Us");
  zheermao17 had "the One Among Us mailbox".
- Arts's 我搜索之后发现 (after I searched,
  I found) is dropped,
  as are two 也 (in Arts's "Arts also hates this unfair world" and in the Zhihu sentence).
- "only" is added twice in 得意时想着…失意时想着…,
  "go on" in 只会去伤害,
  and 更不会 (much less) is flattened to "and they would never".
- Three em-dashes stand where the Chinese has commas
  (the 比比皆是 sentence,
  比如Arts,
  and 这是我们的笔永远无法企及的);
  the archive has none,
  and no rule of the run's sheets governs dashes.
- 离世 is "death" for the friend and "departure" for her,
  悼文 is "epitaph",
  `desc` keeps the archive's "Treat you to some beverages."
  while the quote offers milk tea,
  and the note between translators still says its sentence could not be translated:
  all as on zheermao17's page and recorded there.

No departure changes who died,
when,
how she is named,
or what the writers tell her,
and the run shows no log marker of a class.
The losses against zheermao17 are of one kind:
an intensifier or a small word the Chinese carries,
lost where no seat offered the fuller wording,
on a bench of three or four voices.
That is recorded here as a condition of the run,
not as a class with a fix:
the cause read from the log is the empty Synthetic and Hyper meters
and the expired Bedrock key,
and the code that seats a refused credential's model elsewhere is the fix in w249.

### What follows

The rotation does not go on on this build.
Four changes are being landed together before the next launch:
the NeuralWatt day anchored at the account's 15:15 EDT and counted account-wide (w247),
the DeepSeek V4.1 Flash 1017 served ids (w248),
the credential refusal that holds out the provider instead of the seat (w249),
and the readonly fix already landed as `506972d0e4`.
They change the build,
so a read counts only on the combined one,
and the rotation restarts there at `zheermao101`,
then `CuspariaKLSY` and the rest,
each read before the next launch.
