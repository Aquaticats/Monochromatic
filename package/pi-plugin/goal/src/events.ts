/**
 Persisted goal event validation and active-branch extraction.
 
 @module
 */

import { GOAL_STATE_ENTRY_TYPE, } from './constants.ts';
import type { GoalEvent, } from './types.ts';

//region Branch entry shape

/**
 Minimal Pi custom-entry shape accepted by branch extractor.
 
 @example
 ```ts
 const entry: GoalBranchEntry = { type: 'custom', customType: 'goal:state', data: event };
 ```
 */
type GoalBranchEntry = {
  /**
   Pi session entry discriminator.
   */
  readonly type: string;
  /**
   Extension custom type when entry is custom.
   */
  readonly customType?: unknown;
  /**
   Persisted custom payload.
   */
  readonly data?: unknown;
};

//endregion Branch entry shape

//region Value guards

/**
 Narrow unknown value to property record.
 
 @param value - candidate event payload
 
 @returns whether string property lookup is safe
 
 @example
 ```ts
 isRecord({ kind: 'run_started' });
 ```
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return (value !== null)
    && ((typeof value) === 'object');
}

/**
 Require named string properties on record.
 
 @param record - candidate event record
 
 @param names - required string property names
 
 @returns whether every named property is string
 
 @example
 ```ts
 hasStringProperties({ runId: 'r' }, ['runId']);
 ```
 */
function hasStringProperties(
  {
    record,
    names,
  }: {
    readonly record: Readonly<Record<string, unknown>>;
    readonly names: readonly string[];
  },
): boolean {
  return names.every(function propertyIsString(name,) {
    return (typeof record[name]) === 'string';
  },);
}

/**
 Validate unknown value as string array.
 
 @param value - candidate array
 
 @returns whether every entry is string
 
 @example
 ```ts
 isStringArray(['review/model']);
 ```
 */
function isStringArray(value: unknown,): value is string[] {
  return Array.isArray(value,)
    && value.every(function entryIsString(entry,) {
      return (typeof entry) === 'string';
    },);
}

//endregion Value guards

//region Event validation and extraction

/**
 Validate persisted unknown payload as one supported goal event.
 
 @param value - custom-entry payload
 
 @returns whether payload is goal event
 
 @example
 ```ts
 isGoalEvent({ kind: 'run_cleared', runId: 'r', generationId: 'g', clearedAt: 'now' });
 ```
 */
function isGoalEvent(value: unknown,): value is GoalEvent {
  if (!isRecord(value,))
    return false;
  /**
   Event kind inspected by guarded branches.
   */
  const { kind, } = value;
  if ((typeof kind) !== 'string')
    return false;
  if (kind === 'run_started') {
    return hasStringProperties({
      record: value,
      names: [
        'runId',
        'generationId',
        'objective',
        'startedAt',
        'startBoundary',
        'transitionedAt',
      ],
    },)
      && (value.continuationSequence === 0)
      && ((value.supersededRunId === undefined)
        || ((typeof value.supersededRunId) === 'string'));
  }
  if (kind === 'generation_rotated') {
    return hasStringProperties({
      record: value,
      names: [
        'runId',
        'previousGenerationId',
        'generationId',
        'transitionedAt',
      ],
    },)
      && ((typeof value.continuationSequence) === 'number')
      && ((value.cause === 'runtime_restore')
        || (value.cause === 'tree_navigation'));
  }
  if (kind === 'review_denied') {
    /**
     Shared identity and sequence validity for legacy and current denials.
     */
    const baseValid = hasStringProperties({
      record: value,
      names: [
        'runId',
        'generationId',
        'transitionedAt',
      ],
    },)
      && ((typeof value.continuationSequence) === 'number');
    if (!baseValid)
      return false;
    if ((typeof value.remainingWork) === 'string') {
      return hasStringProperties({
        record: value,
        names: [
          'reviewerIdentity',
          'reviewerRationale',
        ],
      },)
        && isStringArray(value.attemptedReviewerIdentities,)
        && ((typeof value.transcriptTruncated) === 'boolean');
    }
    return (typeof value.feedback) === 'string';
  }
  if (kind === 'continuation_issued') {
    return hasStringProperties({
      record: value,
      names: [
        'runId',
        'generationId',
        'transitionedAt',
      ],
    },)
      && ((typeof value.continuationSequence) === 'number');
  }
  if (kind === 'run_completed_model') {
    /**
     Shared model-completion identity validity.
     */
    const baseValid = hasStringProperties({
      record: value,
      names: [
        'runId',
        'generationId',
        'reviewerIdentity',
        'completedAt',
      ],
    },);
    if (!baseValid)
      return false;
    if ((typeof value.reviewerRationale) === 'string') {
      return isStringArray(value.attemptedReviewerIdentities,)
        && ((typeof value.transcriptTruncated) === 'boolean');
    }
    return hasStringProperties({
      record: value,
      names: [
        'summary',
        'reviewerFeedback',
      ],
    },);
  }
  if (kind === 'run_completed_manual') {
    /**
     Shared manual-completion identity validity.
     */
    const baseValid = hasStringProperties({
      record: value,
      names: [
        'runId',
        'generationId',
        'completedAt',
      ],
    },);
    if (!baseValid)
      return false;
    if ((typeof value.reviewerRationale) === 'string')
      return isStringArray(value.attemptedReviewerIdentities,);
    return hasStringProperties({
      record: value,
      names: [
        'summary',
        'reviewerFeedback',
      ],
    },);
  }
  if (kind === 'review_unavailable') {
    return hasStringProperties({
      record: value,
      names: [
        'runId',
        'generationId',
        'diagnostic',
        'terminalAt',
      ],
    },)
      && isStringArray(value.attemptedReviewerIdentities,)
      && ((value.summary === undefined) || ((typeof value.summary) === 'string'));
  }
  if (kind === 'run_cleared') {
    return hasStringProperties({
      record: value,
      names: [
        'runId',
        'generationId',
        'clearedAt',
      ],
    },);
  }
  return false;
}

/**
 Extract validated goal events from ordered active branch entries.
 
 @param entries - `SessionManager.getBranch()` result or compatible fixture
 
 @returns ordered validated goal events
 
 @example
 ```ts
 const events = goalEventsFromBranch(entries);
 ```
 */
function goalEventsFromBranch(entries: readonly GoalBranchEntry[],): GoalEvent[] {
  return entries
    .filter(function isGoalCustomEntry(entry,) {
      return (entry.type === 'custom')
        && (entry.customType === GOAL_STATE_ENTRY_TYPE)
        && isGoalEvent(entry.data,);
    },)
    .map(function goalEventData(entry,) {
      if (!isGoalEvent(entry.data,))
        throw new Error('Validated goal custom entry lost event shape',);
      return entry.data;
    },);
}

//endregion Event validation and extraction

export {
  goalEventsFromBranch,
  isGoalEvent,
};
export type { GoalBranchEntry, };
