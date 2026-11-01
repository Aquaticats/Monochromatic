/**
 The ring session: the focus ring photographed with and without the ring on a pill, a circle and a rounded row, in one color
 scheme and accent, and proved to change only the ring.
 
 @example
 ```ts
 import { runRingSession, } from './ring-session.ts';
 
 await runRingSession({ context, scheme: 'dark', accent: '3B82F6', },);
 ```
 
 @packageDocumentation
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  clickAt,
  isRingHidden,
  isRingVisible,
  routeIs,
  tapKey,
} from './actions.ts';
import {
  type RunContext,
  startContextSession,
} from './context.ts';
import { expect, } from './errors.ts';
import {
  type Box,
  type Color,
  colorAt,
  countColor,
  diffBoxWithin,
  differencesWithin,
  type Image,
  loadImage,
} from './image.ts';
import {
  WIDE_POINTS,
  WIDE_WINDOW,
} from './layout.ts';
import {
  press,
  tabTo,
} from './press.ts';
import type { Session, } from './session.ts';
import { checkMenuCard, } from './ring-menu.ts';
import { SESSION_READY_MS, } from './timing.ts';
import type { UiState, } from './ui-state.ts';

/**
 Time given to a page to settle after a key or a pointer press, in milliseconds.
 */
const SETTLE_MS = 500;

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
 The Settings button of the wide window, where a pointer press opens Settings without turning the keyboard ring on.
 */
const SETTINGS_BUTTON = {
  x: 968,
  y: 28,
} as const;

/**
 The first template row of the wide Settings page, where a pointer press selects the row that is already selected.
 */
const SETTINGS_FIRST_ROW = {
  x: 180,
  y: 144,
} as const;

/**
 Where the Open folder pill and its ring are, in pixels of the wide window: the ring is searched for only here, so a change
 elsewhere on the page (a pointer press that selects a folder) never counts as part of it.
 */
const PILL_AREA: Box = {
  x: 330,
  y: 0,
  width: 180,
  height: 60,
};

/**
 Where the Play circle and its ring are, in pixels of the wide window.
 */
const PLAY_AREA: Box = {
  x: 210,
  y: 670,
  width: 80,
  height: 70,
};

/**
 Where the first Settings row and its ring are, in pixels of the wide window.
 */
const ROW_AREA: Box = {
  x: 0,
  y: 100,
  width: 364,
  height: 90,
};

/**
 A point at the bottom right corner of the window, where nothing is, so no hover layer shows anywhere.
 */
const IDLE_POINTER = {
  x: 995,
  y: 795,
} as const;

/**
 Pixels left between the element's edge and the rectangle that is compared for change inside the element, so the anti-aliased
 edge of the element is not part of the comparison.
 */
const EDGE_MARGIN = 1;

/**
 How far the ring reaches outside its element, in pixels: the 2 px near ring and the 1 px outer ring.
 */
const RING_REACH = 3;

/**
 Share of a corner radius that a rectangle must be inset to lie inside the rounded corner: one minus the cosine of 45 degrees.
 */
const CORNER_INSET_SHARE = 0.2929;

/**
 Most pixels a ring may change, per pixel of its outline: the 2 px near ring and the 1 px outer ring are 3 px wide, and a
 fill would change far more.
 */
const MAX_CHANGED_PER_OUTLINE_PIXEL = 3.5;

/**
 The other focus stops of the player page, each photographed with its ring: the seek bar, the transport circles, the mode
 group and the two header buttons.
 */
const OTHER_RING_STOPS = [
  'seek',
  'previous',
  'next',
  'mode-group',
  'search',
  'settings',
] as const;

/**
 How far below the top of the Play ring the primary role is sampled, in pixels: past the ring's 3 px and above the glyph.
 */
const PRIMARY_SAMPLE_DEPTH = 8;

/**
 Turns the keyboard ring off with a pointer press on a control whose press changes nothing, moves the pointer to an empty
 corner, and photographs the page. The page is then the one with the ring, less the ring.
 
 @param session - the running session
 
 @param point - the control to press
 
 @param name - screenshot name
 
 @returns the decoded screenshot
 
 @example
 ```ts
 const plain = await ringOff({ session, point: WIDE_POINTS.folderRow, name: 'plain', },);
 ```
 */
async function ringOff(
  {
    session,
    point,
    name,
  }: {
    readonly name: string;
    readonly point: {
      readonly x: number;
      readonly y: number;
    };
    readonly session: Session;
  },
): Promise<Image> {
  await press({
    session,
    act: clickAt(point,),
    until: isRingHidden,
    what: 'the ring turning off after a pointer press',
  },);
  await session.ok(`move ${IDLE_POINTER.x} ${IDLE_POINTER.y}`,);
  await wait(SETTLE_MS,);
  return loadImage(await session.shot(name,),);
}

/**
 Checks that a focus ring is only a ring: it changes pixels along the outline of its element and none inside the element, and its
 near ring is the `primary` role.
 
 @param plain - screenshot without the ring on the element
 
 @param ringed - screenshot of the same page with the ring on the element
 
 @param primary - the primary role's color of this scheme and accent
 
 @param radius - corner radius of the element in pixels, which sets the rectangle compared for change inside it
 
 @param what - name of the element, for the message
 
 @returns a short description of what was measured
 
 @throws CheckFailureError when the ring changed the inside of the element, drew more than a ring, or is not in the primary role
 
 @example
 ```ts
 checkRingOnly({ plain, ringed, primary: [69, 84, 129], what: 'the Play circle', },);
 ```
 */
function checkRingOnly(
  {
    area,
    plain,
    ringed,
    primary,
    radius,
    what,
  }: {
    readonly area: Box;
    readonly plain: Image;
    readonly primary: Color;
    readonly radius: number;
    readonly ringed: Image;
    readonly what: string;
  },
): string {
  /**
   Where the two screenshots differ: the ring.
   */
  const delta = diffBoxWithin({
    a: plain,
    b: ringed,
    area,
  },);
  expect({
    condition: delta.box !== undefined,
    message: `${what}: no ring was drawn`,
  },);

  /**
   The ring's bounding box.
   */
  const box = nonNullishOrThrow(delta.box,);

  /**
   Length of the ring's outline in pixels.
   */
  const outline = 2 * (box.width + box.height);
  expect({
    condition: delta.count <= (MAX_CHANGED_PER_OUTLINE_PIXEL * outline),
    message: `${what}: ${delta.count} pixels changed along an outline of ${outline}, so focus drew more than a ring`,
  },);

  /**
   The element inside the ring, away from its anti-aliased edge.
   */
  const inset = RING_REACH
    + Math.ceil(radius * CORNER_INSET_SHARE,)
    + EDGE_MARGIN;

  /**
   The rectangle inscribed in the element's rounded shape, away from its edge and from the curves the ring follows.
   */
  const inner: Box = {
    x: box.x + inset,
    y: box.y + inset,
    width: box.width - (2 * inset),
    height: box.height - (2 * inset),
  };

  /**
   How many pixels inside the element focus changed.
   */
  const inside = differencesWithin({
    a: plain,
    b: ringed,
    box: inner,
  },);
  expect({
    condition: inside === 0,
    message: `${what}: focus changed ${inside} pixels inside the element (a fill, a tint or a size change)`,
  },);

  /**
   How many pixels of the ring are exactly the primary role.
   */
  const primaryPixels = countColor({
    image: ringed,
    box,
    color: primary,
  },);
  expect({
    condition: primaryPixels >= outline,
    message: `${what}: only ${primaryPixels} ring pixels are the primary role ${primary.join(',',)}, fewer than the outline ${outline}`,
  },);
  return `${what}: ${delta.count} ring pixels, ${primaryPixels} in the primary role`;
}

/**
 Runs the ring check of one visual session: focus on a pill, a circle and a rounded row, in one color scheme and accent.
 
 @param context - the run's context
 
 @param scheme - color scheme of the window
 
 @param accent - accent color without the leading hash
 
 @throws CheckFailureError when the session cannot start
 
 @example
 ```ts
 await runRingSession({ context, scheme: 'dark', accent: '3B82F6', },);
 ```
 */
export async function runRingSession(
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
  const label = `rings-${scheme}-${accent}`;

  /**
   The running session; leaving the function stops it.
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
  await checkMenuCard({
    context,
    session,
    scheme,
    accent,
  },);
  await recorder.check({
    id: `visual/rings-${scheme}-${accent}`,
    title: `the focus ring in ${scheme} with accent #${accent} is a ring only on a pill, a circle and a rounded row, in the primary role`,
    body: async function ringOnPillCircleAndRow(): Promise<string> {
      await tabTo({
        session,
        label: 'open-pill',
      },);

      /**
       Screenshot with the ring on the Open folder pill.
       */
      const pill = await loadImage(await session.shot('1-pill',),);

      /**
       The same page with the ring turned off by a pointer press.
       */
      const pillPlain = await ringOff({
        session,
        point: WIDE_POINTS.folderRow,
        name: '1-pill-plain',
      },);
      await tabTo({
        session,
        label: 'play',
      },);

      /**
       Screenshot with the ring on the Play circle, whose fill is the primary role.
       */
      const circle = await loadImage(await session.shot('2-circle',),);

      /**
       The same page with the ring turned off.
       */
      const circlePlain = await ringOff({
        session,
        point: WIDE_POINTS.folderRow,
        name: '2-circle-plain',
      },);

      /**
       Where the Play ring is, which gives the circle to sample the primary role from.
       */
      const circleDelta = diffBoxWithin({
        a: circlePlain,
        b: circle,
        area: PLAY_AREA,
      },);
      expect({
        condition: circleDelta.box !== undefined,
        message: 'no ring on the Play circle',
      },);

      /**
       The Play ring's box.
       */
      const circleBox = nonNullishOrThrow(circleDelta.box,);

      /**
       The primary role: the Play circle's fill, a few pixels inside its top edge and above its glyph.
       */
      const primary = colorAt({
        image: circle,
        x: circleBox.x + Math.floor(circleBox.width / 2,),
        y: circleBox.y + PRIMARY_SAMPLE_DEPTH,
      },);
      for (const stop of OTHER_RING_STOPS) {
        // oxlint-disable-next-line eslint/no-await-in-loop -- each focus position is photographed after the previous one
        await tabTo({
          session,
          label: stop,
        },);
        // oxlint-disable-next-line eslint/no-await-in-loop -- see above
        await session.shot(`ring-${stop}`,);
      }
      await press({
        session,
        act: clickAt(SETTINGS_BUTTON,),
        until: routeIs('Settings',),
        what: 'Settings opened with the pointer',
      },);
      await wait(SETTLE_MS,);
      await press({
        session,
        act: tapKey('home',),
        until: isRingVisible,
        what: 'the ring on the first Settings row',
      },);
      await wait(SETTLE_MS,);

      /**
       Screenshot of Settings with the ring on the first row, which is selected, so it is a pill.
       */
      const row = await loadImage(await session.shot('3-settings-row',),);

      /**
       The same page with the ring turned off by a pointer press on that row.
       */
      const rowPlain = await ringOff({
        session,
        point: SETTINGS_FIRST_ROW,
        name: '3-settings-row-plain',
      },);
      return [
        checkRingOnly({
          plain: pillPlain,
          ringed: pill,
          area: PILL_AREA,
          primary,
          radius: 20,
          what: 'the Open folder pill',
        },),
        checkRingOnly({
          plain: circlePlain,
          ringed: circle,
          area: PLAY_AREA,
          primary,
          radius: 20,
          what: 'the Play circle',
        },),
        checkRingOnly({
          plain: rowPlain,
          ringed: row,
          area: ROW_AREA,
          primary,
          radius: 21,
          what: 'the first Settings row',
        },),
      ].join('; ',);
    },
  },);
}
