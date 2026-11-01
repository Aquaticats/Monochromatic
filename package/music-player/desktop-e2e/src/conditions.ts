/**
 Conditions on the app's state that several checks share.
 
 @example
 ```ts
 import { hasTrackRing, } from './conditions.ts';
 ```
 
 @packageDocumentation
 */

import {
  NO_INDEX,
  type UiState,
} from './ui-state.ts';

/**
 Condition: some folder row has the ring.
 
 @param state - a state of the app
 
 @returns whether the folder list shows a ring row
 
 @example
 ```ts
 hasFolderRing(state,);
 ```
 */
export function hasFolderRing(state: UiState,): boolean {
  return state.ring_folder !== '';
}

/**
 Condition: the folder ring is on a folder whose name starts with Z, in either case.
 
 @param state - a state of the app
 
 @returns whether the ring row starts with Z
 
 @example
 ```ts
 ringStartsWithZ(state,);
 ```
 */
export function ringStartsWithZ(state: UiState,): boolean {
  return state.ring_folder
    .toLowerCase()
    .startsWith('z',);
}

/**
 Condition: the shown folder is the last one, which holds the file the Delete check sends to the trash.
 
 @param state - a state of the app
 
 @returns whether the shown folder name starts with Zzz
 
 @example
 ```ts
 isLastFolder(state,);
 ```
 */
export function isLastFolder(state: UiState,): boolean {
  return state.folder
    .startsWith('Zzz',);
}

/**
 Condition: a track row has the ring.
 
 @param state - a state of the app
 
 @returns whether the track list shows a ring row
 
 @example
 ```ts
 hasTrackRing(state,);
 ```
 */
export function hasTrackRing(state: UiState,): boolean {
  return state.ring_track_title !== '';
}

/**
 Condition: the ring is on the file the Delete check sends to the trash.
 
 @param state - a state of the app
 
 @returns whether the ring row is `99 Trash me`
 
 @example
 ```ts
 ringIsOnTrashFile(state,);
 ```
 */
export function ringIsOnTrashFile(state: UiState,): boolean {
  return state.ring_track_title
    .includes('Trash me',);
}

/**
 Condition: the track menu is open.
 
 @param state - a state of the app
 
 @returns whether a menu shows
 
 @example
 ```ts
 isMenuOpen(state,);
 ```
 */
export function isMenuOpen(state: UiState,): boolean {
  return state.menu_open;
}

/**
 Condition: the track menu is closed.
 
 @param state - a state of the app
 
 @returns whether no menu shows
 
 @example
 ```ts
 isMenuClosed(state,);
 ```
 */
export function isMenuClosed(state: UiState,): boolean {
  return !state.menu_open;
}

/**
 Condition: the track menu is open and a menu row has the ring. A closed menu never counts, because the app keeps the
 ring of the menu it closed until the next menu opens.
 
 @param state - a state of the app
 
 @returns whether the open menu shows a ring row
 
 @example
 ```ts
 menuHasRing(state,);
 ```
 */
export function menuHasRing(state: UiState,): boolean {
  return state.menu_open && (state.menu_ring !== NO_INDEX);
}

/**
 Condition: the app shows the player page.
 
 @param state - a state of the app
 
 @returns whether the page is Player
 
 @example
 ```ts
 isPlayerPage(state,);
 ```
 */
export function isPlayerPage(state: UiState,): boolean {
  return state.route === 'Player';
}

/**
 Condition: the Search page shows with its query field focused.
 
 @param state - a state of the app
 
 @returns whether Search is the page and the text field has the keyboard
 
 @example
 ```ts
 isSearchWithField(state,);
 ```
 */
export function isSearchWithField(state: UiState,): boolean {
  return (state.route === 'Search') && (state.zone === 'text-field');
}

/**
 Condition: a search result has the ring.
 
 @param state - a state of the app
 
 @returns whether a result is highlighted
 
 @example
 ```ts
 hasResultRing(state,);
 ```
 */
export function hasResultRing(state: UiState,): boolean {
  return state.search_ring !== NO_INDEX;
}

/**
 Condition: the search query holds the typed word.
 
 @param state - a state of the app
 
 @returns whether the query includes `tone`
 
 @example
 ```ts
 queryHasTone(state,);
 ```
 */
export function queryHasTone(state: UiState,): boolean {
  return state.search_query
    .includes('tone',);
}

/**
 Condition: a result was opened and its track is selected on the player page.
 
 @param state - a state of the app
 
 @returns whether the page is Player and a track is selected
 
 @example
 ```ts
 resultOpened(state,);
 ```
 */
export function resultOpened(state: UiState,): boolean {
  return (state.route === 'Player') && (state.current_index !== NO_INDEX);
}

/**
 The fields that must not move while bare keys are typed into the Search field.
 
 @param state - a state of the app
 
 @returns page, current track, playback and mode
 
 @example
 ```ts
 searchFieldsWatched(state,);
 ```
 */
export function searchFieldsWatched(state: UiState,): readonly unknown[] {
  return [
    state.route,
    state.current_index,
    state.playing,
    state.mode,
  ];
}
