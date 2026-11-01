/**
 Shared types for browser-side controller event wiring modules.
 
 @example
 ```ts
 import type { Commit, Session } from './controller-event-types.ts';
 ```
 */

import type { SceneBounds, } from '../deck-config.ts';
import type { AppState, } from './state.ts';

//region Types

/**
 Mutable session view used by wire functions.
 
 The full controller-side session in `./controller.ts` carries additional
 fields that event modules do not touch; leaving them out keeps structural
 assignability without casts.
 */
export type Session = {
  state: AppState;
  bounds: SceneBounds;
  visibleIndices: ReadonlySet<number>;
};

/**
 Callback invoked by every wire function after a state mutation.
 */
export type Commit = () => void;

//endregion Types
