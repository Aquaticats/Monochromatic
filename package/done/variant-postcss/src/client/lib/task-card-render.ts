/**
 DOM construction and chip building for `\<task-card\>`.
 
 Extracted from task-card.ts to keep each file under the line-count limit.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import type { Task, } from '../../lib/types.ts';
import { formatRunningTrackedTime, } from './format-tracked-time.ts';
import { TASK_CARD_STYLES, } from './task-card-styles.ts';

/**
 Configuration for a `\<task-card\>` instance, passed via `createTaskCard`.
 */
export type TaskCardOptions = {
  /**
   Whether to show a red "blocked" badge chip.
   */
  readonly showBlockedBadge?: boolean;
  /**
   Callback when the card body is clicked (navigates to task detail).
   */
  readonly onOpen: (taskId: string,) => void;
  /**
   Callback when the checkbox is clicked (completes the task).
   */
  readonly onToggleComplete?: (taskId: string,) => Promise<void>;
};

/**
 Collects all metadata chip labels for a task.
 
 @param task - Task whose metadata to extract
 
 @returns Array of chip label strings
 
 @example
 ```ts
 const chips = buildChipTexts(task);
 // ['# shopping, errands', 'tracked: 1h30min0s', 'priority: high']
 ```
 */
export function buildChipTexts(task: Task,): string[] {
  /**
   Chip strings accumulated in display order; each block below pushes conditionally.
   */
  const chips: string[] = [];
  if (task.tags
    .length
    > 0)
    chips.push(`# ${task.tags
      .join(', ',)}`,);
  chips.push(`tracked: ${formatRunningTrackedTime(task,)}`,);
  if (task.locations
    .length
    > 0)
    chips.push(`where: ${task.locations
      .join(', ',)}`,);
  if (task.priority
    !== undefined)
    chips.push(`priority: ${task.priority}`,);
  if (task.dueDate
    !== undefined)
    chips.push(`due: ${task.dueDate}`,);
  if (task.complexity
    !== undefined)
    chips.push(`complexity: ${task.complexity}`,);
  if (task.reminders
    .length
    > 0)
    chips.push(`reminders: ${task.reminders[0]}`,);
  if (task.blockedBy
    .length
    > 0)
    chips.push(`blockedBy: ${String(task.blockedBy
      .length,)}`,);
  else
    chips.push('blockedBy: none',);
  return chips;
}

/**
 Renders the full card content into the shadow root.
 
 @param shadow - Shadow root to render into
 
 @param task - Task data to display
 
 @param options - Callbacks and display flags
 
 @example
 ```ts
 renderTaskCardContent({ shadow, task, options: { onOpen: openTask } });
 ```
 */
export function renderTaskCardContent(
  {
    shadow,
    task,
    options,
  }: {
    readonly shadow: ShadowRoot;
    readonly task: Task;
    readonly options: TaskCardOptions;
  },
): void {
  /**
   Rendered chip elements; the optional blocked badge is appended below when requested.
   */
  const chipElements: HTMLElement[] = buildChipTexts(task,)
    .map(
    function createChip(text,): HTMLElement {
      return h({
        tag: 'span',
        class: 'chip',
        text,
      },);
    },
    );
  if (options.showBlockedBadge
    === true) {
    chipElements.push(h({
      tag: 'span',
      class: 'chip blocked',
      text: 'blocked',
    },),);
  }

  shadow.replaceChildren(
    h({
      tag: 'style',
      text: TASK_CARD_STYLES,
    },),
    h({
      tag: 'div',
      class: 'row',
      children: [
        h({
          tag: 'button',
          class: 'checkbox',
          attrs: { title: 'Complete task', },
          children: [h({
            tag: 'span',
            class: 'checkbox-box',
          },),],
          on: {
            click: function onCheckboxClick(event,): void {
              event.stopPropagation();
              /**
               Caller-supplied complete handler; the click is a no-op when not provided.
               */
              const { onToggleComplete, } = options;
              if (onToggleComplete !== undefined) {
                void (async function onCheckboxClickAsync(): Promise<void> {
                  try {
                    await onToggleComplete(task.id,);
                  }
                  catch (error: unknown) {
                    console.error(
                      'toggle complete failed',
                      error,
                    );
                  }
                })();
              }
            },
          },
        },),
        h({
          tag: 'span',
          class: 'title',
          text: task.title,
        },),
      ],
      on: { click: function onRowClick(): void {
        options.onOpen(task.id,);
      }, },
    },),
    h({
      tag: 'div',
      class: 'chips',
      children: chipElements,
    },),
  );
}
