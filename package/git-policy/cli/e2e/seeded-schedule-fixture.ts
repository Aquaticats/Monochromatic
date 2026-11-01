/**
 Seeded scheduling decisions for concurrent workloads.

 The scheduler only decides:
 start offsets,
 operation interleavings,
 and fault timing.
 Barrier-ordered scenarios make those decisions observable as fixed interleavings;
 free-running scenarios still replay the same decisions under the same seed.

 @module
 */

import type { SeededRandom, } from './seeded-random-fixture.ts';

//region Types

/**
 One step of an interleaved schedule.
 */
export type InterleavedStep = Readonly<{
  /**
   Index of the actor whose operation runs.
   */
  actor: number;
  /**
   Position of the operation within that actor's sequence.
   */
  position: number;
}>;

//endregion Types

//region Planning

/**
 Plans worker start offsets in milliseconds.

 @param random - seeded source

 @param count - number of workers

 @param maxJitterMs - largest offset

 @returns offset per worker index

 @example
 ```ts
 planStartOffsets({ random, count: 4, maxJitterMs: 50 });
 ```
 */
export function planStartOffsets({
  random,
  count,
  maxJitterMs,
}: Readonly<{
  random: SeededRandom;
  count: number;
  maxJitterMs: number;
}>,): readonly number[] {
  return Array.from(
    { length: count, },
    function offset() {
      return random.integer({
        min: 0,
        max: maxJitterMs,
      },);
    },
  );
}

/**
 Merges several operation sequences into one order that keeps each actor's own order.
 Each step picks an actor with remaining work,
 weighted by how much work it has left,
 so every valid interleaving is reachable.

 @param random - seeded source

 @param lengths - operation count per actor

 @returns steps in execution order

 @example
 ```ts
 interleaveSequences({ random, lengths: [2, 1] });
 // => e.g. [{ actor: 0, position: 0 }, { actor: 1, position: 0 }, { actor: 0, position: 1 }]
 ```
 */
export function interleaveSequences({
  random,
  lengths,
}: Readonly<{
  random: SeededRandom;
  lengths: readonly number[];
}>,): readonly InterleavedStep[] {
  /**
   Total operation count.
   */
  const total = lengths.reduce(
    function sum(
      accumulated,
      length,
    ) {
      return accumulated + length;
    },
    0,
  );
  return Array.from({ length: total, },)
    .reduce<readonly InterleavedStep[]>(
    function nextStep(steps,) {
    /**
     Operations each actor has already scheduled.
     */
      const used = lengths.map(function usedCount(
        _length,
        actor,
      ) {
        return steps.filter(function byActor(step,) {
          return step.actor === actor;
        },)
          .length;
      },);
    /**
     Remaining operation count per actor.
     */
      const remaining = lengths.map(function remainingCount(
        length,
        actor,
      ) {
        return length - (used[actor] ?? 0);
      },);
    /**
     Weighted choice over remaining operations.
     */
      const ticket = random.integer({
        min: 0,
        max: (total - steps.length) - 1,
      },);
    /**
     Actor owning the chosen ticket.
     */
      const actor = remaining.findIndex(function owner(
        _count,
        index,
      ) {
      /**
       Tickets held by actors before and including this one.
       */
        const through = remaining.slice(
          0,
          index + 1,
        )
          .reduce(
          function sum(
            accumulated,
            count,
          ) {
            return accumulated + count;
          },
          0,
          );
        return ticket < through;
      },);
      return [
        ...steps,
        {
          actor,
          position: used[actor] ?? 0,
        },
      ];
    },
    [],
    );
}

/**
 Plans a fault offset inside a window.

 @param random - seeded source

 @param minMs - earliest offset

 @param maxMs - latest offset

 @returns offset in milliseconds

 @example
 ```ts
 planFaultOffset({ random, minMs: 50, maxMs: 400 });
 ```
 */
export function planFaultOffset({
  random,
  minMs,
  maxMs,
}: Readonly<{
  random: SeededRandom;
  minMs: number;
  maxMs: number;
}>,): number {
  return random.integer({
    min: minMs,
    max: maxMs,
  },);
}

//endregion Planning
