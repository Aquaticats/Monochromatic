/**
 Button row construction and action event wiring for task-detail.
 
 Extracted from task-detail-render.ts to keep each file
 under the line-count limit.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import type { Task, } from '../../lib/types.ts';

/**
 Builds the start/stop/complete/delete button row for a task.
 
 @param task - Task whose state determines button disabled states
 
 @param isCreate - Whether the detail panel is in create mode
 
 @returns Button row element
 
 @example
 ```ts
 const row = buildActionButtonRow({ task, isCreate: false });
 shadow.append(row);
 ```
 */
export function buildActionButtonRow(
  {
    task,
    isCreate,
  }: {
    readonly task: Task;
    readonly isCreate: boolean;
  },
): HTMLElement {
  /**
   Start button attrs; `disabled` is appended when a timer is already running.
   */
  const startAttrs: Record<string, string> = { 'data-action': 'start', };
  if (task.timerStartedAt
    !== undefined)
    startAttrs.disabled = '';
  /**
   Stop button attrs; `disabled` is appended when no timer is running.
   */
  const stopAttrs: Record<string, string> = { 'data-action': 'stop', };
  if (task.timerStartedAt
    === undefined)
    stopAttrs.disabled = '';
  /**
   Complete button attrs; `disabled` is appended when blockers remain.
   */
  const completeAttrs: Record<string, string> = { 'data-action': 'complete', };
  if (task.blockedBy
    .length
    > 0)
    completeAttrs.disabled = '';

  /**
   Row captured separately so the create-mode hidden flag can be toggled.
   */
  const btnRow = h({
    tag: 'div',
    class: 'btn-row',
    children: [
      h({
        tag: 'button',
        class: 'btn-outline',
        attrs: startAttrs,
        text: 'Start',
      },),
      h({
        tag: 'button',
        class: 'btn-outline',
        attrs: stopAttrs,
        text: 'Stop',
      },),
      h({
        tag: 'button',
        class: 'btn-primary',
        attrs: completeAttrs,
        text: 'Complete',
      },),
      h({
        tag: 'button',
        class: 'btn-outline',
        attrs: { 'data-action': 'delete', },
        text: 'Delete',
      },),
    ],
  },);
  if (isCreate)
    btnRow.dataset
      .hidden = '';
  return btnRow;
}

/**
 Wires a click-delegation handler on the shadow root that dispatches
 `action` custom events from `[data-action]` buttons.
 
 @param shadow - Shadow root to listen on
 
 @param host - Host element that receives the dispatched events
 
 @param titleInput - Title input whose value is included in the event detail
 
 @param descInput - Description textarea whose value is included in the event detail
 
 @mutates shadow - `shadow.addEventListener` changes listener state and retains callback.
 
 @mutates host - `shadow.addEventListener` retains callback that calls
 `DOM commit 5796f716 dispatchEvent invokes listeners with event` on host.
 
 @mutates titleInput - `shadow.addEventListener` retains callback that reads titleInput before
 `DOM commit 5796f716 dispatchEvent invokes listeners with event`.
 
 @mutates descInput - `shadow.addEventListener` retains callback that reads descInput before
 `DOM commit 5796f716 dispatchEvent invokes listeners with event`.
 
 @example
 ```ts
 attachActionHandler({ shadow, host, titleInput, descInput });
 ```
 */
export function attachActionHandler(
  {
    shadow,
    host,
    titleInput,
    descInput,
  }: {
    readonly shadow: ShadowRoot;
    readonly host: HTMLElement;
    readonly titleInput: HTMLInputElement;
    readonly descInput: HTMLTextAreaElement;
  },
): void {
  shadow.addEventListener(
    'click',
    function onAction(event,): void {
      /* oxlint-disable typescript/no-unsafe-type-assertion -- event.target is always an Element in shadow DOM click handlers */
      /**
       Click target narrowed to `HTMLElement` so `closest()` can be invoked.
       */
      const target = event.target as HTMLElement;
      /* oxlint-enable typescript/no-unsafe-type-assertion */
      /**
       Nearest `[data-action]` button ancestor, or `null` when the click was outside any action.
       */
      const button = target.closest<HTMLElement>('[data-action]',);
      if (button === null)
        return;
      /**
       Action name from the button's `data-action` attribute, forwarded to the custom event.
       */
      const { action, } = (button as HTMLElement).dataset;

      host.dispatchEvent(
        new CustomEvent(
          'action',
          {
            bubbles: true,
            detail: {
              action,
              title: titleInput.value,
              description: descInput.value,
            },
          },
        ),
      );
    },
  );
}
