/**
 Public event-wiring surface for the deck.gl scene controller.
 
 Implementation lives in smaller sibling modules so each control-surface group
 stays below the linted max-lines limit.
 
 @example
 ```ts
 import { wireDimDropdowns, wireToggles } from './controller-events.ts';
 wireDimDropdowns({ session, probes, commit });
 wireToggles({ session, commit });
 ```
 */

export {
  wireDisplay,
  wireReset,
} from './controller-display-events.ts';
export type {
  Commit,
  Session,
} from './controller-event-types.ts';
export {
  wireDimDropdowns,
  wireRanges,
} from './controller-range-events.ts';
export {
  wireSearch,
  wireToggles,
} from './controller-toggle-events.ts';
