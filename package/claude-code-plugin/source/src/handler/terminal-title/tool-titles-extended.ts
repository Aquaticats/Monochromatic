/**
 Claude Code title entries for plan mode, worktrees, tasks, and cron tools.
 
 @module
 */

import {
  fieldTitleEntry,
  staticTitleEntry,
  textTitleEntry,
  type ToolTitleEntry,
} from '@monochromatic-dev/agent-harness-shared-terminal-title/ts';

/**
 Built-in tool names held in the extended registry segment.
 */
type ExtendedToolTitleName =
  | 'EnterPlanMode'
    | 'ExitPlanMode'
    | 'EnterWorktree'
    | 'TaskCreate'
    | 'TaskGet'
    | 'TaskList'
    | 'TaskOutput'
    | 'TaskStop'
    | 'TaskUpdate'
    | 'CronCreate'
    | 'CronDelete'
    | 'CronList';

/**
 Title entries for plan mode, worktree, task, and cron tools.
 */
const EXTENDED_TOOL_TITLES: Record<ExtendedToolTitleName, ToolTitleEntry> = {
  EnterPlanMode: staticTitleEntry({
    pre: 'Entering plan mode',
    post: 'Entered plan mode',
  },),
  ExitPlanMode: staticTitleEntry({
    pre: 'Exiting plan mode',
    post: 'Exited plan mode',
  },),
  EnterWorktree: textTitleEntry({
    field: 'name',
    labels: {
      pre: 'Creating worktree',
      post: 'Created worktree',
    },
    fallback: {
      pre: 'Creating worktree',
      post: 'Created worktree',
    },
  },),
  TaskCreate: textTitleEntry({
    field: 'subject',
    labels: {
      pre: 'Creating task',
      post: 'Created task',
    },
    fallback: {
      pre: 'Creating task',
      post: 'Created task',
    },
  },),
  TaskGet: fieldTitleEntry({
    field: 'taskId',
    fallback: {
      pre: 'Getting task',
      post: 'Got task',
    },
    format({
      value,
      tense,
    }): string {
      return `${tense === 'pre' ? 'Getting' : 'Got'} task #${value}`;
    },
  },),
  TaskList: staticTitleEntry({
    pre: 'Listing tasks',
    post: 'Listed tasks',
  },),
  TaskOutput: fieldTitleEntry({
    field: 'task_id',
    fallback: {
      pre: 'Reading task output',
      post: 'Read task output',
    },
    format({
      value,
      tense,
    }): string {
      return `${tense === 'pre' ? 'Reading' : 'Read'} task output #${value}`;
    },
  },),
  TaskStop: fieldTitleEntry({
    field: 'task_id',
    fallback: {
      pre: 'Stopping task',
      post: 'Stopped task',
    },
    format({
      value,
      tense,
    }): string {
      return `${tense === 'pre' ? 'Stopping' : 'Stopped'} task #${value}`;
    },
  },),
  TaskUpdate: fieldTitleEntry({
    field: 'taskId',
    fallback: {
      pre: 'Updating task',
      post: 'Updated task',
    },
    format({
      value,
      tense,
    }): string {
      return `${tense === 'pre' ? 'Updating' : 'Updated'} task #${value}`;
    },
  },),
  CronCreate: textTitleEntry({
    field: 'prompt',
    labels: {
      pre: 'Scheduling cron',
      post: 'Scheduled cron',
    },
    fallback: {
      pre: 'Scheduling cron',
      post: 'Scheduled cron',
    },
  },),
  CronDelete: fieldTitleEntry({
    field: 'id',
    fallback: {
      pre: 'Deleting cron',
      post: 'Deleted cron',
    },
    format({
      value,
      tense,
    }): string {
      return `${tense === 'pre' ? 'Deleting' : 'Deleted'} cron #${value}`;
    },
  },),
  CronList: staticTitleEntry({
    pre: 'Listing cron jobs',
    post: 'Listed cron jobs',
  },),
};

export { EXTENDED_TOOL_TITLES, };
