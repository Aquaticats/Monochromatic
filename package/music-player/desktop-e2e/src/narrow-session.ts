/**
 The narrow session: the app in the narrow window, light scheme, pink accent, where the layout folds and the folder
 list becomes a picker overlay. Its checks cover the picker, Escape, and the Settings editor page.
 
 @example
 ```ts
 import { runNarrowSession, } from './narrow-session.ts';
 
 await runNarrowSession(context,);
 ```
 
 @packageDocumentation
 */

import {
  chordKeys,
  QUICK_DOUBLE_SHIFT,
  routeIs,
  tapKey,
} from './actions.ts';
import { isPlayerPage, } from './conditions.ts';
import {
  type RunContext,
  startContextSession,
} from './context.ts';
import { expect, } from './errors.ts';
import {
  FOLDED_ORDER,
  NARROW_WINDOW,
} from './layout.ts';
import {
  noEffect,
  nothingMoved,
  press,
  tabSequence,
} from './press.ts';
import { checkNarrowClickThrough, } from './narrow-interaction.ts';
import { SESSION_READY_MS, } from './timing.ts';
import type { UiState, } from './ui-state.ts';

/**
 Settle time after the double Shift that should do nothing on the editor page.
 */
const DOUBLE_SHIFT_SETTLE_MS = 700;

/**
 Condition: the folder picker overlay is open and its list has the keyboard.
 
 @param state - a state of the app
 
 @returns whether the picker shows and the folder list is focused
 
 @example
 ```ts
 isPickerWithFocus(state,);
 ```
 */
function isPickerWithFocus(state: UiState,): boolean {
  return state.picker_open && (state.focus === 'folder-list');
}

/**
 Condition: the picker is closed and the track list has the keyboard.
 
 @param state - a state of the app
 
 @returns whether no picker shows and the track list is focused
 
 @example
 ```ts
 isPickerClosedOnTracks(state,);
 ```
 */
function isPickerClosedOnTracks(state: UiState,): boolean {
  return (!state.picker_open) && (state.focus === 'track-list');
}

/**
 Condition: the picker is open.
 
 @param state - a state of the app
 
 @returns whether the picker overlay shows
 
 @example
 ```ts
 isPickerOpen(state,);
 ```
 */
function isPickerOpen(state: UiState,): boolean {
  return state.picker_open;
}

/**
 Condition: the picker is closed.
 
 @param state - a state of the app
 
 @returns whether no picker shows
 
 @example
 ```ts
 isPickerClosed(state,);
 ```
 */
function isPickerClosed(state: UiState,): boolean {
  return !state.picker_open;
}

/**
 Condition: the layout is folded and a folder is shown.
 
 @param state - a state of the app
 
 @returns whether a folder shows and the layout is folded
 
 @example
 ```ts
 isFoldedWithFolder(state,);
 ```
 */
function isFoldedWithFolder(state: UiState,): boolean {
  return (state.folder !== '') && (!state.unfolded);
}

/**
 Runs the narrow session from start to quit.
 
 @param context - the run's context
 
 @throws CheckFailureError when the session cannot start or never shows the folded player page
 
 @example
 ```ts
 await runNarrowSession(context,);
 ```
 */
export async function runNarrowSession(context: RunContext,): Promise<void> {
  /**
   The recorder of the run.
   */
  const { recorder, } = context;

  /**
   The running narrow session; leaving the function stops it.
   */
  await using session = await startContextSession({
    context,
    options: {
      ...NARROW_WINDOW,
      scheme: 'light',
      accent: 'E91E63',
      library: context.library,
      label: 'narrow',
    },
  },);
  await session.waitState({
    predicate: isFoldedWithFolder,
    what: 'the folded player page',
    timeoutMs: SESSION_READY_MS,
  },);
  recorder.setGroup('folded',);
  await recorder.check({
    id: 'folded/tab-order',
    title: 'folded layout: Tab walks the folder trigger and Open, the track list, the deck, search, settings',
    body: async function foldedTabOrder(): Promise<string> {
      return await tabSequence({
        session,
        expected: FOLDED_ORDER,
        key: tapKey('tab',),
      },);
    },
  },);
  await recorder.check({
    id: 'folded/alt-1-opens-picker',
    title: 'Alt+1 opens the folder picker overlay and focuses its list',
    body: async function altOneOpensPicker(): Promise<string> {
      /**
       The state with the picker open.
       */
      const state = await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: '1',
        },),
        until: isPickerWithFocus,
        what: 'the picker with focus',
      },);
      return `picker_open ${state.picker_open}, focus ${state.focus}`;
    },
  },);
  await recorder.verify({
    id: 'folded/escape-closes-picker',
    title: 'Escape closes the picker overlay and focuses the track list, without leaving the page',
    body: async function escapeClosesPicker(): Promise<void> {
      /**
       The state with the picker closed.
       */
      const state = await press({
        session,
        act: tapKey('escape',),
        until: isPickerClosedOnTracks,
        what: 'the picker closing',
      },);
      expect({
        condition: state.route === 'Player',
        message: `route ${state.route}`,
      },);
    },
  },);
  await recorder.verify({
    id: 'folded/ctrl-o-toggles',
    title: 'Ctrl+O opens the picker and Ctrl+O closes it',
    body: async function ctrlOTogglesPicker(): Promise<void> {
      await press({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'o',
        },),
        until: isPickerWithFocus,
        what: 'the picker',
      },);
      await press({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'o',
        },),
        until: isPickerClosed,
        what: 'the picker closing',
      },);
    },
  },);
  await recorder.verify({
    id: 'folded/alt-2-closes-picker',
    title: 'Alt+2 closes the picker and focuses the track list',
    body: async function altTwoClosesPicker(): Promise<void> {
      await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: '1',
        },),
        until: isPickerOpen,
        what: 'the picker',
      },);
      await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: '2',
        },),
        until: isPickerClosedOnTracks,
        what: 'the track list',
      },);
    },
  },);
  await recorder.verify({
    id: 'folded/escape-on-player',
    title: 'Escape on the player page with nothing open does nothing',
    body: async function escapeOnPlayerDoesNothing(): Promise<void> {
      await noEffect({
        session,
        act: tapKey('escape',),
        watch: nothingMoved,
        what: 'Escape on the player page',
      },);
    },
  },);
  await recorder.check({
    id: 'folded/picker-enter-focuses-tracks',
    title: 'Enter on a folder in the picker shows it and moves the keyboard to the track list',
    body: async function enterInPickerShowsFolder(): Promise<string> {
      await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: '1',
        },),
        until: isPickerWithFocus,
        what: 'the picker',
      },);

      /**
       The folder shown before the picker choice.
       */
      const start = (await session.last()).folder;

      /**
       The state with the ring on another folder.
       */
      const down = await press({
        session,
        act: tapKey('down',),
        until: function ringOnOtherFolder(state: UiState,): boolean {
          return (state.ring_folder !== '') && (state.ring_folder !== start);
        },
        what: 'Down',
      },);

      /**
       The state once the picker closed on Enter.
       */
      const state = await press({
        session,
        act: tapKey('enter',),
        until: isPickerClosedOnTracks,
        what: 'the picker closing on Enter',
      },);
      expect({
        condition: state.folder === down.ring_folder,
        message: `folder ${state.folder}, ring was ${down.ring_folder}`,
      },);
      return `${start} > ${state.folder}`;
    },
  },);

  recorder.setGroup('editor-page',);
  await recorder.check({
    id: 'editor-page/enter-and-escape',
    title: 'Settings list Enter opens the editor page; Escape returns to the Settings list; Escape leaves Settings',
    body: async function editorPageEnterAndEscape(): Promise<string> {
      await press({
        session,
        act: chordKeys({
          mods: [
            'ctrl',
            'alt',
          ],
          key: 's',
        },),
        until: routeIs('Settings',),
        what: 'Settings',
      },);

      /**
       The state on the editor page.
       */
      const editor = await press({
        session,
        act: tapKey('enter',),
        until: routeIs('TemplateEditor',),
        what: 'the editor page',
      },);
      await noEffect({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'f',
        },),
        watch: nothingMoved,
        what: 'Ctrl+F on the editor page',
      },);
      await noEffect({
        session,
        act: QUICK_DOUBLE_SHIFT,
        watch: nothingMoved,
        what: 'double Shift on the editor page',
        settleMs: DOUBLE_SHIFT_SETTLE_MS,
      },);

      /**
       The state back on the Settings list.
       */
      const back = await press({
        session,
        act: tapKey('escape',),
        until: routeIs('Settings',),
        what: 'Escape to the Settings list',
      },);

      /**
       The state after leaving Settings.
       */
      const left = await press({
        session,
        act: tapKey('escape',),
        until: isPlayerPage,
        what: 'Escape leaving Settings',
      },);
      return `${editor.route} (focus ${editor.focus}) > ${back.route} (focus ${back.focus}) > ${left.route}`;
    },
  },);
  await recorder.verify({
    id: 'editor-page/ctrl-enter',
    title: 'Ctrl+Enter applies the template and returns to the Settings list',
    body: async function ctrlEnterAppliesTemplate(): Promise<void> {
      await press({
        session,
        act: chordKeys({
          mods: [
            'ctrl',
            'alt',
          ],
          key: 's',
        },),
        until: routeIs('Settings',),
        what: 'Settings',
      },);
      await press({
        session,
        act: tapKey('enter',),
        until: routeIs('TemplateEditor',),
        what: 'the editor page',
      },);
      await press({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'enter',
        },),
        until: routeIs('Settings',),
        what: 'the Settings list',
      },);
      await press({
        session,
        act: tapKey('escape',),
        until: isPlayerPage,
        what: 'the player page',
      },);
    },
  },);
  await checkNarrowClickThrough({
    context,
    session,
  },);
}
