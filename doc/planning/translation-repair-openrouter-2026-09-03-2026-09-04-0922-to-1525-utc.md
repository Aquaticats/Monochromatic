# OpenRouter and the corpus passes: 2026-09-04, 09:22 to 15:25 UTC

What was launched,
read and decided on 2026-09-04 09:22 to 15:25 UTC.
Entries:
luxuanwen3.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## The luxuanwen3 re-run, 2026-09-04, 09:22 UTC onward: slice 0 passes, slice 1 stops the entry

Launched on `592562992` (alias containment,
withheld standing,
endpoint ignores),
OpenRouter alone,
log `~/temp/agent/luxuanwen3-shapes2-20260904.log`,
runs dir `~/temp/agent/luxuanwen3-shapes2-20260904`.

- Slice 0,
  the front matter (56 source characters):
  both lanes `exit=computed`,
  no eligibility warning at the lane contest or the consolidation.
    The archive's `alias: 鲵鲵, Nini` passes the containment rule.
- Slice 1,
  the description paragraph (95 source characters):
  the translate lane's winner "fails publication invariants" at the contest,
    the standing "fails publication eligibility" at the consolidation,
    and at 09:56:43 UTC,
    34 minutes in,
    `ConsolidationStandingIneligibleError: slice 1 ... (incumbent-only)` stopped the entry.
    The pass of the morning had run 63 minutes before the page guard refused it.
- The cause,
  read from the slice records (`slice-cache/luxuanwen3/translate.*.json`,
  slice 1 findings):
  the original links `https://twitter.com/Deaver1229`,
    the archive's page `https://x.com/Deaver1229`,
    and `mergeAtoms` in `translate-atom-floor.ts` (commit `3ca134e57`:
    "protected atoms take the larger demand of the two references") owes the per-key maximum of both,
    so a candidate carrying either URL is told it lacks the other.
    The four translators asked to repair were told so in both directions:
    two restored the original's URL,
    two kept the page's,
    and every proposal failed the floor.
    The standing,
    the archive's own paragraph,
    fails the same rule,
    so the slate had nothing and the terminal was `incumbent-only`.
- The log could not say this:
  neither warning named the rule or the findings.
  Both drivers now print the deterministic findings,
    and the consolidation warning distinguishes an invalid standing from an unendorsed one.

Census of the pinned corpus (`~/temp/agent/atom-census-20260904.mjs` over `readSliceSkeleton` on whole pages,
`~/temp/agent/link-census-20260904.mjs` over a plain link scan):
of the 70 pages the strict grammar reads whole,
7 carry a link the archive rewrote,
meaning `link-url` diverges in both directions:
MizuharaNagisa (`www.` added),
Rentable_A (trailing slash dropped),
SS3B_0016 and shihai4h (Chinese Wikipedia to English Wikipedia),
aiyysk (`google.cn` to `.com`),
homoyamakaze (an archived Twitter capture to `mtf.wiki`),
luxuanwen3 (`twitter.com` to `x.com`).
XIEPT2,
whose page the whole-page grammar refuses,
is the eighth by the plain scan.
Footnotes diverge in one direction only (5 pages add one,
1 drops one),
never both.
Under the per-key maximum none of the eight can ship:
the archive's paragraph is ineligible,
and no rendering carries both destinations without inventing a link.

Tally:
`TALLY luxuanwen3 status=ERROR ms=2724158` at 10:08 UTC,
587 OpenRouter calls,
2.01 USD by the log's `cost=` fields,
balance 25.79 USD after.
The in-flight consolidation buys for slices 2 to 4 ran on for 11 minutes after the slice 1 error before the tally;
the stop is at the slice,
not the process.

What the tally exposed next:
`entryErrorOutcome` classed `ConsolidationStandingIneligibleError` as a resumable failure,
so the pass logged `REATTEMPT luxuanwen3 queued` and started the lanes again at
10:08:12 UTC against the same deterministic refusal;
it was stopped by hand at 10:09 UTC and the error joined the stopped list (`ae1d2b55f`,
guard shown to fail without it).
The log now names both verdicts and their findings (`558b46e11`).

Question put to the owner:
which rendering a candidate owes where the archive rewrote a destination.

## The luxuanwen3 re-run on the either-rendering build, 2026-09-04: every slice passes, the publisher refuses

Launched at 12:43:50 UTC on `82888d43b` (the rendering pools) plus the docs commit,
OpenRouter alone,
into `~/temp/agent/luxuanwen3-shapes3-20260904` (log of the same name).
Balance 22.53 USD at the SS3B_0016 launch that followed at 13:20 UTC.

What the slices did:
slice 1,
the paragraph whose link the archive moved from `twitter.com` to `x.com`,
passed the deterministic rule in every lane.
The translate lane's winner carried the original's `twitter.com` once and `x.com` never;
three of its translators matched the incumbent.
The contest recorded eight ballots across the slices (four `repair`,
two `translate`,
two `neither`,
five to seven usable each).
Consolidation started at 13:18:47 UTC on all nine slices;
slice 1 exited `computed` in 536 s with no verdict warning;
slices 2 and 6 logged `consolidation standing text lacks contest endorsement and remains retryable`,
and slice 2's single attempt kept the standing (`slate-declined-standing`) with the finding recorded.
No `must carry exactly`,
no `fails the deterministic publication rule`,
no `REATTEMPT`.

Tally:
`TALLY luxuanwen3 status=INCOMPLETE ms=3365280 aborted=false error=entry luxuanwen3 would drop 1
source destination(s)` at 13:39:55 UTC,
56 minutes,
765 calls,
4.24 USD by the `cost=` fields,
balance 19.26 USD at the last meter reading.
Seats:
gpt-oss-120b 120 of 120 usable,
minimax-m3 117 of 117,
gemma 126 of 126,
deepseek-v4-pro 121 of 122,
GLM-5.3-Flash 35 of 39,
deepseek-v4-flash 82 of 94 (12 threw,
the cut-mid-reply shape of the morning),
Qwen3.8-27B 107 of 127 (20 threw),
glm-5.3 57 of 60.

What refused it:
`DroppedDestinationError` in `publish-fixed.ts`,
the document-level check,
which compared the would-ship page's destinations against the source's alone.
The page carried the archive's `x.com` where the source carries `twitter.com`,
which the slice rule had accepted and the publisher counted as one source destination dropped.
The decision record of the morning had said the publisher was unchanged;
that clause was the defect,
and it is struck.
Fixed in `1c9663666` (`corpus-run/destination-renderings.ts`,
the same pool over the whole page;
findings `destinations-archive-rendering` and `destinations-both-renderings` on the `DESTINATIONS` line),
guards shown to fail with the archive side of the pool neutralised,
and `8d7b151ef` (lint).
Two tests of the morning (`consolidate-ineligible-standing`,
`consolidate-standing-verdict`) still asserted the maximum rule through a candidate keeping the page's link;
the full suite had last run before `82888d43b`.
Moved to a candidate carrying neither rendering in `e66da50ef`;
suite 931 groups,
0 FAIL.

SS3B_0016 (the Wikipedia shape,
912 source characters) had been launched at 13:20 UTC on the same build;
it was stopped at 13:51 UTC during its lanes (422 calls,
1.84 USD) because its page could only reach the same refusal,
and relaunched with luxuanwen3 on `e66da50ef` at 13:51:31 UTC (`~/temp/agent/<id>-shapes4-20260904`),
balance 18.49 USD.

## The two runs on the publisher fix, 2026-09-04, 13:51 UTC onward: both ship, and what the reading found

luxuanwen3 and SS3B_0016 ran concurrently on `e66da50ef`,
OpenRouter alone,
from 13:51:31 UTC,
into `~/temp/agent/<id>-shapes4-20260904` (logs of the same name).
Balance 18.49 USD at launch,
5.41 USD at the last meter reading.

luxuanwen3:
`TALLY luxuanwen3 status=SETTLED slices=9 ... pageChanged=7 pageSilent=0 selection=contested ms=3596918` (60 minutes,
764 calls,
4.65 USD by the `cost=` fields),
`DESTINATIONS luxuanwen3 source=2 page=2 dropped=0` with no finding:
the page carries the original's `twitter.com` for Deaver1229 and for the contributor,
so the pool was met from the original's side.
verify-published:
`wordings=9 silent=0 chars=2467=expected missing=0`.
The front matter's `alias` reads `鲵鲵, Nini`;
the `PhotoScroll` line is intact;
the Camus quotation and its right-aligned attribution are intact.
Slice 3 logged `consolidation
standing text lacks contest endorsement and remains retryable` and shipped on the single attempt.
No `must carry exactly`,
no `fails the deterministic publication rule`,
no `REATTEMPT`.
Seats:
minimax-m3 117 of 117 usable,
gpt-oss-120b 115 of 115,
GLM-5.3-Flash 38 of 39,
glm-5.3 53 of 61 (8 threw),
deepseek-v4-flash 89 of 93,
deepseek-v4-pro 120 of 123,
gemma 123 of 124,
Qwen3.8-27B 109 of 125 (16 threw).
Read in full:
the page is publishable as it stands.
One wrap nit,
not a page defect:
the per-slice wrap broke `August 4,
2024` after the date's comma,
which Markdown rejoins on render.

SS3B_0016:
`TALLY SS3B_0016 status=SETTLED slices=9 ... pageChanged=8 pageSilent=0 selection=contested ms=3365773` (56 minutes,
887 calls,
5.12 USD),
`DESTINATIONS SS3B_0016 source=3 page=3 dropped=0 destinations-archive-rendering`:
the page carries the archive's English Wikipedia `Railfan` where the source links the Chinese article,
and keeps the `mailto:` and GitHub destinations.
verify-published:
`wordings=9 silent=0 chars=2291=expected missing=0`.
Slice 2 lacked contest endorsement and shipped on the single attempt.
Seats:
gpt-oss-120b 141 of 141,
minimax-m3 132 of 132,
gemma 147 of 147,
glm-5.3 62 of 69 (7 threw),
deepseek-v4-pro 138 of 140,
Qwen3.8-27B 121 of 148 (27 threw),
GLM-5.3-Flash 39 of 40,
deepseek-v4-flash 107 of 111.
The page restores what the archive's rewrite had changed (the source's "community",
not "transgender community";
the source's "the day before Yantian's birthday",
not the archive's added date and stations) and reads well,
with two findings from the reading.

WHAT THE READING FOUND,
ONE DEFECT AND ONE RISK.

The defect:
slice 5 ships `we have temporarily set up a small room for Ta, to give Ta's memorial a little warmth`,
a bare `Ta` twice,
on a page that says "they" and "them" for Yantian everywhere else.
The
translate lane won that slice with three of five contest ballots reasoning that it "keeps the original's neutral Ta",
and the consolidation gate kept it.
The cause is in the pipeline,
not the judges:
the source writes its neutral pronoun as `Ta`,
and `countNeutralPronoun` in `identity-context.ts` counts only `TA`,
so the declared-identity pronoun line said nothing for this page;
and the house rule names TA as a pronoun the original uses without saying how English renders it.
Measured over the pinned corpus:
sources write the neutral pronoun as `TA` in 2 entries,
`Ta` in 7 and `ta` in 8 (every occurrence a pronoun,
`TA 们` the plural in 6);
of the archives of those entries,
one (XingZ60,
a rewrite) keeps a bare `TA`,
and the rest render it "they" (Uekawakuyuurei 22 "they" to 0 "she",
Hangmster 18 to 0).
Fixed in the next commit:
the counter reads all three spellings,
the house rule states the English rendering,
and the deterministic floor refuses a translation carrying the pronoun untranslated.

The risk:
slice 5 also renders 「那些秋叶」 as `「One Among Us」`.
The name is right (the archives of all five entries naming 那些秋叶 render it "One Among Us",
one as "One Among Us Transgender Support") but the
gate kept it 4 to 3 over a consolidated candidate reading "Those Autumn Leaves",
the judges arguing the rendering from the archive alone.
The corner brackets around an English name are the source's punctuation carried across;
72 sources use them and 3 archives keep any,
all in quoted Chinese or a design element.
FIXED THE SAME AFTERNOON in `d5e4d1ea0`,
since both halves are house rules rather than judgement calls:
the community-vocabulary rule now names 那些秋叶 and its renderings (members,
the maintenance group),
and the punctuation rule says what corner brackets become,
quotation marks around a quotation and nothing around a name or a term.
Guarded by two cases in `house-policy-reaches-the-judges.unit.test.ts`,
shown to fail with the sentences removed.

WHAT LANDED FOR THE PRONOUN:
`d98e656cb` (counter,
house rule,
`translate-neutral-pronoun.ts` wired into `validateTranslatedSlice`,
tests;
each guard shown to fail with its rule neutralised),
`cd5288fa9` (lint shapes),
`dd132bc76` (README).
Measured over the pinned corpus with the built finding:
one archive flagged (XingZ60,
`TA` 3 times) and 15 sources.
The next pass launched at 15:03:34 UTC on `cd5288fa9` (pipeline `sha256-tree-v1:946ab54f`),
OpenRouter alone:
Uekawakuyuurei,
whose subject is `ta` throughout (12 `ta` and 1 `Ta`,
archive 22 "they" to 0 "she"),
into `~/temp/agent/Uekawakuyuurei-pronoun-20260904`.
Balance 5.32 USD at launch,
which is about one pass.

## The Uekawakuyuurei run on the pronoun build, 2026-09-04, 15:03 UTC: stopped by its own pictures

`TALLY Uekawakuyuurei status=INCOMPLETE ms=111956 aborted=false error=visual evidence incomplete for 3
referenced assets` at 15:05 UTC,
33 calls,
0.08 USD.
The entry shows eight pictures;
five carry nothing past the deterministic reader and were never sent.
The other three are an oil painting of ships (`IMG_1308.webp`,
24 characters of canvas noise from tesseract),
an ink drawing of two cats (`img197.webp`,
18) and an ink drawing of a destroyer whose only text is the hull number (`img370.webp`,
24).
Three readers each asked four times;
every reply either reported that the picture carries no text or transcribed `DE581`,
and the screen called the first a refusal and the second too short,
so all three pictures ended `no-reader-available`,
transient,
and the entry stopped at the completeness gate.
Across every run log this session,
56 entry-and-picture pairs corroborated and 8 failed,
on 5 pictures:
these three on both rosters,
`dogesir_/photo2.webp` on both,
`gqt/photo3.webp` at one reader of two.

A probe of the three OpenRouter readers over those five pictures (15 calls) put the wording on record:
14 refused replies name text ("There is no visible text in this image",
"I cannot read any text in this image.
There are no visible words",
"I can read the image,
but there is no visible text to transcribe"),
one declines without naming it ("I cannot read the image."),
and the destroyer draws `DE581`,
`DE581` and `D650`.

WHAT LANDED:
`20ea56cd0` logs the opening of a refused reply;
`d6ffc4812` splits absence from inability (`refusalReportsAbsence` in `reading-refusal.ts`:
names text,
no quality or access marker),
gives the screen three verdicts (usable,
short,
refused with clause `too-short`,
`reads-as-refusal` or `reports-no-text`),
stops re-asking an absence report,
and lets two readers reporting absence or answering short confirm a
picture textless in `readImagePair` (`no-text` with `confirmedBy`,
resumed).
Each guard shown to fail with its rule neutralised (refusal 2,
sense 3,
stage 3,
pair 5,
re-ask 2 failures),
then pass.
Rule recorded in `doc/planning/when-an-image-reading-makes-no-sense.md` (clause six) and the README.

Relaunched at 15:25 UTC on `d6ffc4812` into `~/temp/agent/Uekawakuyuurei-pictures-20260904`,
OpenRouter alone,
to read both today's fixes on one entry:
the pronoun rule on a `ta` subject and the textless confirmation on three pictures.
Balance about 5.3 USD at launch,
which is one pass.

## Uekawakuyuurei on the picture-fix build, 2026-09-04, 15:25 UTC: ships, and the balance runs out under it

`TALLY Uekawakuyuurei status=SETTLED slices=7 ... pageChanged=5 pageSilent=0 selection=contested ms=3190452`
(53 minutes,
524 calls,
3.84 USD by the `cost=` fields),
`DESTINATIONS Uekawakuyuurei source=4 page=4 dropped=0`;
verify-published `wordings=7 silent=0 chars=2774=expected missing=0`.
Tip `d6ffc4812`,
pipeline `sha256-tree-v1:646ad738`.

BOTH FIXES READ CLEAN ON THE PAGE.
Pictures:
`IMG_1308.webp` confirmed textless by 3 of 3 readers on the first ask,
`img197.webp` by 2 of 3,
`img370.webp` by 3 of 3 (all three short readings of the hull number),
no re-asks,
no visual-evidence stop;
the `PhotoScroll` and `ChannelBackupButton` lines ship as written.
Pronouns:
the page says they,
them or their 21 times for Ying,
he or she never,
and carries no bare `ta`;
the source writes `ta` 12 times and `Ta` once.
No `untranslated as` finding was raised in any lane,
so the house rule alone carried it.

THE FRONT MATTER went as the guard decision of 2026-09-02 provides.
The translate lane rendered `name: Hotaru`,
`alias: Ah Hotaru, Hotaru-chan, Akigumo`,
`location: China`;
the archive has no `location`,
so the shape rule excluded that candidate from the contest (`lane-contest-eligibility-floor`),
the standing lacked endorsement,
and the single consolidation attempt kept the archive's `name: uekawakuyuurei`,
`alias: Ying (Hotaru), Qiu Yun (Akigumo)`.
The contest's log line named the floor and not the finding,
which a later change corrected:
the line now names each excluded lane and its finding (`lane-contest-eligibility.ts`).

THE BALANCE RAN OUT UNDER CONSOLIDATION.
The lanes finished at 16:06 UTC with 1.98 USD left;
consolidation began at 16:11 UTC;
from 16:15:00 UTC OpenRouter refused calls while the meter still read 1.12 USD (the last reading before the refusal),
and `EveryProviderDryError` was raised 166 times through the gate and refiner rounds,
205 voices lost,
with the meter reading `openrouter=wet` and falling to 0.36 USD as smaller calls kept buying.
Every consolidation slice shipped its standing,
the lane-contest winner,
unpolished.
Seats:
minimax-m3 82 of 117 usable,
GLM-5.3-Flash 26 of 50,
gemma 93 of 106,
glm-5.3 27 of 45,
gpt-oss-120b 85 of 99,
deepseek-v4-flash 59 of 83,
Qwen3.8-27B 69 of 106,
deepseek-v4-pro 84 of 122,
the shortfall being the refusals.
This is the first live exercise of the every-provider-dry path:
the run did not hold and did not error;
it settled on what it had.

THE READING OF THE PAGE:
publishable.
Two notes,
neither a class.
The diagnosis carries italics the source does not (`*signet ring cell carcinoma of the ascending colon*`);
51 of 92 archives use italics,
for work titles,
so emphasis is not a marker the floor can refuse.
The quotation "just an unhealthy-looking boy in real life.",
carries the archive's own period-inside-quotes shape,
which the archive ends with a second period.
"Here's a selected few" is the archive's wording kept.

## The picture gather's wait, re-read on three-reader pictures, 2026-09-04

The open item since 2026-08-29 was to re-read the gather's straggler wait once passes had run pictures
with three readers rather than two.
Today's runs did,
so this is that reading.
It is a measurement and a recommendation,
not a change.

THE GATHER HAS NO STRAGGLER WINDOW.
`readImagePair` calls every reader through one `Promise.allSettled` and waits for all of them,
bounded only by each call's own timeout.
`STRAGGLER_GRACE_MS`,
120 seconds after quorum,
governs the judging rounds in `stage-round.ts` and never reaches the picture gather.

WHAT THE SPREAD COSTS NORMALLY.
Over every asset in today's logs that drew two or more reader replies,
20 of them,
the gap between the first reply and the last runs a median of 11.6 seconds,
a ninetieth percentile of 45 seconds,
and a maximum of 66 seconds.
The two clean three-reader assets of the picture-fix build spread 9.2 and 38.4 seconds.
A whole picture stage costs 66 seconds for the eight assets of Uekawakuyuurei and 95 seconds for the three of Toka_ls.
At those numbers a window would save nothing worth having.

WHAT IT COSTS WHEN A READER HANGS.
The BI4PBV pass of 04:43 UTC spent 367 seconds on four pictures,
almost all of it on `image1.webp`:
two readers answered and corroborated at 0.614 overlap,
and the gather then waited about six minutes for `hf:zai-org/GLM-5.3-Flash`,
which ended in `StreamCutShortError` and contributed nothing.
Corroboration needs two readings,
so those six minutes bought a verdict that was already settled.
The apparent 2707-second stage in the luxuanwen3 pass of 09:23 UTC is not a second case:
that log holds two picture stages,
a 13-second gather and a resume from cache 45 minutes later.

THE OPTION,
for the owner rather than for me.
A straggler cut in the gather,
once two readings are in hand,
would bound that case at the cut and change nothing else,
since the median spread is a tenth of any sensible window.
It costs the third reading on a slow-but-alive reader,
which is evidence the corroboration rule does not use.
Against it:
the gather is the one stage whose output nothing requires,
so its wall time is only ever a delay,
and one six-minute delay per corpus pass may not be worth another dial.
Recorded as measured;
no dial added.
