/**
 Tests for choosing a folder by its name through the app's own folder list, with a stand-in for the app that models the
 three keys the walk uses: Alt+1 for the folder list, a plain letter for the first name that starts with it, Down for the
 next row, and Enter for the folder under the ring.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CheckFailureError, } from './errors.ts';
import {
  FOLDER_WALK_STEP_TIMEOUT_MS,
  folderLetterOf,
  selectFolderByName,
} from './folder-choice.ts';
import {
  decodeUiState,
  type UiState,
} from './ui-state.ts';
import type { WaitForState, } from './ui-state-log.ts';

/**
 A state of the stand-in app, before a field is replaced.
 */
const BASE_FIELDS = {
  auto_play_on_select: false,
  badges: [],
  current_index: null,
  deck_disabled: false,
  duration: 0,
  first_run_title: '',
  focus: 'track-list',
  focus_request: 0,
  folder: 'Aaa Tones',
  folder_scroll: 0,
  menu_height: 0,
  menu_open: false,
  menu_ring: null,
  menu_width: 0,
  menu_x: 0,
  menu_y: 0,
  mode: 1,
  picker_open: false,
  playing: false,
  position: 0,
  reason: '',
  ring_folder: null,
  ring_track: null,
  ring_track_title: null,
  ring_visible: false,
  route: 'Player',
  search_query: '',
  search_ring: null,
  seq: 1,
  settings_selected: 0,
  text_size_folder_name_px: 12,
  text_size_group_heading_px: 12,
  text_size_pane_title_px: 12,
  text_size_track_title_px: 12,
  track_scroll: 0,
  unfolded: true,
  zone: 'surface',
} as const;

/**
 The keys the walk sends, as a string, so a test can prove which ones the app got and in what order.
 */
type Key = string;

/**
 The keys the app reads as a jump to a letter: a plain letter or digit in the folder list.
 */
const JUMP_KEYS = 'abcdefghijklmnopqrstuvwxyz0123456789';

/**
 A stand-in for one session: the folder list of the app, the keys it received, and the state log the walk reads.
 */
type StandIn = {
  readonly keys: readonly Key[];
  readonly session: {
    readonly chord: (chord: { readonly key: string; readonly mods: readonly string[]; },) => Promise<void>;
    readonly last: () => Promise<UiState>;
    readonly seq: () => Promise<number>;
    readonly tap: (key: string,) => Promise<void>;
    readonly waitState: (wait: WaitForState,) => Promise<UiState>;
  };
};

/**
 Builds the stand-in app.
 
 @param names - the folder names of the list, in the app's order
 
 @param shown - the folder the track pane shows before the walk
 
 @returns the stand-in and the keys it receives
 
 @example
 ```ts
 const { session, keys } = standIn({ names: ['Aaa Tones', 'Mmm Tones',], shown: 'Aaa Tones', },);
 ```
 */
function standIn(
  {
    names,
    shown,
  }: {
    readonly names: readonly string[];
    readonly shown: string;
  },
): StandIn {
  /**
   Every state the app logged, oldest first; each key that changes a field logs one more.
   */
  const states: UiState[] = [
    decodeUiState({
      ...BASE_FIELDS,
      folder: shown,
    },),
  ];
  const keys: Key[] = [];
  /**
   The state the app is in, which is the last one it logged.
   */
  function current(): UiState {
    return states.at(-1) as UiState;
  }
  /**
   Logs one new state over the current one, the way the app logs each observable change.
   */
  function log(over: (state: UiState,) => UiState,): void {
    states.push({
      ...over(current(),),
      seq: states.length + 1,
    },);
  }
  /**
   The stand-in session.
   */
  const session: StandIn['session'] = {
    async chord(chord,) {
      const movesTheKeyboard = (chord.key === '1')
        && (chord.mods.length === 1)
        && (chord.mods[0] === 'alt');
      if (movesTheKeyboard && (current().focus !== 'folder-list')) {
        log(function folderListFocused(state,) {
          return {
            ...state,
            focus: 'folder-list',
            zone: 'folder-list',
          };
        },);
      }
    },
    async last() {
      return current();
    },
    async seq() {
      return current().seq;
    },
    async tap(key,) {
      keys.push(key,);
      const ring = current().ring_folder;
      if (key === 'down') {
        const next = names[names.indexOf(ring,) + 1];
        if (next !== undefined) {
          log(function ringOnNext(state,) {
            return {
              ...state,
              ring_folder: next,
              ring_visible: true,
            };
          },);
        }
      }
      else if (key === 'enter') {
        if (ring !== '') {
          log(function folderOpened(state,) {
            return {
              ...state,
              focus: 'track-list',
              folder: ring,
              zone: 'track-list',
            };
          },);
        }
      }
      else if ((key.length === 1) && JUMP_KEYS.includes(key,)) {
        const first = names.find(function startsWith(name,): boolean {
          return name
            .toLowerCase()
            .startsWith(key,);
        },);
        if ((first !== undefined) && (first !== ring)) {
          log(function ringOnFirstOfLetter(state,) {
            return {
              ...state,
              ring_folder: first,
              ring_visible: true,
            };
          },);
        }
      }
    },
    async waitState({ after, predicate, what, },) {
      const match = states.find(function newerAndWanted(state,): boolean {
        return ((after ?? 0) < state.seq) && predicate(state,);
      },);
      if (match !== undefined)
        return match;
      throw new CheckFailureError(
        `timed out after ${String(FOLDER_WALK_STEP_TIMEOUT_MS,)} ms waiting for ${what}`,
      );
    },
  };
  return {
    keys,
    session,
  };
}

await describe({
  name: folderLetterOf.name,
  children: [
    it({
      name: 'reads the letter the app jumps to from the first character of the name, in either case',
      fn: async ({ expect, },) => {
        expect(folderLetterOf('Mmm Tones',),).toBe('m',);
        expect(folderLetterOf('zzz tOnes',),).toBe('z',);
        expect(folderLetterOf('',),).toBe('',);
      },
    },),
  ],
},);

await describe({
  name: selectFolderByName.name,
  children: [
    it({
      name: 'jumps to the letter, steps down to the named folder, and opens it, so no pixel position decides',
      fn: async ({ expect, },) => {
        const stand = standIn({
          names: [
            'Aaa Tones',
            'Mmm Discs',
            'Mmm Tones',
            'Zzz Tones',
          ],
          shown: 'Zzz Tones',
        },);
        const shown = await selectFolderByName({
          session: stand.session,
          name: 'Mmm Tones',
          stepLimit: 7,
        },);
        expect(shown.folder,).toBe('Mmm Tones',);
        expect(stand.keys,).toStrictEqual([
          'm',
          'down',
          'enter',
        ],);
      },
    },),
    it({
      name: 'opens a folder the letter jump already landed on without a pointless Down',
      fn: async ({ expect, },) => {
        const stand = standIn({
          names: [
            'Aaa Tones',
            'Mmm Tones',
            'Nnn Tones',
          ],
          shown: 'Aaa Tones',
        },);
        const shown = await selectFolderByName({
          session: stand.session,
          name: 'Mmm Tones',
          stepLimit: 7,
        },);
        expect(shown.folder,).toBe('Mmm Tones',);
        expect(stand.keys,).toStrictEqual([
          'm',
          'enter',
        ],);
      },
    },),
    it({
      name: 'fails naming the folder, the ring it stopped on and the steps it took, when the list holds no such folder',
      fn: async ({ expect, },) => {
        const stand = standIn({
          names: [
            'Aaa Tones',
            'Mmm Tones',
          ],
          shown: 'Aaa Tones',
        },);
        let caught: unknown;
        try {
          await selectFolderByName({
            session: stand.session,
            name: 'Nnn Tones',
            stepLimit: 1,
          },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(CheckFailureError,);
        expect((caught as Error).message,).toContain('Nnn Tones',);
        expect((caught as Error).message,).toContain('Aaa Tones',);
      },
    },),
    it({
      name: 'leaves the keyboard untouched when the folder name has no letter to jump to',
      fn: async ({ expect, },) => {
        const stand = standIn({
          names: ['Aaa Tones',],
          shown: 'Aaa Tones',
        },);
        let caught: unknown;
        try {
          await selectFolderByName({
            session: stand.session,
            name: '',
            stepLimit: 3,
          },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(CheckFailureError,);
        expect(stand.keys,).toStrictEqual([],);
      },
    },),
  ],
},);
