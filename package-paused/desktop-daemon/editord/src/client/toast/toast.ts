/**
 * Ephemeral toast notifications.
 *
 * Creates temporary fixed-position elements that auto-dismiss
 * after a short duration. Cursor-anchored toasts display near the editor
 * caret; fixed toasts center at the top of the viewport for warnings
 * not tied to a specific location.
 *
 * Visual styles are loaded at build time via `global.css`;
 * see {@link toast.styles.ts} for the h-css definitions.
 */

import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 * Duration in milliseconds before the toast is removed.
 */
const DISMISS_MS = 2_000;

/**
 * Vertical gap below the cursor in pixels.
 */
const VERTICAL_OFFSET = 4;

/**
 * Shows a brief toast message centered at the top of the viewport.
 * Used for warnings not tied to a specific cursor location.
 * The toast auto-dismisses after {@link DISMISS_MS}.
 *
 * @param message - text to display
 *
 * @example
 * ```ts
 * showFixedToast({ message: 'File too large to open' });
 * ```
 */
export function showFixedToast({ message, }: { readonly message: string; },): void {
  /**
   * Toast element styled by the `[data-variant="fixed"]` selector.
   */
  const toast = h({
    tag: 'div',
    class: 'toast',
    text: message,
    attrs: { 'data-variant': 'fixed', },
  },);
  document.body
    .append(toast,);

  globalThis.setTimeout(
    function dismissToast() {
      toast.remove();
    },
    DISMISS_MS,
  );
}

/**
 * Shows a brief toast message near the editor cursor.
 * The toast auto-dismisses after {@link DISMISS_MS}.
 *
 * @param message - text to display
 *
 * @param rect - bounding rectangle of the editor cursor for positioning
 *
 * @example
 * ```ts
 * const rect = editorPane.getCursorRect();
 * if (rect !== null) showCursorToast({ message: 'No definition found', rect });
 * ```
 */
export function showCursorToast(
  {
    message,
    rect,
  }: {
    readonly message: string;
    readonly rect: DOMRect;
  },
): void {
  /**
   * Toast element styled by the `[data-variant="cursor"]` selector.
   */
  const toast = h({
    tag: 'div',
    class: 'toast',
    text: message,
    attrs: { 'data-variant': 'cursor', },
  },);

  /**
   * Dynamic position properties that vary per toast instance.
   */
  toast.style
    .setProperty(
    'inset-inline-start',
    `${rect.left}px`,
  );
  toast.style
    .setProperty(
    'inset-block-start',
    `${rect.bottom
      + VERTICAL_OFFSET}px`,
  );

  document.body
    .append(toast,);

  globalThis.setTimeout(
    function dismissToast() {
      toast.remove();
    },
    DISMISS_MS,
  );
}
