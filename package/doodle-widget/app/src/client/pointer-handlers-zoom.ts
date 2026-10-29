/**
 Zoom tool pointer event handlers for the doodle widget.
 
 Handles click-to-zoom, drag-to-pan, shift-to-zoom-out, and
 long-press-to-zoom-out gestures when the zoom tool is active.
 */

import type { PointerHandlerDeps, } from './pointer-handler-deps.ts';
import {
  continuePan,
  endPan,
  startPan,
} from './zoom-pan.ts';
import {
  getPanX,
  getPanY,
  zoomAt,
} from './zoom.ts';

/**
 Hold duration in milliseconds to trigger long-press zoom-out
 */
const LONG_PRESS_MS = 500;

/**
 Long-press countdown timer slot.
 
 `idle` when no countdown is armed; `pending` while the timeout runs, carrying
 its handle so a drag or release can cancel it.
 */
type LongPressTimer =
  | { readonly kind: 'idle'; }
    | {
      readonly kind: 'pending';
      readonly id: ReturnType<typeof setTimeout>;
    };

/**
 Captured pointer-down for the active gesture.
 
 `none` between gestures; `down` from press to release, carrying the event so
 the deferred long-press handler can read its screen coordinates.
 */
type DownEvent =
  | { readonly kind: 'none'; }
    | {
      readonly kind: 'down';
      readonly event: PointerEvent;
    };

/**
 Attaches zoom-specific pointer and keyboard handlers to the canvas.
 
 @param deps - shared state and element references, see {@link PointerHandlerDeps}
 
 @mutates deps - `canvas.addEventListener` changes the event target and retains handlers; `canvas.setPointerCapture` changes pointer capture state.
 
 @example
 ```ts
 setupZoomPointerHandlers(deps);
 ```
 */
export function setupZoomPointerHandlers(deps: PointerHandlerDeps,): void {
  /**
   Destructured up front so each handler closure captures the same handles.
   */
  const {
    canvas,
    getToolMode,
    getCanvasSize,
    page,
    zoomLayer,
  } = deps;

  /**
   Per-gesture mutable state container.
   
   Stored as object properties so function-root state stays in a `const`
   container (`no-function-root-let` would otherwise reject top-level `let`).
   */
  const gestureState: {
    longPressTimer: LongPressTimer;
    longPressFired: boolean;
    downEvent: DownEvent;
  } = {
    longPressTimer: { kind: 'idle', },
    longPressFired: false,
    downEvent: { kind: 'none', },
  };

  /**
   Sets the canvas cursor based on current zoom tool state.
   
   @param style - CSS cursor keyword to apply
   */
  function setZoomCursor(style: string,): void {
    canvas.style
      .cursor = style;
  }

  /**
   Cancels any pending long-press timer
   */
  function clearLongPress(): void {
    /**
     Captured so the discriminant check and clear act on one timer.
     */
    const timer = gestureState.longPressTimer;
    if (timer.kind === 'pending') {
      clearTimeout(timer.id,);
      gestureState.longPressTimer = { kind: 'idle', };
    }
  }

  canvas.addEventListener(
    'pointerdown',
    function handleZoomPointerDown(event: PointerEvent,): void {
      if (getToolMode()
        !== 'zoom')
        return;
      /**
       Prevent iOS Safari from cancelling the pointer sequence via native gesture recognition
       */
      event.preventDefault();
      canvas.setPointerCapture(event.pointerId,);
      startPan({
        event,
        currentPanX: getPanX(),
        currentPanY: getPanY(),
      },);
      gestureState.downEvent = {
        kind: 'down',
        event,
      };
      gestureState.longPressFired = false;
      clearLongPress();
      /**
       Timer handle wrapped into the pending state after scheduling.
       */
      const longPressId = setTimeout(
        function fireLongPress(): void {
          gestureState.longPressFired = true;
          gestureState.longPressTimer = { kind: 'idle', };
          /**
           Captured so the discriminant check and coordinate reads use one event.
           */
          const down = gestureState.downEvent;
          if (down.kind === 'none')
            return;
          /**
           Container layout captured per gesture so panning offsets stay consistent across reflows.
           */
          const containerRect = page.getBoundingClientRect();
          /**
           Canvas dimensions resolved here so the zoom math uses fresh sizing.
           */
          const {
            cw,
            ch,
          } = getCanvasSize();
          zoomAt({
            screenX: down.event
              .clientX
              - containerRect
                .left,
            screenY: down.event
              .clientY
              - containerRect
                .top,
            direction: 'out',
            containerWidth: cw,
            containerHeight: ch,
            zoomLayer,
          },);
        },
        LONG_PRESS_MS,
      );
      gestureState.longPressTimer = {
        kind: 'pending',
        id: longPressId,
      };
    },
  );

  canvas.addEventListener(
    'pointermove',
    function handleZoomPointerMove(event: PointerEvent,): void {
      if (getToolMode()
        !== 'zoom')
        return;
      /**
       Canvas dimensions resolved each move so pan math stays in sync with the live layout.
       */
      const {
        cw,
        ch,
      } = getCanvasSize();
      /**
       True only when the pointer has crossed the drag threshold, so the cursor flip is conditional.
       */
      const dragging = continuePan({
        event,
        containerWidth: cw,
        containerHeight: ch,
        zoomLayer,
      },);
      if (dragging) {
        clearLongPress();
        setZoomCursor('move',);
      }
    },
  );

  canvas.addEventListener(
    'pointerup',
    function handleZoomPointerUp(event: PointerEvent,): void {
      if (getToolMode()
        !== 'zoom')
        return;
      clearLongPress();
      /**
       True when the gesture moved past the drag threshold; suppresses the tap-to-zoom action.
       */
      const wasDrag = endPan();
      if ((!wasDrag) && (!gestureState.longPressFired)) {
        /**
         Container layout captured at release so the zoom origin matches the screen tap.
         */
        const containerRect = page.getBoundingClientRect();
        /**
         Canvas dimensions resolved at release so the zoom math uses fresh sizing.
         */
        const {
          cw,
          ch,
        } = getCanvasSize();
        zoomAt({
          screenX: event.clientX
            - containerRect
              .left,
          screenY: event.clientY
            - containerRect
              .top,
          direction: event.shiftKey ? 'out' : 'in',
          containerWidth: cw,
          containerHeight: ch,
          zoomLayer,
        },);
      }
      gestureState.downEvent = { kind: 'none', };
      setZoomCursor(event.shiftKey ? 'zoom-out' : 'zoom-in',);
    },
  );

  canvas.addEventListener(
    'pointercancel',
    function handleZoomPointerCancel(): void {
      clearLongPress();
      endPan();
      gestureState.downEvent = { kind: 'none', };
    },
  );

  //region iOS touch fallback
  /**
   iOS Safari may not fully honor `touch-action: none` in CSS and can
   fire `pointercancel` to take over touch handling for native gestures
   (scroll, page zoom, context menu). Explicitly preventing default on
   `touchstart` and `touchmove` stops this at the touch-event level,
   before pointer events are generated.
   */

  canvas.addEventListener(
    'touchstart',
    function handleZoomTouchStart(event: TouchEvent,): void {
      if (getToolMode()
        === 'zoom')
        event.preventDefault();
    },
    { passive: false, },
  );

  canvas.addEventListener(
    'touchmove',
    function handleZoomTouchMove(event: TouchEvent,): void {
      if (getToolMode()
        === 'zoom')
        event.preventDefault();
    },
    { passive: false, },
  );

  //endregion iOS touch fallback

  //region Shift key cursor toggle

  document.addEventListener(
    'keydown',
    function handleZoomKeyDown(event: KeyboardEvent,): void {
      if ((getToolMode()
        === 'zoom') && (event.key
          === 'Shift'))
        setZoomCursor('zoom-out',);
    },
  );

  document.addEventListener(
    'keyup',
    function handleZoomKeyUp(event: KeyboardEvent,): void {
      if ((getToolMode()
        === 'zoom') && (event.key
          === 'Shift'))
        setZoomCursor('zoom-in',);
    },
  );

  //endregion Shift key cursor toggle
}
