import { createHash, } from 'node:crypto';
import type {
  FileStamp,
  GlobStamp,
} from './staleness-types.ts';

/**
 Hashes generated content for manifest diagnostics and destination identity.
 
 @param content - Generated destination content.
 
 @returns SHA-256 hex digest.
 
 @example
 ```ts
 const hash = hashContent('hello');
 ```
 */
export function hashContent(content: string,): string {
  return createHash('sha256',)
    .update(content,)
    .digest('hex',);
}

/**
 Hashes source metadata into a compact manifest field via {@link hashContent}.
 
 @param sourceFiles - Source metadata to hash.
 
 @param sourceGlobs - Source glob expansions to hash.
 
 @returns SHA-256 hex digest for source metadata.
 
 @mutates sourceFiles - `JSON.stringify` may invoke array or entry accessors and proxy traps.
 
 @mutates sourceGlobs - `JSON.stringify` may invoke array or entry accessors and proxy traps.
 
 @example
 ```ts
 const hash = hashSourceSet({ sourceFiles, sourceGlobs });
 ```
 */
export function hashSourceSet(
  {
    sourceFiles,
    sourceGlobs,
  }: {
    sourceFiles: readonly FileStamp[];
    sourceGlobs: readonly GlobStamp[];
  },
): string {
  return hashContent(JSON.stringify({
    sourceFiles,
    sourceGlobs,
  },),);
}
