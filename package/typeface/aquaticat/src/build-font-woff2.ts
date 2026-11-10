// oxlint-disable typescript/no-unsafe-type-assertion -- nano-spawn error type is untyped
/**
 WOFF2 conversion utility for the Aquaticat font build.
 Uses fonttools via uv to convert OTF to WOFF2 format.
 
 @module
 */

import { stat, } from 'node:fs/promises';
import { resolve, } from 'node:path';

import spawn from 'nano-spawn';

/**
 Converts an OTF font file to WOFF2 format using Python fonttools.
 
 @param otfPath - absolute path to source OTF file
 
 @param distDir - output directory for the WOFF2 file
 
 @throws When the conversion fails, carrying the caught value as its cause
 
 @example
 ```ts
 await convertToWoff2({
   otfPath: '/dist/Aquaticat-Regular.otf',
   distDir: '/dist',
 });
 ```
 */
export async function convertToWoff2({
  otfPath,
  distDir,
}: {
  readonly otfPath: string;
  readonly distDir: string;
},): Promise<void> {
  console.log('Converting to WOFF2 via fonttools...',);
  /**
   Output path for the WOFF2 font file.
   */
  const woff2Path = resolve(
    distDir,
    'Aquaticat-Regular.woff2',
  );
  /**
   Python one-liner for fonttools WOFF2 conversion.
   */
  const woff2Script =
    `from fontTools.ttLib import TTFont; f = TTFont("${otfPath}"); f.flavor = "woff2"; f.save("${woff2Path}")`;
  try {
    await spawn(
      'uv',
      [
        'run',
        '--with',
        'fonttools',
        '--with',
        'brotli',
        'python3',
        '-c',
        woff2Script,
      ],
    );
    /**
     File stats for the generated WOFF2 file.
     */
    const { size, } = await stat(woff2Path,);
    console.log(`Wrote ${woff2Path} (${size} bytes)`,);
  }
  catch (error: unknown) {
    throw new Error(
      `WOFF2 conversion failed for ${otfPath}: ${(error as { stderr: string; }).stderr}`,
      { cause: error, },
    );
  }
}
