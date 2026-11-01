/**
 Active-branch post-start evidence serialization for settlement reviewers.
 
 @module
 */

import type {
  ImageContent,
  TextContent,
  ThinkingContent,
  ToolCall,
} from '@earendil-works/pi-ai';
import type { SessionEntry, } from '@earendil-works/pi-coding-agent';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  GOAL_MESSAGE_TYPE,
  GOAL_STATE_ENTRY_TYPE,
} from './constants.ts';
import type {
  GoalReviewEvidence,
  GoalSettlementReviewRequest,
} from './completion-types.ts';
import { isGoalEvent, } from './events.ts';

/**
 Sentinel for branch entries intentionally excluded from reviewer evidence.
 */
const EVIDENCE_ENTRY_OMITTED: unique symbol = Symbol('goal/evidence-entry-omitted',);

/**
 Convert finalized content block to reviewer-visible text.
 
 @param content - finalized message content block
 
 @returns reviewer-visible text
 
 @example
 ```ts
 contentBlockText({ type: 'text', text: 'done' });
 ```
 */
function contentBlockText(
  content: ForeignBorrowed<TextContent | ImageContent | ToolCall | ThinkingContent>,
): string {
  if (content.type === 'text')
    return content.text;
  if (content.type === 'image')
    return `[image evidence omitted: ${content.mimeType}]`;
  if (content.type === 'thinking')
    return '[private reasoning omitted]';
  return `tool call ${content.name}`;
}

/**
 Convert string or block-array message content to reviewer text.
 
 @param content - finalized message content
 
 @returns joined textual evidence
 
 @example
 ```ts
 messageContentText('done');
 ```
 */
function messageContentText(
  content: ForeignBorrowed<
    string | (TextContent | ImageContent | ToolCall | ThinkingContent)[]
    >,
): string {
  if ((typeof content) === 'string')
    return content;
  return content
    .map(contentBlockText,)
    .join('\n',);
}

/**
 Validate current-generation task-message provenance.
 
 @param details - unknown custom-message details
 
 @param runId - active run identity
 
 @param generationId - active generation identity
 
 @returns whether details identify current kickoff or continuation
 
 @example
 ```ts
 isCurrentGoalMessageDetails({ details, runId: 'run-1', generationId: 'generation-1' });
 ```
 */
function isCurrentGoalMessageDetails(
  {
    details,
    runId,
    generationId,
  }: {
    readonly details: unknown;
    readonly runId: string;
    readonly generationId: string;
  },
): boolean {
  if ((details === null) || ((typeof details) !== 'object'))
    return false;
  if (!(('runId' in details)
    && ('generationId' in details)
    && ('continuationSequence' in details)
    && ('marker' in details)
    && ('kind' in details))) {
    return false;
  }
  return (details.runId === runId)
    && (details.generationId === generationId)
    && ((typeof details.continuationSequence) === 'number')
    && ((typeof details.marker) === 'string')
    && ((details.kind === 'kickoff') || (details.kind === 'continuation'));
}

/**
 Serialize one eligible branch entry or return omission sentinel.
 
 @param entry - active-branch session entry
 
 @param runId - current active run identity
 
 @param generationId - current active generation identity
 
 @returns labeled evidence chunk or omission sentinel
 
 @example
 ```ts
 serializeEvidenceEntry({ entry, runId: 'run-1', generationId: 'generation-1' });
 ```
 */
function serializeEvidenceEntry(
  {
    entry,
    runId,
    generationId,
  }: {
    readonly entry: ForeignBorrowed<SessionEntry>;
    readonly runId: string;
    readonly generationId: string;
  },
): string | typeof EVIDENCE_ENTRY_OMITTED {
  if (entry.type === 'custom_message') {
    if ((entry.customType !== GOAL_MESSAGE_TYPE)
      || (!entry.display)
      || (!isCurrentGoalMessageDetails({
        details: entry.details,
        runId,
        generationId,
      },))) {
      return EVIDENCE_ENTRY_OMITTED;
    }
    return `Task context:\n${messageContentText(entry.content,)}`;
  }
  if (entry.type !== 'message')
    return EVIDENCE_ENTRY_OMITTED;
  /**
   Finalized agent message stored by selected branch.
   */
  const { message, } = entry;
  if (message.role === 'user')
    return `User:\n${messageContentText(message.content,)}`;
  if (message.role === 'assistant')
    return `Assistant:\n${messageContentText(message.content,)}`;
  if (message.role === 'toolResult')
    return `Finalized tool result ${message.toolName}:\n${messageContentText(message.content,)}`;
  if (message.role === 'bashExecution') {
    if (message.excludeFromContext === true)
      return EVIDENCE_ENTRY_OMITTED;
    return `Shell command ${message.command} (exit ${String(message.exitCode,)}):\n${message.output}`;
  }
  return EVIDENCE_ENTRY_OMITTED;
}

/**
 Find matching run-start index on selected branch.
 
 @param branch - selected active branch
 
 @param runId - active run identity
 
 @returns matching entry index
 
 @throws when active run start is missing from branch
 
 @example
 ```ts
 findRunStartIndex({ branch, runId: 'run-1' });
 ```
 */
function findRunStartIndex(
  {
    branch,
    runId,
  }: {
    readonly branch: readonly ForeignBorrowed<SessionEntry>[];
    readonly runId: string;
  },
): number {
  /**
   Matching run-start entry position.
   */
  const index = branch.findIndex(function isMatchingRunStart(entry,) {
    if ((entry.type !== 'custom') || (entry.customType !== GOAL_STATE_ENTRY_TYPE))
      return false;
    if (!isGoalEvent(entry.data,))
      return false;
    return (entry.data
      .kind
      === 'run_started')
      && (entry.data
        .runId
        === runId);
  },);
  if (index === (-1))
    throw new Error(`Active goal run start is absent from selected branch: ${runId}`,);
  return index;
}

/**
 Build reviewer evidence from selected branch after active run start.
 
 @param branch - `SessionManager.getBranch()` result
 
 @param request - captured settlement identity
 
 @returns objective and finalized post-start chunks
 
 @example
 ```ts
 buildGoalReviewEvidence({ branch, request });
 ```
 */
function buildGoalReviewEvidence(
  {
    branch,
    request,
  }: {
    readonly branch: readonly ForeignBorrowed<SessionEntry>[];
    readonly request: GoalSettlementReviewRequest;
  },
): GoalReviewEvidence {
  /**
   Matching run-start position defining transcript seam.
   */
  const startIndex = findRunStartIndex({
    branch,
    runId: request.goal
      .runId,
  },);
  /**
   Active run identities filtering task messages.
   */
  const {
    runId,
    generationId,
  } = request.goal;
  /**
   Eligible serialized chunks after current run started.
   */
  const transcriptChunks = branch
    .slice(startIndex + 1,)
    .map(function serializeEntry(entry,) {
      return serializeEvidenceEntry({
        entry,
        runId,
        generationId,
      },);
    },)
    .filter(function keepEvidenceChunk(
      chunk,
    ): chunk is string {
      return (typeof chunk) === 'string';
    },);
  return {
    objective: request.goal
      .objective,
    transcriptChunks,
  };
}

export {
  buildGoalReviewEvidence,
  contentBlockText,
  findRunStartIndex,
  isCurrentGoalMessageDetails,
  messageContentText,
  serializeEvidenceEntry,
};
