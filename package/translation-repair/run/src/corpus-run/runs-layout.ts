import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';

import { artifactsDirOf, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { DECLINED_DIR, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { FIXED_TREE_DIR, } from '@monochromatic-dev/translation-repair-page-check/ts';

//region Runs directory layout
// Where a pass puts what it leaves behind, split out of `corpus-pass.ts` at
// its line budget. Every path is under the one runs directory so a throwaway
// run leaves a throwaway tree and nothing carrying corpus wording can reach
// git: runs directories live outside this repository.
//
// EACH DIRECTORY NAME HAS ONE SPELLING, kept by the module that owns what the
// directory holds and imported here, so a reader that walks a runs directory
// (the cap census skips the large ones) cannot drift from what a pass writes.

/**
 Name of the default runs directory, under the worktree's
 `node_modules/.monochromatic/`; a runs directory set by
 `TRANSLATION_REPAIR_RUNS_DIR` beside it carries it as a prefix by habit
 (`translation-repair-runs-pass16`), which the cache-account audit reads.
 */
export const RUNS_DIR_NAME = 'translation-repair-runs';

/**
 The default runs directory of one worktree, which `resolveRunsDir`
 (`run-config.ts`) uses unless `TRANSLATION_REPAIR_RUNS_DIR` names another.

 @param worktreeRoot - top directory of the worktree

 @returns Its gitignored runs directory

 @example
 ```ts
 const runsDir = defaultRunsDirIn({ worktreeRoot, },);
 ```
 */
export function defaultRunsDirIn({ worktreeRoot, }: { readonly worktreeRoot: string; },): string {
  return join(
    worktreeRoot,
    'node_modules',
    '.monochromatic',
    RUNS_DIR_NAME,
  );
}

/**
 Directory under a runs dir holding per-entry slice caches.
 */
export const SLICE_CACHE_DIR = 'slice-cache';

/**
 Directory under a runs dir holding the prompt payloads a run's client stores.
 */
export const PROMPT_PAYLOADS_DIR = 'prompt-payloads';

/**
 The directories and files one pass reads and writes under its runs dir.

 @example
 ```ts
 const layout: RunsLayout = await prepareRunsLayout({ runsDir, },);
 ```
 */
export type RunsLayout = {
  /**
   Per-entry artifact directory, one JSON per settled entry.
   */
  readonly artifactsDir: string;

  /**
   Root of the corpus tree the pass publishes its fixed pages into.
   */
  readonly publishDir: string;

  /**
   Directory of decline records, one per entry the pipeline declined to
   repair because the archive's note says the page is the author's own
   English (the owner's rule of 2026-09-08); read into the skip set beside
   the artifacts, since a declined entry is done for every later pass too.
   */
  readonly declinedDir: string;

  /**
   Root of per-entry slice caches making large documents resumable.
   */
  readonly sliceCacheDir: string;

  /**
   Directory the run's client stores prompt payloads in.
   */
  readonly promptPayloadDir: string;

  /**
   Persisted attempt-count map path.
   */
  readonly attemptsPath: string;
};

/**
 Names every path under a runs dir and creates the two the pass promises to
 leave behind even when it settles nothing.

 THE ARTIFACTS DIRECTORY AND THE PUBLISHED TREE ARE CREATED HERE rather than
 lazily at the first entry, so a pass that settles no entry still leaves the
 empty tree it promised rather than nothing. The tree sits BESIDE the
 artifacts, under the same runs directory, so a tree carrying corpus wording
 inherits the property that keeps that wording safe.

 @param runsDir - durable, gitignored output root for this run

 @returns Every path the pass uses

 @example
 ```ts
 const { artifactsDir, publishDir, } = await prepareRunsLayout({ runsDir, },);
 ```
 */
export async function prepareRunsLayout(
  { runsDir, }: { readonly runsDir: string; },
): Promise<RunsLayout> {
  /**
   Per-entry artifact directory.
   */
  const artifactsDir = artifactsDirOf({ runsDir, },);
  await mkdir(
    artifactsDir,
    { recursive: true, },
  );
  /**
   Root of the published tree.
   */
  const publishDir = join(
    runsDir,
    FIXED_TREE_DIR,
  );
  await mkdir(
    publishDir,
    { recursive: true, },
  );
  return {
    artifactsDir,
    publishDir,
    declinedDir: join(
      runsDir,
      DECLINED_DIR,
    ),
    sliceCacheDir: join(
      runsDir,
      SLICE_CACHE_DIR,
    ),
    promptPayloadDir: join(
      runsDir,
      PROMPT_PAYLOADS_DIR,
    ),
    attemptsPath: join(
      runsDir,
      'attempts.json',
    ),
  };
}

//endregion Runs directory layout
