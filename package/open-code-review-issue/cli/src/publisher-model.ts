/**
 Issue publication result and scheduler contracts.
 
 @module
 */

import type { InputPosition, } from './model.ts';

/**
 Injectable asynchronous delay for deterministic retry tests.
 */
export type PublicationWait = (
  milliseconds: number,
) => Promise<void>;


/**
 Reads publication interrupt state without mutating it.
 */
export type PublicationStopCheck = () => boolean;

/**
 Confirmed created GitHub Issue identity.
 */
export type CreatedIssue = {
  readonly position: InputPosition;
  readonly number: number;
  readonly url: string;
};

/**
 Complete successful publication result.
 */
export type PublicationResult = {
  readonly created: readonly CreatedIssue[];
};
