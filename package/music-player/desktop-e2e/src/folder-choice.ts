/**
 Choosing a folder by its name through the app's own folder list.
 
 A check that needs a track pane full of rows names the folder it needs; this module walks the list to that name with
 the list's own keys (Alt+1 for the list, a plain letter for the first name that starts with it, Down for the next row,
 Enter for the folder under the ring) and waits for the app to report the folder it shows. No pixel position takes part,
 so a library of another shape, another window size or another font size still reaches the same folder.
 
 @example
 ```ts
 import { selectFolderByName, } from './folder-choice.ts';
 
 await selectFolderByName({ session, name: 'Mmm Tones', stepLimit: 61, },);
 ```
 
 @packageDocumentation
 */

import { hasFolderRing, } from './conditions.ts';
import {
  CheckFailureError,
  expect,
} from './errors.ts';
import { numbersBelow, } from './numbers-below.ts';
import type { Session, } from './session.ts';
import type { UiState, } from './ui-state.ts';

/**
 How long a key that may change nothing waits for the state it would cause, in milliseconds.
 
 A ring move and a folder jump each log a state line in the same frame the key reaches the app, so a second is generous;
 a key that changes nothing waits it out once and the walk carries on.
 */
export const FOLDER_WALK_STEP_TIMEOUT_MS = 1_000;

/**
 What choosing a folder needs from a session: the state log and the keys.
 */
export type FolderChoiceSession = Pick<Session, 'chord' | 'last' | 'seq' | 'tap' | 'waitState'>;

/**
 One key press in a session this module only knows part of.
 */
export type FolderChoiceAction = (session: FolderChoiceSession,) => Promise<void>;

/**
 Taps a key.
 
 @param key - key name in the compositor's vocabulary
 
 @returns the action
 
 @example
 ```ts
 await keyStep('down',)(session,);
 ```
 */
function keyStep(key: string,): FolderChoiceAction {
  return function tapIn(session,): Promise<void> {
    return session.tap(key,);
  };
}

/**
 Presses Alt and a digit, which the app reads as a Focus action.
 
 @param digit - the digit pressed with Alt
 
 @returns the action
 
 @example
 ```ts
 await altDigit('1',)(session,);
 ```
 */
function altDigit(digit: string,): FolderChoiceAction {
  return function chordIn(session,): Promise<void> {
    return session.chord({
      mods: ['alt',],
      key: digit,
    },);
  };
}

/**
 Reads the key the app jumps to for a folder name: its first character, in lower case.
 
 The app maps a plain letter or digit in the folder list to the first folder whose name starts with it.
 
 @param name - folder name
 
 @returns the key, or the empty string when the name has no first character
 
 @example
 ```ts
 folderLetterOf('Mmm Tones',); // 'm'
 ```
 */
export function folderLetterOf(name: string,): string {
  return name.slice(
    0,
    1,
  )
    .toLowerCase();
}

/**
 Sends a key that is allowed to change nothing, and waits at most one step for a state line.
 
 @param session - the session to press in
 
 @param act - what to send
 
 @example
 ```ts
 await sendTolerantly({ session, act: keyStep('m',), },);
 ```
 */
async function sendTolerantly(
  {
    session,
    act,
  }: {
    readonly act: FolderChoiceAction;
    readonly session: FolderChoiceSession;
  },
): Promise<void> {
  /**
   Sequence number of the latest state before the press.
   */
  const after = await session.seq();
  await act(session,);
  try {
    await session.waitState({
      after,
      predicate: anyState,
      timeoutMs: FOLDER_WALK_STEP_TIMEOUT_MS,
      what: 'a state line after a key that may change nothing',
    },);
  }
  catch (error: unknown) {
    if (!(error instanceof CheckFailureError))
      throw error;
  }
}

/**
 Sends a key that should move the folder ring, and reads the ring it left.
 
 A key that reaches the last row logs nothing: the walk reads the ring of the latest state then, and the caller's step
 limit is what ends a walk that never reaches its name.
 
 @param session - the session to press in
 
 @param act - what to send
 
 @returns the folder name under the ring after the key
 
 @example
 ```ts
 const ring = await ringAfterStep({ session, act: keyStep('down',), },);
 ```
 */
async function ringAfterStep(
  {
    session,
    act,
  }: {
    readonly act: FolderChoiceAction;
    readonly session: FolderChoiceSession;
  },
): Promise<string> {
  /**
   Sequence number of the latest state before the press.
   */
  const after = await session.seq();
  await act(session,);
  try {
    /**
     The first state the key caused that shows a ring row.
     */
    const moved = await session.waitState({
      after,
      predicate: hasFolderRing,
      timeoutMs: FOLDER_WALK_STEP_TIMEOUT_MS,
      what: 'the folder ring moving',
    },);
    return moved.ring_folder;
  }
  catch (error: unknown) {
    if (!(error instanceof CheckFailureError))
      throw error;
    return (await session.last()).ring_folder;
  }
}

/**
 Condition: the app shows the named folder in the track pane.
 
 @param name - folder name
 
 @returns the condition
 
 @example
 ```ts
 shownFolderIs('Mmm Tones',);
 ```
 */
function shownFolderIs(name: string,): (state: UiState,) => boolean {
  return function showsFolder(state: UiState,): boolean {
    return state.folder === name;
  };
}

/**
 Condition: any state at all, which is what a key that may change nothing is waited for.
 
 @param _state - a state of the app
 
 @returns true for every state
 */
function anyState(_state: UiState,): boolean {
  return true;
}

/**
 What one row of a walk found: the state that shows the named folder, or the ring standing elsewhere.
 */
type FolderAtRing =
  | {
    readonly kind: 'shown';
    readonly state: UiState;
  }
    | { readonly kind: 'elsewhere'; };

/**
 Opens the named folder when the ring stands on it, else steps the ring down one row.
 
 @param session - the session to drive
 
 @param name - the folder name the app shows for the folder
 
 @returns the state that shows the folder when the ring stood on it, else that the ring stood elsewhere
 
 @example
 ```ts
 const reached = await openWhenRingIsName({ session, name: 'Mmm Tones', },);
 ```
 */
async function openWhenRingIsName(
  {
    session,
    name,
  }: {
    readonly name: string;
    readonly session: FolderChoiceSession;
  },
): Promise<FolderAtRing> {
  /**
   The latest state, which names the folder under the ring.
   */
  const before = await session.last();
  if (before.ring_folder !== name) {
    await ringAfterStep({
      act: keyStep('down',),
      session,
    },);
    return {
      kind: 'elsewhere',
    };
  }
  await session.tap('enter',);
  return {
    kind: 'shown',
    state: await session.waitState({
      after: before.seq,
      predicate: shownFolderIs(name,),
      what: `the folder ${name} shown in the track pane`,
    },),
  };
}

/**
 Chooses a folder by its name and opens it, through the app's own folder list.
 
 @param session - the session to drive
 
 @param name - the folder name the app shows for the folder
 
 @param stepLimit - how many rows the walk may step over; a library's own folder count bounds it
 
 @returns the state that shows the folder
 
 @throws CheckFailureError when the name has no key to jump to, when the folder list does not take the keyboard, or when
 the walk reaches its step limit without finding the name
 
 @example
 ```ts
 const shown = await selectFolderByName({ session, name: 'Mmm Tones', stepLimit: 61, },);
 ```
 */
export async function selectFolderByName(
  {
    session,
    name,
    stepLimit,
  }: {
    readonly name: string;
    readonly session: FolderChoiceSession;
    readonly stepLimit: number;
  },
): Promise<UiState> {
  /**
   The key the app jumps to for the first letter of the name.
   */
  const letter = folderLetterOf(name,);
  expect({
    condition: letter !== '',
    message: `the folder name "${name}" starts with no character the folder list can jump to`,
  },);

  // Alt+1 moves the keyboard to the folder list, which the unfolded layout always shows and the folded one opens as
  // the picker overlay over the player.
  await sendTolerantly({
    act: altDigit('1',),
    session,
  },);
  /**
   The latest state, after the keyboard was asked for the folder list.
   */
  const focused = await session.last();
  expect({
    condition: focused.focus === 'folder-list',
    message: `Alt+1 left the keyboard on ${focused.focus || 'nothing'}, not the folder list`,
  },);

  // A plain letter jumps to the first folder whose name starts with it; the names of one letter are adjacent, so the
  // name the caller wants sits at or below the ring from here.
  await sendTolerantly({
    act: keyStep(letter,),
    session,
  },);
  for (const _step of numbersBelow(stepLimit,)) {
    /**
     What this row of the walk found: the showing state when the ring stood on the name, else that it stood elsewhere.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one row per step, each waiting for the state the key caused
    const reached = await openWhenRingIsName({
      name,
      session,
    },);
    if (reached.kind === 'shown')
      return reached.state;
  }
  /**
   The folder the ring ended on, for the failure message.
   */
  const stoppedOn = (await session.last()).ring_folder;
  throw new CheckFailureError(
    `the folder list shows no folder named ${name}: after ${String(stepLimit,)} steps down from the letter ${letter} the ring stands on ${
      stoppedOn || 'no folder'}`,
  );
}
