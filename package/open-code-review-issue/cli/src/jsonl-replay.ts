/**
 OCR JSONL event replay state transitions.
 
 @module
 */

import {
  checkpointFindings,
  eventString,
  jsonlResolvedHeadMetadata,
} from './jsonl-event.ts';
import type {
  NormalizedFinding,
  NormalizedInput,
} from './model.ts';

/**
 Parsed JSONL event with physical source line.
 
 @example
 ```ts
 const event: PositionedJsonlRecord = {
   line: 1,
   record: { type: 'session_start' },
 };
 ```
 */
export type PositionedJsonlRecord = {
  readonly line: number;
  readonly record: Readonly<Record<string, unknown>>;
};

/**
 Replay group preserving first checkpoint order across supersession.
 */
type ReplayGroup = {
  readonly findings: readonly NormalizedFinding[];
};

/**
 Mutable replay accumulator hidden inside one ownership boundary.
 */
type ReplayState = {
  readonly groups: ReplayGroup[];
  readonly groupByFingerprint: Map<string, number>;
  resolvedHeadMetadata: { readonly resolvedHead?: string; };
};

/**
 Applies completed or reused checkpoint to replay state.
 
 @param state - Replay accumulator owned by one input parse.
 
 @param record - Completed or reused item event.
 
 @param line - One-based event line.
 
 @example
 ```ts
 applyCheckpoint({ state, record: { type: 'review_item_done', comments: [] }, line: 1 });
 ```
 */
function applyCheckpoint({
  state,
  record,
  line,
}: {
  readonly state: ReplayState;
  readonly record: Readonly<Record<string, unknown>>;
  readonly line: number;
},): void {
  /**
   Raw diff fingerprint identifying superseded group.
   */
  const fingerprint = eventString({
    record,
    key: 'fingerprint',
    line,
  });
  /**
   Replacement group normalized from latest checkpoint.
   */
  const group: ReplayGroup = {
    findings: checkpointFindings({
      record,
      line,
    }),
  };
  /**
   Existing ordered slot for non-empty fingerprint.
   */
  const existingIndex = fingerprint === ''
    ? undefined
    : state.groupByFingerprint
      .get(fingerprint,);
  if (existingIndex !== undefined) {
    state.groups[existingIndex] = group;
    return;
  }
  state.groups
    .push(group,);
  if (fingerprint !== '') {
    state.groupByFingerprint
      .set(
      fingerprint,
      state.groups
        .length
        - 1,
      );
  }
}

/**
 Applies failed checkpoint by clearing matching ordered group.
 
 @param state - Replay accumulator owned by one input parse.
 
 @param record - Failed item event.
 
 @param line - One-based event line.
 
 @example
 ```ts
 applyFailure({ state, record: { fingerprint: 'x' }, line: 2 });
 ```
 */
function applyFailure({
  state,
  record,
  line,
}: {
  readonly state: ReplayState;
  readonly record: Readonly<Record<string, unknown>>;
  readonly line: number;
},): void {
  /**
   Failed item fingerprint used to find existing group.
   */
  const fingerprint = eventString({
    record,
    key: 'fingerprint',
    line,
  });
  /**
   Existing group slot when fingerprint was previously completed.
   */
  const existingIndex = fingerprint === ''
    ? undefined
    : state.groupByFingerprint
      .get(fingerprint,);
  if (existingIndex !== undefined) {
    state.groups[existingIndex] = { findings: [], };
  }
}

/**
 Replays parsed OCR events into normalized input.
 
 @param records - Validated JSONL event objects with physical lines.
 
 @returns Replayed findings and available head provenance.
 
 @example
 ```ts
 replayJsonlRecords({ records: [] });
 ```
 */
export function replayJsonlRecords({
  records,
}: {
  readonly records: readonly PositionedJsonlRecord[];
},): NormalizedInput {
  /**
   Single replay state whose mutation stays inside this ownership boundary.
   */
  const state: ReplayState = {
    groups: [],
    groupByFingerprint: new Map(),
    resolvedHeadMetadata: {},
  };
  records.forEach(function applyRecord({
    record,
    line,
  },): void {
    if ((record.type === 'review_item_done') || (record.type === 'review_item_reused')) {
      applyCheckpoint({
        state,
        record,
        line,
      });
      return;
    }
    if (record.type === 'review_item_failed') {
      applyFailure({
        state,
        record,
        line,
      });
      return;
    }
    if (record.type === 'session_end') {
      /**
       Head metadata from latest session-end event carrying one.
       */
      const metadata = jsonlResolvedHeadMetadata({
        record,
        line,
      });
      if (metadata.resolvedHead !== undefined) {
        state.resolvedHeadMetadata = metadata;
      }
    }
  },);
  return {
    inputKind: 'jsonl',
    ...state.resolvedHeadMetadata,
    findings: state.groups
      .flatMap(function groupFindings(group,): readonly NormalizedFinding[] {
        return group.findings;
      },),
  };
}
