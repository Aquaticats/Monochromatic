/**
 Shared structured model-review transport and fallback orchestration.
 
 @module
 */

//region Package exports -- public surface of the review transport

export * from './attempt.ts';
export * from './fallback.ts';
export * from './json.ts';
export {
  dispatchProviderStream,
  PROVIDER_STREAMS_BY_API,
} from './provider-streams.ts';
export type { LazyProviderStreams, } from './provider-streams.ts';
export { EmptyStructuredReviewTextError, } from './stream-collection.ts';
export * from './tool-choice.ts';
export type * from './types.ts';

//endregion Package exports -- public surface of the review transport
