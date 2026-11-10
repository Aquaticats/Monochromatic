/**
 WOFF2 conversion utility for the Aquaticat font build.
 Uses fonttools via uv to convert OTF to WOFF2 format.
 
 @module
 */

import { stat, } from 'node:fs/promises';
import { resolve, } from 'node:path';

import spawn, { SubprocessError, } from 'nano-spawn';

/**
 Renders the diagnostic a failed conversion carries.
 
 @param error - value the conversion rejected with
 
 @returns the subprocess message and its stderr, the innermost cause message when
   nothing was captured, or the value's own text
 
 @example
 ```ts
 conversionFailureText({ error });
 ```
 */
function conversionFailureText({ error, }: { readonly error: unknown; },): string {
  if (!(error instanceof SubprocessError))
    return String(error,);
  if (error.stderr !== '')
    return `${error.message}\n${error.stderr}`;
  /**
   A child that never started has no captured output; its innermost cause names
   the failed executable and its code, which is what a reader can act on
   (doc/troubleshooting/nano-spawn-failure-shapes.md).
   */
  /**
   Innermost failure nano-spawn wrapped, when it recorded one.
   */
  const { cause, } = error;
  if (Error.isError(cause,))
    return cause.message;
  return error.message;
}

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
      `WOFF2 conversion failed for ${otfPath}: ${conversionFailureText({ error, },)}`,
      { cause: error, },
    );
  }
}
