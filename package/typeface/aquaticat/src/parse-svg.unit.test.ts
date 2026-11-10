/**
 The master glyph strip parser reads one attribute by name and one glyph per cell.

 Attribute lookup locates a value by the attribute's name boundary. A raw
 substring search for `name=<quote>` also matches an attribute whose name merely
 ends with the sought one, and a `name=<quote>` sequence inside another
 attribute's value, so a path carrying `id` or `data-d` returned the wrong `d`
 and silently distorted glyph geometry.
 */
/* oxlint-disable test-import/require-eventual-artifact -- this package ships src/ (package.json `files`) and builds font binaries rather than a JS artifact, so the modules beside this test are the shipped ones; see doc/planning/oxlint-test-import-eventual-artifact.md, section "Packages that ship source". */
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseSvg, } from './parse-svg.ts';

//region Fixtures

/**
 Wraps element markup in the root element the parser ignores.
 
 @param elements - Self-closing `<rect>` and `<path>` markup
 
 @returns SVG document text
 */
function svgDocument({ elements, }: { readonly elements: string; },): string {
  return `<svg width='12860' height='1000'>${elements}</svg>`;
}

/**
 One cell rect at the given strip offset.
 
 @param xOffset - Horizontal offset of the cell, or an empty string for the first cell
 
 @returns Self-closing `<rect>` markup
 */
function cellRect({ xOffset, }: { readonly xOffset: string; },): string {
  const transform = xOffset === ''
    ? ''
    : ` transform='translate(${xOffset})'`;
  return `<rect width='605' height='1000'${transform} fill='white' />`;
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: parseSvg.name,
      children: [
        //region Attribute lookup

        it({
          name: 'reads the path data of a path whose attributes come before d',
          fn: async () => {
            /**
             Path data the element declares.
             */
            const d = 'M0 0 L10 0 L10 10Z';
            /**
             Parsed cells for one element declaring id before d.
             */
            const [cell,] = parseSvg(svgDocument({
              elements: `${cellRect({ xOffset: '', },)}<path id='glyph-a' d='${d}' fill='black' />`,
            },),);

            expect(cell?.paths[0]?.d,).toBe(d,);
          },
        },),

        it({
          name: 'ignores an attribute whose name ends with the sought name',
          fn: async () => {
            /**
             Path data the element declares after the decoy attribute.
             */
            const d = 'M0 0 L20 0 L20 20Z';
            /**
             Parsed cells for one element declaring data-d before d.
             */
            const [cell,] = parseSvg(svgDocument({
              elements: `${cellRect({ xOffset: '', },)}<path data-d='M99 99' d='${d}' fill='black' />`,
            },),);

            expect(cell?.paths[0]?.d,).toBe(d,);
          },
        },),

        it({
          name: 'ignores an attribute-shaped sequence inside another attribute value',
          fn: async () => {
            /**
             Path data the element declares after the decoy value.
             */
            const d = 'M0 0 L30 0 L30 30Z';
            /**
             Parsed cells for one element whose title mentions a d attribute.
             */
            const [cell,] = parseSvg(svgDocument({
              elements: `${cellRect({ xOffset: '', },)}<path title="copied from d='M99 99'" d='${d}' fill='black' />`,
            },),);

            expect(cell?.paths[0]?.d,).toBe(d,);
          },
        },),

        it({
          name: 'reads a double-quoted attribute value',
          fn: async () => {
            /**
             Path data the element declares in double quotes.
             */
            const d = 'M0 0 L40 0 L40 40Z';
            /**
             Parsed cells for one element declaring d in double quotes.
             */
            const [cell,] = parseSvg(svgDocument({
              elements: `${cellRect({ xOffset: '', },)}<path d="${d}" fill="black" />`,
            },),);

            expect(cell?.paths[0]?.d,).toBe(d,);
          },
        },),

        it({
          name: 'skips a path that declares no d attribute',
          fn: async () => {
            /**
             Parsed cells for one cell holding a d-less path.
             */
            const [cell,] = parseSvg(svgDocument({
              elements: `${cellRect({ xOffset: '', },)}<path fill='black' />`,
            },),);

            expect(cell?.paths,).toStrictEqual([],);
          },
        },),

        //endregion Attribute lookup

        //region Cells

        it({
          name: 'reads the strip offset from the cell transform',
          fn: async () => {
            /**
             Strip offset the rect declares.
             */
            const xOffset = 645;
            /**
             Parsed cells for a strip holding one cell at that offset.
             */
            const [cell,] = parseSvg(svgDocument({
              elements: cellRect({ xOffset: String(xOffset,), },),
            },),);

            expect(cell?.xOffset,).toBe(xOffset,);
          },
        },),

        it({
          name: 'gives the first cell offset zero when its rect carries no transform',
          fn: async () => {
            /**
             Parsed cells for a strip whose only rect has no transform.
             */
            const [cell,] = parseSvg(svgDocument({
              elements: cellRect({ xOffset: '', },),
            },),);

            expect(cell?.xOffset,).toBe(0,);
          },
        },),

        //endregion Cells

        //region Stroke detection

        it({
          name: 'marks a path stroked when it has a stroke and no fill',
          fn: async () => {
            /**
             Stroke width the element declares.
             */
            const strokeWidth = 80;
            /**
             Parsed cells for one stroked hexagon.
             */
            const [cell,] = parseSvg(svgDocument({
              elements: `${cellRect({ xOffset: '', },)}<path d='M0 0 L10 0Z' stroke='black' stroke-width='${strokeWidth}' />`,
            },),);

            expect(cell?.paths[0]?.isStroked,).toBe(true,);
            expect(cell?.paths[0]?.strokeWidth,).toBe(strokeWidth,);
          },
        },),

        it({
          name: 'marks a path filled when it declares both fill and stroke',
          fn: async () => {
            /**
             Parsed cells for one filled path that also declares a stroke.
             */
            const [cell,] = parseSvg(svgDocument({
              elements: `${cellRect({ xOffset: '', },)}<path d='M0 0 L10 0Z' fill='black' stroke='black' stroke-width='80' />`,
            },),);

            expect(cell?.paths[0]?.isStroked,).toBe(false,);
            expect(cell?.paths[0]?.strokeWidth,).toBe(0,);
          },
        },),

        //endregion Stroke detection
      ],
    },),
  ],
},);
