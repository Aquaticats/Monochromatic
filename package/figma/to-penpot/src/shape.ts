/**
 Shape-assembly helpers shared by node and document conversion.
 
 @module figma-to-penpot-shape
 */

import {
  IDENTITY_TRANSFORM,
  ROOT_FRAME_EXTENT,
  ZERO_UUID,
} from './constants.ts';
import {
  computeSelRect,
  figmaTransformToPenpot,
} from './geometry.ts';
import {
  collectPaints,
  figmaPaintToFill,
  figmaPaintToStroke,
} from './paint.ts';
import {
  type FigmaRecord,
  isRecord,
  numberOr,
  recordArray,
  SKIP,
  stringOr,
} from './read.ts';
import type {
  PenpotFill,
  PenpotPoints,
  PenpotSelRect,
  PenpotShape,
  PenpotStroke,
  Uuid,
} from './types.ts';

/**
 Effective bounding geometry derived from a Figma node.
 */
export type ShapeGeometry = {
  hasGeometry: boolean;
  x: number;
  y: number;
  width: number;
  height: number;
  selrect: PenpotSelRect;
  points: PenpotPoints;
};

/**
 Build a zeroed selrect for shapes without measurable bounds.
 
 @returns fresh all-zero selrect
 
 @example
 ```ts
 const selrect = zeroSelRect();
 ```
 */
function zeroSelRect(): PenpotSelRect {
  return {
    x: 0,
    y: 0,
    width: 0,
    height: 0,
    x1: 0,
    y1: 0,
    x2: 0,
    y2: 0,
  };
}

/**
 Build zeroed corner points for shapes without measurable bounds.
 
 @returns fresh 4-tuple of origin points
 
 @example
 ```ts
 const points = zeroPoints();
 ```
 */
function zeroPoints(): PenpotPoints {
  return [
    {
      x: 0,
      y: 0,
    },
    {
      x: 0,
      y: 0,
    },
    {
      x: 0,
      y: 0,
    },
    {
      x: 0,
      y: 0,
    },
  ];
}

/**
 Derive effective geometry (position, size, selrect, points) for a node.
 
 Position comes from the transform's translation; size from the optional
 `size` struct. Nodes without measurable bounds report `hasGeometry: false`
 and zeroed rect/points.
 
 @param nc - Figma NodeChange record
 
 @returns effective geometry for the shape
 
 @example
 ```ts
 const geom = geometryOf(nc);
 ```
 */
export function geometryOf(nc: FigmaRecord,): ShapeGeometry {
  /**
   SVG-shaped transform; its `e`/`f` translation doubles as the shape's x/y.
   */
  const transform = figmaTransformToPenpot(nc.transform,);
  /**
   Optional size struct from Figma.
   */
  const {size} = nc;
  /**
   Width from the size struct, 0 when absent.
   */
  const width = isRecord(size,)
    ? numberOr({
      value: size.x,
      fallback: 0,
    },)
    : 0;
  /**
   Height from the size struct, 0 when absent.
   */
  const height = isRecord(size,)
    ? numberOr({
      value: size.y,
      fallback: 0,
    },)
    : 0;
  /**
   True only when the node has a measurable bounding rect.
   */
  const hasGeometry = (width > 0) && (height > 0);
  if (!hasGeometry) {
    return {
      hasGeometry,
      x: 0,
      y: 0,
      width: 0,
      height: 0,
      selrect: zeroSelRect(),
      points: zeroPoints(),
    };
  }
  /**
   Selrect and corner points computed from the measured bounds.
   */
  const {
    selrect,
    points,
  } = computeSelRect({
    x: transform.e,
    y: transform.f,
    width,
    height,
  },);
  return {
    hasGeometry,
    x: transform.e,
    y: transform.f,
    width,
    height,
    selrect,
    points,
  };
}

/**
 Convert a node's fill paints to Penpot fills, dropping unsupported paints.
 
 @param nc - Figma NodeChange record
 
 @returns Penpot fills
 
 @example
 ```ts
 const fills = collectFills(nc);
 ```
 */
export function collectFills(nc: FigmaRecord,): PenpotFill[] {
  return collectPaints({
    paints: nc.fillPaints,
    convert: figmaPaintToFill,
  },);
}

/**
 Convert a node's stroke paints to Penpot strokes, dropping unsupported paints.
 
 @param nc - Figma NodeChange record
 
 @returns Penpot strokes
 
 @example
 ```ts
 const strokes = collectStrokes(nc);
 ```
 */
export function collectStrokes(nc: FigmaRecord,): PenpotStroke[] {
  /**
   Stroke width with 0-fallback so a missing weight stays encodable.
   */
  const strokeWeight = numberOr({
    value: nc.strokeWeight,
    fallback: 0,
  },);
  /**
   Stroke alignment enum, defaulting to centered.
   */
  const strokeAlign = stringOr({
    value: nc.strokeAlign,
    fallback: 'CENTER',
  },);
  /**
   Converter binding this node's stroke width and alignment to one paint.
   
   @param paint - Figma paint record
   
   @returns Penpot stroke, or {@link SKIP} for unsupported paints
   */
  function convertStroke(paint: FigmaRecord,): PenpotStroke | typeof SKIP {
    return figmaPaintToStroke({
      paint,
      strokeWeight,
      strokeAlign,
    },);
  }
  return collectPaints({
    paints: nc.strokePaints,
    convert: convertStroke,
  },);
}

/**
 Extract the first SVG path string from a Figma geometry array.
 
 @param geometry - Figma fill/stroke geometry value of unknown type
 
 @returns path string, or {@link SKIP} when no path is present
 
 @example
 ```ts
 const path = geometryPath(nc.fillGeometry);
 if (path !== SKIP) shape.content = path;
 ```
 */
export function geometryPath(geometry: unknown,): string | typeof SKIP {
  /**
   First geometry entry, or undefined for an empty/non-array value.
   */
  const [first,] = recordArray(geometry,);
  if (!isRecord(first,))
    return SKIP;
  /**
   Candidate SVG path string off the first geometry entry.
   */
  const {path} = first;
  return ((typeof path) === 'string') ? path : SKIP;
}

/**
 Build the implicit root frame Penpot requires on every page.
 
 @param pageId - page the root frame belongs to
 
 @returns root-frame {@link PenpotShape} with a degenerate non-zero extent
 
 @example
 ```ts
 shapes.set(ZERO_UUID, makeRootFrame(pageId));
 ```
 */
export function makeRootFrame(pageId: Uuid,): PenpotShape {
  /**
   Right/bottom edge of the degenerate root-frame rect.
   */
  const extent = ROOT_FRAME_EXTENT;
  return {
    id: ZERO_UUID,
    name: 'Root Frame',
    type: 'frame',
    x: 0,
    y: 0,
    width: extent,
    height: extent,
    rotation: 0,
    selrect: {
      x: 0,
      y: 0,
      width: extent,
      height: extent,
      x1: 0,
      y1: 0,
      x2: extent,
      y2: extent,
    },
    points: [
      {
        x: 0,
        y: 0,
      },
      {
        x: extent,
        y: 0,
      },
      {
        x: extent,
        y: extent,
      },
      {
        x: 0,
        y: extent,
      },
    ],
    transform: { ...IDENTITY_TRANSFORM, },
    transformInverse: { ...IDENTITY_TRANSFORM, },
    parentId: ZERO_UUID,
    frameId: ZERO_UUID,
    flipX: null,
    flipY: null,
    proportion: 1,
    proportionLock: false,
    fills: [{
      fillColor: '#FFFFFF',
      fillOpacity: 1,
    },],
    strokes: [],
    pageId,
    hideFillOnExport: false,
    shapes: [],
  };
}
