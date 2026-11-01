import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { parseModelJson, } from '@monochromatic-dev/translation-repair-provider-stream/ts';

//region Coverage file
// Ledger T8: what the census reads out of one file `NODE_V8_COVERAGE` wrote,
// namely the scripts loaded from the build directory, each as V8 reported its
// functions and their counted ranges. Every other script in the file (Node's
// own, the test runner's) is left unread.

/**
 One counted range of a function, as V8 reports it.

 @example
 ```ts
 const range: CoverageRange = { startOffset: 0, endOffset: 40, count: 2, };
 ```
 */
export type CoverageRange = {
  /**
   Offset of the range's first character in the script.
   */
  readonly startOffset: number;

  /**
   Offset one past its last character.
   */
  readonly endOffset: number;

  /**
   How often the code under it ran in this process.
   */
  readonly count: number;
};

/**
 One function of a script, as V8 reports it: its whole extent first, then the
 blocks whose counts differ from their parent's.

 @example
 ```ts
 const napping: FunctionCoverage = { functionName: 'nap', ranges: [{ startOffset: 0, endOffset: 40, count: 1, },], };
 ```
 */
export type FunctionCoverage = {
  /**
   Name V8 gave the function, empty for a script's top level and for anonymous ones.
   */
  readonly functionName: string;

  /**
   Its ranges, the whole function first.
   */
  readonly ranges: readonly CoverageRange[];
};

/**
 One bundle's coverage from one process.

 @example
 ```ts
 const script: BundleScript = { bundle: 'index.mjs', functions: [], };
 ```
 */
export type BundleScript = {
  /**
   Bundle file name inside the build directory.
   */
  readonly bundle: string;

  /**
   Every function V8 reported for it.
   */
  readonly functions: readonly FunctionCoverage[];
};

/**
 A coverage file that does not read as V8 writes one.

 @example
 ```ts
 throw new CoverageFileError({ path: '/tmp/coverage-1.json', says: 'it has no result list', },);
 ```
 */
export class CoverageFileError extends Error {
  /**
   Declares this message safe to forward: it names a file this process
   listed, a bundle URL under the package's build directory and a shape this
   module describes.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the refusal naming the file and what did not read.

   @param path - coverage file read

   @param says - what in it did not read

   @example
   ```ts
   new CoverageFileError({ path: '/tmp/coverage-1.json', says: 'it has no result list', },);
   ```
   */
  constructor({
    path,
    says,
  }: {
    readonly path: string;
    readonly says: string;
  },) {
    super(`coverage file ${path} does not read as V8 writes one: ${says}`,);
    this.name = 'CoverageFileError';
  }
}

/**
 Narrows one parsed range.

 @param value - candidate from parsed JSON

 @returns Whether it carries three finite numbers

 @example
 ```ts
 isCoverageRange({ startOffset: 0, endOffset: 4, count: 1, },); // true
 ```
 */
function isCoverageRange(value: unknown,): value is CoverageRange {
  return isJsonRecord(value,)
    && Number.isFinite(value.startOffset,)
    && Number.isFinite(value.endOffset,)
    && Number.isFinite(value.count,);
}

/**
 Narrows one parsed function.

 @param value - candidate from parsed JSON

 @returns Whether it carries a name and at least its whole extent

 @example
 ```ts
 isFunctionCoverage({ functionName: 'nap', ranges: [], },); // false: no extent
 ```
 */
function isFunctionCoverage(value: unknown,): value is FunctionCoverage {
  if (!isJsonRecord(value,))
    return false;
  /**
   Its ranges as parsed.
   */
  const { ranges, } = value;
  return ((typeof value.functionName) === 'string')
    && isJsonArray(ranges,)
    && (ranges.length > 0)
    && ranges.every(isCoverageRange,);
}

/**
 Reads the bundle scripts out of one coverage file, leaving every other script
 (Node's own, the test runner's) unread.

 @param path - file read, named in the refusal

 @param text - its contents

 @param bundleUrlPrefix - `file://` URL of the build directory with a trailing slash

 @param isBundleName - whether a script under that prefix is a bundle, asked
 of its name from the prefix; every script under it is one where absent, which
 is right where the prefix is a build directory and no other file stands in it

 @returns Each bundle script the process loaded

 @throws CoverageFileError where the file is not JSON (a write cut short
 leaves one), or it or a bundle script in it does not read as V8 writes one;
 the refusal never repeats the parser's message, which quotes the text it
 refused

 @example
 ```ts
 const scripts = bundleScriptsOf({ path, text, bundleUrlPrefix: 'file:///pkg/dist/final/node/', },);
 ```
 */
export function bundleScriptsOf(
  {
    path,
    text,
    bundleUrlPrefix,
    isBundleName,
  }: {
    readonly path: string;
    readonly text: string;
    readonly bundleUrlPrefix: string;
    readonly isBundleName?: (bundle: string) => boolean;
  },
): readonly BundleScript[] {
  /**
   Parse attempt over the file, its failure taken as data.
   */
  const attempt = parseModelJson({ text, },);
  if (!attempt.parsed)
    throw new CoverageFileError({
      path,
      says: 'it is not JSON',
    },);
  /**
   The file as JSON.
   */
  const parsed = attempt.value;
  if (!isJsonRecord(parsed,))
    throw new CoverageFileError({
      path,
      says: 'it is not an object',
    },);
  /**
   Every script the process reported.
   */
  const { result, } = parsed;
  if (!isJsonArray(result,))
    throw new CoverageFileError({
      path,
      says: 'it has no result list',
    },);
  return result.flatMap(function bundleScript(script,): readonly BundleScript[] {
    if (!isJsonRecord(script,))
      throw new CoverageFileError({
        path,
        says: 'a script is not an object',
      },);
    /**
     Where the script was loaded from, and what V8 counted in it.
     */
    const {
      url,
      functions,
    } = script;
    if ((typeof url) !== 'string')
      throw new CoverageFileError({
        path,
        says: 'a script carries no url',
      },);
    if (!url.startsWith(bundleUrlPrefix,))
      return [];
    if ((isBundleName !== undefined) && (!isBundleName(url.slice(bundleUrlPrefix.length,),)))
      return [];
    if ((!isJsonArray(functions,)) || (!functions.every(isFunctionCoverage,)))
      throw new CoverageFileError({
        path,
        says: `${url} carries a function that is not a name with ranges`,
      },);
    return [{
      bundle: url.slice(bundleUrlPrefix.length,),
      functions,
    },];
  },);
}

//endregion Coverage file
