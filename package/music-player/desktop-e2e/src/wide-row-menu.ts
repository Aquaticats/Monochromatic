/**
 Checks of the wide session about the track menu and Delete.
 
 @example
 ```ts
 import { checkMenuAndDelete, } from './wide-row-menu.ts';
 
 await checkMenuAndDelete({ context, session, },);
 ```
 
 @packageDocumentation
 */

import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  chordKeys,
  focusIs,
  tapKey,
} from './actions.ts';
import {
  hasTrackRing,
  isMenuClosed,
  isMenuOpen,
  menuHasRing,
} from './conditions.ts';
import type { RunContext, } from './context.ts';
import { expect, } from './errors.ts';
import { pathExists, } from './files.ts';
import { WIDE_POINTS, } from './layout.ts';
import { pollUntil, } from './poll.ts';
import {
  closeMenuIfOpen,
  press,
} from './press.ts';
import type { Session, } from './session.ts';
import {
  indexText,
  type UiState,
} from './ui-state.ts';

/**
 How many times the file is looked for after Delete.
 */
const DELETE_LOOKS = 25;

/**
 Pause between two looks for the deleted file.
 */
const DELETE_PAUSE_MS = 200;

/**
 Wait before the screenshot that follows Delete, so the undo message is drawn.
 */
const AFTER_DELETE_MS = 400;

/**
 How long the track menu gets to open after the right click.
 */
const MENU_OPEN_TIMEOUT_MS = 4_000;

/**
 Records the track-menu and Delete checks.
 
 @param context - the run's context
 
 @param session - the wide session, with the ring on the file that Delete sends to the trash
 
 @example
 ```ts
 await checkMenuAndDelete({ context, session, },);
 ```
 */
export async function checkMenuAndDelete(
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

  /**
   The file the Delete check sends to the trash.
   */
  const trashFile = join(
    context.library,
    'Zzz Tones',
    '99 Trash me.wav',
  );
  await recorder.check({
    id: 'row/menu-keys',
    title: 'in the open track menu Down, Up, Enter and Escape work (menu opened by right click)',
    body: async function menuKeysWork(): Promise<string> {
      await closeMenuIfOpen({ session, },);

      /**
       Sequence number of the latest state before the right click, so an earlier menu's state never counts as this one.
       */
      const beforeClick = await session.seq();
      await session.click({
        ...WIDE_POINTS.trackRow,
        button: 'right',
      },);

      /**
       The state with the menu open.
       */
      const open = await session.waitState({
        predicate: isMenuOpen,
        what: 'the track menu opening',
        after: beforeClick,
        timeoutMs: MENU_OPEN_TIMEOUT_MS,
      },);

      /**
       The state after the first Down.
       */
      const down = await press({
        session,
        act: tapKey('down',),
        until: menuHasRing,
        what: 'Down in the menu',
      },);

      /**
       The state after the second Down.
       */
      const down2 = await press({
        session,
        act: tapKey('down',),
        until: function menuRingMovedDown(state: UiState,): boolean {
          return state.menu_ring === (down.menu_ring
            + 1);
        },
        what: 'Down again',
      },);

      /**
       The state after Up.
       */
      const up = await press({
        session,
        act: tapKey('up',),
        until: function menuRingBackUp(state: UiState,): boolean {
          return state.menu_ring === down.menu_ring;
        },
        what: 'Up',
      },);

      /**
       The state after Escape.
       */
      const closed = await press({
        session,
        act: tapKey('escape',),
        until: isMenuClosed,
        what: 'Escape closing the menu',
      },);
      expect({
        condition: closed.route === 'Player',
        message: `Escape left the page: ${closed.route}`,
      },);
      await session.click({
        ...WIDE_POINTS.trackRow,
        button: 'right',
      },);
      await session.waitState({
        predicate: isMenuOpen,
        what: 'the menu reopening',
        after: closed.seq,
        timeoutMs: MENU_OPEN_TIMEOUT_MS,
      },);
      await press({
        session,
        act: tapKey('down',),
        until: menuHasRing,
        what: 'Down',
      },);

      /**
       The row under the ring when Enter runs it.
       */
      const first = (await session.last()).menu_ring;

      /**
       The state after Enter closed the menu.
       */
      const chosen = await press({
        session,
        act: tapKey('enter',),
        until: isMenuClosed,
        what: 'Enter running a row',
      },);

      /**
       The ring positions seen while the menu was open, as text.
       */
      const rings = [
        open,
        down,
        down2,
        up,
      ]
        .map(function ringText(state: UiState,): string {
          return indexText(state.menu_ring,);
        },)
        .join(' > ',);
      return `menu_ring ${rings}; Enter ran row ${indexText(first,)}; closed (index ${indexText(chosen.current_index,)})`;
    },
  },);
  await recorder.check({
    id: 'row/delete',
    title: 'Delete moves the file of the row to the trash (scratch XDG trash), with an Undo message',
    body: async function deleteMovesFileToTrash(): Promise<string> {
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
        act: tapKey('end',),
        until: hasTrackRing,
        what: 'End',
      },);

      /**
       Title of the row under the ring.
       */
      const ringTitle = (await session.last()).ring_track_title;
      expect({
        condition: ringTitle.includes('Trash me',),
        message: `the ring is on ${ringTitle}`,
      },);
      expect({
        condition: await pathExists(trashFile,),
        message: 'the file to trash is missing before the key',
      },);
      await session.tap('delete',);

      /**
       Whether the file left the library.
       */
      const gone = await pollUntil({
        attempts: DELETE_LOOKS,
        intervalMs: DELETE_PAUSE_MS,
        order: 'wait-then-check',
        check: async function fileIsGone(): Promise<boolean> {
          return !(await pathExists(trashFile,));
        },
      },);
      expect({
        condition: gone,
        message: 'the file is still in the library after Delete',
      },);

      /**
       Where the file lands in the scratch trash.
       */
      const trashed = join(
        session.xdg,
        'data',
        'Trash',
        'files',
        '99 Trash me.wav',
      );
      expect({
        condition: await pathExists(trashed,),
        message: `the file is not in the scratch trash at ${trashed}`,
      },);
      await wait(AFTER_DELETE_MS,);
      await session.shot('after-delete',);
      return `moved to ${trashed}`;
    },
  },);
}
