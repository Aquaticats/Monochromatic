/**
 Zoom state and CSS transform application for the doodle widget canvas.
 
 Tracks zoom scale and pan offset. Zoom is purely visual via CSS
 transforms on a wrapper element; content coordinates are unchanged.
 */

//region Constants

/**
 Minimum zoom scale (no zoom-out past original size)
 */
const MIN_SCALE = 1;

/**
 Maximum zoom scale
 */
const MAX_SCALE = 8;

/**
 Zoom step multiplier per click
 */
const ZOOM_STEP = 2;

//endregion Constants

//region State

/**
 Zoom and pan state container.
 
 Stored as object properties so module-root state stays in a `const`
 container (`no-module-root-let` would otherwise reject top-level `let`).
 */
const zoomState: {
  /**
   Current zoom scale (1 = no zoom)
   */
  scale: number;
  /**
   Pan offset X in CSS pixels relative to container
   */
  panX: number;
  /**
   Pan offset Y in CSS pixels relative to container
   */
  panY: number;
} = {
  scale: 1,
  panX: 0,
  panY: 0,
};

//endregion State

/**
 Returns the current zoom scale.
 
 @returns zoom scale where 1 means no zoom
 
 @example
 ```ts
 const s = getScale();
 ```
 */
export function getScale(): number {
  return zoomState.scale;
}

/**
 Returns the current pan X offset.
 
 @returns pan X in CSS pixels
 
 @example
 ```ts
 const x = getPanX();
 ```
 */
export function getPanX(): number {
  return zoomState.panX;
}

/**
 Returns the current pan Y offset.
 
 @returns pan Y in CSS pixels
 
 @example
 ```ts
 const y = getPanY();
 ```
 */
export function getPanY(): number {
  return zoomState.panY;
}

/**
 Applies the current zoom and pan as a CSS transform on the zoom layer.
 
 Clears the transform entirely at default zoom (scale 1) to
 avoid unnecessary compositing layers.
 
 @param zoomLayer - container element for zoomed content
 
 @example
 ```ts
 applyZoomTransform(zoomLayer);
 ```
 */
export function applyZoomTransform(zoomLayer: HTMLElement,): void {
  if (zoomState.scale
    === 1) {
    zoomLayer.style
      .transform = '';
    return;
  }
  zoomLayer.style
    .transformOrigin = '0 0';
  zoomLayer.style
    .transform = `translate(${String(zoomState.panX,)}px, ${
    String(zoomState.panY,)
  }px) scale(${String(zoomState.scale,)})`;
}

/**
 Clamps pan offset so content always covers the container viewport.
 
 @param containerWidth - container width in CSS pixels
 
 @param containerHeight - container height in CSS pixels
 
 @example
 ```ts
 clampPan({ containerWidth: 800, containerHeight: 600 });
 ```
 */
export function clampPan({
  containerWidth,
  containerHeight,
}: {
  readonly containerWidth: number;
  readonly containerHeight: number;
},): void {
  if (zoomState.scale
    <= 1) {
    zoomState.panX = 0;
    zoomState.panY = 0;
    return;
  }
  zoomState.panX = Math.max(
    (-(zoomState.scale
      - 1)) * containerWidth,
    Math.min(
      0,
      zoomState.panX,
    ),
  );
  zoomState.panY = Math.max(
    (-(zoomState.scale
      - 1)) * containerHeight,
    Math.min(
      0,
      zoomState.panY,
    ),
  );
}

/**
 Sets the pan offset directly, called by the pan gesture module.
 
 @param x - new pan X offset in CSS pixels
 
 @param y - new pan Y offset in CSS pixels
 
 @example
 ```ts
 setPan({ x: 10, y: 20 });
 ```
 */
export function setPan({
  x,
  y,
}: {
  readonly x: number;
  readonly y: number;
},): void {
  zoomState.panX = x;
  zoomState.panY = y;
}

/**
 Zooms in or out centered on a screen-space point relative to the container.
 
 Adjusts pan offset so the content point under the cursor stays
 stationary after the scale change.
 
 @param screenX - pointer X relative to container left edge
 
 @param screenY - pointer Y relative to container top edge
 
 @param direction - 'in' to zoom in, 'out' to zoom out
 
 @param containerWidth - container width in CSS pixels
 
 @param containerHeight - container height in CSS pixels
 
 @param zoomLayer - element to apply CSS transform to
 
 @example
 ```ts
 zoomAt({ screenX: 400, screenY: 300, direction: 'in', containerWidth: 800, containerHeight: 600, zoomLayer });
 ```
 */
export function zoomAt(
  {
    screenX,
    screenY,
    direction,
    containerWidth,
    containerHeight,
    zoomLayer,
  }: {
    readonly screenX: number;
    readonly screenY: number;
    readonly direction: 'in' | 'out';
    readonly containerWidth: number;
    readonly containerHeight: number;
    readonly zoomLayer: HTMLElement;
  },
): void {
  /**
   Direction-aware multiplier so a single formula handles both zoom in and zoom out.
   */
  const factor = direction === 'in' ? ZOOM_STEP : 1 / ZOOM_STEP;
  /**
   Clamped destination scale so the gesture cannot push past the configured bounds.
   */
  const newScale = Math.max(
    MIN_SCALE,
    Math.min(
      MAX_SCALE,
      zoomState.scale
        * factor,
    ),
  );
  if (newScale === zoomState
    .scale)
    return;
  /**
   Ratio between new and old scale for pan adjustment
   */
  const actualFactor = newScale / zoomState
    .scale;
  zoomState.panX = (screenX * (1 - actualFactor)) + (zoomState.panX
    * actualFactor);
  zoomState.panY = (screenY * (1 - actualFactor)) + (zoomState.panY
    * actualFactor);
  zoomState.scale = newScale;
  clampPan({
    containerWidth,
    containerHeight,
  },);
  applyZoomTransform(zoomLayer,);
}

/**
 Resets zoom to 1x with no pan offset.
 
 @param zoomLayer - element to clear CSS transform from
 
 @example
 ```ts
 resetZoom(zoomLayer);
 ```
 */
export function resetZoom(zoomLayer: HTMLElement,): void {
  zoomState.scale = 1;
  zoomState.panX = 0;
  zoomState.panY = 0;
  applyZoomTransform(zoomLayer,);
}

/**
 Re-clamps pan after a container resize and reapplies the transform.
 
 @param containerWidth - new container width in CSS pixels
 
 @param containerHeight - new container height in CSS pixels
 
 @param zoomLayer - element to apply CSS transform to
 
 @example
 ```ts
 refreshZoomTransform({ containerWidth: 800, containerHeight: 600, zoomLayer });
 ```
 */
export function refreshZoomTransform({
  containerWidth,
  containerHeight,
  zoomLayer,
}: {
  readonly containerWidth: number;
  readonly containerHeight: number;
  readonly zoomLayer: HTMLElement;
},): void {
  clampPan({
    containerWidth,
    containerHeight,
  },);
  applyZoomTransform(zoomLayer,);
}
