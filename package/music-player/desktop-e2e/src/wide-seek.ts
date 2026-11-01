/**
 Checks of the wide session about seeking, run while paused so the position holds still: Ctrl+Home, the four Alt
 seek chords, a held chord, and the arrow keys on the seek bar and the mode group.
 
 @example
 ```ts
 import { checkSeeking, } from './wide-seek.ts';
 
 await checkSeeking({ context, session, },);
 ```
 
 @packageDocumentation
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  chordKeys,
  holdKey,
  isPaused,
  positionNear,
  releaseKey,
  tapKey,
} from './actions.ts';
import type { RunContext, } from './context.ts';
import { expect, } from './errors.ts';
import {
  press,
  tabTo,
} from './press.ts';
import type { Session, } from './session.ts';
import type { UiState, } from './ui-state.ts';

/**
 Position below which a track counts as restarted, in seconds.
 */
const RESTART_BELOW_S = 1.5;

/**
 Position above which a long seek counts as having moved forward, in seconds.
 */
const FAR_FORWARD_S = 50;

/**
 Distance from the expected position that still counts as the seek landing, in seconds.
 */
const SEEK_TOLERANCE_S = 1.5;

/**
 Least gain that proves a held chord kept seeking, in seconds.
 */
const HELD_GAIN_S = 15;

/**
 How long the chord is held, in milliseconds.
 */
const HOLD_MS = 900;

/**
 Time given to the last repeat to land after the release, in milliseconds.
 */
const AFTER_HOLD_MS = 300;

/**
 Repeats a second announced for the counted hold.
 */
const SLOW_REPEAT_RATE = 5;

/**
 Delay before the first repeat announced for the counted hold, in milliseconds.
 */
const SLOW_REPEAT_DELAY_MS = 400;

/**
 How long the counted hold lasts, in milliseconds: the press, then repeats at 400, 600, 800, 1000 and 1200 ms.
 */
const COUNTED_HOLD_MS = 1_300;

/**
 Least and most seconds the counted hold may gain, around the six presses of 10 s each (the press and five repeats).
 */
const COUNTED_MIN_GAIN_S = 45;

/**
 See `COUNTED_MIN_GAIN_S`.
 */
const COUNTED_MAX_GAIN_S = 75;

/**
 Seconds one Alt+Right press seeks.
 */
const SEEK_PRESS_S = 10;

/**
 The repeat rate the compositor announces unless a check changes it, in repeats a second.
 */
const DEFAULT_REPEAT_RATE = 25;

/**
 The repeat delay the compositor announces unless a check changes it, in milliseconds.
 */
const DEFAULT_REPEAT_DELAY_MS = 200;

/**
 Smallest step of the seek bar's arrow keys, in seconds.
 */
const BAR_STEP_S = 1;

/**
 The four seek chords, with the distance each moves, in seconds.
 */
export const SEEK_STEPS = [
  {
    id: 'seek/alt-right',
    title: 'Alt+Right seeks +10 s',
    mods: ['alt',],
    key: 'right',
    delta: 10,
  },
  {
    id: 'seek/alt-shift-right',
    title: 'Alt+Shift+Right seeks +60 s',
    mods: [
      'alt',
      'shift',
    ],
    key: 'right',
    delta: 60,
  },
  {
    id: 'seek/alt-left',
    title: 'Alt+Left seeks -10 s',
    mods: ['alt',],
    key: 'left',
    delta: -10,
  },
  {
    id: 'seek/alt-shift-left',
    title: 'Alt+Shift+Left seeks -60 s',
    mods: [
      'alt',
      'shift',
    ],
    key: 'left',
    delta: -60,
  },
] as const;

/**
 Condition: the position is back at the start of the track.
 
 @param state - a state of the app
 
 @returns whether the position is below the restart limit
 
 @example
 ```ts
 isAtStart(state,);
 ```
 */
function isAtStart(state: UiState,): boolean {
  return state.position < RESTART_BELOW_S;
}

/**
 Condition: the position is far into the track.
 
 @param state - a state of the app
 
 @returns whether the position is above the far-forward limit
 
 @example
 ```ts
 isFarForward(state,);
 ```
 */
function isFarForward(state: UiState,): boolean {
  return state.position > FAR_FORWARD_S;
}

/**
 Records the seek checks of the wide session. It pauses first, so the position holds still between key presses.
 
 @param context - the run's context
 
 @param session - the wide session, with a track playing
 
 @example
 ```ts
 await checkSeeking({ context, session, },);
 ```
 */
export async function checkSeeking(
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
  await press({
    session,
    act: tapKey('space',),
    until: isPaused,
    what: 'pause for the seek checks',
  },);
  await press({
    session,
    act: chordKeys({
      mods: ['ctrl',],
      key: 'home',
    },),
    until: isAtStart,
    what: 'start of the track',
  },);
  await recorder.check({
    id: 'seek/ctrl-home',
    title: 'Ctrl+Home restarts the track',
    body: async function ctrlHomeRestartsTrack(): Promise<string> {
      await press({
        session,
        act: chordKeys({
          mods: [
            'alt',
            'shift',
          ],
          key: 'right',
        },),
        until: isFarForward,
        what: 'a seek forward',
      },);

      /**
       The state back at the start.
       */
      const state = await press({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'home',
        },),
        until: isAtStart,
        what: 'the start',
      },);
      return `position ${state.position
        .toFixed(2,)} s`;
    },
  },);

  /**
   The position the next seek should land on, kept in a one-slot holder.
   */
  const target = { expected: (await session.last()).position, };
  for (const step of SEEK_STEPS)
    // oxlint-disable-next-line eslint/no-await-in-loop -- each seek starts where the previous one landed
    await recorder.check({
      id: step.id,
      title: step.title,
      body: async function seekLands(): Promise<string> {
        target.expected += step.delta;

        /**
         The state once the seek landed.
         */
        const state = await press({
          session,
          act: chordKeys({
            mods: step.mods,
            key: step.key,
          },),
          until: positionNear({
            expected: target.expected,
            tolerance: SEEK_TOLERANCE_S,
          },),
          what: `position near ${target.expected
            .toFixed(1,)} s`,
        },);
        return `position ${state.position
          .toFixed(2,)} s`;
      },
    },);
  await recorder.check({
    id: 'seek/alt-right-held',
    title: 'a held Alt+Right keeps seeking (key repeat)',
    body: async function heldChordKeepsSeeking(): Promise<string> {
      await press({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'home',
        },),
        until: isAtStart,
        what: 'the start',
      },);

      /**
       Position before the hold.
       */
      const before = (await session.last()).position;
      await session.down('alt',);
      await session.down('right',);
      await wait(HOLD_MS,);
      await session.up('right',);
      await session.up('alt',);
      await wait(AFTER_HOLD_MS,);

      /**
       Seconds gained while the chord was held.
       */
      const gained = (await session.last()).position - before;
      expect({
        condition: gained > HELD_GAIN_S,
        message: `only ${gained.toFixed(1,)} s gained while held: no repeat reached the seek`,
      },);
      return `${gained.toFixed(1,)} s gained in 0.9 s (each press is 10 s)`;
    },
  },);
  await recorder.check({
    id: 'seek/hold-repeat-count',
    title: 'a held Alt+Right repeats at the rate and delay the compositor announces, and not at all when repeat is off',
    body: async function holdRepeatFollowsAnnouncedRate(): Promise<string> {
      await press({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'home',
        },),
        until: isAtStart,
        what: 'the start',
      },);

      /**
       Position before the first hold.
       */
      const first = (await session.last()).position;
      await session.ok(`repeat-info ${SLOW_REPEAT_RATE} ${SLOW_REPEAT_DELAY_MS}`,);
      await session.ok(`key alt+right hold ${COUNTED_HOLD_MS}`,);
      await wait(AFTER_HOLD_MS,);

      /**
       Seconds gained with a repeat of 5 a second after 400 ms: the press itself and five repeats.
       */
      const repeated = (await session.last()).position - first;
      await press({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'home',
        },),
        until: isAtStart,
        what: 'the start again',
      },);

      /**
       Position before the second hold.
       */
      const second = (await session.last()).position;
      await session.ok('repeat-info 0 400',);
      await session.ok(`key alt+right hold ${COUNTED_HOLD_MS}`,);
      await wait(AFTER_HOLD_MS,);

      /**
       Seconds gained with repeat off: the press alone.
       */
      const single = (await session.last()).position - second;
      await session.ok(`repeat-info ${DEFAULT_REPEAT_RATE} ${DEFAULT_REPEAT_DELAY_MS}`,);
      expect({
        condition: (repeated >= COUNTED_MIN_GAIN_S) && (repeated <= COUNTED_MAX_GAIN_S),
        message: `${repeated.toFixed(1,)} s gained at 5 repeats a second after 400 ms; expected about 60 s`,
      },);
      expect({
        condition: Math.abs(single - SEEK_PRESS_S,) < SEEK_TOLERANCE_S,
        message: `${single.toFixed(1,)} s gained with repeat off; expected one press, ${SEEK_PRESS_S} s`,
      },);
      return `${repeated.toFixed(1,)} s with repeat 5 a second after 400 ms, ${single.toFixed(1,)} s with repeat off`;
    },
  },);
  await recorder.check({
    id: 'seek/bar-arrows',
    title: 'a focused seek bar moves with Left and Right',
    body: async function barArrowsMoveSeekBar(): Promise<string> {
      await press({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'home',
        },),
        until: isAtStart,
        what: 'the start',
      },);
      await tabTo({
        session,
        label: 'seek',
      },);

      /**
       Position before the arrows.
       */
      const base = (await session.last()).position;

      /**
       The state after a step forward.
       */
      const right = await press({
        session,
        act: tapKey('right',),
        until: function movedForward(state: UiState,): boolean {
          return state.position > (base
            + BAR_STEP_S);
        },
        what: 'a step forward',
      },);

      /**
       The state after a step back.
       */
      const left = await press({
        session,
        act: tapKey('left',),
        until: function movedBack(state: UiState,): boolean {
          return state.position < (right.position
            - BAR_STEP_S);
        },
        what: 'a step back',
      },);
      return `${base.toFixed(1,)} > ${right.position
        .toFixed(1,)} > ${left.position
          .toFixed(1,)} s`;
    },
  },);
  await recorder.check({
    id: 'mode/group-arrows',
    title: 'a focused mode group moves with Left and Right',
    body: async function groupArrowsChangeMode(): Promise<string> {
      await tabTo({
        session,
        label: 'mode-group',
      },);

      /**
       Mode before the arrows.
       */
      const start = (await session.last()).mode;

      /**
       The state after Right.
       */
      const right = await press({
        session,
        act: tapKey('right',),
        until: function modeChanged(state: UiState,): boolean {
          return state.mode !== start;
        },
        what: 'a mode change',
      },);

      /**
       The state after Left.
       */
      const back = await press({
        session,
        act: tapKey('left',),
        until: function modeRestored(state: UiState,): boolean {
          return state.mode === start;
        },
        what: 'back to the first mode',
      },);
      return `${start} > ${right.mode} > ${back.mode}`;
    },
  },);
}
