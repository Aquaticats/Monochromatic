/**
 * `<rename-input>` web component.
 *
 * Displays a floating text input near the cursor for renaming symbols.
 * The user types the new name and presses Enter to confirm or Escape to cancel.
 * Dispatches `rename-confirm` with the new name, or `rename-cancel` on dismissal.
 */

import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';

import { STYLES, } from './rename-input.styles.ts';

/**
 * Vertical offset from the cursor to position the input below the symbol.
 */
const VERTICAL_OFFSET = 4;

/**
 * `<rename-input>`: floating input for symbol renaming.
 *
 * Shows a text input pre-filled with the current symbol name.
 * Enter confirms, Escape cancels. The component auto-focuses the input
 * and selects all text on show.
 */
export class RenameInput extends HTMLElement {
  /**
   * Shadow root for encapsulated rendering.
   */
  readonly #shadow: ShadowRoot;

  /**
   * Text input element.
   */
  #input: HTMLInputElement | null = null;

  /**
   * Initializes the shadow root.
   */
  constructor() {
    super();
    this.#shadow = this.attachShadow({ mode: 'open', },);
  }

  /**
   * Renders the input and wires keyboard handlers.
   */
  connectedCallback(): void {
    this.#input = h({
      tag: 'input',
      attrs: {
        type: 'text',
        spellcheck: 'false',
        autocomplete: 'off',
      },
    },) as HTMLInputElement;

    this.#shadow
      .replaceChildren(
      h({
        tag: 'style',
        text: STYLES,
      },),
      this.#input,
    );

    this.setAttribute(
      'popover',
      'manual',
    );

    /**
     * Local alias for the private input field used by both keydown branches below.
     */
    const input = this.#input;
    /**
     * Captured for the listener closures because event callbacks rebind `this`.
     */
    const component = this;

    input.addEventListener(
      'keydown',
      function handleInputKey(event,) {
        event.stopPropagation();
        if (event.key
          === 'Enter') {
          event.preventDefault();
          /**
           * Whitespace-only entries are dropped to avoid no-op renames.
           */
          const newName = input.value
            .trim();
          if (newName !== '') {
            component.dispatchEvent(new CustomEvent(
              'rename-confirm',
              {
                detail: { newName, },
                bubbles: true,
                composed: true,
              },
            ),);
          }
          component.hide();
          return;
        }
        if (event.key
          === 'Escape') {
          event.preventDefault();
          component.dispatchEvent(new CustomEvent(
            'rename-cancel',
            {
              bubbles: true,
              composed: true,
            },
          ),);
          component.hide();
        }
      },
    );

    /**
     * Prevent clicks inside the rename input from propagating to the editor.
     */
    input.addEventListener(
      'mousedown',
      function stopPropagation(event,) {
        event.stopPropagation();
      },
    );
  }

  /**
   * Shows the rename input at the given screen position.
   *
   * @param placeholder - current symbol name to pre-fill
   *
   * @param x - horizontal viewport coordinate (pixels)
   *
   * @param y - vertical viewport coordinate (pixels, below the symbol)
   */
  show({
    placeholder,
    x,
    y,
  }: {
    readonly placeholder: string;
    readonly x: number;
    readonly y: number;
  },): void {
    if (this.#input
      === null)
      return;

    this.#input
      .value = placeholder;
    this.style
      .setProperty(
      'inset-inline-start',
      `${x}px`,
    );
    this.style
      .setProperty(
      'inset-block-start',
      `${y + VERTICAL_OFFSET}px`,
    );

    if (!this.matches(':popover-open',))
      this.showPopover();

    this.#input
      .focus();
    this.#input
      .select();
  }

  /**
   * Hides the rename input.
   */
  hide(): void {
    if (this.matches(':popover-open',))
      this.hidePopover();
  }

  /**
   * Whether the rename input is currently visible.
   *
   * @returns true if visible
   */
  get visible(): boolean {
    return this.matches(':popover-open',);
  }
}

customElements.define(
  'rename-input',
  RenameInput,
);
