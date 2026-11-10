/**
 Built entry for range regression tests, kept separate from the normal Node entry
 so importing the library does not initialize browser visualization dependencies.

 @internal
 */

export { computeSceneBounds, } from './deck-config.ts';
export type { PackageProbe, } from './probe.ts';
export { CHANNEL_KEYS, } from './script/controller-event-constants.ts';
export type { Session, } from './script/controller-event-types.ts';
export { wireDimDropdowns, } from './script/controller-range-events.ts';
export { computeRangeExtent, } from './script/range-extent.ts';
export { defaultState, } from './script/state.ts';
