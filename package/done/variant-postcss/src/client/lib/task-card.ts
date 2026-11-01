/**
 `\<task-card\>` web component and its factory function `createTaskCard()`.
 
 Unlike other web components that live in `components/`, task-card is in `lib/`
 because it's always created programmatically (never placed in server HTML).
 Client entry scripts call `createTaskCard(task, options)` which creates the
 element, calls `configure()` to pass data, and returns it for appending to the DOM.
 */
import type { Task, } from '../../lib/types.ts';
import {
  renderTaskCardContent,
  type TaskCardOptions,
} from './task-card-render.ts';

export { formatRunningTrackedTime, } from './format-tracked-time.ts';

/**
 Sentinel returned by {@link TaskCard.getChipElement} when no chip matches the prefix.
 */
const CHIP_NOT_FOUND: unique symbol = Symbol('metadata chip element absent for prefix',);

/**
 `\<task-card\>` -- displays a task as a clickable card with checkbox, title, and metadata chips.
 */
class TaskCard extends HTMLElement {
  /**
   Shadow root for encapsulated rendering.
   */
  readonly #shadow: ShadowRoot;

  /**
   Currently displayed task; absent before configuration.
   */
  #task?: Task;

  /**
   Current rendering options; absent before configuration.
   */
  #options?: TaskCardOptions;

  /**
   Initializes the shadow root for encapsulated rendering.
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
   
   @param prefix - Text prefix to match (e.g. `"tracked:"`)
   
   @returns Matching chip element, or {@link CHIP_NOT_FOUND} when none matches
   */
  getChipElement(prefix: string,): HTMLSpanElement | typeof CHIP_NOT_FOUND {
    for (const chip of this.#shadow
      .querySelectorAll<HTMLSpanElement>('.chip',)) {
      if (chip.textContent
        .startsWith(prefix,)) {
        return chip;
      }
    }
    return CHIP_NOT_FOUND;
  }

  /**
   Renders the full card content into the shadow root.
   */
  #render(): void {
    /**
     Local snapshot used for the absence guard and the renderer call below.
     */
    const task = this.#task;
    /**
     Local snapshot used for the absence guard and the renderer call below.
     */
    const options = this.#options;
    if ((task === undefined) || (options === undefined))
      return;
    renderTaskCardContent({
      shadow: this.#shadow,
      task,
      options,
    },);
  }
}

customElements.define(
  'task-card',
  TaskCard,
);

/**
 Factory: creates and configures a `\<task-card\>` element.
 
 @param task - Task data to display
 
 @param options - Callbacks for open/complete interactions
 
 @returns Configured {@link TaskCard} element
 
 @example
 ```ts
 const card = createTaskCard({ task, options: { onOpen: openTask, onToggleComplete: completeTask } });
 list.append(card);
 ```
 */
export function createTaskCard(
  {
    task,
    options,
  }: {
    readonly task: Task;
    readonly options: TaskCardOptions;
  },
): TaskCard {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- custom element registered as "task-card" */
  /**
   Configured custom element returned to the caller for DOM insertion.
   */
  const card = document.createElement('task-card',) as TaskCard;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  card.configure(
    task,
    options,
  );
  return card;
}
