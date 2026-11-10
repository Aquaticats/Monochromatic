/**
 Unit-test surface of the doodle widget.

 The widget ships as one self-contained HTML file, so the browser bundle in
 `dist/client/` exports nothing a unit test can import. This entry re-exports
 the DOM-free stroke state, drawing configuration, and eraser modules, which
 `rolldown.node.config.ts` bundles to `dist/final/node/`, so unit tests load
 built output rather than sibling source, as `test-import/require-eventual-artifact`
 requires.

 Every export is internal: the package declares no `exports` map and nothing
 outside the package imports this entry, so it tracks the tests rather than a
 consumer contract.

 @module
 */

/**
 Stroke state the eraser reads and replaces, so a test can arrange a canvas.

 @internal
 */
export {
  getStrokes,
  setStrokes,
  type NormalizedPoint,
  type StrokeData,
} from './client/drawing.ts';

/**
 Active stroke width, which is also the eraser radius the assertions derive.

 @internal
 */
export { getStrokeWidth, } from './client/drawing-config.ts';

/**
 Eraser geometry: per-point and per-segment erasure, and stroke splitting.

 @internal
 */
export { eraseStrokesAt, } from './client/eraser-strokes.ts';
