/**
 Checks of the wide session about clicks that must not pass through the Search page or through a toast: while Search
 is open, a click on its empty area does nothing to the player under it, and a click on the body of a toast does not
 reach the track row under the toast. Each check proves its click points are live first (the same click answers on a row once
 the page or the toast is gone), so a pass means the page or the toast took the click, not that there was nothing under it.

 @example
 ```ts
 import { checkClickThrough, } from './wide-interaction.ts';

 await checkClickThrough({ context, session, },);
 ```

 @packageDocumentation
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  type Action,
  chordKeys,
  clickAt,
  focusIs,
  isPaused,
  modeIs,
  pauseFor,
  sequenceOf,
  tapKey,
  typeText,
} from './actions.ts';
import {
  hasTrackRing,
  isPlayerPage,
  isSearchWithField,
} from './conditions.ts';
import type { RunContext, } from './context.ts';
import { expect, } from './errors.ts';
import {
  WIDE_CLICK_POINTS,
  WIDE_POINTS,
} from './layout.ts';
import {
  noEffect,
  press,
} from './press.ts';
import type { Session, } from './session.ts';
import {
  NO_INDEX,
  type UiState,
} from './ui-state.ts';

/**
 A window position in pixels.
 */
type Point = {
  readonly x: number;
  readonly y: number;
};

/**
 How long a one-line notice stays: the 4 s floor of a brief message plus its reading time, rounded up.
 */
const NOTICE_GONE_MS = 5_200;

/**
 Time for a toast to finish its fade in before it is clicked.
 */
const TOAST_FADE_MS = 300;

/**
 Distance from the first track row's click point to a point on the second row, in pixels: a row is about 72 px tall with
 its supporting line, so the second row spans the 70 px to 140 px below the first row's top edge.
 */
const SECOND_ROW_OFFSET_PX = 90;

/**
 Row A: the first track row of the folder on screen, the point the track menu checks use as well.
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
 How long a selected row is watched to see that it stays paused, in milliseconds.
 */
const SELECT_STAYS_PAUSED_MS = 1_500;

/**
 Fields that a click on a page or a toast must leave alone: the page, the playing track and its state, the mode, the folder,
 the focused element and its zone, and the query.

 @param state - a state of the app

 @returns the watched fields

 @example
 ```ts
 clickLeftThingsAlone(state,);
 ```
 */
export function clickLeftThingsAlone(state: UiState,): readonly unknown[] {
  return [
    state.route,
    state.current_index,
    state.playing,
    state.mode,
    state.folder,
    state.focus,
    state.zone,
    state.search_query,
  ];
}

/**
 Fields that a click on a toast must leave alone: the page, the playing track and its state, the mode, and the folder.
 
 @param state - a state of the app
 
 @returns the watched fields
 
 @example
 ```ts
 clickLeftPlaybackAlone(state,);
 ```
 */
function clickLeftPlaybackAlone(state: UiState,): readonly unknown[] {
  return [
    state.route,
    state.current_index,
    state.playing,
    state.mode,
    state.folder,
  ];
}

/**
 Time the app gets to apply typed text before the key that makes it report it, in milliseconds.
 */
const TYPED_TEXT_SETTLE_MS = 600;

/**
 Types the word the checks search for, then presses Down.
 
 The app writes its query into the state log only when something else makes it log a line, so typed text is not visible in
 the log right away; Down on the results is such an event, and it carries the query the field really holds.
 */
export const TYPE_QUERY_THEN_DOWN: Action = sequenceOf([
  typeText('tone',),
  pauseFor(TYPED_TEXT_SETTLE_MS,),
  tapKey('down',),
],);

/**
 Condition: the Search query is the word the checks type.
 
 @param state - a state of the app
 
 @returns whether the query reads `tone`
 
 @example
 ```ts
 queryIsTone(state,);
 ```
 */
export function queryIsTone(state: UiState,): boolean {
  return state.search_query === 'tone';
}

/**
 Clicks each point in turn.
 
 @param points - window positions in pixels
 
 @returns the action
 
 @example
 ```ts
 clicksAt([{ x: 700, y: 300, },],);
 ```
 */
export function clicksAt(points: readonly Point[],): Action {
  return sequenceOf(points.map(function clickPoint(point,): Action {
    return clickAt(point,);
  },),);
}

/**
 Pauses playback when it runs.
 
 The tracks of the long folder are a second or two long, so the playing track would move on by itself while a check watches
 it. A key press pauses it: the lists and the surface keep Space for play and pause.
 
 @param session - the wide session
 
 @example
 ```ts
 await pausePlayback(session,);
 ```
 */
async function pausePlayback(session: Session,): Promise<void> {
  if ((await session.last()).playing)
    await press({
      session,
      act: tapKey('space',),
      until: isPaused,
      what: 'playback pausing',
    },);
}

/**
 Records the click-through checks of the wide session.

 @param context - the run's context

 @param session - the wide session, on the player page with a folder shown

 @example
 ```ts
 await checkClickThrough({ context, session, },);
 ```
 */
export async function checkClickThrough(
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
  recorder.setGroup('click-through',);

  // The earlier checks may leave a page open; the clicks below are about the page Search covers, the player.
  if ((await session.last()).route !== 'Player')
    await press({
      session,
      act: tapKey('escape',),
      until: isPlayerPage,
      what: 'the player page',
    },);
  await pausePlayback(session,);

  await recorder.check({
    id: 'click-through/row-select-then-toggle',
    title: 'a click on an unselected row selects it paused, and clicks on the selected row play it, then pause it (D124)',
    body: async function rowClicksFollowTheRule(): Promise<string> {
      // The tracks of the folder on screen are short, so Repeat keeps the track under the clicks from moving on by itself.
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

      /**
       The state before the first click on row A.
       */
      const beforeA = await session.last();

      // Row A: whatever it was, one or two clicks leave it selected and paused.
      await press({
        session,
        act: clickAt(ROW_A,),
        until: function rowAAnswered(state: UiState,): boolean {
          return (state.current_index !== beforeA.current_index) || (state.playing !== beforeA.playing);
        },
        what: 'row A answering the click',
      },);
      if ((await session.last()).playing)
        await press({
          session,
          act: clickAt(ROW_A,),
          until: isPaused,
          what: 'row A paused by a second click',
        },);

      /**
       Library index of row A, which is now the selected track and is paused.
       */
      const indexA = (await session.last()).current_index;
      expect({
        condition: indexA !== NO_INDEX,
        message: 'a click on row A selected no track',
      },);
      const playingA = await press({
        session,
        act: clickAt(ROW_A,),
        until: function rowAPlays(state: UiState,): boolean {
          return (state.current_index === indexA) && state.playing;
        },
        what: 'a click on the selected row A playing it',
      },);

      // Row B is another row: the click selects it, and the track that played stops.
      const selectedB = await press({
        session,
        act: clickAt(ROW_B,),
        until: function rowBSelectedPaused(state: UiState,): boolean {
          return (state.current_index !== indexA) && (state.current_index !== NO_INDEX)
            && (!state.playing);
        },
        what: 'a click on row B selecting it and stopping row A',
      },);
      await session.shot('row-b-selected',);

      /**
       Library index of row B.
       */
      const indexB = selectedB.current_index;
      await wait(SELECT_STAYS_PAUSED_MS,);
      expect({
        condition: (!(await session.last()).playing) && ((await session.last()).current_index === indexB),
        message: 'row B started playing by itself after it was selected',
      },);
      const playingB = await press({
        session,
        act: clickAt(ROW_B,),
        until: function rowBPlays(state: UiState,): boolean {
          return (state.current_index === indexB) && state.playing;
        },
        what: 'a click on the selected row B playing it',
      },);
      await session.shot('row-b-playing',);
      const pausedB = await press({
        session,
        act: clickAt(ROW_B,),
        until: function rowBPaused(state: UiState,): boolean {
          return (state.current_index === indexB) && (!state.playing);
        },
        what: 'a click on the playing row B pausing it',
      },);
      await press({
        session,
        act: chordKeys({
          mods: [
            'ctrl',
            'shift',
          ],
          key: '2',
        },),
        until: modeIs(1,),
        what: 'in order again',
      },);
      return `A (${indexA}) selected, played ${playingA.playing}; B (${indexB}) selected paused ${!selectedB.playing}, played ${
        playingB.playing}, paused ${!pausedB.playing}`;
    },
  },);
  // The click points need a track row under each of them, so the pane shows the folder of thirty tracks.
  /**
   The folder shown before the click.
   */
  const folderBefore = (await session.last()).folder;
  await press({
    session,
    act: clickAt(WIDE_CLICK_POINTS.longFolder,),
    until: function otherFolderShown(state: UiState,): boolean {
      return state.folder !== folderBefore;
    },
    what: 'the folder of thirty tracks shown in the track pane',
  },);
  await recorder.check({
    id: 'click-through/search-wide',
    title: 'clicks on the empty Search page do not reach the hidden track list, and typing still works',
    body: async function searchTakesItsClicks(): Promise<string> {
      await press({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'f',
        },),
        until: isSearchWithField,
        what: 'the Search page with the query field focused',
      },);
      await noEffect({
        session,
        act: clicksAt(WIDE_CLICK_POINTS.underSearch,),
        watch: clickLeftThingsAlone,
        what: 'clicks on the empty Search page',
      },);
      await session.shot('search-after-clicks',);

      /**
       The state after typing, which proves the query field kept the keyboard focus through the clicks.
       */
      const typed = await press({
        session,
        act: TYPE_QUERY_THEN_DOWN,
        until: queryIsTone,
        what: 'the typed query reaching the field',
      },);
      await press({
        session,
        act: tapKey('escape',),
        until: isPlayerPage,
        what: 'the player page',
      },);

      /**
       The track playing before the control click.
       */
      const before = await session.last();
      await session.click(WIDE_CLICK_POINTS.underSearch[0],);
      await session.waitState({
        predicate: function aRowAnswered(state: UiState,): boolean {
          return (state.current_index !== before.current_index) || (state.playing !== before.playing);
        },
        what: 'the track row answering the same click on the player page, by loading or by toggling play (the control for the clicks above)',
        after: before.seq,
      },);
      return `query "${typed.search_query}" typed after the clicks; the same click on the player page answers`;
    },
  },);
  await recorder.check({
    id: 'click-through/toast-wide',
    title: 'a click on the body of a toast does not play the track row under it',
    body: async function toastTakesItsClicks(): Promise<string> {
      await pausePlayback(session,);
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
        act: tapKey('down',),
        until: hasTrackRing,
        what: 'the ring on a row',
      },);
      await session.chord({
        mods: [
          'ctrl',
          'shift',
        ],
        key: 'c',
      },);
      await wait(TOAST_FADE_MS,);
      await session.shot('toast-before-click',);
      await noEffect({
        session,
        act: clickAt(WIDE_CLICK_POINTS.toastBody,),
        watch: clickLeftPlaybackAlone,
        what: 'a click on the body of the toast',
      },);
      await session.shot('toast-after-click',);

      // The toast leaves; the same click then selects the row under it, or toggles play on it when it is the selected
      // track, which proves the click point was over a live row.
      await wait(NOTICE_GONE_MS,);

      /**
       The playing track before the control click.
       */
      const before = await session.last();
      await session.click(WIDE_CLICK_POINTS.toastBody,);
      await session.waitState({
        predicate: function aRowAnswered(state: UiState,): boolean {
          return (state.current_index !== before.current_index) || (state.playing !== before.playing);
        },
        what: 'the track row answering the same click once the toast is gone, by loading or by toggling play (the control for the click above)',
        after: before.seq,
      },);
      expect({
        condition: (await session.last()).route === 'Player',
        message: 'the page changed',
      },);
      return 'the click on the toast changed nothing; the same click after the toast left reaches the row under it';
    },
  },);
}
