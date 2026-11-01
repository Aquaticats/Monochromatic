import { RUN_SEATS, } from '../../dist/final/node/index.mjs';

//region Fresh run seats
// EMPTIES THE RUN-WIDE SEAT TALLY FOR ONE CASE, which the run client counts
// into and every command prints when it ends.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. `cli-refusal.unit.test.ts` and
// `run-config.unit.test.ts` kept a copy each until 2026-10-10.

/**
 Empties the run-wide seat tally for the life of a scope and again on exit,
 so a case reads only what it caused and leaves nothing for the next one.

 @returns Disposable emptying the tally again

 @example
 ```ts
 using _fresh = withFreshRunSeats();
 ```
 */
export function withFreshRunSeats(): Disposable {
  RUN_SEATS.reset();
  return {
    [Symbol.dispose](): void {
      RUN_SEATS.reset();
    },
  };
}

//endregion Fresh run seats
