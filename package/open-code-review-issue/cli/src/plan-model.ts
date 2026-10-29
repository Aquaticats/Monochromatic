/**
 Publication and preview plan contracts.
 
 @module
 */

import type { InputPosition, } from './model.ts';
import type { RenderedIssue, } from './issue-model.ts';

/**
 Existing-label or title-prefix behavior selected by preflight.
 */
export type LabelStrategy = 'needs-triage-label' | 'needs-triage-title-prefix';

/**
 Plain or commit-pinned source-reference behavior selected by preflight.
 */
export type SourceReferenceStrategy = 'plain' | 'commit-pinned';

/**
 Complete internal plan retaining all issue content.
 
 @example
 ```ts
 const plan: PublicationPlan = {
   repository: 'https://github.com/owner/repo',
   labelStrategy: 'needs-triage-label',
   sourceReference: 'plain',
   issues: [],
 };
 ```
 */
export type PublicationPlan = {
  readonly repository: string;
  readonly labelStrategy: LabelStrategy;
  readonly sourceReference: SourceReferenceStrategy;
  readonly issues: readonly RenderedIssue[];
};

/**
 Non-security issue representation emitted in preview JSON.
 */
export type PreviewIssue = Omit<RenderedIssue, 'security'>;

/**
 Security summary that exposes positions but no finding content.
 */
export type SecurityPreview = {
  readonly count: number;
  readonly positions: readonly InputPosition[];
};

/**
 Exact machine-readable non-interactive preview.
 */
export type NonInteractivePreview = {
  readonly outcome: 'preview';
  readonly repository: string;
  readonly labelStrategy: LabelStrategy;
  readonly sourceReference: SourceReferenceStrategy;
  readonly issues: readonly PreviewIssue[];
  readonly security: SecurityPreview;
};


/**
 Explicit non-interactive publication authority.
 */
export type ApplyAuthority = 'default' | 'non-security-only' | 'all';

/**
 Issues authorized for creation plus redacted withheld positions.
 */
export type ApplySelection = {
  readonly issues: PublicationPlan['issues'];
  readonly withheldPositions: readonly InputPosition[];
};
