/**
 Checks of the wide session about the row chords that copy and show in the file manager, and the menu keys (Alt+Enter and the access letters).
 
 @example
 ```ts
 import { checkRowChords, } from './wide-rows.ts';
 
 await checkRowChords({ context, session, fileManagerRecord, },);
 ```
 
 @packageDocumentation
 */

import { readFile, } from 'node:fs/promises';

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  chordKeys,
  tapKey,
} from './actions.ts';
import { readClipboard, } from './clipboard.ts';
import {
  isMenuClosed,
  ringIsOnTrashFile,
} from './conditions.ts';
import type { RunContext, } from './context.ts';
import { expect, } from './errors.ts';
import { pollUntil, } from './poll.ts';
import {
  closeMenuIfOpen,
  noEffect,
  nothingMoved,
  press,
} from './press.ts';
import type { Session, } from './session.ts';
import { EFFECT_SETTLE_MS, } from './timing.ts';
import {
  indexText,
  type UiState,
} from './ui-state.ts';

/**
 How many times the file manager stub's record is read.
 */
const RECORD_LOOKS = 20;

/**
 Pause between two reads of the record.
 */
const RECORD_PAUSE_MS = 200;

/**
 Condition: the track menu is open with its first row highlighted, as the keyboard opens it.
 
 @param state - a state of the app
 
 @returns whether the menu is open and its ring is on the first row
 
 @example
 ```ts
 menuHasFirstRow(state,);
 ```
 */
function menuHasFirstRow(state: UiState,): boolean {
  return state.menu_open && (state.menu_ring === 0);
}

/**
 Records the copy, file-manager and menu-key checks of the wide session, with the ring in the last folder.
 
 @param context - the run's context
 
 @param session - the wide session, with the track list focused in the last folder
 
 @param fileManagerRecord - file in which the stub that stands in for the file manager appends its arguments
 
 @example
 ```ts
 await checkRowChords({ context, session, fileManagerRecord, },);
 ```
 */
export async function checkRowChords(
  {
    context,
    session,
    fileManagerRecord,
  }: {
    readonly context: RunContext;
    readonly fileManagerRecord: string;
    readonly session: Session;
  },
): Promise<void> {
  /**
   The recorder of the run.
   */
  const { recorder, } = context;
  recorder.setGroup('row',);
  await press({
    session,
    act: tapKey('down',),
    until: ringIsOnTrashFile,
    what: 'the ring on the file to trash',
  },);
  await recorder.check({
    id: 'row/ctrl-shift-c',
    title: 'Ctrl+Shift+C copies the file name of the row under the ring to the clipboard',
    body: async function ctrlShiftCCopiesFileName(): Promise<string> {
      await session.chord({
        mods: [
          'ctrl',
          'shift',
        ],
        key: 'c',
      },);
      await wait(EFFECT_SETTLE_MS,);

      /**
       Name of the nested Wayland display, whose clipboard is read.
       */
      const display = await session.waylandDisplay();
      expect({
        condition: display !== '',
        message: 'the nested Wayland display name is unknown',
      },);

      /**
       Text on that display's clipboard.
       */
      const text = await readClipboard(display,);
      expect({
        condition: text.includes('99 Trash me',),
        message: `the clipboard holds ${JSON.stringify(text,)}`,
      },);
      return `clipboard of ${display}: ${JSON.stringify(text,)}`;
    },
  },);
  await recorder.check({
    id: 'row/ctrl-shift-e',
    title: 'Ctrl+Shift+E opens the file manager on the folder of the row',
    body: async function ctrlShiftEOpensFileManager(): Promise<string> {
      await session.chord({
        mods: [
          'ctrl',
          'shift',
        ],
        key: 'e',
      },);

      /**
       What the stub recorded, once it recorded anything.
       */
      const seen = { recorded: '', };
      await pollUntil({
        attempts: RECORD_LOOKS,
        intervalMs: RECORD_PAUSE_MS,
        order: 'wait-then-check',
        check: async function stubRecorded(): Promise<boolean> {
          seen.recorded = await readFile(
            fileManagerRecord,
            'utf8',
          );
          return seen.recorded !== '';
        },
      },);
      expect({
        condition: seen.recorded
          .includes('Zzz Tones',),
        message: `the file manager stub was asked for ${JSON.stringify(seen.recorded,)}`,
      },);
      return `xdg-open ${seen.recorded
        .trim()}`;
    },
  },);
  await recorder.check({
    id: 'row/shift-f10-unbound',
    title: 'Shift+F10 opens nothing: it is Run in JetBrains and is not used (D114)',
    body: async function shiftF10IsUnbound(): Promise<string> {
      await noEffect({
        session,
        act: chordKeys({
          mods: ['shift',],
          key: 'f10',
        },),
        watch: nothingMoved,
        what: 'Shift+F10 on a track row',
      },);
      return 'no menu, no page change, no shortcut use';
    },
  },);
  await recorder.check({
    id: 'row/alt-enter',
    title: 'Alt+Enter opens the row menu with its first row highlighted (D114)',
    body: async function altEnterOpensMenu(): Promise<string> {
      /**
       The state with the menu open and the first row highlighted.
       */
      const state = await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: 'enter',
        },),
        until: menuHasFirstRow,
        what: 'the menu with its first row highlighted',
      },);
      await press({
        session,
        act: tapKey('escape',),
        until: isMenuClosed,
        what: 'the menu closing',
      },);
      return `menu_open ${state.menu_open}, menu_ring ${indexText(state.menu_ring,)}, ${state.menu_width} px wide`;
    },
  },);
  await recorder.check({
    id: 'row/menu-key',
    title: 'the Menu key opens the row menu with its first row highlighted (D114)',
    body: async function menuKeyOpensMenu(): Promise<string> {
      await closeMenuIfOpen({ session, },);

      /**
       The state with the menu open and the first row highlighted.
       */
      const state = await press({
        session,
        act: tapKey('menu',),
        until: menuHasFirstRow,
        what: 'the menu with its first row highlighted',
      },);

      // Leave the menu closed, so the checks after this one start from the same state in any run order.
      await press({
        session,
        act: tapKey('escape',),
        until: isMenuClosed,
        what: 'Escape closing the menu the Menu key opened',
      },);
      return `menu_open ${state.menu_open}, menu_ring ${indexText(state.menu_ring,)}, ${state.menu_width} px wide`;
    },
  },);
  await recorder.check({
    id: 'row/menu-access-key',
    title: 'the access letter D of File details runs that row and closes the menu',
    body: async function accessLetterRunsRow(): Promise<string> {
      await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: 'enter',
        },),
        until: menuHasFirstRow,
        what: 'the menu opening',
      },);

      /**
       How often an access letter ran a row before this one.
       */
      const used = await session.used('menu-access',);
      await session.shot('before-access-key',);
      await press({
        session,
        act: tapKey('d',),
        until: isMenuClosed,
        what: 'the menu closing',
      },);
      await wait(EFFECT_SETTLE_MS,);
      expect({
        condition: await session.used('menu-access',) === (used
          + 1),
        message: 'the letter D was not used as an access letter',
      },);

      /**
       Screenshot with the notice that File details shows.
       */
      const after = await session.shot('access-key-notice',);
      return `the menu closed after one use of an access letter; ${after}`;
    },
  },);
}
