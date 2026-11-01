/**
 Checks of the wide session about the keys of the track list, run in the last folder.
 
 @example
 ```ts
 import { checkTrackList, } from './wide-track-list.ts';
 
 await checkTrackList({ context, session, },);
 ```
 
 @packageDocumentation
 */

import {
  chordKeys,
  focusIs,
  isPaused,
  tapKey,
} from './actions.ts';
import {
  hasTrackRing,
  isLastFolder,
  ringStartsWithZ,
} from './conditions.ts';
import type { RunContext, } from './context.ts';
import { press, } from './press.ts';
import type { Session, } from './session.ts';
import type { UiState, } from './ui-state.ts';

/**
 Records the track-list checks of the wide session.
 
 @param context - the run's context
 
 @param session - the wide session, with the folder list focused
 
 @example
 ```ts
 await checkTrackList({ context, session, },);
 ```
 */
export async function checkTrackList(
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
  recorder.setGroup('track-list',);
  await press({
    session,
    act: tapKey('z',),
    until: ringStartsWithZ,
    what: 'Z folder',
  },);
  await press({
    session,
    act: tapKey('enter',),
    until: isLastFolder,
    what: 'the Zzz folder',
  },);
  await press({
    session,
    act: chordKeys({
      mods: ['alt',],
      key: '2',
    },),
    until: focusIs('track-list',),
    what: 'the track list',
  },);
  await recorder.check({
    id: 'track-list/keys',
    title: 'Down, Up, Home, End, PageUp and PageDown move the track ring',
    body: async function keysMoveTrackRing(): Promise<string> {
      /**
       The state after Home.
       */
      const home = await press({
        session,
        act: tapKey('home',),
        until: hasTrackRing,
        what: 'Home',
      },);

      /**
       The state after Down.
       */
      const down = await press({
        session,
        act: tapKey('down',),
        until: function ringLeftHome(state: UiState,): boolean {
          return state.ring_track !== home.ring_track;
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
          return state.ring_track === home.ring_track;
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
          return state.ring_track !== home.ring_track;
        },
        what: 'End',
      },);

      /**
       The state after PageUp.
       */
      const pageUp = await press({
        session,
        act: tapKey('pageup',),
        until: function ringOnHomeAfterPageUp(state: UiState,): boolean {
          return state.ring_track === home.ring_track;
        },
        what: 'PageUp',
      },);

      /**
       The state after PageDown.
       */
      const pageDown = await press({
        session,
        act: tapKey('pagedown',),
        until: function ringOnEndAfterPageDown(state: UiState,): boolean {
          return state.ring_track === end.ring_track;
        },
        what: 'PageDown',
      },);
      return [
        `Home "${home.ring_track_title}"`,
        `Down "${down.ring_track_title}"`,
        `Up "${up.ring_track_title}"`,
        `End "${end.ring_track_title}"`,
        `PageUp "${pageUp.ring_track_title}"`,
        `PageDown "${pageDown.ring_track_title}"`,
      ].join(', ',);
    },
  },);
  await recorder.check({
    id: 'track-list/enter-selects-then-toggles',
    title: 'Enter on a row selects it paused, Enter on the selected row plays it, and the next Enter pauses it (D124)',
    body: async function enterSelectsThenToggles(): Promise<string> {
      await press({
        session,
        act: tapKey('home',),
        until: hasTrackRing,
        what: 'Home',
      },);

      /**
       The row under the ring.
       */
      const ring = (await session.last()).ring_track;

      /**
       The state with the row selected, which is not playing yet.
       */
      const selected = await press({
        session,
        act: tapKey('enter',),
        until: function rowIsSelected(state: UiState,): boolean {
          return (state.current_index === ring) && (!state.playing);
        },
        what: 'the row selected and paused',
      },);

      /**
       The state after the second Enter.
       */
      const playing = await press({
        session,
        act: tapKey('enter',),
        until: function rowIsPlaying(state: UiState,): boolean {
          return (state.current_index === ring) && state.playing;
        },
        what: 'the row playing',
      },);

      /**
       The state after the third Enter.
       */
      const paused = await press({
        session,
        act: tapKey('enter',),
        until: isPaused,
        what: 'a pause',
      },);
      return `index ${selected.current_index} selected paused, then ${playing.playing ? 'playing' : 'paused'}, then ${paused
        .playing ? 'playing' : 'paused'}`;
    },
  },);
}
