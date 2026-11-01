/**
 Conditions and constants of the states sessions.

 @packageDocumentation
 */

import type { UiState, } from './ui-state.ts';

/**
 How long the folder scan pauses after its last batch in the reading session, in milliseconds. It is the window in
 which the reading state can be seen.
 */
export const SCAN_PAUSE_MS = 6_000;

/**
 How long the app may take to play two short tracks to the end and stop, in milliseconds; generous because a busy machine
 delays the audio.
 */
export const FOLDER_PLAYS_OUT_MS = 45_000;

/**
 The queue position of the first track of the main library's first folder: three loose files come first in the walk.
 */
export const FIRST_FOLDER_TRACK = 3;

/**
 How far into the track the position may be for the track to count as rewound to its start, in seconds.
 */
export const REWOUND_POSITION_SECONDS = 0.5;

/**
 Condition: the first-run pane says the library is being read and the deck is off.

 @param state - a state of the app

 @returns whether the reading state shows

 @example
 ```ts
 isReading(state,);
 ```
 */
export function isReading(state: UiState,): boolean {
  return (state.first_run_title === 'Reading your library') && state.deck_disabled;
}

/**
 Condition: the library is shown with a folder, no pane, and a working deck.

 @param state - a state of the app

 @returns whether the player shows its folder

 @example
 ```ts
 isLibraryReady(state,);
 ```
 */
export function isLibraryReady(state: UiState,): boolean {
  return (state.folder !== '')
    && (state.first_run_title === '')
    && (!state.deck_disabled);
}

/**
 Condition: the library is shown in the wide two-pane layout. A session starts with the window at its default width
 and the compositor widens it a moment later, so a key pressed before this state meets the narrow layout and its
 effect never arrives.

 @param state - a state of the app

 @returns whether the library is shown with a folder, no pane, a working deck and the unfolded layout

 @example
 ```ts
 isLibraryReadyUnfolded(state,);
 ```
 */
export function isLibraryReadyUnfolded(state: UiState,): boolean {
  return isLibraryReady(state,) && state.unfolded;
}

/**
 Condition: the wide layout shows the first real folder of the main library, ready to play.

 @param state - a state of the app

 @returns whether `Aaa Folder` is shown in the unfolded layout with a working deck

 @example
 ```ts
 firstFolderShownWide(state,);
 ```
 */
export function firstFolderShownWide(state: UiState,): boolean {
  return isLibraryReadyUnfolded(state,) && (state.folder === 'Aaa Folder');
}

/**
 Condition: the folder list ring is on the folder called `L`.

 @param state - a state of the app

 @returns whether the ring is on `L`

 @example
 ```ts
 ringIsOnL(state,);
 ```
 */
export function ringIsOnL(state: UiState,): boolean {
  return state.ring_folder === 'L';
}

/**
 Condition: playback started on the first track of the first folder.

 @param state - a state of the app

 @returns whether that track plays

 @example
 ```ts
 playsFirstFolderTrack(state,);
 ```
 */
export function playsFirstFolderTrack(state: UiState,): boolean {
  return state.playing && (state.current_index === FIRST_FOLDER_TRACK);
}

/**
 Condition: the second track of the first folder plays.

 @param state - a state of the app

 @returns whether that track plays

 @example
 ```ts
 playsSecondFolderTrack(state,);
 ```
 */
export function playsSecondFolderTrack(state: UiState,): boolean {
  return state.playing && (state.current_index === (FIRST_FOLDER_TRACK + 1));
}

/**
 Condition: playback stopped with the first track of the folder loaded again, at its start.

 @param state - a state of the app

 @returns whether the folder ended and rewound

 @example
 ```ts
 stoppedAndRewound(state,);
 ```
 */
export function stoppedAndRewound(state: UiState,): boolean {
  return (!state.playing)
    && (state.current_index === FIRST_FOLDER_TRACK)
    && (state.position < REWOUND_POSITION_SECONDS);
}

/**
 Condition: the folder list has the keyboard.

 @param state - a state of the app

 @returns whether the folder list is focused

 @example
 ```ts
 folderListFocused(state,);
 ```
 */
export function folderListFocused(state: UiState,): boolean {
  return state.focus === 'folder-list';
}

/**
 Condition: the track list has the keyboard.

 @param state - a state of the app

 @returns whether the track list is focused

 @example
 ```ts
 trackListFocused(state,);
 ```
 */
export function trackListFocused(state: UiState,): boolean {
  return state.focus === 'track-list';
}

/**
 Condition: the folder list ring is on the first real folder of the main library.

 @param state - a state of the app

 @returns whether the ring is on `Aaa Folder`

 @example
 ```ts
 ringIsOnFirstFolder(state,);
 ```
 */
export function ringIsOnFirstFolder(state: UiState,): boolean {
  return state.ring_folder === 'Aaa Folder';
}

/**
 Condition: the first real folder of the main library is shown.

 @param state - a state of the app

 @returns whether `Aaa Folder` is shown

 @example
 ```ts
 firstFolderShown(state,);
 ```
 */
export function firstFolderShown(state: UiState,): boolean {
  return state.folder === 'Aaa Folder';
}

/**
 Condition: the first-run pane names a folder that has no audio.

 @param state - a state of the app

 @returns whether the no-audio pane shows

 @example
 ```ts
 paneNamesFolder(state,);
 ```
 */
export function paneNamesFolder(state: UiState,): boolean {
  return state.first_run_title
    .startsWith('No audio found in',);
}

/**
 Condition: the track list ring is on a row.

 @param state - a state of the app

 @returns whether a row title is under the ring

 @example
 ```ts
 ringHasTitle(state,);
 ```
 */
export function ringHasTitle(state: UiState,): boolean {
  return state.ring_track_title !== '';
}
