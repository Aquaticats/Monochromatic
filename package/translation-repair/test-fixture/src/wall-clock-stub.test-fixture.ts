import type { DisposableSandbox, } from '@monochromatic-dev/test-harness-core/ts';

//region Wall clock stub
// Stands in for the system clock in the cases that set it, the way a person
// or a time daemon does (ledger B78): `Date.now` answers a reading the case
// moves. The stub goes through the case's own sandbox, `ctx.sinon`, which
// module-test answers only to code running in that case's async context
// (`sandbox-slot.ts`), so the cases a suite runs beside it, sixteen at a time
// by default, and the logger's stamps read the real clock; the runner puts
// the clock back when the case ends. The first form of this fixture replaced
// `Date.now` for the whole process, and a pacer in a sibling case read the
// stepped clock and slept on it (ledger M100). This file and its own test are
// the two that `wall-clock-reads.unit.test.ts` lets name the wall clock, since
// replacing it is the point.

/**
 Milliseconds in an hour, the step these cases set the clock by.
 */
export const HOUR_MS = 3_600_000;

/**
 Where a stubbed wall clock starts: 2026-10-01T00:00:00.000Z, a reading no
 process-relative clock comes near.
 */
export const WALL_START_MS = 1_790_812_800_000;

/**
 A wall clock a case sets, which the runner restores when the case ends.
 */
type WallClockStub = {
  /**
   Sets the clock by a signed span, as setting the system time does.

   @param byMs - milliseconds to move it, negative to set it back
   */
  readonly step: ({ byMs, }: { readonly byMs: number; },) => void;
};

/**
 Answers `Date.now` with a reading the case sets, inside that case only.

 Code the case runs reads the stub, through its awaits and the timers it
 sets; the cases running beside it read the real clock.

 @param sinon - calling case's own sandbox (`ctx.sinon`), whose stub module-test
 scopes to the case and restores when it ends; a sandbox made any other way
 would set the clock for every case in the process

 @param atMs - reading the clock answers first

 @returns Stub, to step

 @example
 ```ts
 const wall = stubWallClock({ sinon: ctx.sinon, atMs: WALL_START_MS, },);
 wall.step({ byMs: -HOUR_MS, },);
 ```
 */
export function stubWallClock(
  {
    sinon,
    atMs,
  }: {
    readonly sinon: DisposableSandbox;
    readonly atMs: number;
  },
): WallClockStub {
  /**
   The reading the stub answers.
   */
  const reading = { ms: atMs, };
  sinon
    .stub(
    Date,
    'now',
    )
    .callsFake(function steppedNow(): number {
      return reading.ms;
    },);
  return {
    step({ byMs, },): void {
      reading.ms += byMs;
    },
  };
}

//endregion Wall clock stub
