import type {
  ChannelDelta,
  DegenerationDetector,
} from '../dist/final/node/index.mjs';

//region Delta channel routing
// SENDS ONE SCANNED DELTA TO THE DEGENERATION DETECTOR WATCHING ITS CHANNEL,
// the reasoning channel to one detector and every other channel to another,
// the way a drain wires a scanner's output to a pair of watchers.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The anthropic-delta-scan and
// stream-delta-scan tests kept their own copy of this routing callback; all
// now import it from here.

/**
 Routes one scanned delta to the detector watching its channel.

 @param delta - delta the scanner produced

 @param thinking - detector fed text arriving on the reasoning channel

 @param answering - detector fed text arriving on every other channel

 @example
 ```ts
 routeDeltaToDetector({ delta, thinking, answering, },);
 ```
 */
export function routeDeltaToDetector(
  {
    delta,
    thinking,
    answering,
  }: {
    readonly delta: ChannelDelta;
    readonly thinking: DegenerationDetector;
    readonly answering: DegenerationDetector;
  },
): void {
  if (delta.channel === 'reasoning')
    thinking.notifyText({ text: delta.text, },);
  else
    answering.notifyText({ text: delta.text, },);
}

//endregion Delta channel routing
