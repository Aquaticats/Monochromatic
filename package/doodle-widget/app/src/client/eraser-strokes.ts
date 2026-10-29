/**
 Stroke erasure for the doodle widget.
 
 Tests stroke points **and stroke segments** against the eraser's
 travel segment between frames, so fast drags do not skip over
 strokes even when stroke points are widely spaced.
 */

import { getStrokeWidth, } from './drawing-config.ts';
import {
  denormalizePoint,
  getStrokes,
  type NormalizedPoint,
  setStrokes,
  type StrokeData,
} from './drawing.ts';
import {
  distToSegmentSq,
  segToSegDistSq,
} from './geometry.ts';

/**
 Minimum number of points required for a valid sub-stroke after splitting
 */
const MIN_SEGMENT_POINTS = 2;

/**
 Erases stroke segments near the eraser path.
 
 Tests each stroke point (via {@link distToSegmentSq}) **and each
 stroke segment** (via {@link segToSegDistSq}) against the line
 segment from `previousPoint` to `point` (the eraser's travel path
 between frames). When `previousPoint` is omitted (first event of a
 gesture), tests against the single point only.
 
 The segment-to-segment check catches cases where the eraser crosses
 a stroke line between two widely-spaced points without being close
 enough to either individual point.
 
 @param point - current eraser position in normalized [0..1] space
 
 @param previousPoint - previous eraser position; omitted on first event
 
 @param cw - current canvas width in CSS pixels
 
 @param ch - current canvas height in CSS pixels
 
 @returns `true` if any stroke was modified
 
 @example
 ```ts
 const erased = eraseStrokesAt({
   point: [0.5, 0.5], previousPoint: [0.4, 0.4], cw: 800, ch: 600,
 });
 if (erased) redraw({ ctx, cw, ch });
 ```
 */
export function eraseStrokesAt({
  point,
  previousPoint,
  cw,
  ch,
}: {
  readonly point: NormalizedPoint;
  readonly previousPoint?: NormalizedPoint;
  readonly cw: number;
  readonly ch: number;
},): boolean {
  /**
   Eraser radius in CSS pixels, matching the active stroke width
   */
  const radiusPx = getStrokeWidth();
  /**
   Eraser segment endpoint in pixel space
   */
  const {
    px: bx,
    py: by,
  } = denormalizePoint({
    point,
    cw,
    ch,
  },);
  /**
   Eraser segment start in pixel space (same as end when no previous point)
   */
  const {
    px: ax,
    py: ay,
  } = previousPoint !== undefined
    ? denormalizePoint({
      point: previousPoint,
      cw,
      ch,
    },)
    : {
      px: bx,
      py: by,
    };
  /**
   Squared radius for distance comparison without sqrt
   */
  const radiusSq = radiusPx * radiusPx;

  /**
   Snapshot taken once so the loop can build the replacement list in isolation.
   */
  const oldStrokes = getStrokes();
  /**
   Flag flipped only when at least one stroke loses a point, so the caller can skip redraws.
   */
  let erased = false;
  /**
   Replacement list built alongside the iteration to avoid in-place mutation.
   */
  const newStrokes: StrokeData[] = [];

  for (const stroke of oldStrokes) {
    /**
     Per-stroke flag separating untouched strokes from segment-split survivors.
     */
    let strokeModified = false;
    /**
     Sub-stroke segments surviving after erasure
     */
    const segments: NormalizedPoint[][] = [];
    /**
     Points accumulating for the current sub-stroke
     */
    let currentSegment: NormalizedPoint[] = [];

    /**
     Previous stroke point in pixel space, for segment-to-segment checks
     */
    let prevStrokePx = 0;
    /**
     Companion to {@link prevStrokePx} on the y axis.
     */
    let prevStrokePy = 0;

    for (let loopIndex = 0; loopIndex
      < stroke
        .points
        .length; loopIndex++) {
      /**
       Skipped when undefined so sparse arrays do not break segment math.
       */
      const p = stroke.points[loopIndex];
      if (p === undefined)
        continue;
      /**
       Stroke point in pixel space
       */
      const {
        px,
        py,
      } = denormalizePoint({
        point: p,
        cw,
        ch,
      },);
      /**
       Squared distance from stroke point to eraser travel segment
       */
      const pointDistSq = distToSegmentSq({
        px,
        py,
        ax,
        ay,
        bx,
        by,
      },);

      /**
       Mutable so the fallback segment-distance check can promote it to `true`.
       */
      let shouldErase = pointDistSq <= radiusSq;

      // When the point itself is not close enough, check whether the
      // stroke *segment* (line from previous point to this point) is
      // close to or intersects the eraser segment. This catches fast
      // erases that cross between widely-spaced stroke points.
      if ((!shouldErase) && (loopIndex > 0)) {
        /**
         Segment-to-segment distance covers fast drags that skip between stroke samples.
         */
        const segDistSq = segToSegDistSq({
          a1x: prevStrokePx,
          a1y: prevStrokePy,
          a2x: px,
          a2y: py,
          b1x: ax,
          b1y: ay,
          b2x: bx,
          b2y: by,
        },);
        shouldErase = segDistSq <= radiusSq;
      }

      if (shouldErase) {
        strokeModified = true;
        if (currentSegment.length
          >= MIN_SEGMENT_POINTS)
          segments.push(currentSegment,);
        currentSegment = [];
      }
      else {
        currentSegment.push(p,);
      }

      prevStrokePx = px;
      prevStrokePy = py;
    }

    if (currentSegment.length
      >= MIN_SEGMENT_POINTS)
      segments.push(currentSegment,);

    if (strokeModified) {
      erased = true;
      for (const seg of segments) {
        newStrokes.push({
          points: [...seg,],
          color: stroke.color,
          width: stroke
            .width,
        },);
      }
    }
    else {
      newStrokes.push(stroke,);
    }
  }

  if (erased)
    setStrokes(newStrokes,);

  return erased;
}
