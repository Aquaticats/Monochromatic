/**
 Machine-readable applied-run result contracts.
 
 @module
 */

import type { InputPosition, } from './model.ts';
import type { CreatedIssue, } from './publisher-model.ts';

/**
 Positioned or preflight failure safe for machine output.
 */
export type AppliedFailure = {
  readonly message: string;
  readonly position?: InputPosition;
  readonly matchingUrls?: readonly string[];
};

/**
 Final standard-output object for non-interactive applied run.
 */
export type AppliedResult = {
  readonly outcome: 'success' | 'failed' | 'interrupted';
  readonly repository: string;
  readonly created: readonly CreatedIssue[];
  readonly withheldSecurityPositions: readonly InputPosition[];
  readonly failure?: AppliedFailure;
};
