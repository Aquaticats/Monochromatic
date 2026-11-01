/**
 Checks of the wide session about launch and the focus model: Tab order, the focus ring, and the two list shortcuts.
 
 @example
 ```ts
 import { checkLaunchAndFocus, } from './wide-focus.ts';
 
 await checkLaunchAndFocus({ context, session, },);
 ```
 
 @packageDocumentation
 */

import {
  chordKeys,
  clickAt,
  focusIs,
  isRingHidden,
  isRingVisible,
  tapKey,
} from './actions.ts';
import type { RunContext, } from './context.ts';
import { expect, } from './errors.ts';
import {
  diffBox,
  loadImage,
} from './image.ts';
import {
  UNFOLDED_ORDER,
  WIDE_POINTS,
} from './layout.ts';
import {
  press,
  tabSequence,
  tabTo,
} from './press.ts';
import type { Session, } from './session.ts';
import { NO_INDEX, } from './ui-state.ts';

/**
 Fewest differing pixels that prove a focus ring moved.
 */
const MIN_RING_PIXELS = 40;

/**
 Records the launch check and the focus-model checks of the wide session.
 
 @param context - the run's context
 
 @param session - the wide session, ready and with its app found
 
 @example
 ```ts
 await checkLaunchAndFocus({ context, session, },);
 ```
 */
export async function checkLaunchAndFocus(
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
  recorder.setGroup('start',);
  await recorder.check({
    id: 'start/page',
    title: 'launch shows the player page, nothing playing, no ring',
    body: async function launchShowsPlayerPage(): Promise<string> {
      /**
       The state at launch.
       */
      const state = await session.last();
      expect({
        condition: state.route === 'Player',
        message: `route ${state.route}`,
      },);
      expect({
        condition: (state.current_index === NO_INDEX) && (!state.playing),
        message: 'something plays at launch',
      },);
      expect({
        condition: !state.ring_visible,
        message: 'a ring shows before any key',
      },);
      expect({
        condition: session.appPid() !== '',
        message: 'the app process was not found in its own unit',
      },);
      return `folder "${state.folder}", app pid ${session.appPid()} in unit ${session.unit}`;
    },
  },);

  recorder.setGroup('focus',);
  await recorder.check({
    id: 'focus/tab-order',
    title: 'Tab walks folder picker, track list, deck controls, search, settings, then wraps',
    body: async function tabWalksTheOrder(): Promise<string> {
      return await tabSequence({
        session,
        expected: [
          ...UNFOLDED_ORDER,
          'open-pill',
          'folder-list',
          'track-list',
        ],
        key: tapKey('tab',),
      },);
    },
  },);
  await recorder.check({
    id: 'focus/shift-tab',
    title: 'Shift+Tab walks the same order backwards, no trap',
    body: async function shiftTabWalksBackwards(): Promise<string> {
      return await tabSequence({
        session,
        expected: [
          'folder-list',
          'open-pill',
          'settings',
          'search',
          'mode-group',
        ],
        key: chordKeys({
          mods: ['shift',],
          key: 'tab',
        },),
      },);
    },
  },);
  await recorder.verify({
    id: 'focus/ring-visible',
    title: 'a key press turns the ring on',
    body: async function keyTurnsRingOn(): Promise<void> {
      expect({
        condition: (await session.last()).ring_visible,
        message: 'the ring is off after Tab',
      },);
    },
  },);
  await recorder.check({
    id: 'focus/ring-moves',
    title: 'the focus ring (2 px primary and 1 px on-primary) moves with Tab: pixels differ between two focus positions',
    body: async function ringMovesWithTab(): Promise<string> {
      await tabTo({
        session,
        label: 'open-pill',
      },);

      /**
       Screenshot with the ring on the folder picker.
       */
      const first = await session.shot('ring-open-pill',);
      await press({
        session,
        act: tapKey('tab',),
        until: focusIs('folder-list',),
        what: 'focus on the folder list',
      },);

      /**
       Screenshot with the ring on the folder list.
       */
      const second = await session.shot('ring-folder-list',);

      /**
       How the two screenshots differ.
       */
      const delta = diffBox({
        a: await loadImage(first,),
        b: await loadImage(second,),
      },);
      expect({
        condition: (delta.box !== undefined) && (delta.count > MIN_RING_PIXELS),
        message: `the two focus positions look the same (${delta.count} differing pixels)`,
      },);
      return `${delta.count} pixels differ, in ${JSON.stringify(delta.box,)}`;
    },
  },);
  await recorder.verify({
    id: 'focus/alt-1-2',
    title: 'Alt+1 and Alt+2 focus the folder list and the track list',
    body: async function altDigitsFocusLists(): Promise<void> {
      await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: '2',
        },),
        until: focusIs('track-list',),
        what: 'Alt+2 to the track list',
      },);
      await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: '1',
        },),
        until: focusIs('folder-list',),
        what: 'Alt+1 to the folder list',
      },);
    },
  },);
  await recorder.verify({
    id: 'focus/pointer-hides-ring',
    title: 'a pointer press turns the ring off',
    body: async function pointerHidesRing(): Promise<void> {
      await press({
        session,
        act: clickAt(WIDE_POINTS.folderRow,),
        until: isRingHidden,
        what: 'the ring turning off after a click on a folder',
      },);
      await press({
        session,
        act: tapKey('tab',),
        until: isRingVisible,
        what: 'the ring coming back on a key press',
      },);
    },
  },);
}
