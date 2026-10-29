/**
 * `<search-overlay>` web component.
 *
 * Self-contained modal search dialog that detects double-Shift internally.
 * Prefix the query with `%` to search file contents only.
 * Dispatches a `result-select` CustomEvent when the user picks a result.
 */

import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';

import type { SearchResult, } from '../../../protocol.ts';
import {
  l as rootLogger,
  tagged,
} from '../log.ts';
import {
  buildResultDetail,
  handleSearchKeydown,
  moveSearchSelection,
} from './nav.ts';
import {
  highlightMatches,
  renderResultElements,
} from './render.ts';
import { STYLES, } from './search-overlay.styles.ts';
import {
  performSearch,
  scheduleSearch,
  type SearchState,
} from './search.ts';

export type { ResultSelectDetail, } from './nav.ts';

/**
 * Tagged logger for the search overlay subsystem.
 */
const l = tagged({
  tag: 'search-overlay',
  l: rootLogger,
},);
/**
 * Maximum milliseconds between two Shift keyup events to count as a double-shift.
 */
const DOUBLE_SHIFT_THRESHOLD_MS = 400;

/**
 * `<search-overlay>`: self-contained modal search dialog.
 */
export class SearchOverlay extends HTMLElement {
  /**
   * Shadow root for encapsulated rendering.
   */
  readonly #shadow: ShadowRoot;
  /**
   * Modal dialog element containing the search UI.
   */
  #dialog: HTMLDialogElement | null = null;
  /**
   * Text input element for the search query.
   */
  #input: HTMLInputElement | null = null;
  /**
   * Container div for rendered search result elements.
   */
  #resultsContainer: HTMLDivElement | null = null;
  /**
   * 0-based index of the currently highlighted result (-1 = none).
   */
  #selectedIndex = -1;
  /**
   * Cached search results from the last query.
   */
  #results: readonly SearchResult[] = [];
  /**
   * Root directory path used to compute relative display paths.
   */
  #rootDir = '';
  /**
   * Width of a single monospace character in pixels for budget calculation.
   */
  #charWidthPx = 0;
  /**
   * Timestamp of the last Shift keyup event for double-shift detection.
   */
  #lastShiftUp = 0;
  /**
   * True if a non-Shift key was pressed between two Shift releases.
   */
  #interveningKey = false;
  /**
   * Mutable debounce state shared across search invocations.
   */
  readonly #searchState: SearchState = {
    debouncedSearch: null,
    searchGeneration: 0,
  };
  /**
   * Bound global keydown handler for cleanup in disconnectedCallback.
   */
  #boundKeydown: ((event: KeyboardEvent,) => void) | null = null;
  /**
   * Bound global keyup handler for cleanup in disconnectedCallback.
   */
  #boundKeyup: ((event: KeyboardEvent,) => void) | null = null;
  /**
   * Callback that performs a search and returns results.
   */
  onSearch: ((query: string,) => Promise<readonly SearchResult[]>) | null = null;
  /**
   * Callback that returns the current search scope directory.
   */
  getRootDir: (() => string) | null = null;

  /**
   * Initializes the shadow root.
   */
  constructor() {
    super();
    this.#shadow = this.attachShadow({ mode: 'open', },);
  }

  /**
   * Renders the dialog and attaches keyboard listeners for double-shift.
   */
  connectedCallback(): void {
    /**
     * Local alias for `this`; captured by the input/dialog event handler closures.
     */
    const overlay = this;
    this.#input = h({
      tag: 'input',
      class: 'search-input',
      attrs: {
        type: 'text',
        placeholder: 'Search files... (prefix with % for content only)',
        autocomplete: 'off',
      },
      on: {
        input: function handleInput() {
          overlay.#scheduleSearch();
        },
        keydown: function handleInputKeydown(event,) {
          handleSearchKeydown({
            event,
            moveSelection: function move(delta,) {
              overlay.#moveSelection({ delta, },);
            },
            confirmSelection: function confirm() {
              overlay.#confirmSelection();
            },
          },);
        },
        blur: function handleInputBlur(event,) {
          /**
           * Element receiving focus next; null when focus left the document entirely.
           */
          const related = event.relatedTarget;
          if ((related === null)
            || (!(related instanceof Node))
            || (overlay.#dialog
              ?.contains(related,)
              !== true))
          {
            overlay.#close();
          }
        },
      },
    },);
    this.#resultsContainer = h({
      tag: 'div',
      class: 'results',
    },);
    this.#dialog = h({
      tag: 'dialog',
      children: [
        this.#input,
        this.#resultsContainer,
      ],
      on: {
        close: function handleClose() {
          l.info('overlay closed',);
        },
      },
    },);
    this.#shadow
      .replaceChildren(
      h({
        tag: 'style',
        text: STYLES,
      },),
      this.#dialog,
    );
    this.#boundKeydown = function handleGlobalKeydown(event: KeyboardEvent,): void {
      if (event.key
        !== 'Shift')
        overlay.#interveningKey = true;
    };
    this.#boundKeyup = function handleGlobalKeyup(event: KeyboardEvent,): void {
      if (event.key
        !== 'Shift')
        return;
      /**
       * Current timestamp; compared against `lastShiftUp` to detect a double-shift within the threshold.
       */
      const now = Date.now();
      if ((!overlay.#interveningKey)
        && (overlay.#lastShiftUp
          > 0)
        && ((now - overlay
          .#lastShiftUp) < DOUBLE_SHIFT_THRESHOLD_MS))
      {
        overlay.#lastShiftUp = 0;
        overlay.#interveningKey = false;
        overlay.#show();
        return;
      }
      overlay.#lastShiftUp = now;
      overlay.#interveningKey = false;
    };
    document.addEventListener(
      'keydown',
      this.#boundKeydown,
    );
    document.addEventListener(
      'keyup',
      this.#boundKeyup,
    );
  }

  /**
   * Removes global keyboard listeners added in connectedCallback.
   */
  disconnectedCallback(): void {
    if (this.#boundKeydown
      !== null) {
      document.removeEventListener(
        'keydown',
        this.#boundKeydown,
      );
    }
    if (this.#boundKeyup
      !== null) {
      document.removeEventListener(
        'keyup',
        this.#boundKeyup,
      );
    }
  }

  /**
   * Opens the overlay.
   */
  #show(): void {
    this.#rootDir = this.getRootDir?.()
      ?? '';
    if ((this.#dialog
      === null)
      || (this.#input
        === null)
      || (this.#resultsContainer
        === null))
    {
      return;
    }
    this.#input
      .value = '';
    this.#resultsContainer
      .replaceChildren();
    this.#results = [];
    this.#selectedIndex = -1;
    this.#dialog
      .showModal();
    this.#measureCharWidth();
    this.#input
      .focus();
    l.info('overlay opened',);
  }

  /**
   * Closes the search overlay.
   */
  #close(): void {
    if (this.#dialog
      !== null)
      this.#dialog
        .close();
  }

  /**
   * Measures the width of a single monospace character.
   * Skips re-measurement when a valid cached value already exists,
   * since the dialog font does not change between opens.
   */
  #measureCharWidth(): void {
    if (this.#charWidthPx
      > 0)
      return;
    if (this.#dialog
      === null)
      return;
    /**
     * Resolved `font` shorthand for the dialog; passed straight to the canvas context for measurement.
     */
    const { font, } = getComputedStyle(this.#dialog,);
    /**
     * Off-screen canvas used as a measurement surface; never attached to the document.
     */
    const canvas = document.createElement('canvas',);
    /**
     * 2D drawing context used for `measureText`; null when the browser denies the canvas.
     */
    const ctx = canvas.getContext('2d',);
    if (ctx === null)
      return;
    ctx.font = font;
    this.#charWidthPx = ctx.measureText('0',)
      .width;
  }

  /**
   * Computes how many monospace characters fit in one result row.
   *
   * @returns number of monospace characters that fit in one result row
   */
  #charBudget(): number {
    if ((this.#charWidthPx
      <= 0) || (this.#resultsContainer
        === null))
      return 0;
    return Math.floor(this.#resultsContainer
      .clientWidth
      / this
      .#charWidthPx,);
  }

  /**
   * Schedules a debounced search.
   */
  #scheduleSearch(): void {
    /**
     * Local alias for `this`; captured by the `execute` callback so it can call back into the instance.
     */
    const overlay = this;
    scheduleSearch({
      state: this.#searchState,
      execute: function run() {
        void overlay.#performSearch();
      },
    },);
  }

  /**
   * Reads the input value, invokes `onSearch`, and renders results.
   */
  async #performSearch(): Promise<void> {
    if ((this.onSearch
      === null) || (this.#input
        === null))
      return;
    /**
     * Local alias for `this`; captured by the `onResults` callback.
     */
    const overlay = this;
    await performSearch({
      raw: this.#input
        .value,
      state: this.#searchState,
      onSearch: this.onSearch,
      onResults: function render(opts,) {
        overlay.#renderResults(opts,);
      },
    },);
  }

  /**
   * Renders search results into the results container.
   */
  #renderResults(
    {
      results,
      query,
    }: {
      readonly results: readonly SearchResult[];
      readonly query: string;
    },
  ): void {
    if (this.#resultsContainer
      === null)
      return;
    this.#results = results;
    this.#selectedIndex = results.length
      > 0 ? 0 : -1;
    if (results.length
      === 0) {
      /**
       * True when the input contains non-whitespace text; selects the "No results" message over a blank placeholder.
       */
      const hasInput = (this.#input
        !== null) && (this.#input
          .value
          .trim()
          !== '');
      this.#resultsContainer
        .replaceChildren(
        hasInput
          ? h({
            tag: 'div',
            class: 'empty',
            text: 'No results',
          },)
          : h({ tag: 'div', },),
      );
      return;
    }
    /**
     * Local alias for `this`; captured by the `onSelect` callback.
     */
    const overlay = this;
    /**
     * Root directory with a guaranteed trailing slash; stripped from result paths when displaying them.
     */
    const rootPrefix = this.#rootDir
      .endsWith('/',) ? this.#rootDir : `${this.#rootDir}/`;
    /**
     * Rendered DOM nodes for each result row, ready to swap into `resultsContainer`.
     */
    const elements = renderResultElements({
      results,
      query,
      rootPrefix,
      budget: this.#charBudget(),
      onSelect: function select(index,) {
        overlay.#selectResult({ index, },);
      },
    },);
    this.#resultsContainer
      .replaceChildren(...elements,);
    highlightMatches({
      query,
      container: this.#resultsContainer,
    },);
  }

  /**
   * Moves the keyboard selection by the given delta.
   */
  #moveSelection({ delta, }: { readonly delta: number; },): void {
    if (this.#resultsContainer
      === null)
      return;
    this.#selectedIndex = moveSearchSelection({
      delta,
      results: this.#results,
      selectedIndex: this.#selectedIndex,
      container: this.#resultsContainer,
    },);
  }
  /**
   * Confirms the current selection.
   */
  #confirmSelection(): void {
    this.#selectResult({ index: this.#selectedIndex, },);
  }
  /**
   * Dispatches a `result-select` event for the result at the given index and closes.
   */
  #selectResult({ index, }: { readonly index: number; },): void {
    /**
     * Event payload for `result-select`; null when `index` is out of range or `results` is empty.
     */
    const detail = buildResultDetail({
      index,
      results: this.#results,
    },);
    if (detail === null)
      return;
    this.dispatchEvent(new CustomEvent(
      'result-select',
      {
        detail,
        bubbles: true,
      },
    ),);
    this.#close();
  }
}

customElements.define(
  'search-overlay',
  SearchOverlay,
);
