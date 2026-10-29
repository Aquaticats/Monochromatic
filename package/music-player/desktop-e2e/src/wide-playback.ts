/**
 Checks of the wide session about playing: Enter on a row twice, the private-sink proof, Space, the four modes, and the
 next and previous track chords.
 
 @example
 ```ts
 import { checkPlayback, } from './wide-playback.ts';
 
 await checkPlayback({ context, session, },);
 ```
 
 @packageDocumentation
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  chordKeys,
  focusIs,
  isPaused,
  isPlaying,
  modeIs,
  tapKey,
} from './actions.ts';
import type { RunContext, } from './context.ts';
import {
  CheckFailureError,
  expect,
} from './errors.ts';
import { currentRouting, } from './pipewire.ts';
import { pollUntil, } from './poll.ts';
import {
  press,
  tabTo,
} from './press.ts';
import type { Session, } from './session.ts';
import {
  NO_INDEX,
  type UiState,
} from './ui-state.ts';

/**
 How many times the app's streams are looked for in the PipeWire graph.
 */
const ROUTING_LOOKS = 30;

/**
 Pause between two looks at the PipeWire graph.
 */
const ROUTING_PAUSE_MS = 300;

/**
 Wait that lets a double toggle show itself after a single key press.
 */
const DOUBLE_TOGGLE_WAIT_MS = 600;

/**
 Number of playback modes: Repeat, In order, Shuffle folder, Shuffle all.
 */
const MODE_COUNT = 4;

/**
 Ctrl+Shift digit to the mode it picks.
 */
const MODE_DIGITS = [
  {
    key: '1',
    mode: 0,
  },
  {
    key: '2',
    mode: 1,
  },
  {
    key: '3',
    mode: 2,
  },
  {
    key: '4',
    mode: 3,
  },
] as const;

/**
 Condition: a track is selected and playing.
 
 @param state - a state of the app
 
 @returns whether a track plays
 
 @example
 ```ts
 isTrackPlaying(state,);
 ```
 */
function isTrackPlaying(state: UiState,): boolean {
  return (state.current_index !== NO_INDEX) && state.playing;
}

/**
 Records the play, audio, mode and track checks of the wide session.
 
 @param context - the run's context
 
 @param session - the wide session
 
 @example
 ```ts
 await checkPlayback({ context, session, },);
 ```
 */
export async function checkPlayback(
  {
    context,
    session,
  }: {
    readonly context: RunContext;
    readonly session: Session;
  },
): Promise<void> {
  /**
   The recorder and the private sink of the run.
   */
  const {
    recorder,
    sink,
  } = context;
  recorder.setGroup('playback',);
  await recorder.check({
    id: 'play/enter-plays-row',
    title: 'Alt+2 then Enter selects the row under the ring, and Enter again plays it (D124)',
    body: async function enterPlaysRow(): Promise<string> {
      await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: '2',
        },),
        until: focusIs('track-list',),
        what: 'the track list',
      },);

      /**
       Title of the row under the ring.
       */
      const ring = (await session.last()).ring_track_title;

      /**
       Index of the row under the ring.
       */
      const ringIndex = (await session.last()).ring_track;

      // A row that is not the selected track is only selected by the first Enter, and paused; the second Enter plays it.
      if ((await session.last()).current_index !== ringIndex) {
        await press({
          session,
          act: tapKey('enter',),
          until: function rowIsSelectedAndPaused(state: UiState,): boolean {
            return (state.current_index === ringIndex) && (!state.playing);
          },
          what: 'the row selected and paused',
        },);
      }

      /**
       The state once the selected row plays.
       */
      const state = (await session.last()).playing
        ? await session.last()
        : await press({
          session,
          act: tapKey('enter',),
          until: isTrackPlaying,
          what: 'a playing track',
        },);
      return `ring on "${ring}", now playing index ${state.current_index}, duration ${state.duration
        .toFixed(0,)} s`;
    },
  },);
  await recorder.check({
    id: 'audio/private-sink',
    title: 'playback goes to the private null sink (node id) and nowhere else',
    body: async function playbackGoesToPrivateSink(): Promise<string> {
      /**
       The latest routing description.
       */
      const seen = { detail: '', };

      /**
       One look at the app's streams and links; a stream linked elsewhere ends the run.
       
       @returns whether the app has a link to the private sink
       */
      async function lookAtRouting(): Promise<boolean> {
        /**
         Streams and links of the app.
         */
        const routing = await currentRouting({
          appPid: session.appPid(),
          sink,
        },);
        seen.detail = routing.detail;
        return routing.linked > 0;
      }

      /**
       Whether a link showed up in time.
       */
      const linked = await pollUntil({
        attempts: ROUTING_LOOKS,
        intervalMs: ROUTING_PAUSE_MS,
        order: 'check-then-wait',
        check: lookAtRouting,
      },);
      if (!linked)
        throw new CheckFailureError(`the app never linked to the private sink: ${seen.detail}`,);
      return `${seen.detail}; sink node ${sink.nodeId}`;
    },
  },);
  await recorder.verify({
    id: 'play/space-list',
    title: 'Space toggles play and pause in the track list',
    body: async function spaceTogglesInTrackList(): Promise<void> {
      await press({
        session,
        act: tapKey('space',),
        until: isPaused,
        what: 'pause',
      },);
      await press({
        session,
        act: tapKey('space',),
        until: isPlaying,
        what: 'play',
      },);
    },
  },);
  await recorder.verify({
    id: 'play/space-folder-list',
    title: 'Space toggles play and pause in the folder list',
    body: async function spaceTogglesInFolderList(): Promise<void> {
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
        act: tapKey('space',),
        until: isPaused,
        what: 'pause',
      },);
      await press({
        session,
        act: tapKey('space',),
        until: isPlaying,
        what: 'play',
      },);
    },
  },);
  await recorder.verify({
    id: 'play/space-button',
    title: 'Space on the focused Play button presses it once',
    body: async function spacePressesButtonOnce(): Promise<void> {
      await tabTo({
        session,
        label: 'play',
      },);
      await press({
        session,
        act: tapKey('space',),
        until: isPaused,
        what: 'pause',
      },);
      await wait(DOUBLE_TOGGLE_WAIT_MS,);
      expect({
        condition: !(await session.last()).playing,
        message: 'Space toggled twice',
      },);
      await press({
        session,
        act: tapKey('enter',),
        until: isPlaying,
        what: 'play by Enter',
      },);
      await wait(DOUBLE_TOGGLE_WAIT_MS,);
      expect({
        condition: (await session.last()).playing,
        message: 'Enter toggled twice',
      },);
    },
  },);
  await recorder.check({
    id: 'mode/ctrl-shift-digits',
    title: 'Ctrl+Shift+1 to 4 pick Repeat, In order, Shuffle folder, Shuffle all',
    body: async function ctrlShiftDigitsPickModes(): Promise<string> {
      /**
       The pairs of key and resulting mode, as text.
       */
      const seen: string[] = [];
      for (const {
        key,
        mode,
      } of MODE_DIGITS) {
        /**
         The state once the mode changed.
         */
        // oxlint-disable-next-line eslint/no-await-in-loop -- each press must follow the previous mode's arrival
        const state = await press({
          session,
          act: chordKeys({
            mods: [
              'ctrl',
              'shift',
            ],
            key,
          },),
          until: modeIs(mode,),
          what: `mode ${mode}`,
        },);
        seen.push(`${key}->${state.mode}`,);
      }
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
      return seen.join(' ',);
    },
  },);
  await recorder.check({
    id: 'mode/ctrl-m',
    title: 'Ctrl+M cycles the mode through all four and wraps',
    body: async function ctrlMCyclesModes(): Promise<string> {
      /**
       The modes seen, starting with the current one.
       */
      const seen: number[] = [(await session.last()).mode,];
      for (let step = 0; step < MODE_COUNT; step++) {
        /**
         The mode one step after the latest.
         */
        const expected = ((seen.at(-1,) ?? 0) + 1) % MODE_COUNT;

        /**
         The state once the mode changed.
         */
        // oxlint-disable-next-line eslint/no-await-in-loop -- each press must follow the previous mode's arrival
        const state = await press({
          session,
          act: chordKeys({
            mods: ['ctrl',],
            key: 'm',
          },),
          until: modeIs(expected,),
          what: `mode ${expected}`,
        },);
        seen.push(state.mode,);
      }
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
      return seen.join(' > ',);
    },
  },);
  await recorder.check({
    id: 'track/ctrl-right-left',
    title: 'Ctrl+Right is the next track and Ctrl+Left the previous',
    body: async function ctrlArrowsChangeTrack(): Promise<string> {
      /**
       Index of the track that plays now.
       */
      const start = (await session.last()).current_index;
      expect({
        condition: start !== NO_INDEX,
        message: 'nothing plays',
      },);

      /**
       The state on the next track.
       */
      const next = await press({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'right',
        },),
        until: function isNextTrack(state: UiState,): boolean {
          return state.current_index === (start
            + 1);
        },
        what: `track ${start + 1}`,
      },);

      /**
       The state back on the first track.
       */
      const previous = await press({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'left',
        },),
        until: function isFirstTrack(state: UiState,): boolean {
          return state.current_index === start;
        },
        what: `track ${start} again`,
      },);
      return `${start} > ${next.current_index} > ${previous.current_index}`;
    },
  },);
}
