/**
 Pan drag gesture handling for the doodle widget zoom tool.
 
 Tracks pointer movement during a drag and updates the zoom
 pan offset. Distinguishes clicks from drags via a distance
 threshold so that small-movement clicks still register as zooms.
 */

import {
  applyZoomTransform,
  clampPan,
  setPan,
} from './zoom.ts';

/**
 Minimum pointer movement in pixels to distinguish drag from click
 */
const DRAG_THRESHOLD = 4;

//region State

/**
 Pan drag gesture state container.
 
 Stored as object properties so module-root state stays in a `const`
 container (`no-module-root-let` would otherwise reject top-level `let`).
 */
const panState: {
  /**
   Whether a pan drag gesture is active
   */
  panning: boolean;
  /**
   Pointer X at pan start in screen pixels
   */
  panStartPointerX: number;
  /**
   Pointer Y at pan start in screen pixels
   */
  panStartPointerY: number;
  /**
   Pan offset X at drag start
   */
  panStartOffsetX: number;
  /**
   Pan offset Y at drag start
   */
  panStartOffsetY: number;
  /**
   Whether pointer moved enough to count as drag (not click)
   */
  dragExceededThreshold: boolean;
} = {
  panning: false,
  panStartPointerX: 0,
  panStartPointerY: 0,
  panStartOffsetX: 0,
  panStartOffsetY: 0,
  dragExceededThreshold: false,
};

//endregion State

/**
 Begins a pan drag gesture at the given pointer position.
 
 @param event - pointer event that started the drag
 
 @param currentPanX - current pan X offset to snapshot
 
 @param currentPanY - current pan Y offset to snapshot
 
 @example
 ```ts
 startPan({ event, currentPanX: getPanX(), currentPanY: getPanY() });
 ```
 */
export function startPan({
  event,
  currentPanX,
  currentPanY,
}: {
  readonly event: PointerEvent;
  readonly currentPanX: number;
  readonly currentPanY: number;
},): void {
  panState.panning = true;
  panState.dragExceededThreshold = false;
  panState.panStartPointerX = event.clientX;
  panState.panStartPointerY = event.clientY;
  panState.panStartOffsetX = currentPanX;
  panState.panStartOffsetY = currentPanY;
}

/**
 Continues a pan drag gesture, updating the pan offset.
 
 @param event - pointer move event
 
 @param containerWidth - container width in CSS pixels
 
 @param containerHeight - container height in CSS pixels
 
 @param zoomLayer - element to apply CSS transform to
 
 @returns whether the pointer has moved enough to count as a drag
 
 @example
 ```ts
 const dragging = continuePan({ event, containerWidth: cw, containerHeight: ch, zoomLayer });
 ```
 */
export function continuePan({
  event,
  containerWidth,
  containerHeight,
  zoomLayer,
}: {
  readonly event: PointerEvent;
  readonly containerWidth: number;
  readonly containerHeight: number;
  readonly zoomLayer: HTMLElement;
},): boolean {
  if (!panState.panning)
    return false;

  /**
   Pointer displacement on the x axis since the gesture began.
   */
  const dx = event.clientX
    - panState
      .panStartPointerX;
  /**
   Companion to {@link dx} on the y axis.
   */
  const dy = event.clientY
    - panState
      .panStartPointerY;

  if (!panState.dragExceededThreshold) {
    if ((Math.abs(dx,)
      > DRAG_THRESHOLD) || (Math.abs(dy,)
        > DRAG_THRESHOLD))
      panState.dragExceededThreshold = true;
    else
      return false;
  }

  setPan({
    x: panState.panStartOffsetX
      + dx,
    y: panState.panStartOffsetY
      + dy,
  },);
  clampPan({
    containerWidth,
    containerHeight,
  },);
  applyZoomTransform(zoomLayer,);
  return true;
}

/**
 Ends a pan drag gesture.
 
 @returns whether the gesture was a drag (moved beyond threshold) or a click
 
 @example
 ```ts
 const wasDrag = endPan();
 ```
 */
export function endPan(): boolean {
  panState.panning = false;
  /**
   Captured before resetting so the caller can distinguish a drag from a click.
   */
  const wasDrag = panState.dragExceededThreshold;
  panState.dragExceededThreshold = false;
  return wasDrag;
}
