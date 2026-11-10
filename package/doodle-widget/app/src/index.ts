/**
 Unit-test surface of the doodle widget.

 The widget ships as one self-contained HTML file, so the browser bundle in
 `dist/client/` exports nothing a unit test can import. This entry re-exports
 the stroke state, drawing configuration, eraser, PDF export, and page-state
 modules, which `rolldown.node.config.ts` bundles to `dist/final/node/`, so unit
 tests load built output rather than sibling source, as
 `test-import/require-eventual-artifact` requires.

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

/**
 Multi-page PDF export and the overlay-markup borrowing it restores through, so
 a test can fail a page raster and read the overlay back.

 @internal
 */
export { exportAsPdf, } from './client/export-pdf.ts';
export { borrowOverlayMarkup, } from './client/borrow-overlay-markup.ts';

/**
 Page-state initializer and page switch, so a test can start the export with a
 known page set and a known open page.

 @internal
 */
export {
  initPages,
  switchToPage,
} from './client/pages.ts';
