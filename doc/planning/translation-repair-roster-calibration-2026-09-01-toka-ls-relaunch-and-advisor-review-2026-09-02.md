# Roster calibration: the Toka_ls relaunch and the advisor review, 2026-09-02 from 04:00 to 11:00 UTC

The next-action status of that morning,
with the Toka_ls relaunch killed in consolidation and its rerun that ended incomplete on the naturalness floor;
the advisor review at 09:30 UTC with its five fixes and six guard-failure proofs.

Part of [Roster calibration under the owner blocklist, 2026-09-01](translation-repair-roster-calibration-2026-09-01.md),
which lists every file of this record.

## Next action

Nothing is running.
The Toka_ls relaunch was killed at 04:02:54 UTC (00:02:54 local,
three minutes past
midnight,
recorded as a correlation and not a cause) and Carena0442 never launched.
Measured across the
Toka_ls run's 37 meter readings:
Synthetic's weekly fell from 5.24% at launch (02:46) to 3.16% at 03:32 and
2.0% at 04:02,
2.7 points an hour under one entry at overlap 2,
so it is gone within the half hour.

In order:

1.  DONE at 04:43 UTC as commit `6f70a2085`,
    restructured contest-first as `1160ebb4c` at 05:20 after
    the sol review:
    the guard reads the stage that shipped the metadata (see the Toka_ls section).
    Any
    rerun must be on `1160ebb4c` or later.
2.  DONE at 05:05 UTC:
    `doc/planning/translation-repair-toka-ls-reading-2026-09-02.md` reads every
    Toka_ls slice against its source off the cache (preparation rebuilt from the cached pairings).
    Findings
    a reader would not pass:
    a neutral pronoun at slice 9 for a person the page calls "she" (eight of
    eight ballots,
    on purpose,
    because the Chinese sentence has no subject),
    a re-rendered work title
    at slice 10,
    an unidiomatic heading at slice 12,
    and `> ` trailing-space churn copied from the source,
    including a whitespace-only "replacement" judged five to two at slice 1.
3.  Toka_ls is running again since 08:40 UTC on `8e3171b34` (writer 180 s,
    round 60 s,
    overlap 2),
    launched
    on the owner's word that one dry provider is normal operation (rule `QPW`);
    it predates the guard
    reduction,
    the judge unseating and the #473,
    #474 and pronoun fixes,
    so its metadata is guarded by the
    reshaped rule and its judges include GLM-5.3-Flash.
    Its slice 9 will still read "they":
    it is the
    baseline for Carena's slice-level pronouns,
    not a regression.
    When it exits,
    a chain (requeued at 09:45
    UTC after the advisor review's fixes,
    see "Advisor review at 09:30 UTC") launches Carena0442 (fixture
    clone,
    the clone variables passed to its `verify-published` too),
    then keyword233,
    one per launch on the
    tree at that moment,
    each followed by `verify-published`;
    logs `~/temp/agent/carena-rerun-20260902.log`,
    `~/temp/agent/pin-rerun-keyword233-20260902.log`.
    XIEPT2 is held back under `FIT` (measured below).
    Carena is the first live run of the unseated roster,
    the structural guard,
    the 429 backoff,
    the
    recovery nudge and the pronoun line at once,
    so its first two minutes are read for the roster banner
    and any assertion before anything else.
    As it stands at 10:55 UTC:
    Toka_ls ended INCOMPLETE (see
    "The Toka_ls rerun ended INCOMPLETE at 117 minutes"),
    Carena0442 runs since 10:38:50 on
    `3852e4d86`,
    keyword233 follows it,
    and Toka_ls is queued after both on the tree with `321b12673`.
    The chain builds at each launch (the corpus-pass task
    depends on build),
    so Carena and keyword233 will run on different commits by design:
    the
    work-title rule (task 21) and the post-launch batch (task 22) land as whole commits inside Carena's
    run,
    the tree clean between them and before Carena's TALLY line;
    the chain echoes each launch's
    short hash.
    Each landed page and artifact gets the task 3 reading (`~/temp/agent/read-artifact-20260902.mjs`),
    with slice 9's pronoun and the metadata slice's outcome checked first.
    Each landed artifact goes through
`verify-published` (which reads `artifacts/` and `fixed/` and never touches `pass.lock`) and the reader
script (`~/temp/agent/read-artifact-20260902.mjs`) for task 3,
    in this order:
    `git status --short` on the
real clone,
    `verify-published`,
    the reader script,
    then the translate-round heard counts under the writer
dial against the first run's two Qwen cuts at 60 s,
    which is the dial's first live measurement.

## The Toka_ls relaunch was killed at 77 minutes, in consolidation

At 04:02:54 UTC both background tasks of this session,
the Toka_ls launcher and the queued chain,
were
reported killed at the same moment.
I did not stop them;
the pass process died with the launcher's process
group (`sh exited with non-zero status: no exit status`,
`Finished in 4623.56s`).
No TALLY,
no artifact,
no
page.
The chain never reached the meter gate,
so Carena0442 did not launch.
The cause is outside this
session's tool calls and I cannot name it from here.

Where it was:
repair,
refine and translate lanes settled (14,
15 and 16 cache records),
lane contest 15 of
16 slices (15 differ),
consolidation 2 of 16,
killed inside `settleConsolidation` with 0 holds and 0 429s.
The cache is keyed by a digest over `dist/final/node` (`corpus-run/pipeline-digest.ts`),
and the guard
commits changed those bytes,
so a rerun on the current build starts from zero:
about 77 minutes at
overlap 2 to the same point,
longer to a page.
Synthetic's weekly read 2.0% at 04:02,
draining at 2.7
points an hour,
so a rerun launched now runs Hyper-only inside the hour,
the #474 shape.
Not relaunched.

What the run measured before it died,
all off `~/temp/agent/pin-relaunch-Toka_ls-20260902.log`:

- Writer rounds under the 180 s writer dial were full:
  editor 36/36 over 12 rounds,
  refiner 15/15 over 5,
  translate 110/112 over 16,
  consolidation refiner 6/6.
  One writer cut in the whole run (GLM-5.3-Flash at
  180 s).
  The dial did what it was added for.
- The 60 s round window cut GLM-5.3-Flash in 12 of 13 panel rounds,
  12 of 21 translate select rounds,
  11 of
  29 repair select rounds,
  11 of 15 critic rounds and 5 of 15 contest rounds:
  51 of the run's 78 cuts,
  every
  one at 60 s.
  Qwen3.8-27B took 15,
  Kimi-K3 6,
  minimax-m3 4.
  Every round reached quorum (0 quorum-unmet
  lines).
  Four rounds declined in the whole run:
  a repair select tie at weight 4 with all 8 heard and a
  contest "neither" with 9 of 9 usable,
  neither caused by a cut;
  and two translate select rounds that were
  cut-starved,
  a 6-of-8 tie at weight 2 and a 5-of-8 leader at 1.5 with one abstention,
  each decided in the
  challenge round that followed (8 of 8 heard,
  Kimi-K3 at 5.5;
  5 of 8 heard with three cuts,
  gemma at 4).
  One more round decided at weight 2.5 with GLM-5.3-Flash and minimax-m3 cut.
  So the cuts cost two
  challenge rounds and thinner margins,
  never a final decision.
  Whether GLM's ballot is worth two extra
  minutes per judge round (about 100 such rounds an entry) is the owner's speed-versus-completeness call,
  listed under decisions.
- The metadata slice (slice 0) is a live instance of the contest-keep case the reshaped guard leaves open.
  Source `name: 左橋瞳華 / alias: 瞳華 / location: 上海`;
  archive `name: Toka Sakyo / alias: Nonamev /
  location: Shanghai`.
  The translate lane's seven translators agreed on `alias: Toka` and the judges chose it
  6 of 8 ballots at weight 3.5 (`decision=judged origin=fresh changed=true`),
  the lane contest then ran,
  and
  the consolidation gate settled 9 of 9 on the standing text,
  which is the archive's `alias: Nonamev`
  (`terminal: gate-kept-standing`).
  Consolidation itself had chosen `alias: Toka` at weight 4.5;
  the
  fidelity gate reversed it 6 ballots to 2,
  every standing ballot reasoning that the declared names attest
  the alias as `Nonamev` and `Toka` is an unattested rendering,
  the two consolidated ballots reasoning that
  the source's alias is the given-name part of the name and `Toka` is that part of `Toka Sakyo`.
  That is a
  review of the incumbent by a full panel,
  with the house rule on both sides.
  The contest verdict for the
  slice,
  read off the record whose ballots argue about the alias (`contest.513b69ae`):
  the repair lane won
  with 9 of 9 usable ballots,
  "'translate' gives the alias as 'Toka' where the declared facts attest
  'Nonamev',
  so 'repair' matching all attested renderings is the faithful choice",
  which is the
  `lane-won repair` the contest-first walk needs and not an unendorsed `neither`.
  The final page would carry the archive's metadata byte for byte with a
  translate selection reading judged,
  fresh,
  shipped:
  the reshaped guard refuses that as
  `incumbent-fallback: replacement-not-carried`,
  the 2026-08-28 rule refused it too,
  and 503ec902c would
  have published it.
  An eight-ballot gate keeping the archive six to two is a review of it,
  so the
  standing now reads the stage that shipped the text,
  the way `wouldShipTextFor` walks consolidation,
  contest and lanes (commit `6f70a2085`,
  `corpus-run/front-matter-standing.ts`).
  On the record,
  both runs against the real Toka_ls
  cache records (translate selection,
  gate ballots `csscssss`,
  terminal `gate-kept-standing`):
  before,
  standing `replaced` and the guard refused `incumbent-fallback: replacement-not-carried`;
  after,
  standing
  `gate-keep` with 8 usable ballots and the guard accepted.
  The gate's ballots are re-settled with the
  stage's own `settleGateBallots`,
  so a gate that settled `neither` under the same terminal stays a
  fallback (`gate-neither`);
  a contest that chose the lane carrying the archive's bytes,
  or endorsed the
  archive when it chose neither,
  is `contest-keep`;
  a consolidation slate that endorsed the standing text is
  `slate-keep`;
  every other terminal or verdict is a fallback named after it.
  Recorded for veto as
  restoring the 2026-08-28 intent:
  reviewed metadata publishes,
  unreviewed refuses.
  The sol review of that commit (`~/temp/agent/sol-front-matter-standing-review-20260902.txt`) named two
  real gaps,
  fixed by commit `1160ebb4c`:
  the walk read consolidation first,
  so a contest that settled
  neither without endorsing the archive could be promoted to a reviewed keep by a slate endorsement or a
  quorum-backed gate,
  which contradicts `contestStandingMayShip` (the standing baseline may ship unchanged
  only when the contest chose a lane or endorsed the archive);
  and a contest's review of the archive was
  erased by consolidation terminals that merely left the standing text alone (`incumbent-only`,
  `slate-unjudged-standing`,
  `slate-declined-standing`,
  `wrap-erased-difference`,
  a gate that settled
  neither).
  The walk now reads the contest first,
  passes an unreviewed baseline through untouched,
  keeps a
  reviewed one through every transparent terminal,
  and adds a review only where the slate endorsed the
  standing text,
  the gate kept it with its quorum (two ballots,
  the stage's own),
  or the gate accepted a
  consolidated candidate carrying the archive's bytes;
  gate ballots that re-settle against their terminal
  are named `gate-ballots-contradict-terminal`.
  The Toka_ls records still read `gate-keep` with 8 usable
  ballots and the guard still accepts.
  Two review points stand as recorded rather than acted on:
  the
  classifier tests hand-assemble records (a lane-won verdict with no ballots) rather than driving the real
  settlers,
  and the equality relation is raw bytes at every step,
  which is also what the guard compares.
- Slice 15 is the first live matched-keep:
  `decision=sole-candidate origin=incumbent`,
  incumbent producer
  matched by all seven translators.
  The `matched` list is filled in practice.

## The Toka_ls rerun ended INCOMPLETE at 117 minutes, on a guard the no-loop design had superseded

The rerun of 08:39 to 10:37 UTC on `8e3171b34` exited 0 with `TALLY Toka_ls status=INCOMPLETE ms=7063121
error=slice 10 did not meet absolute naturalness floor`:
no page,
no artifact,
the real clone clean,
0
refusals and 0 holds in 7,063 seconds.
Seats:
GLM-5.3-Flash threw 78 of 211 (the 60 s window,
as
before),
Qwen3.8-27B 24 of 212,
every other seat 3 or fewer.

What happened to slice 10 (the 29-line letter in blockquote that follows the 《奇妙漂流》 sentence),
read
off the log and the source:

- The lane contest settled on neither lane without endorsing the archive,
  so the standing text lacked
  contest endorsement (`standingMayShip` false).
- Consolidation's single attempt kept the standing text (`slate-declined-standing`,
  judges tied at
  weight 3),
  and the no-loop rule shipped it "with the finding recorded"
  (`consolidation-standing-unendorsed`,
  `consolidate-slice-buy.ts`,
  design of 2026-09-01).
- Polish is not run over a baseline the fidelity gates never admitted (`not-run`,
  `unsafe-baseline`).
- At artifact time `assertFinalNaturalnessComplete` (commit `6fadd3be0`,
  2026-08-28,
  "require absolute
  naturalness") refused any body slice whose polish is not settled,
  and the whole entry was dropped.
  The same guard refused the `Weideriche_` first attempt after 4,840,305 ms on 2026-08-29 (recorded in
  the multi-provider decision doc).

Fixed by `321b12673`:
the guard accepts the no-loop record (`not-run` with reason `unsafe-baseline`)
and still refuses every other body slice without a settled polish;
the artifact carries the reason and
the reading catches it.
The test fails with the acceptance removed (2 FAIL lines) and passes restored.
Recorded for veto as the later stated policy winning over the earlier guard;
the alternative,
refusing
the entry,
is what fast iteration cannot afford at two hours an attempt.

A second defect,
found on the way:
the absolute reviewer was shown only the "refinable" paragraphs
(`paragraph` blocks),
and a finding naming any other paragraph number was refused with the whole
ballot.
Slice 10's candidate has zero refinable paragraphs (it is one blockquote),
so in its 10:35 review
six of nine reviewers who located findings by stanza were refused as schema-mismatch and only the three
"acceptable" ballots survived.
Across the run,
38 of 1,582 recorded replies are unacceptable verdicts
with located findings.
Fixed by `8384166e9` as artifact generation ten:
the reviewer is shown every body
block and the recorded paragraph count and digests are of those blocks;
the reader recomputes them by
generation,
so generations eight and nine keep the refinable set.
The blockquote test fails with the
writer's paragraphs emptied and passes restored.

The chain that was to follow Toka_ls never fired on its own:
its wait loop matched its own command line
(`pgrep -f 'corpus-pass.mjs --only Toka_ls'` sees the bash that carries that text).
Carena0442 was
launched by hand at 10:38:50 on `3852e4d86`,
before `321b12673`,
so it can still die on the same guard
if any of its body slices ends unendorsed;
keyword233 follows on the fixed tree,
and Toka_ls is queued
after the chain at overlap 3 (about 117 minutes at overlap 2 is the measurement;
overlap 3 is the
choice,
veto invited) into `~/temp/agent/toka-rerun2-20260902`.

## Advisor review at 09:30 UTC: six proofs and five fixes before the chain

The reviewer read the transcript up to commit `c29003671` and named two defects,
three measurements
to make before Carena launched unattended,
and a list of smaller items.
The chain was stopped,
the
fixes landed as five commits with the whole suite green after each (`~/temp/agent/test-unit-fixes-20260902.log`,
exit 0,
oxlint 0 warnings,
types clean),
and the chain was requeued on the fixed tree.

- `b547e80ed`:
  the reading after a waited-out hold folded the routing refusal in a second time.
  With
  Hyper dry by meter and Synthetic refusing on a wet meter,
  the call waited out Synthetic's hold and then
  ended the run as both-dry,
  the mirror image of today's regime.
  The refusal is what the hold became;
  an
  expired hold is the provider coming back.
  Mirror tests in the hold-wait and router suites.
- `8fa961d2e`:
  `sourcePronounLines` counted 他们,
  其他,
  其他人,
  他人 and 她们 as pronouns and TA inside
  DATA or STATION.
  Positive control over the four pages after the fix:
  Toka_ls 她 20 (raw 20 她,
  1 他 all
  inside 其他 and 他人),
  XIEPT2 她 76 (raw 77 她 against 17 他,
  of which 9 他们,
  5 其他,
  3 他人),
  keyword233 她 9,
  Carena0442 她 110 (raw 111,
  6 他 of which 3 他们,
  1 其他,
  2 他人).
  No page emits TA.
- `6d85b619a`:
  the directory-id refusal fired on any assembled page whose visible name is the entry id.
  Census at the pinned commit:
  23 of 92 archives name the directory,
  and 8 of them (Anilovr,
  Arita,
  ArtsEpiphany,
  Hangmster,
  keyword233,
  Mio,
  mone,
  s5ehfr9) do so in the source too,
  because the handle is
  the person's name.
  The rule now refuses only where the source names the person differently;
  the other
  15 (Acheron,
  DarlinChit,
  dogesir_,
  donotexist_A,
  homoyamakaze,
  Huasheng,
  interrgned,
  Kotori,
  lintong,
  lxyddice,
  MioCardMeow,
  MocaKawai,
  noname,
  Weideriche_,
  XingZ60) are the #269 shape and stay refused
  until a lane renders the name.
  Without this,
  keyword233 would have been refused tonight.
- `b68bae845`:
  every refusal started its own meter reading;
  the pin pass saw eight 429s inside one
  second.
  A reading still in flight is now shared by refusals arriving during it.
  The 30 s backoff did
  not outlast the 31 s burst its own comment cited,
  so the backoff is now `BUDGET_FRESH_MS` (60 s):
  the
  hold expires with the reading that excused it.
- `4d7c47159`:
  rule `QDF` measured 211 characters after whitespace normalisation against `RLM`'s 200;
  trimmed to 181 (measured after the edit) by dropping a clause its first sentence already says.
  `QPW`
  measures 161.

One count to reconcile:
the house-rule commit,
the identity-context comment and its test say the Toka_ls
page uses 她 "sixteen times".
That figure was a line count (`rg -c`,
16 lines carry the character);
the
occurrence count is 20,
on the whole page and on the body alike,
and 20 is what the identity line now
states.
The two source wordings get corrected in the post-launch batch below.

GFP for all six behaviours,
each neutralised in source,
rebuilt,
its test run alone,
restored and rebuilt
(`~/temp/agent/gfp-20260902/`):
the router hold-wait test (3 FAIL lines with the wait skipped),
the
recovery-nudge test (2 with `RECOVERY_NUDGE` dropped),
the fold tests (2 and 2 with the refusal folded
back in),
the pronoun tests (4 with compounds and word bounds off),
the directory-id test (2 with the
source check dropped),
the coalescing test (2 with the in-flight check off).
All six pass on the restored
tree.

Measurements the reviewer asked for,
with what they change:

- The unseated rosters pass the judgeable-roster assertions (`assertJudgeableProducerRoster` for the
  translators against the wide seats and the roster against the late judges;
  the editor assertion
  against the wide seats),
  run through `node --input-type=module-typescript` against the source.
  Carena
  will not die at launch on a roster shape.
- `verify-published` reads only the runs directory (`resolveRunsDir`,
  `artifacts/`,
  `fixed/`) and never
  the corpus clone,
  so the earlier chain's Carena verify was not wrong;
  the clone variables are passed
  to it anyway so the shape is the same as the pass.
- Task 8 (bind the guard to an adjudicated incumbent win) is closed as superseded by the structural
  reduction;
  nothing in it is open.

Left as recorded,
not acted on:
the house-policy pronoun rule reaches Carena and keyword233 only;
the
Toka_ls rerun predates it.

The hold on source edits was lifted at 10:08 UTC after the owner asked why the named work had stopped:
the chain was stopped,
the work landed as whole commits with the suite green after each,
and the chain
was re-armed at 10:28 on `4cdc85f69` with Toka_ls still running (in consolidation since 10:07).

- `12ed82cee`:
  the owner's title rule as a house-policy bullet (official English title when one exists,
  the Life of Pi play,
  notes as established vocabulary,
  a web lookup as evidence);
  the notes both pages
  carry folded into the identity context,
  labelled by side and kind;
  every 《…》 work in the source
  looked up through Exa once and cached under the user's cache directory;
  the critic sheet told what the
  new line kinds license;
  the prepared-pair types moved to their own file at the line budget.
  Positive
  control over eight real entries:
  XIEPT2 18 lines (its archive's 17 comments,
  translation hint and
  glossary included),
  hulicaijia 17 lines and 2410 bytes (the largest block,
  riding every call of that
  entry),
  yulianNyanner 26 lines and 1914 bytes,
  Toka_ls and keyword233 two contributor-credit lines
  each,
  Carena0442 its two footnotes.
  GFP:
  the notes test fails with the collector emptied (2 FAIL
  lines),
  the lookup test fails with the cache read made a miss (2).
- `4cdc85f69`:
  the collapse key folds trailing spaces on blank and blank-quote lines before judging
  (65 pinned pages use Markdown hard breaks on content lines,
  saurikissa's archive 35 of them,
  so
  content lines keep theirs);
  the both-dry error states the meter states and holds it was decided on;
  the router's re-route comment reads holds;
  the pronoun record says twenty on sixteen lines.
  GFP:
  the
  collapse test fails with the fold removed (2),
  the hold-wait test with the measurement clause dropped
  (2).
