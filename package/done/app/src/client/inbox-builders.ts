/**
 DOM building helpers for the Inbox page.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import type {
  BlockedTaskLink,
  Task,
} from '../lib/types.ts';
import { createTaskCard, } from './lib/task-card.ts';

/**
 Map of blocker task ID to the tasks it blocks.
 
 Deeply readonly; under `noUncheckedIndexedAccess` an index lookup already
 yields `... | undefined` for blocker IDs with no blocked tasks, so no
 explicit `| undefined` value union (banned by `no-nullish-union`) is needed.
 */
export type BlockedTasksByBlocker = Readonly<
  Record<string, readonly BlockedTaskLink[]>
  >;

/**
 Shape of the JSON blob embedded in the inbox page by the server.
 */
export type InboxPageData = {
  /**
   AI-prioritized suggested tasks for the current context.
   */
  readonly suggestedTasks: readonly Task[];
  /**
   All inbox tasks.
   */
  readonly allTasks: readonly Task[];
  /**
   Map of blocker task ID to the tasks it blocks.
   */
  readonly blockedTasksByBlocker: BlockedTasksByBlocker;
};

/**
 Builds a task list with optional blocked-child nesting, rendering each
 task with {@link createTaskCard}.
 
 @returns UL element containing task cards with nested blocker children
 
 @example
 ```ts
 const list = buildTaskList({ tasks, blockedTasksByBlocker, onOpen, onToggleComplete });
 app.append(list);
 ```
 */
export function buildTaskList(
  {
    tasks,
    blockedTasksByBlocker,
    onOpen,
    onToggleComplete,
  }: {
    readonly tasks: readonly Task[];
    readonly blockedTasksByBlocker: BlockedTasksByBlocker;
    readonly onOpen: (taskId: string,) => void;
    readonly onToggleComplete: (taskId: string,) => Promise<void>;
  },
): HTMLUListElement {
  /**
   Top-level list mutated in-place as the loop appends cards and child branches.
   */
  const list = h({
    tag: 'ul',
    class: 'task-list',
  },);

  for (const task of tasks) {
    list.append(createTaskCard({
      task,
      options: {
        onOpen,
        onToggleComplete,
      },
    },),);

    /**
     Tasks blocked by `task`; empty when nothing depends on it.
     */
    const childLinks = blockedTasksByBlocker[task.id]
      ?? [];
    if (childLinks.length
      > 0) {
      list.append(
        h({
          tag: 'div',
          class: 'task-children',
          children: [
            h({
              tag: 'ul',
              class: 'task-list',
              children: childLinks.map(function buildBlockedCard(childLink,) {
                return createTaskCard({
                  task: childLink.task,
                  options: {
                    showBlockedBadge: true,
                    onOpen,
                    onToggleComplete,
                  },
                },);
              },),
            },),
          ],
        },),
      );
    }
  }

  return list;
}
