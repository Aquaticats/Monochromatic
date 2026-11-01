/**
 Goal domain state, event, message, and controller types.
 
 @module
 */

//region Domain identifiers

/**
 Stable user-visible goal run identity.
 */
type GoalRunId = string;

/**
 Private stale-callback generation identity.
 */
type GoalGenerationId = string;

/**
 Stable marker identifying current run start boundary.
 */
type GoalStartBoundary = string;

/**
 Unique extension-authored message identity.
 */
type GoalMessageMarker = string;

/**
 Runtime-instance identity invalidating old callbacks.
 */
type GoalRuntimeEpoch = string;

//endregion Domain identifiers

//region Persisted states

/**
 No current goal record on selected branch.
 */
type AbsentGoalState = {
  readonly phase: 'absent';
};

/**
 Active goal retained across continuation turns.
 */
type ActiveGoalState = {
  readonly phase: 'active';
  readonly runId: GoalRunId;
  readonly generationId: GoalGenerationId;
  readonly objective: string;
  readonly startedAt: string;
  readonly startBoundary: GoalStartBoundary;
  readonly continuationSequence: number;
  readonly transitionedAt: string;
  readonly remainingWork?: string;
};

/**
 Model-approved terminal completion.
 */
type ModelCompletedGoalState = {
  readonly phase: 'completed';
  readonly runId: GoalRunId;
  readonly generationId: GoalGenerationId;
  readonly objective: string;
  readonly approvalSource: 'model';
  readonly reviewerIdentity: string;
  readonly reviewerRationale: string;
  readonly attemptedReviewerIdentities: readonly string[];
  readonly transcriptTruncated: boolean;
  readonly completedAt: string;
  readonly legacySummary?: string;
};

/**
 Manually approved terminal completion.
 */
type ManualCompletedGoalState = {
  readonly phase: 'completed';
  readonly runId: GoalRunId;
  readonly generationId: GoalGenerationId;
  readonly objective: string;
  readonly approvalSource: 'manual';
  readonly reviewerRationale: string;
  readonly attemptedReviewerIdentities: readonly string[];
  readonly completedAt: string;
  readonly legacySummary?: string;
};

/**
 Terminal state when every reviewer attempt fails without TUI fallback.
 */
type ReviewUnavailableGoalState = {
  readonly phase: 'review_unavailable';
  readonly runId: GoalRunId;
  readonly generationId: GoalGenerationId;
  readonly objective: string;
  readonly attemptedReviewerIdentities: readonly string[];
  readonly diagnostic: string;
  readonly terminalAt: string;
  readonly legacySummary?: string;
};

/**
 Branch-reduced current goal state.
 */
type GoalState =
  | AbsentGoalState
    | ActiveGoalState
    | ModelCompletedGoalState
    | ManualCompletedGoalState
    | ReviewUnavailableGoalState;

//endregion Persisted states

//region Persisted events

/**
 One run start or atomic replacement event.
 */
type GoalRunStartedEvent = {
  readonly kind: 'run_started';
  readonly runId: GoalRunId;
  readonly generationId: GoalGenerationId;
  readonly objective: string;
  readonly startedAt: string;
  readonly startBoundary: GoalStartBoundary;
  readonly continuationSequence: 0;
  readonly transitionedAt: string;
  readonly supersededRunId?: GoalRunId;
};

/**
 Active restoration generation rotation.
 */
type GoalGenerationRotatedEvent = {
  readonly kind: 'generation_rotated';
  readonly runId: GoalRunId;
  readonly previousGenerationId: GoalGenerationId;
  readonly generationId: GoalGenerationId;
  readonly continuationSequence: number;
  readonly transitionedAt: string;
  readonly cause: 'runtime_restore' | 'tree_navigation';
};

/**
 Valid reviewer denial retaining active state.
 */
type GoalReviewDeniedEvent = {
  readonly kind: 'review_denied';
  readonly runId: GoalRunId;
  readonly generationId: GoalGenerationId;
  readonly remainingWork: string;
  readonly reviewerIdentity: string;
  readonly reviewerRationale: string;
  readonly attemptedReviewerIdentities: readonly string[];
  readonly transcriptTruncated: boolean;
  readonly continuationSequence: number;
  readonly transitionedAt: string;
};

/**
 Legacy tool-era denial retained for session reconstruction.
 */
type LegacyGoalReviewDeniedEvent = {
  readonly kind: 'review_denied';
  readonly runId: GoalRunId;
  readonly generationId: GoalGenerationId;
  readonly feedback: string;
  readonly continuationSequence: number;
  readonly transitionedAt: string;
};

/**
 Visible continuation issuance event retaining auditable sequence.
 */
type GoalContinuationIssuedEvent = {
  readonly kind: 'continuation_issued';
  readonly runId: GoalRunId;
  readonly generationId: GoalGenerationId;
  readonly continuationSequence: number;
  readonly transitionedAt: string;
};

/**
 Model-approved completion event.
 */
type GoalModelCompletedEvent = {
  readonly kind: 'run_completed_model';
  readonly runId: GoalRunId;
  readonly generationId: GoalGenerationId;
  readonly reviewerIdentity: string;
  readonly reviewerRationale?: string;
  readonly attemptedReviewerIdentities?: readonly string[];
  readonly transcriptTruncated?: boolean;
  readonly completedAt: string;
  readonly summary?: string;
  readonly reviewerFeedback?: string;
};

/**
 Manually approved completion event.
 */
type GoalManualCompletedEvent = {
  readonly kind: 'run_completed_manual';
  readonly runId: GoalRunId;
  readonly generationId: GoalGenerationId;
  readonly reviewerRationale?: string;
  readonly attemptedReviewerIdentities?: readonly string[];
  readonly completedAt: string;
  readonly summary?: string;
  readonly reviewerFeedback?: string;
};

/**
 Terminal reviewer-unavailable event.
 */
type GoalReviewUnavailableEvent = {
  readonly kind: 'review_unavailable';
  readonly runId: GoalRunId;
  readonly generationId: GoalGenerationId;
  readonly attemptedReviewerIdentities: readonly string[];
  readonly diagnostic: string;
  readonly terminalAt: string;
  readonly summary?: string;
};

/**
 Clear tombstone reducing selected branch state to absent.
 */
type GoalRunClearedEvent = {
  readonly kind: 'run_cleared';
  readonly runId: GoalRunId;
  readonly generationId: GoalGenerationId;
  readonly clearedAt: string;
};

/**
 Immutable goal event payload persisted in Pi custom entry.
 */
type GoalEvent =
  | GoalRunStartedEvent
    | GoalGenerationRotatedEvent
    | GoalReviewDeniedEvent
    | LegacyGoalReviewDeniedEvent
    | GoalContinuationIssuedEvent
    | GoalModelCompletedEvent
    | GoalManualCompletedEvent
    | GoalReviewUnavailableEvent
    | GoalRunClearedEvent;

//endregion Persisted events

//region Commands, messages, and effects

/**
 Parsed accepted or rejected `/goal` command.
 */
type ParsedGoalCommand =
  | {
    readonly kind: 'start';
    readonly objective: string
  }
    | { readonly kind: 'clear'; }
    | {
      readonly kind: 'rejected';
      readonly diagnostic: string
    };

/**
 Visible goal custom-message metadata.
 */
type GoalMessageDetails = {
  readonly runId: GoalRunId;
  readonly generationId: GoalGenerationId;
  readonly continuationSequence: number;
  readonly marker: GoalMessageMarker;
  readonly kind: 'kickoff' | 'continuation';
};

/**
 Visible custom message sent through Pi.
 */
type GoalMessage = {
  readonly customType: string;
  readonly content: string;
  readonly display: true;
  readonly details: GoalMessageDetails;
};

/**
 Deferred kickoff retained only while matching generation remains active.
 */
type PendingGoalKickoff = {
  readonly runId: GoalRunId;
  readonly generationId: GoalGenerationId;
  readonly runtimeEpoch: GoalRuntimeEpoch;
  readonly marker: GoalMessageMarker;
};

/**
 Renderable terminal reviewer-exhaustion diagnostic.
 */
type GoalReviewUnavailableDiagnostic = {
  readonly runId: GoalRunId;
  readonly generationId: GoalGenerationId;
  readonly attemptedReviewerIdentities: readonly string[];
  readonly diagnostic: string;
  readonly terminalAt: string;
};

/**
 Durable human-only completion audit.
 */
type GoalCompletionDiagnostic = {
  readonly runId: GoalRunId;
  readonly generationId: GoalGenerationId;
  readonly approvalSource: 'model' | 'manual';
  readonly reviewerIdentity: string;
  readonly reviewerRationale: string;
  readonly attemptedReviewerIdentities: readonly string[];
  readonly transcriptTruncated: boolean;
  readonly completedAt: string;
};

/**
 Immutable controller state held by Pi adapter.
 */
type GoalControllerState = {
  readonly goal: GoalState;
  readonly runtimeEpoch: GoalRuntimeEpoch;
  readonly pendingKickoff?: PendingGoalKickoff;
  readonly settlementSequence: number;
  readonly lastEmittedSettlementSequence?: number;
  readonly shutdown: boolean;
};

/**
 Semantic effect returned by controller transitions.
 */
type GoalEffect =
  | {
    readonly type: 'persist';
    readonly event: GoalEvent
  }
    | {
      readonly type: 'set_footer';
      readonly text: string
    }
    | { readonly type: 'clear_footer'; }
    | {
      readonly type: 'send_message';
      readonly message: GoalMessage;
      readonly triggerTurn: true
    }
    | {
      readonly type: 'persist_review_unavailable_diagnostic';
      readonly diagnostic: GoalReviewUnavailableDiagnostic
    }
    | {
      readonly type: 'persist_completion_diagnostic';
      readonly diagnostic: GoalCompletionDiagnostic
    }
    | {
      readonly type: 'notify';
      readonly level: 'info' | 'warning' | 'error';
      readonly message: string
    }
    | {
      readonly type: 'log';
      readonly level: 'debug' | 'warn' | 'error';
      readonly message: string
    };

/**
 Controller transition result with next immutable state and effects.
 */
type GoalControllerTransition = {
  readonly controller: GoalControllerState;
  readonly effects: readonly GoalEffect[];
};

//endregion Commands, messages, and effects

export type {
  AbsentGoalState,
  ActiveGoalState,
  GoalCompletionDiagnostic,
  GoalContinuationIssuedEvent,
  GoalControllerState,
  GoalControllerTransition,
  GoalEffect,
  GoalEvent,
  GoalGenerationId,
  GoalGenerationRotatedEvent,
  GoalManualCompletedEvent,
  GoalMessage,
  GoalMessageDetails,
  GoalMessageMarker,
  GoalModelCompletedEvent,
  GoalReviewDeniedEvent,
  GoalReviewUnavailableDiagnostic,
  GoalReviewUnavailableEvent,
  GoalRunClearedEvent,
  GoalRunId,
  GoalRunStartedEvent,
  GoalRuntimeEpoch,
  GoalStartBoundary,
  GoalState,
  ManualCompletedGoalState,
  ModelCompletedGoalState,
  ParsedGoalCommand,
  PendingGoalKickoff,
  ReviewUnavailableGoalState,
};
