/**
 PDF export for the doodle widget.
 
 Renders all pages as a multi-page PDF with rasterized backgrounds
 and strokes, but preserves text annotations as real, selectable
 PDF text. Drawing strokes are composited between SVG fills and
 SVG linework, matching the on-screen layer order.
 */

import { jsPDF, } from 'jspdf';

import { renderPageCanvas, } from './export-pdf-page.ts';
import {
  TEXT_COLOR_RGB,
  textEntriesToExport,
} from './export-text-config.ts';
import {
  type ExportDeps,
  getExportSize,
} from './export.ts';
import { snapshotAllPages, } from './pages.ts';

//region Constants

/**
 Conversion factor from CSS pixels to PDF points (72/96)
 */
const PX_TO_PT = 0.75;

/**
 Bit shift for extracting red channel from packed 24-bit RGB
 */
const RED_SHIFT = 16;

/**
 Bit shift for extracting green channel from packed 24-bit RGB
 */
const GREEN_SHIFT = 8;

/**
 Bit mask for isolating a single 8-bit color channel
 */
const CHANNEL_MASK = 0xFF;

/**
 Hexadecimal radix for Number.parseInt
 */
const HEX_RADIX = 16;

//endregion Constants

/**
 Converts a `#rrggbb` hex color string to RGB components.
 
 @param hex - color string in `#rrggbb` format
 
 @returns RGB components as 0-255 integers
 
 @example
 ```ts
 const { r, g, b } = hexToRgb('#c24e2e');
 ```
 */
function hexToRgb(hex: string,): {
  r: number;
  g: number;
  b: number;
} {
  /**
   24-bit integer parsed from the hex digits
   */
  const packed = Number.parseInt(
    hex.slice(1,),
    HEX_RADIX,
  );
  return {
    r: (packed >> RED_SHIFT) & CHANNEL_MASK,
    g: (packed >> GREEN_SHIFT) & CHANNEL_MASK,
    b: packed & CHANNEL_MASK,
  };
}

/**
 Exports all pages as a multi-page PDF file.
 
 Pages are gathered with {@link snapshotAllPages}, each rendered with
 strokes beneath SVG linework via {@link renderPageCanvas}. Text
 annotations filtered by {@link textEntriesToExport} are overlaid as
 real PDF text so they remain selectable and searchable in the output.
 
 @param deps - shared {@link ExportDeps} export dependencies
 
 @example
 ```ts
 await exportAsPdf({ container, overlay, drawCanvas, textLayer });
 ```
 */
export async function exportAsPdf(deps: ExportDeps,): Promise<void> {
  /**
   Destructured up front so subsequent calls reuse the same handles.
   */
  const {
    container,
    overlay,
    textLayer,
  } = deps;
  /**
   Page dimensions resolved once so layout math stays consistent across pages.
   */
  const {
    cw,
    ch,
  } = getExportSize();

  /**
   Snapshot all pages (saves current page's live state)
   */
  const allPages = snapshotAllPages({
    overlay,
    textLayer,
  },);
  /**
   Save overlay HTML to restore after export
   */
  const savedOverlayHtml = overlay.innerHTML;

  //region PDF document setup (all dimensions in points)
  /**
   Page width in PDF points
   */
  const pageW = cw * PX_TO_PT;
  /**
   Page height in PDF points
   */
  const pageH = ch * PX_TO_PT;

  /**
   PDF document built with one orientation guess so portrait and landscape inputs both fit.
   */
  const doc = new jsPDF({
    orientation: pageW >= pageH ? 'l' : 'p',
    unit: 'pt',
    format: [
      pageW,
      pageH,
    ],
  },);
  //endregion PDF document setup

  //region Render each page
  for (const [pageIndex, page,] of allPages.entries()) {
    if (pageIndex > 0) {
      doc.addPage([
        pageW,
        pageH,
      ],);
    }

    /**
     Composited raster image for this page
     */
    // oxlint-disable-next-line no-await-in-loop -- pages render sequentially; each mutates the shared overlay element
    const pageCanvas = await renderPageCanvas({
      svgBackground: page.svgBackground,
      strokes: page.strokes,
      container,
      overlay,
    },);

    /**
     PNG image data as byte array for jsPDF embedding
     */
    // oxlint-disable-next-line no-await-in-loop -- depends on sequential page rendering above
    const blob = await pageCanvas.convertToBlob({ type: 'image/png', },);
    /* oxlint-disable no-await-in-loop -- depends on sequential blob above */
    /**
     Bytes pulled off the blob so the synchronous {@link Uint8Array} can wrap them.
     */
    const buffer = await blob.arrayBuffer();
    /* oxlint-enable no-await-in-loop */
    /**
     Typed view jsPDF requires for binary image embedding.
     */
    const imageData = new Uint8Array(buffer,);
    doc.addImage(
      imageData,
      'PNG',
      0,
      0,
      pageW,
      pageH,
    );

    //region Overlay text as real PDF text
    /**
     Filtered to the entries safe to render as selectable PDF text.
     */
    const textEntries = textEntriesToExport(page.textEntries,);
    for (const entry of textEntries) {
      /**
       Font size in points
       */
      const fontSizePt = entry.fontSizePx
        * PX_TO_PT;
      doc.setFontSize(fontSizePt,);
      if (entry.color
        .startsWith('#',)) {
        /**
         Decomposed channels so jsPDF's three-arg setter receives integers, not a hex string.
         */
        const rgb = hexToRgb(entry.color,);
        doc.setTextColor(
          rgb.r,
          rgb.g,
          rgb.b,
        );
      }
      else {
        doc.setTextColor(
          TEXT_COLOR_RGB.r,
          TEXT_COLOR_RGB.g,
          TEXT_COLOR_RGB.b,
        );
      }
      doc.text(
        entry.value,
        entry.xFraction
          * pageW,
        entry.yFraction
          * pageH,
        {
          baseline: 'top',
        },
      );
    }
    //endregion Overlay text
  }
  //endregion Render each page

  /**
   Restore overlay to its original state
   */
  overlay.innerHTML = savedOverlayHtml;

  doc.save('doodle.pdf',);
}
