/**
 Actions and conditions that checks combine: a key press or a click as an action, and a test of the app's state as a
 condition to wait for.
 
 @example
 ```ts
 import { chordKeys, focusIs, } from './actions.ts';
 
 const act = chordKeys({ mods: ['alt',], key: '2', },);
 const until = focusIs('track-list',);
 ```
 
 @packageDocumentation
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import type { MouseButton, } from './control.ts';
import type { Session, } from './session.ts';
import type { UiState, } from './ui-state.ts';

/**
 Something done to the app through the compositor.
 */
export type Action = (session: Session,) => Promise<void>;

/**
 A test of the app's state.
 */
export type Condition = (state: UiState,) => boolean;

/**
 Taps a key.
 
 @param key - key name in the compositor's vocabulary
 
 @returns the action
 
 @example
 ```ts
 tapKey('tab',);
 ```
 */
export function tapKey(key: string,): Action {
  return function tap(session: Session,): Promise<void> {
    return session.tap(key,);
  };
}

/**
 Presses a key and keeps it down.
 
 @param key - key name
 
 @returns the action
 
 @example
 ```ts
 holdKey('shift',);
 ```
 */
export function holdKey(key: string,): Action {
  return function hold(session: Session,): Promise<void> {
    return session.down(key,);
  };
}

/**
 Releases a key.
 
 @param key - key name
 
 @returns the action
 
 @example
 ```ts
 releaseKey('shift',);
 ```
 */
export function releaseKey(key: string,): Action {
  return function release(session: Session,): Promise<void> {
    return session.up(key,);
  };
}

/**
 Presses modifiers, taps a key, and releases the modifiers.
 
 @param mods - modifier names, pressed in this order
 
 @param key - key tapped while the modifiers are down
 
 @returns the action
 
 @example
 ```ts
 chordKeys({ mods: ['ctrl', 'shift',], key: 'g', },);
 ```
 */
export function chordKeys(
  {
    mods,
    key,
  }: {
    readonly key: string;
    readonly mods: readonly string[];
  },
): Action {
  return function chord(session: Session,): Promise<void> {
    return session.chord({
      mods,
      key,
    },);
  };
}

/**
 Clicks at a position.
 
 @param x - column
 
 @param y - row
 
 @param button - mouse button; left when absent
 
 @returns the action
 
 @example
 ```ts
 clickAt({ x: 100, y: 118, },);
 ```
 */
export function clickAt(
  {
    x,
    y,
    button,
  }: {
    readonly button?: MouseButton;
    readonly x: number;
    readonly y: number;
  },
): Action {
  return function click(session: Session,): Promise<void> {
    return session.click({
      x,
      y,
      ...button === undefined ? {} : { button, },
    },);
  };
}

/**
 Types text.
 
 @param text - the text
 
 @returns the action
 
 @example
 ```ts
 typeText('tone',);
 ```
 */
export function typeText(text: string,): Action {
  return function typeIn(session: Session,): Promise<void> {
    return session.type(text,);
  };
}

/**
 Condition: the keyboard focus label is the given one.
 
 @param label - focus label such as `track-list`
 
 @returns the condition
 
 @example
 ```ts
 focusIs('play',);
 ```
 */
export function focusIs(label: string,): Condition {
  return function hasFocus(state: UiState,): boolean {
    return state.focus === label;
  };
}

/**
 Condition: the page is the given one.
 
 @param route - page name such as `Search`
 
 @returns the condition
 
 @example
 ```ts
 routeIs('Settings',);
 ```
 */
export function routeIs(route: string,): Condition {
  return function onRoute(state: UiState,): boolean {
    return state.route === route;
  };
}

/**
 Condition: the playback mode is the given one.
 
 @param mode - mode number from 0 to 3
 
 @returns the condition
 
 @example
 ```ts
 modeIs(2,);
 ```
 */
export function modeIs(mode: number,): Condition {
  return function inMode(state: UiState,): boolean {
    return state.mode === mode;
  };
}

/**
 Condition: any state at all.
 
 @returns true for every state
 
 @example
 ```ts
 anyState();
 ```
 */
export function anyState(): boolean {
  return true;
}

/**
 Condition: a track is playing.
 
 @param state - a state of the app
 
 @returns whether playback runs
 
 @example
 ```ts
 isPlaying(state,);
 ```
 */
export function isPlaying(state: UiState,): boolean {
  return state.playing;
}

/**
 Condition: playback is paused or stopped.
 
 @param state - a state of the app
 
 @returns whether playback does not run
 
 @example
 ```ts
 isPaused(state,);
 ```
 */
export function isPaused(state: UiState,): boolean {
  return !state.playing;
}

/**
 Condition: the focus ring is shown.
 
 @param state - a state of the app
 
 @returns whether the ring is visible
 
 @example
 ```ts
 isRingVisible(state,);
 ```
 */
export function isRingVisible(state: UiState,): boolean {
  return state.ring_visible;
}

/**
 Condition: the focus ring is hidden.
 
 @param state - a state of the app
 
 @returns whether the ring is not visible
 
 @example
 ```ts
 isRingHidden(state,);
 ```
 */
export function isRingHidden(state: UiState,): boolean {
  return !state.ring_visible;
}

/**
 Condition: the position is within a tolerance of an expected position.
 
 @param expected - expected position in seconds
 
 @param tolerance - allowed distance in seconds
 
 @returns the condition
 
 @example
 ```ts
 positionNear({ expected: 10, tolerance: 1.5, },);
 ```
 */
export function positionNear(
  {
    expected,
    tolerance,
  }: {
    readonly expected: number;
    readonly tolerance: number;
  },
): Condition {
  return function isNear(state: UiState,): boolean {
    return Math.abs(state.position - expected,) <= tolerance;
  };
}

/**
 Waits for a time, as a step between other actions.
 
 @param ms - milliseconds to wait
 
 @returns the action
 
 @example
 ```ts
 pauseFor(120,);
 ```
 */
export function pauseFor(ms: number,): Action {
  return async function pause(): Promise<void> {
    await wait(ms,);
  };
}

/**
 Runs actions one after another.
 
 @param steps - the actions, in order
 
 @returns an action that runs them all
 
 @example
 ```ts
 sequenceOf([tapKey('shift',), pauseFor(120,), tapKey('shift',),],);
 ```
 */
export function sequenceOf(steps: readonly Action[],): Action {
  return async function sequence(session: Session,): Promise<void> {
    for (const step of steps)
      // oxlint-disable-next-line eslint/no-await-in-loop -- the steps are key presses that must reach the app in order
      await step(session,);
  };
}

/**
 Gap between the two taps of a quick double Shift.
 */
const QUICK_GAP_MS = 120;

/**
 Two quick taps of Shift, as the double-Shift detector expects.
 */
export const QUICK_DOUBLE_SHIFT: Action = sequenceOf([
  tapKey('shift',),
  pauseFor(QUICK_GAP_MS,),
  tapKey('shift',),
],);
