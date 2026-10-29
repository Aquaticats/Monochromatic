/**
 Autofill controller for the `\<task-detail\>` component.
 
 Manages debounced AI metadata suggestion requests with
 abort support and loading state tracking.
 */
import type {
  TaskComplexity,
  TaskPriority,
} from '../../lib/types.ts';
import {
  type AutofillResult,
  METADATA_UNSET,
} from './task-detail-types.ts';

/**
 Delay before triggering AI autofill after the user stops typing.
 */
const AUTOFILL_DEBOUNCE_MS = 500;

/**
 Sentinel for "no debounce timer is scheduled".
 */
const NO_TIMER: unique symbol = Symbol('autofill debounce timer not currently scheduled',);

/**
 Sentinel for "no request is in flight".
 */
const NO_ABORT: unique symbol = Symbol('autofill fetch request abort controller absent',);

/**
 Callback interface for the autofill controller to update the host component.
 */
export type AutofillCallbacks = {
  /**
   Returns current metadata values.
   */
  readonly getState: () => {
    readonly tags: readonly string[];
    readonly locations: readonly string[];
    readonly priority: TaskPriority | typeof METADATA_UNSET;
    readonly complexity: TaskComplexity | typeof METADATA_UNSET;
  };
  /**
   Applies new metadata values; omitted fields are left unchanged.
   */
  readonly setState: (
    update: {
      readonly tags?: readonly string[];
      readonly locations?: readonly string[];
      readonly priority?: TaskPriority;
      readonly complexity?: TaskComplexity;
    },
  ) => void;
  /**
   Refreshes the pill display.
   */
  readonly updateDisplay: () => void;
};

/**
 Public surface returned by {@link createAutofillController}.
 */
export type AutofillController = {
  /**
   Debounces an autofill request for the given title.
   */
  readonly request: (title: string,) => void;
  /**
   Whether an autofill request is currently in flight.
   */
  readonly loading: boolean;
  /**
   Field names that were filled by the most recent AI suggestion.
   */
  readonly autofilled: ReadonlySet<string>;
  /**
   Clears the autofilled-field tracking set.
   */
  readonly clearAutofilled: () => void;
};

/**
 Creates a debounced autofill controller.
 
 Delegates display and state reads/writes to the host via `callbacks`,
 keeping its own mutable timer/abort/loading state in the closure.
 
 @param callbacks - Host component callbacks
 
 @returns Controller surface for the host to drive
 
 @example
 ```ts
 const autofill = createAutofillController(callbacks);
 autofill.request('Buy groceries');
 ```
 */
export function createAutofillController(callbacks: AutofillCallbacks,): AutofillController {
  /**
   Mutable controller state captured by the closure below.
   */
  const state: {
    readonly autofilled: Set<string>;
    loading: boolean;
    timer: ReturnType<typeof setTimeout> | typeof NO_TIMER;
    abort: AbortController | typeof NO_ABORT;
  } = {
    autofilled: new Set<string>(),
    loading: false,
    timer: NO_TIMER,
    abort: NO_ABORT,
  };

  /**
   Sends an autofill request and merges results into empty metadata fields.
   
   @param title - Trimmed title text
   */
  async function fetchSuggestions(title: string,): Promise<void> {
    /**
     Captured locally so the abort signal can be reused for the in-flight check and abort.
     */
    const controller = new AbortController();
    state.abort = controller;
    state.loading = true;
    callbacks.updateDisplay();

    try {
      /**
       Server response containing the AI-suggested metadata payload.
       */
      const response = await fetch(
        '/api/ai/autofill',
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json', },
          body: JSON.stringify({ title, },),
          signal: controller.signal,
        },
      );

      if (response.ok) {
        /* oxlint-disable typescript/no-unsafe-type-assertion -- API response shape enforced by the server endpoint contract */
        /**
         Parsed autofill payload, cast to {@link AutofillResult}; shape is enforced by the server endpoint contract.
         */
        const result = (await response.json()) as AutofillResult;
        /* oxlint-enable typescript/no-unsafe-type-assertion */
        state.autofilled
          .clear();
        /**
         Current field values from the host so empty-field merging can be done.
         */
        const current = callbacks.getState();
        /**
         Patch accumulating only fields that were empty before the AI suggestion.
         */
        const update: {
          tags?: readonly string[];
          locations?: readonly string[];
          priority?: TaskPriority;
          complexity?: TaskComplexity;
        } = {};

        if ((result.tags
          .length
          > 0) && (current.tags
            .length
            === 0)) {
          update.tags = result.tags;
          state.autofilled
            .add('tags',);
        }
        if ((result.locations
          .length
          > 0) && (current.locations
            .length
            === 0)) {
          update.locations = result.locations;
          state.autofilled
            .add('locations',);
        }
        if ((result.priority
          !== undefined) && (current.priority
            === METADATA_UNSET)) {
          update.priority = result.priority;
          state.autofilled
            .add('priority',);
        }
        if ((result.complexity
          !== undefined) && (current.complexity
            === METADATA_UNSET)) {
          update.complexity = result.complexity;
          state.autofilled
            .add('complexity',);
        }

        callbacks.setState(update,);
      }
    }
    catch (error: unknown) {
      if (!((error instanceof DOMException) && (error.name
        === 'AbortError'))) {
        console.error(
          'Autofill request failed:',
          error,
        );
      }
    }

    state.loading = false;
    callbacks.updateDisplay();
  }

  /**
   Debounces an autofill request.
   
   @param title - Current title input value
   */
  function request(title: string,): void {
    if (state.timer
      !== NO_TIMER)
      clearTimeout(state.timer,);
    if (state.abort
      !== NO_ABORT) {
      state.abort
        .abort();
      state.abort = NO_ABORT;
    }
    if (title.trim()
      .length
      === 0)
      return;

    state.timer = setTimeout(
      function triggerAutofill(): void {
        void fetchSuggestions(title.trim(),);
      },
      AUTOFILL_DEBOUNCE_MS,
    );
  }

  /**
   Clears the autofilled-field tracking set.
   */
  function clearAutofilled(): void {
    state.autofilled
      .clear();
  }

  return {
    request,
    /**
     Whether an autofill request is currently in flight.
     */
    get loading(): boolean {
      return state.loading;
    },
    /**
     Field names filled by the most recent AI suggestion.
     */
    get autofilled(): ReadonlySet<string> {
      return state.autofilled;
    },
    clearAutofilled,
  };
}
