/**
 All recognized priority levels, ordered ascending.
 */
export const TASK_PRIORITIES: readonly [
  'low',
  'medium',
  'high',
] = [
  'low',
  'medium',
  'high',
];

/**
 Discriminated union of valid priority values.
 */
export type TaskPriority = (typeof TASK_PRIORITIES)[number];

/**
 All recognized complexity levels, ordered ascending.
 */
export const TASK_COMPLEXITIES: readonly [
  'low',
  'medium',
  'high',
] = [
  'low',
  'medium',
  'high',
];

/**
 Discriminated union of valid complexity values.
 */
export type TaskComplexity = (typeof TASK_COMPLEXITIES)[number];

/**
 All recognized task statuses.
 */
export const TASK_STATUSES: readonly [
  'inbox',
  'in_progress',
  'done',
] = [
  'inbox',
  'in_progress',
  'done',
];

/**
 Discriminated union of valid task lifecycle statuses.
 */
export type TaskStatus = (typeof TASK_STATUSES)[number];

/**
 External system from which a task was imported, or `"local"` for manually created tasks.
 */
export type TaskSource = 'local' | 'github' | 'linear' | 'calendar' | 'codebase';

/**
 Sentinel returned by task lookups when no row matches the requested ID.
 
 A unique `Symbol` keeps "not found" out of a `Task | null` union (banned by
 `no-nullish-union`); callers narrow with `=== TASK_NOT_FOUND`.
 */
export const TASK_NOT_FOUND: unique symbol = Symbol('task row not found by id',);

/**
 Canonical task shape shared between server (DB layer) and client (page data JSON).
 */
export type Task = {
  readonly id: string;
  readonly title: string;
  readonly description?: string;
  readonly tags: readonly string[];
  readonly locations: readonly string[];
  readonly priority?: TaskPriority;
  readonly dueDate?: string;
  readonly complexity?: TaskComplexity;
  readonly reminders: readonly string[];
  readonly blockedBy: readonly string[];
  /**
   Accumulated tracked time in seconds (excludes any running timer).
   */
  readonly trackedTime: number;
  /**
   ISO timestamp when the current timer was started; absent when idle.
   */
  readonly timerStartedAt?: string;
  readonly status: TaskStatus;
  readonly source: TaskSource;
  readonly sourceId?: string;
  readonly sourceMeta?: string;
  readonly createdAt: string;
  readonly updatedAt: string;
};

/**
 Associates a blocked inbox task with the blocker task that gates it.
 */
export type BlockedTaskLink = {
  readonly blockerId: string;
  readonly task: Task;
};

/**
 Search result task with an additional `isBlocked` flag for UI badge display.
 */
export type SearchTask = Task & {
  readonly isBlocked: boolean;
};

/**
 Payload accepted by {@link createTask}: only `title` is required; all others default.
 */
export type TaskCreateInput = {
  readonly title: string;
  readonly description?: string;
  readonly tags?: readonly string[];
  readonly locations?: readonly string[];
  readonly priority?: TaskPriority;
  readonly dueDate?: string;
  readonly complexity?: TaskComplexity;
  readonly reminders?: readonly string[];
  readonly blockedBy?: readonly string[];
};

/**
 Partial update payload accepted by {@link updateTask}: omitted fields stay unchanged.
 */
export type TaskUpdateInput = {
  readonly title?: string;
  readonly description?: string;
  readonly tags?: readonly string[];
  readonly locations?: readonly string[];
  readonly priority?: TaskPriority;
  readonly dueDate?: string;
  readonly complexity?: TaskComplexity;
  readonly reminders?: readonly string[];
  readonly blockedBy?: readonly string[];
  readonly status?: TaskStatus;
};
