/**
 Type definitions for the `\<task-detail\>` component.
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
export const METADATA_UNSET: unique symbol = Symbol('editor metadata field has no selected value',);

/**
 Blocker task summary displayed as a pill in the task detail view.
 */
export type BlockerSummary = {
  /**
   UUID of the blocking task.
   */
  readonly id: string;
  /**
   Title of the blocking task.
   */
  readonly title: string;
  /**
   Current status of the blocking task.
   */
  readonly status: string;
};

/**
 Shape of the JSON response from the `/api/ai/autofill` endpoint.
 */
export type AutofillResult = {
  /**
   Suggested tags for the task.
   */
  readonly tags: readonly string[];
  /**
   Suggested locations for the task.
   */
  readonly locations: readonly string[];
  /**
   Suggested priority level; absent when none was inferred.
   */
  readonly priority?: TaskPriority;
  /**
   Suggested complexity level; absent when none was inferred.
   */
  readonly complexity?: TaskComplexity;
};

/**
 Determines whether the component renders as a new-task creator or an editor.
 */
export type TaskDetailMode = 'create' | 'edit';

/**
 Configuration payload passed to {@link TaskDetail.configure}.
 */
export type TaskDetailData = {
  /**
   Task being viewed or edited.
   */
  readonly task: Task;
  /**
   Summaries of tasks that block this one.
   */
  readonly blockerSummaries: readonly BlockerSummary[];
  /**
   Display mode: "create" for new tasks, "edit" for existing.
   */
  readonly mode?: TaskDetailMode;
};
