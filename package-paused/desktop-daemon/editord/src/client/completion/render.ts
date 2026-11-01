/**
 * Rendering logic for the completion popup item list.
 *
 * Builds DOM elements for completion items and manages
 * the `data-selected` attribute for keyboard navigation.
 */

import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';

import type { CompletionItem, } from '../../../protocol.ts';

/**
 * Creates DOM elements for a list of completion items.
 * The first item is marked as selected by default.
 *
 * @param items - completion items to render
 *
 * @returns array of item div elements
 *
 * @example
 * ```ts
 * const result = renderItems({ items: [{ label: "useState", detail: "function" }], });
 * ```
 */
export function renderItems({ items, }: { readonly items: readonly CompletionItem[]; },): HTMLDivElement[] {
  return items.map(function createItemElement(
    item,
    index,
  ) {
    /**
     * Per-item row populated below; first item gets `data-selected`.
     */
    const el = h({
      tag: 'div',
      class: 'item',
    },);
    el.textContent = item.label;
    if (item.detail
      !== '') {
      el.append(h({
        tag: 'span',
        class: 'detail',
        text: item.detail,
      },),);
    }
    if (index === 0)
      el.dataset
        .selected = '';
    return el;
  },);
}

/**
 * Updates the `data-selected` attribute on list children.
 *
 * @param list - container element with item children
 *
 * @param selectedIndex - index of the item to select
 *
 * @example
 * ```ts
 * updateSelection({ list: [], selectedIndex: 0, });
 * ```
 */
export function updateSelection(
  {
    list,
    selectedIndex,
  }: {
    readonly list: HTMLElement;
    readonly selectedIndex: number;
  },
): void {
  /**
   * Live HTMLCollection of rendered item rows.
   */
  const { children, } = list;
  for (let i = 0; i < children
    .length; i++) {
    /**
     * Per-row element; the dataset flag flips based on whether i matches the selection.
     */
    const child = children[i];
    if (child instanceof HTMLElement) {
      if (i === selectedIndex) {
        child.dataset
          .selected = '';
        child.scrollIntoView({ block: 'nearest', },);
      }
      else {
        delete child.dataset
          .selected;
      }
    }
  }
}
