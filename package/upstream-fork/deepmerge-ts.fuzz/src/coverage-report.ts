/**
 Coverage-reachability gate over the installed deepmerge-ts `dist` file.

 Reads a `NODE_V8_COVERAGE` directory produced by the `fuzz:coverage` task,
 projects it onto the target file's lines through `./coverage-v8.ts`, then
 either freezes the baseline (`write`) or fails when fewer lines are covered
 than the committed baseline (`check`). The baseline records the target's
 version; a different installed version fails the check with a refreeze
 instruction instead of comparing unrelated files.

 Run as a script by the task:

 ```sh
 node src/coverage-report.ts <check|write> <coverageDir> <baselinePath>
 ```

 @module
 */

import {
  readdir,
  readFile,
  realpath,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import {
  fileURLToPath,
  pathToFileURL,
} from 'node:url';

import {
  codeLineCount,
  coveredLines,
  isCoverageFile,
  uncalledFunctions,
  type V8Script,
} from './coverage-v8.ts';

/**
 Committed baseline shape.
 */
export type Baseline = {
  readonly version: string;
  readonly coveredLines: number;
  readonly codeLines: number;
};

/**
 Error failing the gate, carrying the human-readable reason.
 */
export class CoverageGateError extends Error {
  /**
   @param message - Why the gate failed and how to proceed.
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'CoverageGateError';
  }
}

/**
 Percentage scale for the summary line.
 */
const PERCENT_SCALE = 100;

/**
 Whether parsed JSON has the baseline shape.

 @param value - Parsed baseline file.

 @returns Whether every baseline field is present with the right type.

 @example
 ```ts
 isBaseline({ version: '8.0.2', coveredLines: 1, codeLines: 2, }); // true
 ```
 */
function isBaseline(value: unknown,): value is Baseline {
  return ((typeof value) === 'object')
    && (value !== null)
    && ((typeof Reflect.get(
      value,
      'version',
    )) === 'string')
    && Number.isInteger(Reflect.get(
      value,
      'coveredLines',
    ),)
    && Number.isInteger(Reflect.get(
      value,
      'codeLines',
    ),);
}

/**
 Locate the installed target file and its version as V8 reports it.

 @returns Real file path (symlinks resolved, as the ESM loader loads it) and
   the package version.

 @example
 ```ts
 const { path, version } = await installedTarget();
 ```
 */
async function installedTarget(): Promise<{
  readonly path: string;
  readonly version: string;
}> {
  /**
   Real path of the entry `deepmerge-ts` resolves to.
   */
  const path = await realpath(
    fileURLToPath(import.meta.resolve('deepmerge-ts',),),
  );
  /**
   Package manifest next to `dist/`.
   */
  const manifest: unknown = JSON.parse(await readFile(
    join(
      dirname(path,),
      '..',
      'package.json',
    ),
    'utf8',
  ),);
  if (((typeof manifest) !== 'object') || (manifest === null))
    throw new CoverageGateError(`deepmerge-ts manifest beside ${path} is not an object`,);
  /**
   Version field of the manifest.
   */
  const version: unknown = Reflect.get(
    manifest,
    'version',
  );
  if ((typeof version) !== 'string')
    throw new CoverageGateError(`deepmerge-ts manifest beside ${path} has no version`,);
  return {
    path,
    version,
  };
}

/**
 Collect every process's coverage entry for the target.

 @param coverageDir - `NODE_V8_COVERAGE` output directory.
 
 @param targetUrl - File URL of the target as V8 records it.

 @returns One script entry per process that loaded the target.

 @example
 ```ts
 const scripts = await targetScripts({ coverageDir, targetUrl, });
 ```
 */
export async function targetScripts(
  {
    coverageDir,
    targetUrl,
  }: {
    readonly coverageDir: string;
    readonly targetUrl: string
  },
): Promise<readonly V8Script[]> {
  /**
   Coverage JSON files, one per process.
   */
  const names = (await readdir(coverageDir,)).filter(function isJson(name,) {
    return name.endsWith('.json',);
  },);
  /**
   Parsed files.
   */
  const files = await Promise.all(names.map(async function parseFile(name,): Promise<unknown> {
    /**
     Parsed coverage JSON.
     */
    const parsed: unknown = JSON.parse(await readFile(
      join(
        coverageDir,
        name,
      ),
      'utf8',
    ),);
    return parsed;
  },),);
  return files.flatMap(function targetEntries(file,) {
    return isCoverageFile(file,)
      ? file.result
        .filter(function isTarget(script,) {
          return script.url === targetUrl;
        },)
      : [];
  },);
}

/**
 Run the gate.

 @param mode - `check` compares against the baseline, `write` refreezes it.
 
 @param coverageDir - `NODE_V8_COVERAGE` output directory.
 
 @param baselinePath - Committed baseline JSON.

 @throws {@link CoverageGateError} When the target was never loaded, the version
   changed, or covered lines fell below the baseline.

 @example
 ```ts
 await runGate({ mode: 'check', coverageDir, baselinePath, });
 ```
 */
export async function runGate(
  {
    mode,
    coverageDir,
    baselinePath,
  }: {
    readonly mode: 'check' | 'write';
    readonly coverageDir: string;
    readonly baselinePath: string;
  },
): Promise<void> {
  /**
   Installed target and its version.
   */
  const {
    path,
    version,
  } = await installedTarget();
  /**
   Coverage entries of the target across processes.
   */
  const scripts = await targetScripts({
    coverageDir,
    targetUrl: pathToFileURL(path,)
      .href,
  },);
  if (scripts.length === 0)
    throw new CoverageGateError(`No coverage recorded for ${path}; did the tests load deepmerge-ts?`,);
  /**
   Target source text.
   */
  const source = await readFile(
    path,
    'utf8',
  );
  /**
   Union of covered lines across processes.
   */
  const covered = new Set(scripts.flatMap(function linesOf(script,) {
    return [
      ...coveredLines({
        script,
        source,
      },),
    ];
  },),);
  /**
   Measured baseline candidate.
   */
  const current: Baseline = {
    codeLines: codeLineCount(source,),
    coveredLines: covered.size,
    version,
  };
  console.log(
    `deepmerge-ts@${version} dist: ${String(current.coveredLines,)}/${String(current.codeLines,)} lines (${
      String(Math.round((current.coveredLines / current.codeLines) * PERCENT_SCALE,),)
    }%)`,
  );
  console.log(`never called: ${uncalledFunctions(scripts,)
    .join(', ',)
    || '(none)'}`,);
  if (mode === 'write') {
    await writeFile(
      baselinePath,
      `${JSON.stringify(
        current,
        undefined,
        2,
      )}\n`,
    );
    console.log(`baseline written to ${baselinePath}`,);
    return;
  }
  /**
   Committed baseline.
   */
  const baseline: unknown = JSON.parse(await readFile(
    baselinePath,
    'utf8',
  ),);
  if (!isBaseline(baseline,))
    throw new CoverageGateError(`${baselinePath} is not a coverage baseline`,);
  if (baseline.version !== version) {
    throw new CoverageGateError(
      `Baseline is for deepmerge-ts@${baseline.version}, installed is ${version}; review coverage and rerun with --write`,
    );
  }
  if (current.coveredLines < baseline.coveredLines) {
    throw new CoverageGateError(
      `Covered lines fell from ${String(baseline.coveredLines,)} to ${String(current.coveredLines,)}`,
    );
  }
  if (current.coveredLines > baseline.coveredLines)
    console.log('coverage grew; rerun with --write to ratchet the baseline',);
}

if (import.meta.main) {
  /**
   Positional script arguments.
   */
  const [mode, coverageDir, baselinePath,] = process.argv
    .slice(2,);
  if (((mode !== 'check') && (mode !== 'write')) || (coverageDir === undefined)
    || (baselinePath === undefined))
    throw new CoverageGateError('usage: node src/coverage-report.ts <check|write> <coverageDir> <baselinePath>',);
  await runGate({
    baselinePath,
    coverageDir,
    mode,
  },);
}
