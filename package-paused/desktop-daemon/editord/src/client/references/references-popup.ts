/**
 * `<references-popup>` web component.
 *
 * Displays a navigable list of reference locations from the language server.
 * Supports keyboard navigation (arrow keys, Enter) and click selection.
 * Emits a `reference-select` event when a location is chosen.
 *
 * Uses the Popover API (`popover="auto"`) for top-layer rendering and
 * light dismiss. An invisible anchor div is placed at the editor cursor
 * and CSS anchor positioning auto-flips when overflowing the viewport.
 */

import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';

import {
  computeNextIndex,
  createReferenceAnchor,
  positionAnchor,
  renderReferenceItems,
  updateItemSelection,
} from './behavior.ts';
import { STYLES, } from './references-popup.styles.ts';
import type {
  ReferenceLocation,
  ReferenceSelectDetail,
} from './types.ts';

export type {
  ReferenceLocation,
  ReferenceSelectDetail,
};

/**
 * `<references-popup>`: language server references dropdown.
 *
 * Dispatches `reference-select` CustomEvent when a location is accepted.
 */
export class ReferencesPopup extends HTMLElement {
  /**
   * Shadow root for encapsulated rendering.
   */
  readonly #shadow: ShadowRoot;
  /**
   * Container for the list items.
   */
  #list: HTMLDivElement | null = null;
  /**
   * Currently displayed locations.
   */
  #locations: readonly ReferenceLocation[] = [];
  /**
   * Index of the selected item (-1 = none).
   */
  #selectedIndex = -1;
  /**
   * Invisible anchor div positioned at the editor cursor.
   */
  readonly #anchor: HTMLDivElement;

  /**
   * Initializes the shadow root and creates the anchor div.
   */
  constructor() {
    super();
    this.#shadow = this.attachShadow({ mode: 'open', },);
    this.#anchor = createReferenceAnchor();
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

    /**
     * Clean up anchor div when popover is light-dismissed.
     */
    const popup = this;
    this.addEventListener(
      'toggle',
      function handleToggle(event,) {
        if ((event as ToggleEvent).newState
          === 'closed')
          popup.#cleanup();
      },
    );
  }

  /**
   * Shows the popup with reference locations anchored near the editor cursor.
   *
   * @param locations - reference locations to display
   *
   * @param x - horizontal viewport coordinate of the editor cursor (pixels)
   *
   * @param y - top of the editor cursor (pixels)
   *
   * @param cursorHeight - height of the editor cursor (pixels)
   */
  show(
    {
      locations,
      x,
      y,
      cursorHeight,
    }: {
      readonly locations: readonly ReferenceLocation[];
      readonly x: number;
      readonly y: number;
      readonly cursorHeight: number;
    },
  ): void {
    if ((this.#list
      === null) || (locations.length
        === 0))
      return;
    this.#locations = locations;
    this.#selectedIndex = 0;
    this.parentElement
      ?.insertBefore(
      this.#anchor,
      this,
    );
    positionAnchor({
      anchor: this.#anchor,
      x,
      y,
      cursorHeight,
    },);
    this.showPopover();
    this.#list
      .replaceChildren(...renderReferenceItems({ locations, },),);
  }

  /**
   * Hides the popup and removes the anchor div.
   */
  hide(): void {
    if (this.matches(':popover-open',))
      this.hidePopover();
    this.#cleanup();
  }

  /**
   * Resets internal state and removes the anchor element.
   */
  #cleanup(): void {
    this.#anchor
      .remove();
    this.#locations = [];
    this.#selectedIndex = -1;
  }

  /**
   * Whether the popup is currently visible.
   *
   * @returns true when the popover is in the open state
   */
  get visible(): boolean {
    return this.matches(':popover-open',);
  }

  /**
   * Moves the selection up or down.
   */
  navigate({ direction, }: { readonly direction: 'up' | 'down'; },): void {
    if ((this.#locations
      .length
      === 0) || (this.#list
        === null))
      return;
    this.#selectedIndex = computeNextIndex({
      current: this.#selectedIndex,
      total: this.#locations
        .length,
      direction,
    },);
    updateItemSelection({
      list: this.#list,
      selectedIndex: this.#selectedIndex,
    },);
  }

  /**
   * Selects a reference location and dispatches a `reference-select` event.
   *
   * @param location - reference location to select
   *
   * @returns emitted navigation detail
   *
   * @example
   * ```ts
   * popup.selectReference({ path: '/tmp/a.ts', line: 0, character: 3, label: 'a.ts', });
   * ```
   */
  selectReference(location: ReferenceLocation,): ReferenceSelectDetail {
    /**
     * Public event detail; line is converted to 1-based to match editor convention.
     */
    const detail: ReferenceSelectDetail = {
      path: location.path,
      line: location.line
        + 1,
      character: location.character,
    };
    this.dispatchEvent(
      new CustomEvent(
        'reference-select',
        {
          detail,
          bubbles: true,
          composed: true,
        },
      ),
    );
    return detail;
  }

  /**
   * Accepts the selected location and dispatches a `reference-select` event.
   *
   * @returns selected location detail, or null if nothing selected
   */
  accept(): ReferenceSelectDetail | null {
    if ((this.#selectedIndex
      < 0) || (this.#selectedIndex
        >= this
        .#locations
        .length))
      return null;
    /**
     * Currently highlighted reference entry; bounds-checked above.
     */
    const loc = this.#locations[this.#selectedIndex];
    if (loc === undefined)
      return null;
    this.hide();
    return this.selectReference(loc,);
  }
}

customElements.define(
  'references-popup',
  ReferencesPopup,
);
