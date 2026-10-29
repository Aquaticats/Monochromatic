/**
 `\<task-detail\>`: full-page task editor with title, description, metadata pills,
 action buttons (start/stop/complete/delete), and debounced AI autofill.
 */
import type {
  TaskComplexity,
  TaskPriority,
} from '../../lib/types.ts';
import {
  type AutofillController,
  createAutofillController,
} from './task-detail-autofill.ts';
import {
  buildPillData,
  buildPillElements,
} from './task-detail-pills.ts';
import { renderTaskDetail, } from './task-detail-render.ts';
import {
  METADATA_UNSET,
  type TaskDetailData,
  type TaskDetailMode,
} from './task-detail-types.ts';

/**
 `\<task-detail\>` web component for viewing and editing a single task.
 */
class TaskDetail extends HTMLElement {
  /**
   Shadow root for encapsulated rendering.
   */
  readonly #shadow: ShadowRoot;

  /**
   Current task configuration data.
   */
  #data?: TaskDetailData;

  /**
   Current display mode.
   */
  #mode: TaskDetailMode = 'edit';

  /**
   Mutable metadata state.
   */
  #tags: readonly string[] = [];

  /**
   Mutable locations state.
   */
  #locations: readonly string[] = [];

  /**
   Mutable priority state; {@link METADATA_UNSET} until a value is selected.
   */
  #priority: TaskPriority | typeof METADATA_UNSET = METADATA_UNSET;

  /**
   Mutable complexity state; {@link METADATA_UNSET} until a value is selected.
   */
  #complexity: TaskComplexity | typeof METADATA_UNSET = METADATA_UNSET;

  /**
   Autofill controller managing debounced AI requests.
   */
  readonly #autofill: AutofillController;

  /**
   Initializes the shadow root and autofill controller.
   */
  constructor() {
    super();
    this.#shadow = this.attachShadow({ mode: 'open', },);
    this.#autofill = createAutofillController({
      getState: this.#getAutofillState
        .bind(this,),
      setState: this.#applyAutofillState
        .bind(this,),
      updateDisplay: this.#updatePillsDisplay
        .bind(this,),
    },);
  }

  /**
   Snapshots current metadata for the autofill controller's empty-field check.
   
   @returns Current tags, locations, priority, and complexity
   */
  #getAutofillState(): {
    readonly tags: readonly string[];
    readonly locations: readonly string[];
    readonly priority: TaskPriority | typeof METADATA_UNSET;
    readonly complexity: TaskComplexity | typeof METADATA_UNSET;
  } {
    return {
      tags: this.#tags,
      locations: this.#locations,
      priority: this.#priority,
      complexity: this.#complexity,
    };
  }

  /**
   Applies AI-suggested metadata; omitted fields are left unchanged.
   
   @param update - Fields the autofill controller wants to set
   */
  #applyAutofillState(
    update: {
      readonly tags?: readonly string[];
      readonly locations?: readonly string[];
      readonly priority?: TaskPriority;
      readonly complexity?: TaskComplexity;
    },
  ): void {
    if (update.tags
      !== undefined)
      this.#tags = update.tags;
    if (update.locations
      !== undefined)
      this.#locations = update.locations;
    if (update.priority
      !== undefined)
      this.#priority = update.priority;
    if (update.complexity
      !== undefined)
      this.#complexity = update.complexity;
  }

  /**
   Sets task data, resets metadata state, and triggers a full render.
   
   @param data - Task data and configuration
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
    this.#tags = [...data.task
      .tags,];
    this.#locations = [...data.task
      .locations,];
    this.#priority = data.task
      .priority
      ?? METADATA_UNSET;
    this.#complexity = data.task
      .complexity
      ?? METADATA_UNSET;
    this.#autofill
      .clearAutofilled();
    this.#render();
  }

  /**
   Returns the current metadata state for save payloads.
   
   @returns Current metadata values
   */
  getMetadata(): {
    readonly tags: readonly string[];
    readonly locations: readonly string[];
    readonly priority: TaskPriority | typeof METADATA_UNSET;
    readonly complexity: TaskComplexity | typeof METADATA_UNSET;
  } {
    return {
      tags: this.#tags,
      locations: this.#locations,
      priority: this.#priority,
      complexity: this.#complexity,
    };
  }

  /**
   Rebuilds pill elements from current metadata state.
   */
  #updatePillsDisplay(): void {
    /**
     Container the pill list is rendered into; absent before initial render.
     */
    const pillsContainer = this.#shadow
      .querySelector<HTMLElement>('.pills',);
    if (pillsContainer === null)
      return;
    /**
     Task data extracted from `#data`; updates are skipped until {@link TaskDetail.configure} has run.
     */
    const task = this.#data
      ?.task;
    if (task === undefined)
      return;

    /**
     Descriptor list driving the pill render below.
     */
    const pills = buildPillData({
      task,
      tags: this.#tags,
      locations: this.#locations,
      priority: this.#priority,
      complexity: this.#complexity,
    },);
    pillsContainer.replaceChildren(
      ...buildPillElements({
        pills,
        loading: this.#autofill
          .loading,
        autofilled: this.#autofill
          .autofilled,
      },),
    );
  }

  /**
   Delegates to renderTaskDetail and wires autofill on title input.
   */
  #render(): void {
    /**
     Snapshot of `#data` to satisfy the absence check and stable destructure below.
     */
    const data = this.#data;
    if (data === undefined)
      return;
    /**
     Title input returned by the shared renderer; autofill is wired on its `input` events.
     */
    const { titleInput, } = renderTaskDetail({
      shadow: this.#shadow,
      task: data.task,
      mode: this.#mode,
      host: this,
    },);
    this.#updatePillsDisplay();
    /**
     Captured so the input listener reaches the controller without a `this`-bound handler.
     */
    const autofill = this.#autofill;
    titleInput.addEventListener(
      'input',
      function onTitleInput(): void {
        autofill.request(titleInput.value,);
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
