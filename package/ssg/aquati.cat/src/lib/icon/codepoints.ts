/**
 Material Symbols Outlined icon name → Private Use Area codepoint mapping.
 
 Parsed from the upstream `codepoints` data file shipped alongside the
 variable woff2 in Google's material-design-icons repository. Keys are
 snake_case ligature names (`info`, `priority_high`, …) and values are
 single-codepoint strings in the Unicode Private Use Area (e.g. U+E88E
 for `info`).
 
 Rendering icons by PUA codepoint (instead of ligature) lets the
 subsetter retain only the specific icon glyphs actually in use. A
 ligature-based render would force HarfBuzz to close every icon whose
 name can be spelled from the letters present in source, which is
 essentially the whole font.
 
 @example
 ```ts
 ICON_CODEPOINTS['info'] === '\ue88e';
 ```
 */
import { readFile, } from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import { fileURLToPath, } from 'node:url';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

/**
 Absolute directory of this module, used to locate the sibling codepoints file.
 */
const HERE = import.meta.dirname;

/**
 Raw text contents of the upstream `codepoints` file.
 */
const raw = await readFile(
  join(
    HERE,
    'material-symbols-outlined.codepoints',
  ),
  'utf8',
);

/**
 Radix of hex codepoint values in the upstream `codepoints` file.
 */
const HEX_RADIX = 16;

/**
 Parses one `name codepoint` line into a `[name, char]` tuple.
 
 @param line - raw line from the codepoints file (e.g. `"info e88e"`)
 
 @returns tuple of icon name and single-codepoint string
 
 @throws when the line is malformed (missing the name or codepoint field)
 
 @example
 ```ts
 parseLine('info e88e'); // ['info', '\ue88e']
 ```
 */
function parseLine(line: string,): readonly [
  string,
  string,
] {
  /**
   Space-delimited fields; a well-formed line is `name codepoint`, so both fields are required.
   */
  const [name, hex,] = line.split(' ',);
  return [
    nonNullishOrThrow(name,),
    String.fromCodePoint(Number.parseInt(
      nonNullishOrThrow(hex,),
      HEX_RADIX,
    ),),
  ] as const;
}

/**
 Complete Material Symbols Outlined icon name → codepoint map.
 */
export const ICON_CODEPOINTS: Readonly<Record<string, string>> = Object
  .fromEntries(
  raw
    .trim()
    .split('\n',)
    .map(function parseEachLine(line,) {
      return parseLine(line,);
    },),
  );
