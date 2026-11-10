/**
 Tests for the pure evdev helpers: sysfs bitmap parsing and the double-shift
 detector reducer. The reducer cases capture the exact fire/no-fire rules so the
 detector cannot silently regress, one case per branch of the reducer: a clean
 single-key tap, a tap repeated inside and outside the window, a hold a
 non-shift key disturbs, and a hold the other Shift key joins.

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
} from '../dist/final/node/testing.mjs';

/**
 Detector state, named from the reducer's own parameter so the test follows the
 built declarations instead of importing source.
 */
type ShiftState = Parameters<typeof reduceShiftEvent>[0]['state'];

/**
 One key event, named from the reducer's own parameter.
 */
type ShiftEvent = Parameters<typeof reduceShiftEvent>[0]['event'];

/**
 One physical Shift key, named from the event the reducer accepts.
 */
type ShiftKey = Extract<ShiftEvent, { readonly type: 'shift'; }>['key'];

/**
 evdev `value` for a key press.
 */
const KEY_PRESS = 1;

/**
 evdev `value` for a key release.
 */
const KEY_RELEASE = 0;

/** Timestamp of the first tap in a double-shift sequence. */
const FIRST_TAP_MS = 1_000;

/** Second tap within the detection window (100ms gap, under 300ms). */
const SECOND_TAP_FAST_MS = 1_100;

/** Second tap outside the detection window (400ms gap, over 300ms). */
const SECOND_TAP_SLOW_MS = 1_400;

/**
 Gap between the events of one hold, small enough to stay inside the window.
 */
const HOLD_STEP_MS = 10;

/**
 Build one Shift event.
 
 @param key - Which Shift key changed
 @param value - evdev value of the change
 @param now - Timestamp of the event
 @returns Event the reducer accepts
 @example
 ```ts
 shift({ key: 'left', value: KEY_PRESS, now: FIRST_TAP_MS });
 ```
 */
function shift({ key, value, now }: {
  readonly key: ShiftKey;
  readonly value: number;
  readonly now: number;
}): ShiftEvent {
  return {
    type: 'shift',
    key,
    value,
    now,
  };
}

/**
 Build one non-Shift key event.
 
 @param value - evdev value of the change
 @returns Event the reducer accepts
 @example
 ```ts
 otherKey({ value: KEY_PRESS });
 ```
 */
function otherKey({ value }: { readonly value: number; }): ShiftEvent {
  return {
    type: 'other',
    value,
  };
}

/**
 Fold a press then release of one Shift key into the detector, returning the
 result of the release (where a double-shift, if any, fires).
 
 @param state - State to start from
 @param releaseNow - Timestamp of the press and release events
 @param key - Which Shift key is tapped
 @returns Reducer result for the release event
 @example
 ```ts
 tapShift({ state: INITIAL_SHIFT_STATE, releaseNow: FIRST_TAP_MS });
 ```
 */
function tapShift({ state, releaseNow, key = 'left' }: {
  readonly state: ShiftState;
  readonly releaseNow: number;
  readonly key?: ShiftKey;
}): {
  state: ShiftState;
  doubleShift: boolean;
} {
  /** State after the Shift press. */
  const pressed = reduceShiftEvent({
    state,
    event: shift({ key, value: KEY_PRESS, now: releaseNow, }),
  }).state;
  return reduceShiftEvent({
    state: pressed,
    event: shift({ key, value: KEY_RELEASE, now: releaseNow, }),
  });
}

/**
 Fold an event sequence into the detector.
 
 @param events - Events in arrival order
 @returns Final state and whether any event fired a double-shift
 @example
 ```ts
 foldShiftEvents([shift({ key: 'left', value: KEY_PRESS, now: FIRST_TAP_MS })]);
 ```
 */
function foldShiftEvents(events: readonly ShiftEvent[]): {
  readonly state: ShiftState;
  readonly fired: boolean;
} {
  /** Running fold result. */
  const folded = events.reduce(function step(
    carried: { readonly state: ShiftState; readonly fired: boolean; },
    event: ShiftEvent,
  ) {
    /** Result of folding this event in. */
    const result = reduceShiftEvent({ state: carried.state, event, });
    return { state: result.state, fired: carried.fired || result.doubleShift, };
  }, { state: INITIAL_SHIFT_STATE, fired: false, });
  return folded;
}

/**
 Every one-key Shift tap pair the detector must complete, so no case depends on
 a single key being the one the detector happens to favour.
 */
const TAP_KEY_PAIRS: readonly (readonly [ShiftKey, ShiftKey])[] = [
  ['left', 'left'],
  ['right', 'right'],
  ['left', 'right'],
  ['right', 'left'],
];

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
        //region Clean taps

        ...TAP_KEY_PAIRS.map(function mapKeyPair([firstKey, secondKey]: readonly [
          ShiftKey,
          ShiftKey,
        ]) {
          return it({
            name: `fires on a second ${secondKey} Shift tap after a ${firstKey} one inside the window`,
            fn: async () => {
              expect(tapShift({
                state: tapShift({
                  state: INITIAL_SHIFT_STATE,
                  releaseNow: FIRST_TAP_MS,
                  key: firstKey,
                }).state,
                releaseNow: SECOND_TAP_FAST_MS,
                key: secondKey,
              }).doubleShift).toBe(true);
            },
          });
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
          name: 'does not re-fire on an immediate third tap',
          fn: async () => {
            expect(tapShift({
              state: tapShift({
                state: tapShift({ state: INITIAL_SHIFT_STATE, releaseNow: FIRST_TAP_MS }).state,
                releaseNow: SECOND_TAP_FAST_MS,
              }).state,
              releaseNow: SECOND_TAP_FAST_MS + HOLD_STEP_MS,
            }).doubleShift).toBe(false);
          },
        }),

        //endregion Clean taps

        //region Holds that are not a tap

        it({
          name: 'invalidates the tap when another key is pressed while Shift is held',
          fn: async () => {
            expect(foldShiftEvents([
              shift({ key: 'left', value: KEY_PRESS, now: FIRST_TAP_MS, }),
              otherKey({ value: KEY_PRESS, }),
              shift({ key: 'left', value: KEY_RELEASE, now: FIRST_TAP_MS + HOLD_STEP_MS, }),
            ]).fired).toBe(false);
          },
        }),
        it({
          name: 'fires nothing when a second Shift key joins a hold a letter disturbed',
          fn: async () => {
            expect(foldShiftEvents([
              shift({ key: 'left', value: KEY_PRESS, now: FIRST_TAP_MS, }),
              otherKey({ value: KEY_PRESS, }),
              shift({ key: 'right', value: KEY_PRESS, now: FIRST_TAP_MS + HOLD_STEP_MS, }),
              shift({ key: 'left', value: KEY_RELEASE, now: FIRST_TAP_MS + (2 * HOLD_STEP_MS), }),
              shift({ key: 'right', value: KEY_RELEASE, now: FIRST_TAP_MS + (3 * HOLD_STEP_MS), }),
            ]).fired).toBe(false);
          },
        }),
        it({
          name: 'completes no tap when the other Shift key is still held',
          fn: async () => {
            expect(foldShiftEvents([
              shift({ key: 'left', value: KEY_PRESS, now: FIRST_TAP_MS, }),
              shift({ key: 'right', value: KEY_PRESS, now: FIRST_TAP_MS + HOLD_STEP_MS, }),
              shift({ key: 'left', value: KEY_RELEASE, now: FIRST_TAP_MS + (2 * HOLD_STEP_MS), }),
            ]).fired).toBe(false);
          },
        }),
        ...([['left', 'right'], ['right', 'left']] as const).map(function mapHoldOrder([
          firstKey,
          secondKey,
        ]: readonly [ShiftKey, ShiftKey]) {
          return it({
            name: `fires nothing when both Shift keys are held and released together twice, ${firstKey} first`,
            fn: async () => {
              expect(foldShiftEvents([
                shift({ key: firstKey, value: KEY_PRESS, now: FIRST_TAP_MS, }),
                shift({ key: secondKey, value: KEY_PRESS, now: FIRST_TAP_MS + HOLD_STEP_MS, }),
                shift({ key: firstKey, value: KEY_RELEASE, now: FIRST_TAP_MS + (2 * HOLD_STEP_MS), }),
                shift({ key: secondKey, value: KEY_RELEASE, now: FIRST_TAP_MS + (3 * HOLD_STEP_MS), }),
                shift({ key: firstKey, value: KEY_PRESS, now: FIRST_TAP_MS + (4 * HOLD_STEP_MS), }),
                shift({ key: secondKey, value: KEY_PRESS, now: FIRST_TAP_MS + (5 * HOLD_STEP_MS), }),
                shift({ key: firstKey, value: KEY_RELEASE, now: FIRST_TAP_MS + (6 * HOLD_STEP_MS), }),
                shift({ key: secondKey, value: KEY_RELEASE, now: FIRST_TAP_MS + (7 * HOLD_STEP_MS), }),
              ]).fired).toBe(false);
            },
          });
        }),
        it({
          name: 'fires nothing when a second Shift key joins inside a pending window',
          fn: async () => {
            // A clean left tap starts the window; the next hold is disturbed by
            // the right key and released left-first, so nothing may fire even
            // though the left release lands inside the window.
            expect(foldShiftEvents([
              shift({ key: 'left', value: KEY_PRESS, now: FIRST_TAP_MS, }),
              shift({ key: 'left', value: KEY_RELEASE, now: FIRST_TAP_MS, }),
              shift({ key: 'left', value: KEY_PRESS, now: FIRST_TAP_MS + HOLD_STEP_MS, }),
              shift({ key: 'right', value: KEY_PRESS, now: FIRST_TAP_MS + (2 * HOLD_STEP_MS), }),
              shift({ key: 'left', value: KEY_RELEASE, now: FIRST_TAP_MS + (3 * HOLD_STEP_MS), }),
              shift({ key: 'right', value: KEY_RELEASE, now: FIRST_TAP_MS + (4 * HOLD_STEP_MS), }),
            ]).fired).toBe(false);
          },
        }),
        it({
          name: 'still fires a genuine double-shift after a two-key hold',
          fn: async () => {
            expect(foldShiftEvents([
              shift({ key: 'left', value: KEY_PRESS, now: FIRST_TAP_MS, }),
              shift({ key: 'right', value: KEY_PRESS, now: FIRST_TAP_MS + HOLD_STEP_MS, }),
              shift({ key: 'left', value: KEY_RELEASE, now: FIRST_TAP_MS + (2 * HOLD_STEP_MS), }),
              shift({ key: 'right', value: KEY_RELEASE, now: FIRST_TAP_MS + (3 * HOLD_STEP_MS), }),
              shift({ key: 'left', value: KEY_PRESS, now: SECOND_TAP_FAST_MS, }),
              shift({ key: 'left', value: KEY_RELEASE, now: SECOND_TAP_FAST_MS + HOLD_STEP_MS, }),
              shift({ key: 'left', value: KEY_PRESS, now: SECOND_TAP_FAST_MS + (2 * HOLD_STEP_MS), }),
              shift({ key: 'left', value: KEY_RELEASE, now: SECOND_TAP_FAST_MS + (3 * HOLD_STEP_MS), }),
            ]).fired).toBe(true);
          },
        }),

        //endregion Holds that are not a tap
      ],
    }),
  ],
});
