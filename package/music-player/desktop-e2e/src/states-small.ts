/**
 The checks of the small states libraries: a single top-level folder and a folder without audio.

 @packageDocumentation
 */

import {
  chordKeys,
  tapKey,
} from './actions.ts';
import type { RunContext, } from './context.ts';
import { expect, } from './errors.ts';
import { press, } from './press.ts';
import type { Session, } from './session.ts';
import {
  isLibraryReady,
  isLibraryReadyUnfolded,
  paneNamesFolder,
  ringHasTitle,
  trackListFocused,
} from './states-conditions.ts';
import { SESSION_READY_MS, } from './timing.ts';
import type { UiState, } from './ui-state.ts';

/**
 Records the checks of the single-folder library: the folder keeps its own name and its tracks are in natural order.

 @param context - the run's context

 @param session - the session on the single-folder library

 @example
 ```ts
 await checkSingleFolder({ context, session, },);
 ```
 */
export async function checkSingleFolder(
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
    id: 'states/single-folder-name',
    title: 'a library with one top-level folder shows that folder by its own name, not a letter',
    body: async function singleFolderKeepsName(): Promise<string> {
      /**
       The state with the folder shown.
       */
      const state = await session.waitState({
        predicate: isLibraryReady,
        what: 'the single folder shown',
        timeoutMs: SESSION_READY_MS,
      },);
      expect({
        condition: state.folder === 'Only Artist',
        message: `the folder was shown as "${state.folder}"`,
      },);
      return `folder "${state.folder}"`;
    },
  },);
  await recorder.check({
    id: 'states/natural-order',
    title: 'tracks are listed in natural order: Take 2 before Take 3 before Take 10',
    body: async function tracksInNaturalOrder(): Promise<string> {
      /*
       The window opens narrow and is widened a moment later; Alt+2 pressed in the narrow layout writes no state line,
       so the check starts from the state of the wide layout. The previous check's state is not enough: it matches the
       first state with the folder, which comes before the widening.
       */
      await session.waitState({
        predicate: isLibraryReadyUnfolded,
        what: 'the single folder shown in the wide layout',
        timeoutMs: SESSION_READY_MS,
      },);
      await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: '2',
        },),
        until: trackListFocused,
        what: 'the track list',
      },);
      /**
       The state with the ring on the first row.
       */
      const first = await press({
        session,
        act: tapKey('home',),
        until: ringHasTitle,
        what: 'the ring on the first row',
      },);
      /**
       The state with the ring on the last row.
       */
      const last = await press({
        session,
        act: tapKey('end',),
        until: function ringMoved(state,): boolean {
          return (state.ring_track_title !== '') && (state.ring_track_title !== first.ring_track_title);
        },
        what: 'the ring on the last row',
      },);
      expect({
        condition: (first.ring_track_title === '2 Zed') && (last.ring_track_title === '10 Zed'),
        message: `first row "${first.ring_track_title}", last row "${last.ring_track_title}"`,
      },);
      return `first "${first.ring_track_title}", last "${last.ring_track_title}"`;
    },
  },);
}

/**
 Records the check of the empty folder: the pane names it and the deck is off.

 @param context - the run's context

 @param session - the session on the folder without audio

 @param name - the folder's name

 @example
 ```ts
 await checkEmptyFolder({ context, session, name: 'states-empty', },);
 ```
 */
export async function checkEmptyFolder(
  {
    context,
    session,
    name,
  }: {
    readonly context: RunContext;
    readonly name: string;
    readonly session: Session;
  },
): Promise<void> {
  /**
   The recorder of the run.
   */
  const { recorder, } = context;
  recorder.setGroup('states',);
  await recorder.check({
    id: 'states/no-audio-folder',
    title: 'a folder without audio is named on its own line and the deck is off',
    body: async function emptyFolderIsNamed(): Promise<string> {
      /**
       The state with the pane shown.
       */
      const state = await session.waitState({
        predicate: paneNamesFolder,
        what: 'the no-audio pane',
        timeoutMs: SESSION_READY_MS,
      },);
      expect({
        condition: (state.first_run_title === `No audio found in\n${name}`) && state.deck_disabled,
        message: `pane "${state.first_run_title}", deck_disabled ${state.deck_disabled}`,
      },);
      /**
       The pane headline on one line.
       */
      const headline = state.first_run_title
        .replace(
        '\n',
        ' | ',
        );
      return `pane "${headline}", shot ${await session.shot('states-no-audio',)}`;
    },
  },);
}
