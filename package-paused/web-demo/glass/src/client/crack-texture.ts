/**
 * Canvas painter for the cracked-but-holding stage.
 *
 * Real glass shows its spider web the instant it is struck, then the web
 * collapses. Drawing the exact Voronoi cell edges that later become the
 * shards makes the collapse continuous: every crack line turns into a
 * real shard boundary.
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  type PaneCell,
  type PanePoint,
  polygonCentroid,
} from './fracture.ts';

/**
 * Logger root shared by the glass demo client modules.
 */
const parentLogger = tagged({ tag: 'web-demo-glass', },);

/**
 * Tagged logger for the crack texture painter.
 */
const l = tagged({
  tag: 'crack-texture',
  l: parentLogger,
},);

/**
 * Tuning constants for crack rendering. One table instead of scattered
 * magic values.
 */
export const CRACK_TUNING = {
  /**
   * Canvas edge length in pixels; square regardless of pane aspect.
   */
  textureSize: 1_024,
  /**
   * Line width in pixels for cracks touching the impact point.
   */
  strokeWidthNear: 4.5,
  /**
   * Line width in pixels for cracks at the far pane corner.
   */
  strokeWidthFar: 1.1,
  /**
   * Stroke alpha for cracks touching the impact point.
   */
  strokeAlphaNear: 0.9,
  /**
   * Stroke alpha for cracks at the far pane corner.
   */
  strokeAlphaFar: 0.28,
  /**
   * Radius of the bright pulverized blob at the impact, in pixels.
   */
  impactBlobRadius: 26,
  /**
   * Gradient stop where the blob starts cooling off.
   */
  blobMidStop: 0.4,
} as const;

/**
 * Paints the fracture web onto a fresh canvas: every cell outline stroked
 * in pale glass-white, brighter and wider near the impact, plus a hot blob
 * at the impact itself. Transparent background so the canvas overlays the
 * pane as an additive decal.
 *
 * @param cells - fracture cells whose edges become the crack lines
 *
 * @param impact - impact point in pane-local meters
 *
 * @param halfWidth - pane half width in meters
 *
 * @param halfHeight - pane half height in meters
 *
 * @param omitImpactBlob - set when the impact region is a real hole with
 * no glass left to whiten
 *
 * @returns canvas ready to wrap in a texture
 *
 * @throws Error when the 2d canvas context is unavailable
 *
 * @example
 * ```ts
 * const canvas = paintCrackWeb({
 *   cells,
 *   impact: { x: 0.1, y: -0.2 },
 *   halfWidth: 1.1,
 *   halfHeight: 1.5,
 * },);
 * ```
 */
export function paintCrackWeb(
  {
    cells,
    impact,
    halfWidth,
    halfHeight,
    omitImpactBlob,
  }: Readonly<{
    cells: readonly PaneCell[];
    impact: PanePoint;
    halfWidth: number;
    halfHeight: number;
    omitImpactBlob?: true;
  }>,
): HTMLCanvasElement {
  /**
   * Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: paintCrackWeb.name,
    l,
  },);
  /**
   * Fresh canvas per crack; panes crack rarely, so allocation churn is
   * negligible next to texture upload cost.
   */
  const canvas = document.createElement('canvas',);
  canvas.width = CRACK_TUNING.textureSize;
  canvas.height = CRACK_TUNING.textureSize;
  /**
   * 2d context the whole web draws through.
   */
  const context = canvas.getContext('2d',);
  if (context === null)
    throw new Error('2d canvas context unavailable for crack texture',);
  /**
   * Pane-local x to canvas pixel u.
   *
   * @param x - pane-local x in meters
   *
   * @returns canvas x in pixels
   */
  function toU(x: number,): number {
    return ((x + halfWidth) / (2 * halfWidth)) * CRACK_TUNING.textureSize;
  }
  /**
   * Pane-local y to canvas pixel v; canvas v grows downward.
   *
   * @param y - pane-local y in meters
   *
   * @returns canvas y in pixels
   */
  function toV(y: number,): number {
    return ((halfHeight - y) / (2 * halfHeight)) * CRACK_TUNING.textureSize;
  }
  /**
   * Farthest pane corner distance from the impact, normalizing the
   * width/alpha falloff.
   */
  const maxReach = Math.hypot(
    halfWidth + Math.abs(impact.x,),
    halfHeight + Math.abs(impact.y,),
  );
  context.lineCap = 'round';
  context.lineJoin = 'round';
  // 'lighter' stacks the two strokes each shared edge receives, which
  // brightens the dense center exactly where real webs whiten.
  context.globalCompositeOperation = 'lighter';
  for (const cell of cells) {
    /**
     * Cell centroid reused for the distance falloff.
     */
    const centroid = polygonCentroid(cell,);
    /**
     * Distance from this cell to the impact, driving width and alpha.
     */
    const distance = Math.hypot(
      centroid.x - impact.x,
      centroid.y - impact.y,
    );
    /**
     * Normalized falloff, 0 at the impact and 1 at the farthest corner.
     */
    const falloff = Math.min(
      1,
      distance / maxReach,
    );
    context.lineWidth = CRACK_TUNING.strokeWidthNear
      + ((CRACK_TUNING.strokeWidthFar - CRACK_TUNING.strokeWidthNear)
      * falloff);
    context.strokeStyle = `rgba(228, 246, 255, ${
      String(
        CRACK_TUNING.strokeAlphaNear
          + ((CRACK_TUNING.strokeAlphaFar - CRACK_TUNING.strokeAlphaNear)
          * falloff),
      )
    })`;
    context.beginPath();
    for (const [index, vertex,] of cell.entries()) {
      if (index === 0)
        context.moveTo(
          toU(vertex.x,),
          toV(vertex.y,),
        );
      else
        context.lineTo(
          toU(vertex.x,),
          toV(vertex.y,),
        );
    }
    context.closePath();
    context.stroke();
  }
  //region Impact blob: pulverized hot center where glass whitens fully
  if (omitImpactBlob === true) {
    innerL.debug(`painted crack web: ${String(cells.length,)} rim cells, hole left clear`,);
    return canvas;
  }
  /**
   * Radial gradient for the pulverized center.
   */
  const blob = context.createRadialGradient(
    toU(impact.x,),
    toV(impact.y,),
    0,
    toU(impact.x,),
    toV(impact.y,),
    CRACK_TUNING.impactBlobRadius,
  );
  blob.addColorStop(
    0,
    'rgba(255, 255, 255, 0.95)',
  );
  blob.addColorStop(
    CRACK_TUNING.blobMidStop,
    'rgba(220, 242, 255, 0.5)',
  );
  blob.addColorStop(
    1,
    'rgba(200, 235, 255, 0)',
  );
  context.fillStyle = blob;
  context.beginPath();
  context.arc(
    toU(impact.x,),
    toV(impact.y,),
    CRACK_TUNING.impactBlobRadius,
    0,
    Math.PI * 2,
  );
  context.fill();
  //endregion
  innerL.debug(`painted crack web: ${String(cells.length,)} cells`,);
  return canvas;
}
