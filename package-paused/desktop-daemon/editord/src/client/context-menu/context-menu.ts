/**
 * Context menu for the file tree.
 *
 * Uses the Popover API (`popover="auto"`) for top-layer rendering and
 * light dismiss. An invisible anchor div is positioned at the click point
 * and CSS anchor positioning places the menu beside it.
 *
 * Items can include an inline text input for actions that need user text
 * (rename, copy, move, new). The user types directly in the menu row
 * and presses Enter to confirm.
 */

import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';

import {
  type ContextMenuItem,
  renderButtonItem,
  renderInputItem,
} from './items.ts';

export type { ContextMenuItem, } from './items.ts';

/**
 * Mutable context menu state captured by the factory closure.
 */
type ContextMenuState = {
  /**
   * Popover element, or null when hidden.
   */
  popup: HTMLDivElement | null;
};

/**
 * Context menu handle returned by {@link createContextMenu}.
 */
export type ContextMenu = Readonly<{
  /**
   * Shows menu items at viewport coordinates.
   */
  readonly show: (opts: {
    readonly x: number;
    readonly y: number;
    readonly items: readonly ContextMenuItem[];
  },) => void;
  /**
   * Hides the context menu and cleans up the anchor.
   */
  readonly hide: () => void;
}>;

/**
 * Creates a context menu instance.
 *
 * The popup lives in the top layer via `popover="auto"`, escaping
 * any overflow clipping. Light dismiss (click outside, Escape)
 * is handled natively by the browser.
 *
 * @returns frozen context menu handle
 *
 * @example
 * ```ts
 * const menu = createContextMenu();
 * menu.show({ x: 10, y: 20, items: [], });
 * ```
 */
export function createContextMenu(): ContextMenu {
  /**
   * Invisible anchor div positioned at the click point.
   */
  const anchor = h({
    tag: 'div',
    class: 'ctx-anchor',
  },);
  /**
   * Mutable popup slot kept private to this handle.
   */
  const state: ContextMenuState = {
    popup: null,
  };
  /**
   * Removes the popover element when dismissed by the browser.
   *
   * @param event - popover toggle event
   */
  function handlePopoverToggle(event: Event,): void {
    // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- toggle event on popover elements always carries newState
    if ((event as ToggleEvent).newState
      === 'closed') {
      /* oxlint-disable typescript-eslint/no-unsafe-type-assertion -- currentTarget is always the popover div */
      /**
       * Popover element fired the toggle; remove it so the next show creates a fresh one.
       */
      const popup = event.currentTarget as HTMLDivElement;
      /* oxlint-enable typescript-eslint/no-unsafe-type-assertion */
      popup.remove();
      if (state.popup
        === popup)
        state.popup = null;
    }
  }

  /**
   * Callback fired when the popover is dismissed by the browser.
   */
  const onToggleBound = handlePopoverToggle;

  /**
   * Hides the context menu and cleans up the anchor.
   */
  function hide(): void {
    if (state.popup
      !== null) {
      /**
       * Popup captured before clearing state so event callbacks cannot observe a stale handle.
       */
      const { popup, } = state;
      state.popup = null;
      popup.hidePopover();
      popup.remove();
    }
    anchor.remove();
  }

  /**
   * Shows the context menu at the given coordinates.
   *
   * @param x - horizontal click position in viewport pixels
   *
   * @param y - vertical click position in viewport pixels
   *
   * @param items - menu items to display
   */
  function show({
    x,
    y,
    items,
  }: {
    readonly x: number;
    readonly y: number;
    readonly items: readonly ContextMenuItem[];
  },): void {
    hide();

    /**
     * Hides the menu and invokes the given action callback.
     *
     * @param action - callback to invoke after hiding
     */
    function onActivate(action: () => void,): void {
      hide();
      action();
    }

    /**
     * Rendered DOM rows for every item; each row is either an input or a button row.
     */
    const menuItems = items.map(function renderItem(item,) {
      return item.defaultValue
        !== undefined
        ? renderInputItem({
          item,
          onActivate,
        },)
        : renderButtonItem({
          item,
          onActivate,
        },);
    },);

    /**
     * Position the invisible anchor at the click point.
     */
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
    document.body
      .append(anchor,);

    state.popup = h({
      tag: 'div',
      class: 'ctx-popup',
      attrs: { popover: 'auto', },
      children: menuItems,
    },);

    state.popup
      .addEventListener(
      'toggle',
      onToggleBound,
    );
    document.body
      .append(state.popup,);
    state.popup
      .showPopover();

    /**
     * First rendered row; focused on open so keyboard users land on a real item, not the dialog.
     */
    const [firstItem,] = menuItems;
    if (firstItem !== undefined)
      firstItem.focus();
  }

  return Object.freeze({
    show,
    hide,
  },);
}
