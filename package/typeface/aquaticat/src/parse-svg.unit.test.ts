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
import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  parseSvg,
  parseSvgPathD,
} from './parse-svg.ts';

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

        //region Transforms

        it({
          name: 'reads a fractional strip offset from the transform',
          fn: async () => {
            /**
             Parsed cells for one cell whose rect translates by a fraction of a unit.
             */
            const [cell,] = parseSvg(svgDocument({
              elements: `<rect width='605' height='1000' transform='translate(645.5)' fill='white' />`,
            },),);

            expect(cell?.xOffset,).toBe(645.5,);
          },
        },),

        it({
          name: 'reads the offset of a canonical transform that follows a malformed one',
          fn: async () => {
            /**
             Strip offset the canonical transform declares, after one the parser must skip.
             */
            const xOffset = 12;
            /**
             Parsed cells for one cell whose transform opens with an unparsable candidate.
             */
            const [cell,] = parseSvg(svgDocument({
              elements: `<rect width='605' height='1000' transform='translate(x) translate(${xOffset})' fill='white' />`,
            },),);

            expect(cell?.xOffset,).toBe(xOffset,);
          },
        },),

        it({
          name: 'falls back to offset zero without a canonical translate',
          fn: async () => {
            /**
             Parsed cells for one cell whose transform holds a second point rather than a closing parenthesis.
             */
            const [cell,] = parseSvg(svgDocument({
              elements: `<rect width='605' height='1000' transform='translate(645.5.5)' fill='white' />`,
            },),);

            expect(cell?.xOffset,).toBe(0,);
          },
        },),

        it({
          name: 'reads a leading point as no canonical transform',
          fn: async () => {
            /**
             Parsed cells for one cell whose transform writes the fraction without an integer digit.
             */
            const [cell,] = parseSvg(svgDocument({
              elements: `<rect width='605' height='1000' transform='translate(.5)' fill='white' />`,
            },),);

            expect(cell?.xOffset,).toBe(0,);
          },
        },),

        //endregion Transforms

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

        //region Element boundaries

        it({
          name: 'needs whitespace between the element name and its attributes',
          fn: async () => {
            /**
             Parsed cells for a rect whose name runs straight into its attributes, beside one real cell.
             */
            const cells = parseSvg(svgDocument({
              elements: `<rectwidth='605' height='1000' fill='white' />${cellRect({ xOffset: '', },)}`,
            },),);

            expect(cells.length,).toBe(1,);
          },
        },),

        it({
          name: 'reads no cell from an element whose attribute value holds a closing angle bracket',
          fn: async () => {
            /**
             Parsed cells for one rect whose title holds the `>` the terminator search must not cross.
             */
            const cells = parseSvg(svgDocument({
              elements: `<rect title='a>b' width='605' height='1000' fill='white' />`,
            },),);

            expect(cells,).toStrictEqual([],);
          },
        },),

        it({
          name: 'reads a cell whose attributes start on the following line',
          fn: async () => {
            /**
             Parsed cells for a rect the export wrapped across three lines.
             */
            const cells = parseSvg(svgDocument({
              elements: `<rect\n  width='605' height='1000'\n  fill='white' />`,
            },),);

            expect(cells.length,).toBe(1,);
          },
        },),

        it({
          name: 'reads no cell from an export truncated before any element terminates',
          fn: async () => {
            /**
             Opener repeated the way a truncated Figma export ends, with no `>` anywhere after it.
             */
            const truncated = '<rect '.repeat(2_000,);

            expect(parseSvg(truncated,),).toStrictEqual([],);
          },
        },),

        //endregion Element boundaries

        //region Unexpected input

        it({
          name: 'throws when a path arrives before the first cell rect',
          fn: async () => {
            expect(function callParse(): void {
              parseSvg(svgDocument({
                elements: `<path d='M0 0 L10 0' fill='black' />${cellRect({ xOffset: '', },)}`,
              },),);
            },).toThrow('before the first cell rect',);
          },
        },),

        //endregion Unexpected input
      ],
    },),

    describe({
      name: parseSvgPathD.name,
      children: [
        it({
          name: 'tokenizes move, line, and close commands in path order',
          fn: async () => {
            expect(parseSvgPathD('M0 0 L10 10 Z',),).toStrictEqual([
              {
                type: 'M',
                x: 0,
                y: 0,
              },
              {
                type: 'L',
                x: 10,
                y: 10,
              },
              { type: 'Z', },
            ],);
          },
        },),

        it({
          name: 'keeps horizontal and vertical commands as single-axis commands',
          fn: async () => {
            expect(parseSvgPathD('M10 20 H30 V40',),).toStrictEqual([
              {
                type: 'M',
                x: 10,
                y: 20,
              },
              {
                type: 'H',
                x: 30,
              },
              {
                type: 'V',
                y: 40,
              },
            ],);
          },
        },),

        it({
          name: 'tokenizes commands written without separating spaces',
          fn: async () => {
            expect(parseSvgPathD('M0 0L10 0Z',),).toStrictEqual([
              {
                type: 'M',
                x: 0,
                y: 0,
              },
              {
                type: 'L',
                x: 10,
                y: 0,
              },
              { type: 'Z', },
            ],);
          },
        },),

        it({
          name: 'tokenizes negative and fractional coordinates',
          fn: async () => {
            expect(parseSvgPathD('M-5 2.5 L-10 -20.25',),).toStrictEqual([
              {
                type: 'M',
                x: -5,
                y: 2.5,
              },
              {
                type: 'L',
                x: -10,
                y: -20.25,
              },
            ],);
          },
        },),

        it({
          name: 'throws when the path data ends after an X coordinate',
          fn: async () => {
            expect(function callTokenize(): void {
              parseSvgPathD('M10',);
            },).toThrow('Y coordinate',);
          },
        },),

        it({
          name: 'throws when a command letter sits where a Y coordinate belongs',
          fn: async () => {
            expect(function callTokenize(): void {
              parseSvgPathD('M10 Z',);
            },).toThrow('Y coordinate',);
          },
        },),
      ],
    },),

    describe({
      name: 'tokenizer loops',
      children: [
        it({
          name: 'drive no loop with RegExp.exec',
          fn: async () => {
            /**
             Source of the parser module holding both tokenizers.
             */
            const source = await readFile(
              join(
                import.meta.dirname,
                'parse-svg.ts',
              ),
              'utf8',
            );
            /**
             Number of `RegExp.prototype.exec` calls the module makes.
             
             Each tokenizer needs one call per token when it drives the loop
             itself, so any count above the one-shot translate lookup means a
             loop is reading its tokens from shared regex state.
             */
            const execCalls = source.split('.exec(',).length - 1;

            expect(execCalls,).toBeLessThan(2,);
          },
        },),
      ],
    },),
  ],
},);
