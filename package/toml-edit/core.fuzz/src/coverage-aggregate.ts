/**
 Aggregates raw V8 coverage JSON into per-file toml-edit line coverage.
 
 Kept separate from `coverage-v8.ts` so the V8 range projection primitives and
 the asynchronous file aggregation pipeline each stay below the line-count
 budget.
 
 @module
 */

import {
  readFile,
  readdir,
} from 'node:fs/promises';
import { join, } from 'node:path';


import {
  type CoverageMap,
  type V8Function,
  classifyUrl,
  fileCoverageFrom,
  isCoverageFile,
  projectCovered,
} from './coverage-v8.ts';

//region Aggregation shapes

/**
 Per-target accumulation across coverage files before line projection.
 */
type TargetAccumulator = {
  /**
   On-disk source text for the target file.
   */
  readonly source: string;
  /**
   Covered line numbers unioned across coverage files.
   */
  readonly covered: Set<number>;
};

/**
 Coverage target extracted from one V8 coverage JSON file.
 */
type CoverageTarget = {
  /**
   Package-relative target path used as the coverage map key.
   */
  readonly relPath: string;
  /**
   Absolute path to the target source file.
   */
  readonly absPath: string;
  /**
   V8 function coverage ranges for this target occurrence.
   */
  readonly functions: readonly V8Function[];
};

/**
 Target projected onto covered line numbers after source loading.
 */
type ProjectedTarget = {
  /**
   Package-relative target path used as the coverage map key.
   */
  readonly relPath: string;
  /**
   On-disk source text for the target file.
   */
  readonly source: string;
  /**
   Covered line numbers for this target occurrence.
   */
  readonly covered: ReadonlySet<number>;
};

//endregion Aggregation shapes

//region Coverage file loading

/**
 Reads one V8 coverage JSON file and returns its package target scripts.
 
 @param coverageDir - directory holding V8 coverage JSON files
 
 @param file - coverage JSON filename inside `coverageDir`
 
 @param packageRoot - absolute package root used for target classification
 
 @returns coverage targets extracted from the file
 */
async function readCoverageTargets(
  {
    coverageDir,
    file,
    packageRoot,
  }: {
    readonly coverageDir: string;
    readonly file: string;
    readonly packageRoot: string;
  },
): Promise<readonly CoverageTarget[]> {
  /**
   Parsed coverage file, narrowed by assertion from `unknown`.
   */
  const parsed: unknown = JSON.parse(await readFile(
    join(
      coverageDir,
      file,
    ),
    'utf8',
  ),);
  if (!isCoverageFile(parsed,)) throw new Error('Malformed V8 coverage JSON: expected a result array',);

  return parsed.result
    .flatMap(function targetFromScript(script,): readonly CoverageTarget[] {
      /**
       Target classification for this script's URL.
       */
      const cls = classifyUrl({
        url: script.url,
        packageRoot,
      },);
      if (cls.kind !== 'target') return [];
      return [{
        relPath: cls.relPath,
        absPath: cls.absPath,
        functions: script.functions,
      },];
    },);
}

//endregion Coverage file loading

//region Source projection

/**
 Builds a cached source reader for one aggregation run.
 
 @returns function that reads each absolute source path at most once
 
 @example
 ```ts
 const readSource = createSourceReader();
 await readSource('/repo/packages/module/toml-edit/src/index.ts');
 ```
 */
function createSourceReader(): (absPath: string,) => Promise<string> {
  /**
   In-flight or fulfilled source reads keyed by absolute source path.
   */
  const sourceCache = new Map<string, Promise<string>>();

  return function readSource(absPath: string,): Promise<string> {
    /**
     Existing in-flight or fulfilled read for this source file.
     */
    const existing = sourceCache.get(absPath,);
    if (existing !== undefined)
      return existing;

    /**
     New source read stored immediately so concurrent target projections share it.
     */
    const sourcePromise = readFile(
      absPath,
      'utf8',
    );
    sourceCache.set(
      absPath,
      sourcePromise,
    );
    return sourcePromise;
  };
}

/**
 Projects one target's V8 ranges onto covered line numbers.
 
 @param target - coverage target to project
 
 @param readSource - cached source reader for this aggregation run
 
 @returns projected target with source and covered lines
 
 @mutates readSource - Invoking cached source reader can change its caller-owned cache and asynchronous state.
 */
async function projectTarget(
  {
    target,
    readSource,
  }: {
    readonly target: CoverageTarget;
    readonly readSource: (absPath: string,) => Promise<string>;
  },
): Promise<ProjectedTarget> {
  /**
   Source text loaded through the shared cache.
   */
  const source = await readSource(target.absPath,);
  return {
    relPath: target.relPath,
    source,
    covered: projectCovered({
      source,
      functions: target.functions,
    },),
  };
}

//endregion Source projection

//region Coverage merge

/**
 Unions projected target coverage by package-relative path.
 
 @param targets - projected target occurrences across all coverage files
 
 @returns accumulator map keyed by package-relative target path
 */
function mergeProjectedTargets(targets: readonly ProjectedTarget[],): Map<string, TargetAccumulator> {
  /**
   Per-file accumulation of covered lines across all projected targets.
   */
  const perFile = new Map<string, TargetAccumulator>();

  for (const target of targets) {
    /**
     Existing file accumulator, when another coverage script already reached the file.
     */
    const existing = perFile.get(target.relPath,);
    if (existing === undefined) {
      perFile.set(
        target.relPath,
        {
          source: target.source,
          covered: new Set(target.covered,),
        },
      );
      continue;
    }

    for (const lineNumber of target.covered) {
      existing.covered
        .add(lineNumber,);
    }
  }

  return perFile;
}

/**
 Converts per-file accumulators to the public coverage map shape.
 
 @param perFile - accumulation keyed by package-relative target path
 
 @returns coverage map keyed by package-relative target path
 */
function coverageMapFrom(perFile: ReadonlyMap<string, TargetAccumulator>,): CoverageMap {
  return Object.fromEntries(
    [...perFile.entries(),].map(function project([relPath, info,],) {
      return [
        relPath,
        fileCoverageFrom({
          source: info.source,
          coveredLines: info.covered,
        },),
      ] as const;
    },),
  );
}

//endregion Coverage merge

//region Public aggregation

/**
 Read every `NODE_V8_COVERAGE` JSON file in `coverageDir` and project the
 target files' ranges onto per-file line coverage.
 
 @param coverageDir - V8 coverage output directory
 
 @param packageRoot - absolute package root used for target classification
 
 @returns Coverage keyed by package-relative path; only target files appear.
 
 @example
 ```ts
 const map = await aggregateCoverage({ coverageDir, packageRoot, },);
 ```
 */
export async function aggregateCoverage(
  {
    coverageDir,
    packageRoot,
  }: {
    readonly coverageDir: string;
    readonly packageRoot: string;
  },
): Promise<CoverageMap> {
  /**
   Coverage JSON filenames, filtered before parallel reading.
   */
  const coverageFiles = (await readdir(coverageDir,))
    .filter(function isCoverageJson(file,) {
      return file.endsWith('.json',);
    },);
  /**
   Target script records extracted from every coverage file.
   */
  const targets = (await Promise.all(coverageFiles.map(function readTargets(file,) {
    return readCoverageTargets({
      coverageDir,
      file,
      packageRoot,
    },);
  },)))
    .flat();
  /**
   Source reader shared across target projections.
   */
  const readSource = createSourceReader();
  /**
   Per-target covered-line projections, run concurrently once targets are known.
   */
  const projectedTargets = await Promise.all(targets.map(function project(target,) {
    return projectTarget({
      target,
      readSource,
    },);
  },));
  return coverageMapFrom(mergeProjectedTargets(projectedTargets,),);
}

//endregion Public aggregation
