/**
 PNG export for the doodle widget.
 
 Renders all layers (background, strokes, text) as a rasterized
 PNG image and triggers a download.
 */

import { readTextEntries, } from './export-text-config.ts';
import {
  type ExportDeps,
  getExportSize,
  renderBaseCanvas,
  triggerDownload,
} from './export.ts';

/**
 Exports the doodle as a PNG file.
 
 Composites background, strokes, and text onto a single canvas via
 {@link renderBaseCanvas}, then triggers a PNG download with
 {@link triggerDownload}.
 
 @param deps - shared {@link ExportDeps} export dependencies
 
 @example
 ```ts
 await exportAsPng({ container, overlay, drawCanvas, textLayer });
 ```
 */
export async function exportAsPng(deps: ExportDeps,): Promise<void> {
  /**
   Text layer destructured separately so the rest of {@link deps} can be passed straight into the renderer.
   */
  const {
    textLayer,
  } = deps;
  /**
   Export dimensions resolved once so both rendering and text placement agree on the canvas size.
   */
  const {
    cw,
    ch,
  } = getExportSize();

  /**
   Composited base canvas plus its drawing context, retained so the text layer can be painted on top.
   */
  const {
    canvas,
    ctx,
  } = await renderBaseCanvas(deps,);

  //region Layer 4: text annotations
  ctx.textBaseline = 'top';

  /**
   Text snapshot pulled before download so DOM mutations during await cannot reorder it.
   */
  const textEntries = readTextEntries({ textLayer, },);
  for (const entry of textEntries) {
    ctx.font = `${String(entry.fontSizePx,)}px system-ui, sans-serif`;
    ctx.fillStyle = entry.color;
    ctx.fillText(
      entry.value,
      entry.xFraction
        * cw,
      entry.yFraction
        * ch,
    );
  }
  //endregion Layer 4

  /**
   PNG-encoded blob the browser can stream into the download anchor.
   */
  const blob = await canvas.convertToBlob({ type: 'image/png', },);
  triggerDownload({
    blob,
    filename: 'doodle.png',
  },);
}
