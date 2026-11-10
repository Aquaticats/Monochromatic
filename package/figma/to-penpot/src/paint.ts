/**
 Figma paint to Penpot fill/stroke conversion.
 
 Only solid paints convert today; gradient and image paints return the {@link SKIP}
 sentinel so callers drop them.
 
 @module figma-to-penpot-paint
 */

import {
  figmaColorToFill,
  figmaColorToHex,
} from './color.ts';
import {
  asString,
  type FigmaRecord,
  isRecord,
  numberOr,
  recordArray,
  SKIP,
} from './read.ts';
import type {
  PenpotFill,
  PenpotStroke,
} from './types.ts';

/**
 Resolve a Figma paint's type across the enum-style `type` and schema-style
 `__type` keys Figma emits.
 
 @param paint - Figma paint record
 
 @returns paint type string, empty when neither key is a string
 
 @example
 ```ts
 paintTypeOf({ type: 'PaintType.SOLID', }); // "PaintType.SOLID"
 ```
 */
function paintTypeOf(paint: FigmaRecord,): string {
  return asString(paint.type,)
    || asString(paint.__type,);
}

/**
 Test whether a paint type string denotes a solid paint.
 
 Figma spells paint types as schema enum strings, and every non-solid type
 (`PaintType.GRADIENT_LINEAR`, `PaintType.IMAGE`, ...) names its own kind, so a
 type containing `SOLID` is a solid paint. The match stays loose on purpose: a
 future solid variant converts without a code change, and the family prefix is
 not part of the test.
 
 @param paintType - normalised paint type string
 
 @returns whether the paint is solid
 
 @example
 ```ts
 isSolidPaint('PaintType.SOLID'); // true
 ```
 */
function isSolidPaint(paintType: string,): boolean {
  return paintType.includes('SOLID',);
}

/**
 Convert a Figma paint to a Penpot fill.
 
 @param paint - Figma paint record
 
 @returns Penpot fill, or {@link SKIP} for unsupported (non-solid, color-less) paints
 
 @example
 ```ts
 const fill = figmaPaintToFill(paint);
 if (fill !== SKIP) fills.push(fill);
 ```
 */
export function figmaPaintToFill(
  paint: FigmaRecord,
): PenpotFill | typeof SKIP {
  if (!isSolidPaint(paintTypeOf(paint,),))
    return SKIP;
  /**
   Solid paint's color struct; absence drops the paint.
   */
  const {color} = paint;
  if (!isRecord(color,))
    return SKIP;
  /**
   Base fill from the color; opacity may be overridden by the paint below.
   */
  const fill = figmaColorToFill(color,);
  if ((typeof paint.opacity) === 'number')
    fill.fillOpacity = paint.opacity;
  return fill;
}

/**
 Convert a Figma paint to a Penpot stroke.
 
 @param paint - Figma paint record
 
 @param strokeWeight - stroke width in px
 
 @param strokeAlign - Figma alignment enum (`INSIDE`/`OUTSIDE`/`CENTER`)
 
 @returns Penpot stroke, or {@link SKIP} for unsupported paints
 
 @example
 ```ts
 const stroke = figmaPaintToStroke({ paint, strokeWeight: 2, strokeAlign: 'CENTER', });
 if (stroke !== SKIP) strokes.push(stroke);
 ```
 */
export function figmaPaintToStroke(
  {
    paint,
    strokeWeight,
    strokeAlign,
  }: Readonly<{
    paint: FigmaRecord;
    strokeWeight: number;
    strokeAlign: string;
  }>,
): PenpotStroke | typeof SKIP {
  if (!isSolidPaint(paintTypeOf(paint,),))
    return SKIP;
  /**
   Solid paint's color struct; absence drops the stroke.
   */
  const {color} = paint;
  if (!isRecord(color,))
    return SKIP;
  return {
    strokeStyle: 'solid',
    strokeAlignment: strokeAlign === 'OUTSIDE'
      ? 'outer'
      : (strokeAlign === 'INSIDE'
        ? 'inner'
        : 'center'),
    strokeWidth: strokeWeight,
    strokeColor: figmaColorToHex(color,),
    strokeOpacity: numberOr({
      value: paint.opacity,
      fallback: 1,
    },),
  };
}

/**
 Map a Figma paint list through a converter, dropping paints with no Penpot
 counterpart.
 
 One owner for the iterate-convert-drop-{@link SKIP} shape that fills, strokes,
 and text fills all need; each caller supplies the converter that fits its
 Penpot target, so a change to that shape reaches every paint list at once.
 
 @typeParam Converted - Penpot fill or stroke the converter produces
 
 @param paints - raw paint list read off a NodeChange (`fillPaints`, `strokePaints`)
 
 @param convert - converter turning one paint into a Penpot value or {@link SKIP}
 
 @returns converted paints in list order, unsupported paints omitted
 
 @example
 ```ts
 const fills = collectPaints({ paints: nc.fillPaints, convert: figmaPaintToFill, });
 ```
 */
export function collectPaints<Converted>(
  {
    paints,
    convert,
  }: Readonly<{
    paints: unknown;
    convert: (paint: FigmaRecord,) => Converted | typeof SKIP;
  }>,
): Converted[] {
  /**
   Converted paints accumulated in list order.
   */
  const converted: Converted[] = [];
  for (const paint of recordArray(paints,)) {
    /**
     Penpot counterpart for this paint; {@link SKIP} means the paint type is unsupported.
     */
    const result = convert(paint,);
    if (result !== SKIP)
      converted.push(result,);
  }
  return converted;
}
