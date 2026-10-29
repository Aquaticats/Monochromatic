/**
 Pill element builder for the `<task-detail>` web component.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import type { Task, } from '../../lib/types.ts';
import { formatRunningTrackedTime, } from '../lib/task-card.ts';
import {
  METADATA_UNSET,
  type MetadataState,
} from './task-detail-types.ts';

/**
 Builds pill elements from current metadata state and autofill status,
 formatting the tracked-time pill via {@link formatRunningTrackedTime}.
 
 @returns Array of pill span elements
 
 @example
 ```ts
 const pills = buildPillElements({ task, metadata, autofillLoading: false, autofilled: true });
 container.replaceChildren(...pills);
 ```
 */
export function buildPillElements({
  task,
  metadata,
  autofillLoading,
  autofilled,
}: {
  readonly task: Task;
  readonly metadata: MetadataState;
  readonly autofillLoading: boolean;
  readonly autofilled: ReadonlySet<string>;
},): HTMLElement[] {
  /**
   Ordered pill descriptors; field names align with the autofilled set lookup below.
   */
  const pillData = [
    {
      field: 'tags',
      text: metadata.tags
        .length
        > 0 ? `# ${metadata.tags
          .join(', ',)}` : '# ?',
    },
    {
      field: 'tracked',
      text: `tracked: ${formatRunningTrackedTime(task,)}`,
    },
    {
      field: 'locations',
      text: metadata.locations
        .length
        > 0
        ? `where: ${metadata.locations
          .join(', ',)}`
        : 'where: ?',
    },
    {
      field: 'priority',
      text: `priority: ${metadata.priority
          === METADATA_UNSET
        ? '?'
        : metadata.priority}`,
    },
    {
      field: 'due',
      text: `due: ${task.dueDate
        ?? '?'}`,
    },
    {
      field: 'complexity',
      text: `complexity: ${metadata.complexity
          === METADATA_UNSET
        ? '?'
        : metadata.complexity}`,
    },
    {
      field: 'reminders',
      text: task.reminders
        .length
        > 0
        ? `reminders: ${task.reminders[0]}`
        : 'reminders: None',
    },
    {
      field: 'blockedBy',
      text: task.blockedBy
        .length
        > 0
        ? `blockedBy: ${String(task.blockedBy
          .length,)}`
        : 'blockedBy: none',
    },
  ] as const;

  return pillData.map(function toPillElement(pill,) {
    /**
     Built once so the conditional data attributes below can be appended in place.
     */
    const element = h({
      tag: 'span',
      class: 'pill',
      text: pill.text,
    },);
    if (autofillLoading)
      element.dataset
        .loading = '';
    else if (autofilled.has(pill.field,))
      element.dataset
        .autofilled = '';
    return element;
  },);
}
