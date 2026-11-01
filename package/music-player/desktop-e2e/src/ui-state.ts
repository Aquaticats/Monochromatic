/**
 The app's `ui_state` log lines: extraction from a log line and decoding into a typed record.
 
 The app writes one `ui_state {json}` line at each observable change. The JSON mirrors the app's Rust record; here the
 JSON `null` of an absent index or title becomes a sentinel value (`NO_INDEX`, the empty string), so no field is a
 nullish union.
 
 @example
 ```ts
 import { parseUiStateLine, } from './ui-state.ts';
 
 const states = parseUiStateLine('INFO ui_state {"seq":1}',);
 ```
 
 @packageDocumentation
 */

import {
  tagged,
  logger
} from '@monochromatic-dev/logger-core/ts';

import { stripAnsi, } from './ansi.ts';

/**
 Marker that starts the JSON object of a state line.
 */
const STATE_MARKER = 'ui_state {';

/**
 Value of an index field (track, menu row, search result, current track) that points at nothing.
 */
export const NO_INDEX = -1;

/**
 One observable state of the app, as it logged it.
 */
export type UiState = {
  /**
   Whether the Auto-play on select setting is on (D129).
   */
  readonly auto_play_on_select: boolean;
  readonly badges: readonly string[];
  readonly current_index: number;
  /**
   Whether the transport deck is switched off, as it is while the library is read or holds nothing to play.
   */
  readonly deck_disabled: boolean;
  readonly duration: number;
  /**
   Headline of the first-run pane, or the empty string while that pane is not shown.
   */
  readonly first_run_title: string;
  readonly focus: string;
  readonly focus_request: number;
  readonly folder: string;
  /**
   How far the folder picker's names are scrolled down, in whole pixels, 0 at the top.
   */
  readonly folder_scroll: number;
  /**
   Height of the open track menu in window pixels, 0 while no menu is open.
   */
  readonly menu_height: number;
  readonly menu_open: boolean;
  readonly menu_ring: number;
  /**
   Width of the open track menu in window pixels, 0 while no menu is open.
   */
  readonly menu_width: number;
  /**
   Left edge of the open track menu in window pixels, 0 while no menu is open.
   */
  readonly menu_x: number;
  /**
   Top edge of the open track menu in window pixels, 0 while no menu is open.
   */
  readonly menu_y: number;
  readonly mode: number;
  readonly picker_open: boolean;
  readonly playing: boolean;
  readonly position: number;
  readonly reason: string;
  readonly ring_folder: string;
  readonly ring_track: number;
  readonly ring_track_title: string;
  readonly ring_visible: boolean;
  readonly route: string;
  readonly search_query: string;
  readonly search_ring: number;
  readonly seq: number;
  readonly settings_selected: number;
  /**
   The Text size setting: the size in pixels the folder names are drawn at.
   */
  readonly text_size_folder_name_px: number;
  /**
   The Text size setting: the size in pixels of the group heading letter.
   */
  readonly text_size_group_heading_px: number;
  /**
   The Text size setting: the size in pixels of the pane titles.
   */
  readonly text_size_pane_title_px: number;
  /**
   The Text size setting: the size in pixels of the track titles.
   */
  readonly text_size_track_title_px: number;
  /**
   How far the track list on screen is scrolled down, in whole pixels, 0 at the top.
   */
  readonly track_scroll: number;
  readonly unfolded: boolean;
  readonly zone: string;
};

/**
 The record a state line decodes from, before any field is checked.
 */
type Fields = Readonly<Record<string, unknown>>;

/**
 A state line whose JSON does not have the shape of a state. The reader skips such a line.
 
 @example
 ```ts
 throw new UiStateShapeError('field "seq" is not a number',);
 ```
 */
export class UiStateShapeError extends Error {
  /**
   Names the error so a skipped line is logged with its kind.
   
   @param message - which field has the wrong shape
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'UiStateShapeError';
  }
}

/**
 Narrows a decoded JSON value to a plain object.
 
 @param value - decoded JSON
 
 @returns whether the value is a non-null, non-array object
 
 @example
 ```ts
 isFields({ seq: 1, },); // true
 ```
 */
function isFields(value: unknown,): value is Fields {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Reads a text field.
 
 @param fields - decoded state object
 
 @param key - field name
 
 @returns the text
 
 @throws UiStateShapeError when the field is not text
 
 @example
 ```ts
 textField({ fields: { route: 'Player', }, key: 'route', },); // 'Player'
 ```
 */
function textField(
  {
    fields,
    key,
  }: {
    readonly fields: Fields;
    readonly key: string;
  },
): string {
  /**
   Raw field value.
   */
  const value = fields[key];
  if ((typeof value) !== 'string')
    throw new UiStateShapeError(`field "${key}" is not text`,);
  return value;
}

/**
 Reads a number field.
 
 @param fields - decoded state object
 
 @param key - field name
 
 @returns the number
 
 @throws UiStateShapeError when the field is not a number
 
 @example
 ```ts
 numberField({ fields: { seq: 3, }, key: 'seq', },); // 3
 ```
 */
function numberField(
  {
    fields,
    key,
  }: {
    readonly fields: Fields;
    readonly key: string;
  },
): number {
  /**
   Raw field value.
   */
  const value = fields[key];
  if ((typeof value) !== 'number')
    throw new UiStateShapeError(`field "${key}" is not a number`,);
  return value;
}

/**
 Reads a flag field.
 
 @param fields - decoded state object
 
 @param key - field name
 
 @returns the flag
 
 @throws UiStateShapeError when the field is not a flag
 
 @example
 ```ts
 flagField({ fields: { playing: true, }, key: 'playing', },); // true
 ```
 */
function flagField(
  {
    fields,
    key,
  }: {
    readonly fields: Fields;
    readonly key: string;
  },
): boolean {
  /**
   Raw field value.
   */
  const value = fields[key];
  if ((typeof value) !== 'boolean')
    throw new UiStateShapeError(`field "${key}" is not a flag`,);
  return value;
}

/**
 Reads an index field whose JSON `null` means nothing is pointed at.
 
 @param fields - decoded state object
 
 @param key - field name
 
 @returns the index, or `NO_INDEX` for `null`
 
 @throws UiStateShapeError when the field is neither a number nor `null`
 
 @example
 ```ts
 indexField({ fields: { ring_track: null, }, key: 'ring_track', },); // -1
 ```
 */
function indexField(
  {
    fields,
    key,
  }: {
    readonly fields: Fields;
    readonly key: string;
  },
): number {
  if (fields[key] === null)
    return NO_INDEX;
  return numberField({
    fields,
    key,
  },);
}

/**
 Reads a title field whose JSON `null` means there is no title.
 
 @param fields - decoded state object
 
 @param key - field name
 
 @returns the title, or the empty string for `null`
 
 @throws UiStateShapeError when the field is neither text nor `null`
 
 @example
 ```ts
 titleField({ fields: { ring_folder: null, }, key: 'ring_folder', },); // ''
 ```
 */
function titleField(
  {
    fields,
    key,
  }: {
    readonly fields: Fields;
    readonly key: string;
  },
): string {
  if (fields[key] === null)
    return '';
  return textField({
    fields,
    key,
  },);
}

/**
 Reads the list of badge names.
 
 @param fields - decoded state object
 
 @returns the badge names
 
 @throws UiStateShapeError when the field is not a list of text
 
 @example
 ```ts
 badgesField({ badges: ['search=Ctrl+F',], },); // ['search=Ctrl+F']
 ```
 */
function badgesField(fields: Fields,): readonly string[] {
  /**
   Raw field value.
   */
  const value = fields.badges;
  if (!Array.isArray(value,))
    throw new UiStateShapeError('field "badges" is not a list',);
  return value.map(function checkBadge(badge: unknown,): string {
    if ((typeof badge) !== 'string')
      throw new UiStateShapeError('a badge is not text',);
    return badge;
  },);
}

/**
 Decodes the JSON value of a state line into a state.
 
 @param value - decoded JSON
 
 @returns the state, with `NO_INDEX` and the empty string standing for the JSON `null` of absent fields
 
 @throws UiStateShapeError when the value is not an object or a field has the wrong type
 
 @example
 ```ts
 decodeUiState(JSON.parse(json,),);
 ```
 */
export function decodeUiState(value: unknown,): UiState {
  if (!isFields(value,))
    throw new UiStateShapeError('the state is not an object',);
  return {
    auto_play_on_select: flagField({
      fields: value,
      key: 'auto_play_on_select',
    },),
    badges: badgesField(value,),
    current_index: indexField({
      fields: value,
      key: 'current_index',
    },),
    deck_disabled: flagField({
      fields: value,
      key: 'deck_disabled',
    },),
    duration: numberField({
      fields: value,
      key: 'duration',
    },),
    first_run_title: textField({
      fields: value,
      key: 'first_run_title',
    },),
    focus: textField({
      fields: value,
      key: 'focus',
    },),
    focus_request: numberField({
      fields: value,
      key: 'focus_request',
    },),
    folder: textField({
      fields: value,
      key: 'folder',
    },),
    folder_scroll: numberField({
      fields: value,
      key: 'folder_scroll',
    },),
    menu_height: numberField({
      fields: value,
      key: 'menu_height',
    },),
    menu_open: flagField({
      fields: value,
      key: 'menu_open',
    },),
    menu_ring: indexField({
      fields: value,
      key: 'menu_ring',
    },),
    menu_width: numberField({
      fields: value,
      key: 'menu_width',
    },),
    menu_x: numberField({
      fields: value,
      key: 'menu_x',
    },),
    menu_y: numberField({
      fields: value,
      key: 'menu_y',
    },),
    mode: numberField({
      fields: value,
      key: 'mode',
    },),
    picker_open: flagField({
      fields: value,
      key: 'picker_open',
    },),
    playing: flagField({
      fields: value,
      key: 'playing',
    },),
    position: numberField({
      fields: value,
      key: 'position',
    },),
    reason: textField({
      fields: value,
      key: 'reason',
    },),
    ring_folder: titleField({
      fields: value,
      key: 'ring_folder',
    },),
    ring_track: indexField({
      fields: value,
      key: 'ring_track',
    },),
    ring_track_title: titleField({
      fields: value,
      key: 'ring_track_title',
    },),
    ring_visible: flagField({
      fields: value,
      key: 'ring_visible',
    },),
    route: textField({
      fields: value,
      key: 'route',
    },),
    search_query: textField({
      fields: value,
      key: 'search_query',
    },),
    search_ring: indexField({
      fields: value,
      key: 'search_ring',
    },),
    seq: numberField({
      fields: value,
      key: 'seq',
    },),
    settings_selected: numberField({
      fields: value,
      key: 'settings_selected',
    },),
    text_size_folder_name_px: numberField({
      fields: value,
      key: 'text_size_folder_name_px',
    },),
    text_size_group_heading_px: numberField({
      fields: value,
      key: 'text_size_group_heading_px',
    },),
    text_size_pane_title_px: numberField({
      fields: value,
      key: 'text_size_pane_title_px',
    },),
    text_size_track_title_px: numberField({
      fields: value,
      key: 'text_size_track_title_px',
    },),
    track_scroll: numberField({
      fields: value,
      key: 'track_scroll',
    },),
    unfolded: flagField({
      fields: value,
      key: 'unfolded',
    },),
    zone: textField({
      fields: value,
      key: 'zone',
    },),
  };
}

/**
 Cuts the JSON object out of a log line: from the first `ui_state {` to the last closing brace of the line.
 
 @param line - one log line, with or without color sequences
 
 @returns the JSON text, or the empty string when the line holds no state
 
 @example
 ```ts
 stateJsonOf('INFO ui_state {"seq":1}',); // '{"seq":1}'
 ```
 */
export function stateJsonOf(line: string,): string {
  /**
   The line without color sequences.
   */
  const plain = stripAnsi(line,);

  /**
   Where the marker starts, or -1.
   */
  const marker = plain.indexOf(STATE_MARKER,);
  if (marker === (-1))
    return '';

  /**
   Where the JSON object starts: the brace that ends the marker.
   */
  const start = (marker + STATE_MARKER.length) - 1;

  /**
   Where the last brace of the line is.
   */
  const end = plain.lastIndexOf('}',);
  return end < start ? '' : plain.slice(
    start,
    end + 1,
  );
}

/**
 Reads the state a log line carries.
 
 A line that is cut short is skipped and logged.
 
 @param line - one log line
 
 @returns a list of the one state the line carries, or an empty list for any other line
 
 @throws UiStateShapeError when the line carries JSON that is not a state
 
 @example
 ```ts
 parseUiStateLine('INFO unrelated',); // []
 ```
 */
export function parseUiStateLine(line: string,): readonly UiState[] {
  /**
   Tagged logger naming the line reader.
   */
  const rl = tagged({
    tag: parseUiStateLine.name,
    l: logger,
  },);

  /**
   JSON text of the line.
   */
  const json = stateJsonOf(line,);
  if (json === '')
    return [];
  try {
    return [decodeUiState(JSON.parse(json,),),];
  }
  catch (error: unknown) {
    // A line cut short is skipped; a state of the wrong shape means the app changed its contract, which must be loud.
    if (error instanceof SyntaxError) {
      rl.debug(`skipped a state line that is not JSON: ${error.message}`,);
      return [];
    }
    throw error;
  }
}

/**
 Writes an index field the way the app's JSON does: `null` for no index, else the number.
 
 @param index - an index field of a state
 
 @returns the text for result lines
 
 @example
 ```ts
 indexText(NO_INDEX,); // 'null'
 ```
 */
export function indexText(index: number,): string {
  return index === NO_INDEX ? 'null' : String(index,);
}
