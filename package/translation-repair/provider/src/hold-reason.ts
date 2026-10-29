import {
  neuralWattDayStartOfInstant,
  neuralWattTimeOfDayShort,
} from './neuralwatt-day.ts';
import { durationWordsOf, } from './duration-words.ts';
import type { NeuralWattDryness, } from './neuralwatt-return-hold.ts';
import { keyUnblockTextOf, } from './neuralwatt-return-words.ts';
import {
  PROVIDER_ORDER,
  type ProviderRecord,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Hold reason
// THE WORDS A WAIT AND A REFUSAL GIVE FOR A HOLD, so a read of the run can tell which provider's hold a wait is for,
// and a refusal states why NeuralWatt reads dry and nothing it does not know. NeuralWatt's day starting again is no hold
// and is never waited for (the owner's answer of 2026-10-29, "Never wait, relaunch"), so a hold's reason names a
// provider and nothing about NeuralWatt's day.

/**
 Names the providers whose hold is the shortest.

 @param holds - milliseconds of hold left per provider

 @param shortest - shortest running hold, as `shortestHold` read it

 @returns Providers in spending order, or `no provider` where nothing is held

 @example
 ```ts
 holdReasonOf({ holds, shortest: 48_000, },); // => 'hyper'
 ```
 */
export function holdReasonOf(
  {
    holds,
    shortest,
  }: {
    readonly holds: ProviderRecord<number>;
    readonly shortest: number;
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
  return holders.join(', ',);
}

/**
 The clause a refusal adds for why NeuralWatt reads dry, only the true cause: a blocked key states its own time, energy
 held by calls in flight says so, a day the account still shows spent past its start says that, and a run that learned
 no day start says that. A spent day whose return is named states the time of day it starts again and the distance, so
 the operator knows when to relaunch.

 @param dryness - why NeuralWatt reads dry as the last reading found it

 @returns One sentence, or none where NeuralWatt is not dry

 @example
 ```ts
 dryCauseClausesOf({ dryness: { kind: 'key-blocked', until: '2026-10-23T05:00:00.000Z', }, },);
 ```
 */
export function dryCauseClausesOf({ dryness, }: { readonly dryness: NeuralWattDryness; },): readonly string[] {
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
  return [
    `NeuralWatt reads dry and its day starts again at ${at} (in ${durationWordsOf({ ms: dryness.returnsInMs, },)})`,
  ];
}

//endregion Hold reason
