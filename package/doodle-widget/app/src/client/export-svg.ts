/**
 SVG export for the doodle widget.
 
 Builds a self-contained SVG document with vector paths for strokes,
 real `<text>` elements for annotations, and the background SVG
 embedded as a nested `<svg>`.
 */

import {
  getStrokes,
  type NormalizedPoint,
} from './drawing.ts';
import { readTextEntries, } from './export-text-config.ts';
import {
  type ExportDeps,
  getExportSize,
  getRenderedSize,
  triggerDownload,
} from './export.ts';
import { MIN_STROKE_POINTS, } from './stroke-renderer.ts';
import {
  measureSvgOverlay,
  NO_SVG_OVERLAY,
} from './svg-overlay-measure.ts';

/**
 SVG XML namespace
 */
const SVG_NS = 'http://www.w3.org/2000/svg';

/**
 Exports the doodle as an SVG file.
 
 Produces a self-contained SVG with vector `<path>` elements for
 strokes, `<text>` elements for annotations (selectable and
 searchable), and the background SVG embedded as a nested `<svg>`.
 
 @param container - canvas container for sizing
 
 @param overlay - SVG overlay div
 
 @param textLayer - div containing text input elements
 
 @example
 ```ts
 exportAsSvg({ container, overlay, drawCanvas, textLayer });
 ```
 */
export function exportAsSvg(
  {
    container,
    overlay,
    textLayer,
  }: ExportDeps,
): void {
  /**
   Export dimensions resolved once so both root SVG and child paths share the same coordinate space.
   */
  const {
    cw,
    ch,
  } = getExportSize();

  /**
   Scale from rendered page size to letter export size
   */
  const {
    cw: renderedCw,
  } = getRenderedSize(container,);
  /**
   Ratio applied to overlay positions so the embedded SVG lines up with the rasterized geometry.
   */
  const exportScale = cw / renderedCw;

  /**
   Root SVG element accumulating every layer before serialization.
   */
  const svg = document.createElementNS(
    SVG_NS,
    'svg',
  );
  svg.setAttribute(
    'xmlns',
    SVG_NS,
  );
  svg.setAttribute(
    'viewBox',
    `0 0 ${String(cw,)} ${String(ch,)}`,
  );
  svg.setAttribute(
    'width',
    String(cw,),
  );
  svg.setAttribute(
    'height',
    String(ch,),
  );

  //region White background
  /**
   Opaque rectangle so the export renders against a fixed colour rather than the host page background.
   */
  const bgRect = document.createElementNS(
    SVG_NS,
    'rect',
  );
  bgRect.setAttribute(
    'width',
    String(cw,),
  );
  bgRect.setAttribute(
    'height',
    String(ch,),
  );
  bgRect.setAttribute(
    'fill',
    'white',
  );
  svg.append(bgRect,);
  //endregion White background

  //region Strokes (behind SVG overlay)
  /**
   Snapshot pulled before the loop so concurrent edits cannot reshape the array mid-render.
   */
  const strokes = getStrokes();
  for (const stroke of strokes) {
    if (stroke.points
      .length
      < MIN_STROKE_POINTS)
      continue;
    /**
     One path per stroke so the colour and width attributes do not leak between strokes.
     */
    const path = document.createElementNS(
      SVG_NS,
      'path',
    );
    /**
     SVG path data built from normalized stroke coordinates
     */
    const d = stroke
      .points
      .map(
      function formatPoint(
        [nx, ny,]: NormalizedPoint,
        index: number,
      ): string {
          /**
           First point opens the path with a move, later points draw lines.
           */
        const cmd = index === 0 ? 'M' : 'L';
        return `${cmd}${String(nx * cw,)},${String(ny * ch,)}`;
      },
      )
      .join(' ',);
    path.setAttribute(
      'd',
      d,
    );
    path.setAttribute(
      'stroke',
      stroke.color,
    );
    path.setAttribute(
      'stroke-width',
      String(stroke.width,),
    );
    path.setAttribute(
      'fill',
      'none',
    );
    path.setAttribute(
      'stroke-linecap',
      'round',
    );
    path.setAttribute(
      'stroke-linejoin',
      'round',
    );
    svg.append(path,);
  }
  //endregion Strokes

  //region Background SVG with multiply blending (on top of strokes)
  /**
   Position metadata so the cloned overlay lines up with the rasterized geometry.
   */
  const overlayInfo = measureSvgOverlay({
    container,
    overlay,
  },);
  if (overlayInfo !== NO_SVG_OVERLAY) {
    overlayInfo.clone
      .setAttribute(
      'x',
      String(overlayInfo.offsetX
        * exportScale,),
      );
    overlayInfo.clone
      .setAttribute(
      'y',
      String(overlayInfo.offsetY
        * exportScale,),
      );
    overlayInfo.clone
      .setAttribute(
      'width',
      String(overlayInfo.width
        * exportScale,),
      );
    overlayInfo.clone
      .setAttribute(
      'height',
      String(overlayInfo.height
        * exportScale,),
      );
    overlayInfo.clone
      .setAttribute(
      'style',
      'mix-blend-mode:multiply',
      );
    svg.append(overlayInfo.clone,);
  }
  //endregion Background SVG

  //region Text annotations
  /**
   Snapshot of the live text inputs so DOM order, not iteration order, drives the SVG output.
   */
  const textEntries = readTextEntries({ textLayer, },);
  for (const entry of textEntries) {
    /**
     One `<text>` node per entry so each annotation can carry its own colour and size.
     */
    const text = document.createElementNS(
      SVG_NS,
      'text',
    );
    text.setAttribute(
      'x',
      String(entry.xFraction
        * cw,),
    );
    text.setAttribute(
      'y',
      String(entry.yFraction
        * ch,),
    );
    text.setAttribute(
      'font-family',
      'system-ui, sans-serif',
    );
    text.setAttribute(
      'font-size',
      String(entry.fontSizePx,),
    );
    text.setAttribute(
      'fill',
      entry.color,
    );
    text.setAttribute(
      'dominant-baseline',
      'hanging',
    );
    text.textContent = entry.value;
    svg.append(text,);
  }
  //endregion Text annotations

  /**
   Serialized SVG markup for download
   */
  const markup = new XMLSerializer().serializeToString(svg,);
  /**
   Wrapped in a blob so the download helper can stream it through an object URL.
   */
  const blob = new Blob(
    [markup,],
    { type: 'image/svg+xml;charset=utf-8', },
  );
  triggerDownload({
    blob,
    filename: 'doodle.svg',
  },);
}
