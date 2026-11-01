/**
 * `<completion-popup>` web component.
 *
 * Displays a filterable list of completion items from the language server.
 * Supports keyboard navigation, selection, and dismissal.
 */

import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';

import type { CompletionItem, } from '../../../protocol.ts';
import { STYLES, } from './completion-popup.styles.ts';
import {
  renderItems,
  updateSelection,
} from './render.ts';

/**
 * Vertical offset from the cursor in pixels.
 */
const VERTICAL_OFFSET = 4;

/**
 * Editor cursor position when the popup was shown.
 */
type ShownAt = {
  readonly line: number;
  readonly character: number;
};

/**
 * `<completion-popup>`: language server autocompletion dropdown.
 *
 * Dispatches `completion-select` CustomEvent when an item is accepted.
 */
export class CompletionPopup extends HTMLElement {
  /**
   * Shadow root for encapsulated rendering.
   */
  readonly #shadow: ShadowRoot;
  /**
   * Container for the list items.
   */
  #list: HTMLDivElement | null = null;
  /**
   * Currently displayed items.
   */
  #items: readonly CompletionItem[] = [];
  /**
   * Index of the selected item (-1 = none).
   */
  #selectedIndex = -1;
  /**
   * Cursor position captured at the latest `show()`; null while hidden.
   */
  #shownAt: ShownAt | null = null;

  /**
   * Initializes the shadow root.
   */
  constructor() {
    super();
    this.#shadow = this.attachShadow({ mode: 'open', },);
  }

  /**
   * Renders the container and sets up popover behavior.
   */
  connectedCallback(): void {
    this.#list = h({
      tag: 'div',
      class: 'list',
    },);
    this.#shadow
      .replaceChildren(
      h({
        tag: 'style',
        text: STYLES,
      },),
      this.#list,
    );
    this.setAttribute(
      'popover',
      'auto',
    );
  }

  /**
   * Shows the popup with items at the given position.
   */
  show({
    items,
    x,
    y,
    cursor,
  }: {
    readonly items: readonly CompletionItem[];
    readonly x: number;
    readonly y: number;
    readonly cursor: ShownAt;
  },): void {
    if ((this.#list
      === null) || (items.length
        === 0))
      return;
    this.#items = items;
    this.#selectedIndex = 0;
    this.#shownAt = cursor;
    this.#list
      .replaceChildren(...renderItems({ items, },),);
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
  }

  /**
   * Hides the popup and clears items.
   */
  hide(): void {
    if (this.matches(':popover-open',))
      this.hidePopover();
    this.#items = [];
    this.#selectedIndex = -1;
    this.#shownAt = null;
  }

  /**
   * Cursor position captured the last time `show()` was called.
   *
   * @returns position when visible, null when hidden
   */
  get shownAt(): ShownAt | null {
    return this.#shownAt;
  }

  /**
   * Whether the popup is currently visible.
   *
   * @returns true if visible
   */
  get visible(): boolean {
    return this.matches(':popover-open',);
  }

  /**
   * Moves the selection up or down.
   */
  navigate({ direction, }: { readonly direction: 'up' | 'down'; },): void {
    if (this.#items
      .length
      === 0)
      return;
    if (direction === 'up') {
      this.#selectedIndex = this.#selectedIndex
        <= 0
        ? this.#items
          .length
          - 1
        : this.#selectedIndex
          - 1;
    }
    else {
      this.#selectedIndex = this.#selectedIndex
        >= (this.#items
          .length
          - 1)
        ? 0
        : this.#selectedIndex
          + 1;
    }
    if (this.#list
      !== null) {
      updateSelection({
        list: this.#list,
        selectedIndex: this.#selectedIndex,
      },);
    }
  }

  /**
   * Accepts the selected item.
   *
   * @returns insert text, or null if nothing selected
   */
  accept(): string | null {
    if ((this.#selectedIndex
      < 0) || (this.#selectedIndex
        >= this
        .#items
        .length))
      return null;
    /**
     * Currently highlighted completion entry; bounds-checked above.
     */
    const item = this.#items[this.#selectedIndex];
    if (item === undefined)
      return null;
    /**
     * Text the caller will splice into the document at the cursor.
     */
    const { insertText, } = item;
    this.hide();
    this.dispatchEvent(
      new CustomEvent(
        'completion-select',
        {
          detail: { text: insertText, },
          bubbles: true,
          composed: true,
        },
      ),
    );
    return insertText;
  }
}

customElements.define(
  'completion-popup',
  CompletionPopup,
);
