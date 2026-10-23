import {
  neuralWattDayStartOfInstant,
  neuralWattTimeOfDayShort,
} from './neuralwatt-day.ts';
import type { NeuralWattReturn, } from './budget-routing.ts';
import { durationWordsOf, } from './duration-words.ts';
import type { NeuralWattDryness, } from './neuralwatt-return-hold.ts';
import {
  type BoundOwner,
  pastPassWordsOf,
} from './neuralwatt-return-fit.ts';
import { keyUnblockTextOf, } from './neuralwatt-return-words.ts';
import {
  PROVIDER_ORDER,
  type ProviderRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Hold reason
// THE WORDS A WAIT AND A REFUSAL GIVE FOR A HOLD, so a read of the run can tell a refusal's hold from NeuralWatt's day
// starting again, and a refusal states why NeuralWatt reads dry and nothing it does not know.

/**
 Names the providers whose hold is the shortest, with NeuralWatt's day start where NeuralWatt is one of them.

 @param holds - milliseconds of hold left per provider

 @param shortest - shortest running hold, as `shortestHold` read it

 @param neuralwatt - what the run knows of NeuralWatt's day

 @param neuralwattHeldByReturn - whether NeuralWatt's hold is its day start and not a refusal's, since a refusal that
 holds it longer is the hold that won and the day start is not its reason (default: it is)

 @returns Providers in spending order, or `no provider` where nothing is held

 @example
 ```ts
 holdReasonOf({ holds, shortest: 48_000, neuralwatt: { kind: 'at', msOfDay: 69_308_000, }, },);
 // => 'neuralwatt, whose day starts again at 19:15:08 UTC'
 ```
 */
export function holdReasonOf(
  {
    holds,
    shortest,
    neuralwatt,
    neuralwattHeldByReturn = true,
  }: {
    readonly holds: ProviderRecord<number>;
    readonly shortest: number;
    readonly neuralwatt: NeuralWattReturn;
    readonly neuralwattHeldByReturn?: boolean;
  },
): string {
  /**
   Providers whose hold is the shortest running one.
   */
  const holders = PROVIDER_ORDER.filter(function isShortest(provider,): boolean {
    return (shortest > 0) && (holds[provider] === shortest);
  },);
  if (holders.length === 0)
    return 'no provider';
  return holders.map(function nameOf(provider,): string {
    return ((provider === 'neuralwatt') && neuralwattHeldByReturn
      && (neuralwatt.kind === 'at'))
      ? `neuralwatt, whose day starts again at ${neuralWattTimeOfDayShort({ msOfDay: neuralwatt.msOfDay, },)} UTC`
      : provider;
  },)
    .join(', ',);
}

/**
 The clause a refusal adds for why NeuralWatt reads dry, only the true cause: a blocked key states its own time, energy
 held by calls in flight says so, a day the account still shows spent past its start says that, and a run that learned
 no day start says that. A spent day whose return is named states the time and the distance.

 @param dryness - why NeuralWatt reads dry as the last reading found it

 @param passLeftMs - milliseconds the pass has left, where the reader's wait is bounded by it: a spent day's return past
 it is stated as past what the pass has left

 @param boundOwner - whose bound passLeftMs is, so the sentence names the entry when its own cap is the smaller bound
 (default: the pass)

 @returns One sentence, or none where NeuralWatt is not dry

 @example
 ```ts
 dryCauseClausesOf({ dryness: { kind: 'key-blocked', until: '2026-10-23T05:00:00.000Z', }, },);
 ```
 */
export function dryCauseClausesOf(
  {
    dryness,
    passLeftMs,
    boundOwner,
  }: {
    readonly dryness: NeuralWattDryness;
    readonly passLeftMs?: number;
    readonly boundOwner?: BoundOwner;
  },
): readonly string[] {
  if (dryness.kind === 'wet')
    return [];
  if (dryness.kind === 'key-blocked')
    return [
      `NeuralWatt reads dry because its key allowance is blocked until ${keyUnblockTextOf({ until: dryness.until, },)}`,
    ];
  if (dryness.kind === 'reserved')
    return ['NeuralWatt reads dry because calls in flight hold the energy the settled calls leave',];
  if (dryness.kind === 'no-day-start')
    return ['NeuralWatt reads dry because this run learned no day start for it',];

  /**
   The time of day the day starts again, when the reading's day start parses.
   */
  const { msOfDay, } = neuralWattDayStartOfInstant({ instant: dryness.dayStart, },);
  /**
   The day start as the sentence states it.
   */
  const at = (msOfDay === undefined) ? 'its day start' : `${neuralWattTimeOfDayShort({ msOfDay, },)} UTC`;
  if (dryness.returnsInMs === undefined)
    return [`NeuralWatt reads dry because its account still shows the day spent past the day start at ${at}`,];
  /**
   The clause saying the pass would end first, empty where the return fits what the pass has left.
   */
  const pastPass = pastPassWordsOf({
    returnsInMs: dryness.returnsInMs,
    ...((passLeftMs === undefined) ? {} : { passLeftMs, }),
    ...((boundOwner === undefined) ? {} : { boundOwner, }),
  },)
    .map(function clauseOf(words,): string {
      return `, which is ${words}`;
    },)
    .join('',);
  return [
    `NeuralWatt reads dry and its day starts again at ${at} `
      + `(in ${durationWordsOf({ ms: dryness.returnsInMs, },)})${pastPass}`,
  ];
}

//endregion Hold reason
