/**
 Checks of the wide session about the two lists of the player page: the folder picker scrolls to the folder a Search result
 selected (it stayed where it was and showed no selection), and the track list keeps its scroll position when the window
 narrows across the 600 px breakpoint and widens again (it jumped back to the first row).

 Both checks read the scroll offsets the app writes into its `ui_state` lines. A state line is written when something
 changes in the window, and a scroll settles a frame after the change that caused it, so each check waits a moment and
 then provokes one more line (holding Control shows the key hint badges, which writes a line and changes nothing else).

 @example
 ```ts
 import { checkLists, } from './wide-lists.ts';

 await checkLists({ context, session, },);
 ```

 @packageDocumentation
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  anyState,
  chordKeys,
  focusIs,
  holdKey,
  releaseKey,
  tapKey,
} from './actions.ts';
import {
  hasResultRing,
  hasTrackRing,
  isLastFolder,
  isSearchWithField,
  queryHasTone,
  resultOpened,
  ringStartsWithZ,
} from './conditions.ts';
import type { RunContext, } from './context.ts';
import {
  CheckFailureError,
  expect,
} from './errors.ts';
import {
  NARROW_WINDOW,
  WIDE_WINDOW,
} from './layout.ts';
import { press, } from './press.ts';
import type { Session, } from './session.ts';
import type { UiState, } from './ui-state.ts';

/**
 Time a scroll needs to settle after the change that asked for it: a frame for the list to react, and the 180 ms of the
 wheel and reveal animation.
 */
const SCROLL_SETTLE_MS = 600;

/**
 Smallest scroll offset, in pixels, that proves a list really scrolled (about six rows of the track list).
 */
const MIN_SCROLL_PX = 300;

/**
 Smallest picker offset, in pixels, that proves the folder picker scrolled (the picker is short in the short window, so its
 whole range is only a few hundred pixels).
 */
const MIN_PICKER_SCROLL_PX = 100;

/**
 Most folders the check visits while it looks for one whose track list scrolls.
 */
const MAX_FOLDERS_TRIED = 8;

/**
 Pixels the scroll offsets may differ after a layout change: the offset is carried over exactly, and a rounding step of the
 log line is the only difference.
 */
const SCROLL_TOLERANCE_PX = 1;

/**
 Time the analysis needs to give the rows of a freshly shown folder their lengths and peaks, which changes their heights.
 */
const MEASURES_SETTLE_MS = 3_000;

/**
 Height of the short window in which the folder picker has to scroll to show every folder.
 */
const SHORT_WINDOW_HEIGHT = 400;

/**
 How long a Search result takes to open and the picker to follow.
 */
const RESULT_TIMEOUT_MS = 8_000;

/**
 Condition: the folder list has a ring on some folder.
 
 @param state - a state of the app
 
 @returns whether a folder has the ring
 
 @example
 ```ts
 ringIsOnAFolder(state,);
 ```
 */
function ringIsOnAFolder(state: UiState,): boolean {
  return state.ring_folder !== '';
}

/**
 Condition: the track list shows some folder.
 
 @param state - a state of the app
 
 @returns whether a folder is shown
 
 @example
 ```ts
 aFolderIsShown(state,);
 ```
 */
function aFolderIsShown(state: UiState,): boolean {
  return state.folder !== '';
}

/**
 Condition: the window is on the folded layout.
 
 @param state - a state of the app
 
 @returns whether the layout is folded
 
 @example
 ```ts
 isFolded(state,);
 ```
 */
function isFolded(state: UiState,): boolean {
  return !state.unfolded;
}

/**
 Condition: the window is on the two-pane layout.
 
 @param state - a state of the app
 
 @returns whether the layout is unfolded
 
 @example
 ```ts
 isUnfolded(state,);
 ```
 */
function isUnfolded(state: UiState,): boolean {
  return state.unfolded;
}

/**
 Waits for a scroll to settle and reads a state written after it.

 @param session - the wide session

 @returns a state line written after the scroll settled

 @example
 ```ts
 const state = await settledState(session,);
 ```
 */
async function settledState(session: Session,): Promise<UiState> {
  await wait(SCROLL_SETTLE_MS,);
  /**
   The line written when Control goes down and the key hint badges appear.
   */
  const state = await press({
    session,
    act: holdKey('ctrl',),
    until: anyState,
    what: 'a state line after the scroll settled',
  },);
  await press({
    session,
    act: releaseKey('ctrl',),
    until: anyState,
    what: 'Control released',
  },);
  return state;
}

/**
 Shows the folder under the folder list's ring, focuses its track list, moves to the last row, and reads the state.
 
 @param session - the wide session, with the folder list focused and a ring on a folder
 
 @returns the state with the track list scrolled to its last row, if the list scrolls
 
 @example
 ```ts
 const atEnd = await openFolderAndScrollToEnd(session,);
 ```
 */
async function openFolderAndScrollToEnd(session: Session,): Promise<UiState> {
  await press({
    session,
    act: tapKey('enter',),
    until: aFolderIsShown,
    what: 'a folder',
  },);
  // A row grows from one line to two when its length and peak arrive, which moves everything below it, so the list is
  // scrolled only after the folder's rows have their final heights.
  await wait(MEASURES_SETTLE_MS,);
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
    what: 'the last track',
  },);
  /**
   The state with the ring on the last row.
   */
  const last = await settledState(session,);
  // Page Up moves the ring up a page and scrolls the list so the ring is at the top of the pane, which keeps the ring in
  // view in the folded layout too (a ring left at the bottom of the tall pane would be scrolled again in the short one).
  await press({
    session,
    act: tapKey('pageup',),
    until: function ringMovedUp(state: UiState,): boolean {
      return state.ring_track !== last.ring_track;
    },
    what: 'Page Up',
  },);
  return await settledState(session,);
}

/**
 Moves the folder list's ring to the next folder, with the folder list focused.
 
 @param session - the wide session, on the player page
 
 @param from - the folder the ring is on now
 
 @example
 ```ts
 await focusNextFolder({ session, from: 'Aaa Tones', },);
 ```
 */
async function focusNextFolder(
  {
    session,
    from,
  }: {
    readonly from: string;
    readonly session: Session;
  },
): Promise<void> {
  await press({
    session,
    act: chordKeys({
      mods: ['alt',],
      key: '1',
    },),
    until: focusIs('folder-list',),
    what: 'the folder list',
  },);
  await press({
    session,
    act: tapKey('down',),
    until: function ringLeft(state: UiState,): boolean {
      return state.ring_folder !== from;
    },
    what: 'the next folder',
  },);
}

/**
 Moves the track list to a folder whose list scrolls, and scrolls it down.
 
 @param session - the wide session, with the track list or the folder list focused
 
 @returns the state with the track list scrolled down
 
 @throws CheckFailureError when none of the first folders has a list long enough to scroll
 
 @example
 ```ts
 const scrolled = await scrollTrackListDown(session,);
 ```
 */
async function scrollTrackListDown(session: Session,): Promise<UiState> {
  await press({
    session,
    act: chordKeys({
      mods: ['alt',],
      key: '1',
    },),
    until: focusIs('folder-list',),
    what: 'the folder list',
  },);
  await press({
    session,
    act: tapKey('home',),
    until: ringIsOnAFolder,
    what: 'the first folder',
  },);
  for (let tried = 0; tried < MAX_FOLDERS_TRIED; tried += 1) {
    /**
     The state with this folder's track list moved to its last row.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- each folder is tried only after the previous one proved too short
    const atEnd = await openFolderAndScrollToEnd(session,);
    if (atEnd.track_scroll >= MIN_SCROLL_PX) {
      return atEnd;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- the ring moves on from the folder that was just tried
    await focusNextFolder({
      session,
      from: atEnd.ring_folder,
    },);
  }
  throw new CheckFailureError(`none of the first ${MAX_FOLDERS_TRIED} folders has a track list that scrolls`,);
}

/**
 Records the checks of the two lists in the wide session.

 @param context - the run's context

 @param session - the wide session, on the player page

 @example
 ```ts
 await checkLists({ context, session, },);
 ```
 */
export async function checkLists(
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
  recorder.setGroup('lists',);
  await recorder.check({
    id: 'lists/picker-follows-search-result',
    title: 'after a Search result opens a folder far from the picker position, the folder picker scrolls to it',
    body: async function pickerFollowsSearchResult(): Promise<string> {
      // A short window makes the picker too short for the library's folders, so it has a position to follow.
      await session.ok(`resize ${WIDE_WINDOW.width} ${SHORT_WINDOW_HEIGHT}`,);
      await wait(SCROLL_SETTLE_MS,);
      await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: '1',
        },),
        until: focusIs('folder-list',),
        what: 'the folder list',
      },);
      await press({
        session,
        act: tapKey('z',),
        until: ringStartsWithZ,
        what: 'the Z folder',
      },);
      await press({
        session,
        act: tapKey('enter',),
        until: isLastFolder,
        what: 'the last folder',
      },);

      /**
       The state with the picker scrolled to the last folder.
       */
      const atBottom = await settledState(session,);
      expect({
        condition: atBottom.folder_scroll >= MIN_PICKER_SCROLL_PX,
        message: `the picker is scrolled to the last folder (offset ${atBottom.folder_scroll}), so a result in another folder moves it`,
      },);
      await press({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'f',
        },),
        until: isSearchWithField,
        what: 'Search',
      },);
      // Typing in the field writes no state line of its own and nothing plays, so the line that shows the results is the one
      // that the probe provokes after the answer had time to arrive.
      await session.type('tone',);
      /**
       The state after the results arrived.
       */
      const searched = await settledState(session,);
      expect({
        condition: queryHasTone(searched) && hasResultRing(searched),
        message: `Search shows results for "tone" (query "${searched.search_query}", ring ${searched.search_ring})`,
      },);
      await press({
        session,
        act: tapKey('enter',),
        until: resultOpened,
        what: 'the result opened',
      },);

      /**
       The state after the picker had time to follow the opened result's folder.
       */
      const after = await settledState(session,);
      expect({
        condition: after.folder !== atBottom.folder,
        message: `the result opened a folder other than ${atBottom.folder}`,
      },);
      expect({
        condition: after.folder_scroll < (atBottom.folder_scroll - MIN_PICKER_SCROLL_PX),
        message: `the picker stayed at offset ${after.folder_scroll} (it was ${atBottom.folder_scroll}) although ${after.folder} is selected`,
      },);
      await session.ok(`resize ${WIDE_WINDOW.width} ${WIDE_WINDOW.height}`,);
      await wait(SCROLL_SETTLE_MS,);
      return `${atBottom.folder} at offset ${atBottom.folder_scroll}, then ${after.folder} at offset ${after.folder_scroll}`;
    },
  },);
  await recorder.check({
    id: 'lists/track-list-keeps-place-across-breakpoint',
    title: 'the track list keeps its scroll position when the window narrows below 600 px and widens again',
    body: async function listKeepsPlace(): Promise<string> {
      /**
       The state with the track list scrolled down.
       */
      const scrolled = await scrollTrackListDown(session,);
      await session.ok(`resize ${NARROW_WINDOW.width} ${WIDE_WINDOW.height}`,);
      await session.waitState({
        predicate: isFolded,
        what: 'the folded layout',
        timeoutMs: RESULT_TIMEOUT_MS,
      },);

      /**
       The state in the folded layout.
       */
      const narrow = await settledState(session,);
      expect({
        condition: Math.abs(narrow.track_scroll - scrolled.track_scroll) <= SCROLL_TOLERANCE_PX,
        message: `the folded layout shows offset ${narrow.track_scroll}, the two-pane layout had ${scrolled.track_scroll}`,
      },);
      await session.ok(`resize ${WIDE_WINDOW.width} ${WIDE_WINDOW.height}`,);
      await session.waitState({
        predicate: isUnfolded,
        what: 'the two-pane layout',
        timeoutMs: RESULT_TIMEOUT_MS,
      },);

      /**
       The state back in the two-pane layout.
       */
      const wide = await settledState(session,);
      expect({
        condition: Math.abs(wide.track_scroll - scrolled.track_scroll) <= SCROLL_TOLERANCE_PX,
        message: `widening again shows offset ${wide.track_scroll}, it was ${scrolled.track_scroll} (narrow ${narrow.track_scroll}; ring ${scrolled.ring_track} then ${narrow.ring_track} then ${wide.ring_track}; folder ${wide.folder})`,
      },);
      return `offset ${scrolled.track_scroll} kept at ${NARROW_WINDOW.width} px (${narrow.track_scroll}) and at ${WIDE_WINDOW.width} px (${wide.track_scroll})`;
    },
  },);
}
