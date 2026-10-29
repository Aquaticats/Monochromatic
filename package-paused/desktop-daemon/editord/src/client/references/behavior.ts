/**
 * Behavior helpers for the references popup component.
 *
 * Provides anchor management, item rendering, and selection
 * update functions used by the ReferencesPopup class.
 */

import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';

import type { ReferenceLocation, } from './types.ts';

/**
 * Creates the invisible anchor div used for CSS anchor positioning.
 *
 * @returns fixed-position anchor div
 *
 * @example
 * ```ts
 * const result = createReferenceAnchor();
 * ```
 */
export function createReferenceAnchor(): HTMLDivElement {
  /**
   * Invisible anchor element used as the `anchor-name` target for the popover.
   */
  const anchor = document.createElement('div',);
  anchor.style
    .setProperty(
    'position',
    'fixed',
  );
  anchor.style
    .setProperty(
    'anchor-name',
    '--ref-anchor',
  );
  anchor.style
    .setProperty(
    'inline-size',
    '0.125rem',
  );
  anchor.style
    .setProperty(
    'pointer-events',
    'none',
  );
  anchor.style
    .setProperty(
    'z-index',
    '9999',
  );
  return anchor;
}

/**
 * Positions the anchor div at the editor cursor location.
 *
 * @param anchor - anchor div to position
 *
 * @param x - horizontal viewport coordinate (pixels)
 *
 * @param y - top of the editor cursor (pixels)
 *
 * @param cursorHeight - height of the editor cursor (pixels)
 *
 * @example
 * ```ts
 * positionAnchor({ anchor: anchorElement, x: 120, y: 240, cursorHeight: 20, });
 * ```
 */
export function positionAnchor({
  anchor,
  x,
  y,
  cursorHeight,
}: {
  readonly anchor: HTMLDivElement;
  readonly x: number;
  readonly y: number;
  readonly cursorHeight: number;
},): void {
  anchor.style
    .setProperty(
    'inset-inline-start',
    `${x}px`,
  );
  anchor.style
    .setProperty(
    'inset-block-start',
    `${y}px`,
  );
  anchor.style
    .setProperty(
    'block-size',
    `${cursorHeight}px`,
  );
}

/**
 * Renders reference location items as DOM elements.
 *
 * @param locations - reference locations to render
 *
 * @returns array of item div elements
 *
 * @example
 * ```ts
 * const result = renderReferenceItems({ locations: [{ path: "/src/app.ts", line: 10, character: 5 }], });
 * ```
 */
export function renderReferenceItems(
  { locations, }: { readonly locations: readonly ReferenceLocation[]; },
): HTMLElement[] {
  return locations.map(function renderItem(
    loc,
    index,
  ) {
    /**
     * Row element for a single reference; first row is auto-selected via dataset below.
     */
    const item = h({
      tag: 'div',
      class: 'item',
    },);
    item.append(
      h({
        tag: 'span',
        class: 'item-path',
        text: loc.label,
      },),
      h({
        tag: 'span',
        class: 'line-num',
        text: `:${String(loc.line
          + 1,)}`,
      },),
    );
    /**
     * Without dataset: prefer-dom-node-dataset lint error for setAttribute on data- attributes.
     */
    if (index === 0)
      item.dataset
        .selected = '';
    return item;
  },);
}

/**
 * Updates the selected item visual state in the list.
 *
 * @param list - list container div
 *
 * @param selectedIndex - index of the newly selected item
 *
 * @example
 * ```ts
 * updateItemSelection({ list: [], selectedIndex: 0, });
 * ```
 */
export function updateItemSelection({
  list,
  selectedIndex,
}: {
  readonly list: HTMLDivElement;
  readonly selectedIndex: number;
},): void {
  /**
   * Without querySelectorAll: unsafe type assertion from Element to HTMLElement on children.
   */
  const items = list.querySelectorAll<HTMLElement>('.item',);
  for (const [i, item,] of [...items,].entries()) {
    if (i === selectedIndex) {
      item.dataset
        .selected = '';
      item.scrollIntoView({ block: 'nearest', },);
    }
    else {
      delete item.dataset
        .selected;
    }
  }
}

/**
 * Computes the next selected index after a navigation step.
 *
 * @param current - current selected index
 *
 * @param total - total number of items
 *
 * @param direction - navigation direction
 *
 * @returns new selected index (wraps around)
 *
 * @example
 * ```ts
 * const result = computeNextIndex({ current: 0, total: 0, direction: 'next', });
 * ```
 */
export function computeNextIndex({
  current,
  total,
  direction,
}: {
  readonly current: number;
  readonly total: number;
  readonly direction: 'up' | 'down';
},): number {
  if (direction === 'up')
    return current <= 0 ? total - 1 : current - 1;
  return current >= (total - 1) ? 0 : current + 1;
}
