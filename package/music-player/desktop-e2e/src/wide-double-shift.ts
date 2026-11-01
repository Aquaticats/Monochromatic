/**
 Checks of the wide session about double Shift: two quick taps open Search on the player page, and three near misses
 do nothing.
 
 @example
 ```ts
 import { checkDoubleShift, } from './wide-double-shift.ts';
 
 await checkDoubleShift({ context, session, },);
 ```
 
 @packageDocumentation
 */

import {
  chordKeys,
  focusIs,
  holdKey,
  pauseFor,
  QUICK_DOUBLE_SHIFT,
  releaseKey,
  routeIs,
  sequenceOf,
  tapKey,
} from './actions.ts';
import { isPlayerPage, } from './conditions.ts';
import type { RunContext, } from './context.ts';
import {
  noEffect,
  nothingMoved,
  press,
} from './press.ts';
import type { Session, } from './session.ts';

/**
 How long Search gets to open after double Shift.
 */
const DOUBLE_SHIFT_TIMEOUT_MS = 3_000;

/**
 Gap between the two taps of a slow double Shift, longer than the detector window.
 */
const SLOW_GAP_MS = 600;

/**
 Gap used around the key that interrupts a double Shift.
 */
const INTERRUPT_GAP_MS = 60;

/**
 Settle time after a double Shift that should do nothing.
 */
const DOUBLE_SHIFT_SETTLE_MS = 700;

/**
 Records the double Shift checks of the wide session.
 
 @param context - the run's context
 
 @param session - the wide session, on the player page
 
 @example
 ```ts
 await checkDoubleShift({ context, session, },);
 ```
 */
export async function checkDoubleShift(
  {
    context,
    session,
  }: {
    readonly context: RunContext;
    readonly session: Session;
  },
): Promise<void> {
  /**
   The recorder of the run.
   */
  const { recorder, } = context;
  recorder.setGroup('double-shift',);
  await press({
    session,
    act: chordKeys({
      mods: ['alt',],
      key: '2',
    },),
    until: focusIs('track-list',),
    what: 'the track list',
  },);
  await recorder.verify({
    id: 'double-shift/opens-search',
    title: 'two quick Shift taps open Search on the player page',
    body: async function quickDoubleShiftOpensSearch(): Promise<void> {
      await QUICK_DOUBLE_SHIFT(session,);
      await session.waitState({
        predicate: routeIs('Search',),
        what: 'Search opening',
        timeoutMs: DOUBLE_SHIFT_TIMEOUT_MS,
      },);
      await press({
        session,
        act: tapKey('escape',),
        until: isPlayerPage,
        what: 'the player page',
      },);
    },
  },);
  await recorder.verify({
    id: 'double-shift/slow',
    title: 'two slow Shift taps (600 ms apart) do nothing',
    body: async function slowDoubleShiftDoesNothing(): Promise<void> {
      await noEffect({
        session,
        act: sequenceOf([
          tapKey('shift',),
          pauseFor(SLOW_GAP_MS,),
          tapKey('shift',),
        ],),
        watch: nothingMoved,
        what: 'slow double Shift',
        settleMs: DOUBLE_SHIFT_SETTLE_MS,
      },);
    },
  },);
  await recorder.verify({
    id: 'double-shift/interrupted',
    title: 'a key between the Shift taps cancels the double tap',
    body: async function interruptedDoubleShiftDoesNothing(): Promise<void> {
      await noEffect({
        session,
        act: sequenceOf([
          tapKey('shift',),
          pauseFor(INTERRUPT_GAP_MS,),
          tapKey('x',),
          pauseFor(INTERRUPT_GAP_MS,),
          tapKey('shift',),
        ],),
        watch: nothingMoved,
        what: 'interrupted double Shift',
        settleMs: DOUBLE_SHIFT_SETTLE_MS,
      },);
    },
  },);
  await recorder.verify({
    id: 'double-shift/with-key',
    title: 'Shift held with another key is not a tap',
    body: async function shiftWithKeyIsNotATap(): Promise<void> {
      await noEffect({
        session,
        act: sequenceOf([
          holdKey('shift',),
          tapKey('x',),
          releaseKey('shift',),
          pauseFor(INTERRUPT_GAP_MS,),
          tapKey('shift',),
        ],),
        watch: nothingMoved,
        what: 'Shift with another key',
        settleMs: DOUBLE_SHIFT_SETTLE_MS,
      },);
    },
  },);
  recorder.skip({
    id: 'double-shift/repeat',
    title: 'a held Shift with repeats is not a double tap',
    reason:
      'a modifier key never repeats on Wayland (the client skips modifiers when it repeats a held key, so holding Shift gives one press and one release, as on a real desktop); the case cannot occur, and the pure detector test covers the sequence',
  },);
}
