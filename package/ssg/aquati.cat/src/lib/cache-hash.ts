/**
 Hashing utilities for the build cache.
 
 Provides SHA-256 content hashing and mtime-based fingerprinting
 for cache invalidation.
 */
import { createHash, } from 'node:crypto';
import { stat, } from 'node:fs/promises';

import readdir from 'tiny-readdir-glob';

/**
 Computes a SHA-256 hex digest of a string.
 
 @param input - string to hash
 
 @returns hex-encoded SHA-256 digest
 
 @example
 ```ts
 const hash = sha256('hello world');
 ```
 */
export function sha256(input: string,): string {
  return createHash('sha256',)
    .update(input,)
    .digest('hex',);
}

/**
 Computes a fingerprint from file mtimes for all files matching a glob pattern.
 
 When any matched file is modified (mtime changes), added, or removed,
 all cached content entries are invalidated.
 Uses `stat()` calls instead of reading file contents,
 avoiding file I/O and hashing overhead entirely.
 
 @param glob - glob pattern matching pipeline source files
 
 @returns fingerprint string encoding file count and max mtime
 
 @example
 ```ts
 const fingerprint = await computePipelineFingerprint('src/{lib,component,client}/**\/*.ts');
 ```
 */
export async function computePipelineFingerprint(
  glob: string,
): Promise<string> {
  /**
   Glob expansion result; `.files` holds the matched paths used downstream.
   */
  const result = await readdir(glob,);
  /**
   File path list feeding the per-entry mtime fan-out.
   */
  const paths = result.files;
  /**
   Modification timestamps gathered in parallel for the max-mtime aggregation.
   */
  const mtimes = await Promise.all(
    paths.map(async function getMtime(path,) {
      /**
       Per-file stat result used solely for `mtimeMs`.
       */
      const stats = await stat(path,);
      return stats.mtimeMs;
    },),
  );
  /**
   Greatest modification time, or zero when no files match, encoded into the fingerprint.
   */
  const maxMtime = mtimes.length
    > 0
    ? Math.max(...mtimes,)
    : 0;
  return `${paths.length}:${maxMtime}`;
}
