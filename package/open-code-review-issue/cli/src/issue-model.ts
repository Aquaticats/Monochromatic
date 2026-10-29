/**
 Planned GitHub Issue contracts.
 
 @module
 */

import type { InputPosition, } from './model.ts';

/**
 Explicit ordinary classification marker for interactive and preview surfaces.
 */
export type ClassificationMarker = 'OTHER' | 'UNCATEGORIZED';

/**
 Verified destination source-link coordinates.
 
 @example
 ```ts
 const sourceLink: SourceLink = {
   repository: 'owner/repository',
   commit: '0123456789abcdef0123456789abcdef01234567',
 };
 ```
 */
export type SourceLink = {
  readonly repository: string;
  readonly commit: string;
};

/**
 Complete create-only Issue request after deterministic rendering.
 
 @example
 ```ts
 const issue: RenderedIssue = {
   position: { kind: 'record', value: 1 },
   security: false,
   title: '[bug] src/a.ts: Correct the branch',
   body: '## Finding',
   labels: ['needs-triage'],
 };
 ```
 */
export type RenderedIssue = {
  readonly position: InputPosition;
  readonly security: boolean;
  readonly classificationMarker?: ClassificationMarker;
  readonly title: string;
  readonly body: string;
  readonly labels: readonly string[];
};
