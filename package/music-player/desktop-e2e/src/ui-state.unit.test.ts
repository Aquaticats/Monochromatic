/**
 Tests for the decoding of `ui_state` log lines into typed states.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  decodeUiState,
  indexText,
  NO_INDEX,
  parseUiStateLine,
  stateJsonOf,
  UiStateShapeError,
} from './ui-state.ts';

/**
 A state object as the app writes it, with the absent fields as JSON `null`.
 */
const APP_STATE = {
  seq: 7,
  reason: 'key',
  route: 'Player',
  zone: 'surface',
  focus: 'play',
  unfolded: true,
  picker_open: false,
  menu_open: false,
  menu_x: 0,
  menu_y: 0,
  menu_width: 0,
  menu_height: 0,
  folder_scroll: 0,
  track_scroll: 0,
  folder: 'Aaa Tones',
  ring_track: null,
  ring_track_title: null,
  ring_folder: 'Aaa Tones',
  menu_ring: null,
  search_ring: 2,
  search_query: '',
  current_index: 30,
  playing: true,
  mode: 1,
  position: 12.5,
  duration: 200,
  settings_selected: 0,
  ring_visible: true,
  focus_request: 0,
  deck_disabled: false,
  auto_play_on_select: false,
  text_size_pane_title_px: 20,
  text_size_group_heading_px: 20,
  text_size_folder_name_px: 14,
  text_size_track_title_px: 14,
  first_run_title: '',
  badges: ['search=Ctrl+F',],
};

await describe({
  name: stateJsonOf.name,
  children: [
    it({
      name: 'cuts the JSON object out of a plain line',
      fn: async ({ expect, }) => {
        expect(stateJsonOf('INFO music_player: ui_state {"seq":1}',),).toBe('{"seq":1}',);
      },
    },),
    it({
      name: 'ignores color sequences around the marker',
      fn: async ({ expect, }) => {
        expect(stateJsonOf('\u001B[32m INFO\u001B[0m ui_state {"a":{"b":2}}',),).toBe('{"a":{"b":2}}',);
      },
    },),
    it({
      name: 'reaches the last closing brace of the line',
      fn: async ({ expect, }) => {
        expect(stateJsonOf('ui_state {"a":"}"} trailing',),).toBe('{"a":"}"}',);
      },
    },),
    it({
      name: 'gives the empty string for lines without a state',
      fn: async ({ expect, }) => {
        expect(stateJsonOf('INFO other line',),).toBe('',);
        expect(stateJsonOf('ui_state {',),).toBe('',);
        expect(stateJsonOf('ui_state without brace',),).toBe('',);
      },
    },),
  ],
},);

await describe({
  name: decodeUiState.name,
  children: [
    it({
      name: 'turns JSON null into the index and title sentinels',
      fn: async ({ expect, }) => {
        const state = decodeUiState(APP_STATE,);
        expect(state.ring_track,).toBe(NO_INDEX,);
        expect(state.menu_ring,).toBe(NO_INDEX,);
        expect(state.ring_track_title,).toBe('',);
        expect(state.search_ring,).toBe(2,);
        expect(state.current_index,).toBe(30,);
        expect(state.ring_folder,).toBe('Aaa Tones',);
        expect(state.badges,).toEqual(['search=Ctrl+F',],);
        expect(state.position,).toBe(12.5,);
        expect(state.auto_play_on_select,).toBe(false,);
      },
    },),
    it({
      name: 'rejects values that are not objects',
      fn: async ({ expect, }) => {
        expect(() => decodeUiState('text',),).toThrow('the state is not an object',);
        expect(() => decodeUiState(null,),).toThrow(UiStateShapeError,);
        expect(() => decodeUiState([],),).toThrow(UiStateShapeError,);
      },
    },),
    it({
      name: 'names the field that has the wrong type',
      fn: async ({ expect, }) => {
        expect(() => decodeUiState({ ...APP_STATE, seq: '7', },),).toThrow('field "seq" is not a number',);
        expect(() => decodeUiState({ ...APP_STATE, route: 3, },),).toThrow('field "route" is not text',);
        expect(() => decodeUiState({ ...APP_STATE, playing: 'yes', },),).toThrow('field "playing" is not a flag',);
        expect(() => decodeUiState({ ...APP_STATE, auto_play_on_select: 1, },),).toThrow('field "auto_play_on_select" is not a flag',);
        expect(() => decodeUiState({ ...APP_STATE, text_size_track_title_px: '14', },),).toThrow('field "text_size_track_title_px" is not a number',);
        expect(() => decodeUiState({ ...APP_STATE, ring_folder: 5, },),).toThrow('field "ring_folder" is not text',);
        expect(() => decodeUiState({ ...APP_STATE, search_ring: 'x', },),).toThrow('field "search_ring" is not a number',);
        expect(() => decodeUiState({ ...APP_STATE, badges: 'x', },),).toThrow('field "badges" is not a list',);
        expect(() => decodeUiState({ ...APP_STATE, badges: [1,], },),).toThrow('a badge is not text',);
      },
    },),
    it({
      name: 'rejects a state with a missing field',
      fn: async ({ expect, }) => {
        const { focus: _focus, ...withoutFocus } = APP_STATE;
        expect(() => decodeUiState(withoutFocus,),).toThrow('field "focus" is not text',);
      },
    },),
  ],
},);

await describe({
  name: parseUiStateLine.name,
  children: [
    it({
      name: 'gives one state for a state line',
      fn: async ({ expect, }) => {
        const states = parseUiStateLine(`INFO ui_state ${JSON.stringify(APP_STATE,)}`,);
        expect(states,).toHaveLength(1,);
        expect(states[0]?.seq,).toBe(7,);
      },
    },),
    it({
      name: 'gives nothing for other lines',
      fn: async ({ expect, }) => {
        expect(parseUiStateLine('INFO unrelated',),).toEqual([],);
      },
    },),
    it({
      name: 'skips a line whose JSON is cut short',
      fn: async ({ expect, }) => {
        expect(parseUiStateLine('ui_state {"seq":1,"reason":"ke}',),).toEqual([],);
      },
    },),
    it({
      name: 'throws for JSON that is not a state',
      fn: async ({ expect, }) => {
        expect(() => parseUiStateLine('ui_state {"seq":1}',),).toThrow(UiStateShapeError,);
      },
    },),
  ],
},);

await describe({
  name: indexText.name,
  children: [
    it({
      name: 'writes no index as null and an index as its number',
      fn: async ({ expect, }) => {
        expect(indexText(NO_INDEX,),).toBe('null',);
        expect(indexText(0,),).toBe('0',);
        expect(indexText(1_004,),).toBe('1004',);
      },
    },),
  ],
},);
