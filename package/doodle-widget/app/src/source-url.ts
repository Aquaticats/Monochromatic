/**
 Build-time source code URL resolver for the doodle widget.
 
 Constructs a browsable GitHub URL from the git remote and
 `package.json` repository metadata. Both parts fall back to
 hardcoded literals when their primary source is unavailable
 (e.g. archive builds without `.git`, or a missing field).
 */
import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  findRootCached,
  MISE_MONOREPO,
} from '@monochromatic-dev/fs-path-core/ts';
import spawn from 'nano-spawn';

/**
 Fallback base repository URL when git remote is unavailable
 */
const FALLBACK_REPO_URL = 'https://github.com/Aquaticat/Monochromatic';

/**
 Fallback subdirectory when `repository.directory` is missing from package.json
 */
const FALLBACK_DIRECTORY = 'package/doodle-widget/app';

/**
 Queries `git remote get-url origin` and strips a trailing `.git` suffix.
 
 @returns browsable repository URL, or {@link FALLBACK_REPO_URL} on failure
 */
async function resolveRepoUrl(): Promise<string> {
  try {
    /**
     Monorepo root so `git remote` runs against the correct working tree.
     */
    const repoRoot = await findRootCached({ marker: MISE_MONOREPO, },);
    /**
     Captured so the trailing `.git` can be trimmed before returning.
     */
    const result = await spawn(
      'git',
      [
        'remote',
        'get-url',
        'origin',
      ],
      { cwd: repoRoot, },
    );
    /**
     Trimmed remote URL; the trailing `.git` suffix is stripped if present so the result is browsable.
     */
    const trimmed = result.output
      .trim();
    return trimmed.endsWith('.git',)
      ? trimmed.slice(
        0,
        -'.git'.length,
      )
      : trimmed;
  }
  catch (error) {
    console.error(`[doodle-widget] git remote unavailable; using fallback repository URL: ${String(error,)}`,);
    return FALLBACK_REPO_URL;
  }
}

/**
 Type guard that narrows `unknown` to a string-keyed record.
 
 @param value - value to check
 
 @returns true when {@link value} is a plain object (non-null, non-array)
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return ((typeof value) === 'object') && (value !== null);
}

/**
 Reads `repository.directory` from the package's `package.json`.
 
 @param packageDir - absolute path to the package root directory
 
 @returns subdirectory string, or {@link FALLBACK_DIRECTORY} on failure
 */
async function resolveDirectory(packageDir: string,): Promise<string> {
  try {
    /**
     Raw manifest text so the JSON parse error stays scoped to this function.
     */
    const raw = await readFile(
      join(
        packageDir,
        'package.json',
      ),
      'utf8',
    );
    /**
     Untyped tree narrowed via {@link isRecord} before field access.
     */
    const parsed: unknown = JSON.parse(raw,);
    if (!isRecord(parsed,))
      return FALLBACK_DIRECTORY;

    /**
     Repository field destructured so the runtime shape is checked before reading `directory`.
     */
    const { repository, } = parsed;
    if (!isRecord(repository,))
      return FALLBACK_DIRECTORY;

    /**
     Directory field destructured so a non-string value falls through to the fallback.
     */
    const { directory, } = repository;
    if ((typeof directory) !== 'string')
      return FALLBACK_DIRECTORY;

    return directory;
  }
  catch (error) {
    console.error(`[doodle-widget] package.json unreadable or malformed; using fallback directory: ${String(error,)}`,);
    return FALLBACK_DIRECTORY;
  }
}

/**
 Resolves the full source code URL for this package.
 
 Combines the git remote resolved by {@link resolveRepoUrl} (or
 fallback) with the package.json directory resolved by
 {@link resolveDirectory} (or fallback) into a `/tree/main/{directory}`
 GitHub URL.
 
 @param packageDir - absolute path to the package root directory
 
 @returns browsable source code URL
 
 @example
 ```ts
 const url = await resolveSourceUrl('/path/to/doodle-widget');
 // 'https://github.com/Aquaticat/Monochromatic/tree/main/packages/webapp-productivity/doodle-widget'
 ```
 */
export async function resolveSourceUrl(packageDir: string,): Promise<string> {
  /**
   Two halves resolved together so the slower I/O paths overlap.
   */
  const [repoUrl, directory,] = await Promise.all([
    resolveRepoUrl(),
    resolveDirectory(packageDir,),
  ],);

  return `${repoUrl}/tree/main/${directory}`;
}
