/**
 The dialog session: Ctrl+Shift+O in a session of its own.
 
 The system folder dialog is a window of its own. Compositor 0.3.0 and older did not hand keyboard focus back to the
 player window after that dialog closed (probed on 0.3.0: after Escape closed it and after a click on the player, no
 key reached the app), which is why this session is separate from the wide one. From compositor 0.4.0 on, focus returns
 to the window that had it, and the second check proves that a key reaches the player afterwards.
 
 @example
 ```ts
 import { runDialogSession, } from './dialog-session.ts';
 
 await runDialogSession(context,);
 ```
 
 @packageDocumentation
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import { chordKeys, } from './actions.ts';
import { isSearchWithField, } from './conditions.ts';
import {
  type RunContext,
  startContextSession,
} from './context.ts';
import { expect, } from './errors.ts';
import {
  diffBox,
  type Difference,
  loadImage,
} from './image.ts';
import { WIDE_WINDOW, } from './layout.ts';
import { pollUntil, } from './poll.ts';
import { press, } from './press.ts';
import { SESSION_READY_MS, } from './timing.ts';
import type { UiState, } from './ui-state.ts';

/**
 Share of the window a dialog must cover to count as shown.
 */
const DIALOG_COVER_SHARE = 0.3;

/**
 Share of the window that may still differ once the dialog is closed.
 */
const DIALOG_CLOSED_SHARE = 0.01;

/**
 How many times the screen is looked at for the dialog.
 */
const DIALOG_LOOKS = 20;

/**
 Pause between two looks at the screen.
 */
const DIALOG_LOOK_PAUSE_MS = 300;

/**
 Wait after Escape, so the dialog is gone before the screenshot.
 */
const DIALOG_CLOSE_WAIT_MS = 800;

/**
 Condition: the wide layout shows the player page with its first folder.
 
 @param state - a state of the app
 
 @returns whether a folder is shown and the layout is unfolded
 
 @example
 ```ts
 isUnfoldedWithFolder(state,);
 ```
 */
function isUnfoldedWithFolder(state: UiState,): boolean {
  return (state.folder !== '') && state.unfolded;
}

/**
 Runs the dialog session from start to quit.
 
 @param context - the run's context
 
 @throws CheckFailureError when the session cannot start or never shows the player page
 
 @example
 ```ts
 await runDialogSession(context,);
 ```
 */
export async function runDialogSession(context: RunContext,): Promise<void> {
  /**
   The recorder of the run.
   */
  const { recorder, } = context;

  /**
   The running dialog session; leaving the function stops it.
   */
  await using session = await startContextSession({
    context,
    options: {
      ...WIDE_WINDOW,
      scheme: 'dark',
      accent: '3B82F6',
      library: context.library,
      label: 'dialog',
    },
  },);
  await session.waitState({
    predicate: isUnfoldedWithFolder,
    what: 'the unfolded player page',
    timeoutMs: SESSION_READY_MS,
  },);
  recorder.setGroup('dialog',);
  await recorder.check({
    id: 'browse/ctrl-shift-o',
    title: 'Ctrl+Shift+O opens the system folder dialog',
    body: async function ctrlShiftOOpensDialog(): Promise<string> {
      /**
       The player window before the chord.
       */
      const before = await loadImage(await session.shot('dialog-before',),);

      /**
       Folder-dialog requests before the chord.
       */
      const used = await session.used('open-folder-dialog',);
      await session.chord({
        mods: [
          'ctrl',
          'shift',
        ],
        key: 'o',
      },);

      /**
       Pixels that make a dialog count as shown.
       */
      const coverPixels = before.width * before.height
        * DIALOG_COVER_SHARE;

      /**
       The latest difference between the window before and now, kept in a one-slot holder.
       */
      const seen: { difference: Difference; } = { difference: { count: 0, }, };

      /**
       Looks at the screen once.
       
       @returns whether a dialog covers the player
       */
      async function dialogCoversPlayer(): Promise<boolean> {
        seen.difference = diffBox({
          a: before,
          b: await loadImage(await session.shot('dialog-open',),),
        },);
        return seen.difference
          .count
          >= coverPixels;
      }

      await pollUntil({
        attempts: DIALOG_LOOKS,
        intervalMs: DIALOG_LOOK_PAUSE_MS,
        order: 'wait-then-check',
        check: dialogCoversPlayer,
      },);
      expect({
        condition: await session.used('open-folder-dialog',) === (used
          + 1),
        message: 'the app did not use Ctrl+Shift+O as the folder dialog request',
      },);
      expect({
        condition: seen.difference
          .count
          > coverPixels,
        message: `no dialog window covered the player (${seen.difference
          .count} pixels differ)`,
      },);
      await session.tap('escape',);
      await wait(DIALOG_CLOSE_WAIT_MS,);

      /**
       How the window differs from the start once Escape was pressed.
       */
      const closed = diffBox({
        a: before,
        b: await loadImage(await session.shot('dialog-closed',),),
      },);
      expect({
        condition: closed.count < (before.width
          * before.height
          * DIALOG_CLOSED_SHARE),
        message: `the dialog is still showing after Escape (${closed.count} pixels differ)`,
      },);
      return `a dialog covered ${seen.difference
        .count} pixels, and Escape closed it`;
    },
  },);
  await recorder.check({
    id: 'dialog/focus-returns',
    title: 'after the system folder dialog closes, the player window has the keyboard again and a key reaches it',
    body: async function keysReachPlayerAfterDialog(): Promise<string> {
      /**
       The compositor's list of windows: only the player is left, and it has keyboard focus.
       */
      const windows = await session.send('windows',);
      expect({
        condition: windows.startsWith('ok 1 ',) && windows.includes('focused=true',),
        message: `the compositor lists "${windows}" after the dialog closed`,
      },);

      /**
       The state with Search open, which only a key that reached the player can cause.
       */
      const state = await press({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'f',
        },),
        until: isSearchWithField,
        what: 'Search opened by Ctrl+F after the dialog',
      },);
      return `${windows}; Ctrl+F opened ${state.route} with focus ${state.focus}`;
    },
  },);
}
