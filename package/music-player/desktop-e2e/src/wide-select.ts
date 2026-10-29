/**
 Checks of the wide session about the select-only rule (D124, D127) and the unreadable row (D128): a Search result that
 is not the current track only selects it, the result of the current track toggles play, and a row whose file cannot
 be opened stays the selected row until a click tries it again.

 @example
 ```ts
 import { checkSelectOnly, } from './wide-select.ts';

 await checkSelectOnly({ context, session, },);
 ```

 @packageDocumentation
 */

import {
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  type Action,
  chordKeys,
  clickAt,
  isPaused,
  isPlaying,
  modeIs,
  pauseFor,
  sequenceOf,
  tapKey,
  typeText,
} from './actions.ts';
import {
  isPlayerPage,
  isSearchWithField,
} from './conditions.ts';
import type { RunContext, } from './context.ts';
import { expect, } from './errors.ts';
import { WIDE_POINTS, } from './layout.ts';
import { waveBytes, } from './library.ts';
import { press, } from './press.ts';
import type { Session, } from './session.ts';
import {
  NO_INDEX,
  type UiState,
} from './ui-state.ts';

/**
 How long a selected row is watched to see that it stays paused and selected, in milliseconds.
 */
const STAYS_MS = 1_500;

/**
 Time the app gets to apply typed text and answer the query before the keys that make it report them, in milliseconds.
 */
const ANSWER_SETTLE_MS = 1_500;

/**
 Time the file watcher and the refresh sweep get to see a changed file before the next click, in milliseconds.
 */
const WATCHER_SETTLE_MS = 2_500;

/**
 Time the app gets to find a file that was just written in the library, in milliseconds.
 */
const NEW_FILE_TIMEOUT_MS = 10_000;

/**
 The query that finds exactly one invented track of the long tone folder.
 */
const TONE_QUERY = 'Charlie';

/**
 The query that finds exactly the unreadable file this check writes.
 */
const BROKEN_QUERY = 'Qqq';

/**
 Name of the unreadable file: it starts with a letter no other file of the invented library starts with.
 */
const BROKEN_NAME = 'Qqq unreadable tone.wav';

/**
 Seconds of tone in the file that replaces the unreadable one.
 */
const REPAIRED_SECONDS = 30;

/**
 Hertz of the tone in the file that replaces the unreadable one.
 */
const REPAIRED_HERTZ = 330;

/**
 Types a query on the open Search page, waits for its answer, and moves the highlight down and up so the app reports the
 query and the highlighted result in one state line.

 @param query - the words to type

 @returns the action

 @example
 ```ts
 typeQueryAndHighlight('Charlie',);
 ```
 */
function typeQueryAndHighlight(query: string,): Action {
  return sequenceOf([
    typeText(query,),
    pauseFor(ANSWER_SETTLE_MS,),
    tapKey('down',),
    tapKey('up',),
  ],);
}

/**
 Opens Search, types the query, and opens the highlighted result with Enter.

 @param session - the wide session, on the player page

 @param query - the words to type

 @param until - what the player must show after the result is opened

 @param what - words for the timeout message

 @returns the state that satisfied the condition

 @throws CheckFailureError when Search does not open, no result is highlighted, or the player never shows the outcome

 @example
 ```ts
 await openResultFor({ session, query: 'Charlie', until: isPlayerPage, what: 'the player', },);
 ```
 */
async function openResultFor(
  {
    session,
    query,
    until,
    what,
  }: {
    readonly query: string;
    readonly session: Session;
    readonly until: (state: UiState,) => boolean;
    readonly what: string;
  },
): Promise<UiState> {
  await press({
    session,
    act: chordKeys({
      mods: ['ctrl',],
      key: 'f',
    },),
    until: isSearchWithField,
    what: 'the Search page with the query field focused',
  },);
  await press({
    session,
    act: typeQueryAndHighlight(query,),
    until: function queryAnswered(state: UiState,): boolean {
      return (state.search_query === query) && (state.search_ring !== NO_INDEX);
    },
    what: `a highlighted result for ${query}`,
    timeoutMs: NEW_FILE_TIMEOUT_MS,
  },);
  return await press({
    session,
    act: tapKey('enter',),
    until,
    what,
  },);
}

/**
 Records the select-only checks of the wide session.

 @param context - the run's context

 @param session - the wide session, on the player page

 @example
 ```ts
 await checkSelectOnly({ context, session, },);
 ```
 */
export async function checkSelectOnly(
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
  recorder.setGroup('search',);

  // The earlier checks may leave a page open; the checks below start from the player.
  if ((await session.last()).route !== 'Player')
    await press({
      session,
      act: tapKey('escape',),
      until: isPlayerPage,
      what: 'the player page',
    },);

  await recorder.check({
    id: 'search/result-selects-then-toggles',
    title: 'a Search result for another track selects it paused and stops the playing track; the same result again plays it (D127)',
    body: async function resultFollowsTheRowRule(): Promise<string> {
      // The long tone tracks are 200 s, but Repeat keeps the track under the check from ever moving on by itself.
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
      if (!(await session.last()).playing)
        await press({
          session,
          act: tapKey('space',),
          until: isPlaying,
          what: 'a track playing before the search',
        },);

      /**
       The track that plays while the result is activated.
       */
      const before = await session.last();
      const selected = await openResultFor({
        session,
        query: TONE_QUERY,
        until: function selectedNotPlaying(state: UiState,): boolean {
          return (state.route === 'Player') && (state.current_index !== NO_INDEX)
            && (state.current_index !== before.current_index)
            && (!state.playing);
        },
        what: 'the result selecting another track, paused, with the playing track stopped',
      });
      await session.shot('search-result-selected',);
      await wait(STAYS_MS,);
      expect({
        condition: (!(await session.last()).playing) && ((await session.last()).current_index === selected.current_index),
        message: 'the track started playing by itself after its Search result selected it',
      },);
      const playing = await openResultFor({
        session,
        query: TONE_QUERY,
        until: function currentPlays(state: UiState,): boolean {
          return (state.route === 'Player') && (state.current_index === selected.current_index)
            && state.playing;
        },
        what: 'the result of the current track playing it',
      });
      await session.shot('search-result-playing',);
      await press({
        session,
        act: tapKey('space',),
        until: isPaused,
        what: 'playback pausing again',
      },);
      return `track ${before.current_index} played; the result selected ${selected.current_index} paused (playing ${selected.playing}); `
        + `the same result played it (playing ${playing.playing})`;
    },
  },);

  recorder.setGroup('click-through',);
  await recorder.check({
    id: 'click-through/unreadable-row-stays-selected',
    title: 'a row whose file cannot be opened stays the selected row, and the next click on it loads it once it decodes (D128)',
    body: async function unreadableRowStays(): Promise<string> {
      /**
       Path of the unreadable file, which the app finds while it runs.
       */
      const broken = join(
        context.library,
        BROKEN_NAME,
      );
      await writeFile(
        broken,
        'not audio',
      );
      try {
        const selected = await openResultFor({
          session,
          query: BROKEN_QUERY,
          until: function brokenSelected(state: UiState,): boolean {
            // Nothing is loaded for the unreadable row, so its length is zero; its library index may equal the one the
            // playing track had before the new file shifted the queue, so the index cannot tell the two apart.
            return (state.route === 'Player') && (state.current_index !== NO_INDEX)
              && (!state.playing)
              && (state.duration === 0);
          },
          what: 'the unreadable file selected from its Search result with nothing loaded',
        });
        await session.shot('unreadable-selected',);
        await wait(STAYS_MS,);
        expect({
          condition: (!(await session.last()).playing) && ((await session.last()).current_index === selected.current_index),
          message: 'the selection left the unreadable row or something played after the select',
        },);

        // The row is the current row: the click is the toggle, which tries the file again and fails again.
        await session.click(WIDE_POINTS.trackRow,);
        await wait(STAYS_MS,);
        expect({
          condition: (!(await session.last()).playing) && ((await session.last()).current_index === selected.current_index),
          message: 'a click on the unreadable current row moved the selection or played something',
        },);
        await session.shot('unreadable-clicked-again',);

        // The file is repaired; the next click on the same row loads and plays it.
        await writeFile(
          broken,
          waveBytes({
            seconds: REPAIRED_SECONDS,
            hertz: REPAIRED_HERTZ,
          },),
        );
        await wait(WATCHER_SETTLE_MS,);
        const played = await press({
          session,
          act: clickAt(WIDE_POINTS.trackRow,),
          until: function repairedPlays(state: UiState,): boolean {
            return (state.current_index === selected.current_index) && state.playing;
          },
          what: 'a click on the selected row playing the repaired file',
        },);
        await session.shot('unreadable-repaired-playing',);
        await press({
          session,
          act: tapKey('space',),
          until: isPaused,
          what: 'playback pausing again',
        },);
        return `row ${selected.current_index} stayed selected and paused through two tries; after repair it played (${played.playing})`;
      } finally {
        await rm(
          broken,
          { force: true, },
        );
        await wait(WATCHER_SETTLE_MS,);
      }
    },
  },);
}
