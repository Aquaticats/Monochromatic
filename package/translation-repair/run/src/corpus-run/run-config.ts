import { homedir, } from 'node:os';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import spawn from 'nano-spawn';

import {
  childEnvironment,
  contextRoot,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  CORPUS_COMMIT_SHA,
  type CorpusPin,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  type CorpusPinSetting,
  readCorpusPinSetting,
} from './corpus-pin-override.ts';
import {
  assertCheckerIndependence,
  assertCheckerQuorumReachable,
  type RepairModels,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import type { TranslateModels, } from '@monochromatic-dev/translation-repair-translate-lane/ts';
import {
  fetchTransport,
  type ModelTransport,
  type QuotaSnapshot,
  requireStreamBoundsInsideCallTimeout,
  STREAM_FIRST_CONTENT_MS,
  STREAM_IDLE_MS,
} from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  DECISION_ONLY_ROSTER_IDS,
  holdSet,
  type ProviderRecord,
  ROSTER_MODEL_IDS,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type BudgetView,
  createRoutingClient,
  foldCredentialHolds,
  readsImages,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';
import type { RunClient, } from './run-client-contract.ts';
import {
  promptPayloadStore,
  promptUniqueClient,
  RUN_SEATS,
  SEAT_WITHHOLD_ASKS,
  seatTallyClient,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { ProviderOff, } from './providers-off.ts';
import { configureProviders, } from './run-providers.ts';
import { writingProvidersOf, } from './run-writing-providers.ts';
import { refusingCulledSeats, } from '../culled-seat-guard.ts';
import { resolveGit, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { defaultRunsDirIn, } from './runs-layout.ts';

//region Corpus-run configuration
// Shared roster, budgets, corpus pin, and location resolvers for the corpus-run
// entrypoints (`corpus-pass.ts`, `sentinel-probe.ts`). These are operational
// runners, not library API: they invoke the fully tested pipeline over the real
// UNLICENSED corpus and write corpus-derived artifacts, so their output goes to
// the gitignored durable dir `node_modules/.monochromatic/translation-repair-runs/`
// (AGENTS.md TMP/NMD), never into git.

export { RunConfigError, } from './run-config-error.ts';

/**
 Directory of this source file, for locating the worktree via git.
 */
const HERE = import.meta.dirname;

/**
 Candidates no judge fidelity probe has seated (`judge-unmeasured` on the
 card): they hold no seat until measured.
 */
const JUDGE_UNMEASURED: ReadonlySet<RosterModelId> = holdSet({ hold: 'judge-unmeasured', },);

/**
 Models the owner culled from every role (`owner-culled` on the card): off
 the run roster, every derived bench, the readers and the static checker
 bench, while the card keeps the identity typed for the catalogs and the
 unit fixture. `hf:openai/gpt-oss-120b` since 2026-09-24.
 */
export const OWNER_CULLED: ReadonlySet<RosterModelId> = holdSet({ hold: 'owner-culled', },);

/**
 New candidates that hold no judge or preparation seat until measured
 (`judge-unmeasured` on the card). Multiple serving routes do not establish
 eligibility or transfer a predecessor's calibration.
 */
const UNMEASURED_UNTIL_SEATED: ReadonlySet<RosterModelId> = JUDGE_UNMEASURED;

/**
 Roster models no producer calibration has measured, so they hold no
 writing seat: not the translate lane, not the consolidation. The judge
 fidelity probe measures reading, and reading is what they were seated for.

 EMPTY SINCE 2026-09-08, AND AGAIN SINCE 2026-09-09. `google.gemma-4-e2b` sat here from its judge seat
 of 2026-09-07 until the 40-round producer calibration of 2026-09-08 measured
 it beside the nine measured writers: 30 of 298 disinterested ballots
 (10.1 percent, z -1.40 against a 12.8 percent pooled null, 39 of 40
 candidates, 94 of 94 asks usable), not separated from the null, where the
 two writers {@link TRANSLATOR_DROPPED} named then sat at z -4.5 on 2026-09-01.
 By the rule of that day it takes the translator seat and, as a measured
 writer, the consolidation seat. The next single-provider candidate the owner
 approves starts here again. Record: the 2026-09-08 addendum of
 `doc/decision/translation-repair-roster-seating-2026-09-01.md`.

 `inception/mercury-2.5` SAT HERE ON 2026-09-09 from its judge seat
 (its card in `model-cards.ts`) of 17:00 UTC until the 40-round producer
 calibration launched at 17:06 UTC that day (`producer-calibrate 40
 --candidates inception/mercury-2.5`, read at 20:02 UTC, its reading recorded in
 `doc/decision/translation-repair-roster-seating-2026-09-01.md`) measured it
 beside the ten measured writers: 18 of 101 disinterested
 ballots (17.8 percent, adjusted 11.4 for the 23 of 40 candidates it wrote,
 z -0.43 against a 19.5 percent pooled null over 1,229 ballots, 49 of 49
 asks usable, completed streams p50 4.7 s and max 10.1 s), not separated
 from the null, so by the same rule it takes the translator seat and the
 consolidation seat. The bench that day sat without Hyper (its daily limit
 spent, so `glm-5.3` threw 48 of 48) and without Synthetic from 17:25 UTC
 (Qwen3.8-27B threw 35 of 56), and `deepseek-v4-pro-0813` judged and wrote
 from NextBit without its reasoning (`openrouter-catalog.ts`); every
 candidate was compared under the same judges, so the standing among them
 holds. Record: the second 2026-09-09 addendum of the seating decision.
 */
export const WRITER_UNMEASURED: ReadonlySet<RosterModelId> = holdSet({ hold: 'writer-unmeasured', },);

/**
 Every model this run may seat, across both providers.
 Critics and the adjudication panel both use the whole roster so coverage
 overlaps across models rather than partitioning the work.

 DERIVED RATHER THAN LISTED since 2026-08-24, when the roster stopped being
 one provider's model list. A hand-written copy of a two-catalog union goes
 stale the first time either catalog moves, and it goes stale silently: a
 missing model is a seat nobody notices is empty.

 NINE MODELS NOW. `qwen3.8-max` was culled on 2026-08-28 for disproportionate
 metered cost, then Nemotron left every stage on 2026-08-29 after contradictory
 adjacent required-correction reviews, leaving eight. The owner's blocklist
 decision of 2026-09-01 (`doc/decision/translation-repair-roster-blocklist.md`)
 refreshed the catalog under it: `glm-5.3` was admitted on the forced-tool
 probe, GLM-5.3-Flash gained a second route, and two Qwen3.8 routes were
 culled as automatic-only. Remaining seats retain full weight.

 TEN SINCE 2026-09-07. Of the two Bedrock-only Gemma 4 sizes the owner
 approved that day, `google.gemma-4-e2b` joined on the judge fidelity probe
 (its card in `model-cards.ts`) and `google.gemma-4-31b` stayed out: over the
 same twelve questions it chose the complete text on 6, declining the rest,
 against a seated median of 9.5 (the planning log of 2026-09-07, "Measuring
 the two Bedrock-only sizes").

 ELEVEN SINCE 2026-09-09, when the owner approved Mercury 2.5 on OpenRouter
 and the judge fidelity probe of 17:00 UTC read it at 14 of 14
 (its card in `model-cards.ts`); it wrote nothing until the producer
 calibration read at 20:02 UTC the same day seated it as a writer
 ({@link WRITER_UNMEASURED}); the calibration of 2026-09-19 read it below
 the pooled null and took the translator seat back ({@link TRANSLATOR_DROPPED}).

 DeepSeek V4.1 Flash joins on 2026-09-11 after choosing the source-reviewed
 reference on all fourteen distinct comparisons, with no damaged pick or abstention.
 All nine peers answered every question; their median clean count was fourteen
 and maximum damaged count one. Its image-reading hold remains; the producer
 calibration of 2026-09-19 seated it as a writer.
 Record: the 2026-09-11 addendum in
 `doc/decision/translation-repair-roster-seating-2026-09-01.md`.
 */
export const RUN_ROSTER: readonly RosterModelId[] = ROSTER_MODEL_IDS
  .filter(function measured(modelId,): boolean {
    return !UNMEASURED_UNTIL_SEATED.has(modelId,);
  },)
  // THE OWNER'S CULL EMPTIES EVERY BENCH, so it is applied where every bench
  // derives from rather than on each bench in turn.
  .filter(function notCulled(modelId,): boolean {
    return !OWNER_CULLED.has(modelId,);
  },)
  // A DECISION-ONLY SEAT TAKES NO COMPLETION, so it is on no chat bench
  // however it measures; it reaches the select judges through
  // `RUN_DECISION_JUDGES` alone.
  .filter(function chatSeat(modelId,): boolean {
    return !DECISION_ONLY_ROSTER_IDS.includes(modelId,);
  },);

/**
 Consolidation writers: the run roster less {@link WRITER_UNMEASURED}. The
 whole measured roster writes there, GLM-5.3-Flash included, which keeps its
 writing seats and left every judge seat on 2026-09-02, and since 2026-09-08
 `google.gemma-4-e2b`, the first Bedrock-only writer a producer calibration
 measured, and since 2026-09-09 `inception/mercury-2.5`, the first
 OpenRouter-only one. Filtered off {@link RUN_ROSTER} rather than off the catalog so a
 Bedrock-only size the roster seats writes once measured and never before.
 */
export const RUN_WRITERS: readonly RosterModelId[] = RUN_ROSTER
  .filter(function measuredWriter(modelId,): boolean {
    return !WRITER_UNMEASURED.has(modelId,);
  },);

/**
 Writers a 40-round producer calibration measured out of the translator
 seat, dropped on the owner's authorization of 2026-09-01 ("drop any model
 from any role, as long as you have evidence").

 TWO SAT UNDER THE POOLED NULL WITH FULL AVAILABILITY on 2026-09-01:
 `gpt-oss-120b` took 5 of 207 disinterested ballots (z -4.53 against a
 13.02 percent null) and `deepseek-v4-flash-0731` 5 of 208 (z -4.55), each
 having written 40 of 40 candidates, so the finding is about the writing
 rather than about rounds missed. Record and method:
 `doc/planning/translation-repair-roster-calibration-2026-09-01.md`.

 `deepseek-v4-pro-0813` SINCE 2026-09-09, ON ITS SECOND READING BELOW THE
 NULL: 19 of 243 disinterested ballots on 2026-09-08 (z -2.31, Parasail
 serving it with its reasoning) and 13 of 144 over 29 candidates on
 2026-09-09 (adjusted 7.3 percent, z -3.18 against a 19.5 percent null,
 threshold 2.81, NextBit serving it without; 62 of 62 asks usable), the
 second crossing the threshold. The fidelity probe of 20:15 UTC that day
 read the no-reasoning endpoint at 11 of 14 against 4 of 14 with reasoning
 on the same fourteen questions, so the endpoint the run buys stays and the
 reading is of what the run buys. Record: the second 2026-09-09 addendum of
 `doc/decision/translation-repair-roster-seating-2026-09-01.md`.

 ALL THREE KEPT EVERY OTHER SEAT: nothing here measures judging, critique,
 checking or the consolidation, and the anchor judge stays the anchor judge.

 THE TWO DEEPSEEK V4 MODELS LEFT THE ROSTER ON 2026-09-16 at the owner's
 instruction ("DeepSeek V4.1 Flash is much better than both V4 Pro and V4
 Flash, so please remove V4 Pro and V4 Flash"), so only gpt-oss-120b is
 listed; their readings stay recorded because they are the method's worked
 examples.
 */
export const TRANSLATOR_DROPPED: ReadonlySet<RosterModelId> = holdSet({ hold: 'translator-dropped', },);

/**
 Models measured out of every nine-wide seat (critic, panel, judge) on the
 same authorization, for wall clock rather than for quality.

 `glm-5.3` IS THE SLOWEST VOICE ON THE ROSTER in every role measured: its
 completed streams run p50 61 to 66 s and p90 166 to 172 s against p90s under
 110 s for everyone but GLM-5.3-Flash. It lost 11 of 38 select asks to the
 180000 ms production window in the producer calibration, one panel stream
 to the 360000 ms per-call deadline after 3.4 M raw characters and another
 to the 300000 ms calibration window, and each loss holds the round for the
 whole window after quorum at the roster's highest output rate. Across both
 calibration logs 75 to 83 percent of round time is that wait, and the wide
 seats are where a ninth voice adds least: the stage reached quorum without
 it every time. It keeps the translator seat and any editor seat the editor
 standing gives it, where its text is what is being measured.

 `hf:zai-org/GLM-5.3-Flash` LEFT EVERY JUDGE SEAT ON 2026-09-02 by the owner's
 decision ("Unseat GLM-5.3-Flash as a judge, keep it as editor"). Its
 reasoning streams run to a million raw characters, and under the 60 s round
 window of the Toka_ls relaunch it was cut in 12 of 13 panel rounds, 12 of 21
 translate-select rounds, 11 of 29 repair-select rounds, 11 of 15 critic
 rounds and 5 of 15 contest rounds: 51 of the run's 78 cuts. No round lost its
 decision without it and two needed a challenge round, so the seat cost a
 window's wait far more often than it cast a ballot. It keeps the first editor
 seat (top three in every one of 4000 resamples), its refiner seat and its
 translator seat, where its text is what is being measured. Record:
 `doc/planning/translation-repair-roster-calibration-2026-09-01.md`, "The
 Toka_ls relaunch was killed at 77 minutes, in consolidation".

 `hf:Qwen/Qwen3.8-27B` IS NOT DROPPED HERE BUT SEATED PER PROVIDER, the
 owner's decision of 2026-09-03 after a morning in which it left every judge
 seat outright: served by Hyper it reasons past the round window (30 of 34
 translate-lane select rounds cut on XIEPT2), served by Synthetic it answers
 (25 of 28 on Toka_ls). `run-seats.ts` reads the meters before each phase
 and withholds the seat while Hyper is the provider that would serve it, that
 is while Synthetic is dry and Hyper is wet; with both dry no provider serves
 it, and a round reads it as unreachable and hands its place on (ledger P12
 corrected "while Synthetic is dry"). These static benches are the
 Synthetic-wet ones. `hf:moonshotai/Kimi-K3` is likewise withheld from the
 select seats alone while Hyper would serve it (`HYPER_SLOW_SELECT_JUDGES`).
 */
const WIDE_SEAT_DROPPED: ReadonlySet<RosterModelId> = holdSet({ hold: 'wide-seat-dropped', },);

/**
 Models unseated from the roster-wide judge rounds that run after the lanes:
 the lane contest, the consolidation slate's judges and the consolidation
 gate. Those rounds seat the whole roster, `glm-5.3` included, since they were
 built after the wide-seat drop; only the owner's 2026-09-02 decision on
 GLM-5.3-Flash reaches them, for the reason on {@link WIDE_SEAT_DROPPED}.
 Qwen3.8-27B was seated here per provider by `run-seats.ts` until the owner's
 answer of 2026-10-22 (UTC), "Remove from both benches": 17 of 19 lane-contest
 and 7 of 14 consolidation-gate rounds cut on Carena0442 when Hyper served it,
 and in XingZ6017, served by Synthetic, 40 of 95 streams cut after the grace
 with no answer (ledger B810, B815). Its card holds it off these rounds now,
 whichever provider would serve it.
 Pairing and insertion-admission rounds are not judgments of text and lost no
 voice to the window (27 of 27 and 9 of 9 heard), so they keep the roster.
 */
const LATE_JUDGE_DROPPED: ReadonlySet<RosterModelId> = holdSet({ hold: 'late-judge-dropped', },);

/**
 Translators for the translate lane: the roster less
 {@link TRANSLATOR_DROPPED} and less {@link WRITER_UNMEASURED}. Eight since
 2026-09-08 (seven from 2026-09-01): `google.gemma-4-e2b` was measured in
 that day, on 2026-09-09 `inception/mercury-2.5` was measured in and
 `deepseek-v4-pro-0813` out the same evening, and on 2026-09-19
 `deepseek-v4.1-flash` was measured in and `inception/mercury-2.5` out by
 the same calibration, so the stage quorum stays 4
 and every slate keeps at least two disinterested judges under
 `assertJudgeableProducerRoster`.
 */
export const RUN_TRANSLATORS: readonly RosterModelId[] = RUN_ROSTER
  .filter(function stillWrites(modelId,): boolean {
    return (!TRANSLATOR_DROPPED.has(modelId,)) && (!WRITER_UNMEASURED.has(modelId,));
  },);

/**
 Critics, adjudication panel and judges for both lanes: the roster less
 {@link WIDE_SEAT_DROPPED}. Seven since 2026-09-02 (eight from 2026-09-01)
 with Synthetic wet, so each of those stages reaches quorum at 4 voices and
 `minBallotWeight` 3 is 3 of 7; six with Synthetic dry (`run-seats.ts`
 withholds the Hyper-slow judge), quorum 3 (`ceil(6 / 2)`), and five of
 those judge the slates (the select-slow judge withheld too), quorum 3.
 */
export const RUN_WIDE_SEATS: readonly RosterModelId[] = RUN_ROSTER
  .filter(function stillSeated(modelId,): boolean {
    return !WIDE_SEAT_DROPPED.has(modelId,);
  },);

/**
 Judges for the lane contest, the consolidation slate and the consolidation
 gate: the roster less {@link LATE_JUDGE_DROPPED}. Eight from 2026-09-02
 with Synthetic wet, seven with it dry; seven either way since Qwen3.8-27B
 left on 2026-10-22, the seat `run-seats.ts` used to withhold while Synthetic
 was dry.
 */
export const RUN_LATE_JUDGES: readonly RosterModelId[] = RUN_ROSTER
  .filter(function stillJudges(modelId,): boolean {
    return !LATE_JUDGE_DROPPED.has(modelId,);
  },);

/**
 Decision-only seats the judge fidelity probe has seated: they judge slates
 through the select stage's decision adapter and nothing else, since a
 typed-decision model takes no completion. Empty until a probe removes a
 card's `judge-unmeasured` hold.
 */
export const RUN_DECISION_JUDGES: readonly RosterModelId[] = DECISION_ONLY_ROSTER_IDS
  .filter(function measured(modelId,): boolean {
    return !JUDGE_UNMEASURED.has(modelId,);
  },);

/**
 Restoration judges the recall benchmark seats: the wide seats, the chat
 bench that judges both lanes' slates. DERIVED, NOT LISTED (ledger P4): the
 benchmark's own default named three judges by hand, and two days after the
 owner's cull of 2026-09-24 one of them was the culled seat.
 */
export const RECALL_JUDGE_MODEL_IDS: readonly RosterModelId[] = RUN_WIDE_SEATS;

/**
 Judges for both lanes' slates: the wide seats, then the seated
 decision-only seats, in that order so a ballot index reads the same
 whichever bench a reader has in mind.
 */
export const RUN_SELECT_JUDGES: readonly RosterModelId[] = [
  ...RUN_WIDE_SEATS,
  ...RUN_DECISION_JUDGES,
];

/**
 Checkers in the order the checker benchmark of 2026-09-27 ranks them; the
 bench a reading seats is the first three a wet provider serves
 (`run-seats.ts`), and the static bench is the first three.

 THE BENCHMARK (ledger M8 records how it was built and scored): 85 accepted issues from twelve settled runs, each
 shown once against the patch every heard checker had called fixed and once
 against the unchanged archive text, where not fixed is certain. Ranked on
 that certain half first, then on the fixes, unusable answers and latency:
 Qwen3.8-27B 82 and 80, Mimo v2.6 Flash 81 and 80 through the run client
 (its only zero-retention endpoint, 3.9 s median, 22 s at the ninetieth
 percentile), Kimi-K3 78 and 81, `google.gemma-4-31b` 76 and 82, Mercury
 2.5 73 and 76 with 11 unusable.
 The fix half is circular for every model that sat on those runs' checker
 benches, Qwen3.8-27B, Kimi-K3, `gemma-4-26b-a4b-it` and `google.gemma-4-e2b`,
 whose unanimous ballots defined those cases, and independent for the other
 three; the certain half is independent for all. Differences of a few cases
 are inside the noise of 85; the order decides only which served seats fill
 the bench.

 MEASURED OUT OF THE CHECKER ROLE ON THAT BENCHMARK, and only that role:
 `gemma-4-26b-a4b-it` called 40 of 85 unchanged texts fixed, and
 `google.gemma-4-e2b`, the substitute since 2026-09-24, called 50 of 85
 real fixes not fixed; together they resolved 35 fixes and wrongly resolved
 9 unchanged texts where every bench of three from this order resolved 81 or
 82 and wrongly resolved 2 to 5. Solar Mini 4 called 78 of 85 unchanged
 texts fixed and Mimo v2.6 Pro read below Flash on both halves while sharing
 8 of its 9 errors; neither holds a card.

 NONE OF THESE EDITS OR REFINES, so the independence the contract checks
 holds on every bench this order can seat.
 */
export const RUN_CHECKER_ORDER: readonly RosterModelId[] = [
  'hf:Qwen/Qwen3.8-27B',
  'xiaomi/mimo-v2.6-flash',
  'hf:moonshotai/Kimi-K3',
  'google.gemma-4-31b',
  'inception/mercury-2.5',
];

/**
 Places on the checker bench: the contract's floor, so a split verdict
 still reads as a majority.
 */
const CHECKER_BENCH_WIDTH = 3;

/**
 The run's repair roster: the contract, with the refiners and the checkers'
 self-certification a run always sets required rather than optional, so no
 reader of the run's roster needs a fallback for either (ledger T8).

 @example
 ```ts
 const refiners: readonly RosterModelId[] = RUN_MODELS.refinerModelIds;
 ```
 */
export type RunRepairModels = RepairModels & {
  /**
   Naturalness refiners, which every run seats.
   */
  readonly refinerModelIds: readonly RosterModelId[];

  /**
   Whether a checker may confirm a repair it helped write, which every run states.
   */
  readonly checkerSelfCertificationPermitted: boolean;
};

/**
 Role roster for a corpus run: SEVEN of the nine critique and adjudicate (six while Synthetic is dry), THREE edit
 against each other, THREE refine the result for naturalness, and three check
 the shipped repair.

 THREE refiners rather than the two first proposed, because two does not
 achieve what widening was for. The quorum is `ceil(rosterSize / 2)`, which is
 1 on a roster of two, so a two-refiner lane could still ship on one voice and
 the single-model failure this change exists to prevent would survive it. At
 three the quorum is 2 and a lone survivor cannot carry the stage. The same
 arithmetic is why editors are three and not two.

 Editors went from two to three and refiners from one to three on 2026-08-12,
 on the user's rule that the system must not have single-model failures.
 The reason is structural rather than a reaction to one incident:
 `gatherStageVoices` computes its quorum as `ceil(rosterSize / 2)`, which on a
 roster of two is satisfied by ONE voice and on a roster of one cannot fail at
 all. So the two-editor pair could ship a repair written by a single model
 while reporting a met quorum, and the single refiner could vanish entirely
 with nothing to report.

 Both stages retried to `full-roster` from 2026-08-12 until the user removed
 that target outright on 2026-08-15: waiting for every voice let one model
 degraded for a day spend four deadlines per gather on a voice that was not
 coming. Quorum on a roster of three is two, so the ensemble property survives
 the removal.

 WHAT FOLLOWS IS THE HISTORY OF THESE SEATS, kept because each rule in it was
 argued from it. The seats themselves are the measured ones in the constant:
 the roster spans two providers since 2026-08-24 (ten then, nine now), the three editors
 and three refiners were chosen by the 40-round writer calibration of that day
 and reseated by the 40-slice editor calibration of 2026-09-01, and the
 checkers are the three the width measurement settled.

 GLM-4.7-Flash took the third editor seat on 2026-08-12 because it was the
 only model not already checking or editing, and the constraints left no
 other choice at the time: checkers had to exclude every editor and refiner,
 judges needed two disinterested seats, and the other three models held the
 checker roster. It left the roster on 2026-08-24.

 ONE OF THOSE CONSTRAINTS IS GONE. Producers judge as of 2026-08-15, with a
 ballot for their own work counted at half weight, so seating another producer
 no longer starves selection: the discount applies to a judge's ballot for its
 OWN candidate only, and every producer votes on every other candidate at full
 weight.

 What still bounds a producing roster is not that arithmetic. It is checker
 disjointness, which is the binding one: checkers exclude every editor and
 refiner, so a fourth editor would leave two checkers at a quorum of one, the
 exact single-voice failure the 2026-08-12 roster change closed. Next to it
 sit `assertJudgeableProducerRoster`'s two-disinterested-judge floor, now a
 policy rather than an arithmetic necessity, and judge quality, which the judge-quality
 bench has not measured. Widening was question 1 of the handover's next steps and
 was answered by the calibration recorded at `editorModelIds`. GLM-4.7-Flash
 was also the model that most often lost its voice, which argued FOR seating
 it as a third editor rather than against: a third editor that sometimes
 drops still leaves two, whereas the same model in the checker set would have
 cost proof.

 The panel was six rather than seven from 2026-08-05, when the provider
 withdrew two models and only one replacement (Kimi-K3) appeared, and has
 been ten since 2026-08-24, when Charm Hyper's five seats joined, eight after
 the removals of 2026-08-28 and 2026-08-29, and nine since the 2026-09-01
 catalog refresh. `gatherStageVoices` computes the stage quorum as
 `voices >= ceil(modelIds.length / 2)`, so seven models need 4 voices, six
 need 3, ten need 5, eight need 4, and nine need 5.

 The ISSUE-acceptance gate does move, and the user accepted the move rather
 than it happening unnoticed: `DEFAULT_ADJUDICATION_CONFIG.minBallotWeight` is
 the absolute value 3, so the share of the panel that must cast a non-abstain
 ballot before any decision rises from 3-of-7 (43 percent) to 3-of-6 (50).
 User decision, 2026-08-06: "50% is okay here." At nine the same absolute 3
 is 3-of-9 (33 percent), inside the range already lived at ten (30 percent)
 and at eight (37.5 percent).

 MORE THAN ONE editor, on the user's rule that no single model should control
 any part of the pipeline. Kimi-K3 is one of them because the user reports it
 as much stronger than anything else currently offered, and the editor is
 where model strength converts most directly into repair quality. The
 round-two grading supports spending strength there specifically: four of the
 37 true positives carried notes saying detection was right but the proposed
 repair was poor ("is there a better way?"), which is an editor complaint, not
 a critic one. GLM-5.2 was the second editor, having held the role alone
 before, and GLM-4.7-Flash was the third until 2026-08-24, when the measured
 seats replaced both.

 The count was TWO until 2026-08-12, and the paragraph that follows is kept as
 the reasoning for that earlier choice rather than as current policy. Every
 editor was barred from judging its own chunk, so each added editor cost a
 judge as well as its own calls: at two editors four judges remained, at three
 only three, and a plurality got harder to reach exactly as the candidate set
 got wider. Producers judge now, so that arithmetic no longer holds.

 That cost was accepted on the quorum argument this block opens with: a stage
 of two is satisfied by a single voice, so a two-editor pair could ship a
 repair written by one model while reporting a met quorum. Losing a judge is
 the smaller harm.

 Judges are the WHOLE roster, and since 2026-08-15 selection seats all of it
 rather than removing producers per round. An editor judging a set holding its
 own text is allowed and counts half for that candidate alone; every other
 ballot it casts carries full weight.

 Checkers EXCLUDED every editor until 2026-08-24, so nothing checked its own
 work. That dropped GLM-5.2 from the checker set it held while it was also
 editing, and gpt-oss-120b took the seat. GLM-4.7-Flash stayed out: it was
 the model that most often lost its voice to schema mismatch, and the checker
 stage is where a lost voice costs proof rather than coverage. Since
 2026-08-24 one model both edits and checks, bounded by the half-weight
 discount described at `checkerModelIds`.

 The naturalness lane shares two of its three seats with the editors since
 2026-09-01 (GLM-5.3-Flash and deepseek-v4-pro-0813), with minimax-m3 as its
 third. Nothing forbids a refiner also editing: a judge's ballot for its own
 candidate has counted half since 2026-08-15 and a checker's verdict on text
 it helped write has counted half since 2026-08-24, so neither stage is
 decided by the model whose text it is. The tradeoff is real and is accepted
 rather than hidden: a model that just wrote a paragraph is a poor judge of
 whether that paragraph reads awkwardly.

 This paragraph read "ONE refiner runs the naturalness lane" until 2026-08-13,
 left stale by `eb21ffa6b`, which took the lane from one refiner to three. The
 staleness mattered rather than being cosmetic: a one-model lane loses the
 whole stage to a single lost voice, and 34 such losses across 7 entries were
 measured on the corpus run that ended 2026-08-11, every one recorded as
 `refiner 0/1`. On a roster of three the same failure no longer empties the
 stage, because quorum is two. Recorded in
 `doc/planning/naturalness-lane-reach.md`.

 Attribution cost, accepted by the user ("Bundle all the improvements that
 could be made, in"): round three changes the roster, the editor, the checker
 set, and adds a naturalness pass at once, so a precision delta cannot be
 attributed to any single change. Record this in the round-three verdict
 rather than rediscovering it during analysis.

 TYPED AS `RunRepairModels` since 2026-09-29: typed as the contract, the
 run's refiners and self-certification read as optional, and the
 independence check read them through fallbacks no run could reach
 (ledger T8).
 */
export const RUN_MODELS: RunRepairModels = {
  // THE ROSTER LESS ITS SLOWEST VOICE since 2026-09-01 in every nine-wide
  // seat; the reason sits on `WIDE_SEAT_DROPPED`.
  criticModelIds: RUN_WIDE_SEATS,
  panelModelIds: RUN_WIDE_SEATS,
  // MEASURED ON THE EDITOR'S OWN JOB since 2026-09-01. `editor-calibrate`
  // over 40 bench slices at build `48799e6d1` drove the whole repair lane
  // with every roster model editing and judging: 111 judged editor rounds
  // from the 30 slices that carried an accepted issue, 6131 disinterested
  // ballots against a pooled null of 15.8 percent. Availability-adjusted
  // share (raw share times candidates over the fullest model's 126):
  // GLM-5.3-Flash 32.2 percent (221 of 669, z +12.25), `glm-5.3` 21.4 (143
  // of 657, z +4.21), `deepseek-v4-pro-0813` 18.8 (130 of 657, z +2.82),
  // then Qwen3.8-27B 17.3 (121 of 688, z +1.31), Kimi-K3 14.2, gpt-oss-120b
  // 12.6, minimax-m3 8.4, gemma-4-26b-a4b-it 7.0 (56 of 767, z -6.44),
  // deepseek-v4-flash-0731 5.6. The two incumbents it unseats sat fourth and
  // eighth; gemma, second-best WRITER in the producer calibration of the same
  // day, wrote the worst edits, which is the divergence the editor instrument
  // exists to catch. Record, method and the slice-clustered reading:
  // `doc/decision/translation-repair-roster-seating-2026-09-01.md`.
  //
  // THE SEAT BEFORE THAT was the writer calibration of 2026-08-24, kept here
  // because its reading of availability still governs how a standing is read:
  // 40 rounds in which all ten roster models wrote a candidate for the same
  // slices and every other model judged them, giving 2492 disinterested
  // ballots against a pooled null of 13.48 percent.
  //
  // AVAILABILITY IS PART OF THE SCORE, and reading it that way is what moved
  // the ranking. A model absent from a round produced nothing to be judged, so
  // its headline rate is computed over the rounds it survived and flatters a
  // model that skipped the hard ones. Charging every model zero for the rounds
  // it missed, `hf:Qwen/Qwen3.8-27B` leads at 22.3 percent (z 4.27, clearing
  // the Bonferroni threshold of 2.81 for ten comparisons),
  // `gemma-4-26b-a4b-it` takes 18.3 percent having answered 40 rounds of 40
  // with no cut, and `hf:zai-org/GLM-5.2` sits at 8.2 percent, below the null
  // in this pass and in the 12-round pass before it. GLM-5.2 loses the seat.
  //
  // `qwen3.8-max` IS EXCLUDED DESPITE THE BEST HEADLINE, 27.0 percent. It
  // answered 28 rounds of 40, and the 12 it missed were the LARGER slices:
  // among the models that did answer, median answer length was 588 characters
  // in the rounds it missed against 366 in the rounds it made, a ratio of 1.61,
  // Mann-Whitney z +3.51, p 0.0004. Its headline is survivorship on the easy
  // half of the corpus, and it took 29 cuts, more than any other model. It
  // remained in wide roles at this calibration point, then was culled from
  // whole roster on 2026-08-28 because metered cost was disproportionate and
  // exceptionally expensive.
  //
  // THE SEAT STILL CROSSES PROVIDERS: Charm Hyper serves `glm-5.3` and
  // `deepseek-v4-pro-0813`, Synthetic serves GLM-5.3-Flash (Hyper carries it
  // too), so an outage on either side leaves the stage a writer instead of
  // emptying it. TWO OF THE THREE ARE ONE MODEL FAMILY, which the provider
  // argument alone would hide: a GLM blind spot is now shared by two seats,
  // and the third seat is the only one outside it. That is accepted on the
  // measurement rather than designed around, and it is the reason the
  // standing is re-read rather than the seat assumed. Neither reads images,
  // which costs nothing here: pictures are read in their own stage over a
  // catalog-derived roster, and `document-lanes.ts` records that no
  // repair-lane stage ever asks what one says.
  //
  // WALL CLOCK IS THE PRICE OF THIS SEAT. The two GLM models are the roster's
  // slowest voices (completed streams p50 52 to 66 s, p90 153 to 199 s), so
  // an editor round now waits on them where it once waited on nobody; the
  // corpus pass bounds that with its straggler window rather than by seating
  // a faster, worse editor.
  //
  // THE THIRD SEAT PASSED TO `deepseek-v4.1-flash` ON 2026-09-16 when the
  // owner removed `deepseek-v4-pro-0813` from the roster ("DeepSeek V4.1
  // Flash is much better than both V4 Pro and V4 Flash"). The owner's words
  // are the evidence for the seat; no editor calibration has measured the
  // successor, and the next `editor-calibrate` re-reads it.
  editorModelIds: [
    'hf:zai-org/GLM-5.3-Flash',
    'glm-5.3',
    'deepseek-v4.1-flash',
  ],
  // An editor still judges a slate holding its own text at half weight for
  // that candidate alone.
  judgeModelIds: RUN_SELECT_JUDGES,
  // MEASURED ON THE REFINER'S OWN JOB, NOT TRANSFERRED FROM THE EDITORS.
  // `editor-calibrate` runs the naturalness lane after the accuracy lane and
  // reports the refiner standing off the same spend: 25 judged refiner rounds
  // from the 25 of 40 slices that carried a paragraph over the eligibility
  // floor, pooled with the 4 rounds of the 6-slice replicate run the same
  // day, 1393 disinterested ballots against a pooled null of 14.0 percent.
  // Availability-adjusted: GLM-5.3-Flash 32.5 percent (66 of 172, z +9.21),
  // `deepseek-v4-pro-0813` 17.1 (35 of 205), `glm-5.3` 12.1 (25 of 190),
  // `minimax-m3` 9.7 (20 of 103, having proposed a rewrite in 13 of 26
  // opportunities), then Kimi-K3 7.8, Qwen3.8-27B 7.7, gpt-oss-120b 4.3,
  // gemma 3.4, deepseek-v4-flash-0731 0.5.
  //
  // THE THIRD SEAT WENT ON RELIABILITY, by the rule written before the
  // numbers arrived. Bootstrapping whole slices (26 winner-bearing rounds,
  // 4000 resamples) puts GLM-5.3-Flash in the top three 99.9 percent of the
  // time and deepseek-v4-pro-0813 97.5, but leaves `glm-5.3` (24.3) and
  // `minimax-m3` (42.8) unseparated; where the third and fourth seats are
  // not separated, the seat goes to the model that lost fewer voices under
  // the production window, and minimax-m3 lost none (p90 48 s) where
  // `glm-5.3` is the roster's slowest voice and lost 15 of 78 asks.
  //
  // THE SECOND SEAT PASSED TO `deepseek-v4.1-flash` ON 2026-09-16 on the
  // owner's removal of `deepseek-v4-pro-0813`, for the reason the editor
  // seat gives; unmeasured in this role until the next `editor-calibrate`.
  refinerModelIds: [
    'hf:zai-org/GLM-5.3-Flash',
    'deepseek-v4.1-flash',
    'minimax-m3',
  ],
  // THREE, MEASURED RATHER THAN PREFERRED, and the wide arm is gone. The
  // owner ruled on 2026-08-23 that checker width is settled by measurement:
  // the same entries run once with these three and once with all six,
  // per-issue resolution compared, winner shipped and loser deleted. Four wide
  // runs answered it on 2026-08-24 with 231 rounds and 1360 ballots, and NOT
  // ONE verdict moved.
  //
  // THE NULL IS ABOUT WIDTH RATHER THAN ABOUT SILENCE, which is the only way a
  // null settles anything here. The six disagreed on 14 rounds, and on 10 of
  // those a writer answered something no checker of the narrow three said, so
  // the extra ballots carried real information. The arithmetic absorbed it:
  // a checker judging text it helped write counted half, so three writers
  // brought 1.5 against 3.0 and could not overturn a unanimous three. They
  // could only reach a split three, which happened on 4 rounds, and on none of
  // those did all three writers dissent together.
  //
  // NEMOTRON LEFT EVERY ROLE on 2026-08-29 at the owner's instruction after it
  // contradicted its own adjacent required-correction guidance. Kimi-K3 takes
  // that checker seat rather than shrinking below the hard floor of three. It
  // was already one of the added checker voices in all 231 measured wide-arm
  // rounds, where added voices changed no resolution verdict. That evidence is
  // about ensemble effect rather than individual checker ranking; a fresh
  // checker-seat calibration remains required before calling the new narrow
  // roster independently optimal.
  //
  // DISJOINT AGAIN SINCE 2026-09-01, BY MEASUREMENT RATHER THAN BY RULE: the
  // editor calibration seated no checker as editor or refiner. Overlap stays
  // PERMITTED, the owner's decision of 2026-08-24 (enable the discount and let
  // every model do both; the checker-side discount was found unreachable in
  // production for exactly the reason the old note here gave), so a future
  // seating that overlaps needs no rule change.
  //
  // WHAT MAKES OVERLAP SAFE is measured rather than assumed, and it is the
  // discount the DISJOINT AGAIN paragraph names: a checker judging text it helped write counts half, so
  // three writers bring 1.5 against 3.0 and cannot overturn a unanimous three.
  // Between 2026-08-24 and 2026-09-01 two checker ids were also producer ids,
  // and only actual authorship of text under review received half weight: had
  // both helped write one refined result, their combined weight would have
  // been 1.0, equal to disinterested GPT-OSS alone, unable to resolve an issue
  // against that independent vote.
  //
  // GPT-OSS LEFT EVERY ROLE on 2026-09-24 at the owner's instruction (the
  // card's own note has the two misreadings). `gemma-4-26b-a4b-it` took the
  // third checker seat then, unmeasured as a checker, with
  // `google.gemma-4-e2b` as the substitute.
  //
  // MEASURED SINCE 2026-09-27: the checker benchmark ranks the seats
  // ({@link RUN_CHECKER_ORDER}), both Gemma sizes left the checker role on
  // it, and the static bench is the top three of that order. A reading
  // seats the first three a wet provider serves (`run-seats.ts`).
  checkerSelfCertificationPermitted: true,
  checkerModelIds: RUN_CHECKER_ORDER.slice(
    0,
    CHECKER_BENCH_WIDTH,
  ),
};

// Refuse invalid production role composition at configuration load rather than
// after a live corpus pass has already paid for critics, panels, and editors.
assertCheckerIndependence({
  editorModelIds: RUN_MODELS.editorModelIds,
  refinerModelIds: RUN_MODELS.refinerModelIds,
  checkerModelIds: RUN_MODELS.checkerModelIds,
  selfCertificationPermitted: RUN_MODELS.checkerSelfCertificationPermitted,
},);
assertCheckerQuorumReachable({
  checkerModelIds: RUN_MODELS.checkerModelIds,
},);

/**
 Roster the translate lane runs under during a corpus pass.

 BOTH ROLES TOOK THE WHOLE ROSTER until 2026-09-01, which was a narrower
 claim than it looked. The translate lane has two stages and no third: models
 write a candidate, then models rank the slate. There is no editor stage to
 keep a producer out of and no checker stage certifying its own work, so the
 exclusions {@link RUN_MODELS} spends most of its rationale on have nothing to
 exclude here. What narrows the seats now is measurement rather than role
 structure: {@link RUN_TRANSLATORS} drops the two writers the 40-round
 producer calibration placed under its pooled null, and {@link RUN_WIDE_SEATS}
 drops the judge that lost a third of its select asks to the production
 window, both on the owner's authorization of that day.

 Self-certification is HANDLED RATHER THAN FORBIDDEN: a judge ranking a slate
 that holds its own translation counts half for that candidate alone, exactly
 as the repair lane's selection round does. Whether that weighting is the right
 one is the same open question for both lanes rather than a
 new one this constant introduces.

 The width the judge-fidelity probe and the window trial MEASURED was the
 whole roster of their day, six models in both roles. A pass under seven
 translators and eight judges reports on a lane neither measurement covers
 exactly, as every roster change since has, which is one reason the pass's
 own output is read before any readiness claim.

 @example
 ```ts
 const lanes = await runDocumentLanes({ translateModels: RUN_TRANSLATE_MODELS, ... },);
 ```
 */
export const RUN_TRANSLATE_MODELS: TranslateModels = {
  translatorModelIds: RUN_TRANSLATORS,
  judgeModelIds: RUN_SELECT_JUDGES,
};

/**
 Reported image capability is not yet a verified reader seat for a new model.
 Keep this separate from judge admission: Gemma 4 31B already reads without judging.
 */
const READER_UNMEASURED: ReadonlySet<RosterModelId> = holdSet({ hold: 'reader-unmeasured', },);

/**
 Models that read this run's pictures.

 DERIVED FROM THE CATALOG rather than listed by hand, so the roster is
 whatever the provider's own `input_modalities` says can be sent an image. A
 hand-written list would go stale the day a model gains or loses the
 capability, and it would go stale silently: a text-only model sent a picture
 answers about nothing, and the call is spent either way.

 READING IS ITS OWN STAGE, and the roster's narrowness when the stage was
 built is why. Exactly two models read images while the roster was one
 provider's six; selection needs a minimum weight of two, and a producer's
 ballot for its own work counts half, so if those two also translated then no
 disinterested judge would remain on any slice carrying a picture. Asking them
 only to READ turns the picture into text, and the whole roster translates
 and judges from that text with its weights untouched. Four of the nine read
 images since the 2026-09-01 catalog refresh (`glm-5.3` reads none), and six
 since 2026-09-08, when two Gemma sizes were measured as readers through
 Bedrock, so a Bedrock-only pass has two reachable readers; the stage stays
 separate because the reasoning in this note is about weights, not about how
 many readers there happen to be.

 @example
 ```ts
 const readings = await readDocumentPictures({ readerModelIds: RUN_READER_MODELS, ... },);
 ```
 */
export const RUN_READER_MODELS: readonly RosterModelId[] = ROSTER_MODEL_IDS
  .filter(function reads(modelId,): boolean {
    return readsImages({ modelId, },)
      && (!READER_UNMEASURED.has(modelId,))
      && (!OWNER_CULLED.has(modelId,));
  },);

/**
 Deadline granted to one model exchange during a corpus run.

 Was 240_000, which measurably clipped real work. Run 013 sampled every call
 unfiltered: 748 succeeded and 35 were cut at the deadline, a 4.5 percent
 censoring rate, while the surviving time-to-first-byte distribution ran p50
 45_837 ms, p90 163_296 ms, p99 218_976 ms, and max 235_151 ms. Fifteen calls
 landed in the last 25 seconds before the cut, so the distribution had real
 density right up to the boundary with NO cliff ahead of it. That is the
 signature of clipping, not of connections hanging: a call completing at
 245_000 ms would be unremarkable beside the ones observed at 235_151 ms.

 Timeouts still arrive in correlated batches, about five per retry round,
 which once looked like evidence of hangs. It reconciles if the provider slows
 every concurrent call together under load, so a batch crosses the deadline
 together. That explains the correlation without hangs, and it means the added
 waiting falls during congested periods specifically.

 360_000 is chosen against the measurement rather than as a round multiple: it
 clears the observed p99 by 64 percent and the observed maximum by 53 percent,
 while keeping a worst-case stage bounded. `STAGE_RETRY_ROUNDS` allows four
 deadlines in one stage, so this caps a pathological stage near 24 minutes
 (the recovery round after them waits only a straggler window), where 480_000
 would put it past 32. The per-entry ceiling it was weighed against was 90
 minutes when this was written; it has been 420 since 2026-08-17
 (`HARD_CAP_MINUTES` in `corpus-pass.ts`), and this comment said 90 until
 ledger D6.
 Raising it should also REDUCE retry rounds by losing fewer voices, so the
 worst case gets rarer as well as no worse.

 Sampling stays unfiltered, so the next run reports how much tail still gets
 clipped at 360_000 and this can be tuned on evidence again.
 */
export const RUN_PER_CALL_TIMEOUT_MS = 360_000;

/**
 Call-timing knobs an artifact was produced under, so a pool spanning more
 than one configuration can still be analyzed per cohort.

 @example
 ```ts
 const config: RunCallConfig = {
   perCallTimeoutMs: 240_000,
   streamFirstContentMs: 150_000,
   streamIdleMs: 60_000,
 };
 ```
 */
export type RunCallConfig = {
  /**
   Total-duration deadline one model exchange was granted.
   */
  readonly perCallTimeoutMs: number;

  /**
   Silence allowed before a stream's first content or reasoning delta. Named
   `streamFirstByteMs` on artifacts written before ledger B804, when the
   window counted bytes of any kind.
   */
  readonly streamFirstContentMs: number;

  /**
   Silence allowed between a stream's content or reasoning deltas once flowing.
   */
  readonly streamIdleMs: number;
};

/**
 Builds the call-timing knobs, refusing a set whose stream bounds cannot fire
 before the per-call deadline.

 @param perCallTimeoutMs - deadline one model exchange is granted

 @param streamFirstContentMs - silence allowed before the first generated text

 @param streamIdleMs - silence allowed between pieces of generated text

 @returns The three knobs, unchanged, to stamp on an artifact

 @throws {@link StreamBoundsOutsideCallTimeoutError} when a bound is not a
 positive finite number shorter than the deadline (ledger B804)

 @example
 ```ts
 const config = runCallConfigOf({ perCallTimeoutMs: 360_000, streamFirstContentMs: 180_000, streamIdleMs: 120_000, },);
 ```
 */
export function runCallConfigOf(
  {
    perCallTimeoutMs,
    streamFirstContentMs,
    streamIdleMs,
  }: RunCallConfig,
): RunCallConfig {
  requireStreamBoundsInsideCallTimeout({
    perCallTimeoutMs,
    streamFirstContentMs,
    streamIdleMs,
  },);
  return {
    perCallTimeoutMs,
    streamFirstContentMs,
    streamIdleMs,
  };
}

/**
 Call-timing configuration stamped into every artifact this pass writes.

 The pool it labels is deliberately MIXED, by a decision the user made twice:
 keep already-settled entries rather than discard the compute.

 An earlier version of this note promised more than the stamp can deliver: it
 said precision could be split by cohort at analysis time, turning the
 confound into a number. RETRACTED, because the arithmetic does not support
 it. The graded sample is 50 items and the pool at the coverage bar is about
 30 entries split near evenly between cohorts, so a per-cohort precision
 estimate rests on roughly 25 graded items and carries a standard error near
 8 points. A difference small enough to matter cannot resolve at that width,
 and the binding constraint is human grading effort, not compute, so widening
 the sample to the several hundred per cohort that would resolve it is not
 available. Claiming the number anyway would repeat the exact error retracted
 from the panel-coverage analysis: pooling across a noisy dimension and
 reading the result as signal.

 What the stamp is still for: identifying which cohort any artifact came from,
 so the mixed pool is disclosed QUALITATIVELY with the verdict rather than
 left unstated, and so a later analysis over a larger graded set is possible
 if one is ever funded.

 Three cohorts exist in the round-two pool, and the first two are equivalent
 for call timing even though they look different:

 -   Ten entries with NO `callConfig` field at all, settled before the field
     existed. Their absence identifies them exactly.
 -   Five entries stamped `perCallTimeoutMs: 240_000`, from run 013. The
     stream idle guard existed during this run but fired ZERO times, so these
     five ran under the same effective timing as the ten with no `callConfig`. Treat the
     fifteen as ONE cohort.
 -   Entries stamped `perCallTimeoutMs: 360_000` and later, which are the
     first to run without the deadline clipping roughly 4.5 percent of calls.

 Deliberately not surfaced on the grading sheet: a grader who could see which
 cohort an issue came from would be a worse instrument than one who could not.
 */
export const RUN_CALL_CONFIG: RunCallConfig = runCallConfigOf({
  perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
  streamFirstContentMs: STREAM_FIRST_CONTENT_MS,
  streamIdleMs: STREAM_IDLE_MS,
},);

/**
 Built-in corpus read location: the user's local clone at the benchmark
 commit. Content is read at runtime and never committed here (the clone is
 UNLICENSED).
 */
const RUN_CORPUS_PIN_FALLBACK: CorpusPin = {
  cloneDir: join(
    homedir(),
    'one-among-us',
    'data',
  ),
  commitSha: CORPUS_COMMIT_SHA,
};

/**
 Reads the run's corpus pin: the built-in clone and commit unless the
 environment overrides either half (`TRANSLATION_REPAIR_CORPUS_CLONE_DIR`,
 `TRANSLATION_REPAIR_CORPUS_COMMIT`), which fixture runs against an unmerged
 corpus pull request need, beside where each half came from, for the launch
 line `corpusPinOverrideNote` makes of it.

 CALLED BY AN ENTRY INSIDE ITS REFUSAL BOUNDARY, and handed down as a value.
 Until 2026-10-10 this was a module constant, resolved while this file
 loaded, so an unreadable override exited every command that loads this
 file at 1 under a dump of bundled source, before `reportingRefusals` ran,
 whether or not the command reads the corpus (ledger B302, B381).

 @param env - environment of the process the entry runs in

 @returns The pin beside a per-half source

 @throws StatedRefusalError when an override is written and unreadable

 @example
 ```ts
 const { pin, } = readRunCorpusPinSetting({ env: process.env, },);
 ```
 */
export function readRunCorpusPinSetting(
  { env, }: { readonly env: Readonly<NodeJS.ProcessEnv>; },
): CorpusPinSetting {
  return readCorpusPinSetting({
    env,
    fallback: RUN_CORPUS_PIN_FALLBACK,
  },);
}

/**
 Worktree root of this checkout, resolved through git from this file's dir.

 @returns Absolute path to the worktree top level

 @example
 ```ts
 const root = await resolveWorktreeRoot();
 ```
 */
async function resolveWorktreeRoot(): Promise<string> {
  /**
   Captured git stdout: the worktree top-level path.
   */
  const { stdout, } = await spawn(
    await resolveGit(),
    [
      '-C',
      HERE,
      'rev-parse',
      '--show-toplevel',
    ],
    { env: childEnvironment({ parent: process.env, },), },
  );
  return stdout;
}

/**
 Current HEAD commit of this worktree, recorded into run artifacts so every
 result names the pipeline tip that produced it.

 @returns Full HEAD sha

 @example
 ```ts
 const tip = await readHeadSha();
 ```
 */
export async function readHeadSha(): Promise<string> {
  /**
   Captured git stdout: the HEAD sha.
   */
  const { stdout, } = await spawn(
    await resolveGit(),
    [
      '-C',
      HERE,
      'rev-parse',
      'HEAD',
    ],
    { env: childEnvironment({ parent: process.env, },), },
  );
  return stdout;
}

/**
 Durable, gitignored directory that holds run artifacts, logs, and the
 attempts map. Honors `TRANSLATION_REPAIR_RUNS_DIR`, else defaults under the
 worktree's `node_modules/.monochromatic/`.

 @returns Absolute runs directory path

 @example
 ```ts
 const runsDir = await resolveRunsDir();
 ```
 */
export async function resolveRunsDir(): Promise<string> {
  /**
   Explicit runs-dir override from the environment, when set.
   */
  const override = process.env
    .TRANSLATION_REPAIR_RUNS_DIR;
  if ((override !== undefined) && (override !== ''))
    return override;
  return defaultRunsDirIn({ worktreeRoot: await resolveWorktreeRoot(), },);
}

/**
 Logger root for the corpus-run wiring layer.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/* oxlint-disable require-await, typescript/require-await -- compatibility meter contract is asynchronous */
/**
 Synthetic quota shape used only when Hyper is sole configured provider.

 Router receives absent Synthetic as dry directly;
 this compatibility method keeps client surface stable for observational callers.

 @returns Explicitly exhausted synthetic quota

 @example
 ```ts
 const quota = await unconfiguredSyntheticQuota();
 ```
 */
async function unconfiguredSyntheticQuota(): Promise<QuotaSnapshot> {
  return {
    fiveHour: {
      remaining: 0,
      max: 0,
      limited: true,
      nextTickAt: '',
    },
    weekly: {
      percentRemaining: 0,
      nextRegenAt: '',
    },
  };
}
/* oxlint-enable require-await, typescript/require-await */

/**
 The providers that keep a run going while they read wet: those that serve a model seated in a writing bench, the
 editors, the refiners or the translators. Every provider but Xiaomi in the roster of 2026-10-22, whose one seat is a
 checker. A wet provider outside this record does not count against "every provider dry", so a fleet that can write
 nothing ends the run instead of idling (`run-writing-providers.ts`; the lead's ruling of 2026-10-22, open to the
 owner's veto).
 */
export const RUN_WRITING_PROVIDERS: ProviderRecord<boolean> = writingProvidersOf({
  modelIds: [
    ...RUN_MODELS.editorModelIds,
    ...RUN_MODELS.refinerModelIds,
    ...RUN_TRANSLATORS,
  ],
},);

/**
 Builds client every run calls from every configured provider,
 counting every call.

 ONE FACTORY, so provider routing reaches every corpus-run entrypoint at once
 rather than each one growing its own configuration.

 IT STILL ANSWERS `quotas`, which is the first provider's meter and nothing
 else. The routing client does not offer one, because the two providers meter
 differently and there is no single reading; this wiring layer is where the
 knowledge that `quotas` means the Synthetic meter belongs, and keeping the
 method here leaves every existing caller and the bench recorder untouched.

 ONE KEY IS ENOUGH.
 Missing provider is marked dry before routing,
 so its seats become unavailable without unauthorized calls.
 Exact-half participation lets the other providers operate normally;
 no provider family or cross-provider response is mandatory.
 Every key missing remains a launch refusal because no call can run.

 THE THIRD KEY IS OPTIONAL AND ITS ABSENCE IS LOUD, as the second one's is:
 OpenRouter is the paid fallback the owner chose on 2026-09-03, and a run
 without its key still starts on the other two.

 THE DRYNESS VIEW IS EXPOSED for the seat reader, so the benches are
 derived from the same reading the router routes by.

 EVERY CALL IS COUNTED on `RUN_SEATS`, the process-wide tally the refusal
 boundary prints when the command ends, so a seat that produced nothing
 usable is named in the closing lines of every command rather than only in
 the calibration's coverage sentence.

 @param env - environment the provider keys, the Hyper pace and the Bedrock
 ledger's place are read from: `process.env` in a run, a test's own otherwise

 @param transport - HTTP every provider client calls over: `fetchTransport`
 in a run. Both are REQUIRED (ledger M43, M68); `createRunClient` is how a
 runner hands over the process's own

 @param promptPayloadDir - optional durable payload checkpoint beneath run root

 @param passLeftMs - what the pass has left, which bounds the seat reader's wait for NeuralWatt's
 return; absent outside a pass

 @param off - providers left off, whose keys are read as unset; the owner's rulings by
 default (`PROVIDERS_LEFT_OFF`), an empty list in a case that exercises an off provider's client

 @returns Ready client, routed across configured providers and counted per seat

 @throws {@link RunConfigError} when every provider key variable is unset or empty

 @example
 ```ts
 const client = runClientFrom({ env: { TRANSLATION_REPAIR_SYNTHETIC_API_KEY: 'test', }, transport, },);
 ```
 */
export function runClientFrom(
  {
    env,
    transport,
    promptPayloadDir,
    passLeftMs,
    off,
  }: {
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly transport: ModelTransport;
    readonly promptPayloadDir?: string;
    readonly passLeftMs?: () => number;
    readonly off?: readonly ProviderOff[];
  },
): RunClient {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: runClientFrom.name,
    l,
  },);

  /**
   Every configured provider's client and the budget view over them.
   */
  const {
    decisions,
    budgets,
    credentialHolds,
    ...clients
  } = configureProviders({
    env,
    transport,
    ...((off === undefined) ? {} : { off, }),
  },);
  /**
   Each configured provider's client, absent where its key is unset.
   */
  const {
    neuralwatt,
    synthetic,
    hyper,
    bedrock,
    deepseek,
    xiaomi,
    openrouter,
  } = clients;

  // THE HOLDS ON A REFUSED CREDENTIAL come from `configureProviders`, which gives them to the budgets too, so the
  // `METERS` line reads a held provider as dry as the router and `providerDryness` do. The router reads them to
  // route, and `providerDryness` reads them for the benches. ONE CLIENT PER RUN is what a corpus pass builds
  // (`corpus-pass-run.ts`), so a 401 is learned once per run there; the commands that build a client per slice or
  // per arm learn it again for each.

  /**
   Routed client with stable compatibility quota surface.
   */
  const routed: ModelClient = {
    ...createRoutingClient({
      // THE CONFIGURED CLIENTS AND NO OTHERS: a provider with no key has no
      // caller, reads dry, and is never routed to (ledger T8). A stand-in
      // caller refusing by name stood here for each absent provider until
      // 2026-09-29, and no call could reach one.
      callers: clients,
      budgets,
      credentialHolds,
      // The providers whose wet reading keeps the run going; a wet provider outside it is no budget
      // to buy writing with.
      writing: RUN_WRITING_PROVIDERS,
      // Hyper paces its calls to the account's hourly limit; while that
      // window is full, a model another provider serves goes there instead
      // of queueing (class one hundred forty-nine).
      // Xiaomi documents 100 requests a minute for its one model; its window
      // is full before the account's limit answers 429.
      paces: {
        ...((hyper === undefined) ? {} : { hyper: hyper.pace, }),
        ...((xiaomi === undefined) ? {} : { xiaomi: xiaomi.pace, }),
      },
      // Conditional spread keeps the decisions transport absent where the
      // OpenRouter key is unset.
      ...((decisions === undefined) ? {} : { decider: decisions, }),
    },),
    quotas: synthetic?.quotas ?? unconfiguredSyntheticQuota,
  };

  rl.debug(
    `provider configuration neuralwatt=${String(neuralwatt !== undefined,)} synthetic=${String(synthetic !== undefined,)} hyper=${
      String(hyper !== undefined,)
    } bedrock=${String(bedrock !== undefined,)} deepseek=${String(deepseek !== undefined,)} xiaomi=${
      String(xiaomi !== undefined,)
    } openrouter=${String(openrouter !== undefined,)}`,
  );

  return {
    // THE TALLY SITS OUTSIDE THE UNIQUENESS WRAPPER (ledger P6), where a JSON
    // call settles as the outcome its caller gets. Inside it, the tally saw
    // only the `chatText` the wrapper buys JSON through, so every reply that
    // arrived counted as usable ("SEAT inception/mercury-2.5 asked=1007
    // usable=1007 unusable=0" beside 40 schema losses). A replayed payload
    // counts as an ask, since its answer is this run's evidence all the same.
    ...seatTallyClient({
      inner: promptUniqueClient({
        // THE OWNER'S CULL AT THE DOOR (ledger P4): a culled model is refused
        // before any provider is asked, whatever path named it, and tallied
        // as unreachable, not as an ask (ledger B711).
        inner: refusingCulledSeats({
          inner: routed,
          culled: OWNER_CULLED,
        },),
        ...((promptPayloadDir === undefined)
          ? {}
          : { store: promptPayloadStore({ dir: promptPayloadDir, },), }),
      },),
      tally: RUN_SEATS,
      // A SEAT THAT THREW ON EVERY ASK IS WITHHELD: refused at this door, and
      // left out of the gathers' benches, so no later stage asks it again. A
      // refusal the router gives is no ask, so it never counts toward this.
      minAsks: SEAT_WITHHOLD_ASKS,
    },),
    // THE VIEW THE SEAT READERS AND BENCHES ARE DERIVED FROM READS A PROVIDER
    // WHOSE CREDENTIAL WAS REFUSED AS DRY: its meter reads money left and says
    // nothing of whether the key is accepted (zheermao19).
    providerDryness: async function providerDryness({ signal, },): Promise<BudgetView> {
      return foldCredentialHolds({
        dry: await budgets.read({ signal, },),
        holds: credentialHolds,
      },);
    },
    providerMeterStates: budgets.readStates,
    providerHolds: budgets.holds,
    neuralwattReturn: budgets.neuralwattReturn,
    neuralwattDryness: budgets.neuralwattDryness,
    neuralwattHeldByReturn: budgets.neuralwattHeldByReturn,
    writingProviders: RUN_WRITING_PROVIDERS,
    // Conditional spread keeps the bound absent outside a pass.
    ...((passLeftMs === undefined) ? {} : { passLeftMs, }),
  };
}

/**
 The run client a runner calls over: the process's environment and the live
 transport, handed to `runClientFrom`. The one place those two are named for
 every runner, so no library function or test reaches them by default.

 @param options - payload checkpoint, when the runner keeps one

 @returns Ready client, routed across configured providers and counted per seat

 @throws {@link RunConfigError} when every provider key variable is unset or empty

 @example
 ```ts
 const client = createRunClient();
 ```
 */
export function createRunClient(
  options: {
    readonly promptPayloadDir?: string;
    readonly passLeftMs?: () => number
  } = {},
): RunClient {
  return runClientFrom({
    ...options,
    env: process.env,
    transport: fetchTransport,
  },);
}

//endregion Corpus-run configuration
