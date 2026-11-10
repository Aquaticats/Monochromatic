/**
 Figma text node to Penpot text-content tree conversion.
 
 @module figma-to-penpot-text
 */

import { DEFAULT_FONT_SIZE, } from './constants.ts';
import {
  collectPaints,
  figmaPaintToFill,
} from './paint.ts';
import {
  type FigmaRecord,
  numberOr,
  stringOr,
} from './read.ts';
import type { PenpotFill, } from './types.ts';

/**
 Default font family when the Figma node names none.
 */
const DEFAULT_FONT_FAMILY = 'Source Sans 3';

/**
 Default font weight string when the Figma node carries none.
 */
const DEFAULT_FONT_WEIGHT = '400';

/**
 Penpot bundled font id paired with the default family.
 */
const DEFAULT_FONT_ID = 'sourcesanspro';

/**
 Collect Penpot text fills from a Figma text node, defaulting to opaque black.
 
 @param nc - Figma text NodeChange record
 
 @returns at least one Penpot fill
 
 @example
 ```ts
 const fills = textFills(nc);
 ```
 */
function textFills(nc: FigmaRecord,): PenpotFill[] {
  /**
   Fills converted from the node's paints, dropping unsupported paints.
   */
  const fills = collectPaints({
    paints: nc.fillPaints,
    convert: figmaPaintToFill,
  },);
  if (fills.length === 0) {
    fills.push({
      fillColor: '#000000',
      fillOpacity: 1,
    },);
  }
  return fills;
}

/**
 Convert a Figma text node to a Penpot text-content tree.
 
 @param nc - Figma text NodeChange record
 
 @returns Penpot rich-text root node
 
 @example
 ```ts
 shape.content = convertTextContent(nc);
 ```
 */
export function convertTextContent(
  nc: FigmaRecord,
): Record<string, unknown> {
  /**
   Numeric font size with a default for nodes whose Figma data is incomplete.
   */
  const fontSize = numberOr({
    value: nc.fontSize,
    fallback: DEFAULT_FONT_SIZE,
  },);
  /**
   Font family string with a sans-serif default Penpot can resolve.
   */
  const fontFamily = stringOr({
    value: nc.fontName,
    fallback: DEFAULT_FONT_FAMILY,
  },);
  /**
   Font weight stringified to match Penpot's text-attribute shape.
   */
  const fontWeight = (typeof nc.fontWeight) === 'number'
    ? String(nc.fontWeight,)
    : DEFAULT_FONT_WEIGHT;
  /**
   Raw character payload, empty when absent so output stays well-formed.
   */
  const textContent = stringOr({
    value: nc.characters,
    fallback: '',
  },);

  /**
   Paragraph-level attribute block shared across the single text run.
   */
  const paragraphAttrs = {
    lineHeight: '1.2',
    fontStyle: 'normal',
    textTransform: 'none',
    textAlign: 'left',
    fontId: DEFAULT_FONT_ID,
    fontSize: String(fontSize,),
    fontWeight,
    textDirection: 'ltr',
    type: 'paragraph',
    fontVariantId: fontWeight,
    textDecoration: 'none',
    letterSpacing: '0',
    fills: textFills(nc,),
    fontFamily,
  };

  return {
    type: 'root',
    children: [{
      type: 'paragraph-set',
      children: [{
        ...paragraphAttrs,
        children: [{
          text: textContent,
        },],
      },],
    },],
  };
}
