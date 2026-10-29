/**
 The visual sessions: the focus ring and the badges drawn in light and dark, with two accents, one session each.
 
 @example
 ```ts
 import { runVisualSessions, } from './visual-session.ts';
 
 await runVisualSessions(context,);
 ```
 
 @packageDocumentation
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  chordKeys,
  focusIs,
} from './actions.ts';
import {
  type RunContext,
  startContextSession,
} from './context.ts';
import { expect, } from './errors.ts';
import {
  diffBox,
  loadImage,
} from './image.ts';
import {
  VISUAL_VARIANTS,
  WIDE_WINDOW,
} from './layout.ts';
import {
  press,
  tabTo,
} from './press.ts';
import { runRingSession, } from './ring-session.ts';
import {
  SESSION_READY_MS,
} from './timing.ts';
import type { UiState, } from './ui-state.ts';

/**
 Fewest differing pixels that prove the ring was drawn.
 */
const MIN_RING_PIXELS = 40;

/**
 How long the badges get to show.
 */
const BADGE_TIMEOUT_MS = 2_000;

/**
 Time given to the badges to narrow once Shift is held.
 */
const NARROW_WAIT_MS = 500;

/**
 Time given to the badges to go away once the keys are released.
 */
const RELEASE_WAIT_MS = 600;

/**
 Condition: a folder is shown, so the player page is up.
 
 @param state - a state of the app
 
 @returns whether a folder shows
 
 @example
 ```ts
 hasFolder(state,);
 ```
 */
function hasFolder(state: UiState,): boolean {
  return state.folder !== '';
}

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
 Runs one visual session: five screenshots, and the proof that the ring and the badges draw and the badges go away.
 
 @param context - the run's context
 
 @param scheme - color scheme of the window
 
 @param accent - accent color without the leading hash
 
 @throws CheckFailureError when the session cannot start
 
 @example
 ```ts
 await runVisualSession({ context, scheme: 'dark', accent: '3B82F6', },);
 ```
 */
export async function runVisualSession(
  {
    context,
    scheme,
    accent,
  }: {
    readonly accent: string;
    readonly context: RunContext;
    readonly scheme: 'dark' | 'light';
  },
): Promise<void> {
  /**
   The recorder of the run.
   */
  const { recorder, } = context;

  /**
   Label that names the session, its unit and its screenshots.
   */
  const label = `visual-${scheme}-${accent}`;

  /**
   The running visual session; leaving the function stops it.
   */
  await using session = await startContextSession({
    context,
    options: {
      ...WIDE_WINDOW,
      scheme,
      accent,
      library: context.library,
      label,
    },
  },);
  await session.waitState({
    predicate: hasFolder,
    what: 'the player page',
    timeoutMs: SESSION_READY_MS,
  },);
  recorder.setGroup('visual',);
  await recorder.check({
    id: `visual/${scheme}-${accent}`,
    title: `focus ring and badges in ${scheme} with accent #${accent}: ring and badges draw, and are gone after release`,
    body: async function ringAndBadgesDraw(): Promise<string> {
      /**
       Screenshot before any key.
       */
      const plain = await session.shot('1-plain',);
      await tabTo({
        session,
        label: 'play',
      },);

      /**
       Screenshot with the ring on Play.
       */
      const ring = await session.shot('2-ring-play',);
      await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: '2',
        },),
        until: focusIs('track-list',),
        what: 'the track list',
      },);
      await session.shot('3-ring-track-list',);
      await session.down('ctrl',);
      await session.waitState({
        predicate: hasBadges,
        what: 'the badges',
        after: await session.seq(),
        timeoutMs: BADGE_TIMEOUT_MS,
      },);
      await session.shot('4-badges-ctrl',);
      await session.down('shift',);
      await wait(NARROW_WAIT_MS,);
      await session.shot('5-badges-ctrl-shift',);
      await session.up('shift',);
      await session.up('ctrl',);
      await wait(RELEASE_WAIT_MS,);

      /**
       The state after the release.
       */
      const released = await session.last();
      expect({
        condition: released.badges
          .length
          === 0,
        message: `badges still showing after release: ${released.badges
          .join(', ',)}`,
      },);

      /**
       How the ring screenshot differs from the plain one.
       */
      const ringDelta = diffBox({
        a: await loadImage(plain,),
        b: await loadImage(ring,),
      },);
      expect({
        condition: ringDelta.count > MIN_RING_PIXELS,
        message: `the ring did not draw (${ringDelta.count} pixels)`,
      },);
      return `ring pixels ${ringDelta.count} in ${JSON.stringify(ringDelta.box,)}; shots in ${session.shots}/${label}-*.png`;
    },
  },);
}

/**
 Runs the visual session of every color scheme and accent.
 
 @param context - the run's context
 
 @example
 ```ts
 await runVisualSessions(context,);
 ```
 */
export async function runVisualSessions(context: RunContext,): Promise<void> {
  for (const {
    scheme,
    accent,
  } of VISUAL_VARIANTS)
    // oxlint-disable-next-line eslint/no-await-in-loop -- the sessions must not overlap: one compositor and one app at a time
    await runVisualSession({
      context,
      scheme,
      accent,
    },);
  for (const {
    scheme,
    accent,
  } of VISUAL_VARIANTS)
    // oxlint-disable-next-line eslint/no-await-in-loop -- the sessions must not overlap: one compositor and one app at a time
    await runRingSession({
      context,
      scheme,
      accent,
    },);
}
