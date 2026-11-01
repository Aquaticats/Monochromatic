/**
 Build script that reads the master glyph strip SVG, extracts individual
 letter shapes, and assembles them into an OpenType font file using opentype.js.
 
 Run: `mise run //package/typeface/aquaticat:build:font`
 */

import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  resolve,
} from 'node:path';

import opentype, { type Glyph, } from 'opentype.js';

import {
  ASCENDER,
  CELL_UNICODE,
  DESCENDER,
  SIDE_BEARING,
  SPACE_ADVANCE,
  UNITS_PER_EM,
} from './build-font-metrics.ts';
import { addStrokedPath, } from './build-font-paths-stroked.ts';
import {
  addFilledPath,
  computeLocalXBounds,
} from './build-font-paths.ts';
import { convertToWoff2, } from './build-font-woff2.ts';
import {
  parseSvg,
  parseSvgPathD,
} from './parse-svg.ts';

//region Main build

/**
 Directory containing this build script.
 */
const scriptDir = dirname(new URL(import.meta.url,).pathname,);

/**
 Absolute path to the master glyph strip SVG.
 */
const svgPath = resolve(
  scriptDir,
  'glyphs.svg',
);

/**
 Output directory for generated font files.
 */
const distDir = resolve(
  scriptDir,
  '..',
  'dist',
);

console.log(
  'Reading glyph SVG:',
  svgPath,
);
/**
 Raw SVG file content.
 */
const svgContent = await readFile(
  svgPath,
  'utf8',
);
/**
 Parsed glyph cells from the SVG strip.
 */
const cells = parseSvg(svgContent,);
console.log(`Parsed ${cells.length} glyph cells`,);

/* oxlint-disable import/no-named-as-default-member -- opentype.js's UMD bundle defeats cjs-module-lexer's named-export detection under Node's CJS/ESM interop, so the default import's .Glyph/.Path/.Font members are the only ones that resolve at runtime; see doc/troubleshooting/opentype-js-cjs-esm-interop.md */
/**
 Required .notdef glyph (empty placeholder for missing characters).
 */
const notdefGlyph = new opentype.Glyph({
  name: '.notdef',
  unicode: 0,
  advanceWidth: SPACE_ADVANCE,
  path: new opentype.Path(),
},);
/* oxlint-enable import/no-named-as-default-member */

/* oxlint-disable import/no-named-as-default-member -- opentype.js's UMD bundle defeats cjs-module-lexer's named-export detection under Node's CJS/ESM interop, so the default import's .Glyph/.Path/.Font members are the only ones that resolve at runtime; see doc/troubleshooting/opentype-js-cjs-esm-interop.md */
/**
 Space character glyph (no visible path, just advance width).
 */
const spaceGlyph = new opentype.Glyph({
  name: 'space',
  unicode: 32,
  advanceWidth: SPACE_ADVANCE,
  path: new opentype.Path(),
},);
/* oxlint-enable import/no-named-as-default-member */

/**
 Assembled letter glyphs from the parsed SVG cells.
 */
const letterGlyphs = cells.flatMap(
  function buildGlyph(
    cell: Readonly<(typeof cells)[number]>,
    cellIndex,
  ): Glyph[] {
    /**
     Unicode code point assigned to this cell position, or undefined for unused slots.
     */
    const unicode = CELL_UNICODE[cellIndex];
    if (unicode === undefined)
      return [];

    /**
     Human-readable letter string used as the OpenType glyph name.
     */
    const letterName = String.fromCodePoint(unicode,);
    /**
     Local X bounds of this glyph's strokes, used to derive shift and advance width.
     */
    const {
      minX,
      maxX,
    } = computeLocalXBounds({
      paths: cell.paths,
      cellX: cell.xOffset,
    },);
    /**
     Horizontal shift that places the leftmost stroke exactly one side-bearing inside the glyph box.
     */
    const xShift = SIDE_BEARING - minX;
    /**
     Total advance width: stroke span plus a side-bearing on each side.
     */
    const advanceWidth = (maxX - minX) + (2 * SIDE_BEARING);

    /* oxlint-disable import/no-named-as-default-member -- opentype.js's UMD bundle defeats cjs-module-lexer's named-export detection under Node's CJS/ESM interop, so the default import's .Glyph/.Path/.Font members are the only ones that resolve at runtime; see doc/troubleshooting/opentype-js-cjs-esm-interop.md */
    /**
     OpenType path that collects every contour from this cell's SVG paths.
     */
    const path = new opentype.Path();
    /* oxlint-enable import/no-named-as-default-member */
    cell.paths
      .forEach(function addCellPath(cellPath,) {
      /**
       Tokenised commands for one SVG path, dispatched into stroked or filled tracing.
       */
        const commands = parseSvgPathD(cellPath.d,);
        if (cellPath.isStroked) {
          addStrokedPath({
            otPath: path,
            commands,
            strokeWidth: cellPath.strokeWidth,
            cellX: cell.xOffset,
            xShift,
          },);
        }
        else {
          addFilledPath({
            otPath: path,
            commands,
            cellX: cell.xOffset,
            xShift,
          },);
        }
      },);

    console.log(
      `  ${letterName}: advance=${Math.round(advanceWidth,)}, paths=${cell.paths
        .length}`,
    );

    // oxlint-disable-next-line import/no-named-as-default-member -- opentype.js's UMD bundle defeats cjs-module-lexer's named-export detection under Node's CJS/ESM interop, so the default import's .Glyph/.Path/.Font members are the only ones that resolve at runtime; see doc/troubleshooting/opentype-js-cjs-esm-interop.md
    return [new opentype.Glyph({
      name: letterName,
      unicode,
      advanceWidth,
      path,
    },),];
  },
);

/* oxlint-disable import/no-named-as-default-member -- opentype.js's UMD bundle defeats cjs-module-lexer's named-export detection under Node's CJS/ESM interop, so the default import's .Glyph/.Path/.Font members are the only ones that resolve at runtime; see doc/troubleshooting/opentype-js-cjs-esm-interop.md */
/**
 Assembled OpenType font with all glyphs.
 */
const font = new opentype.Font({
  familyName: 'Aquaticat',
  styleName: 'Regular',
  unitsPerEm: UNITS_PER_EM,
  ascender: ASCENDER,
  descender: DESCENDER,
  glyphs: [
    notdefGlyph,
    spaceGlyph,
    ...letterGlyphs,
  ],
},);
/* oxlint-enable import/no-named-as-default-member */

await mkdir(
  distDir,
  { recursive: true, },
);

/**
 Output path for the OTF font file.
 */
const otfPath = resolve(
  distDir,
  'Aquaticat-Regular.otf',
);
/**
 Raw OTF binary data.
 */
const buffer = font.toArrayBuffer();
await writeFile(
  otfPath,
  Buffer.from(buffer,),
);
console.log(`Wrote ${otfPath} (${buffer.byteLength} bytes)`,);

await convertToWoff2({
  otfPath,
  distDir,
},);

//endregion Main build
