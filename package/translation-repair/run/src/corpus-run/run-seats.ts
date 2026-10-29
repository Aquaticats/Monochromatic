import {
  type BudgetView,
  NO_PROVIDER,
  OPENROUTER_WITHHELD,
  providerServing,
  reachOf,
  visionReachOf,
} from '@monochromatic-dev/translation-repair-provider/ts';
import {
  holdSet,
  type ProviderName,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  assertCheckerIndependence,
  assertCheckerQuorumReachable,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import type { TranslateModels, } from '@monochromatic-dev/translation-repair-translate-lane/ts';
import {
  RUN_CHECKER_ORDER,
  RUN_DECISION_JUDGES,
  RUN_LATE_JUDGES,
  RUN_MODELS,
  RUN_READER_MODELS,
  RUN_ROSTER,
  RUN_TRANSLATE_MODELS,
  RUN_TRANSLATORS,
  RUN_WIDE_SEATS,
  RUN_WRITERS,
  type RunRepairModels,
} from './run-config.ts';

//region Provider-aware judge seats
// WHICH JUDGES SIT, DECIDED FROM WHO WOULD SERVE THEM. The owner's decision of
// 2026-09-03 ("seat per provider reach"): a seat is withheld while the
// provider that would take its calls is one that serves it too slowly for the
// round window, or one the owner declined to pay that model's rate on, and
// seated otherwise.
//
// THE FIRST CASE. `hf:Qwen/Qwen3.8-27B` served by Hyper reasons past the 60 s
// round window in every judge role: cut in 30 of 34 translate-lane and 21 of
// 24 consolidation-slate select rounds on XIEPT2 with Hyper the only provider,
// and in 14 of 19, 11 of 19, 15 of 19 panel and 17 of 19 lane-contest rounds
// on Carena0442 (1,648 of 1,938 calls on Hyper). Served by Synthetic (Toka_ls,
// 2026-09-02) it answered 25 of 28 select rounds. The seat is lost to one
// provider's serving speed, not to the model, so dropping it outright (the
// 2026-09-03 morning's `4ad08d5dc`) threw away a judge Synthetic serves well.
//
// THE SECOND CASE. `hf:moonshotai/Kimi-K3` on OpenRouter costs 3 and 15 USD
// per million tokens, 52 to 61 percent of an entry's all-OpenRouter cost, and
// the owner decided on 2026-09-03 to withhold it wherever only OpenRouter
// would buy it (`doc/decision/translation-repair-openrouter-fallback.md`).
// That reaches its checker seat too, and the next checker of the measured
// order (`RUN_CHECKER_ORDER`) a wet provider serves takes the seat, so the
// static bench stays three wide although `assertCheckerQuorumReachable` has
// held a floor of two since the owner's answer of 2026-10-25 ("Allow two
// checkers").
//
// WHERE A MODEL WOULD BE SERVED is the router's own answer: the first provider
// in `PROVIDER_ORDER` that serves the model and reads wet, holds folded in
// (`providerServing`). The seat cannot follow every call, since a burst can
// overflow some of a wet Synthetic's calls to Hyper; it follows the state that
// decides most of them.
//
// READ AT EACH PHASE BOUNDARY (lanes, lane contest, consolidation), because a
// provider can run dry inside an entry: XIEPT2 read wet at 08:16 UTC on
// 2026-09-03, Synthetic ran dry at 08:19 UTC, and a once-per-entry reading left the Hyper-slow seat
// asked for three and a half hours, abandoned in 102 judge calls.
//
// AN UNREADABLE VIEW SEATS THE FULL BENCH. A budget read that fails is not
// evidence of dryness; the router will still route each call by what it
// learns at the wire, and a seat asked in vain costs one cut, while a seat
// withheld on a guess costs a voice.

/**
 Models that Hyper serves too slowly for the round window in every judge
 role: withheld from each seat they hold while Hyper is the provider that
 would serve them.

 THE ONE MEMBER, Qwen3.8-27B, HOLDS NO JUDGE SEAT NOW: its card keeps it off
 the wide benches (owner, 2026-10-29, ledger B1030) and the late bench (owner,
 2026-10-22, ledger B815), and the checker order does not list it. What this
 set withholds is its translator, picture reader, consolidation writer and
 roster seats, which the owner's answers did not take away.
 */
export const HYPER_SLOW_JUDGES: ReadonlySet<RosterModelId> = new Set<RosterModelId>(['hf:Qwen/Qwen3.8-27B',],);

/**
 Judges that Hyper serves too slowly in the SELECT role alone (both lanes'
 slate select and the consolidation slate), and fast enough everywhere else:
 withheld from the select seats while Hyper would serve them, kept as
 critics, panel, contest judges and gate.

 THE CASE IS `hf:moonshotai/Kimi-K3`, 2026-09-03: cut in 0 of 69 select
 rounds when Synthetic served it (Toka_ls, 2026-09-02) and in 43 of 83 and 38
 of 101 when Hyper mostly or wholly did (XIEPT2 rerun5, 55 of its 61 cut
 streams Hyper-served, and the postscript run), against 0 and 1 of 28
 lane-contest rounds, 0 of 9 critic, 0 to 1 of 5 panel and 0 of 14 gate
 rounds. Its cut streams ran 71 s on average, its answers 14 s: the slate
 prompt is where its reasoning runs long. Same evidence bar as the owner's
 authorisation to drop a model from a role.
 */
export const HYPER_SLOW_SELECT_JUDGES: ReadonlySet<RosterModelId> = holdSet({ hold: 'hyper-slow-select', },);

/**
 Every bench one entry runs with, derived from one reading.

 @example
 ```ts
 const seats: JudgeSeats = judgeSeatsFor({ dry, },);
 ```
 */
export type JudgeSeats = {
  /**
   Which providers were dry when the seats were derived.
   */
  readonly dry: BudgetView;

  /**
   Models withheld from at least one seat by this reading, for the log line.
   */
  readonly withheld: readonly RosterModelId[];

  /**
   Seats on at least one bench that no wet provider serves in this reading,
   named once, in bench order. They stay seated and are lost at call time as
   unreachable (the owner's rule of 2026-09-09), so the `JUDGE SEATS` lines
   name them: a bench count includes them, and the checker bench is padded
   with them when fewer than its width are served (ledger B711).
   */
  readonly unserved: readonly RosterModelId[];

  /**
   Critics and adjudication panel.
   */
  readonly wideSeats: readonly RosterModelId[];

  /**
   Both lanes' slate select judges: the wide seats less the select-slow
   judges Hyper would serve.
   */
  readonly selectJudges: readonly RosterModelId[];

  /**
   Lane contest and consolidation gate.
   */
  readonly lateJudges: readonly RosterModelId[];

  /**
   Consolidation slate judges: the late judges less the select-slow judges
   Hyper would serve.
   */
  readonly slateJudges: readonly RosterModelId[];

  /**
   Checkers: the best-ranked of the measured order this reading serves,
   padded with unserved ranked seats only while fewer than three are served.
   */
  readonly checkers: readonly RosterModelId[];

  /**
   Translate lane writers: the static translators less any model withheld
   on the provider that would serve it. A WITHHELD MODEL WRITES NOTHING
   EITHER: the owner's withholding is on what a call costs, whatever the
   role, and the first OpenRouter-only pass (keyword233, 2026-09-03 18:15
   UTC) bought six translations from the withheld model, a quarter of that
   pass's bill, while every judge bench had it out.
   */
  readonly translators: readonly RosterModelId[];

  /**
   Picture readers: the catalog-derived readers less any withheld model,
   for the same reason as the translators, with an image the dearest call
   of all.
   */
  readonly readers: readonly RosterModelId[];

  /**
   The readers a wet provider can show a picture in this reading: the
   readers bench less the seats only a text route is wet for. The seat line
   prints its count beside the bench's, because a reader with no wet
   picture route stays seated and is lost at call time (XingZ6016 printed
   `readers=5` while two could see).
   */
  readonly readersSeeing: readonly RosterModelId[];

  /**
   Consolidation writers for this reading: the measured writers less any
   withheld model.
   */
  readonly writers: readonly RosterModelId[];

  /**
   The whole roster less any withheld model, for every stage that asks the
   roster rather than a named bench: block pairing and archive review in
   preparation, insertion admission, and the consolidation writers. The
   second OpenRouter-only pass (2026-09-03 19:33 UTC) bought the withheld
   model's first call from the pairing round, six seconds before any bench
   was read.
   */
  readonly roster: readonly RosterModelId[];

  /**
   Repair lane roles with the wide, select and checker seats applied, the
   run's roster otherwise, so the refiners every run seats stay required and
   the consolidation's naturalness roles never read as off (ledger T8).
   */
  readonly repairModels: RunRepairModels;

  /**
   Translate lane roles with the translator and select seats applied.
   */
  readonly translateModels: TranslateModels;
};

/**
 Derives the benches for one reading of every provider's meter.

 @param dry - which providers have nothing buyable, holds folded in

 @returns Benches, with each withholding applied where its provider would
 serve the seat, and the checker floor kept

 @example
 ```ts
 const seats = judgeSeatsFor({ dry: { synthetic: true, hyper: true, openrouter: false, }, },);
 ```
 */
export function judgeSeatsFor(
  { dry, }: { readonly dry: BudgetView; },
): JudgeSeats {
  /**
   Provider the router would send one model's calls to, or none.

   @param modelId - seat under question

   @returns First provider in order that serves it and reads wet
   */
  function servedBy(modelId: RosterModelId,): ProviderName | typeof NO_PROVIDER {
    return providerServing({
      reach: reachOf({ modelId, },),
      dry,
    },);
  }
  /**
   Keeps a seat unless the provider that would serve it is one the seat is
   withheld on.

   A SEAT WITHHELD ON OPENROUTER READS AS UNSERVED THERE since 2026-09-09
   (`reachOf`), so the router can never re-route it there mid-phase; what
   this reader withholds is the seat whose only wet provider would have been
   OpenRouter, which is the same seat as before.

   @param modelId - seat under question

   @returns Whether the seat is asked this phase
   */
  function seated(modelId: RosterModelId,): boolean {
    /**
     Where this model's calls would go.
     */
    const provider = servedBy(modelId,);
    if ((provider === 'hyper') && HYPER_SLOW_JUDGES.has(modelId,))
      return false;
    /**
     Whether OpenRouter is the one wet provider that would have served it.
     */
    const onlyOpenRouterWould = (provider === NO_PROVIDER)
      && OPENROUTER_WITHHELD.has(modelId,)
      && (!dry.openrouter);
    return !onlyOpenRouterWould;
  }
  /**
   Keeps a select seat unless Hyper would serve it and serves its slate
   answers too slowly, on top of {@link seated}.

   @param modelId - select seat under question

   @returns Whether the seat judges slates this phase
   */
  function seatedForSelect(modelId: RosterModelId,): boolean {
    return (servedBy(modelId,) !== 'hyper') || (!HYPER_SLOW_SELECT_JUDGES.has(modelId,));
  }
  /**
   Wide bench for this reading.
   */
  const wideSeats = RUN_WIDE_SEATS.filter(seated,);
  /**
   Both lanes' select judges for this reading.
   */
  const selectJudges = [
    ...wideSeats.filter(seatedForSelect,),
    // A DECISION-ONLY SEAT IS SERVED BY OPENROUTER ALONE, off the chat
    // reach, so it is seated by that one meter and nothing else.
    ...RUN_DECISION_JUDGES.filter(function decisionSeated(): boolean {
      return !dry.openrouter;
    },),
  ];
  /**
   Late bench for this reading.
   */
  const lateJudges = RUN_LATE_JUDGES.filter(seated,);
  /**
   Consolidation slate judges for this reading.
   */
  const slateJudges = lateJudges.filter(seatedForSelect,);
  /**
   The static checker roster, whose size is the bench's width.
   */
  const staticCheckers = RUN_MODELS.checkerModelIds;
  /**
   Whether a wet provider would take a seat's calls under this reading.

   @param modelId - seat under question

   @returns True when the router would send the seat's calls somewhere
   */
  function served(modelId: RosterModelId,): boolean {
    return servedBy(modelId,) !== NO_PROVIDER;
  }
  /**
   Measured checkers this reading does not withhold, in rank order.
   */
  const seatable = RUN_CHECKER_ORDER.filter(seated,);
  /**
   The bench: the best-ranked checkers a wet provider serves, then, only
   while fewer than the width are served, the next-ranked seats no
   provider serves, which a stage reads as unreachable (the owner's rule
   of 2026-09-09) rather than the bench falling below the contract's floor.

   UNTIL 2026-09-27 THE STATIC THREE SAT WHOEVER SERVED THEM, with one
   substitute for a withheld seat: five of the six runs before that day
   read Synthetic and Hyper dry and asked Qwen3.8-27B, which nobody served,
   beside two checkers the benchmark then measured at 35 fixes resolved and
   9 unchanged texts wrongly resolved of 85 each (ledger L1).
   */
  const checkers = [
    ...seatable.filter(served,),
    ...seatable.filter(function unserved(modelId,): boolean {
      return !served(modelId,);
    },),
  ].slice(
    0,
    staticCheckers.length,
  );
  // THE DERIVED ROSTER MUST PASS WHAT THE STATIC ONE PASSES AT LOAD, or a
  // phase would start with a checker stage the contract refuses.
  assertCheckerIndependence({
    editorModelIds: RUN_MODELS.editorModelIds,
    refinerModelIds: RUN_MODELS.refinerModelIds,
    checkerModelIds: checkers,
    selfCertificationPermitted: RUN_MODELS.checkerSelfCertificationPermitted,
  },);
  assertCheckerQuorumReachable({ checkerModelIds: checkers, },);
  /**
   Translate lane writers for this reading.
   */
  const translators = RUN_TRANSLATORS.filter(seated,);
  /**
   Picture readers for this reading.
   */
  const readers = RUN_READER_MODELS.filter(seated,);
  /**
   Picture readers a wet provider can show a picture.
   */
  const readersSeeing = readers.filter(function sees(modelId: RosterModelId,): boolean {
    return providerServing({
      reach: visionReachOf({ modelId, },),
      dry,
    },) !== NO_PROVIDER;
  },);
  /**
   Consolidation writers for this reading.
   */
  const writers = RUN_WRITERS.filter(seated,);
  /**
   The roster for this reading.
   */
  const roster = RUN_ROSTER.filter(seated,);
  /**
   Every static seat holder, repeated where a model holds several seats.
   */
  const seatHolders = [
    ...RUN_WIDE_SEATS,
    ...RUN_LATE_JUDGES,
    ...staticCheckers,
    ...RUN_TRANSLATORS,
    ...RUN_READER_MODELS,
    ...RUN_WRITERS,
    ...RUN_ROSTER,
  ];
  /**
   Every model some bench lost to this reading, named once.
   */
  const withheld = seatHolders.filter(function lostASeat(
    modelId: RosterModelId,
    index: number,
    all: readonly RosterModelId[],
  ): boolean {
    /**
     Whether this is the first mention of the model.
     */
    const first = all.indexOf(modelId,) === index;
    /**
     Whether the model keeps every seat it holds.
     */
    const keepsAll = seated(modelId,) && seatedForSelect(modelId,);
    return first && (!keepsAll);
  },);
  /**
   Every model on a bench of this reading, named once in bench order.
   */
  const benched = [
    ...wideSeats,
    ...lateJudges,
    ...checkers,
    ...translators,
    ...readers,
    ...writers,
    ...roster,
  ].filter(function firstMention(
    modelId: RosterModelId,
    index: number,
    all: readonly RosterModelId[],
  ): boolean {
    return all.indexOf(modelId,) === index;
  },);
  /**
   The benched models no wet provider serves in this reading.
   */
  const unserved = benched.filter(function notServed(modelId: RosterModelId,): boolean {
    return !served(modelId,);
  },);
  return {
    dry,
    withheld,
    unserved,
    wideSeats,
    selectJudges,
    lateJudges,
    slateJudges,
    checkers,
    translators,
    readers,
    readersSeeing,
    writers,
    roster,
    repairModels: {
      ...RUN_MODELS,
      criticModelIds: wideSeats,
      panelModelIds: wideSeats,
      judgeModelIds: selectJudges,
      checkerModelIds: checkers,
    },
    translateModels: {
      ...RUN_TRANSLATE_MODELS,
      translatorModelIds: translators,
      judgeModelIds: selectJudges,
    },
  };
}

//endregion Provider-aware judge seats
