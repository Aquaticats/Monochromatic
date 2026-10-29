/**
 The menu check of the ring session: the track menu opened with the Menu key is the pure canvas color of the scheme with an edge,
 and the folder list and the track list show their rings.
 
 @example
 ```ts
 import { checkMenuCard, } from './ring-menu.ts';
 
 await checkMenuCard({ context, session, scheme: 'dark', accent: '3B82F6', },);
 ```
 
 @packageDocumentation
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  chordKeys,
  focusIs,
  tapKey,
} from './actions.ts';
import {
  isMenuClosed,
  menuHasRing,
} from './conditions.ts';
import type { RunContext, } from './context.ts';
import { expect, } from './errors.ts';
import {
  type Color,
  colorAt,
  loadImage,
  sameColor,
} from './image.ts';
import { press, } from './press.ts';
import type { Session, } from './session.ts';

/**
 Time given to a page to settle after a key, in milliseconds.
 */
const SETTLE_MS = 500;

/**
 The canvas color of the dark scheme: pure black.
 */
const CANVAS_DARK: Color = [
  0,
  0,
  0,
];

/**
 The canvas color of the light scheme: pure white.
 */
const CANVAS_LIGHT: Color = [
  255,
  255,
  255,
];

/**
 How far inside the card's top right corner the canvas color is read, in pixels: past the 12 px corner and the outline.
 */
const CARD_PROBE_INSET = 14;

/**
 Records the menu check of one visual session.
 
 @param context - the run's context
 
 @param session - the running session, on the player page
 
 @param scheme - color scheme of the window
 
 @param accent - accent color without the leading hash
 
 @example
 ```ts
 await checkMenuCard({ context, session, scheme: 'dark', accent: '3B82F6', },);
 ```
 */
export async function checkMenuCard(
  {
    context,
    session,
    scheme,
    accent,
  }: {
    readonly accent: string;
    readonly context: RunContext;
    readonly scheme: 'dark' | 'light';
    readonly session: Session;
  },
): Promise<void> {
  /**
   The recorder of the run.
   */
  const { recorder, } = context;
  await recorder.check({
    id: `visual/menu-${scheme}-${accent}`,
    title: `the track menu in ${scheme} with accent #${accent} is the pure canvas color with an edge, and its row has a ring`,
    body: async function menuIsCanvasWithRing(): Promise<string> {
      await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: '1',
        },),
        until: focusIs('folder-list',),
        what: 'the folder list',
      },);
      await session.shot('0-folder-row',);
      await press({
        session,
        act: chordKeys({
          mods: ['alt',],
          key: '2',
        },),
        until: focusIs('track-list',),
        what: 'the track list',
      },);
      await session.shot('0-track-row',);
      await press({
        session,
        act: tapKey('menu',),
        until: menuHasRing,
        what: 'the track menu with a ringed row',
      },);
      await wait(SETTLE_MS,);

      /**
       Where the menu card is, from the app's own state.
       */
      const state = await session.last();

      /**
       Screenshot of the open menu with its first row ringed.
       */
      const menu = await loadImage(await session.shot('0-menu',),);

      /**
       The canvas color of the scheme: pure white in light and pure black in dark (D119).
       */
      const canvas = scheme === 'dark' ? CANVAS_DARK : CANVAS_LIGHT;

      /**
       A point of the card's heading row, right of its text, which is empty card.
       */
      const probe = colorAt({
        image: menu,
        x: Math.round(state.menu_x + state.menu_width,) - CARD_PROBE_INSET,
        y: Math.round(state.menu_y,) + CARD_PROBE_INSET,
      },);
      expect({
        condition: sameColor({
          a: probe,
          b: canvas,
        },),
        message: `the menu card is ${probe.join(',',)}, not the canvas color ${canvas.join(',',)}`,
      },);

      /**
       The card's left edge in the middle of the card, where the outline is.
       */
      const edge = colorAt({
        image: menu,
        x: Math.round(state.menu_x,),
        y: Math.round(state.menu_y + (state.menu_height / 2),),
      },);
      expect({
        condition: !sameColor({
          a: edge,
          b: canvas,
        },),
        message: `the card has no edge: its left column is ${edge.join(',',)}, the canvas color`,
      },);
      await press({
        session,
        act: tapKey('escape',),
        until: isMenuClosed,
        what: 'the menu closing',
      },);
      return `card ${probe.join(',',)}, edge ${edge.join(',',)}`;
    },
  },);
}
