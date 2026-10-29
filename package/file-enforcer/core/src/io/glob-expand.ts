import { relative, } from 'node:path';
import readdirGlob from 'tiny-readdir-glob';

import { caughtErrorHasCode, } from './error.ts';
import { splitGlob, } from './glob-split.ts';

/**
 Reads files matched by a relative glob under a static directory.
 
 @param relativeGlob - Glob suffix passed to {@link readdirGlob}.
 
 @param cwd - Static directory where matching starts.
 
 @returns Matched absolute paths, or empty array when static directory is absent.
 
 @example
 ```ts
 const files = await readGlobFiles({ relativeGlob: '*.ts', cwd: '/tmp/src' });
 ```
 */
async function readGlobFiles(
  {
    relativeGlob,
    cwd,
  }: {
    readonly cwd: string;
    readonly relativeGlob: string;
  },
): Promise<readonly string[]> {
  try {
    /**
     Files matched by glob suffix under `cwd`.
     */
    const { files, } = await readdirGlob(
      relativeGlob,
      { cwd, },
    );
    return files;
  }
  catch (globError: unknown) {
    if (caughtErrorHasCode({
      error: globError,
      code: 'ENOENT',
    },))
      return [];

    throw globError;
  }
}

/**
 Expands a glob pattern against the filesystem via {@link readGlobFiles} and
 returns matched file paths.
 Returned paths preserve prefix format of input pattern: relative patterns
 produce relative paths, absolute patterns produce absolute paths.
 Uses {@link readdirGlob} for matching, which includes dot files.
 
 @param pattern - Glob pattern such as `./packages/*​/src/*.ts`.
 
 @returns Array of matched paths with same prefix style as input pattern.
 
 @example
 ```ts
 const tsFiles = await expandGlob('./src/**​/*.ts');
 ```
 */
export async function expandGlob(pattern: string,): Promise<readonly string[]> {
  /**
   Triple from {@link splitGlob}: matcher cwd, glob suffix, and originally-typed prefix.
   */
  const [cwd, relativeGlob, originalPrefix,] = splitGlob(pattern,);

  if (relativeGlob === '')
    return [cwd,];

  /**
   Files matched by glob suffix under `cwd`.
   */
  const files = await readGlobFiles({
    relativeGlob,
    cwd,
  },);

  return files.map(function toOriginalForm(absolutePath: string,): string {
    /**
     Path relative to resolved cwd.
     */
    const relPath = relative(
      cwd,
      absolutePath,
    );
    /**
     Separator between prefix and relative path.
     */
    const sep = originalPrefix.endsWith('/',) ? '' : '/';
    return `${originalPrefix}${sep}${relPath}`;
  },);
}

/**
 Returns static directory that should be watched for a glob pattern,
 derived via {@link splitGlob}.
 
 @param pattern - Glob pattern tracked during config execution.
 
 @returns Absolute directory whose entries can change glob result.
 
 @example
 ```ts
 const dir = globWatchDirectory('./src/*.ts');
 ```
 */
export function globWatchDirectory(pattern: string,): string {
  /**
   Static matcher directory from glob split.
   */
  const [cwd,] = splitGlob(pattern,);
  return cwd;
}
