/**
 Pi custom-entry renderers for human-only goal outcomes.
 
 @module
 */

import type {
  CustomEntry,
  EntryRenderOptions,
  ExtensionAPI,
  Theme,
} from '@earendil-works/pi-coding-agent';
import { Text, } from '@earendil-works/pi-tui';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  GOAL_COMPLETION_ENTRY_TYPE,
  GOAL_REVIEW_UNAVAILABLE_ENTRY_TYPE,
} from './constants.ts';
import type {
  GoalCompletionDiagnostic,
  GoalReviewUnavailableDiagnostic,
} from './types.ts';

//region Reviewer summary

/**
 Format attempted reviewer identities for human audit.
 
 @param identities - attempted canonical reviewer identities
 
 @returns comma-separated identities or explicit absence
 
 @example
 ```ts
 formatAttemptedReviewers([]);
 ```
 */
function formatAttemptedReviewers(identities: readonly string[],): string {
  return identities.length === 0
    ? 'none'
    : identities.join(', ',);
}

//endregion Reviewer summary

//region Renderer registration

/**
 Register persistent human-only outcome renderers.
 
 @param pi - Pi extension API receiving custom-entry renderers
 
 @mutates pi - pi.registerEntryRenderer stores transcript renderers
 
 @example
 ```ts
 registerGoalTerminalRenderer(pi);
 ```
 */
function registerGoalTerminalRenderer(
  pi: ForeignBorrowed<ExtensionAPI>,
): void {
  pi.registerEntryRenderer<GoalCompletionDiagnostic>(
    GOAL_COMPLETION_ENTRY_TYPE,
    function renderCompletion(
      entry: ForeignBorrowed<CustomEntry<GoalCompletionDiagnostic>>,
      options: ForeignBorrowed<EntryRenderOptions>,
      theme: ForeignBorrowed<Theme>,
    ) {
      /**
       Persisted completion audit payload.
       */
      const { data, } = entry;
      if (data === undefined)
        return undefined;
      /**
       Compact success row with optional expanded audit.
       */
      const lines = [theme.fg(
        'success',
        theme.bold('Goal complete',)
      ),];
      if (options.expanded) {
        lines.push(
          `Approval source: ${data.approvalSource}`,
          `Reviewer: ${data.reviewerIdentity}`,
          `Attempted reviewers: ${formatAttemptedReviewers(data.attemptedReviewerIdentities,)}`,
          `Evidence truncated: ${data.transcriptTruncated ? 'yes' : 'no'}`,
          data.reviewerRationale,
        );
      }
      return new Text(
        lines.join('\n',),
        1,
        0,
      );
    },
  );
  pi.registerEntryRenderer<GoalReviewUnavailableDiagnostic>(
    GOAL_REVIEW_UNAVAILABLE_ENTRY_TYPE,
    function renderReviewUnavailable(
      entry: ForeignBorrowed<CustomEntry<GoalReviewUnavailableDiagnostic>>,
      _options: ForeignBorrowed<EntryRenderOptions>,
      theme: ForeignBorrowed<Theme>,
    ) {
      /**
       Persisted terminal diagnostic payload.
       */
      const { data, } = entry;
      if (data === undefined)
        return undefined;
      return new Text(
        [
          theme.fg(
            'error',
            theme.bold('Goal review unavailable',),
          ),
          `Attempted reviewers: ${formatAttemptedReviewers(data.attemptedReviewerIdentities,)}`,
          data.diagnostic,
        ].join('\n',),
        1,
        0,
      );
    },
  );
}

//endregion Renderer registration

export {
  formatAttemptedReviewers,
  registerGoalTerminalRenderer,
};
