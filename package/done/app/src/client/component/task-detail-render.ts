/**
 DOM building helpers for the `<task-detail>` render method.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import type { Task, } from '../../lib/types.ts';

/**
 Options for building the task detail DOM tree.
 */
type RenderOptions = {
  /**
   Task being displayed.
   */
  readonly task: Task;
  /**
   Whether the component is in create mode.
   */
  readonly isCreate: boolean;
  /**
   Compiled CSS string for styles.
   */
  readonly styles: string;
};

/**
 References to dynamic elements within the rendered tree.
 */
export type RenderRefs = {
  /**
   Title text input.
   */
  titleInput: HTMLInputElement;
  /**
   Description textarea.
   */
  descInput: HTMLTextAreaElement;
  /**
   Container for metadata pill elements.
   */
  pillsContainer: HTMLElement;
  /**
   Action button row (start/stop/complete/delete).
   */
  btnRow: HTMLElement;
};

/**
 Result of building the task detail DOM tree.
 */
type RenderResult = {
  /**
   Top-level elements to insert into the shadow root.
   */
  elements: (HTMLElement | HTMLStyleElement)[];
  /**
   References to elements that need post-render interaction.
   */
  refs: RenderRefs;
};

/**
 Builds the complete `<task-detail>` Shadow DOM tree.
 
 Returns both the element list and refs to interactive elements
 so the caller can wire up event listeners without querying the DOM.
 
 @returns Elements and refs
 
 @example
 ```ts
 const { elements, refs } = buildTaskDetailTree({ task, isCreate: false, styles: [] });
 shadow.replaceChildren(...elements);
 ```
 */
export function buildTaskDetailTree(
  {
    task,
    isCreate,
    styles,
  }: RenderOptions,
): RenderResult {
  // Close button uses innerHTML for SVG because h() creates HTML-namespace
  // elements: SVG requires the SVG namespace.
  /**
   Reusable close button shell; the inline SVG glyph is injected on the next line.
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
   Exposed in refs so the host component can read live title edits.
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
   Exposed in refs so the host component can read live description edits.
   */
  const descInput = h({
    tag: 'textarea',
    class: 'desc-input',
    attrs: { placeholder: 'description', },
  },);
  if (task.description
    !== undefined)
    descInput.textContent = task.description;

  /**
   Mutable attribute map so a `disabled` flag can be appended conditionally below.
   */
  const startAttrs: Record<string, string> = { 'data-action': 'start', };
  if (task.timerStartedAt
    !== undefined)
    startAttrs.disabled = '';
  /**
   Mutable attribute map mirroring `startAttrs`; disabled when no timer is running.
   */
  const stopAttrs: Record<string, string> = { 'data-action': 'stop', };
  if (task.timerStartedAt
    === undefined)
    stopAttrs.disabled = '';
  /**
   Mutable attribute map disabled while any blocker remains unresolved.
   */
  const completeAttrs: Record<string, string> = { 'data-action': 'complete', };
  if (task.blockedBy
    .length
    > 0)
    completeAttrs.disabled = '';

  /**
   Exposed in refs so the host component can toggle button visibility per mode.
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

  /**
   Exposed in refs so the pill manager can replace its children without re-rendering the tree.
   */
  const pillsContainer = h({
    tag: 'div',
    class: 'pills',
  },);

  /**
   Ordered child list returned to the caller for `shadow.replaceChildren`.
   */
  const elements = [
    h({
      tag: 'style',
      text: styles,
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
    pillsContainer,
    btnRow,
  ];

  return {
    elements,
    refs: {
      titleInput,
      descInput,
      pillsContainer,
      btnRow,
    },
  };
}
