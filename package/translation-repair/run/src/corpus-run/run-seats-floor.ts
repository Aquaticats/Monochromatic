import type {
  BudgetView,
  NeuralWattDryness,
} from '@monochromatic-dev/translation-repair-provider/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { JudgeSeats, } from './run-seats.ts';
import {
  type BenchName,
  type JudgeSeatPhase,
  reachableSeats,
} from './run-seats-wait.ts';

//region Writing-bench floor
// THE FIFTEENTH CLASS, the owner's decision of 2026-09-08. The eighth
// hakureico pass ran on Bedrock alone: no editor or refiner had a Bedrock
// seat (`editor round: 0/3 heard` 40 times, `refiner round: 0/3 heard` 52
// times), one translator of seven was reachable, and the pass settled a page
// with five slices that one writer wrote and three judges chose, zero
// repairs and no footnote, as `SETTLED`, told apart from the sixth pass's
// whole-bench page only in its findings. The owner chose that such a pass
// stops INCOMPLETE and leaves the entry for a pass that has the benches
// (`doc/decision/translation-repair-writing-bench-floor.md`).
//
// TWO THRESHOLDS, TWO MEANINGS. A bench short of its quorum under a named
// hold waits (the thirteenth class), and with no hold runs on what is
// reachable and says so. A WRITING bench below its floor with no hold does
// not run: an editor bench of none writes nothing, and a translator bench of
// one writes without a slate, which is the one-writer page. The floor is the
// pair a slate needs, the same number the readers are held to.

/**
 Benches that write text rather than judge it, which the floor applies to: a
 DELIBERATE SUBSET of `BenchName`.

 LEFT OUT are `wide`, `select` and `slate`, which judge, and `readers`, which
 transcribe pictures and are held to a pair by their own quorum in
 `run-seats-wait.ts`. A bench gained later is therefore not a writing bench
 until it is added here.
 */
export const WRITING_BENCHES: ReadonlySet<BenchName> = new Set<BenchName>([
  'editors',
  'refiners',
  'translators',
],);

/**
 Reachable seats a writing bench needs to write a slate rather than a single
 text: a pair, so that a judge has two candidates to choose between.
 */
export const WRITING_BENCH_FLOOR = 2;

/**
 Stops an entry whose phase cannot write: a writing bench it leans on has
 fewer reachable seats than the floor and no provider has named a return the pass can wait for.

 WHEN NEURALWATT'S RETURN WOULD LIFT THE BENCH TO ITS FLOOR the error carries the return as well (the distance, the
 time of day and the spent day, as the all-dry refusal carries them), and the pass may park for it instead of leaving
 the entry to an error (`pass-dry-stop.ts`). It is carried only when the roster says the return reaches the floor.

 @example
 ```ts
 throw new WritingBenchUnreachableError({ phase: 'lanes', clauses: ['editors 0 of 3 reachable, floor 2',], },);
 ```
 */
export class WritingBenchUnreachableError extends Error {
  /**
   Message carries bench names and counts only, nothing from a provider.
   */
  readonly messageNamesOnly: true = true;

  /**
   One clause per bench below the floor, as the seats line prints them.
   */
  readonly clauses: readonly string[];

  /**
   Milliseconds until NeuralWatt's spent day starts again, present only when that return lifts the bench to its floor.
   */
  readonly neuralwattReturnsInMs?: number;

  /**
   The time of day the return falls at, with its zone, present with the distance.
   */
  readonly neuralwattReturnsAt?: string;

  /**
   Why NeuralWatt read dry, present with the distance so the pass tells one spent day from the next.
   */
  readonly neuralwattDryness?: NeuralWattDryness;

  /**
   The instant of the seat reading that named the return, on the monotonic clock (`monotonicMs`), present with the
   distance so a park waits from the read and not from the moment the entry's failure reaches the gate.
   */
  readonly neuralwattReadAtMs?: number;

  /**
   Names the phase that could not write and each bench below the floor.

   @param phase - phase the reading was taken for

   @param clauses - one clause per bench below the floor

   @param neuralwattReturnsInMs - milliseconds until NeuralWatt's day starts again, where that return
   lifts the bench to its floor

   @param neuralwattReturnsAt - time of day the return falls at, with its zone

   @param neuralwattDryness - why NeuralWatt read dry

   @param neuralwattReadAtMs - instant of the reading that named the return, on `monotonicMs`
   */
  public constructor(
    {
      phase,
      clauses,
      neuralwattReturnsInMs,
      neuralwattReturnsAt,
      neuralwattDryness,
      neuralwattReadAtMs,
    }: {
      readonly phase: string;
      readonly clauses: readonly string[];
      readonly neuralwattReturnsInMs?: number;
      readonly neuralwattReturnsAt?: string;
      readonly neuralwattDryness?: NeuralWattDryness;
      readonly neuralwattReadAtMs?: number;
    },
  ) {
    /**
     The sentence about the returns, one of two fixed sentences that name at most the UTC time of day
     NeuralWatt's day starts at.
     */
    const returnSentence = (neuralwattReturnsInMs === undefined)
      ? 'no provider has named a return the pass can wait for, so the entry stops for a pass that has the bench'
      : `NeuralWatt's energy returns at ${neuralwattReturnsAt ?? 'its day start'}, which would reach the floor, `
        + 'so the entry stops; the pass may park for it where the return fits what the pass has left, and otherwise '
        + 'the entry ends as a failure a later pass resumes';
    super(`writing bench unreachable at ${phase}: ${clauses.join('; ',)}; ${returnSentence}`,);
    this.name = 'WritingBenchUnreachableError';
    this.clauses = clauses;
    if (neuralwattReturnsInMs !== undefined)
      this.neuralwattReturnsInMs = neuralwattReturnsInMs;
    if (neuralwattReturnsAt !== undefined)
      this.neuralwattReturnsAt = neuralwattReturnsAt;
    if (neuralwattDryness !== undefined)
      this.neuralwattDryness = neuralwattDryness;
    if (neuralwattReadAtMs !== undefined)
      this.neuralwattReadAtMs = neuralwattReadAtMs;
  }
}

/**
 Every bench the shortfall readings look at, keyed by the name the seats
 line prints, from one derivation of the seats.

 @param seats - benches as derived for one reading

 @returns Bench per name, the writing benches taken from the repair and
 translate lane rosters

 @example
 ```ts
 const benches = benchesOf({ seats: judgeSeatsFor({ dry, },), },);
 ```
 */
export function benchesOf(
  { seats, }: { readonly seats: JudgeSeats; },
): Readonly<Record<BenchName, readonly RosterModelId[]>> {
  /**
   Repair lane writers; the run's roster always seats refiners
   (`RunRepairModels`).
   */
  const {
    editorModelIds,
    refinerModelIds,
  } = seats.repairModels;
  return {
    wide: seats.wideSeats,
    select: seats.selectJudges,
    slate: seats.slateJudges,
    editors: editorModelIds,
    refiners: refinerModelIds,
    translators: seats.translators,
    readers: seats.readers,
  };
}

/**
 Phases that go on when a writing bench they lean on is below the floor and no
 provider has named its return, rather than stopping the entry. The page
 consistency pass is the one: its writers carry a chosen rendering into a place,
 and without them the pass still chooses between the English a place has and the
 chosen words moved in by code, and records that the writers were not asked.
 */
export const PHASES_WITHOUT_WRITER_STOP: ReadonlySet<JudgeSeatPhase> = new Set<JudgeSeatPhase>([
  'page consistency',
],);

/**
 Writing benches among the named ones that cannot reach the floor among the
 seats a wet provider serves, with the counts the clause prints.

 @param benches - each bench the phase leans on, keyed by name

 @param names - benches the phase leans on, in the order the line prints them

 @param dry - dryness per provider, holds folded in

 @returns One entry per writing bench below the floor: its name and its clause

 @example
 ```ts
 belowFloorWritingBenches({ benches, names: phaseBenches({ phase: 'lanes', },), dry, },);
 // => [{ name: 'editors', clause: 'editors 0 of 3 reachable, floor 2', },]
 ```
 */
export function belowFloorWritingBenches(
  {
    benches,
    names,
    dry,
  }: {
    readonly benches: Readonly<Record<BenchName, readonly RosterModelId[]>>;
    readonly names: readonly BenchName[];
    readonly dry: BudgetView;
  },
): readonly {
  readonly name: BenchName;
  readonly clause: string
}[] {
  return names
    .filter(function writes(name,): boolean {
      return WRITING_BENCHES.has(name,);
    },)
    .flatMap(function entryOf(name,): readonly {
      readonly name: BenchName;
      readonly clause: string
    }[] {
      /**
       Seats this bench holds.
       */
      const seats = benches[name];
      /**
       Seats a wet provider would serve.
       */
      const reachable = reachableSeats({
        seats,
        dry,
      },);
      if (reachable.length >= WRITING_BENCH_FLOOR)
        return [];
      return [
        {
          name,
          clause:
            `${name} ${String(reachable.length,)} of ${String(seats.length,)} reachable, floor ${String(WRITING_BENCH_FLOOR,)}`,
        },
      ];
    },);
}

/**
 Writing benches among the named ones that cannot reach the floor among the
 seats a wet provider serves.

 @param benches - each bench the phase leans on, keyed by name

 @param names - benches the phase leans on, in the order the line prints them

 @param dry - dryness per provider, holds folded in

 @returns One clause per writing bench below the floor, empty when every
 writing bench can write a slate or the phase leans on none

 @example
 ```ts
 unreachableWritingBenches({ benches, names: phaseBenches({ phase: 'lanes', },), dry, },);
 // => ['editors 0 of 3 reachable, floor 2', 'refiners 0 of 3 reachable, floor 2',]
 ```
 */
export function unreachableWritingBenches(
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
  return belowFloorWritingBenches({
    benches,
    names,
    dry,
  },)
    .map(function clauseOf({ clause, },): string {
      return clause;
    },);
}

//endregion Writing-bench floor
