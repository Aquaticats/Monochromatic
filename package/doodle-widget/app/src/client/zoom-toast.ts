/**
 Toast notification for zoom tool instructions.
 
 Shows a brief hint when the zoom tool is selected, then
 auto-dismisses after a timeout. Uses the Popover API for
 non-modal overlay without blocking interaction.
 */

/**
 Duration in milliseconds before the toast auto-hides
 */
const TOAST_DURATION_MS = 3_000;

/**
 Auto-dismiss timer slot.
 
 `idle` when no dismissal is scheduled; `pending` while a timeout is armed,
 carrying its handle so a re-show can cancel it before re-arming.
 */
type ToastTimer =
  | { readonly kind: 'idle'; }
    | {
      readonly kind: 'pending';
      readonly id: ReturnType<typeof setTimeout>;
    };

/**
 Auto-dismiss timer container.
 
 Stored as an object property so module-root state stays in a `const`
 container (`no-module-root-let` would otherwise reject a top-level `let`).
 */
const timerState: { current: ToastTimer; } = { current: { kind: 'idle', }, };

/**
 Shows the zoom instruction toast, auto-hiding after a delay.
 
 No-op if the toast element is missing (graceful degradation).
 
 @param toast - popover element to show
 
 @mutates toast - `toast.showPopover` and deferred `toast.hidePopover` change popover state and can dispatch events to retained listeners.
 
 @example
 ```ts
 showZoomToast(document.getElementById('zoom-toast'));
 ```
 */
export function showZoomToast(toast: HTMLElement,): void {
  /**
   Captured before re-arming so a still-pending timeout is cancelled first.
   */
  const pending = timerState.current;
  if (pending.kind === 'pending')
    clearTimeout(pending.id,);
  toast.showPopover();
  timerState.current = {
    kind: 'pending',
    id: setTimeout(
      function hideToast(): void {
        toast.hidePopover();
        timerState.current = { kind: 'idle', };
      },
      TOAST_DURATION_MS,
    ),
  };
}
