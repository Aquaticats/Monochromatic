/**
 * Context menu item type and renderer functions.
 *
 * Creates clickable and input-enabled menu item elements used by
 * the {@link ContextMenu} component. Extracted to keep the menu
 * class under the line limit.
 */

import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 * Single menu item definition.
 * When `defaultValue` is set, an inline text input appears next to the label.
 */
export type ContextMenuItem = {
  /**
   * Display label for the menu item.
   */
  readonly label: string;
  /**
   * Callback invoked when the item is activated (click or Enter in input).
   */
  readonly action: (value?: string,) => void;
  /**
   * When set, renders an inline input pre-filled with this value.
   */
  readonly defaultValue?: string;
};

/**
 * Prevents click events from bubbling to the menu item container.
 *
 * @param event - mouse event to stop
 */
function stopClickPropagation(event: MouseEvent,): void {
  event.stopPropagation();
}

/**
 * Creates a plain clickable menu item element.
 *
 * @param item - menu item definition
 *
 * @param onActivate - hides the menu and fires the action
 *
 * @returns menu item element
 *
 * @example
 * ```ts
 * const result = renderButtonItem({ item: { label: "Copy", type: "button" }, onActivate: handleActivate, });
 * ```
 */
export function renderButtonItem({
  item,
  onActivate,
}: {
  readonly item: ContextMenuItem;
  readonly onActivate: (action: () => void,) => void;
},): HTMLElement {
  return h({
    tag: 'div',
    class: 'ctx-item',
    attrs: { tabindex: '0', },
    text: item.label,
    on: {
      click: function handleClick(): void {
        onActivate(item.action,);
      },
      keydown: function handleKeydown(event: KeyboardEvent,): void {
        if ((event.key
          === 'Enter') || (event.key
            === ' ')) {
          event.preventDefault();
          onActivate(item.action,);
        }
      },
    },
  },);
}

/**
 * Creates a menu item with label + inline text input.
 *
 * @param item - menu item with `defaultValue` set
 *
 * @param onActivate - hides the menu and fires the action
 *
 * @returns menu item element containing label and input
 *
 * @example
 * ```ts
 * const result = renderInputItem({ item: { label: "Copy", type: "button" }, onActivate: handleActivate, });
 * ```
 */
export function renderInputItem({
  item,
  onActivate,
}: {
  readonly item: ContextMenuItem;
  readonly onActivate: (action: () => void,) => void;
},): HTMLElement {
  /**
   * Text input element captured into a const so the keydown handler can read its value.
   */
  const input = h({
    tag: 'input',
    class: 'ctx-input',
    attrs: {
      type: 'text',
      value: item.defaultValue
        ?? '',
    },
    on: {
      keydown: function handleInputKeydown(event: KeyboardEvent,): void {
        if (event.key
          === 'Enter') {
          event.preventDefault();
          /**
           * Trimmed input contents; Enter on empty value is a no-op.
           */
          const { value, } = input;
          if (value !== '') {
            onActivate(function fireAction(): void {
              item.action(value,);
            },);
          }
        }
      },
      click: stopClickPropagation,
    },
  },);

  return h({
    tag: 'div',
    class: 'ctx-input-row',
    children: [
      h({
        tag: 'span',
        class: 'ctx-label',
        text: item.label,
      },),
      input,
    ],
  },);
}
