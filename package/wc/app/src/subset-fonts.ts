/**
 Source-driven font subsetting stage (format task, not in build chain).
 
 Reads the full upstream Inter variable woff2 from `fonts-source/`,
 collects the charset from every TypeScript source file under `src/`
 plus a printable-ASCII safety floor and {@link EXTRA_CODEPOINTS},
 subsets the font, and writes the result to `public/inter.woff2` where
 `src/build.ts` inlines it into the final HTML as a data URI.
 
 Not part of `build`. Run via `mise run format:fonts` whenever:
 
 - page text gains characters outside printable ASCII (the charset
   scan picks up literal characters, not escape sequences)
 - the upstream font in `fonts-source/` is refreshed
 
 The subsetted `public/inter.woff2` is committed alongside source and
 is the canonical artifact the page ships.
 
 Variable font axes are preserved (no `variationAxes` option) so CSS
 `font-weight: 100 900` declarations keep working.
 
 Follows the pipeline in
 `package/ssg/aquati.cat/src/build/subset-fonts.ts`, minus the icon
 font handling wc does not need.
 */
import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';

import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  init as initSubset,
  subset,
} from 'hb-subset-wasm';
import readdir from 'tiny-readdir-glob';
import wawoff2 from 'wawoff2';
import {
  encode as encodeWoff2,
  init as initWoff2,
} from 'woff2-encode-wasm';

 // module boundary marker

/**
 Tagged logger for the subset-fonts pipeline.
 */
const l = tagged({
  tag: 'subset-fonts',
  l: logger,
},);

//region Wasm initialization

/**
 Loads both wasm modules in parallel.
 
 `hb-subset-wasm` performs SFNT subsetting; `woff2-encode-wasm` re-encodes
 the subsetted SFNT back to WOFF2. WOFF2 to SFNT decoding is delegated to
 `wawoff2` (no init needed; it lazy-loads its own wasm on first call).
 */
await Promise.all([
  initSubset(
    await readFile(
      fileURLToPath(import.meta.resolve('hb-subset-wasm/hb-subset.wasm',),),
    ),
  ),
  initWoff2(
    await readFile(
      fileURLToPath(import.meta.resolve('woff2-encode-wasm/encoder.wasm',),),
    ),
  ),
],);

//endregion

/**
 Absolute path to this package's root directory, so the script works
 regardless of the caller's working directory.
 */
const PACKAGE_DIR: string = new URL(
  '..',
  import.meta.url,
)
  .pathname;

/**
 Source files scanned for every charset pass, resolved against
 {@link PACKAGE_DIR} via the `cwd` option so the script works regardless
 of the caller's working directory.
 */
const SOURCE_GLOB = 'src/**/*.ts';

/**
 Path to the full upstream Inter variable woff2 (committed, never shipped).
 */
const INPUT_FONT_PATH = join(
  PACKAGE_DIR,
  'fonts-source',
  'inter.woff2',
);

/**
 Path the subsetted woff2 is written to; `src/build.ts` inlines this file.
 */
const OUTPUT_FONT_PATH = join(
  PACKAGE_DIR,
  'public',
  'inter.woff2',
);

/**
 Lowest printable ASCII code point (space).
 */
const ASCII_PRINTABLE_MIN = 0x20;

/**
 Highest printable ASCII code point (tilde).
 */
const ASCII_PRINTABLE_MAX = 0x7E;

/**
 Code points the page renders at runtime that never appear literally in
 source (the charset scan collects literal characters, not `\u` escapes).
 
 - U+2007 FIGURE SPACE: pads frequency counts and percentages so columns
   of tabular numerals align without any column-width CSS.
 */
const EXTRA_CODEPOINTS = ' ';

//region Charset extraction

/**
 Builds the charset used to subset the body font.
 
 Seeds the set with printable ASCII so runtime-rendered strings that do
 not appear verbatim in any source file still render, plus
 {@link EXTRA_CODEPOINTS}. Then adds every code point found in every
 source file under {@link SOURCE_GLOB}: TypeScript syntax noise is all
 ASCII and therefore free under the seed; non-ASCII code points come
 exclusively from string literals, which is exactly what must render.
 
 @param sourceFiles - pre-scanned list of source file paths
 
 @returns single string with one instance of every code point to retain
 
 @example
 ```ts
 const text = await collectCharset({ sourceFiles });
 ```
 */
async function collectCharset(
  { sourceFiles, }: { readonly sourceFiles: readonly string[]; },
): Promise<string> {
  /**
   Deduplicated character set returned as the final charset.
   */
  const chars = new Set<string>();

  for (let cp = ASCII_PRINTABLE_MIN; cp <= ASCII_PRINTABLE_MAX; cp += 1)
    chars.add(String.fromCodePoint(cp,),);

  for (const ch of EXTRA_CODEPOINTS)
    chars.add(ch,);

  await Promise.all(sourceFiles.map(async function scanFile(filePath,) {
    /**
     Source file contents inspected character-by-character to populate
     `chars`.
     */
    const text = await readFile(
      filePath,
      'utf8',
    );
    for (const ch of text)
      chars.add(ch,);
  },),);

  return [...chars,].join('',);
}

//endregion

//region Font subsetting

/**
 Subsets the Inter woff2: reads {@link INPUT_FONT_PATH}, writes subsetted
 bytes to {@link OUTPUT_FONT_PATH}.
 
 @param text - charset string to retain
 
 @throws when the upstream font is missing from `fonts-source/`
 
 @example
 ```ts
 await subsetInter({ text });
 ```
 */
async function subsetInter(
  { text, }: { readonly text: string; },
): Promise<void> {
  /**
   Buffer loaded from disk; ENOENT is rethrown with an actionable hint
   pointing at the source dir.
   */
  const input: Buffer = await (async function readFontInput(): Promise<Buffer> {
    try {
      return await readFile(INPUT_FONT_PATH,);
    }
    catch (error) {
      if ((Error.isError(error,)) && ('code' in error)
        && (error.code
          === 'ENOENT')) {
        throw new Error(
          `${INPUT_FONT_PATH} not found. Place the full upstream woff2 in fonts-source/ before running format:fonts.`,
          { cause: error, },
        );
      }
      throw error;
    }
  })();

  /**
   Original byte length captured before subsetting for the savings log
   line.
   */
  const before = input.byteLength;

  /**
   WOFF2 to SFNT decode; `hb_face_create` needs raw SFNT bytes.
   */
  const sfntInput = await wawoff2.decompress(input,);

  /**
   SFNT subset. `layoutFeatures: '*'` retains every GSUB/GPOS layout
   feature, so `font-variant-numeric: tabular-nums` (the `tnum` feature)
   keeps working in the subset. STAT is dropped outright:
   hb-subset-wasm compiles HarfBuzz with `HB_NO_STYLE`, which removes
   the subset planner's STAT name-ID closure while STAT itself still
   passes through, so the table's axis and axis-value entries end up
   pointing at pruned name records and Firefox's font sanitizer flags
   them as console errors ("STAT: Invalid nameID") before discarding
   the table anyway. STAT is style-mapping metadata no browser renders
   from (fvar/gvar carry the variable axes), so shipping without it
   changes nothing but the noise. Full trace and alternatives:
   doc/troubleshooting/hb-subset-stat-dangling-nameids.md.
   */
  const sfntSubset = await subset(
    sfntInput,
    {
      text,
      layoutFeatures: '*',
      dropTables: ['STAT',],
    },
  );

  /**
   SFNT to WOFF2 re-encode via Google's official woff2 + brotli wasm
   build.
   */
  const output = await encodeWoff2(sfntSubset,);
  /**
   Final byte length captured after subsetting for the savings log line.
   */
  const after = output.byteLength;

  await writeFile(
    OUTPUT_FONT_PATH,
    output,
  );

  /**
   Compression ratio shown to operators verifying the subset actually
   shrinks the font.
   */
  const savedPercent = Math.round((1 - (after / before)) * 100,);
  l.info(
    `inter.woff2: ${before} → ${after} bytes (−${savedPercent}%)`,
  );
}

//endregion

//region Main pipeline

l.info('starting',);

/**
 Result of scanning the source glob (file list, directory list, etc.).
 */
const scan = await readdir(
  SOURCE_GLOB,
  { cwd: PACKAGE_DIR, },
);
/**
 Source file paths discovered by the scan, used as input for charset
 extraction.
 */
const sourceFiles = scan.files;
l.info(`scanning ${sourceFiles.length} source files`,);

/**
 Charset computed once before the woff2 subset call.
 */
const charset = await collectCharset({ sourceFiles, },);

await subsetInter({ text: charset, },);

l.info('done',);

//endregion
