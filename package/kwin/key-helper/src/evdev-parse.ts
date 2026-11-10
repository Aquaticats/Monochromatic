/**
 Pure evdev helpers: sysfs capability-bitmap parsing and the double-shift
 state machine, kept free of I/O so both are unit-testable without a device.
 
 The double-shift detector is a reducer over shift/non-shift key events. A
 double-shift is two Shift taps within {@link DOUBLE_TAP_MS}, where a "tap" is
 one Shift key's press then release with no other key pressed and no second
 Shift key pressed in between. Any non-shift key pressed while Shift is held
 invalidates the current tap, and so does the other Shift key joining the hold,
 so Shift+letter and a two-key Shift hold never count.
 
 @module
 */

import { DOUBLE_TAP_MS } from './constants.ts';

/**
 Bits per space-separated word in a sysfs hex capability bitmap.
 */
const BITS_PER_WORD = 64;

/**
 evdev `value` for a key press.
 */
const VALUE_PRESS = 1;

/**
 evdev `value` for a key release.
 */
const VALUE_RELEASE = 0;

/**
 Parse a sysfs hex capability bitmap into the set of bit positions it sets.
 
 The sysfs format is space-separated 64-bit hex words, most-significant word
 first, so the words are reversed to make word index map to bit offset.
 
 @param hex - Contents of a `capabilities/*` sysfs node, e.g. `"1f 0 fffe"`
 
 @returns Bit positions (evdev codes) present in the bitmap
 
 @example
 ```ts
 parseBitmap('6'); // Set { 1, 2 }
 ```
 */
export function parseBitmap(hex: string): ReadonlySet<number> {
  /**
   Accumulated bit positions set anywhere in the bitmap.
   */
  const result = new Set<number>();
  /**
   Words reversed so `words[0]` holds bits 0 to 63.
   */
  const words = hex
    .trim()
    .split(' ')
    .filter(function nonEmpty(word: string): boolean {
      return word.length > 0;
    })
    .toReversed();
  for (const [wordIndex, word] of words.entries()) {
    /**
     Numeric value of one 64-bit hex word.
     */
    const value = BigInt(`0x${word}`);
    for (let bit = 0; bit < BITS_PER_WORD; bit += 1) {
      if (((value >> BigInt(bit)) & 1n) === 1n) {
        result.add((wordIndex * BITS_PER_WORD) + bit);
      }
    }
  }
  return result;
}

/**
 Immutable double-shift detector state.

 Both Shift keys are tracked apart: a tap is one Shift key's press and release,
 so a hold that a second Shift key joins is not a tap and never completes one.
 
 @example
 ```ts
 let state = INITIAL_SHIFT_STATE;
 ```
 */
export type ShiftState = {
  /**
   Whether the left Shift key is currently held.
   */
  readonly leftPressed: boolean;
  /**
   Whether the right Shift key is currently held.
   */
  readonly rightPressed: boolean;
  /**
   Whether something happened during the current hold that makes it no tap:
   a non-shift key was pressed, or the other Shift key joined.
   */
  readonly tapInvalidated: boolean;
  /**
   Timestamp (ms) of the previous clean Shift tap, or 0 when none pending.
   */
  readonly lastTapTime: number;
};

/**
 Starting state for the double-shift detector.
 
 @example
 ```ts
 const { state, doubleShift } = reduceShiftEvent({ state: INITIAL_SHIFT_STATE, event });
 ```
 */
export const INITIAL_SHIFT_STATE: ShiftState = {
  leftPressed: false,
  rightPressed: false,
  tapInvalidated: false,
  lastTapTime: 0,
};

/**
 One of the two physical Shift keys.
 
 @example
 ```ts
 const key: ShiftKey = 'left';
 ```
 */
export type ShiftKey = 'left' | 'right';

/**
 One key event fed to {@link reduceShiftEvent}.
 
 @example
 ```ts
 reduceShiftEvent({ state, event: { type: 'shift', key: 'left', value: 1, now: 10 } });
 ```
 */
export type ShiftEvent = {
  /**
   Discriminant: this event is a Left or Right Shift key.
   */
  readonly type: 'shift';
  /**
   Which Shift key changed.
   */
  readonly key: ShiftKey;
  /**
   evdev value: 1 press, 0 release, 2 autorepeat.
   */
  readonly value: number;
  /**
   Monotonic timestamp (ms) of the event.
   */
  readonly now: number;
} | {
  /**
   Discriminant: this event is any key that is not a Shift.
   */
  readonly type: 'other';
  /**
   evdev value: 1 press, 0 release, 2 autorepeat.
   */
  readonly value: number;
};

/**
 Result of reducing one event: the next state and whether a double-shift just
 completed on this event.
 
 @example
 ```ts
 const { state: next, doubleShift } = reduceShiftEvent({ state, event });
 ```
 */
export type ShiftResult = {
  /**
   State to carry into the next event.
   */
  readonly state: ShiftState;
  /**
   Whether this event completed a double-shift.
   */
  readonly doubleShift: boolean;
};

/**
 Advance the double-shift detector by one key event.

 A tap completes only when the last held Shift key is released with the hold
 undisturbed: no non-shift key pressed, and no second Shift key joined it. Two
 such taps inside {@link DOUBLE_TAP_MS} fire one double-shift.
 
 @param state - Current detector state
 
 @param event - Key event to fold in
 
 @returns Next state plus whether a double-shift fired on this event
 
 @example
 ```ts
 reduceShiftEvent({ state: INITIAL_SHIFT_STATE, event: { type: 'shift', key: 'left', value: 0, now: 5 } });
 ```
 */
export function reduceShiftEvent({
  state,
  event
}: {
  readonly state: ShiftState;
  readonly event: ShiftEvent;
}): ShiftResult {
  if (event.type === 'other') {
    // A non-shift press while Shift is held invalidates the pending tap.
    if ((event.value === VALUE_PRESS) && (state.leftPressed || state.rightPressed)) {
      return {
        state: {
          ...state,
          tapInvalidated: true
        },
        doubleShift: false
      };
    }
    return {
      state,
      doubleShift: false
    };
  }
  /**
   Whether the key this event names was held before it.
   */
  const wasPressed = event.key === 'left' ? state.leftPressed : state.rightPressed;
  /**
   Whether the other Shift key is held, which keeps the hold open.
   */
  const otherHeld = event.key === 'left' ? state.rightPressed : state.leftPressed;
  if (event.value === VALUE_PRESS) {
    /**
     State with this key recorded as held.
     */
    const held: ShiftState = event.key === 'left'
      ? {
        ...state,
        leftPressed: true
      }
      : {
        ...state,
        rightPressed: true
      };
    return {
      state: {
        ...held,
        // A second Shift key joining the hold makes it two keys' hold, which
        // is not the single-key tap a double-shift is built from.
        tapInvalidated: held.tapInvalidated || otherHeld
      },
      doubleShift: false
    };
  }
  if (event.value !== VALUE_RELEASE) {
    // Autorepeat or unknown value: no state change.
    return {
      state,
      doubleShift: false
    };
  }
  if (!wasPressed) {
    // A release for a key this detector never saw pressed completes no tap.
    return {
      state,
      doubleShift: false
    };
  }
  /**
   State with this key recorded as released.
   */
  const released: ShiftState = event.key === 'left'
    ? {
      ...state,
      leftPressed: false
    }
    : {
      ...state,
      rightPressed: false
    };
  if (otherHeld) {
    // The other Shift key is still held, so the hold has not ended and no tap
    // completes yet.
    return {
      state: released,
      doubleShift: false
    };
  }
  if (released.tapInvalidated) {
    // The hold was disturbed, so it ends without counting as a tap.
    return {
      state: {
        ...released,
        tapInvalidated: false
      },
      doubleShift: false
    };
  }
  /**
   Time since the previous clean tap, only meaningful when one is pending.
   */
  const delta = event.now - state.lastTapTime;
  if ((state.lastTapTime > 0) && (delta < DOUBLE_TAP_MS)) {
    // Second tap in time: fire, and clear lastTapTime so a third tap does not
    // immediately re-fire.
    return {
      state: {
        ...released,
        tapInvalidated: false,
        lastTapTime: 0
      },
      doubleShift: true,
    };
  }
  // First clean tap (or too slow): remember it as the start of a new window.
  return {
    state: {
      ...released,
      tapInvalidated: false,
      lastTapTime: event.now
    },
    doubleShift: false,
  };
}
