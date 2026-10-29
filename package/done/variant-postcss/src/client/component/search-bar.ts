import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import { css, } from '../css.ts';

/**
 Shadow DOM styles for the `\<search-bar\>` component.
 */
const STYLES = css(`
  :host {
    @apply --sticky-bar;
  }
  .back {
    @apply --appearance-none;
    @apply --flex-center;
    @apply --min-touch-target;
    font-size: 1.5rem;
    color: var(--fg);
  }
  .back:focus-visible {
    outline-width: 0.125rem;
    outline-style: solid;
    outline-color: var(--fg);
    outline-offset: -0.125rem;
  }
  input {
    flex: 1;
    border-style: none;
    background-color: transparent;
    font-size: 1rem;
    font-family: inherit;
    color: var(--fg);
    outline: none;
    block-size: 100%;
  }
  @apply --shadow-dom-globals;
  @media (min-width: 48rem) {
    :host {
      border-block-end-width: calc(1 / 16 * 1rem);
      border-block-end-style: solid;
      border-block-end-color: var(--bg-weaker);
    }
    .back { display: none; }
    input { font-size: 1.5rem; }
  }
`,);

/**
 Debounce delay for search input in milliseconds.
 */
const SEARCH_DEBOUNCE_MS = 300;

/**
 Navigates back one entry in browser history.
 */
function onBackClick(): void {
  history.back();
}

/**
 `\<search-bar\>`: sticky bar with a back button and debounced search input.
 Dispatches a `search` event with `\{ query \}` after the debounce delay.
 */
class SearchBar extends HTMLElement {
  /**
   Shadow root for encapsulated rendering.
   */
  readonly #shadow: ShadowRoot;

  /**
   Initializes the shadow root.
   */
  constructor() {
    super();
    this.#shadow = this.attachShadow({ mode: 'open', },);
  }

  /**
   Current search input value.
   
   @returns Text content of the input, or empty string if not rendered
   */
  get value(): string {
    /**
     Input element from the rendered shadow tree; `null` before `connectedCallback`.
     */
    const input = this.#shadow
      .querySelector<HTMLInputElement>('input',);
    return input?.value
      ?? '';
  }

  /**
   Sets the search input value programmatically.
   
   @param text - New input value
   */
  set value(text: string,) {
    /**
     Input element from the rendered shadow tree; assignment is skipped when not yet rendered.
     */
    const input = this.#shadow
      .querySelector<HTMLInputElement>('input',);
    if (input !== null)
      input.value = text;
  }

  /**
   Renders the search bar with back button and debounced input.
   */
  connectedCallback(): void {
    /**
     Pre-filled query from the `value` attribute, defaulting to empty when absent.
     */
    const query = this.getAttribute('value',)
      ?? '';

    // SVG back arrow built via innerHTML on a container because h() targets
    // HTMLElement creation; SVG elements require the SVG namespace.
    /**
     Back button captured so innerHTML can be set with the SVG payload below.
     */
    const backButton = h({
      tag: 'button',
      class: 'back',
      attrs: { 'aria-label': 'Go back', },
      on: { click: onBackClick, },
    },);
    backButton.innerHTML =
      `<svg width="32" height="32" viewBox="0 0 32 32" fill="none" stroke="currentColor" stroke-width="3"><polyline points="20,6 10,16 20,26"/></svg>`;

    /**
     Search input captured so the debounce closure can read its current value.
     */
    const input = h({
      tag: 'input',
      attrs: {
        type: 'search',
        placeholder: 'Search titles, tags, ...',
        value: query,
        autofocus: '',
      },
    },);

    // Debounced search dispatch
    /**
     Captured so the debounce closures reach this component without `this`-bound functions.
     */
    const self = this;
    /* oxlint-disable no-restricted-syntax/no-function-root-let -- debounce timer handle mutated on each keystroke; closed over by the input listener */
    /**
     Mutable timer handle replaced on each keystroke to debounce dispatch.
     */
    let timeout: ReturnType<typeof setTimeout> = setTimeout(
      function noop() {/* initial */},
      0,
    );
    /* oxlint-enable no-restricted-syntax/no-function-root-let */
    input.addEventListener(
      'input',
      function onInput(): void {
        clearTimeout(timeout,);
        timeout = setTimeout(
          function dispatchSearch(): void {
            self.dispatchEvent(
              new CustomEvent(
                'search',
                {
                  detail: { query: input.value
                    .trim(), },
                  bubbles: true,
                },
              ),
            );
          },
          SEARCH_DEBOUNCE_MS,
        );
      },
    );

    this.#shadow
      .replaceChildren(
      h({
        tag: 'style',
        text: STYLES,
      },),
      backButton,
      input,
      );
  }
}

customElements.define(
  'search-bar',
  SearchBar,
);
