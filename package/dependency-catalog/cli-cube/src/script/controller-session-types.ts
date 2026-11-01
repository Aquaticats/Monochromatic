/**
 Mutable browser-controller session shape.
 
 @module
 */

import type {
  Deck,
  OrbitView,
} from '@deck.gl/core';

import type { SceneBounds, } from '../deck-config.ts';
import type { ChromeColors, } from './scheme.ts';
import type { AppState, } from './state.ts';

/**
 Mutable working state of the controller. The owning binding remains constant.
 */
export type Session = {
  state: AppState;
  bounds: SceneBounds;
  visibleIndices: ReadonlySet<number>;
  chrome: ChromeColors;
  deck: Deck<OrbitView>;
};
