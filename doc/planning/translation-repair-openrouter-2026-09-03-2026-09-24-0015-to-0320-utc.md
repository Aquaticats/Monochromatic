# OpenRouter and the corpus passes: 2026-09-24, 00:15 to 03:20 UTC

What was launched,
read and decided on 2026-09-24 00:15 to 03:20 UTC.
Entries:
zheermao9,
shi_Yumiaoya18,
CuspariaKLSY12,
CuspariaKLSY11,
yingying8,
yingying7,
zheermao8,
zheermao7,
CuspariaKLSY10.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## zheermao9 read, 2026-09-24, 03:20 UTC: SETTLED in 20 min, class one hundred three live under Synthetic's weekly dry-out, class one hundred six not exercised (the contest chose the repair lane 5 of 5), class one hundred seven live on slice 0; but the chunks in flight kept the bench read at their start, so seven of eight checker rounds after the dry-out heard 1 of 3 (class one hundred nine, fixed in 21ea25993); shi_Yumiaoya18 launched

`zheermao9` (entry `zheermao101`,
frozen `85ed2881f`,
pid 4089371) SETTLED at 03:04 UTC 2026-09-24,
20 min after its 02:44 UTC launch (TALLY `ms=1191927`),
one attempt,
988 seats asked,
759 calls,
0.37 USD by the meters (Bedrock 48.41 to 48.28,
OpenRouter 95.71 to 95.47),
Synthetic wet at the launch and dry from 02:56:03 UTC (`syntheticWeekly=0%`,
the weekly allowance,
not the five-hour window,
which read 2734 of 2750),
Bedrock and OpenRouter wet,
Hyper dry.
glm-5.3 dark (59 asked,
59 threw;
the Hyper-only seat,
by design).
Jev 29 of 29;
10 abstention lines with reasons.

Class one hundred six not exercised:
slice 9's contest chose the repair lane 5 of 5 with every ballot calling the archive flawed (`lane-won`,
not `settled-neither`,
so the run-off predicate stood aside by design),
and the page's reply label reads "One Among Us members sent her a reply that very night:"
for 那些秋叶的成员…连夜,
the `zheermao8` observation resolved by the contest itself.
Class one hundred seven live on slice 0 (`translate-archive-disputed`,
two accepted `accuracy/addition` claims;
the contest chose translate 3 of 4;
the page's opening quote comes from the translate lane).
Consolidation terminals:
slate-declined-standing on slices 0 and 9,
slate-unjudged on 1,
slate-endorsed on 3,
6,
13 and 14,
gate-kept on 4 and 5,
consolidated on 11;
no unendorsed-standing line.
Every check as `zheermao8`:
2 headings,
front matter equal,
no TA,
no error finish,
no double blank line;
the email label "English translation of the email conversation:"
one line (class seventy-seven holding;
the archive writes "English translation of the preceding email conversation:"
and "English translation of the preceding letter:"),
curly "we'll" (class ninety-six holding),
classes one hundred one,
two and four holding.
The page differs from `zheermao8`'s by 75 diff lines,
the reply label and the opening quote among them.

Class one hundred three live for the first time:
Synthetic dried at 02:56:03 UTC eleven minutes into the repair lane,
and five chunks starting after it logged "chunk re-seated under a hold:
checkers=hf:Qwen/Qwen3.8-27B,gemma-4-26b-a4b-it,google.gemma-4-e2b" (02:56:16 to 02:56:34 UTC),
the dry reading keeping the unreachable Qwen seat and substituting the withheld Kimi with `google.gemma-4-e2b`.

Class one hundred nine:
seven of the eight checker rounds after the dry-out still heard 1 of 3 (chunks 5,
9,
4,
3 and 0 "stage-quorum-unmet (checker 1/3)",
the introduced-defect probe the same,
"checker:
bench short of quorum,
reachable 1 of 3;
closing on 2 voices"),
asking the static bench Qwen3.8-27B and Kimi-K3 (both `NoProviderForModelError`;
SEAT Qwen 133 asked 116 threw,
Kimi 51 asked 34 threw) beside gemma-26b.
The class one hundred three re-seat lands only at a chunk's start;
with overlap 8 the chunks already in flight kept the bench read at their start while their checker stage ran a minute or more later,
after the dry-out.
Fixed in 21ea25993:
the checker stage re-reads the seating at the stage itself
(`repair-checker-reseat.ts`:
`checkerBenchAtStage` asks the lane's `beforeSlice` hook again,
keeps the chunk's bench when the hook names no checkers or the same ones,
and asserts quorum on a fresh bench before logging "checker stage re-seated: <fresh> (the chunk was seated with <old>)";
`standingSeating` is the default for callers without a hook;
`assertCheckerBench` holds the two bench assertions);
threaded as `reseat` from `repairPreparedDocument` through `settleRepairSlice`,
`buyRepairSlice` and `repairChunk` to `proveRepairedChunk` (`repair-chunk-proof.ts`,
the checker stage,
the region collection and the introduced-defect probe on the stage's bench;
`repair-chunk-evidence.ts` holds the parse and the neighbour window,
both split out of `repair-chunk.ts` for the line budget).
Guards red first at 1ab453219 (`repair-checker-reseat.unit.test.ts`:
keeps the chunk's bench,
asks the bench read at the stage,
refuses a bench below the floor;
`repair-translation.unit.test.ts`:
the hook answering `{}` at the chunk's start and the substitute roster at the stage seats the substitute on the checker round),
lint 0 and 0,
types clean,
full suite green (`suite-class109.log`,
1114 PASS);
the class thirteen "asks before every slice" guard now reads at least once per slice,
since the checker stage asks again.
Not exercised live yet:
it needs a provider to dry out inside the repair lane;
Synthetic is dry for the week,
so the next runs verify no regression only.

`shi_Yumiaoya18` (frozen `21ea25993`,
pid 91959,
overlap 8)
launched 03:23 UTC on Bedrock and OpenRouter
(meters 48.28 USD Bedrock,
95.47 OpenRouter at launch;
Synthetic dry for the week,
Hyper dry),
waiter wait-s18.
Read for any "checker stage re-seated" line (none expected without a hold beginning inside the lane),
the checker rounds hearing 2 of 3 on the dry reading (gemma-26b and gemma-4-e2b),
then classes seventy-eight,
seventy-nine,
ninety-two,
ninety-five,
ninety-six,
one hundred one and one hundred two beside every check of `shi_Yumiaoya17` (4 headings,
front matter equal,
the death paragraphs on the page,
the closing quote as the archive's two lines,
no TA,
no error finish,
no double blank line).
Then `mikaela_khara`.

## CuspariaKLSY12 read, 2026-09-24, 02:50 UTC: SETTLED in 25 min, class one hundred eight live on slice 3 (the softened detail refused on the translate slate, the contest and the gate), no class; zheermao9 launched

`CuspariaKLSY12` (frozen `85ed2881f`,
pid 3947732) SETTLED at 02:42 UTC 2026-09-24,
25 min after its 02:17 UTC launch (TALLY `ms=1478275`),
one attempt,
437 seats asked,
420 calls,
0.13 USD by the meters (Bedrock 48.46 to 48.41,
OpenRouter 95.79 to 95.71),
Synthetic wet again from the launch (five-hour window 2750 of 2750 at 02:17 UTC,
so the handover's "spent since 23:15:27 UTC" line was stale by then),
Hyper dry;
the time is Synthetic's again (Qwen3.8-27B streams there at 39 s,
the naturalness reviews 26 s in grace),
against `CuspariaKLSY11`'s 5 min on Bedrock and OpenRouter alone.
glm-5.3 dark (17 asked,
17 threw;
the Hyper-only seat with Hyper dry,
by design).
Jev 12 of 12;
10 abstention lines with reasons.

Class one hundred eight live on slice 3,
the death paragraph.
The repair lane softened the accepted addition again,
this run into "She fell asleep" (the repair lane's own sheets are unchanged,
as the ninth addendum says),
and the stand-in carried it into every later sheet with the ARCHIVE RENDERING DISPUTED block beside it:
the translate slate declined its first round,
ran off 2 of 6 (class eighty-two) and chose the candidate without it,
one ballot writing "Candidate 2 also adds 'She fell asleep'";
the contest chose translate 4 of 5 with every ballot calling the archive flawed,
two ballots naming the repair lane's "inference" and one "avoiding unsupported claims found in the archive rendering";
the gate heard 4 of 5 (standing 2,
consolidated 1,
neither 1) and kept the contest winner as the standing over a proposal differing only in "her friend" against "a friend",
one standing ballot writing "both omit the rejected medication detail".
The page reads "A conflict broke out while she was trying to console her friend,
and emotions boiled over on both sides.
After waking up,
she argued with her father again.
She then attempted suicide and was sent to the hospital for emergency treatment."
where `CuspariaKLSY11` shipped "She took medication that night";
双方情绪爆发,
又 and 抢救 all rendered,
the archive's "swallowed lots of her meds" and every softening of it gone.
Class one hundred seven live on slice 3 (`translate-archive-disputed`,
three accepted `accuracy/addition` claims,
in the lanes and the consolidation);
slice 1 undisputed this run (no accepted addition claim against it).
Classes one hundred four and five holding (the life in the past;
the gloss inline as "(Cold Water Fishy)" on the nickname line,
the restore standing aside by design).
Every check as `CuspariaKLSY11`:
1 heading,
front matter equal,
no TA,
no error finish,
no double blank line.
Terminals:
slice 1 gate-kept (2 to 2,
the standing carrying the gloss),
slice 2 slate-declined and unendorsed,
slice 3 gate-kept as read.
The archive's gloss line ("Ling Shui Yu Yu Zi" means fish in clear water) is on the page in the parenthetical form only,
as on `CuspariaKLSY8` to `11`.
No class.

`zheermao9` (entry `zheermao101`,
frozen `85ed2881f`,
pid 4089371,
overlap 8)
launched 02:44 UTC on Synthetic,
Bedrock and OpenRouter
(meters 48.41 USD Bedrock,
95.71 OpenRouter at launch),
waiter bhdypi1ya.
Read slice 9 first for class one hundred six (a `translate-declined-retried` finding on a consolidation slice after a contest with the archive flawed on every ballot;
the reply label rendering plural members that same night),
any `translate-archive-disputed` line and its ARCHIVE RENDERING DISPUTED block,
then classes seventy-seven,
ninety-six,
one hundred one,
one hundred two and one hundred four beside every check of `zheermao8` (2 headings,
front matter equal,
both labels as the archive,
no TA,
no error finish,
no double blank line).
Then `shi_Yumiaoya` or `mikaela_khara`.

## CuspariaKLSY11 read, 2026-09-24, 02:20 UTC: SETTLED in 5 min, class one hundred seven live on slices 1 and 3, the stand-in still carrying "took medication that night"; class one hundred eight built (the dispute note on every sheet), CuspariaKLSY12 launched

`CuspariaKLSY11` (frozen `a40c28cad`) SETTLED at 01:55 UTC 2026-09-24,
5 min after its 01:49 UTC launch (TALLY `ms=323459`),
one attempt,
327 seats,
13 retry rounds,
256 calls,
0.10 USD by the meters (Bedrock 48.50 to 48.46,
OpenRouter 95.88 to 95.80),
on Bedrock and OpenRouter alone (Synthetic's window spent,
Hyper dry),
the roster 8 after the gpt-oss-120b cull.
Jev 11 of 11;
5 abstention lines with reasons.

Class one hundred seven live:
`translate-archive-disputed` on slices 1 and 3 in the lanes and again in the consolidation (two accepted `accuracy/addition` claims each),
the repair text the incumbent on both,
slice 3 (the death paragraph) shipped as `{kind: incumbent}` gate-kept,
and the archive's "swallowed lots of her meds" nowhere on the page.
Classes one hundred four and five holding (the life in the past;
the gloss inline as "(Cold Water Fishy)" on the name's first appearance,
so the restore stood aside by design).
Every check as `CuspariaKLSY10`:
1 heading,
front matter equal,
no TA,
no error finish,
no double blank line.

Class one hundred eight,
found on slice 3:
the stand-in still reads "She took medication that night" where 服药 is nowhere in the original
(在劝慰朋友时发生矛盾，双方情绪爆发，醒来后又和其父亲发生争吵，后因尝试自杀送往医院抢救).
The repair lane softened the accepted critical addition instead of removing it;
the contest split 2 repair to 2 translate with one ballot calling the detail "from the original";
the consolidated proposal dropped it;
the gate heard 4 of 5 and split 2 to 2 (standing,
consolidated,
standing,
consolidated) and kept the standing,
the first standing ballot citing "dropped page content ... which the Chinese does not contradict",
the class eighty-five apparatus clause read over a text that stands in precisely because that detail was an accepted addition.
Nothing on any sheet said the slice was disputed or which claims had been accepted.

Built as class one hundred eight (guards red first `65d159d2a`,
built `85ed2881f`):
`archiveDisputeNote` (`archive-dispute.ts`) writes one block per disputed slice,
"ARCHIVE RENDERING DISPUTED:
the repair lane's adjudicators accepted N accuracy/addition claim(s) that the archive rendering says what the ORIGINAL never states;
the claims:
(1) ...;
(2) .... A detail those claims name is not page content and not the page's apparatus,
in the archive's wording or any softer one:
a candidate leaving it out has dropped nothing,
and a candidate keeping it carries an accepted addition.
Judge such a detail against the ORIGINAL alone.",
and every sheet that judges or writes against the stand-in carries it:
the consolidation writer,
slate judge and gate sheets (`consolidate-wire.ts`,
`consolidate-settle-context.ts`,
`consolidate-gate-wire.ts`),
the translate writer and slate judge sheets (`translate-wire.ts`,
`translate-judge.ts`,
threaded through `translate-slice-input.ts`,
`translate-stage.ts` and `translate-stage-repair.ts`),
and the lane contest sheet (`lane-contest-wire.ts`,
in the slice key through `lane-contest-key.ts`,
the notes read off the repair chunks in `corpus-run/pass-contest.ts` through `archiveDisputeNotesOf`).
`ArchiveDispute` now carries `acceptedClaims`.
The wording names no sheet's own incumbent because the lane contest shows the real archive while the consolidation and translate sheets show the stand-in.
Not built:
the repair lane's own sheets are unchanged,
so the lane may still soften an accepted addition rather than remove it;
the note reaches the contest and everything after it.
Lint 0 and 0,
types clean,
twelve touched suites green,
full suite green (`suite-class108.log`,
1113 PASS);
replayed offline over the real `CuspariaKLSY11` artifact:
slices 1 and 3 disputed,
the slice 3 note naming both accepted claims,
the gate sheet carrying it after CANDIDATE "standing".
Ninth ineligible-standing addendum in the docs commit.
Not exercised live yet.

`CuspariaKLSY12` (frozen `85ed2881f`,
pid 3947732,
overlap 8)
launched 02:17 UTC on Bedrock and OpenRouter
(meters 48.46 USD Bedrock,
95.80 OpenRouter at launch),
waiter buax5c511,
the first pass on class one hundred eight.
Read slice 3 first:
the ARCHIVE RENDERING DISPUTED block on the contest,
slate and gate sheets in the artifact,
the death paragraph without "took medication",
`translate-archive-disputed` on slices 1 and 3 again;
then classes one hundred four,
five and seven,
beside every check of `CuspariaKLSY11`.
Then `zheermao101` (slice 9 for class one hundred six),
then `shi_Yumiaoya` or `mikaela_khara`.

## yingying8 read, 2026-09-24, 01:30 UTC: SETTLED in 5 min, every check as yingying7; then gpt-oss-120b culled from every role, every critic claim logged with its filers, class one hundred six built on the owner's answer, class one hundred seven next; no pass running

`yingying8` (frozen `d2e39e095`,
pid 3077650) SETTLED at 00:31 UTC 2026-09-24,
5 min after its 00:26 UTC launch,
one attempt,
310 seats,
13 retry rounds,
266 calls,
0.07 USD by the meters (Bedrock 48.57 to 48.50,
OpenRouter 95.93 to 95.88),
on Bedrock and OpenRouter alone (Synthetic's window spent,
Hyper dry);
no hold,
so class one hundred three not exercised.
Jev 12 of 12;
8 abstention lines with reasons.

Class one hundred four on the wire:
the polish gate ran once (4 of 7 usable ballots,
settled on neither,
the base shipped),
its ballots reading the house rules.
Class one hundred five silent by design (the archive carries no gloss line).
Classes eighty-six,
ninety-seven,
one hundred one and one hundred two holding,
every check as `yingying7`:
1 heading,
4 markers,
2 definitions,
front matter equal,
References line kept,
the [^2] definition "[Farewell.
I miss you,
Yingying.]",
the farewell "May you still have",
no TA,
no error finish,
no double blank line;
terminals slate-endorsed 0,
slate-declined 1,
gate-kept 2.
Observation,
not a class:
slice 1's repair text dropped 硬核视频 ("hardcore video") before "Geek Lab" and shipped unendorsed.

No pass is running:
the source changed four times after `yingying8` (the cull,
the claim-filer record,
class one hundred six),
and class one hundred seven is next before a freeze.

The owner's answers of 2026-09-24,
in order:

- On the `CuspariaKLSY10` slice 1 claim "omits 服用激素 (taking hormones) as the reason for transitioning" (the original writes 服用激素，成为跨性别的原因,
  hormones as the cause):
  "who is the model that said this?
  That particular model got cause and effect wrong.
  Cull it."
  then "cull it from every role".
    The sole proposer was `hf:openai/gpt-oss-120b`,
  the same seat that declined every candidate on `zheermao8` slice 9 for "quoted lines in present tense" on a slice with no quoted line.
    Off `RUN_ROSTER` and the static checker bench by the roster-change runbook,
  blocklisted with the owner's words,
  the checker seat passed on the recorded evidence,
  seating-decision addendum;
  commits through `bc10e3b70`,
  full suite green (`suite-cull2.log`,
  1106 PASS).
- "we're not logging enough then.
  Refine how we log."
  (the reader had to join `sliceCritics[].claimAttributions` by claim id to learn who filed the claim):
    `claim-filers.ts` logs every resolved claim at the critic stage with its filers,
  category,
  severity and summary (`chunk N: claim <id> filed by <models>: ...`),
  every adjudicated issue with its status and each member claim's filers,
  and attaches `filedBy` to the artifact's adjudicated issues beside the claims;
    `SLICE_CACHE_VERSION` 31 (telemetry only);
    guard red first `f049298ac`,
  built `9a6327de5`,
  pins `aa39dcf1c` and `8e2c52ef5`,
  full suite green (`suite-filers2.log`,
  1107 PASS).
- Q1,
  a tied slate over an eligible standing:
  "Run-off only when every contest ballot called the archive flawed."
    Class one hundred six:
  `archiveFlawedByAll` (`consolidate-archive-flawed.ts`) reads a contest settled on neither with every ballot's `archive` set to `flawed`;
    `consolidate-driver.ts` raises `runoffOverStanding` from it,
  `settleConsolidation` then routes the slate through `judgeSlateWithRetry` (the same panel,
  narrowing finalists) as over an ineligible standing;
    the judge stage names the run-off finalists on a declined round even over an incumbent (`runoffFinalists` on `TranslateStageResult`) and the retry loop narrows and reruns on such a decline;
    an undecided run-off still ships the standing (`slate-declined-standing`,
  never a stop).
    Guards red first `200bf7a53` (`consolidate-archive-flawed.unit.test.ts`;
  `consolidate-settle.unit.test.ts`:
  the run-off chooses over an eligible standing,
  a run-off tied throughout ships the standing),
  built `2a872bbed`,
  lint 0 and 0,
  types clean,
  full suite green (`suite-class106.log`,
  1108 PASS);
    seventh ineligible-standing addendum `7c729d339`.
    Not exercised live yet:
  read the next `zheermao101` for a `translate-declined-retried` finding on a consolidation slice after a contest with the archive flawed on every ballot.
- Q2,
  an archive rendering carrying an accepted fabrication claim:
  "Not eligible;
  fall back to the repair text."
    Class one hundred seven,
  next:
  a slice whose repair `ChunkRepairOutcome.issues` carry an accepted `accuracy/addition` claim makes the archive rendering ineligible for the translate slate (a candidate and the fallback no longer),
  the slate runs the run-off path,
  and where nobody wins the repair lane's text for that slice (which removed the addition) stands instead of the archive (`CuspariaKLSY10` slice 3:
  four accepted claims that the overdose detail was invented,
  the translate slate backed nobody,
  the archive stood).
    Guards red first,
  then an addendum;
  then freeze,
  launch `CuspariaKLSY` to read it live,
  then `zheermao101` for class one hundred six.

## zheermao8 read, 2026-09-24, 00:30 UTC: SETTLED in 14 min, the polish gate reading the house rules on seven slices (class one hundred four), every check as zheermao7; the reply label kept the archive's "one member" on a tied contest and a tied slate with both lane texts on offer; yingying8 launched

`zheermao8` (frozen `d2e39e095`,
pid 3041616,
entry `zheermao101`) SETTLED at 00:23 UTC 2026-09-24,
14 min after its 00:09 UTC launch (TALLY `ms=831400`),
one attempt,
859 seats,
33 retry rounds,
716 calls,
19 abandoned streams (0.05 USD;
12 minimax-m3,
7 deepseek-v4.1-flash),
0.25 USD by the meters (Bedrock 48.84 to 48.59,
OpenRouter 96.19 to 95.95),
on Bedrock and OpenRouter alone (Synthetic's window spent,
Hyper dry);
no hold,
so class one hundred three not exercised.
Grace 0.9 s mean over 210 rounds;
Jev 25 of 25;
15 abstention lines with reasons.

Class one hundred four on the wire:
the polish gate ran on seven slices (base 1,
polished 4,
neither 2),
its ballots reading the house rules ("hold the same tense",
"without changing the meaning or violating any house rules");
no ballot called a tense a change of meaning.
Class one hundred five silent by design (the archive carries no gloss line).
Classes seventy-seven,
ninety-six,
one hundred one and one hundred two holding,
every check as `zheermao7`:
2 headings,
front matter equal,
no TA,
no error finish,
no double blank line,
both labels one line as the archive ("English translation of the preceding letter:",
"English translation of the preceding email conversation:"),
the email conversation as the archive,
26 curly apostrophes and 6 curly double quotes against the archive's 27 and 6,
six three-dot ellipses against seven (the bench's line);
terminals consolidated (1,
4,
12),
gate-kept (3,
11),
slate-declined (0,
5,
6,
9),
14 slices,
9 contested.
The read script's one "refusal" hit is the archive's own letter line "Although I cannot comprehend what you've been through",
not a model refusal.
The One Among Us reply's untranslated JSX comment (可以避开黑暗的锋芒…) rendered into the reply this run ("You can avoid the sharpest edges of the darkness and find refuge where there is light.")
where `zheermao7` kept the comment;
the archive block review's call on a picture block,
as on `zheermao3` and `4`.

Observation,
not a class:
slice 9 (the reply label) shipped the archive's "Upon seeing it,
one member sent her a reply immediately:"
for 那些秋叶的成员在看到后连夜给她发送了一份回信 (plural members,
that same night),
which `zheermao7` rendered "When the members saw it,
they stayed up that night to send her a reply:".
Both lanes corrected it (repair "When the members saw it,
they stayed up through the night",
translate "the members of One Among Us sent her a reply that same night"),
the contest tied 2 to 2 with every ballot calling the archive flawed,
so the archive stood;
the class forty offer put both lane texts on the consolidation slate ("lane texts offered on the slate beside the proposals:
repair,
translate"),
the slate split 1/1/1 with gpt-oss declining all on a misread ("the quoted lines in present tense" on a slice with no quoted line),
and the eligible standing kept its single round.
This is the second slice tonight (with `CuspariaKLSY10`'s death paragraph) where an archive defect every judge named shipped because the slate over an eligible standing tied at its single round;
the census over the last six read runs finds tied contests with the archive flawed on every ballot on 5 slices (`XingZ631` 3,
`XingZ628` 1,
`zheermao8` 1),
one of them shipped unendorsed here.
Open question for the owner,
beside the `CuspariaKLSY10` one:
whether the class eighty-two run-off (the same panel again while the finalists narrow) should also run over an eligible standing when the slate ties or falls short,
so a tie among texts that all correct the archive is decided rather than defaulted;
that changes the "an eligible standing keeps its single round" addendum,
so it is not built.

`yingying8` (frozen `d2e39e095`,
pid 3077650,
overlap 8)
launched 00:26 UTC on Bedrock and OpenRouter
(meters 48.57 USD Bedrock,
95.93 OpenRouter at launch),
waiter bm2t3byvd.
Read for the polish gate under the house rules,
class one hundred five silent by design,
classes eighty-six,
ninety-seven,
one hundred one and one hundred two holding,
beside every check of `yingying7` (1 heading,
4 markers,
2 definitions,
front matter equal,
References line kept,
"Farewell.
I miss you,
Yingying.",
"May you still have",
no TA,
no error finish,
no double blank line).

## CuspariaKLSY10 read, 2026-09-24, 00:15 UTC: SETTLED in 7 min, classes one hundred four and five live (the life in the past past the polish gate, the gloss line restored at page assembly); the death paragraph shipped the repair lane's minimal edit with the archive's own omissions, unendorsed; zheermao8 launched

`CuspariaKLSY10` (frozen `d2e39e095`,
pid 3035347) SETTLED at 00:06 UTC 2026-09-24,
7 min after its 23:59 UTC launch (TALLY `ms=425863`),
one attempt,
375 seats,
12 retry rounds,
320 calls,
0.18 USD by the meters (Bedrock 48.96 to 48.85,
OpenRouter 96.27 to 96.20),
on Bedrock and OpenRouter alone (Synthetic's window spent,
Hyper dry);
no hold,
so class one hundred three not exercised again.
Grace 1.4 s mean over 92 rounds;
Jev 13 of 13;
13 abstention lines with reasons.

Class one hundred four live.
Slice 0 (the life) ships every line in the past tense;
the consolidation polish gate on slice 0 settled on base 4 of 5 with a ballot citing "the past tense house rule for the life of the deceased" and no ballot calling the tense a change of meaning (the polished candidate lost on wording,
"deeply devoted" for 执着,
not on tense);
the slice 1 gate chose the polished text 5 of 7 with "without altering meaning,
tense,
pronouns,
or structure".
Class one hundred five live.
The bench dropped the gloss line again (the shipped slice 1 read "She coined her nickname “Ling Shui Yu Yu Zi” while feeding fish."
alone),
the page assembly printed `name-gloss-restored (slice 1: "“Ling Shui Yu Yu Zi” means fish in clear water.")`,
and the page carries the archive's gloss line after the nickname line.
Classes seventy-five,
seventy-six,
eighty-five and ninety-six holding:
1 heading,
front matter equal,
no TA,
no error finish,
no double blank line,
three curly apostrophes and six curly double quotes as the archive,
the one full-width colon the archive's own contributor comment;
terminals consolidated (0,
1),
slate-endorsed (2),
slate-declined (3);
the absolute naturalness review 3 of 8 usable on its first round (quorum-not-met) and 4 of 8 on the second.

Observation,
not a class:
the death paragraph (slice 3) shipped as the repair lane's minimal edit of the archive:
the fabricated "She swallowed lots of her meds that night" removed (four accepted claims) and "dead" corrected to "brain-dead" (two accepted claims),
but the archive's own omissions kept (双方情绪爆发,
抢救,
又,
再次得到消息,
and 于 rendered "by August 1st" for "on"),
which `CuspariaKLSY8` and `9` had rendered from fresh translate candidates.
No critic filed the emotions clause as an omission;
the "rescue" claim went needs-human and the "two events collapsed" claim was rejected.
The translate lane's slate over six candidates (four ballots,
one abstention,
one self-vote) backed none at the minimum,
so the archive stood as the translate text by the eligible-standing single round;
the contest chose repair 3 of 4 ("omits several phrases but avoids inventing methods");
the consolidation slate tied 1/1/0.5 across three proposals and kept the standing (eligible,
single round).
An open question for the owner,
not built:
whether an archive rendering the repair lane's adjudicators found to fabricate a suicide method is still an eligible standing for the translate slate's single round,
or whether an accepted fabrication should send the slate to the class eighty-two run-off.
Smaller observations:
"Taking hormones,
the reason she became transgender:"
for 服用激素，成为跨性别的原因： (a literal rendering where `CuspariaKLSY8` wrote "Reasons for taking hormones and transitioning:"),
the bench's call.

`zheermao8` (frozen `d2e39e095`,
pid 3041616,
entry `zheermao101`,
overlap 8)
launched 00:09 UTC on Bedrock and OpenRouter
(meters 48.84 USD Bedrock,
96.19 OpenRouter at launch),
waiter binlnnzie.
Read for the polish gate ballots under the house rules (class one hundred four on an entry with gate-kept slices),
class one hundred five silent by design (no gloss line in the archive),
classes seventy-seven,
ninety-six,
one hundred one and one hundred two holding,
beside every check of `zheermao7` (2 headings,
front matter equal,
no TA,
no error finish,
no double blank line,
both labels one line,
the email conversation as the archive).
