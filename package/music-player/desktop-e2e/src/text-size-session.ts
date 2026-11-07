/**
 The text-size session: the Text size setting in two launches of the app over the same scratch configuration. The first
 launch checks the default sizes, changes sizes with Left and Right on the stepper rows of Settings (one pixel a press,
 stopping at 10), puts all four back with the Reset text sizes row, and sets two sizes again before quitting; the second
 launch checks that the two sizes survived Ctrl+Q and a restart.

 @example
 ```ts
 import { runTextSizeSession, } from './text-size-session.ts';

 await runTextSizeSession(context,);
 ```

 @packageDocumentation
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  type Action,
  chordKeys,
  pauseFor,
  repeatAction,
  sequenceOf,
  tapKey,
} from './actions.ts';
import {
  type RunContext,
  startContextSession,
} from './context.ts';
import { expect, } from './errors.ts';
import { pathExists, } from './files.ts';
import { WIDE_WINDOW, } from './layout.ts';
import { pollUntil, } from './poll.ts';
import {
  noEffect,
  press,
} from './press.ts';
import type { Session, } from './session.ts';
import { SESSION_READY_MS, } from './timing.ts';
import type { UiState, } from './ui-state.ts';

/**
 Label of both launches, which names the scratch configuration the second launch finds again.
 */
const LABEL = 'text-size';

/**
 How many times the app process is looked for after Ctrl+Q.
 */
const EXIT_LOOKS = 40;

/**
 Pause between two looks for the app process.
 */
const EXIT_LOOK_PAUSE_MS = 200;

/**
 How long the app is given to move the keyboard position after a navigation key, in milliseconds. A move between rows
 writes no state line, so it is waited out instead of waited for.
 */
const MOVE_SETTLE_MS = 250;

/**
 How many Down presses lead from the first Settings row to the first stepper: the two template rows and the switch row.
 */
const DOWNS_TO_FIRST_STEPPER = 3;

/**
 How many Down presses lead from the Pane titles stepper to the Folder names stepper: past the Group headings stepper.
 */
const DOWNS_FROM_PANE_TITLES_TO_FOLDER_NAMES = 2;

/**
 How many Down presses lead from the Pane titles stepper to the Track titles stepper: past the Group headings and
 Folder names steppers.
 */
const DOWNS_FROM_PANE_TITLES_TO_TRACK_TITLES = 3;

/**
 How long the player page is given to redraw with the new sizes after Escape, in milliseconds.
 */
const PLAYER_PAGE_REDRAW_MS = 1_000;

/**
 Pane title size of a fresh configuration, in pixels.
 */
const DEFAULT_PANE_TITLE_PX = 20;

/**
 Group heading size of a fresh configuration, in pixels.
 */
const DEFAULT_GROUP_HEADING_PX = 20;

/**
 Folder name size of a fresh configuration, in pixels.
 */
const DEFAULT_FOLDER_NAME_PX = 14;

/**
 Track title size of a fresh configuration, in pixels.
 */
const DEFAULT_TRACK_TITLE_PX = 14;

/**
 How much one press of Right makes a size larger, and one press of Left makes it smaller, in pixels.
 */
const STEP_PX = 1;

/**
 The smallest size a stepper reaches: Left stops there.
 */
const SMALLEST_PX = 10;

/**
 Pane title size after one press of Right from the default, in pixels.
 */
const PANE_TITLE_ONE_RIGHT_PX = DEFAULT_PANE_TITLE_PX + STEP_PX;

/**
 Pane title size after two presses of Right from the default, in pixels.
 */
const PANE_TITLE_TWO_RIGHTS_PX = PANE_TITLE_ONE_RIGHT_PX + STEP_PX;

/**
 Track title size after one press of Right from the default, in pixels.
 */
const TRACK_TITLE_ONE_RIGHT_PX = DEFAULT_TRACK_TITLE_PX + STEP_PX;

/**
 Track title size after two presses of Right from the default, in pixels.
 */
const TRACK_TITLE_TWO_RIGHTS_PX = TRACK_TITLE_ONE_RIGHT_PX + STEP_PX;

/**
 The four sizes of a state, in the order of the Settings rows.
 */
type Sizes = readonly [
  paneTitle: number,
  groupHeading: number,
  folderName: number,
  trackTitle: number,
];

/**
 Sizes that differ from the default; a size left out stays the default.
 */
type SizeChanges = {
  readonly folderName?: number;
  readonly groupHeading?: number;
  readonly paneTitle?: number;
  readonly trackTitle?: number;
};

/**
 Builds the four sizes as the default ones with some replaced, so that every wanted size is the default moved by the
 presses that are expected, never a typed number.

 @param changes - the sizes that differ from the default

 @returns the four sizes

 @example
 ```ts
 defaultSizesWith({ paneTitle: PANE_TITLE_ONE_RIGHT_PX, },);
 ```
 */
function defaultSizesWith(changes: SizeChanges,): Sizes {
  return [
    changes.paneTitle ?? DEFAULT_PANE_TITLE_PX,
    changes.groupHeading ?? DEFAULT_GROUP_HEADING_PX,
    changes.folderName ?? DEFAULT_FOLDER_NAME_PX,
    changes.trackTitle ?? DEFAULT_TRACK_TITLE_PX,
  ];
}

/**
 The four sizes of a fresh configuration.
 */
const DEFAULT_SIZES = defaultSizesWith({},);

/**
 The two sizes the first launch sets last and the second launch finds again after the restart.
 */
const SAVED_SIZES = defaultSizesWith({
  paneTitle: PANE_TITLE_ONE_RIGHT_PX,
  trackTitle: TRACK_TITLE_TWO_RIGHTS_PX,
},);

/**
 The Folder names sizes that Left passes through from the default down to the smallest size, in order.
 */
const FOLDER_NAME_SIZES_ON_THE_WAY_DOWN: readonly number[] = Array.from(
  { length: (DEFAULT_FOLDER_NAME_PX - SMALLEST_PX) / STEP_PX, },
  function folderNameAfterLeftPresses(
    _unused: unknown,
    index: number,
  ): number {
    return DEFAULT_FOLDER_NAME_PX - ((index + 1) * STEP_PX);
  },
);

/**
 Reads the four sizes of a state.

 @param state - a state of the app

 @returns the pane title, group heading, folder name and track title sizes

 @example
 ```ts
 sizesOf(state,); // [20, 20, 14, 14]
 ```
 */
function sizesOf(state: UiState,): Sizes {
  return [
    state.text_size_pane_title_px,
    state.text_size_group_heading_px,
    state.text_size_folder_name_px,
    state.text_size_track_title_px,
  ];
}

/**
 Condition factory: the four sizes are exactly the given ones.

 @param wanted - the four sizes

 @returns the condition

 @example
 ```ts
 sizesAre(DEFAULT_SIZES,);
 ```
 */
function sizesAre(wanted: Sizes,): (state: UiState,) => boolean {
  return function hasSizes(state: UiState,): boolean {
    return JSON.stringify(sizesOf(state,),) === JSON.stringify(wanted,);
  };
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
 Builds the action that taps a navigation key of the Settings list and gives the app a moment to move the keyboard
 position, which writes no state line.

 @param key - key name

 @returns the action

 @example
 ```ts
 await tapAndSettle('down',)(session,);
 ```
 */
function tapAndSettle(key: string,): Action {
  return sequenceOf([
    tapKey(key,),
    pauseFor(MOVE_SETTLE_MS,),
  ],);
}

/**
 Moves the keyboard to the first row of the Settings list.
 */
const MOVE_TO_FIRST_ROW = tapAndSettle('home',);

/**
 Moves the keyboard to the last row of the Settings list, the Reset text sizes row.
 */
const MOVE_TO_LAST_ROW = tapAndSettle('end',);

/**
 Moves the keyboard up by one row without changing a size.
 */
const MOVE_UP_ONE_ROW = tapAndSettle('up',);

/**
 Moves the keyboard down by one row without changing a size.
 */
const MOVE_DOWN_ONE_ROW = tapAndSettle('down',);

/**
 Opens Settings and puts the keyboard on the first stepper row, Pane titles.

 @param session - the running launch

 @throws CheckFailureError when Settings does not open

 @example
 ```ts
 await openSettingsOnFirstStepper(session,);
 ```
 */
async function openSettingsOnFirstStepper(session: Session,): Promise<void> {
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
  await wait(MOVE_SETTLE_MS,);
  await MOVE_TO_FIRST_ROW(session,);
  await repeatAction({
    action: MOVE_DOWN_ONE_ROW,
    times: DOWNS_TO_FIRST_STEPPER,
  },)(session,);
}

/**
 Presses Left or Right on the stepper row under the keyboard and waits for the size it must cause.

 @param session - the running launch

 @param key - `left` or `right`

 @param wanted - the four sizes after the press

 @returns the state with the wanted sizes

 @throws CheckFailureError when the sizes do not arrive

 @example
 ```ts
 await stepTo({ session, key: 'right', wanted: defaultSizesWith({ paneTitle: PANE_TITLE_ONE_RIGHT_PX, },), },);
 ```
 */
async function stepTo(
  {
    session,
    key,
    wanted,
  }: {
    readonly key: 'left' | 'right';
    readonly session: Session;
    readonly wanted: Sizes;
  },
): Promise<UiState> {
  return await press({
    session,
    act: tapKey(key,),
    until: sizesAre(wanted,),
    what: `${key} arrow giving the sizes ${JSON.stringify(wanted,)}`,
  },);
}

/**
 Builds the action that presses Left on the stepper row under the keyboard and waits for the sizes it must cause.

 @param wanted - the four sizes after the press

 @returns the action

 @example
 ```ts
 await pressingLeftGives(defaultSizesWith({ folderName: SMALLEST_PX, },),)(session,);
 ```
 */
function pressingLeftGives(wanted: Sizes,): Action {
  return async function pressLeft(session: Session,): Promise<void> {
    await stepTo({
      session,
      key: 'left',
      wanted,
    },);
  };
}

/**
 Records the checks of the first launch; the app quits at the end.

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
  recorder.setGroup('text-size',);
  await recorder.check({
    id: 'text-size/defaults-are-todays-sizes',
    title: 'with no saved setting the four sizes are the ones the player always drew: 20, 20, 14 and 14 px',
    body: async function defaults(): Promise<string> {
      /**
       The first state of the launch.
       */
      const state = await session.last();
      expect({
        condition: sizesAre(DEFAULT_SIZES,)(state,),
        message: `the sizes in a fresh configuration are ${JSON.stringify(sizesOf(state,),)}`,
      },);
      await session.shot('text-size-defaults-player',);
      return `sizes ${JSON.stringify(sizesOf(state,),)}`;
    },
  },);

  await recorder.check({
    id: 'text-size/keys-step-a-size',
    title: 'Right and Left on a stepper row of Settings change that size by one pixel, and Left stops at 10 px',
    body: async function keysStep(): Promise<string> {
      await openSettingsOnFirstStepper(session,);
      await stepTo({
        session,
        key: 'right',
        wanted: defaultSizesWith({ paneTitle: PANE_TITLE_ONE_RIGHT_PX, },),
      },);
      await stepTo({
        session,
        key: 'right',
        wanted: defaultSizesWith({ paneTitle: PANE_TITLE_TWO_RIGHTS_PX, },),
      },);
      await repeatAction({
        action: MOVE_DOWN_ONE_ROW,
        times: DOWNS_FROM_PANE_TITLES_TO_FOLDER_NAMES,
      },)(session,);
      await sequenceOf(FOLDER_NAME_SIZES_ON_THE_WAY_DOWN.map(
        function leftGivesFolderName(folderName: number,): Action {
          return pressingLeftGives(defaultSizesWith({
            paneTitle: PANE_TITLE_TWO_RIGHTS_PX,
            folderName,
          },),);
        },
      ),)(session,);
      await noEffect({
        session,
        act: tapKey('left',),
        watch: sizesOf,
        what: 'Left on Folder names at 10 px',
      },);
      await MOVE_DOWN_ONE_ROW(session,);
      await stepTo({
        session,
        key: 'right',
        wanted: defaultSizesWith({
          paneTitle: PANE_TITLE_TWO_RIGHTS_PX,
          folderName: SMALLEST_PX,
          trackTitle: TRACK_TITLE_ONE_RIGHT_PX,
        },),
      },);
      await session.shot('text-size-settings-section',);
      return 'pane titles 22, folder names 10 (stopped there), track titles 15';
    },
  },);

  await recorder.check({
    id: 'text-size/reset-row-restores-the-defaults',
    title: 'End, Up and Enter on the Reset text sizes row put all four sizes back, and the row then does nothing',
    body: async function resetRow(): Promise<string> {
      await MOVE_TO_LAST_ROW(session,);
      await MOVE_UP_ONE_ROW(session,);
      await press({
        session,
        act: tapKey('enter',),
        until: sizesAre(DEFAULT_SIZES,),
        what: 'Enter on the Reset text sizes row putting all four sizes back',
      },);
      await noEffect({
        session,
        act: tapKey('enter',),
        watch: sizesOf,
        what: 'Enter on the Reset text sizes row at the defaults',
      },);
      return 'all four sizes are 20, 20, 14 and 14 px again';
    },
  },);

  await recorder.check({
    id: 'text-size/sizes-show-in-the-player',
    title: 'two sizes set again are the ones the player page logs after Escape',
    body: async function setAgain(): Promise<string> {
      await openSettingsOnFirstStepper(session,);
      await stepTo({
        session,
        key: 'right',
        wanted: defaultSizesWith({ paneTitle: PANE_TITLE_ONE_RIGHT_PX, },),
      },);
      await repeatAction({
        action: MOVE_DOWN_ONE_ROW,
        times: DOWNS_FROM_PANE_TITLES_TO_TRACK_TITLES,
      },)(session,);
      await stepTo({
        session,
        key: 'right',
        wanted: defaultSizesWith({
          paneTitle: PANE_TITLE_ONE_RIGHT_PX,
          trackTitle: TRACK_TITLE_ONE_RIGHT_PX,
        },),
      },);
      await stepTo({
        session,
        key: 'right',
        wanted: SAVED_SIZES,
      },);
      await press({
        session,
        act: tapKey('escape',),
        until: isPlayerRoute,
        what: 'the player page after Escape',
      },);
      await wait(PLAYER_PAGE_REDRAW_MS,);
      await session.shot('text-size-21-and-16-player',);
      expect({
        condition: sizesAre(SAVED_SIZES,)(await session.last(),),
        message: 'leaving Settings changed a size',
      },);
      return 'pane titles 21 and track titles 16 px on the player page';
    },
  },);
  await quitApp(session,);
}

/**
 Records the check of the second launch: the two sizes saved in the first launch are still in force.

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
    id: 'text-size/sizes-survive-restart',
    title: 'the sizes set before Ctrl+Q are still in force after a restart',
    body: async function sizesSurvive(): Promise<string> {
      /**
       The state of the second launch.
       */
      const state = await session.last();
      expect({
        condition: sizesAre(SAVED_SIZES,)(state,),
        message: `the sizes after the restart are ${JSON.stringify(sizesOf(state,),)}, not ${JSON.stringify(SAVED_SIZES,)}`,
      },);
      await session.shot('text-size-after-restart-player',);
      return `sizes ${JSON.stringify(sizesOf(state,),)} after the restart`;
    },
  },);
}

/**
 Runs the text-size session: the first launch with its checks and its quit, then the second launch.

 @param context - the run's context

 @throws CheckFailureError when a launch cannot start or the app does not quit

 @throws SafetyStopError when the app plays anywhere but the private sink

 @example
 ```ts
 await runTextSizeSession(context,);
 ```
 */
export async function runTextSizeSession(context: RunContext,): Promise<void> {
  await runFirstLaunch(context,);
  await runSecondLaunch(context,);
}
