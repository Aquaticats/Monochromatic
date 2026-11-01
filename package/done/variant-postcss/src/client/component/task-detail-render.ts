/**
 Render function for the `\<task-detail\>` Shadow DOM tree.
 
 Builds the layout: header with close/save buttons, title input,
 description textarea, attach/photo actions, pills container, and
 action button row (delegated to task-detail-render-actions.ts).
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import type { Task, } from '../../lib/types.ts';
import {
  attachActionHandler,
  buildActionButtonRow,
} from './task-detail-render-actions.ts';
import { TASK_DETAIL_STYLES, } from './task-detail-styles.ts';
import type { TaskDetailMode, } from './task-detail-types.ts';

/**
 References to interactive elements needed by the caller after render.
 */
export type RenderResult = {
  /**
   Title input for wiring autofill.
   */
  titleInput: HTMLInputElement;
  /**
   Description textarea for reading on save.
   */
  descInput: HTMLTextAreaElement;
};

/**
 Builds the full task-detail Shadow DOM content.
 
 @param options - Shadow root, task data, display mode, and host element.
 
 @returns References to title and description inputs
 
 @mutates options - Documents uncertainty propagated from
 `DOM commit 5796f716 dispatchEvent invokes listeners with event` and `shadow.addEventListener`.
 
 @example
 ```ts
 const { titleInput } = renderTaskDetail({ shadow, task, mode: 'edit', host: this });
 ```
 */
export function renderTaskDetail(
  options: {
    readonly shadow: ShadowRoot;
    readonly task: Task;
    readonly mode: TaskDetailMode;
    readonly host: HTMLElement;
  },
): RenderResult {
  /**
   Render inputs separated after boundary contract attaches to their containing input.
   */
  const {
    shadow,
    task,
    mode,
    host,
  } = options;
  /**
   Branches header label, button class, and button text between create and edit.
   */
  const isCreate = mode === 'create';

  // Close button uses innerHTML for SVG because h() creates HTML-namespace
  // elements. SVG requires the SVG namespace.
  /**
   Header close button captured separately so innerHTML can be assigned after construction.
   */
  const closeButton = h({
    tag: 'button',
    class: 'close',
    attrs: {
      'data-action': 'close',
      'aria-label': 'Close',
    },
  },);
  closeButton.innerHTML =
    `<svg viewBox="0 0 48 48" fill="none"><line x1="14" y1="14" x2="34" y2="34"/><line x1="34" y1="14" x2="14" y2="34"/></svg>`;

  /**
   Title input retained so the typed wrapper and caller can reach it.
   */
  const titleInput = h({
    tag: 'input',
    class: 'title-input',
    attrs: {
      type: 'text',
      value: task.title,
      placeholder: 'Title',
      required: '',
    },
  },);

  /**
   Description textarea; textContent set below when the task already has a description.
   */
  const descInput = h({
    tag: 'textarea',
    class: 'desc-input',
    attrs: { placeholder: 'description', },
  },);
  if (task.description
    !== undefined)
    descInput.textContent = task.description;

  shadow.replaceChildren(
    h({
      tag: 'style',
      text: TASK_DETAIL_STYLES,
    },),
    h({
      tag: 'div',
      class: 'header',
      children: [
        closeButton,
        h({
          tag: 'span',
          class: 'heading',
          text: isCreate ? 'New task' : 'Task details',
        },),
        h({
          tag: 'button',
          class: isCreate ? 'btn-primary' : 'btn-outline',
          attrs: { 'data-action': 'save', },
          text: isCreate ? 'Create' : 'Save',
        },),
      ],
    },),
    titleInput,
    descInput,
    h({
      tag: 'div',
      class: 'actions',
      children: [
        h({
          tag: 'button',
          class: 'btn-outline',
          attrs: { 'data-action': 'attach', },
          text: 'Attach file',
        },),
        h({
          tag: 'button',
          class: 'btn-outline',
          attrs: { 'data-action': 'photo', },
          text: 'Take photo',
        },),
      ],
    },),
    h({
      tag: 'div',
      class: 'pills',
    },),
    buildActionButtonRow({
      task,
      isCreate,
    },),
  );

  /**
   Typed alias of `titleInput` so the action handler and return value receive the precise element type.
   */
  const typedTitleInput = titleInput as HTMLInputElement;
  /**
   Typed alias of `descInput` so callers reading `.value` see the textarea-specific API.
   */
  const typedDescInput = descInput as HTMLTextAreaElement;

  attachActionHandler({
    shadow,
    host,
    titleInput: typedTitleInput,
    descInput: typedDescInput,
  },);

  return {
    titleInput: typedTitleInput,
    descInput: typedDescInput,
  };
}
