/**
 The building blocks of a check: press something and wait for the state it should cause, prove that a key changed
 nothing, and walk the Tab order.
 
 @example
 ```ts
 import { press, } from './press.ts';
 
 const state = await press({ session, act: tapKey('tab',), until: focusIs('play',), what: 'focus on Play', },);
 ```
 
 @packageDocumentation
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import {
  type Action,
  anyState,
  type Condition,
  focusIs,
  tapKey,
} from './actions.ts';
import { isMenuClosed, } from './conditions.ts';
import { expect, } from './errors.ts';
import { pollUntil, } from './poll.ts';
import type { Session, } from './session.ts';
import {
  BRIEF_MS,
  NO_EFFECT_SETTLE_MS,
} from './timing.ts';
import type { UiState, } from './ui-state.ts';

/**
 How many Tab presses a walk to a focus label may take.
 */
const DEFAULT_TAB_LIMIT = 14;

/**
 How long a Tab press waits for the state line it causes.
 */
const TAB_STATE_TIMEOUT_MS = 3_000;

/**
 Presses through an action, then waits for a state line newer than the press that satisfies a condition.
 
 @param session - the session to press in
 
 @param act - what to do
 
 @param until - what the wanted state looks like
 
 @param what - words for the timeout message
 
 @param timeoutMs - milliseconds to wait; the default wait when absent
 
 @returns the state that satisfied the condition
 
 @throws CheckFailureError when no such state arrives in time
 
 @example
 ```ts
 const state = await press({ session, act: tapKey('space',), until: isPaused, what: 'pause', },);
 ```
 */
export async function press(
  {
    session,
    act,
    until,
    what,
    timeoutMs,
  }: {
    readonly act: Action;
    readonly session: Session;
    readonly timeoutMs?: number;
    readonly until: Condition;
    readonly what: string;
  },
): Promise<UiState> {
  /**
   Sequence number of the latest state before the press.
   */
  const after = await session.seq();
  await act(session,);
  return await session.waitState({
    predicate: until,
    what,
    after,
    ...timeoutMs === undefined ? {} : { timeoutMs, },
  },);
}

/**
 Closes the track menu when it is open, so a check starts from a closed menu whatever ran before it. A closed menu
 sends no key: Escape on a closed menu writes no state line, so waiting for one would end only when an unrelated line
 (a position tick, a late focus line) happened to arrive.
 
 @param session - the session to press in
 
 @throws CheckFailureError when the menu stays open after Escape
 
 @example
 ```ts
 await closeMenuIfOpen({ session, },);
 ```
 */
export async function closeMenuIfOpen(
  {
    session,
  }: {
    readonly session: Session;
  },
): Promise<void> {
  if (isMenuClosed(await session.last(),))
    return;
  await press({
    session,
    act: tapKey('escape',),
    until: isMenuClosed,
    what: 'Escape closing the track menu a check before left open',
  },);
}

/**
 Presses through an action and proves nothing happened: the app reported no shortcut use, and none of the watched
 fields moved.
 
 @param session - the session to press in
 
 @param act - what to do
 
 @param watch - picks the fields that must stay the same
 
 @param what - words for the failure message
 
 @param settleMs - milliseconds to wait for a late effect; the default when absent
 
 @throws CheckFailureError when a shortcut was used or a watched field changed
 
 @example
 ```ts
 await noEffect({ session, act: tapKey('escape',), watch: nothingMoved, what: 'Escape', },);
 ```
 */
export async function noEffect(
  {
    session,
    act,
    watch,
    what,
    settleMs = NO_EFFECT_SETTLE_MS,
  }: {
    readonly act: Action;
    readonly session: Session;
    readonly settleMs?: number;
    readonly watch: (state: UiState,) => unknown;
    readonly what: string;
  },
): Promise<void> {
  /**
   State before the press.
   */
  const before = await session.last();

  /**
   Shortcut uses before the press.
   */
  const usedBefore = await session.used();
  await act(session,);
  await wait(settleMs,);

  /**
   State after the press.
   */
  const after = await session.last();

  /**
   Shortcut uses after the press.
   */
  const usedAfter = await session.used();
  expect({
    condition: usedAfter === usedBefore,
    message: `${what}: the app reported using a shortcut (${usedAfter - usedBefore} new "shortcut used" lines)`,
  },);
  expect({
    condition: JSON.stringify(watch(before,),) === JSON.stringify(watch(after,),),
    message: `${what}: state changed from ${JSON.stringify(watch(before,),)} to ${JSON.stringify(watch(after,),)}`,
  },);
}

/**
 The fields that must not move when a key does nothing: page, current track, playback, mode, folder, and the two
 overlays.
 
 @param state - a state of the app
 
 @returns the watched fields
 
 @example
 ```ts
 nothingMoved(state,);
 ```
 */
export function nothingMoved(state: UiState,): readonly unknown[] {
  return [
    state.route,
    state.current_index,
    state.playing,
    state.mode,
    state.folder,
    state.picker_open,
    state.menu_open,
  ];
}

/**
 Taps Tab until the focus label is the wanted one.
 
 @param session - the session to press in
 
 @param label - the focus label to reach
 
 @param limit - the most Tab presses to make
 
 @returns the state with the wanted focus
 
 @throws CheckFailureError when Tab never reaches the label
 
 @example
 ```ts
 await tabTo({ session, label: 'play', },);
 ```
 */
export async function tabTo(
  {
    session,
    label,
    limit = DEFAULT_TAB_LIMIT,
  }: {
    readonly label: string;
    readonly limit?: number;
    readonly session: Session;
  },
): Promise<UiState> {
  /**
   Tagged logger naming the walk.
   */
  const rl = tagged({
    tag: tabTo.name,
    l: logger,
  },);

  /**
   One step of the walk: stop if the label is reached, else press Tab and give the app a moment.
   
   @returns whether the label is reached
   */
  async function step(): Promise<boolean> {
    if ((await session.last()).focus === label)
      return true;
    try {
      await press({
        session,
        act: tapKey('tab',),
        until: anyState,
        what: 'a state line after Tab',
        timeoutMs: TAB_STATE_TIMEOUT_MS,
      },);
    }
    catch (error: unknown) {
      rl.debug(`no state line followed Tab: ${String(error,)}`,);
    }
    await wait(BRIEF_MS,);
    return false;
  }

  await pollUntil({
    attempts: limit,
    intervalMs: 0,
    order: 'check-then-wait',
    check: step,
  },);

  /**
   State after the walk.
   */
  const state = await session.last();
  expect({
    condition: state.focus === label,
    message: `Tab never reached "${label}"; stopped on "${state.focus}"`,
  },);
  return state;
}

/**
 Presses a key once per expected focus label and waits for each label in turn.
 
 @param session - the session to press in
 
 @param expected - focus labels in the order the key should reach them
 
 @param key - the action that moves the focus
 
 @returns the labels seen, joined with ` > `
 
 @throws CheckFailureError when a label is not reached in time
 
 @example
 ```ts
 const walk = await tabSequence({ session, expected: ['a', 'b',], key: tapKey('tab',), },);
 ```
 */
export async function tabSequence(
  {
    session,
    expected,
    key,
  }: {
    readonly expected: readonly string[];
    readonly key: Action;
    readonly session: Session;
  },
): Promise<string> {
  /**
   Labels seen so far.
   */
  const seen: string[] = [];
  for (const label of expected) {
    /**
     The state with the wanted label.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- each press must follow the previous label's arrival
    const state = await press({
      session,
      act: key,
      until: focusIs(label,),
      what: `focus on "${label}"`,
    },);
    seen.push(state.focus,);
  }
  return seen.join(' > ',);
}
