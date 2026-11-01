/**
 Checks of the wide session about browsing: the folder chords (Alt+Down, Alt+Up, Ctrl+Shift+G, Ctrl+G, Ctrl+O), and the
 keys of the folder list.
 
 @example
 ```ts
 import { checkBrowsing, } from './wide-browse.ts';
 
 await checkBrowsing({ context, session, },);
 ```
 
 @packageDocumentation
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  anyState,
  chordKeys,
  focusIs,
  tapKey,
} from './actions.ts';
import {
  hasFolderRing,
  ringStartsWithZ,
} from './conditions.ts';
import type { RunContext, } from './context.ts';
import { expect, } from './errors.ts';
import { NO_EFFECT_SETTLE_MS, } from './timing.ts';
import { press, } from './press.ts';
import type { Session, } from './session.ts';
import type { UiState, } from './ui-state.ts';

/**
 Records the browse checks of the wide session.
 
 @param context - the run's context
 
 @param session - the wide session, with a track playing in the first folder
 
 @example
 ```ts
 await checkBrowsing({ context, session, },);
 ```
 */
export async function checkBrowsing(
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
  recorder.setGroup('browse',);

  /**
   The folder that holds the playing track.
   */
  const firstFolder = (await session.last()).folder;

  /**
   Condition: the shown folder is not the first one.
   
   @param state - a state of the app
   
   @returns whether another folder shows
   */
  function isOtherFolder(state: UiState,): boolean {
    return state.folder !== firstFolder;
  }
  /**
   Condition: the shown folder is the first one.
   
   @param state - a state of the app
   
   @returns whether the first folder shows
   */
  function isFirstFolder(state: UiState,): boolean {
    return state.folder === firstFolder;
  }
  await recorder.check({
    id: 'browse/alt-down-up',
    title: 'Alt+Down and Alt+Up show the next and previous folder',
    body: async function altArrowsShowNeighbourFolders(): Promise<string> {
      /**
       The state after Alt+Down.
       */
      const down = await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: 'down',
        },),
        until: isOtherFolder,
        what: 'the next folder',
      },);

      /**
       The state after the second Alt+Down.
       */
      const down2 = await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: 'down',
        },),
        until: function isFolderAfter(state: UiState,): boolean {
          return state.folder !== down.folder;
        },
        what: 'the folder after',
      },);

      /**
       The state after Alt+Up.
       */
      const up = await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: 'up',
        },),
        until: function isFolderBefore(state: UiState,): boolean {
          return state.folder === down.folder;
        },
        what: 'back one folder',
      },);
      return `${firstFolder} > ${down.folder} > ${down2.folder} > ${up.folder}`;
    },
  },);
  await recorder.check({
    id: 'browse/ctrl-shift-g',
    title: 'Ctrl+Shift+G shows the folder of the playing track',
    body: async function ctrlShiftGShowsPlayingFolder(): Promise<string> {
      /**
       The state showing the playing folder.
       */
      const state = await press({
        session,
        act: chordKeys({
          mods: [
            'ctrl',
            'shift',
          ],
          key: 'g',
        },),
        until: isFirstFolder,
        what: `folder "${firstFolder}"`,
      },);
      return `folder "${state.folder}"`;
    },
  },);
  await recorder.verify({
    id: 'browse/ctrl-g',
    title: 'Ctrl+G shows the playing track',
    body: async function ctrlGRevealsPlayingTrack(): Promise<void> {
      await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: 'down',
        },),
        until: isOtherFolder,
        what: 'another folder',
      },);

      /**
       Reveal uses before the chord.
       */
      const before = await session.used('reveal',);
      await session.chord({
        mods: ['ctrl',],
        key: 'g',
      },);
      await wait(NO_EFFECT_SETTLE_MS,);
      expect({
        condition: await session.used('reveal',) === (before
          + 1),
        message: 'Ctrl+G was not used as Reveal',
      },);
      await press({
        session,
        act: chordKeys({
          mods: [
            'ctrl',
            'shift',
          ],
          key: 'g',
        },),
        until: isFirstFolder,
        what: 'back to the playing folder',
      },);
    },
  },);
  await recorder.verify({
    id: 'browse/ctrl-o',
    title: 'Ctrl+O puts the keyboard in the folder list (two-pane layout)',
    body: async function ctrlOFocusesFolderList(): Promise<void> {
      await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: '2',
        },),
        until: focusIs('track-list',),
        what: 'the track list',
      },);
      await press({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'o',
        },),
        until: focusIs('folder-list',),
        what: 'the folder list',
      },);
    },
  },);
}

/**
 Records the folder-list checks of the wide session.
 
 @param context - the run's context
 
 @param session - the wide session, with the keyboard able to reach the folder list
 
 @example
 ```ts
 await checkFolderList({ context, session, },);
 ```
 */
export async function checkFolderList(
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
  recorder.setGroup('folder-list',);
  await press({
    session,
    act: chordKeys({
      mods: ['alt',],
      key: '1',
    },),
    until: focusIs('folder-list',),
    what: 'the folder list',
  },);
  await recorder.check({
    id: 'folder-list/ring-start',
    title: 'the ring starts on the current folder',
    body: async function ringStartsOnCurrentFolder(): Promise<string> {
      /**
       The state with the folder list focused.
       */
      const state = await session.last();
      expect({
        condition: state.ring_folder === state.folder,
        message: `ring on ${state.ring_folder}, folder ${state.folder}`,
      },);
      return `ring on "${state.ring_folder}"`;
    },
  },);
  await recorder.check({
    id: 'folder-list/down-up-home-end',
    title: 'Down, Up, Home and End move the folder ring',
    body: async function arrowsMoveFolderRing(): Promise<string> {
      /**
       The state after Home.
       */
      const home = await press({
        session,
        act: tapKey('home',),
        until: hasFolderRing,
        what: 'Home',
      },);

      /**
       The state after Down.
       */
      const down = await press({
        session,
        act: tapKey('down',),
        until: function ringLeftHome(state: UiState,): boolean {
          return state.ring_folder !== home.ring_folder;
        },
        what: 'Down',
      },);

      /**
       The state after Up.
       */
      const up = await press({
        session,
        act: tapKey('up',),
        until: function ringBackOnHome(state: UiState,): boolean {
          return state.ring_folder === home.ring_folder;
        },
        what: 'Up',
      },);

      /**
       The state after End.
       */
      const end = await press({
        session,
        act: tapKey('end',),
        until: function ringLeftHomeAgain(state: UiState,): boolean {
          return state.ring_folder !== home.ring_folder;
        },
        what: 'End',
      },);
      return `Home ${home.ring_folder}, Down ${down.ring_folder}, Up ${up.ring_folder}, End ${end.ring_folder}`;
    },
  },);
  await recorder.check({
    id: 'folder-list/page-keys',
    title: 'PageUp and PageDown move the folder ring to the ends of a short list',
    body: async function pageKeysMoveToEnds(): Promise<string> {
      await press({
        session,
        act: tapKey('pageup',),
        until: anyState,
        what: 'PageUp',
      },);

      /**
       The folder under the ring after PageUp.
       */
      const top = (await session.last()).ring_folder;

      /**
       The state after PageDown.
       */
      const bottom = await press({
        session,
        act: tapKey('pagedown',),
        until: function ringLeftTop(state: UiState,): boolean {
          return state.ring_folder !== top;
        },
        what: 'PageDown',
      },);
      return `PageUp ${top}, PageDown ${bottom.ring_folder}`;
    },
  },);
  await recorder.check({
    id: 'folder-list/letter',
    title: 'a letter jumps to the first folder starting with it',
    body: async function letterJumpsToFolder(): Promise<string> {
      /**
       The state with the ring on a folder starting with Z.
       */
      const state = await press({
        session,
        act: tapKey('z',),
        until: ringStartsWithZ,
        what: 'a folder starting with Z',
      },);
      return `ring on "${state.ring_folder}"`;
    },
  },);
  await recorder.check({
    id: 'folder-list/left-right',
    title: 'Left and Right also move the folder ring',
    body: async function leftRightMoveFolderRing(): Promise<string> {
      /**
       The folder under the ring before the arrows.
       */
      const before = (await session.last()).ring_folder;

      /**
       The state after Left.
       */
      const left = await press({
        session,
        act: tapKey('left',),
        until: function ringLeftBefore(state: UiState,): boolean {
          return state.ring_folder !== before;
        },
        what: 'Left',
      },);

      /**
       The state after Right.
       */
      const right = await press({
        session,
        act: tapKey('right',),
        until: function ringBackOnBefore(state: UiState,): boolean {
          return state.ring_folder === before;
        },
        what: 'Right',
      },);
      return `${before} > ${left.ring_folder} > ${right.ring_folder}`;
    },
  },);
  await recorder.check({
    id: 'folder-list/enter',
    title: 'Enter shows the folder under the ring',
    body: async function enterShowsFolder(): Promise<string> {
      /**
       The folder under the ring.
       */
      const ring = (await session.last()).ring_folder;

      /**
       The state showing that folder.
       */
      const state = await press({
        session,
        act: tapKey('enter',),
        until: function isRingFolder(candidate: UiState,): boolean {
          return candidate.folder === ring;
        },
        what: `folder "${ring}"`,
      },);
      expect({
        condition: state.focus === 'folder-list',
        message: `focus left the folder list: ${state.focus}`,
      },);
      return `folder "${state.folder}"`;
    },
  },);
}
