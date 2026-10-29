/**
 * `<hover-popup>` web component.
 *
 * Displays hover information (type signatures, documentation)
 * from the language server in a floating tooltip near the cursor.
 * Positioned absolutely relative to the viewport using fixed positioning.
 */

import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';

import { STYLES, } from './hover-popup.styles.ts';

/**
 * Vertical offset from the cursor in pixels to avoid covering the text.
 */
const VERTICAL_OFFSET = 8;

/**
 * `<hover-popup>`: floating tooltip for language server hover info.
 *
 * Shows type information, documentation, or diagnostic messages
 * at a specified screen position. Controlled via `show()` and `hide()`.
 */
export class HoverPopup extends HTMLElement {
  /**
   * Shadow root for encapsulated rendering.
   */
  readonly #shadow: ShadowRoot;

  /**
   * Content container for the hover text.
   */
  #content: HTMLDivElement | null = null;

  /**
   * Initializes the shadow root.
   */
  constructor() {
    super();
    this.#shadow = this.attachShadow({ mode: 'open', },);
  }

  /**
   * Renders the hover container and sets up popover behavior.
   */
  connectedCallback(): void {
    this.#content = h({
      tag: 'div',
      class: 'content',
    },);
    this.#shadow
      .replaceChildren(
      h({
        tag: 'style',
        text: STYLES,
      },),
      this.#content,
    );
    this.setAttribute(
      'popover',
      'hint',
    );
  }

  /**
   * Shows the hover popup at the given screen position with the provided content.
   *
   * @param text - hover content to display
   *
   * @param x - horizontal viewport coordinate (pixels)
   *
   * @param y - vertical viewport coordinate (pixels, below the hovered text)
   */
  show({
    text,
    x,
    y,
  }: {
    readonly text: string;
    readonly x: number;
    readonly y: number;
  },): void {
    if (this.#content
      === null)
      return;

    this.#content
      .textContent = text;
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
   * Hides the hover popup.
   */
  hide(): void {
    if (this.matches(':popover-open',))
      this.hidePopover();
  }

  /**
   * Whether the popup is currently visible.
   *
   * @returns true if visible
   */
  get visible(): boolean {
    return this.matches(':popover-open',);
  }
}

customElements.define(
  'hover-popup',
  HoverPopup,
);
