/**
 Scratch directories for the offline guards, under the package's gitignored tree: a killed run leaves its directory
 behind, and the next run of the same guard sweeps one old enough to be nobody else's.
 */

import {
  mkdirSync,
  mkdtempSync,
  readdirSync,
  rmSync,
  statSync,
} from 'node:fs';
import { join } from 'node:path';

import { designRoot } from './design-page-dependencies.ts';

/** Directory every guard's scratch lives under. */
const SCRATCH_ROOT = join(
  designRoot,
  'node_modules',
  '.monochromatic',
  'music-player-design',
);

/**
 Age at which a leftover scratch directory is swept, in milliseconds. A guard run finished in about 20 s here, so a
 directory this old belongs to a run that was killed rather than to one still working beside this one.
 */
const STALE_SCRATCH_MS = 600_000;

/** Creates a fresh scratch directory named under `prefix`, sweeping same-prefix leftovers first. */
export function createScratch({ prefix, }: { prefix: string; },): string {
  mkdirSync(
    SCRATCH_ROOT,
    { recursive: true, },
  );
  for (const entry of readdirSync(SCRATCH_ROOT,)) {
    if (!entry.startsWith(prefix,)) {
      continue;
    }
    const path = join(
      SCRATCH_ROOT,
      entry,
    );
    try {
      if (Date.now() - statSync(path,).mtimeMs > STALE_SCRATCH_MS) {
        rmSync(
          path,
          { recursive: true, force: true, },
        );
      }
    }
    catch {
      // A concurrent run removed it first, which is the state this sweep wanted anyway.
    }
  }
  return mkdtempSync(join(
    SCRATCH_ROOT,
    prefix,
  ),);
}
