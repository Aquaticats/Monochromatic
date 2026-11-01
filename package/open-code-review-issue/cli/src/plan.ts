/**
 Publication-plan construction and safe preview projection.
 
 @module
 */

import { renderIssue, } from './issue-render.ts';
import type { SourceLink, } from './issue-model.ts';
import type { NormalizedInput, } from './model.ts';
import type {
  NonInteractivePreview,
  PreviewIssue,
  PublicationPlan,
} from './plan-model.ts';

/**
 Builds complete internal plan after destination preflight.
 
 @param input - Atomically validated normalized OCR input.
 
 @param repository - Canonical destination repository URL.
 
 @param needsTriageLabel - Whether destination contains existing label.
 
 @param sourceLink - Verified source repository and commit coordinates.
 
 @returns Complete deterministic Issue plan including security content.
 
 @example
 ```ts
 buildPublicationPlan({ input, repository, needsTriageLabel: true });
 ```
 */
export function buildPublicationPlan({
  input,
  repository,
  needsTriageLabel,
  sourceLink,
}: {
  readonly input: NormalizedInput;
  readonly repository: string;
  readonly needsTriageLabel: boolean;
  readonly sourceLink?: SourceLink;
},): PublicationPlan {
  return {
    repository,
    labelStrategy: needsTriageLabel
      ? 'needs-triage-label'
      : 'needs-triage-title-prefix',
    sourceReference: sourceLink === undefined ? 'plain' : 'commit-pinned',
    issues: input.findings
      .map(function renderFinding(finding,) {
        return renderIssue({
          finding,
          needsTriageLabel,
          ...(sourceLink === undefined ? {} : { sourceLink, }),
        },);
      },),
  };
}

/**
 Removes internal security marker from safe ordinary preview issue.
 
 @param issue - Rendered non-security Issue.
 
 @returns Exact preview issue shape.
 
 @example
 ```ts
 toPreviewIssue(issue);
 ```
 */
function toPreviewIssue(issue: PublicationPlan['issues'][number],): PreviewIssue {
  return {
    position: issue.position,
    ...(issue.classificationMarker === undefined
      ? {}
      : { classificationMarker: issue.classificationMarker, }),
    title: issue.title,
    body: issue.body,
    labels: issue.labels,
  };
}

/**
 Projects complete plan to security-redacted preview JSON.
 
 @param plan - Complete internal publication plan.
 
 @returns Safe preview with ordinary Issue content and security positions only.
 
 @example
 ```ts
 buildNonInteractivePreview(plan);
 ```
 */
export function buildNonInteractivePreview(plan: PublicationPlan,): NonInteractivePreview {
  /**
   Complete ordinary issues safe for exact preview.
   */
  const issues = plan.issues
    .filter(function isOrdinary(issue,): boolean {
      return !issue.security;
    },)
    .map(toPreviewIssue,);
  /**
   Security positions retained without title, body, path, or code.
   */
  const positions = plan.issues
    .filter(function isSecurity(issue,): boolean {
      return issue.security;
    },)
    .map(function issuePosition(issue,) {
      return issue.position;
    },);
  return {
    outcome: 'preview',
    repository: plan.repository,
    labelStrategy: plan.labelStrategy,
    sourceReference: plan.sourceReference,
    issues,
    security: {
      count: positions.length,
      positions,
    },
  };
}
