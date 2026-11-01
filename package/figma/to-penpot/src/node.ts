/**
 Figma NodeChange to Penpot shape conversion, with child recursion.
 
 @module figma-to-penpot-node
 */

import { figmaColorToFill, } from './color.ts';
import {
  FIGMA_NODE_TYPE_MAP,
  IDENTITY_TRANSFORM,
} from './constants.ts';
import { parseParentIndex, } from './geometry.ts';
import {
  asString,
  type FigmaRecord,
  isRecord,
  numberOr,
  SKIP,
  stringOr,
} from './read.ts';
import {
  collectFills,
  collectStrokes,
  geometryOf,
  geometryPath,
} from './shape.ts';
import { convertTextContent, } from './text.ts';
import type {
  PenpotShape,
  PenpotShapeType,
  Uuid,
} from './types.ts';
import { nextUuid, } from './uuid.ts';

/**
 Shared lookup tables threaded unchanged through the conversion recursion.
 */
export type ConvertContext = {
  nodeByGuid: Map<string, Record<string, unknown>>;
  childrenByParent: Map<string, string[]>;
  guidToUuidMap: Map<string, Uuid>;
  shapes: Map<Uuid, PenpotShape>;
};

/**
 Resolve a node's Penpot parent UUID from its Figma parent index.
 
 @param nc - Figma NodeChange record
 
 @param parentUuid - fallback parent when no Figma parent resolves
 
 @param guidToUuidMap - GUID-key to Penpot-UUID index
 
 @returns resolved parent UUID
 
 @example
 ```ts
 const parentId = resolveParentUuid({ nc, parentUuid, guidToUuidMap, });
 ```
 */
function resolveParentUuid(
  {
    nc,
    parentUuid,
    guidToUuidMap,
  }: Readonly<{
    nc: FigmaRecord;
    parentUuid: Uuid;
    guidToUuidMap: ReadonlyMap<string, Uuid>;
  }>,
): Uuid {
  /**
   Parsed Figma parent reference; {@link SKIP} keeps the recursion's parent.
   */
  const parent = parseParentIndex(nc.parentIndex,);
  if (parent === SKIP)
    return parentUuid;
  /**
   Composite key matching the parent entry in the GUID map.
   */
  const parentKey = `${parent.parentGuid
    .sessionId}:${parent.parentGuid
      .localId}`;
  return guidToUuidMap.get(parentKey,)
    ?? parentUuid;
}

/**
 Stamp uniform border radius onto a shape when the node has a positive corner radius.
 
 @param shape - {@link PenpotShape} being assembled (mutated in place)
 
 @param nc - Figma NodeChange record
 
 @example
 ```ts
 applyCornerRadius({ shape, nc, });
 ```
 */
function applyCornerRadius(
  {
    shape,
    nc,
  }: {
    shape: PenpotShape;
    nc: Record<string, unknown>
  },
): void {
  /**
   Figma corner radius; only positive numbers produce Penpot radii.
   */
  const radius = nc.cornerRadius;
  if (((typeof radius) === 'number') && (radius > 0)) {
    shape.r1 = radius;
    shape.r2 = radius;
    shape.r3 = radius;
    shape.r4 = radius;
  }
}

/**
 Apply type-specific fields (container shapes, radius, path content, text) to a shape.
 
 @param shape - {@link PenpotShape} being assembled (mutated in place)
 
 @param penpotType - resolved Penpot shape type
 
 @param nc - Figma NodeChange record
 
 @example
 ```ts
 applyTypeSpecific({ shape, penpotType, nc, });
 ```
 */
function applyTypeSpecific(
  {
    shape,
    penpotType,
    nc,
  }: {
    shape: PenpotShape;
    penpotType: PenpotShapeType;
    nc: Record<string, unknown>;
  },
): void {
  if (penpotType === 'frame') {
    shape.hideFillOnExport = false;
    shape.showContent = true;
    shape.shapes = [];
    applyCornerRadius({
      shape,
      nc,
    },);
    /**
     Canvas background used as the frame fill when the node has no own fills.
     */
    const bgColor = nc.backgroundColor;
    if (isRecord(bgColor,) && (shape.fills
      .length
      === 0))
      shape.fills = [figmaColorToFill(bgColor,),];
  }
  if (penpotType === 'group')
    shape.shapes = [];
  if (penpotType === 'bool') {
    shape.shapes = [];
    shape.boolType = 'union';
    /**
     First fill-geometry path, used as the boolean shape's content.
     */
    const path = geometryPath(nc.fillGeometry,);
    if (path !== SKIP)
      shape.content = path;
  }
  if (penpotType === 'rect') {
    applyCornerRadius({
      shape,
      nc,
    },);
  }
  if (penpotType === 'path') {
    shape.growType = 'fixed';
    /**
     Preferred fill-geometry path; falls back to stroke geometry for open paths.
     */
    const fillPath = geometryPath(nc.fillGeometry,);
    if (fillPath !== SKIP) {
      shape.content = fillPath;
    }
    else {
      /**
       Stroke-only geometry path for lines and open vectors.
       */
      const strokePath = geometryPath(nc.strokeGeometry,);
      if (strokePath !== SKIP)
        shape.content = strokePath;
    }
  }
  if (penpotType === 'text') {
    shape.growType = 'auto-width';
    shape.content = convertTextContent(nc,);
  }
}

/**
 Convert a single Figma NodeChange to a Penpot shape and recurse into children.
 
 @param nodeKey - composite `"sessionID:localID"` key of the node
 
 @param parentUuid - parent shape UUID from the recursion
 
 @param frameUuid - enclosing frame UUID from the recursion
 
 @param pageId - page the shape belongs to
 
 @param ctx - shared {@link ConvertContext} lookup tables threaded through the recursion
 
 @returns the shape's Penpot UUID, or {@link SKIP} when the node has no equivalent
 
 @example
 ```ts
 const uuid = convertNode({ nodeKey, parentUuid, frameUuid, pageId, ctx, });
 ```
 */
export function convertNode(
  {
    nodeKey,
    parentUuid,
    frameUuid,
    pageId,
    ctx,
  }: {
    nodeKey: string;
    parentUuid: Uuid;
    frameUuid: Uuid;
    pageId: Uuid;
    ctx: ConvertContext;
  },
): Uuid | typeof SKIP {
  /**
   NodeChange for this key; absence means the GUID was never indexed.
   */
  const nc = ctx.nodeByGuid
    .get(nodeKey,);
  if (nc === undefined)
    return SKIP;

  /**
   Penpot shape type, or {@link SKIP} for Figma node types with no equivalent.
   */
  const penpotType = FIGMA_NODE_TYPE_MAP[asString(nc.type,)];
  if ((penpotType === undefined) || (penpotType === SKIP))
    return SKIP;

  /**
   Stable shape UUID, reusing the cross-pass GUID map when present.
   */
  const shapeUuid = ctx.guidToUuidMap
    .get(nodeKey,)
    ?? nextUuid();
  /**
   Effective geometry: position, size, selrect, and corner points.
   */
  const geom = geometryOf(nc,);
  /**
   Frame ancestor UUID: this shape when it is a frame, else the enclosing frame.
   */
  const frameId = penpotType === 'frame' ? shapeUuid : frameUuid;

  /**
   Penpot shape record; type-specific fields are layered on below.
   */
  const shape: PenpotShape = {
    id: shapeUuid,
    name: stringOr({
      value: nc.name,
      fallback: 'Unnamed',
    },),
    type: penpotType,
    x: geom.hasGeometry ? geom.x : null,
    y: geom.hasGeometry ? geom.y : null,
    width: geom.hasGeometry ? geom.width : null,
    height: geom.hasGeometry ? geom.height : null,
    rotation: 0,
    selrect: geom.selrect,
    points: geom.points,
    transform: { ...IDENTITY_TRANSFORM, },
    transformInverse: { ...IDENTITY_TRANSFORM, },
    parentId: resolveParentUuid({
      nc,
      parentUuid,
      guidToUuidMap: ctx.guidToUuidMap,
    },),
    frameId,
    flipX: null,
    flipY: null,
    proportion: 1,
    proportionLock: false,
    fills: collectFills(nc,),
    strokes: collectStrokes(nc,),
    pageId,
  };

  if (((typeof nc.opacity) === 'number') && (nc.opacity !== 1))
    shape.opacity = nc.opacity;
  if (nc.visible === false)
    shape.hidden = true;

  applyTypeSpecific({
    shape,
    penpotType,
    nc,
  },);

  /**
   Penpot UUIDs of converted children, used to populate container `shapes`.
   */
  const childUuids: Uuid[] = [];
  for (const childKey of ctx.childrenByParent
    .get(nodeKey,)
    ?? []) {
    /**
     Converted child UUID; {@link SKIP} children are dropped.
     */
    const childUuid = convertNode({
      nodeKey: childKey,
      parentUuid: shapeUuid,
      frameUuid: frameId,
      pageId,
      ctx,
    },);
    if (childUuid !== SKIP)
      childUuids.push(childUuid,);
  }
  if ((penpotType === 'frame') || (penpotType === 'group')
    || (penpotType === 'bool'))
    shape.shapes = childUuids;

  ctx.shapes
    .set(
    shapeUuid,
    shape,
    );
  return shapeUuid;
}
