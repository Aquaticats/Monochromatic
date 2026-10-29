/**
 Checks of the wide session about the Settings page: opening it, keys that must do nothing there, and the Escape ladder
 out of a text field.
 
 @example
 ```ts
 import { checkSettings, } from './wide-settings.ts';
 
 await checkSettings({ context, session, },);
 ```
 
 @packageDocumentation
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  chordKeys,
  focusIs,
  QUICK_DOUBLE_SHIFT,
  routeIs,
  sequenceOf,
  tapKey,
} from './actions.ts';
import {
  isPlayerPage,
  searchFieldsWatched,
} from './conditions.ts';
import type { RunContext, } from './context.ts';
import { expect, } from './errors.ts';
import { pollUntil, } from './poll.ts';
import {
  noEffect,
  nothingMoved,
  press,
} from './press.ts';
import type { Session, } from './session.ts';
import type { UiState, } from './ui-state.ts';

/**
 How many Tab presses may be needed to reach a text field on the Settings page.
 */
const FIELD_TAB_LIMIT = 12;

/**
 Wait after each Tab press, so the focus change is logged before the zone is read.
 */
const TAB_SETTLE_MS = 150;

/**
 Wait after End moves the keyboard to the last Settings row: a move between rows writes no state line, so the key is
 waited out and the Enter press that follows proves the move happened.
 */
const END_SETTLE_MS = 250;

/**
 Settle time after the double Shift that should do nothing on Settings.
 */
const DOUBLE_SHIFT_SETTLE_MS = 700;

/**
 The shortcuts that must do nothing while the template field has the keyboard.
 */
const FIELD_BARE_KEYS = sequenceOf([
  tapKey('space',),
  tapKey('delete',),
  tapKey('home',),
  chordKeys({
    mods: ['ctrl',],
    key: 'm',
  },),
  chordKeys({
    mods: ['ctrl',],
    key: 'left',
  },),
],);

/**
 Condition: the Settings page shows.
 
 @param state - a state of the app
 
 @returns whether the page is Settings
 
 @example
 ```ts
 isSettingsPage(state,);
 ```
 */
function isSettingsPage(state: UiState,): boolean {
  return state.route === 'Settings';
}

/**
 Records the Settings checks of the wide session.
 
 @param context - the run's context
 
 @param session - the wide session, on the player page
 
 @example
 ```ts
 await checkSettings({ context, session, },);
 ```
 */
export async function checkSettings(
  {
    context,
    session,
  }: {
    readonly context: RunContext;
    readonly session: Session;
  },
): Promise<void> {
  /**
   The recorder of the run.
   */
  const { recorder, } = context;
  recorder.setGroup('settings',);
  await recorder.check({
    id: 'settings/ctrl-alt-s',
    title: 'Ctrl+Alt+S opens Settings with the list focused',
    body: async function ctrlAltSOpensSettings(): Promise<string> {
      /**
       The state with Settings open.
       */
      const state = await press({
        session,
        act: chordKeys({
          mods: [
            'ctrl',
            'alt',
          ],
          key: 's',
        },),
        until: isSettingsPage,
        what: 'Settings',
      },);
      return `focus ${state.focus}`;
    },
  },);
  await recorder.verify({
    id: 'settings/ctrl-f-nothing',
    title: 'Ctrl+F does nothing in Settings',
    body: async function ctrlFDoesNothing(): Promise<void> {
      await noEffect({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'f',
        },),
        watch: nothingMoved,
        what: 'Ctrl+F in Settings',
      },);
    },
  },);
  await recorder.verify({
    id: 'settings/double-shift-nothing',
    title: 'double Shift does nothing in Settings',
    body: async function doubleShiftDoesNothing(): Promise<void> {
      await noEffect({
        session,
        act: QUICK_DOUBLE_SHIFT,
        watch: nothingMoved,
        what: 'double Shift in Settings',
        settleMs: DOUBLE_SHIFT_SETTLE_MS,
      },);
    },
  },);
  await recorder.check({
    id: 'settings/licenses-row',
    title: 'End and Enter on the Licenses row open the Licenses page, and Escape returns to Settings with the list focused',
    body: async function licensesRowOpensAndEscapeReturns(): Promise<string> {
      await session.tap('end',);
      await wait(END_SETTLE_MS,);
      /**
       The state with the Licenses page open.
       */
      const opened = await press({
        session,
        act: tapKey('enter',),
        until: routeIs('Licenses',),
        what: 'the Licenses page',
      },);
      /**
       The state after Escape on the Licenses page.
       */
      const returned = await press({
        session,
        act: tapKey('escape',),
        until: function settingsListHasTheKeyboard(state: UiState,): boolean {
          return isSettingsPage(state,) && (state.zone === 'settings-list');
        },
        what: 'Settings with the list focused after Escape on the Licenses page',
      },);
      return `focus ${opened.focus} (${opened.zone}) on the page, then ${returned.focus} on ${returned.route}`;
    },
  },);
  await recorder.check({
    id: 'settings/escape-ladder',
    title:
      'template field: the first Escape returns to the list row, the second closes Settings; bare keys do nothing in the field',
    body: async function escapeLadderLeavesField(): Promise<string> {
      /**
       Presses Tab and reports whether a text field has the keyboard.
       
       @returns whether the zone is a text field
       */
      async function tabTowardField(): Promise<boolean> {
        await session.tap('tab',);
        await wait(TAB_SETTLE_MS,);
        return (await session.last()).zone === 'text-field';
      }

      /**
       Whether Tab reached a text field in time.
       */
      const reached = await pollUntil({
        attempts: FIELD_TAB_LIMIT,
        intervalMs: 0,
        order: 'check-then-wait',
        check: tabTowardField,
      },);

      /**
       The state with the field focused, or wherever Tab stopped.
       */
      const atField = await session.last();
      expect({
        condition: reached,
        message: `Tab never reached a text field in Settings; stopped on ${atField.focus} (${atField.zone})`,
      },);

      /**
       The focus label of the field.
       */
      const field = atField.focus;
      await noEffect({
        session,
        act: FIELD_BARE_KEYS,
        watch: searchFieldsWatched,
        what: 'bare keys in the template field',
      },);

      /**
       The state after the first Escape.
       */
      const first = await press({
        session,
        act: tapKey('escape',),
        until: function leftTheField(state: UiState,): boolean {
          return (state.zone === 'settings-list') || (state.focus !== field);
        },
        what: 'the first Escape',
      },);
      expect({
        condition: first.route === 'Settings',
        message: `the first Escape left Settings: ${first.route}`,
      },);

      /**
       The state after the second Escape.
       */
      const second = await press({
        session,
        act: tapKey('escape',),
        until: isPlayerPage,
        what: 'the second Escape',
      },);
      return `field "${field}" > "${first.focus}" (${first.zone}) > page ${second.route}`;
    },
  },);
  await recorder.verify({
    id: 'settings/screens-after',
    title: 'the player page is back with focus restored for keys',
    body: async function playerPageIsBack(): Promise<void> {
      await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: '2',
        },),
        until: focusIs('track-list',),
        what: 'the track list',
      },);
    },
  },);
}
