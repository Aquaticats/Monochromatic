/**
 Types and constants for the `<task-detail>` web component.
 */
import type {
  Task,
  TaskComplexity,
  TaskPriority,
} from '../../lib/types.ts';

/**
 Sentinel for an editor metadata field (priority, complexity) with no
 selected value.
 
 A unique `Symbol` keeps "unset" out of a nullish union (banned by
 `no-nullish-union`) while remaining a distinct, comparable value the mutable
 editor state can be reset to; consumers narrow with `=== METADATA_UNSET`.
 */
export const METADATA_UNSET: unique symbol = Symbol('task editor metadata field left unset',);

/**
 Blocker task summary displayed as a pill in the task detail view.
 */
export type BlockerSummary = {
  /**
   Blocker task ID.
   */
  readonly id: string;
  /**
   Blocker task title.
   */
  readonly title: string;
  /**
   Blocker task status.
   */
  readonly status: string;
};

/**
 Shape of the JSON response from the `/api/ai/autofill` endpoint.
 */
export type AutofillResult = {
  /**
   AI-suggested tags.
   */
  readonly tags: readonly string[];
  /**
   AI-suggested locations.
   */
  readonly locations: readonly string[];
  /**
   AI-suggested priority; absent when none was inferred.
   */
  readonly priority?: TaskPriority;
  /**
   AI-suggested complexity; absent when none was inferred.
   */
  readonly complexity?: TaskComplexity;
};

/**
 Determines whether the component renders as a new-task creator or an editor.
 */
export type TaskDetailMode = 'create' | 'edit';

/**
 Metadata state managed by the `<task-detail>` component; reassigned wholesale on edit.
 */
export type MetadataState = {
  /**
   Current tags (user-set or autofilled).
   */
  readonly tags: readonly string[];
  /**
   Current locations (user-set or autofilled).
   */
  readonly locations: readonly string[];
  /**
   Current priority (user-set or autofilled); `METADATA_UNSET` when none chosen.
   */
  readonly priority: TaskPriority | typeof METADATA_UNSET;
  /**
   Current complexity (user-set or autofilled); `METADATA_UNSET` when none chosen.
   */
  readonly complexity: TaskComplexity | typeof METADATA_UNSET;
};

/**
 Configuration payload passed to {@link TaskDetail.configure}.
 */
export type TaskDetailData = {
  /**
   Task being displayed or edited.
   */
  readonly task: Task;
  /**
   Summaries of tasks that block this one.
   */
  readonly blockerSummaries: readonly BlockerSummary[];
  /**
   Component mode: `"create"` for new tasks, `"edit"` (default) for existing.
   */
  readonly mode?: TaskDetailMode;
};

/**
 Delay before triggering AI autofill after the user stops typing.
 */
export const AUTOFILL_DEBOUNCE_MS = 500;
