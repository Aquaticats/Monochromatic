/**
 Pointer event handlers for the doodle widget canvas.
 
 Translates pointer events into drawing strokes (draw mode),
 text input placement (text mode), or erasure (erase mode)
 based on the active tool. Zoom is handled separately in
 {@link import('./pointer-handlers-zoom.ts')}.
 */
import {
  getStrokeColor,
  getStrokeWidth,
} from './drawing-config.ts';
import {
  continueStroke,
  endStroke,
  NO_SEGMENT,
  type NormalizedPoint,
  normalizePointer,
  redraw,
  startStroke,
} from './drawing.ts';
import { eraseStrokesAt, } from './eraser-strokes.ts';
import { eraseTextAt, } from './eraser-text.ts';
import type { PointerHandlerDeps, } from './pointer-handler-deps.ts';
import { placeTextInput, } from './text.ts';

export type { ToolMode, } from './pointer-handler-deps.ts';

/**
 Previous eraser sample within an erase gesture.
 
 `none` on the first event (no prior point to sweep from); `at` once a sample
 exists, carrying it so the next tick can test the eraser travel segment.
 */
type PrevErasePoint =
  | { readonly kind: 'none'; }
    | {
      readonly kind: 'at';
      readonly point: NormalizedPoint;
    };

/**
 Attaches pointerdown, pointermove, pointerup, and pointercancel
 handlers to the canvas for draw, erase, and text tools.
 
 @param deps - shared state and element references, see {@link PointerHandlerDeps}
 
 @mutates deps - `canvas.addEventListener` changes the event target and retains handlers; `canvas.setPointerCapture` changes pointer capture state.
 
 @example
 ```ts
 setupPointerHandlers(deps);
 ```
 */
export function setupPointerHandlers(deps: PointerHandlerDeps,): void {
  /**
   Destructured up front so each handler closure captures the same handles.
   */
  const {
    canvas,
    ctx,
    getToolMode,
    getCanvasSize,
    textLayer,
    pushSnapshot,
  } = deps;

  /**
   Per-gesture mutable state container.
   
   Stored as object properties so function-root state stays in a `const`
   container (`no-function-root-let` would otherwise reject top-level `let`).
   */
  const eraseState: {
    erasing: boolean;
    erasedInGesture: boolean;
    prevErasePoint: PrevErasePoint;
  } = {
    erasing: false,
    erasedInGesture: false,
    prevErasePoint: { kind: 'none', },
  };

  canvas.addEventListener(
    'pointerdown',
    function handlePointerDown(event: PointerEvent,): void {
      /**
       Captured once so branches downstream do not re-invoke the getter.
       */
      const mode = getToolMode();
      if (mode === 'zoom')
        return;

      if (mode === 'text') {
        /**
         Suppress default focus-management so the created input keeps focus
         */
        event.preventDefault();
        placeTextInput(normalizePointer({
          event,
          canvas,
        },),);
        return;
      }

      canvas.setPointerCapture(event.pointerId,);
      /**
       Canvas dimensions captured at gesture start so handlers stay consistent if the layout shifts.
       */
      const {
        cw,
        ch,
      } = getCanvasSize();
      /**
       Normalized pointer reused for the initial draw/erase action and stored as the previous-eraser sample.
       */
      const point = normalizePointer({
        event,
        canvas,
      },);

      if (mode === 'erase') {
        eraseState.erasing = true;
        eraseState.erasedInGesture = false;
        eraseState.prevErasePoint = { kind: 'none', };
        /**
         Tracks whether any stroke geometry was removed in this tick so the redraw is conditional.
         */
        const strokeErased = eraseStrokesAt({
          point,
          cw,
          ch,
        },);
        /**
         Companion flag for text removal so snapshot pushes happen only when something actually changed.
         */
        const textErased = eraseTextAt({
          point,
          cw,
          ch,
          textLayer,
        },);
        if (strokeErased || textErased)
          eraseState.erasedInGesture = true;
        if (strokeErased) {
          redraw({
            ctx,
            cw,
            ch,
          },);
        }
        eraseState.prevErasePoint = {
          kind: 'at',
          point,
        };
        return;
      }

      startStroke(point,);
    },
  );

  canvas.addEventListener(
    'pointermove',
    function handlePointerMove(event: PointerEvent,): void {
      /**
       Captured once so branches downstream do not re-invoke the getter.
       */
      const mode = getToolMode();
      if ((mode === 'zoom') || (mode === 'text'))
        return;

      /**
       Canvas dimensions captured per move so eraser math stays in sync with the live layout.
       */
      const {
        cw,
        ch,
      } = getCanvasSize();
      /**
       Normalized pointer reused for both draw and erase paths in this tick.
       */
      const point = normalizePointer({
        event,
        canvas,
      },);

      if (mode === 'erase') {
        if (!eraseState.erasing)
          return;
        /**
         Captured once so both erasers receive the same previous-sample shape.
         */
        const prev = eraseState.prevErasePoint;
        /**
         Tracks whether stroke geometry was removed this tick so the redraw is conditional.
         */
        const strokeErased = eraseStrokesAt({
          point,
          ...(prev.kind === 'at' ? { previousPoint: prev.point, } : {}),
          cw,
          ch,
        },);
        /**
         Companion flag for text removal so snapshot pushes happen only when something actually changed.
         */
        const textErased = eraseTextAt({
          point,
          ...(prev.kind === 'at' ? { previousPoint: prev.point, } : {}),
          cw,
          ch,
          textLayer,
        },);
        if (strokeErased || textErased)
          eraseState.erasedInGesture = true;
        if (strokeErased) {
          redraw({
            ctx,
            cw,
            ch,
          },);
        }
        eraseState.prevErasePoint = {
          kind: 'at',
          point,
        };
        return;
      }

      /**
       Line segment to draw incrementally, or absent if not drawing
       */
      const segment = continueStroke(point,);
      if (segment === NO_SEGMENT)
        return;
      ctx.strokeStyle = getStrokeColor();
      ctx.lineWidth = getStrokeWidth();
      ctx.lineCap = 'round';
      ctx.lineJoin = 'round';
      ctx.beginPath();
      ctx.moveTo(
        segment.from[0]
          * cw,
        segment.from[1]
          * ch,
      );
      ctx.lineTo(
        segment.to[0]
          * cw,
        segment.to[1]
          * ch,
      );
      ctx.stroke();
    },
  );

  /**
   Resets draw/erase gesture state at pointer release
   */
  function endGesture(): void {
    endStroke();
    eraseState.erasing = false;
    eraseState.erasedInGesture = false;
    eraseState.prevErasePoint = { kind: 'none', };
  }

  canvas.addEventListener(
    'pointerup',
    function handlePointerUp(): void {
      /**
       Captured once so the snapshot decision and the early return share one tool reading.
       */
      const mode = getToolMode();
      if (mode === 'zoom')
        return;
      if ((mode === 'draw') || ((mode === 'erase') && eraseState
        .erasedInGesture))
        pushSnapshot();
      endGesture();
    },
  );

  canvas.addEventListener(
    'pointercancel',
    endGesture,
  );
}
