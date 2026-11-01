/**
 Checks of the wide session about where the track menu opens and how wide it is: a right click opens it at the pointer
 in the two-pane layout (it opened one pane width to the left of the pointer), and it is wide enough for its longest
 shortcut hint (the fixed 280 px cut the longest shortcut hint at the card edge).
 
 @example
 ```ts
 import { checkMenuPlacement, } from './wide-menu.ts';
 
 await checkMenuPlacement({ context, session, },);
 ```
 
 @packageDocumentation
 */

import { tapKey, } from './actions.ts';
import {
  isMenuClosed,
  isMenuOpen,
} from './conditions.ts';
import type { RunContext, } from './context.ts';
import { expect, } from './errors.ts';
import { WIDE_WINDOW, } from './layout.ts';
import { press, } from './press.ts';
import type { Session, } from './session.ts';

/**
 A window position in pixels.
 */
type Point = {
  readonly x: number;
  readonly y: number;
};

/**
 Points of the right-pane rows where the menu is requested: inside the pane near its left edge, near its middle, and
 near the window edge, where the card has to be pulled back to stay inside the window. Each is on the first track row.
 */
const MENU_REQUEST_POINTS: readonly [
  Point,
  Point,
  Point,
] = [
  {
    x: 520,
    y: 80,
  },
  {
    x: 700,
    y: 80,
  },
  {
    x: 990,
    y: 80,
  },
];

/**
 How far, in pixels, the menu card may sit from the pointer (or from the window edge when it is pulled back).
 */
const MENU_POSITION_TOLERANCE = 1;

/**
 The width the menu had when its longest shortcut hint was cut, in pixels.
 */
const MENU_NARROWEST_WIDTH = 280;

/**
 How long the track menu gets to open after the right click.
 */
const MENU_OPEN_TIMEOUT_MS = 4_000;

/**
 Right-clicks one point, proves the menu card opened there (or against the right window edge when it would not fit), and
 closes the menu.
 
 @param session - the wide session, on the player page
 
 @param point - window position of the click, in pixels
 
 @param shotName - name of a screenshot to take while the menu is open; none when absent
 
 @returns a sentence about where the card opened
 
 @throws CheckFailureError when the menu does not open, or opens away from the pointer
 
 @example
 ```ts
 await menuOpensAt({ session, point: { x: 520, y: 80, }, },);
 ```
 */
async function menuOpensAt(
  {
    session,
    point,
    shotName,
  }: {
    readonly point: Point;
    readonly session: Session;
    readonly shotName?: string;
  },
): Promise<string> {
  /**
   Sequence number of the latest state before the click, so a menu that was open before it is not taken for this one.
   */
  const after = await session.seq();
  await session.click({
    ...point,
    button: 'right',
  },);

  /**
   The state with the menu open.
   */
  const open = await session.waitState({
    predicate: isMenuOpen,
    what: 'the track menu opening',
    after,
    timeoutMs: MENU_OPEN_TIMEOUT_MS,
  },);

  /**
   Where the card belongs: at the pointer, or against the right window edge when it would not fit.
   */
  const expectedX = Math.min(
    point.x,
    WIDE_WINDOW.width - open.menu_width,
  );
  expect({
    condition: Math.abs(open.menu_x - expectedX,) <= MENU_POSITION_TOLERANCE,
    message: `the menu opened at x ${open.menu_x} for a click at x ${point.x}; it belongs at ${expectedX}`,
  },);
  expect({
    condition: Math.abs(open.menu_y - point.y,) <= MENU_POSITION_TOLERANCE,
    message: `the menu opened at y ${open.menu_y} for a click at y ${point.y}`,
  },);
  if (shotName !== undefined)
    await session.shot(shotName,);
  await press({
    session,
    act: tapKey('escape',),
    until: isMenuClosed,
    what: 'Escape closing the menu',
  },);
  return `click ${point.x},${point.y} gave a card at ${open.menu_x},${open.menu_y} (${open.menu_width} wide)`;
}

/**
 Records the menu placement checks of the wide session.
 
 @param context - the run's context
 
 @param session - the wide session, on the player page with a folder shown
 
 @example
 ```ts
 await checkMenuPlacement({ context, session, },);
 ```
 */
export async function checkMenuPlacement(
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
  recorder.setGroup('menu',);
  await recorder.check({
    id: 'menu/opens-at-pointer',
    title: 'a right click opens the track menu at the pointer (pulled back only to stay inside the window), in the two-pane layout',
    body: async function menuOpensAtPointer(): Promise<string> {
      /**
       What each request showed, as text, one entry per point.
       */
      const notes = [
        await menuOpensAt({
          session,
          point: MENU_REQUEST_POINTS[0],
          shotName: 'menu-at-pointer',
        },),
        await menuOpensAt({
          session,
          point: MENU_REQUEST_POINTS[1],
        },),
        await menuOpensAt({
          session,
          point: MENU_REQUEST_POINTS[2],
        },),
      ];
      return notes.join('; ',);
    },
  },);
  await recorder.check({
    id: 'menu/shortcut-hints-fit',
    title: 'the track menu is wider than 280 px, so the longest shortcut hint is not cut at the card edge',
    body: async function menuFitsItsHints(): Promise<string> {
      /**
       Sequence number of the latest state before the click.
       */
      const after = await session.seq();
      await session.click({
        ...MENU_REQUEST_POINTS[0],
        button: 'right',
      },);

      /**
       The state with the menu open.
       */
      const open = await session.waitState({
        predicate: isMenuOpen,
        what: 'the track menu opening',
        after,
        timeoutMs: MENU_OPEN_TIMEOUT_MS,
      },);
      expect({
        condition: open.menu_width > MENU_NARROWEST_WIDTH,
        message: `the menu is ${open.menu_width} px wide, no wider than the fixed ${MENU_NARROWEST_WIDTH} px that cut the longest shortcut hint`,
      },);
      await session.shot('menu-hints',);
      await press({
        session,
        act: tapKey('escape',),
        until: isMenuClosed,
        what: 'Escape closing the menu',
      },);
      return `${open.menu_width} px wide`;
    },
  },);
}
