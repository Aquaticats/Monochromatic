/**
 Exact configured-project fingerprint for semantic cache invalidation.
 
 @module
 */

import { createHash, } from 'node:crypto';
import {
  existsSync,
  readFileSync,
} from 'node:fs';
import {
  dirname,
  join,
} from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { Project, } from 'typescript/sync';
import type { SourceFile, } from 'typescript/ast';

import { ancestorDirectories, } from './ancestor-directories.ts';
import { isDeclarationFileName, } from './declaration-file-name.ts';
import {
  updateHashPlainValue,
  updateHashString,
} from './effect-hash-value.ts';
import { contentDigest, } from './effect-summary-cache-identity.ts';
import { configuredProjectConfigFileName, } from './semantic-file-name.ts';
import { isAmbientProgramFile, } from './shared-analysis-ambient.ts';
import {
  type CheckingOptionsKey,
  checkingOptionsKey,
} from './shared-analysis-options.ts';

/**
 Project fingerprint logger.
 */
const l = tagged({ tag: 'effect-project-fingerprint', },);

/**
 Whole-scope invalidation surfaces validating incremental cache entries.
 
 Each digest binds one channel through which a file's semantics can change
 without any of its resolved module dependencies changing: project
 membership, ambient and external declaration files, global or module
 augmentations authored in non-declaration sources, resolved compiler
 options, and governing lockfile content.
 */
export type EffectProjectSurfaces = {
  readonly fileListDigest: string;
  readonly declarationSurfaceDigest: string;
  readonly augmentationSurfaceDigest: string;
  readonly compilerOptionsDigest: string;
  readonly lockfileDigest: string;
};

/**
 Whole-program inputs of the cross-project analysis key.
 
 The ambient surface binds the program files that apply without an import,
 and the checking options bind what the checker concludes;
 neither names the project's file list, its resolution-only options, or its declaration
 files that no source reaches.
 */
export type EffectSharedSurfaces = {
  readonly ambientSurfaceDigest: string;
  readonly checkingOptions: CheckingOptionsKey;
};

/**
 Fingerprint and per-source identities for one project snapshot.
 */
export type EffectProjectFingerprint = {
  readonly digest: string;
  readonly fileListDigest: string;
  readonly sourceDigests: ReadonlyMap<string, string>;
  readonly surfaces: EffectProjectSurfaces;
  readonly shared: EffectSharedSurfaces;
};

/**
 Tests whether source text can carry global or module augmentation.
 
 Token containment over-approximates: comment or string occurrences also
 bind, which only widens invalidation and never misses a real augmentation.
 
 @param sourceText - Non-declaration source text.
 
 @returns whether text contains an augmentation token.
 */
function containsAugmentationToken(sourceText: string,): boolean {
  return sourceText.includes('declare global',)
    || sourceText.includes('declare module',);
}

/**
 Sentinel when no governing pnpm lockfile exists.
 */
const LOCKFILE_NOT_FOUND: unique symbol = Symbol('governing pnpm lockfile not found',);

/**
 Finds nearest pnpm lockfile governing configured project.
 
 @param configFileName - TypeScript configuration path.
 
 @returns lockfile path or domain-specific absent sentinel.
 */
function nearestLockfile(
  configFileName: string,
): string | typeof LOCKFILE_NOT_FOUND {
  for (const directory of ancestorDirectories(dirname(configFileName,),)) {
    /**
     Candidate pnpm lockfile at current ancestor.
     */
    const candidate = join(
      directory,
      'pnpm-lock.yaml',
    );
    // oxlint-disable-next-line no-restricted-syntax/no-sync -- Synchronous semantic visitor resolves cache identity before analysis.
    if (existsSync(candidate,))
      return candidate;
  }
  return LOCKFILE_NOT_FOUND;
}

/**
 Reads exact source text from active overlay, analysed snapshot, or disk.
 
 The fingerprint names the state the summaries beneath it were derived from, and that state is
 the snapshot. Reading disk instead pairs an unchanged syntax tree with a digest of whatever a
 concurrent write left behind, and stores summaries under a key describing text nothing
 analysed. That entry then persists, and a later ordinary run can reuse it.
 
 So the snapshot answers for every source, declarations included. A declaration file decides
 types, so a stale one is as capable of poisoning a cache entry as a stale implementation.
 
 Declarations were briefly excluded here on the belief that including them cost 6.2 warm
 seconds. That figure came from one run against one run inside a band wider than itself. Five
 warm sweeps of each, run back to back:
 
 ```text
 declarations from snapshot   61.2  61.5  62.4  60.5  61.0     mean 61.32s
 declarations from disk       60.4  61.5  62.8  61.5  60.5     mean 61.34s
 ```
 
 Two hundredths of a second apart, in favour of asking the snapshot. Asking it for all 574
 sources of one project does cost 136.9ms the first time against 13.0ms for a disk pass, because
 nothing else decodes declarations. Every project snapshot pays that figure once, because this
 fingerprint is computed per snapshot before any entry is read. Entries addressed by project and
 path, and entries shared across projects by file and key digest, do not change that: sharing
 saves the analysis of a source, not the fingerprint of each project that reaches it.
 
 @param project - TypeScript project providing analysed source.
 
 @param activeSourceFile - Active Oxlint overlay source.
 
 @param fileName - Program source path to fingerprint.
 
 @returns exact source text.
 
 @throws {@link Error} when neither snapshot nor filesystem can produce source.
 */
function projectSourceText({
  project,
  activeSourceFile,
  fileName,
}: {
  readonly project: Project;
  readonly activeSourceFile: SourceFile;
  readonly fileName: string;
},): string {
  if (fileName === activeSourceFile.fileName)
    return activeSourceFile.text;
  /**
   Source text as the analysed snapshot holds it.
   */
  const snapshotText = project.program
    .getSourceFile(fileName,)
    ?.text;
  if (snapshotText !== undefined)
    return snapshotText;
  try {
    /* oxlint-disable no-restricted-syntax/no-sync -- Synchronous semantic visitor reads a source its own snapshot could not produce. */
    /**
     Disk source text for a path the snapshot omits.
     */
    const text = readFileSync(
      fileName,
      'utf8',
    );
    /* oxlint-enable no-restricted-syntax/no-sync */
    return text;
  }
  catch (error) {
    /**
     Decoded virtual or bundled source unavailable through ordinary filesystem.
     */
    const fallback = project.program
      .getSourceFile(fileName,)
      ?.text;
    if (fallback !== undefined)
      return fallback;
    throw new Error(
      `Cannot fingerprint project source ${fileName}: ${String(error,)}`,
      { cause: error },
    );
  }
}

/**
 Computes exact semantic project fingerprint and source digest map.
 
 @param project - Configured TypeScript project.
 
 @param activeSourceFile - Current overlay source.
 
 @returns project digest,
 file-list digest,
 and per-source digests.
 
 @example
 ```ts
 const fingerprint = effectProjectFingerprint({ project, activeSourceFile });
 ```
 */
export function effectProjectFingerprint({
  project,
  activeSourceFile,
}: {
  readonly project: Project;
  readonly activeSourceFile: SourceFile;
},): EffectProjectFingerprint {
  /**
   Stable program file order independent from TypeScript insertion order.
   */
  const fileNames = project.program
    .getSourceFileNames()
    .toSorted();
  /**
   Digest detecting project graph membership changes.
   */
  const fileListDigest = contentDigest(fileNames.join('\0',),);
  /**
   Per-source content identities retained for process-local active-file hits.
   */
  const sourceDigests = new Map<string, string>();
  /**
   Complete semantic identity digest.
   */
  const digest = createHash('sha256',);
  /**
   Declaration-file surface digest covering ambient and external types.
   */
  const declarationSurface = createHash('sha256',);
  /**
   Augmentation surface digest covering non-declaration ambient authorship.
   */
  const augmentationSurface = createHash('sha256',);
  /**
   Ambient surface digest covering program files that apply without an import.
   */
  const ambientSurface = createHash('sha256',);
  updateHashString({
    digest,
    value: configuredProjectConfigFileName({ project, },),
  },);
  updateHashPlainValue({
    digest,
    value: project.parsedCommandLine
      .options,
  },);
  /**
   Resolved compiler-option identity validated by incremental entries.
   */
  const optionsDigest = createHash('sha256',);
  updateHashPlainValue({
    digest: optionsDigest,
    value: project.parsedCommandLine
      .options,
  },);
  fileNames.forEach(function hashSource(fileName,): void {
    /**
     Exact source text from overlay or configured snapshot.
     */
    const sourceText = projectSourceText({
      project,
      activeSourceFile,
      fileName,
    },);
    /**
     Source content identity.
     */
    const sourceDigest = contentDigest(sourceText,);
    sourceDigests.set(
      fileName,
      sourceDigest,
    );
    updateHashString({
      digest,
      value: fileName,
    },);
    updateHashString({
      digest,
      value: sourceDigest,
    },);
    /**
     Decoded program file whose module status decides ambient membership.
     */
    const programFile = fileName === activeSourceFile.fileName
      ? activeSourceFile
      : project.program
        .getSourceFile(fileName,);
    if ((programFile !== undefined) && isAmbientProgramFile({
      fileName,
      sourceFile: programFile,
    },)) {
      updateHashString({
        digest: ambientSurface,
        value: fileName,
      },);
      updateHashString({
        digest: ambientSurface,
        value: sourceDigest,
      },);
    }
    if (isDeclarationFileName(fileName,)) {
      updateHashString({
        digest: declarationSurface,
        value: fileName,
      },);
      updateHashString({
        digest: declarationSurface,
        value: sourceDigest,
      },);
      return;
    }
    if (containsAugmentationToken(sourceText,)) {
      updateHashString({
        digest: augmentationSurface,
        value: fileName,
      },);
      updateHashString({
        digest: augmentationSurface,
        value: sourceDigest,
      },);
    }
  },);
  /**
   Governing lockfile whose package identities affect resolution.
   */
  const lockfile = nearestLockfile(configuredProjectConfigFileName({ project, },),);
  /**
   Lockfile content identity, or absence marker when no lockfile governs.
   */
  const lockfileState = { digest: 'lockfile-absent', };
  if ((typeof lockfile) !== 'symbol') {
    /* oxlint-disable no-restricted-syntax/no-sync -- Synchronous semantic visitor includes exact lockfile identity once per project cache miss. */
    /**
     Lockfile text for package-resolution invalidation.
     */
    const lockfileText = readFileSync(
      lockfile,
      'utf8',
    );
    /* oxlint-enable no-restricted-syntax/no-sync */
    updateHashString({
      digest,
      value: lockfile,
    },);
    lockfileState.digest = contentDigest(`${lockfile}\0${contentDigest(lockfileText,)}`,);
    updateHashString({
      digest,
      value: contentDigest(lockfileText,),
    },);
  }
  return {
    digest: digest.digest('hex',),
    fileListDigest,
    sourceDigests,
    surfaces: {
      fileListDigest,
      declarationSurfaceDigest: declarationSurface.digest('hex',),
      augmentationSurfaceDigest: augmentationSurface.digest('hex',),
      compilerOptionsDigest: optionsDigest.digest('hex',),
      lockfileDigest: lockfileState.digest,
    },
    shared: {
      ambientSurfaceDigest: ambientSurface.digest('hex',),
      checkingOptions: checkingOptionsKey(project.parsedCommandLine
        .options,),
    },
  };
}
