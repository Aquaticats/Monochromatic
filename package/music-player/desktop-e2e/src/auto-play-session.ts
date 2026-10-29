/**
 The auto-play session: the Auto-play on select setting (D129, D133) in two launches of the app over the same scratch
 configuration. The first launch checks that the setting is on in a fresh configuration and a click selects and plays at
 once, turns the setting off in Settings with the keyboard, and checks that a click then only selects (D124); the second
 launch checks that the off setting survived Ctrl+Q and a restart and a click still only selects.

 @example
 ```ts
 import { runAutoPlaySession, } from './auto-play-session.ts';

 await runAutoPlaySession(context,);
 ```

 @packageDocumentation
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  chordKeys,
  clickAt,
  isPaused,
  modeIs,
  tapKey,
} from './actions.ts';
import {
  type RunContext,
  startContextSession,
} from './context.ts';
import { expect, } from './errors.ts';
import { pathExists, } from './files.ts';
import {
  WIDE_POINTS,
  WIDE_WINDOW,
} from './layout.ts';
import { pollUntil, } from './poll.ts';
import { press, } from './press.ts';
import type { Session, } from './session.ts';
import { SESSION_READY_MS, } from './timing.ts';
import {
  NO_INDEX,
  type UiState,
} from './ui-state.ts';

/**
 Label of both launches, which names the scratch configuration the second launch finds again.
 */
const LABEL = 'auto-play';

/**
 How long a selected row is watched to see that it stays paused, in milliseconds.
 */
const STAYS_PAUSED_MS = 1_500;

/**
 Distance from the first track row's click point to a point on the second row, in pixels.
 */
const SECOND_ROW_OFFSET_PX = 90;

/**
 How many Up presses lead from the Licenses row, the last stop of Settings, to the Auto-play on select switch row.
 */
const ROWS_FROM_LICENSES_TO_SWITCH = 6;

/**
 How long the app is given to move the keyboard position after a navigation key of the Settings list, in milliseconds.
 */
const MOVE_SETTLE_MS = 250;

/**
 How many times the app process is looked for after Ctrl+Q.
 */
const EXIT_LOOKS = 40;

/**
 Pause between two looks for the app process.
 */
const EXIT_LOOK_PAUSE_MS = 200;

/**
 Row A: the first track row of the folder on screen.
 */
const ROW_A = WIDE_POINTS.trackRow;

/**
 Row B: the second track row of the folder on screen.
 */
const ROW_B = {
  x: WIDE_POINTS.trackRow
    .x,
  y: WIDE_POINTS.trackRow
    .y
    + SECOND_ROW_OFFSET_PX,
} as const;

/**
 Condition: a folder is shown, so the player page is up.

 @param state - a state of the app

 @returns whether a folder shows

 @example
 ```ts
 hasFolder(state,);
 ```
 */
function hasFolder(state: UiState,): boolean {
  return state.folder !== '';
}

/**
 Condition: the Settings page shows.

 @param state - a state of the app

 @returns whether the page is Settings

 @example
 ```ts
 isSettingsPage(state,);
 ```
 */
function isSettingsPage(state: UiState,): boolean {
  return state.route === 'Settings';
}

/**
 Condition: the player page shows.

 @param state - a state of the app

 @returns whether the page is the player

 @example
 ```ts
 isPlayerRoute(state,);
 ```
 */
function isPlayerRoute(state: UiState,): boolean {
  return state.route === 'Player';
}

/**
 Starts a launch of the app in the wide window and waits for the player page.

 @param context - the run's context

 @returns the running session; disposing it stops the unit

 @throws CheckFailureError when the session cannot start or never shows the player page

 @example
 ```ts
 await using session = await launch(context,);
 ```
 */
async function launch(context: RunContext,): Promise<Session> {
  /**
   The running launch.
   */
  const session = await startContextSession({
    context,
    options: {
      ...WIDE_WINDOW,
      scheme: 'dark',
      accent: '3B82F6',
      library: context.library,
      label: LABEL,
      keepSettingsDefaults: true,
    },
  },);
  await session.waitState({
    predicate: hasFolder,
    what: 'the player page',
    timeoutMs: SESSION_READY_MS,
  },);
  await session.findAppPid();
  return session;
}

/**
 Selects Repeat so that a short track under the clicks never moves on by itself.

 @param session - the running launch

 @example
 ```ts
 await keepTrackFromMovingOn(session,);
 ```
 */
async function keepTrackFromMovingOn(session: Session,): Promise<void> {
  await press({
    session,
    act: chordKeys({
      mods: [
        'ctrl',
        'shift',
      ],
      key: '1',
    },),
    until: modeIs(0,),
    what: 'Repeat',
  },);
}

/**
 Quits the app with Ctrl+Q and waits until its process is gone, so that the shutdown has saved what it saves.

 @param session - the running launch

 @throws CheckFailureError when the process is not found or is still running after Ctrl+Q

 @example
 ```ts
 await quitApp(session,);
 ```
 */
async function quitApp(session: Session,): Promise<void> {
  /**
   Process id of the app, found by its own unit.
   */
  const pid = await session.findAppPid();
  expect({
    condition: pid !== '',
    message: 'the app process was not found',
  },);
  await session.chord({
    mods: ['ctrl',],
    key: 'q',
  },);

  /**
   Whether the process is gone.
   */
  const gone = await pollUntil({
    attempts: EXIT_LOOKS,
    intervalMs: EXIT_LOOK_PAUSE_MS,
    order: 'wait-then-check',
    check: async function processIsGone(): Promise<boolean> {
      return !(await pathExists(`/proc/${pid}`,));
    },
  },);
  expect({
    condition: gone,
    message: `the app process ${pid} is still running after Ctrl+Q`,
  },);
}

/**
 Records the checks of the first launch: the setting is on at first and a click plays at once, the switch in Settings
 turns it off, and a click then only selects; the app quits at the end.

 @param context - the run's context

 @throws CheckFailureError when a launch cannot start or the app does not quit

 @example
 ```ts
 await runFirstLaunch(context,);
 ```
 */
async function runFirstLaunch(context: RunContext,): Promise<void> {
  /**
   The recorder of the run.
   */
  const { recorder, } = context;

  /**
   The first launch; leaving the function stops it.
   */
  await using session = await launch(context,);
  await keepTrackFromMovingOn(session,);
  recorder.setGroup('auto-play',);

  /**
   Library indexes of rows A and B, set by the first check.
   */
  const rows = {
    a: NO_INDEX,
    b: NO_INDEX,
  };
  await recorder.check({
    id: 'auto-play/on-click-selects-and-plays',
    title: 'with no saved setting Auto-play on select is on, and a click on another row selects it and starts it at once; '
      + 'the selected row still toggles (D133)',
    body: async function onByDefault(): Promise<string> {
      expect({
        condition: (await session.last()).auto_play_on_select,
        message: 'the setting is off in a fresh configuration',
      },);
      /**
       The state after the click on row A with the setting on.
       */
      const playingA = await press({
        session,
        act: clickAt(ROW_A,),
        until: function rowAPlaysAtOnce(state: UiState,): boolean {
          return (state.current_index !== NO_INDEX) && state.playing;
        },
        what: 'a click on row A selecting it and playing it at once',
      },);
      rows.a = playingA.current_index;
      await session.shot('auto-play-on-row-a-playing',);
      /**
       The state after the click on row B with the setting on.
       */
      const playingB = await press({
        session,
        act: clickAt(ROW_B,),
        until: function rowBPlaysAtOnce(state: UiState,): boolean {
          return (state.current_index !== rows.a) && (state.current_index !== NO_INDEX) && state.playing;
        },
        what: 'a click on row B selecting it and playing it at once',
      },);
      rows.b = playingB.current_index;
      /**
       The state after the click on the playing row B.
       */
      const pausedB = await press({
        session,
        act: clickAt(ROW_B,),
        until: function rowBPaused(state: UiState,): boolean {
          return (state.current_index === rows.b) && (!state.playing);
        },
        what: 'a click on the playing row B pausing it',
      },);
      return `row A ${rows.a} played at once, row B ${rows.b} played at once (${playingB.playing}) `
        + `and the next click paused it (${!pausedB.playing})`;
    },
  },);

  await recorder.check({
    id: 'auto-play/switch-turns-it-off',
    title: 'End, Up and Space on the Auto-play on select row of Settings turn the setting off and leave Settings open (D129)',
    body: async function switchTurnsItOff(): Promise<string> {
      await press({
        session,
        act: chordKeys({
          mods: [
            'ctrl',
            'alt',
          ],
          key: 's',
        },),
        until: isSettingsPage,
        what: 'Settings',
      },);
      // A move between rows writes no state line, so each key is waited out for a moment instead of waited for: waiting for
      // a newer line only passed when a late hint-badge line of the chord happened to arrive.
      await session.tap('end',);
      await wait(MOVE_SETTLE_MS,);
      // The Licenses row is the last stop; the switch row is six rows above it, past the Reset text sizes row and the four
      // Text size stepper rows.
      for (let rowsUp = 0; rowsUp < ROWS_FROM_LICENSES_TO_SWITCH; rowsUp += 1) {
        await session.tap('up',);
        await wait(MOVE_SETTLE_MS,);
      }
      expect({
        condition: isSettingsPage(await session.last(),) && (await session.last()).auto_play_on_select,
        message: 'moving the keyboard position toggled the setting or left Settings',
      },);
      /**
       The state after Space on the switch row.
       */
      const turnedOff = await press({
        session,
        act: tapKey('space',),
        until: function settingOff(state: UiState,): boolean {
          return !state.auto_play_on_select;
        },
        what: 'Space turning Auto-play on select off',
      },);
      expect({
        condition: isSettingsPage(turnedOff,),
        message: `toggling the setting left Settings (route ${turnedOff.route})`,
      },);
      await session.shot('auto-play-switch-off',);
      await press({
        session,
        act: tapKey('escape',),
        until: isPlayerRoute,
        what: 'the player page after Escape',
      },);
      return 'the switch row turned the setting off in place';
    },
  },);

  await recorder.check({
    id: 'auto-play/off-click-selects-paused',
    title: 'with the setting off, a click on another row selects it paused and stops the playing row (D124, D129)',
    body: async function offClickSelects(): Promise<string> {
      // Row B is selected and paused from the first check; with the setting off a click on row A only selects it, and a
      // second click on the selected row plays it.
      await press({
        session,
        act: clickAt(ROW_A,),
        until: function rowASelectedPaused(state: UiState,): boolean {
          return (state.current_index === rows.a) && (!state.playing);
        },
        what: 'a click on row A selecting it paused',
      },);
      await press({
        session,
        act: clickAt(ROW_A,),
        until: function rowAPlays(state: UiState,): boolean {
          return (state.current_index === rows.a) && state.playing;
        },
        what: 'a click on the selected row A playing it',
      },);

      // Row B is another row while row A plays: with the setting off the click selects it and stops row A.
      /**
       The state after the click on row B.
       */
      const selectedB = await press({
        session,
        act: clickAt(ROW_B,),
        until: function rowBSelectedPaused(state: UiState,): boolean {
          return (state.current_index === rows.b) && (!state.playing);
        },
        what: 'a click on row B selecting it paused and stopping row A',
      },);
      await session.shot('auto-play-off-row-b-selected',);
      await wait(STAYS_PAUSED_MS,);
      expect({
        condition: (!(await session.last()).playing) && ((await session.last()).current_index === rows.b),
        message: 'row B started playing by itself with the setting off',
      },);
      return `row A ${rows.a} played; the click on row B ${selectedB.current_index} left it paused with the setting off`;
    },
  },);
  await quitApp(session,);
}

/**
 Records the check of the second launch: the setting saved off in the first launch is still off, and a click only selects.

 @param context - the run's context

 @throws CheckFailureError when the launch cannot start

 @example
 ```ts
 await runSecondLaunch(context,);
 ```
 */
async function runSecondLaunch(context: RunContext,): Promise<void> {
  /**
   The recorder of the run.
   */
  const { recorder, } = context;

  /**
   The second launch over the same scratch configuration; leaving the function stops it.
   */
  await using session = await launch(context,);
  await recorder.check({
    id: 'auto-play/setting-survives-restart',
    title: 'the setting is still off after Ctrl+Q and a restart, and a click on a row only selects it (D129)',
    body: async function settingSurvives(): Promise<string> {
      expect({
        condition: !(await session.last()).auto_play_on_select,
        message: 'the setting was on again after the restart',
      },);
      await keepTrackFromMovingOn(session,);
      /**
       The state after the click on row B in the second launch.
       */
      const state = await press({
        session,
        act: clickAt(ROW_B,),
        until: function rowSelectedPaused(candidate: UiState,): boolean {
          return (candidate.current_index !== NO_INDEX) && (!candidate.playing);
        },
        what: 'a click on row B selecting it paused after the restart',
      },);
      await session.shot('auto-play-after-restart-selected',);
      await wait(STAYS_PAUSED_MS,);
      expect({
        condition: !(await session.last()).playing,
        message: 'row B started playing by itself after the restart with the setting off',
      },);
      return `the setting stayed off; the click on row ${state.current_index} left it paused`;
    },
  },);
}

/**
 Runs the auto-play session: the first launch with its checks and its quit, then the second launch.

 @param context - the run's context

 @throws CheckFailureError when a launch cannot start or the app does not quit

 @throws SafetyStopError when the app plays anywhere but the private sink

 @example
 ```ts
 await runAutoPlaySession(context,);
 ```
 */
export async function runAutoPlaySession(context: RunContext,): Promise<void> {
  await runFirstLaunch(context,);
  await runSecondLaunch(context,);
}
