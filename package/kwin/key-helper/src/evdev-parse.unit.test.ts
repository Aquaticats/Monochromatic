/**
 Tests for the pure evdev helpers: sysfs bitmap parsing and the double-shift
 detector reducer. The reducer cases capture the exact fire/no-fire rules so the
 detector cannot silently regress.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  INITIAL_SHIFT_STATE,
  parseBitmap,
  reduceShiftEvent,
  type ShiftState,
} from './evdev-parse.ts';

/** Timestamp of the first tap in a double-shift sequence. */
const FIRST_TAP_MS = 1_000;

/** Second tap within the detection window (100ms gap, under 300ms). */
const SECOND_TAP_FAST_MS = 1_100;

/** Second tap outside the detection window (400ms gap, over 300ms). */
const SECOND_TAP_SLOW_MS = 1_400;

/**
 Fold a press then release of Shift into the detector, returning the result of
 the release (where a double-shift, if any, fires).
 
 @param state - State to start from
 @param releaseNow - Timestamp of the release event
 @returns Reducer result for the release event
 @example
 ```ts
 tapShift({ state: INITIAL_SHIFT_STATE, releaseNow: 1000 });
 ```
 */
function tapShift({ state, releaseNow }: { state: ShiftState; releaseNow: number }): {
  state: ShiftState;
  doubleShift: boolean;
} {
  /** State after the Shift press. */
  const pressed = reduceShiftEvent({
    state,
    event: { isShift: true, value: 1, now: releaseNow },
  }).state;
  return reduceShiftEvent({
    state: pressed,
    event: { isShift: true, value: 0, now: releaseNow },
  });
}

await describe({
  name: '',
  children: [
    describe({
      name: parseBitmap.name,
      children: [
        it({
          name: 'sets bit positions within one word',
          fn: async () => {
            expect([...parseBitmap('6')]).toEqual([1, 2]);
          },
        }),
        it({
          name: 'offsets later words by 64 bits (words are MSB-first)',
          fn: async () => {
            expect([...parseBitmap('1 0')]).toEqual([64]);
          },
        }),
        it({
          name: 'returns an empty set for an all-zero bitmap',
          fn: async () => {
            expect([...parseBitmap('0')]).toEqual([]);
          },
        }),
        it({
          name: 'tolerates surrounding whitespace and blank words',
          fn: async () => {
            expect([...parseBitmap('  0   6 ')]).toEqual([1, 2]);
          },
        }),
      ],
    }),
    describe({
      name: reduceShiftEvent.name,
      children: [
        it({
          name: 'fires on a second tap within the window',
          fn: async () => {
            expect(tapShift({
              state: tapShift({ state: INITIAL_SHIFT_STATE, releaseNow: FIRST_TAP_MS }).state,
              releaseNow: SECOND_TAP_FAST_MS,
            }).doubleShift).toBe(true);
          },
        }),
        it({
          name: 'does not fire on a second tap outside the window',
          fn: async () => {
            expect(tapShift({
              state: tapShift({ state: INITIAL_SHIFT_STATE, releaseNow: FIRST_TAP_MS }).state,
              releaseNow: SECOND_TAP_SLOW_MS,
            }).doubleShift).toBe(false);
          },
        }),
        it({
          name: 'does not fire on the first tap',
          fn: async () => {
            expect(tapShift({ state: INITIAL_SHIFT_STATE, releaseNow: FIRST_TAP_MS }).doubleShift)
              .toBe(false);
          },
        }),
        it({
          name: 'invalidates the tap when another key is pressed while Shift is held',
          fn: async () => {
            expect(reduceShiftEvent({
              state: reduceShiftEvent({
                state: reduceShiftEvent({
                  state: INITIAL_SHIFT_STATE,
                  event: { isShift: true, value: 1, now: FIRST_TAP_MS },
                }).state,
                event: { isShift: false, value: 1, now: FIRST_TAP_MS },
              }).state,
              event: { isShift: true, value: 0, now: FIRST_TAP_MS },
            }).doubleShift).toBe(false);
          },
        }),
        it({
          name: 'does not re-fire on an immediate third tap',
          fn: async () => {
            expect(tapShift({
              state: tapShift({
                state: tapShift({ state: INITIAL_SHIFT_STATE, releaseNow: FIRST_TAP_MS }).state,
                releaseNow: SECOND_TAP_FAST_MS,
              }).state,
              releaseNow: SECOND_TAP_FAST_MS + 1,
            }).doubleShift).toBe(false);
          },
        }),
      ],
    }),
  ],
});
