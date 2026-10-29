/**
 Checks of the wide session about the key hint badges: they appear after a held modifier, narrow as more modifiers are
 held, never flash for a quick chord, and never take pointer input.
 
 @example
 ```ts
 import { checkBadges, } from './wide-badges.ts';
 
 await checkBadges({ context, session, },);
 ```
 
 @packageDocumentation
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  chordKeys,
  clickAt,
  focusIs,
  holdKey,
  isPaused,
  releaseKey,
  routeIs,
  tapKey,
} from './actions.ts';
import { isPlayerPage, } from './conditions.ts';
import type { RunContext, } from './context.ts';
import { expect, } from './errors.ts';
import {
  diffBox,
  loadImage,
} from './image.ts';
import { WIDE_POINTS, } from './layout.ts';
import { press, } from './press.ts';
import type { Session, } from './session.ts';
import type { UiState, } from './ui-state.ts';

/**
 How long a badge change gets to show.
 */
const BADGE_TIMEOUT_MS = 2_000;

/**
 Wait that lets the app go quiet before a baseline screenshot.
 */
const QUIET_MS = 400;

/**
 Time into a held modifier at which no badge may show yet.
 */
const BEFORE_BADGES_MS = 150;

/**
 Earliest time after the press at which the badges may appear.
 */
const EARLIEST_BADGES_MS = 250;

/**
 Wait after the release, so the badges are gone before the screenshot.
 */
const AFTER_RELEASE_MS = 300;

/**
 Wait that covers a quick chord and the time the badges would need to appear.
 */
const QUICK_CHORD_WAIT_MS = 700;

/**
 Fewest differing pixels that prove the badges are drawn.
 */
const MIN_BADGE_PIXELS = 100;

/**
 Condition: at least one badge shows.
 
 @param state - a state of the app
 
 @returns whether any badge is on screen
 
 @example
 ```ts
 hasBadges(state,);
 ```
 */
function hasBadges(state: UiState,): boolean {
  return state.badges
    .length
    > 0;
}

/**
 Condition: no badge shows.
 
 @param state - a state of the app
 
 @returns whether the badges are gone
 
 @example
 ```ts
 hasNoBadges(state,);
 ```
 */
function hasNoBadges(state: UiState,): boolean {
  return state.badges
    .length
    === 0;
}

/**
 Tells whether a badge name starts with a prefix.
 
 @param prefix - the start of the badge name
 
 @returns a test for badge names
 
 @example
 ```ts
 badgeStartsWith('search=',);
 ```
 */
function badgeStartsWith(prefix: string,): (badge: string,) => boolean {
  return function startsWithPrefix(badge: string,): boolean {
    return badge.startsWith(prefix,);
  };
}

/**
 Condition factory: some badge starts with a prefix.
 
 @param prefix - the start of the badge name
 
 @returns the condition
 
 @example
 ```ts
 someBadgeStartsWith('mode-repeat',);
 ```
 */
function someBadgeStartsWith(prefix: string,): (state: UiState,) => boolean {
  return function hasBadgeWithPrefix(state: UiState,): boolean {
    return state.badges
      .some(badgeStartsWith(prefix,),);
  };
}

/**
 Records the badge checks of the wide session.
 
 @param context - the run's context
 
 @param session - the wide session, on the player page
 
 @example
 ```ts
 await checkBadges({ context, session, },);
 ```
 */
export async function checkBadges(
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
  recorder.setGroup('badges',);
  await recorder.check({
    id: 'badges/after-300ms',
    title: 'Ctrl held: no badge before 300 ms, badges after, gone on release',
    body: async function badgesAppearAfterDelay(): Promise<string> {
      // A playing track moves the time text, which would differ between two screenshots for another reason.
      if ((await session.last()).playing)
        await press({
          session,
          act: tapKey('space',),
          until: isPaused,
          what: 'pause',
        },);
      await wait(QUIET_MS,);

      /**
       Screenshot without badges.
       */
      const clean = await session.shot('badges-none',);

      /**
       Sequence number before the press.
       */
      const after = await session.seq();

      /**
       The moment of the press.
       */
      const pressedAt = Date.now();
      await session.down('ctrl',);
      await wait(BEFORE_BADGES_MS,);
      expect({
        condition: (await session.last()).badges
          .length
          === 0,
        message: `badges at 150 ms: ${(await session.last()).badges
          .join(',',)}`,
      },);

      /**
       The state with the badges showing.
       */
      const shown = await session.waitState({
        predicate: hasBadges,
        what: 'the badges appearing',
        after,
        timeoutMs: BADGE_TIMEOUT_MS,
      },);

      /**
       Milliseconds from the press to the badges.
       */
      const elapsed = Date.now() - pressedAt;
      expect({
        condition: elapsed >= EARLIEST_BADGES_MS,
        message: `the badges appeared after only ${elapsed} ms`,
      },);

      /**
       Screenshot with the badges.
       */
      const withBadges = await session.shot('badges-ctrl',);
      await session.up('ctrl',);

      /**
       The state with the badges gone.
       */
      const gone = await session.waitState({
        predicate: hasNoBadges,
        what: 'the badges going away',
        after: shown.seq,
        timeoutMs: BADGE_TIMEOUT_MS,
      },);
      await wait(AFTER_RELEASE_MS,);

      /**
       Screenshot after the release.
       */
      const released = await session.shot('badges-released',);

      /**
       The clean screenshot as pixels.
       */
      const cleanImage = await loadImage(clean,);

      /**
       How the badge screenshot differs from the clean one.
       */
      const delta = diffBox({
        a: cleanImage,
        b: await loadImage(withBadges,),
      },);

      /**
       How the screenshot after the release differs from the clean one.
       */
      const afterRelease = diffBox({
        a: cleanImage,
        b: await loadImage(released,),
      },);
      expect({
        condition: delta.count > MIN_BADGE_PIXELS,
        message: `no badge pixels (${delta.count})`,
      },);
      expect({
        condition: afterRelease.count === 0,
        message: `${afterRelease.count} pixels still differ after release, in ${JSON.stringify(afterRelease.box,)}`,
      },);
      return `shown at about ${elapsed} ms with [${shown.badges
        .join(', ',)}]; ${delta.count} pixels differ with badges, 0 after release; badges ${gone.badges
          .length}`;
    },
  },);
  await recorder.check({
    id: 'badges/narrow-with-shift',
    title: 'adding Shift narrows the badges to the Ctrl+Shift chords; releasing Shift brings the Ctrl badges back',
    body: async function shiftNarrowsBadges(): Promise<string> {
      await session.down('ctrl',);

      /**
       The state with the Ctrl badges.
       */
      const plain = await session.waitState({
        predicate: hasBadges,
        what: 'Ctrl badges',
        after: await session.seq(),
        timeoutMs: BADGE_TIMEOUT_MS,
      },);

      /**
       The state with the Ctrl+Shift badges.
       */
      const narrowed = await press({
        session,
        act: holdKey('shift',),
        until: someBadgeStartsWith('mode-repeat',),
        what: 'Ctrl+Shift badges',
      },);
      expect({
        condition: !narrowed.badges
          .some(badgeStartsWith('search=',),),
        message: 'the plain Ctrl badge for Search is still shown with Shift held',
      },);
      await session.shot('badges-ctrl-shift',);

      /**
       The state with the Ctrl badges back.
       */
      const back = await press({
        session,
        act: releaseKey('shift',),
        until: someBadgeStartsWith('search=',),
        what: 'Ctrl badges again',
      },);
      await session.up('ctrl',);
      await session.waitState({
        predicate: hasNoBadges,
        what: 'the badges going away',
        after: back.seq,
        timeoutMs: BADGE_TIMEOUT_MS,
      },);
      return `Ctrl [${plain.badges
        .join(', ',)}] > Ctrl+Shift [${narrowed.badges
          .join(', ',)}]`;
    },
  },);
  await recorder.check({
    id: 'badges/alt',
    title: 'Alt held shows the seek and focus badges',
    body: async function altShowsSeekBadges(): Promise<string> {
      await session.down('alt',);

      /**
       The state with the Alt badges.
       */
      const state = await session.waitState({
        predicate: someBadgeStartsWith('seek-back',),
        what: 'Alt badges',
        after: await session.seq(),
        timeoutMs: BADGE_TIMEOUT_MS,
      },);
      await session.up('alt',);
      await session.waitState({
        predicate: hasNoBadges,
        what: 'the badges going away',
        after: state.seq,
        timeoutMs: BADGE_TIMEOUT_MS,
      },);
      return `[${state.badges
        .join(', ',)}]`;
    },
  },);
  await recorder.verify({
    id: 'badges/quick-chord',
    title: 'a quick chord (under 300 ms) never flashes the badges',
    body: async function quickChordNeverFlashes(): Promise<void> {
      /**
       Sequence number before the chord.
       */
      const after = await session.seq();
      await session.chord({
        mods: ['ctrl',],
        key: 'o',
      },);
      await wait(QUICK_CHORD_WAIT_MS,);
      await session.last();

      /**
       States after the chord that showed badges.
       */
      const flashed = session.states()
        .filter(function showedBadges(state,): boolean {
          return (state.seq > after) && (state.badges
            .length
            > 0);
        },);
      expect({
        condition: flashed.length === 0,
        message: `badges flashed: ${
          flashed.map(function badgeList(state,): string {
            return state.badges
              .join(',',);
          },)
            .join(' | ',)
        }`,
      },);
    },
  },);
  await recorder.verify({
    id: 'badges/no-pointer-input',
    title: 'badges never take pointer input: a click through a shown badge reaches the control beneath',
    body: async function badgesLetClicksThrough(): Promise<void> {
      await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: '2',
        },),
        until: focusIs('track-list',),
        what: 'the track list',
      },);
      await session.down('ctrl',);
      await session.waitState({
        predicate: hasBadges,
        what: 'the badges',
        after: await session.seq(),
        timeoutMs: BADGE_TIMEOUT_MS,
      },);
      // The Search button is at the top right; its badge sits on it. Clicking there must open Search.
      await press({
        session,
        act: clickAt(WIDE_POINTS.searchButton,),
        until: routeIs('Search',),
        what: 'Search opening from a click through its badge',
      },);
      await session.up('ctrl',);
      await press({
        session,
        act: tapKey('escape',),
        until: isPlayerPage,
        what: 'the player page',
      },);
    },
  },);
}
