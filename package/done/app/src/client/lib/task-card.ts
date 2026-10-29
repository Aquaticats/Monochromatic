/**
 `<task-card>` web component and its factory function `createTaskCard()`.
 
 Unlike other web components that live in `components/`, task-card is in `lib/`
 because it's always created programmatically (never placed in server HTML).
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import { MS_PER_SECOND, } from '@monochromatic-dev/const-core/ts';

import type { Task, } from '../../lib/types.ts';
import {
  buildChipTexts,
  formatTrackedTime,
  type TaskCardOptions,
} from './task-card-helpers.ts';
import { TASK_CARD_STYLES, } from './task-card-styles.ts';

/**
 Sentinel returned by `getChipElement` when no chip matches the prefix.
 */
const CHIP_NOT_FOUND: unique symbol = Symbol('task card chip element not found',);

/**
 `<task-card>`: displays a task as a clickable card with checkbox, title, and metadata chips.
 Created programmatically via {@link createTaskCard}, not placed in server HTML.
 */
class TaskCard extends HTMLElement {
  /**
   Shadow root for encapsulated rendering.
   */
  readonly #shadow: ShadowRoot;

  /**
   Task data to display, set via `configure()`; absent before configure.
   */
  #task?: Task;

  /**
   Interaction callbacks and display flags, set via `configure()`; absent before configure.
   */
  #options?: TaskCardOptions;

  /**
   Initializes the shadow root.
   */
  constructor() {
    super();
    this.#shadow = this.attachShadow({ mode: 'open', },);
  }

  /**
   Sets the task data and rendering options, then triggers a full render.
   
   @param task - Task to display
   
   @param options - Callbacks and display flags
   */
  configure(
    task: Task,
    options: TaskCardOptions,
  ): void {
    this.#task = task;
    this.#options = options;
    this.#render();
  }

  /**
   Finds a chip element whose text starts with the given prefix.
   Used by the in-progress timer to update the "tracked:" chip live.
   
   @param prefix - Text prefix to match (e.g. `"tracked:"`)
   
   @returns Matching chip span, or {@link CHIP_NOT_FOUND} when none matches
   */
  getChipElement(prefix: string,): HTMLSpanElement | typeof CHIP_NOT_FOUND {
    for (const chip of this.#shadow
      .querySelectorAll<HTMLSpanElement>('.chip',)) {
      if (chip.textContent
        .startsWith(prefix,))
        return chip;
    }
    return CHIP_NOT_FOUND;
  }

  /**
   Renders the card content (checkbox, title, chips built via
   {@link buildChipTexts}) into the shadow root.
   */
  #render(): void {
    /**
     Snapshot of the configured task; early-returns below if not yet set.
     */
    const task = this.#task;
    /**
     Snapshot of the configured options; early-returns below if not yet set.
     */
    const options = this.#options;
    if ((task === undefined) || (options === undefined))
      return;

    /**
     Chip label strings built from the task fields.
     */
    const chipTexts = buildChipTexts(task,);
    /**
     Chip DOM nodes mutated below when the blocked badge needs to be appended.
     */
    const chipElements: HTMLElement[] = chipTexts.map(function toChipElement(text,) {
      return h({
        tag: 'span',
        class: 'chip',
        text,
      },);
    },);
    if (options.showBlockedBadge
      === true) {
      chipElements.push(h({
        tag: 'span',
        class: 'chip blocked',
        text: 'blocked',
      },),);
    }

    this.#shadow
      .replaceChildren(
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
              click: function handleCheckboxClick(event,): void {
                event.stopPropagation();
                if (options.onToggleComplete
                  !== undefined)
                  void options.onToggleComplete(task.id,);
              },
            },
          },),
          h({
            tag: 'span',
            class: 'title',
            text: task.title,
          },),
        ],
        on: {
          click: function handleCardClick() {
            options.onOpen(task.id,);
          },
        },
      },),
      h({
        tag: 'div',
        class: 'chips',
        children: chipElements,
      },),
      );
  }
}

customElements.define(
  'task-card',
  TaskCard,
);

/**
 Factory: creates and configures a `<task-card>` element ready for DOM insertion.
 
 @param task - Task data to display
 
 @param options - Callbacks for open/complete interactions
 
 @returns Configured task-card element
 
 @example
 ```ts
 const card = createTaskCard({ task, options: { onOpen: handleOpen, onToggleComplete: handleComplete, }, });
 list.append(card);
 ```
 */
export function createTaskCard({
  task,
  options,
}: {
  readonly task: Task;
  readonly options: TaskCardOptions;
},): TaskCard {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- createElement returns HTMLElement but task-card is registered as TaskCard */
  /**
   Live `TaskCard` instance so the imperative `configure()` API is reachable.
   */
  const card = document.createElement('task-card',) as TaskCard;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  card.configure(
    task,
    options,
  );
  return card;
}

/**
 Formats tracked time including elapsed seconds from a running timer.
 If no timer is active, returns the static `trackedTime` formatted via
 {@link formatTrackedTime}.
 
 @param task - Task with optional running timer
 
 @returns Formatted tracked time string
 
 @example
 const label = formatRunningTrackedTime(task);
 // '2:15:00'
 */
export function formatRunningTrackedTime(task: Task,): string {
  if (task.timerStartedAt
    === undefined)
    return formatTrackedTime(task.trackedTime,);

  /**
   Live tick since the timer started; clamped so a clock skew never produces negatives.
   */
  const elapsedSeconds = Math.max(
    0,
    Math.floor((Date.now()
      - Date
        .parse(task.timerStartedAt,)) / MS_PER_SECOND,),
  );
  return formatTrackedTime(task.trackedTime
    + elapsedSeconds,);
}
