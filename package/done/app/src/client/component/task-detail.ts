/**
 `<task-detail>` web component for viewing and editing a single task.
 */
import type {
  TaskComplexity,
  TaskPriority,
} from '../../lib/types.ts';
import {
  type AutofillController,
  createAutofillController,
} from './task-detail-autofill.ts';
import { buildPillElements, } from './task-detail-pills.ts';
import { buildTaskDetailTree, } from './task-detail-render.ts';
import { TASK_DETAIL_STYLES, } from './task-detail-styles.ts';
import {
  METADATA_UNSET,
  type MetadataState,
  type TaskDetailData,
  type TaskDetailMode,
} from './task-detail-types.ts';

/**
 `<task-detail>`: full-page task editor with title, description, metadata pills,
 action buttons (start/stop/complete/delete), and debounced AI autofill.
 */
class TaskDetail extends HTMLElement {
  /**
   Shadow root for encapsulated rendering.
   */
  readonly #shadow: ShadowRoot;

  /**
   Current task data and display configuration; absent before configure.
   */
  #data?: TaskDetailData;

  /**
   Whether the component is in edit or create mode.
   */
  #mode: TaskDetailMode = 'edit';

  /**
   Metadata state updated by autofill and user edits; reassigned wholesale.
   */
  #metadata: MetadataState = {
    tags: [],
    locations: [],
    priority: METADATA_UNSET,
    complexity: METADATA_UNSET,
  };

  /**
   Debounced AI autofill controller.
   */
  readonly #autofill: AutofillController;

  /**
   Initializes the shadow root and the {@link createAutofillController} autofill controller.
   */
  constructor() {
    super();
    this.#shadow = this.attachShadow({ mode: 'open', },);
    this.#autofill = createAutofillController({
      getState: this.#getMetadataState
        .bind(this,),
      setState: this.#applyMetadataState
        .bind(this,),
      updateDisplay: this.#updatePillsDisplay
        .bind(this,),
    },);
  }

  /**
   Snapshots current metadata for the autofill controller's empty-field check.
   
   @returns Current metadata values
   */
  #getMetadataState(): MetadataState {
    return this.#metadata;
  }

  /**
   Applies AI-suggested metadata; omitted fields are left unchanged.
   
   @param update - Fields the autofill controller wants to set
   */
  #applyMetadataState(
    update: {
      readonly tags?: readonly string[];
      readonly locations?: readonly string[];
      readonly priority?: TaskPriority;
      readonly complexity?: TaskComplexity;
    },
  ): void {
    this.#metadata = {
      tags: update.tags
        ?? this.#metadata
          .tags,
      locations: update.locations
        ?? this.#metadata
          .locations,
      priority: update.priority
        ?? this.#metadata
          .priority,
      complexity: update.complexity
        ?? this.#metadata
          .complexity,
    };
  }

  /**
   Sets task data, resets metadata state, and triggers a full render.
   
   @param data - Task data and mode configuration
   */
  configure(data: TaskDetailData,): void {
    console.log(
      '[task-detail] configure() called, mode:',
      data.mode
        ?? 'edit',
    );
    this.#data = data;
    this.#mode = data.mode
      ?? 'edit';
    this.#metadata = {
      tags: [...data.task
        .tags,],
      locations: [...data.task
        .locations,],
      priority: data.task
        .priority
        ?? METADATA_UNSET,
      complexity: data.task
        .complexity
        ?? METADATA_UNSET,
    };
    this.#autofill
      .clearAutofilled();
    this.#render();
  }

  /**
   Returns the current metadata state so the parent can include it in save payloads.
   
   @returns Current tags, locations, priority, and complexity
   */
  getMetadata(): MetadataState {
    return this.#metadata;
  }

  /**
   Rebuilds pill elements via {@link buildPillElements} in the `.pills`
   container from current metadata state.
   */
  #updatePillsDisplay(): void {
    /**
     Shadow-DOM lookup; null when the component has not rendered yet.
     */
    const pillsContainer = this.#shadow
      .querySelector<HTMLElement>('.pills',);
    if (pillsContainer === null)
      return;
    /**
     Current task snapshot; absent until the component has been hydrated.
     */
    const task = this.#data
      ?.task;
    if (task === undefined)
      return;

    /**
     Fresh pill list built from the latest metadata so the container can be wholesale-replaced.
     */
    const pillElements = buildPillElements({
      task,
      metadata: this.#metadata,
      autofillLoading: this.#autofill
        .loading,
      autofilled: this.#autofill
        .autofilled,
    },);
    pillsContainer.replaceChildren(...pillElements,);
  }

  /**
   Builds the complete Shadow DOM via {@link buildTaskDetailTree} and
   {@link TASK_DETAIL_STYLES}, and wires up event listeners.
   */
  #render(): void {
    /**
     Hydration payload captured once; early return below if not yet set.
     */
    const data = this.#data;
    if (data === undefined)
      return;
    /**
     Destructured task forwarded to the tree builder and downstream listeners.
     */
    const { task, } = data;
    /**
     Whether this render is for the create flow; affects header buttons.
     */
    const isCreate = this.#mode
      === 'create';

    /**
     Element list plus refs to interactive nodes that need post-render wiring.
     */
    const {
      elements,
      refs,
    } = buildTaskDetailTree({
      task,
      isCreate,
      styles: TASK_DETAIL_STYLES,
    },);
    this.#shadow
      .replaceChildren(...elements,);
    this.#updatePillsDisplay();

    /**
     Local alias so the input listener reaches the controller without capturing `this`.
     */
    const autofill = this.#autofill;
    /**
     Pre-bound dispatcher so the click handler can bubble events.
     */
    const dispatchFn = this.dispatchEvent
      .bind(this,);

    refs.titleInput
      .addEventListener(
      'input',
      function handleTitleInput(): void {
        autofill.request(refs.titleInput
          .value,);
      },
      );

    this.#shadow
      .addEventListener(
      'click',
      function handleActionClick(event: Event,): void {
        /**
         Click origin; type narrowed below before walking ancestors.
         */
        const { target, } = event;
        if (!(target instanceof HTMLElement))
          return;
        /**
         Nearest ancestor carrying a `data-action`, so children of the button still match.
         */
        const button = target.closest<HTMLElement>('[data-action]',);
        if (button === null)
          return;
        /**
         Action name forwarded as the event detail key.
         */
        const { action, } = button.dataset;
        dispatchFn(
          new CustomEvent(
            'action',
            {
              bubbles: true,
              detail: {
                action,
                title: refs.titleInput
                  .value,
                description: refs.descInput
                  .value,
              },
            },
          ),
        );
      },
      );
  }
}

customElements.define(
  'task-detail',
  TaskDetail,
);
console.log('[task-detail] custom element registered',);

export { TaskDetail, };
