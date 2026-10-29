/**
 Checks of the wide session about the Search button tooltip, the Search page keys, and double Shift.
 
 @example
 ```ts
 import { checkSearchAndTooltip, } from './wide-search.ts';
 
 await checkSearchAndTooltip({ context, session, },);
 ```
 
 @packageDocumentation
 */

import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  chordKeys,
  type Condition,
  focusIs,
  routeIs,
  sequenceOf,
  tapKey,
  typeText,
} from './actions.ts';
import {
  hasResultRing,
  isPlayerPage,
  isSearchWithField,
  queryHasTone,
  resultOpened,
  searchFieldsWatched,
} from './conditions.ts';
import type { RunContext, } from './context.ts';
import { expect, } from './errors.ts';
import {
  diffBox,
  loadImage,
} from './image.ts';
import { WIDE_POINTS, } from './layout.ts';
import {
  noEffect,
  press,
} from './press.ts';
import type { Session, } from './session.ts';
import {
  NO_INDEX,
  type UiState,
} from './ui-state.ts';

/**
 How long the pointer rests on the Search button before the tooltip shows.
 */
const TOOLTIP_DELAY_MS = 1_800;

/**
 Fewest differing pixels that prove a tooltip appeared.
 */
const MIN_TOOLTIP_PIXELS = 300;

/**
 Narrowest bounding box that a tooltip makes, in pixels.
 */
const MIN_TOOLTIP_WIDTH = 60;

/**
 How long the first search result gets to show.
 */
const RESULT_TIMEOUT_MS = 8_000;

/**
 The shortcuts that must do nothing while a text field has the keyboard.
 */
const BARE_KEYS = sequenceOf([
  tapKey('space',),
  tapKey('home',),
  tapKey('end',),
  tapKey('delete',),
  tapKey('left',),
  tapKey('right',),
  chordKeys({
    mods: ['ctrl',],
    key: 'left',
  },),
  chordKeys({
    mods: ['ctrl',],
    key: 'right',
  },),
  chordKeys({
    mods: ['ctrl',],
    key: 'm',
  },),
  chordKeys({
    mods: ['ctrl',],
    key: 'home',
  },),
  chordKeys({
    mods: [
      'ctrl',
      'shift',
    ],
    key: '3',
  },),
],);

/**
 Condition builder: the Search query is exactly the given text.
 
 @param text - the query the field must hold
 
 @returns the condition
 
 @example
 ```ts
 queryIs('Björk',);
 ```
 */
function queryIs(text: string,): Condition {
  return function hasQuery(state: UiState,): boolean {
    return state.search_query === text;
  };
}

/**
 Records the tooltip, Search and double Shift checks of the wide session.
 
 @param context - the run's context
 
 @param session - the wide session, on the player page
 
 @example
 ```ts
 await checkSearchAndTooltip({ context, session, },);
 ```
 */
export async function checkSearchAndTooltip(
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
  recorder.setGroup('tooltip',);
  await recorder.check({
    id: 'tooltip/search-shows-shortcut',
    title: 'hovering the Search button shows a tooltip that names Ctrl+F',
    body: async function hoverShowsTooltip(): Promise<string> {
      /**
       Screenshot before the pointer arrives.
       */
      const plain = await loadImage(await session.shot('tooltip-before',),);
      await session.ok(`move ${WIDE_POINTS.searchButton
        .x} ${WIDE_POINTS.searchButton
          .y}`,);
      await wait(TOOLTIP_DELAY_MS,);

      /**
       Screenshot file with the tooltip showing.
       */
      const file = join(
        session.shots,
        `${session.label}-tooltip-search.png`,
      );
      await session.ok(`screenshot ${file}`,);

      /**
       How the screenshots differ.
       */
      const delta = diffBox({
        a: plain,
        b: await loadImage(file,),
      },);
      expect({
        condition: (delta.box !== undefined) && (delta.box
          .width
          > MIN_TOOLTIP_WIDTH)
          && (delta.count > MIN_TOOLTIP_PIXELS),
        message: `no tooltip-sized change appeared (${delta.count} pixels, ${JSON.stringify(delta.box,)})`,
      },);
      await session.ok('move 2 2',);
      return `${delta.count} pixels changed, in ${JSON.stringify(delta.box,)}; the text is in ${file}`;
    },
  },);

  recorder.setGroup('search',);
  await recorder.check({
    id: 'search/ctrl-f',
    title: 'Ctrl+F opens Search with the query field focused',
    body: async function ctrlFOpensSearch(): Promise<string> {
      await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: '1',
        },),
        until: focusIs('folder-list',),
        what: 'the folder list',
      },);

      /**
       The state with Search open and its field focused.
       */
      const state = await press({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'f',
        },),
        until: isSearchWithField,
        what: 'the Search page with the query field focused',
      },);
      return `focus ${state.focus}`;
    },
  },);
  await recorder.check({
    id: 'search/bare-keys-are-text',
    title: 'bare keys do nothing as shortcuts in the Search field, and typing works',
    body: async function bareKeysAreText(): Promise<string> {
      /**
       Whether playback ran before the keys.
       */
      const playingBefore = (await session.last()).playing;
      await noEffect({
        session,
        act: BARE_KEYS,
        watch: searchFieldsWatched,
        what: 'bare keys in the Search field',
      },);
      expect({
        condition: (await session.last()).playing === playingBefore,
        message: 'playback changed',
      },);
      await press({
        session,
        act: typeText('tone',),
        until: queryHasTone,
        what: 'the typed query',
      },);
      return `query "${(await session.last()).search_query}"`;
    },
  },);
  await recorder.check({
    id: 'search/down-up-enter',
    title: 'Down and Up move through results, Enter opens one and leaves Search',
    body: async function arrowsMoveThroughResults(): Promise<string> {
      /**
       The state with the first result highlighted.
       */
      const first = await session.waitState({
        predicate: hasResultRing,
        what: 'a highlighted result',
        timeoutMs: RESULT_TIMEOUT_MS,
      },);

      /**
       The state after Down.
       */
      const down = await press({
        session,
        act: tapKey('down',),
        until: function ringMovedDown(state: UiState,): boolean {
          return (state.search_ring !== NO_INDEX) && (state.search_ring > first.search_ring);
        },
        what: 'Down',
      },);

      /**
       The state after Up.
       */
      const up = await press({
        session,
        act: tapKey('up',),
        until: function ringBackOnFirst(state: UiState,): boolean {
          return state.search_ring === first.search_ring;
        },
        what: 'Up',
      },);

      /**
       The state after Enter opened a result.
       */
      const opened = await press({
        session,
        act: tapKey('enter',),
        until: resultOpened,
        what: 'the result opened',
      },);
      return `ring ${first.search_ring} > ${down.search_ring} > ${up.search_ring}; opened index ${opened.current_index}`;
    },
  },);
  await recorder.verify({
    id: 'search/escape',
    title: 'Escape leaves Search',
    body: async function escapeLeavesSearch(): Promise<void> {
      await press({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'f',
        },),
        until: routeIs('Search',),
        what: 'Search',
      },);
      await press({
        session,
        act: tapKey('escape',),
        until: isPlayerPage,
        what: 'the player page',
      },);
    },
  },);
  await recorder.check({
    id: 'search/unicode-text',
    title: 'typing Björk, かめりあ and Hangul with an emoji puts exactly that text in the Search field',
    body: async function unicodeTextReachesField(): Promise<string> {
      await press({
        session,
        act: chordKeys({
          mods: ['ctrl',],
          key: 'f',
        },),
        until: isSearchWithField,
        what: 'the Search page with the query field focused',
      },);

      /**
       The words typed in turn; each replaces the whole query.
       */
      const words = [
        'Björk',
        'かめりあ',
        '한국어😀',
      ];
      for (const word of words) {
        // oxlint-disable-next-line eslint/no-await-in-loop -- each word replaces the one before it
        await session.chord({
          mods: ['ctrl',],
          key: 'a',
        },);
        // oxlint-disable-next-line eslint/no-await-in-loop -- each word replaces the one before it
        await press({
          session,
          act: typeText(word,),
          until: queryIs(word,),
          what: `the query ${word}`,
        },);
      }
      await press({
        session,
        act: tapKey('escape',),
        until: isPlayerPage,
        what: 'the player page',
      },);
      return `the field held ${words.join(', ',)} exactly`;
    },
  },);
}
