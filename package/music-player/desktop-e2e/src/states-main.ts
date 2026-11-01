/**
 The checks of the main states library: reading, loose files, Play with nothing selected, the window title and In order.

 @packageDocumentation
 */

import {
  chordKeys,
  isPlaying,
  tapKey,
} from './actions.ts';
import type { RunContext, } from './context.ts';
import { expect, } from './errors.ts';
import { press, } from './press.ts';
import type { Session, } from './session.ts';
import {
  FIRST_FOLDER_TRACK,
  FOLDER_PLAYS_OUT_MS,
  firstFolderShown,
  firstFolderShownWide,
  folderListFocused,
  isLibraryReady,
  isReading,
  playsFirstFolderTrack,
  playsSecondFolderTrack,
  ringIsOnFirstFolder,
  ringIsOnL,
  SCAN_PAUSE_MS,
  stoppedAndRewound,
} from './states-conditions.ts';
import { waitForCleanedTitle, } from './states-title.ts';
import { SESSION_READY_MS, } from './timing.ts';
import {
  NO_INDEX,
  type UiState,
} from './ui-state.ts';

/**
 Makes sure the first track of the first folder plays, starting it with Space when nothing plays yet. A track that
 already plays (the state Play with nothing selected leaves behind) is not touched, because Space would pause it.

 @param session - the session on the main library

 @returns the state in which the first track of the folder plays

 @throws CheckFailureError when the library never shows the first folder in the wide layout, or Space does not start the track

 @example
 ```ts
 await ensureFirstTrackPlaying({ session, },);
 ```
 */
async function ensureFirstTrackPlaying(
  {
    session,
  }: {
    readonly session: Session;
  },
): Promise<UiState> {
  // Run alone, the check starts before the app has written any state line, so it waits for the wide layout first.
  await session.waitState({
    predicate: firstFolderShownWide,
    what: 'the first folder shown in the wide layout',
    timeoutMs: SESSION_READY_MS,
  },);
  /**
   The latest state before the check acts.
   */
  const latest = await session.last();
  if (playsFirstFolderTrack(latest,))
    return latest;
  return await press({
    session,
    act: tapKey('space',),
    until: playsFirstFolderTrack,
    what: 'the first track of the folder playing',
  },);
}

/**
 Records the checks of the main states library: reading, loose files, Play with nothing selected, the window title and
 In order.

 @param context - the run's context

 @param session - the session on the main library, started with a slow scan

 @example
 ```ts
 await checkMainLibrary({ context, session, },);
 ```
 */
export async function checkMainLibrary(
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
  recorder.setGroup('states',);
  await recorder.check({
    id: 'states/reading',
    title: 'while the folder is read the window says "Reading your library" and the deck is off, not "No folder is open"',
    body: async function readingShowsReadingState(): Promise<string> {
      /**
       The state while the scan is paused.
       */
      const state = await session.waitState({
        predicate: isReading,
        what: 'the reading state',
        timeoutMs: SESSION_READY_MS,
      },);
      expect({
        condition: state.first_run_title !== 'No folder is open',
        message: `the pane said "${state.first_run_title}"`,
      },);
      return `pane "${state.first_run_title}", deck_disabled ${state.deck_disabled}, shot ${await session.shot('states-reading',)}`;
    },
  },);
  await recorder.check({
    id: 'states/reading-ends',
    title: 'when the scan ends the pane goes, the first folder shows and the deck works',
    body: async function readingEnds(): Promise<string> {
      /**
       The state once the library is ready.
       */
      const state = await session.waitState({
        predicate: isLibraryReady,
        what: 'the library after the scan',
        timeoutMs: SCAN_PAUSE_MS + SESSION_READY_MS,
      },);
      return `folder "${state.folder}", deck_disabled ${state.deck_disabled}`;
    },
  },);
  await recorder.check({
    id: 'states/loose-letter-folder',
    title: 'loose files sit in the virtual folder of their letter, shown after the real folders',
    body: async function looseFilesSitInLetterFolder(): Promise<string> {
      await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: '1',
        },),
        until: folderListFocused,
        what: 'the folder list',
      },);
      /**
       The state with the ring on the L folder.
       */
      const state = await press({
        session,
        act: tapKey('l',),
        until: ringIsOnL,
        what: 'the ring on the folder L',
      },);
      return `ring on "${state.ring_folder}"`;
    },
  },);
  await recorder.check({
    id: 'states/play-with-nothing-selected',
    title: 'Space with nothing selected starts the first track of the folder on screen instead of flipping to Pause',
    body: async function playWithNothingSelected(): Promise<string> {
      await press({
        session,
        act: tapKey('a',),
        until: ringIsOnFirstFolder,
        what: 'the ring on the first folder',
      },);
      await press({
        session,
        act: tapKey('enter',),
        until: firstFolderShown,
        what: 'the first folder shown',
      },);
      /**
       The state just before Space: nothing is current.
       */
      const before = await session.last();
      expect({
        condition: before.current_index === NO_INDEX,
        message: `a track was already current: ${before.current_index}`,
      },);
      /**
       The state with the first track of the folder playing.
       */
      const state = await press({
        session,
        act: tapKey('space',),
        until: playsFirstFolderTrack,
        what: 'the first track of the folder playing',
      },);
      return `playing track ${state.current_index}, position ${state.position}`;
    },
  },);
  await recorder.check({
    id: 'states/window-title',
    title: 'the window title is the cleaned title the deck shows, without folder, extension or the shared prefix',
    body: async function windowTitleIsCleaned(): Promise<string> {
      /*
       The check needs the first track of the folder playing. After Play with nothing selected it already plays; run
       alone, or after that check failed, nothing plays and the check starts the track itself.
       */
      await ensureFirstTrackPlaying({ session, },);
      /**
       The compositor's answer to the title command, once the window shows the cleaned title.
       */
      const title = await waitForCleanedTitle({ session, },);
      return `title "${title}"`;
    },
  },);
  await recorder.check({
    id: 'states/in-order-stops',
    title: 'In order plays the folder to its end, stops with the first track loaded, and Play starts it again',
    body: async function inOrderStops(): Promise<string> {
      /**
       The state with the second track playing.
       */
      const second = await session.waitState({
        predicate: playsSecondFolderTrack,
        what: 'the second track of the folder',
        timeoutMs: FOLDER_PLAYS_OUT_MS,
      },);
      /**
       The state after the last track ended; only states after the second track began count, because the states of
       the first Play (a paused first track at position zero) look the same.
       */
      const stopped = await session.waitState({
        after: second.seq,
        predicate: stoppedAndRewound,
        what: 'the stop at the end of the folder with the first track loaded',
        timeoutMs: FOLDER_PLAYS_OUT_MS,
      },);
      /**
       The state after Play.
       */
      const again = await press({
        session,
        act: tapKey('space',),
        until: isPlaying,
        what: 'play again',
      },);
      expect({
        condition: again.current_index === FIRST_FOLDER_TRACK,
        message: `Play after the stop started track ${again.current_index}, not the first track of the folder`,
      },);
      return `stopped at track ${stopped.current_index}, played again at track ${again.current_index}`;
    },
  },);
}
