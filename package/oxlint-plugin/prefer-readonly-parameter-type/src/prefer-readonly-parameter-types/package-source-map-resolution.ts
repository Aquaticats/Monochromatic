/**
 Shipped implementation source-map resolution and content identity.
 
 @module
 */

import {
  existsSync,
  readFileSync,
} from 'node:fs';
import {
  dirname,
  extname,
  resolve,
} from 'node:path';

import { contentDigest, } from './effect-summary-cache-identity.ts';

//region Source-map vocabulary
// The reference marker a generated file carries, the sentinel for an
// implementation with no usable map, the suffixes trusted as inspectable
// source, and the analysis evidence one implementation yields.

/**
 Source-map reference marker in generated JavaScript.
 */
const SOURCE_MAP_MARKER = 'sourceMappingURL=';

/**
 Sentinel when implementation has no usable source map.
 */
const SOURCE_MAP_UNAVAILABLE: unique symbol = Symbol(
  'shipped implementation source map could not be resolved',
);

/**
 Supported inspectable source suffixes.
 */
const SOURCE_SUFFIXES: ReadonlySet<string> = new Set([
  '.js',
  '.mjs',
  '.cjs',
  '.ts',
  '.mts',
  '.cts',
  '.jsx',
  '.tsx',
],);

/**
 Implementation analysis path and exact evidence digest.
 */
export type ImplementationAnalysisEvidence = {
  readonly analysisPath: string;
  readonly digest: string;
};

//endregion Source-map vocabulary

//region Reading the reference
// Whether parsed JSON is a single-source map, and the reference extracted from
// the implementation text without rescanning it.

/**
 Tests whether parsed source map has one source list.
 
 @param value - Parsed source-map JSON.
 
 @returns whether source fields can be inspected.
 */
function isSourceMap(value: unknown,): value is {
  readonly sources: readonly unknown[];
  readonly sourceRoot?: unknown;
} {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('sources' in value)
    && Array.isArray(value.sources,);
}

/**
 Extracts authored source-map reference without regular expression rescanning.
 
 @param implementationText - Shipped implementation text.
 
 @returns map reference or unavailable sentinel.
 */
function sourceMapReference(
  implementationText: string,
): string | typeof SOURCE_MAP_UNAVAILABLE {
  /**
   Final source-map marker offset in generated implementation.
   */
  const markerIndex = implementationText.lastIndexOf(SOURCE_MAP_MARKER,);
  if (markerIndex === (-1))
    return SOURCE_MAP_UNAVAILABLE;
  /**
   Reference start after marker.
   */
  const start = markerIndex + SOURCE_MAP_MARKER.length;
  /**
   Newline and block-comment terminator candidates.
   */
  const endCandidates = [
    implementationText.indexOf(
      '\n',
      start,
    ),
    implementationText.indexOf(
      '\r',
      start,
    ),
    implementationText.indexOf(
      '*/',
      start,
    ),
  ].filter(function presentOffset(offset,): boolean {
    return offset >= 0;
  },);
  /**
   Earliest delimiter or end of text.
   */
  const end = endCandidates.length === 0
    ? implementationText.length
    : Math.min(...endCandidates,);
  /**
   Trimmed source-map reference.
   */
  const reference = implementationText.slice(
    start,
    end,
  )
    .trim();
  return (reference.length === 0) || reference.startsWith('data:',)
    ? SOURCE_MAP_UNAVAILABLE
    : reference;
}

//endregion Reading the reference

//region Resolving the source
// The map path from the authored reference or the adjacent convention, and the
// one existing package-local source it names.

/**
 Resolves source-map path from authored reference or adjacent convention.
 
 @param implementationPath - Shipped runtime implementation path.
 
 @param implementationText - Shipped runtime implementation text.
 
 @returns existing map path or unavailable sentinel.
 */
function sourceMapPath({
  implementationPath,
  implementationText,
}: {
  readonly implementationPath: string;
  readonly implementationText: string;
}): string | typeof SOURCE_MAP_UNAVAILABLE {
  /**
   Authored source-map reference when present.
   */
  const reference = sourceMapReference(implementationText,);
  /**
   Authored path or conventional adjacent fallback.
   */
  const candidate = reference === SOURCE_MAP_UNAVAILABLE
    ? `${implementationPath}.map`
    : resolve(
      dirname(implementationPath,),
      reference,
    );
  // oxlint-disable-next-line no-restricted-syntax/no-sync -- Synchronous semantic visitor checks demanded implementation source-map evidence.
  return existsSync(candidate,)
    ? candidate
    : SOURCE_MAP_UNAVAILABLE;
}

/**
 Resolves one existing package-local source from source map.
 
 @param packageRoot - Exact package root constraining trusted sources.
 
 @param mapPath - Shipped source-map path.
 
 @param mapText - Shipped source-map JSON.
 
 @returns existing source path or unavailable sentinel.
 */
function mappedSourcePath({
  packageRoot,
  mapPath,
  mapText,
}: {
  readonly packageRoot: string;
  readonly mapPath: string;
  readonly mapText: string;
}): string | typeof SOURCE_MAP_UNAVAILABLE {
  /**
   Parsed untrusted source-map JSON.
   */
  const parsed: unknown = JSON.parse(mapText,);
  if ((!isSourceMap(parsed,))
    || (parsed.sources
      .length
      !== 1)
    || ((typeof parsed.sources[0]) !== 'string'))
    return SOURCE_MAP_UNAVAILABLE;
  /**
   Optional source root accepted only as string.
   */
  const sourceRoot = (typeof parsed.sourceRoot) === 'string'
    ? parsed.sourceRoot
    : '';
  /**
   Package-local mapped source candidate.
   */
  const candidate = resolve(
    dirname(mapPath,),
    sourceRoot,
    parsed.sources[0],
  );
  if ((!candidate.startsWith(`${packageRoot}/`,))
    || (!SOURCE_SUFFIXES.has(extname(candidate,))))
    return SOURCE_MAP_UNAVAILABLE;
  // oxlint-disable-next-line no-restricted-syntax/no-sync -- Synchronous semantic visitor confirms mapped package source before analysis.
  return existsSync(candidate,)
    ? candidate
    : SOURCE_MAP_UNAVAILABLE;
}

//endregion Resolving the source

//region The evidence
// The inspectable implementation source with its complete evidence digest.

/**
 Resolves inspectable implementation source and complete evidence digest.
 
 @param packageRoot - Exact package root.
 
 @param implementationPath - Shipped runtime implementation entry.
 
 @returns runtime analysis path and exact digest including mapped source evidence when available.
 
 @example
 ```ts
 implementationAnalysisEvidence({ packageRoot, implementationPath });
 ```
 */
export function implementationAnalysisEvidence({
  packageRoot,
  implementationPath,
}: {
  readonly packageRoot: string;
  readonly implementationPath: string;
}): ImplementationAnalysisEvidence {
  /* oxlint-disable no-restricted-syntax/no-sync -- Synchronous semantic visitor hashes demanded implementation and source-map evidence. */
  /**
   Exact shipped implementation text.
   */
  const implementationText = readFileSync(
    implementationPath,
    'utf8',
  );
  /* oxlint-enable no-restricted-syntax/no-sync */
  /**
   Existing source-map path when referenced or adjacent.
   */
  const mapPath = sourceMapPath({
    implementationPath,
    implementationText,
  },);
  if (mapPath === SOURCE_MAP_UNAVAILABLE) {
    return {
      analysisPath: implementationPath,
      digest: contentDigest(implementationText,),
    };
  }
  /* oxlint-disable no-restricted-syntax/no-sync -- Synchronous semantic visitor hashes demanded source-map bytes. */
  /**
   Exact shipped source-map text.
   */
  const mapText = readFileSync(
    mapPath,
    'utf8',
  );
  /* oxlint-enable no-restricted-syntax/no-sync */
  /**
   Existing package-local mapped source when map is unambiguous.
   */
  const sourcePath = mappedSourcePath({
    packageRoot,
    mapPath,
    mapText,
  },);
  if (sourcePath === SOURCE_MAP_UNAVAILABLE) {
    return {
      analysisPath: implementationPath,
      digest: contentDigest(`${implementationText}\0${mapText}`,),
    };
  }
  /* oxlint-disable no-restricted-syntax/no-sync -- Synchronous semantic visitor hashes exact mapped source as cache evidence. */
  /**
   Exact mapped source text.
   */
  const sourceText = readFileSync(
    sourcePath,
    'utf8',
  );
  /* oxlint-enable no-restricted-syntax/no-sync */
  return {
    analysisPath: implementationPath,
    digest: contentDigest(`${implementationText}\0${mapText}\0${sourceText}`,),
  };
}
//endregion The evidence
