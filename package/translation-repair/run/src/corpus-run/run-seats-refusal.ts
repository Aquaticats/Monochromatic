import {
  type BudgetView,
  dryCauseClausesOf,
  everyWritingProviderDry,
  EveryProviderDryError,
  idleWetProviders,
  type NeuralWattDryness,
  type NeuralWattReturn,
  pastPassWordsOf,
  returnTimeOf,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { monotonicMs, } from '@monochromatic-dev/translation-repair-markup/ts';
import { markFailureWritten, } from '@monochromatic-dev/translation-repair-stage/ts';
import {
  PROVIDER_ORDER,
  type ProviderRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { RunClient, } from './run-client-contract.ts';
import { RUN_WRITING_PROVIDERS, } from './run-config.ts';
import { WritingBenchUnreachableError, } from './run-seats-floor.ts';
import { holdsWithinPass, } from './run-seats-pass-bound.ts';
import { returnLiftsWritingBenches, } from './run-seats-hold-need.ts';
import {
  type JudgeSeatPhase,
  phaseBenches,
} from './run-seats-wait.ts';

//region Seat reading refusal
// THE REFUSAL A SEAT READING STOPS THE ENTRY WITH, split out of `run-seats-read.ts` at its line budget: the writing-bench
// error, or the router's every-provider-dry refusal when that is the cause.

/**
 What the refusal reads off the run client: how long each provider is held, and what the run knows of NeuralWatt's day.
 */
export type RefusalClient = Pick<
  RunClient,
  'neuralwattDryness' | 'neuralwattReturn' | 'passBoundOwner' | 'passLeftMs' | 'providerHolds' | 'writingProviders'
  >;

/**
 What the run knows of NeuralWatt's day, for the words of a wait or a refusal.

 @param client - run client, whose NeuralWatt day is read when it reports one

 @returns The day, or that none was decided where the client reports none

 @example
 ```ts
 const neuralwatt = neuralwattOf({ client, },);
 ```
 */
export function neuralwattOf({ client, }: { readonly client: Pick<RunClient, 'neuralwattReturn'>; },): NeuralWattReturn {
  return client.neuralwattReturn?.() ?? { kind: 'unanchored', };
}

/**
 The providers whose wet reading keeps the run going, as the client reports them.

 @param client - run client, whose record is read when it carries one

 @returns Client's record, or the roster's own where the client carries none

 @example
 ```ts
 const writing = writingProvidersOfClient({ client, },);
 ```
 */
export function writingProvidersOfClient(
  { client, }: { readonly client: Pick<RunClient, 'writingProviders'>; },
): ProviderRecord<boolean> {
  return client.writingProviders ?? RUN_WRITING_PROVIDERS;
}

/**
 The clauses of an unreachable writing bench with why NeuralWatt reads dry, only the true cause: a blocked key states
 its own time, energy held by calls in flight says so, a day the account still shows spent says that, and a spent day
 with a return states the time and the distance.

 @param clauses - one clause per writing bench below its floor

 @param dry - the dryness view the reading stopped on

 @param client - run client, whose NeuralWatt day is read when it reports one

 @param refusalFollows - whether a run-ending refusal follows when every provider that keeps the run going is dry, which
 states the cause itself (default: it does); false for a reading that goes on, where nothing else would state it

 @returns Clauses, with the cause last where NeuralWatt reads dry and the client reports one

 @example
 ```ts
 const stated = withNeuralWattReturn({ clauses: ['editors 0 of 3 reachable, floor 2',], dry, client, },);
 ```
 */
export function withNeuralWattReturn(
  {
    clauses,
    dry,
    client,
    refusalFollows = true,
  }: {
    readonly clauses: readonly string[];
    readonly dry: BudgetView;
    readonly client: Pick<RunClient, 'neuralwattDryness' | 'passBoundOwner' | 'passLeftMs' | 'writingProviders'>;
    readonly refusalFollows?: boolean;
  },
): readonly string[] {
  /**
   The sentence about why NeuralWatt reads dry, empty where NeuralWatt is not dry, the client reports no cause, or every
   provider that keeps the run going is dry and a run-ending refusal follows to state it.
   */
  const beyond = (dry.neuralwatt && ((!refusalFollows) || (!everyWritingProviderDry({
    view: dry,
    writing: writingProvidersOfClient({ client, },),
  },))))
    ? dryCauseClausesOf({
      dryness: client.neuralwattDryness?.() ?? { kind: 'wet', },
      ...((client.passLeftMs === undefined) ? {} : { passLeftMs: client.passLeftMs(), }),
      ...((client.passBoundOwner === undefined) ? {} : { boundOwner: client.passBoundOwner(), }),
    },)
    : [];
  return [
    ...clauses,
    ...beyond,
  ];
}

/**
 The router's refusal for a seat reading that found every provider dry with nothing left to wait out.

 @param phase - phase the reading was taken for

 @param holds - how long each provider's refusal still holds it out

 @param neuralwatt - what the run knows of NeuralWatt's day, left out where the client reports none

 @param neuralwattDryness - why NeuralWatt reads dry, left out where the client reports none

 @param neuralwattReturnsInMs - milliseconds until NeuralWatt's spent day starts again while the return lies ahead,
 which the pass reads to park for it when it has the time; left out where no return lies ahead

 @param neuralwattReadAtMs - instant of the reading that named the return, which the pass counts its wait from

 @param pastPass - words saying that return is past what the pass has left, none where it fits or none is named

 @param dry - dryness view the reading stopped on, whose wet providers the refusal names with their reason

 @param writing - providers whose wet reading keeps the run going

 @returns The refusal, stating the reading it was made on

 @example
 ```ts
 throw everyProviderDryRefusal({ phase: 'lanes', holds: holdsWithinPass({ client, },), dry, writing, pastPass: [], },);
 ```
 */
function everyProviderDryRefusal(
  {
    phase,
    holds,
    neuralwatt,
    neuralwattDryness,
    neuralwattReturnsInMs,
    neuralwattReadAtMs,
    pastPass,
    dry,
    writing,
  }: {
    readonly phase: JudgeSeatPhase;
    readonly holds: ProviderRecord<number>;
    readonly neuralwatt?: NeuralWattReturn;
    readonly neuralwattDryness?: NeuralWattDryness;
    readonly neuralwattReturnsInMs?: number;
    readonly neuralwattReadAtMs?: number;
    readonly pastPass: readonly string[];
    readonly dry: BudgetView;
    readonly writing: ProviderRecord<boolean>;
  },
): EveryProviderDryError {
  /**
   Each provider's reading and hold, in the words the router's own refusal uses.
   */
  const measured = `the seat reading for ${phase} read ${
    PROVIDER_ORDER.map(function dryClause(provider,): string {
      return `${provider} ${dry[provider] ? 'dry' : 'wet (serves no seat that writes)'}`;
    },)
      .join(', ',)
  }; holds ${
    PROVIDER_ORDER.map(function holdClause(provider,): string {
      return `${provider} ${String(holds[provider],)}ms`;
    },)
      .join(', ',)
  }${
    pastPass.map(function sentenceOf(words,): string {
      return `; the NeuralWatt return is ${words}`;
    },)
      .join('',)
  }`;
  return new EveryProviderDryError({
    measured,
    idle: idleWetProviders({
      view: dry,
      writing,
    },),
    // Conditional spread keeps the day absent on a client that reports none.
    ...((neuralwatt === undefined) ? {} : { neuralwatt, }),
    ...((neuralwattDryness === undefined) ? {} : { neuralwattDryness, }),
    ...((neuralwattReturnsInMs === undefined) ? {} : { neuralwattReturnsInMs, }),
    ...((neuralwattReadAtMs === undefined) ? {} : { neuralwattReadAtMs, }),
  },);
}

/**
 What a bench error carries of NeuralWatt's return: the distance, the time of day and the spent day.

 @param returnsInMs - milliseconds until the spent day starts again

 @param readAtMs - instant of the reading that named it, on `monotonicMs`

 @param dryness - why NeuralWatt reads dry, where the client reports it

 @param day - what the run knows of NeuralWatt's day

 @returns Fields, with the instant of the read, the time of day left out where neither the reading nor the run
 names one

 @example
 ```ts
 const carried = returnCarriedBy({
   returnsInMs: 301_000,
   readAtMs: 1_000,
   day: { kind: 'at', msOfDay: 69_308_000, },
 },);
 ```
 */
function returnCarriedBy(
  {
    returnsInMs,
    readAtMs,
    dryness,
    day,
  }: {
    readonly returnsInMs: number;
    readonly readAtMs: number;
    readonly dryness?: NeuralWattDryness;
    readonly day: NeuralWattReturn;
  },
): {
  readonly neuralwattReturnsInMs: number;
  readonly neuralwattReturnsAt?: string;
  readonly neuralwattDryness?: NeuralWattDryness;
  readonly neuralwattReadAtMs: number;
} {
  /**
   The time of day the return falls at.
   */
  const { text, } = returnTimeOf({
    day,
    ...((dryness === undefined) ? {} : { dryness, }),
  },);
  return {
    neuralwattReturnsInMs: returnsInMs,
    neuralwattReadAtMs: readAtMs,
    ...((text === undefined) ? {} : { neuralwattReturnsAt: text, }),
    ...((dryness === undefined) ? {} : { neuralwattDryness: dryness, }),
  };
}

/**
 Refusal for a writing bench that stops the entry, marked as already written,
 since the reading writes its own `JUDGE SEATS` line before throwing it.

 WHEN EVERY PROVIDER THAT KEEPS THE RUN GOING READS DRY THE CAUSE IS THAT, NOT THE BENCH (a wet provider that serves no
 seat that writes does not count, `run-writing-providers.ts`): the reading is the first to meet the dry state when
 an entry reaches its lanes with its preparation cached and so makes no chat call, and a bench error there ended the
 entry as a resumable failure that the pass visited and charged an attempt for, entry after entry, with no stop line.
 The owner ruled that the router's every-provider-dry refusal ends the run, so the reading raises that refusal, which
 the entry scheduler files as stopped work and hands to the pass (`endsTheRun`). A bench below its floor with a
 provider still wet stays the bench error, which carries NeuralWatt's return when that return lifts the bench to its
 floor, so the pass can park for it as it does on the refusal (`pass-dry-stop.ts`).

 MARKED SO NO OUTER LAYER WRITES IT AGAIN (ledger B413). The reading runs in
 the per-slice hook of every slice-by-slice phase, so two slices side by side
 can each stop the entry; the overlapped map throws the lower one and wrote
 the higher one a second line under the caller's stop, and would write one
 without a stop too.

 @param phase - phase the reading was taken for

 @param clauses - one clause per writing bench below its floor

 @param dry - the dryness view the reading stopped on

 @param client - run client, whose holds and NeuralWatt day the dry refusal states

 @returns The refusal to throw

 @example
 ```ts
 throw writtenRefusal({ phase: 'lanes', clauses: ['editors 0 of 3 reachable, floor 2',], dry, client, },);
 ```
 */
export function writtenRefusal(
  {
    phase,
    clauses,
    dry,
    client,
  }: {
    readonly phase: JudgeSeatPhase;
    readonly clauses: readonly string[];
    readonly dry: BudgetView;
    readonly client: RefusalClient;
  },
): EveryProviderDryError | WritingBenchUnreachableError {
  /**
   The providers whose wet reading keeps the run going.
   */
  const writing = writingProvidersOfClient({ client, },);
  /**
   Why NeuralWatt reads dry, as the client reports it.
   */
  const dryness = client.neuralwattDryness?.();
  /**
   The instant of that reading, which the pass counts a park's wait from.
   */
  const readAtMs = monotonicMs();
  /**
   Milliseconds until the spent day starts again while that return lies ahead, which the pass reads to park for it.
   */
  const returnsInMs = ((dryness?.kind === 'spent-day') && (dryness.returnsInMs !== undefined)
    && (dryness.returnsInMs > 0))
    ? dryness.returnsInMs
    : undefined;
  /**
   The refusal the entry stops with.
   */
  const refusal = everyWritingProviderDry({
    view: dry,
    writing,
  },)
    ? everyProviderDryRefusal({
      phase,
      // THE HOLDS THE READER WAITED FOR: a NeuralWatt return it declined to wait for is stated beside, not as a hold.
      holds: holdsWithinPass({ client, },),
      dry,
      writing,
      pastPass: (returnsInMs === undefined)
        ? []
        : pastPassWordsOf({
          returnsInMs,
          ...((client.passLeftMs === undefined) ? {} : { passLeftMs: client.passLeftMs(), }),
          ...((client.passBoundOwner === undefined) ? {} : { boundOwner: client.passBoundOwner(), }),
        },),
      ...((returnsInMs === undefined)
        ? {}
        : {
          neuralwattReturnsInMs: returnsInMs,
          neuralwattReadAtMs: readAtMs,
        }),
      ...((client.neuralwattReturn === undefined) ? {} : { neuralwatt: client.neuralwattReturn(), }),
      ...((client.neuralwattDryness === undefined) ? {} : { neuralwattDryness: client.neuralwattDryness(), }),
    },)
    : new WritingBenchUnreachableError({
      phase,
      clauses,
      // THE RETURN IS CARRIED ONLY WHEN IT REACHES THE FLOOR: a wet provider that cannot lift the bench would leave the
      // pass parked for a return that changes nothing. The roster answers it (`returnLiftsWritingBenches`).
      ...(((returnsInMs !== undefined) && dry.neuralwatt
        && returnLiftsWritingBenches({
          dry,
          back: ['neuralwatt',],
          names: phaseBenches({ phase, },),
        },))
        ? returnCarriedBy({
          returnsInMs,
          readAtMs,
          ...((dryness === undefined) ? {} : { dryness, }),
          day: neuralwattOf({ client, },),
        },)
        : {}),
    },);
  markFailureWritten({ error: refusal, },);
  return refusal;
}

//endregion Seat reading refusal
