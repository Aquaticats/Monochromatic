/**
 Tests for the building blocks of a check, against a fake session that replays a scripted sequence of states.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  chordKeys,
  clickAt,
  focusIs,
  holdKey,
  isPaused,
  isPlaying,
  isRingHidden,
  isRingVisible,
  modeIs,
  positionNear,
  QUICK_DOUBLE_SHIFT,
  releaseKey,
  routeIs,
  sequenceOf,
  tapKey,
  typeText,
} from './actions.ts';
import { menuHasRing, } from './conditions.ts';
import type { Session, } from './session.ts';
import {
  closeMenuIfOpen,
  noEffect,
  nothingMoved,
  press,
  tabSequence,
  tabTo,
} from './press.ts';
import type { UiState, } from './ui-state.ts';

/**
 A state with every field at a neutral value, with some fields replaced.
 
 @param fields - fields that replace the defaults
 
 @returns the state
 */
function state(fields: Readonly<Record<string, unknown>> = {},): UiState {
  return {
    seq: 1,
    reason: 'key',
    route: 'Player',
    zone: 'surface',
    focus: '',
    unfolded: true,
    picker_open: false,
    menu_open: false,
    menu_height: 0,
    folder_scroll: 0,
    track_scroll: 0,
    menu_width: 0,
    menu_x: 0,
    menu_y: 0,
    folder: 'F',
    ring_track: -1,
    ring_track_title: '',
    ring_folder: '',
    menu_ring: -1,
    search_ring: -1,
    search_query: '',
    current_index: -1,
    playing: false,
    mode: 1,
    position: 0,
    duration: 0,
    settings_selected: 0,
    ring_visible: false,
    focus_request: 0,
    deck_disabled: false,
    first_run_title: '',
    badges: [],
    auto_play_on_select: false,
  text_size_pane_title_px: 20,
  text_size_group_heading_px: 20,
  text_size_folder_name_px: 14,
  text_size_track_title_px: 14,
    ...fields,
  } as UiState;
}

/**
 What the fake session recorded and how it answers.
 */
type Fake = {
  readonly calls: string[];
  readonly session: Session;
  readonly setLast: (next: UiState,) => void;
  readonly setUsed: (count: number,) => void;
};

/**
 Builds a fake session. Every key press is recorded; a press moves the focus to the next label in `focusScript`.
 
 @param focusScript - focus labels that successive Tab taps reach
 
 @returns the fake and its controls
 */
function fakeSession(focusScript: readonly string[] = [],): Fake {
  const calls: string[] = [];
  const holder = { last: state(), used: 0, tabs: 0, seq: 10, };
  const session = {
    last: async () => holder.last,
    seq: async () => holder.seq,
    used: async () => holder.used,
    waitState: async ({ predicate, what, after, }: { predicate: (s: UiState,) => boolean; what: string; after?: number; },) => {
      calls.push(`wait ${what} after ${after}`,);
      if (!predicate(holder.last,))
        throw new Error(`timed out waiting for ${what}`,);
      return holder.last;
    },
    tap: async (key: string,) => {
      calls.push(`tap ${key}`,);
      if (key === 'escape')
        holder.last = { ...holder.last, menu_open: false, seq: holder.seq++, };
      if (key === 'tab') {
        holder.last = { ...holder.last, focus: focusScript[holder.tabs] ?? holder.last.focus, seq: holder.seq++, };
        holder.tabs++;
      }
    },
    down: async (key: string,) => {
      calls.push(`down ${key}`,);
    },
    up: async (key: string,) => {
      calls.push(`up ${key}`,);
    },
    chord: async ({ mods, key, }: { mods: readonly string[]; key: string; },) => {
      calls.push(`chord ${mods.join('+',)}+${key}`,);
    },
    click: async ({ x, y, button, }: { x: number; y: number; button?: string; },) => {
      calls.push(`click ${x} ${y} ${button ?? 'left'}`,);
    },
    type: async (text: string,) => {
      calls.push(`type ${text}`,);
    },
  } as unknown as Session;
  return {
    calls,
    session,
    setLast: (next,) => {
      holder.last = next;
    },
    setUsed: (count,) => {
      holder.used = count;
    },
  };
}

await describe({
  name: 'actions',
  children: [
    it({
      name: 'send the key presses they name to the session',
      fn: async ({ expect, }) => {
        const { calls, session, } = fakeSession();
        await tapKey('tab',)(session,);
        await holdKey('shift',)(session,);
        await releaseKey('shift',)(session,);
        await chordKeys({ mods: ['ctrl', 'alt',], key: 's', },)(session,);
        await clickAt({ x: 1, y: 2, },)(session,);
        await clickAt({ x: 3, y: 4, button: 'right', },)(session,);
        await typeText('tone',)(session,);
        expect(calls,).toEqual([
          'tap tab',
          'down shift',
          'up shift',
          'chord ctrl+alt+s',
          'click 1 2 left',
          'click 3 4 right',
          'type tone',
        ],);
      },
    },),
    it({
      name: 'run in order through sequenceOf, including the quick double Shift',
      fn: async ({ expect, }) => {
        const { calls, session, } = fakeSession();
        await sequenceOf([tapKey('a',), tapKey('b',),],)(session,);
        await QUICK_DOUBLE_SHIFT(session,);
        expect(calls,).toEqual(['tap a', 'tap b', 'tap shift', 'tap shift',],);
      },
    },),
  ],
},);

await describe({
  name: 'conditions',
  children: [
    it({
      name: 'test the state fields they name',
      fn: async ({ expect, }) => {
        expect(
          focusIs('play',)(state({ focus: 'play', },),),
        ).toBe(true,);
        expect(
          focusIs('play',)(state({ focus: 'seek', },),),
        ).toBe(false,);
        expect(
          routeIs('Search',)(state({ route: 'Search', },),),
        ).toBe(true,);
        expect(
          routeIs('Search',)(state(),),
        ).toBe(false,);
        expect(
          modeIs(2,)(state({ mode: 2, },),),
        ).toBe(true,);
        expect(
          modeIs(2,)(state(),),
        ).toBe(false,);
        expect(
          isPlaying(state({ playing: true, },),),
        ).toBe(true,);
        expect(
          isPaused(state({ playing: true, },),),
        ).toBe(false,);
        expect(
          isRingVisible(state({ ring_visible: true, },),),
        ).toBe(true,);
        expect(
          isRingHidden(state({ ring_visible: true, },),),
        ).toBe(false,);
      },
    },),
    it({
      name: 'accept a position within the tolerance and reject one outside it',
      fn: async ({ expect, }) => {
        expect(
          positionNear({ expected: 10, tolerance: 1.5, },)(state({ position: 11.4, },),),
        ).toBe(true,);
        expect(
          positionNear({ expected: 10, tolerance: 1.5, },)(state({ position: 8.4, },),),
        ).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: press.name,
  children: [
    it({
      name: 'acts, then waits for the condition after the sequence number read before the press',
      fn: async ({ expect, }) => {
        const { calls, session, setLast, } = fakeSession();
        setLast(state({ focus: 'play', },),);
        const result = await press({ session, act: tapKey('x',), until: focusIs('play',), what: 'play focus', },);
        expect(result.focus,).toBe('play',);
        expect(calls,).toEqual(['tap x', 'wait play focus after 10',],);
      },
    },),
    it({
      name: 'fails with the timeout message when the state never arrives',
      fn: async ({ expect, }) => {
        const { session, } = fakeSession();
        let caught: unknown;
        try {
          await press({ session, act: tapKey('x',), until: focusIs('play',), what: 'play focus', timeoutMs: 5, },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('timed out waiting for play focus',);
      },
    },),
  ],
},);

await describe({
  name: noEffect.name,
  children: [
    it({
      name: 'passes when no shortcut was used and the watched fields did not move',
      fn: async ({ expect, }) => {
        const { session, } = fakeSession();
        await noEffect({ session, act: tapKey('x',), watch: nothingMoved, what: 'x', settleMs: 1, },);
        expect(true,).toBe(true,);
      },
    },),
    it({
      name: 'fails when the app reports using a shortcut',
      fn: async ({ expect, }) => {
        const { session, setUsed, } = fakeSession();
        const act = async () => {
          setUsed(1,);
        };
        let caught: unknown;
        try {
          await noEffect({ session, act, watch: nothingMoved, what: 'x', settleMs: 1, },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('x: the app reported using a shortcut (1 new "shortcut used" lines)',);
      },
    },),
    it({
      name: 'fails when a watched field moved, naming both values',
      fn: async ({ expect, }) => {
        const { session, setLast, } = fakeSession();
        const act = async () => {
          setLast(state({ mode: 3, },),);
        };
        let caught: unknown;
        try {
          await noEffect({ session, act, watch: (s,) => s.mode, what: 'x', settleMs: 1, },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('x: state changed from 1 to 3',);
      },
    },),
  ],
},);

await describe({
  name: nothingMoved.name,
  children: [
    it({
      name: 'lists page, current track, playback, mode, folder and both overlays',
      fn: async ({ expect, }) => {
        expect(
          nothingMoved(state({ playing: true, mode: 2, },),),
        ).toEqual(['Player', -1, true, 2, 'F', false, false,],);
      },
    },),
  ],
},);

await describe({
  name: tabTo.name,
  children: [
    it({
      name: 'taps Tab until the label is reached',
      fn: async ({ expect, }) => {
        const { calls, session, } = fakeSession(['a', 'b', 'play',],);
        const reached = await tabTo({ session, label: 'play', },);
        expect(reached.focus,).toBe('play',);
        expect(calls.filter((call,) => call === 'tap tab',),).toHaveLength(3,);
      },
    },),
    it({
      name: 'does not press Tab when the label already has the focus',
      fn: async ({ expect, }) => {
        const { calls, session, setLast, } = fakeSession();
        setLast(state({ focus: 'play', },),);
        await tabTo({ session, label: 'play', },);
        expect(calls,).toEqual([],);
      },
    },),
    it({
      name: 'fails after the limit and names where it stopped',
      fn: async ({ expect, }) => {
        const { session, } = fakeSession(['a', 'b', 'c',],);
        let caught: unknown;
        try {
          await tabTo({ session, label: 'play', limit: 3, },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect((caught as Error).message,).toBe('Tab never reached "play"; stopped on "c"',);
      },
    },),
  ],
},);

await describe({
  name: tabSequence.name,
  children: [
    it({
      name: 'presses once per label, waits for each, and joins the labels seen',
      fn: async ({ expect, }) => {
        const { calls, session, setLast, } = fakeSession();
        setLast(state({ focus: 'a', },),);
        const walk = await tabSequence({ session, expected: ['a', 'a',], key: tapKey('tab',), },);
        expect(walk,).toBe('a > a',);
        expect(calls.filter((call,) => call === 'tap tab',),).toHaveLength(2,);
      },
    },),
  ],
},);

await describe({
  name: closeMenuIfOpen.name,
  children: [
    it({
      name: 'presses nothing and waits for nothing when the menu is already closed',
      fn: async ({ expect, }) => {
        const { calls, session, } = fakeSession();
        await closeMenuIfOpen({ session, },);
        expect(calls,).toEqual([],);
      },
    },),
    it({
      name: 'presses Escape and waits for a closed state newer than the press when the menu is open',
      fn: async ({ expect, }) => {
        const { calls, session, setLast, } = fakeSession();
        setLast(state({ menu_open: true, menu_ring: 0, },),);
        await closeMenuIfOpen({ session, },);
        expect(calls,).toEqual([
          'tap escape',
          'wait Escape closing the track menu a check before left open after 10',
        ],);
      },
    },),
  ],
},);

await describe({
  name: menuHasRing.name,
  children: [
    it({
      name: 'is true for an open menu with a ring row',
      fn: async ({ expect, }) => {
        expect(
          menuHasRing(state({ menu_open: true, menu_ring: 0, },),),
        ).toBe(true,);
      },
    },),
    it({
      name: 'is false for an open menu without a ring row',
      fn: async ({ expect, }) => {
        expect(
          menuHasRing(state({ menu_open: true, menu_ring: -1, },),),
        ).toBe(false,);
      },
    },),
    it({
      name: 'is false for a closed menu that still reports the ring of the menu it closed',
      fn: async ({ expect, }) => {
        expect(
          menuHasRing(state({ menu_open: false, menu_ring: 0, },),),
        ).toBe(false,);
      },
    },),
  ],
},);
