import {
  type BudgetView,
  NO_PROVIDER,
  providerServing,
  reachOf,
  visionReachOf,
} from '@monochromatic-dev/translation-repair-provider/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { rosterQuorumSize, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Bench quorum under a named hold
// WHETHER A PHASE'S BENCHES CAN SETTLE AT ALL, read before the phase starts.
//
// THE THIRTEENTH CLASS, found by the third hakureico pass of 2026-09-07. Hyper
// named its return in 538 s at 21:57 UTC and `markRefused` held it out for exactly
// that (the twelfth class working as built). Bedrock stayed wet, so nothing
// waited: the translate lane started at 21:58 UTC with every Hyper-only writer
// refused as `NoProviderForModelError` in the same millisecond, the lane
// contest and the consolidation at 22:00 UTC ran on the two Bedrock seats alone,
// and every consolidation round read `quorum-not-met` at 0 ms. The footnote
// passage the archive lacks stayed unfilled and three contested slices
// settled on nobody, nine minutes before the provider came back.
//
// THE ROUTER ALREADY WAITS, but only when EVERY provider reads dry
// (`readBudgetsPastHolds`), because that is the case where the alternative is
// ending the run. A phase is the same case one level up: when the bench a
// phase leans on cannot reach quorum among the seats a wet provider can
// serve, running it buys a settlement over nobody, and a hold that names its
// return is a promise the bench will be back. So the seat reader waits out
// the shortest running hold once, as the router does, and reads again.
//
// PER PHASE, NOT PER CALL. A call carries its own deadline
// (`RUN_PER_CALL_TIMEOUT_MS`, 360 s), which a daily-limit hold outlasts, so a
// wait inside a call would be cut by the deadline it was trying to serve. The
// seat reading happens outside every exchange, and the lanes re-read before
// the translate lane starts, which is where the third pass lost its writers.

/**
 Benches a phase may lean on, by the name the `JUDGE SEATS` line prints.

 @example
 ```ts
 const bench: BenchName = 'translators';
 ```
 */
export type BenchName =
  | 'wide'
    | 'select'
    | 'slate'
    | 'editors'
    | 'refiners'
    | 'translators'
    | 'readers';

/**
 Phase a seat reading is taken for.

 `translate lane` since the thirteenth class: the lanes phase seats both
 lanes at once, and the translate lane starts after the repair lane has
 spent minutes, which is long enough for a provider to be held out.

 @example
 ```ts
 const phase: JudgeSeatPhase = 'lane contest';
 ```
 */
export type JudgeSeatPhase =
  | 'preparation'
    | 'pictures'
    | 'insertion admission'
    | 'lanes'
    | 'translate lane'
    | 'lane contest'
    | 'consolidation'
    | 'page consistency';

/**
 Which benches each phase cannot run without.

 Preparation pairs blocks and reviews the archive with the wide bench; the
 pictures phase reads with the readers; the lanes need critics, panel and
 judges (wide), editors and refiners for repair and writers for translate
 (the writing benches since the fifteenth class, `run-seats-floor.ts`); the translate lane
 needs its writers and its slate judges; the contest judges with the wide
 bench; consolidation writes with the roster and judges with the slate and
 gates with the late bench, of which the slate is the narrower. The
 insertion admission, which asks the roster whether the archive already
 covers each source-only passage, re-reads its seats under a hold as its own
 phase (ledger X12) and leans on the wide bench, as the preparation does. The
 page consistency pass asks the late judge bench (`seats.lateJudges`) which
 rendering of a repeated wording every place should follow. The table has no
 name for the late bench and reads the wide bench in its place, as the lane
 contest's row does; the late bench holds every seat of the wide bench and one
 more (`RUN_LATE_JUDGES` against `RUN_WIDE_SEATS`), so the wide bench's
 shortfall is the conservative reading of the late bench's. The pass leans on
 the translators as well: it asks the translate stage's writer bench
 (`seats.translators`) to carry the chosen rendering into each receiving block, so a
 phase that read only the wide bench would neither wait for the writers nor
 check them before the carry asks for them. A writer bench below its floor
 with no hold to wait out does not stop this phase
 (`PHASES_WITHOUT_WRITER_STOP` in `run-seats-floor.ts`): the reading says so
 once and the pass goes on without writers, choosing between the English a
 place has and the chosen words moved in by code, and records that the
 writers were not asked.
 */
const BENCHES_BY_PHASE: Readonly<Record<JudgeSeatPhase, readonly BenchName[]>> = {
  preparation: ['wide',],
  pictures: ['readers',],
  'insertion admission': ['wide',],
  lanes: [
    'wide',
    'editors',
    'refiners',
    'translators',
  ],
  'translate lane': [
    'translators',
    'select',
  ],
  'lane contest': ['wide',],
  consolidation: [
    'slate',
    'wide',
  ],
  'page consistency': [
    'wide',
    'translators',
  ],
};

/**
 Voices the readers bench needs: two, since `readImagePair` corroborates a
 picture from two readings and calls a single one `one-reader-only`. The
 judge benches' quorum is a majority of a bench that votes; readers never
 vote, and holding them to it would call a Bedrock-only pass with its two
 measured readers short before every pictures phase (2026-09-08).
 */
const READER_QUORUM = 2;

/**
 Names the benches one phase cannot run without.

 @param phase - phase about to start

 @returns Bench names in the order the shortfall line prints them

 @example
 ```ts
 phaseBenches({ phase: 'translate lane', },);
 // => ['translators', 'select',]
 ```
 */
export function phaseBenches(
  { phase, }: { readonly phase: JudgeSeatPhase; },
): readonly BenchName[] {
  return BENCHES_BY_PHASE[phase];
}

/**
 Seats some wet provider would take a call for.

 @param seats - bench under question

 @param dry - dryness per provider, holds folded in

 @param reach - which providers count: those that serve the seat text
 (`text`, the default) or those that can show it a picture (`vision`). A
 reader is counted by the second, since a picture reaches a narrower set of
 providers than text does (`visionReachOf`): XingZ6016 counted Gemma reachable
 through OpenRouter, which serves it text only.

 @returns Seats the router could send somewhere right now

 @example
 ```ts
 const reachable = reachableSeats({ seats: wideSeats, dry, },);
 ```
 */
export function reachableSeats(
  {
    seats,
    dry,
    reach = 'text',
  }: {
    readonly seats: readonly RosterModelId[];
    readonly dry: BudgetView;
    readonly reach?: 'text' | 'vision';
  },
): readonly RosterModelId[] {
  return seats.filter(function isServed(modelId,): boolean {
    return providerServing({
      reach: (reach === 'vision') ? visionReachOf({ modelId, },) : reachOf({ modelId, },),
      dry,
    },) !== NO_PROVIDER;
  },);
}

/**
 A bench that cannot reach quorum among the seats a wet provider serves, with the count behind the clause.
 */
export type BenchShortfall = {
  /**
   Bench that stands short.
   */
  readonly name: BenchName;

  /**
   Seats the bench holds.
   */
  readonly seats: readonly RosterModelId[];

  /**
   Clause naming the reachable seats against the quorum.
   */
  readonly clause: string;
};

/**
 Benches that cannot reach quorum among the seats a wet provider serves.

 @param benches - each bench the phase leans on, keyed by name

 @param names - benches to read, in the order the shortfall line prints them

 @param dry - dryness per provider, holds folded in

 @returns One entry per short bench, empty when every bench can settle

 @example
 ```ts
 const shortfalls = benchShortfalls({ benches, names: phaseBenches({ phase, },), dry, },);
 ```
 */
export function benchShortfalls(
  {
    benches,
    names,
    dry,
  }: {
    readonly benches: Readonly<Record<BenchName, readonly RosterModelId[]>>;
    readonly names: readonly BenchName[];
    readonly dry: BudgetView;
  },
): readonly BenchShortfall[] {
  return names.flatMap(function shortfallOf(name,): readonly BenchShortfall[] {
    /**
     Seats this bench holds.
     */
    const seats = benches[name];
    /**
     Voices the bench needs to settle: a pair for the readers, a majority
     for a bench that votes.
     */
    const quorum = (name === 'readers')
      ? READER_QUORUM
      : rosterQuorumSize({ rosterSize: seats.length, },);
    /**
     Seats a wet provider would serve.
     */
    const reachable = reachableSeats({
      seats,
      dry,
      reach: (name === 'readers') ? 'vision' : 'text',
    },);
    if (reachable.length >= quorum)
      return [];
    return [
      {
        name,
        seats,
        clause: `${name} ${String(reachable.length,)} of ${String(seats.length,)} reachable, quorum ${String(quorum,)}`,
      },
    ];
  },);
}

/**
 Benches that cannot reach quorum among the seats a wet provider serves.

 @param benches - each bench the phase leans on, keyed by name

 @param names - benches to read, in the order the shortfall line prints them

 @param dry - dryness per provider, holds folded in

 @returns One clause per short bench naming reachable seats against the
 quorum, empty when every bench can settle

 @example
 ```ts
 shortBenches({ benches, names: phaseBenches({ phase, },), dry, },);
 // => ['translators 2 of 6 reachable, quorum 3',]
 ```
 */
export function shortBenches(
  {
    benches,
    names,
    dry,
  }: {
    readonly benches: Readonly<Record<BenchName, readonly RosterModelId[]>>;
    readonly names: readonly BenchName[];
    readonly dry: BudgetView;
  },
): readonly string[] {
  return benchShortfalls({
    benches,
    names,
    dry,
  },)
    .map(function clauseOf({ clause, },): string {
      return clause;
    },);
}

//endregion Bench quorum under a named hold
