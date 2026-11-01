/**
 Canonical GitHub repository selection.
 
 @module
 */

import { resolve, } from 'node:path';

import type { GitHubRepository, } from './github-model.ts';
import {
  runBoundedProcess,
  type BoundedProcessRunner,
} from './github-process.ts';

/**
 Required repository URL prefix.
 */
const GITHUB_URL_PREFIX = 'https://github.com/';

/**
 Reports explicit or inferred repository selection failure.
 */
export class RepositorySelectionError extends Error {
  /**
   Creates repository selection failure.
   
   @param message - User-facing repository evidence and remediation.
   
   @example
   ```ts
   const error = new RepositorySelectionError('repository URL is invalid');
   ```
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'RepositorySelectionError';
  }
}

/**
 Checks ASCII alphanumeric character.
 
 @param character - One UTF-16 code unit.
 
 @returns Whether character belongs to ASCII alphanumeric set.
 */
function isAsciiAlphanumeric(character: string,): boolean {
  return ((character >= 'a') && (character <= 'z'))
    || ((character >= 'A') && (character <= 'Z'))
    || ((character >= '0') && (character <= '9'));
}

/**
 Checks GitHub owner character grammar.
 
 @param character - One owner code unit.
 
 @returns Whether owner accepts character.
 */
function isOwnerCharacter(character: string,): boolean {
  return isAsciiAlphanumeric(character,) || (character === '-');
}

/**
 Checks GitHub repository-name character grammar.
 
 @param character - One repository-name code unit.
 
 @returns Whether repository name accepts character.
 */
function isRepositoryCharacter(character: string,): boolean {
  return isAsciiAlphanumeric(character,)
    || (character === '-')
    || (character === '_')
    || (character === '.');
}

/**
 Checks every code unit against supplied character grammar.
 
 @param value - Owner or repository segment.
 
 @param accepts - Character predicate for segment grammar.
 
 @returns Whether non-empty segment contains only accepted characters.
 */
function everyCharacter({
  value,
  accepts,
}: {
  readonly value: string;
  readonly accepts: (character: string) => boolean;
},): boolean {
  if (value === '') {
    return false;
  }
  for (let index = 0; index < value.length; index += 1) {
    if (!accepts(value.charAt(index,),)) {
      return false;
    }
  }
  return true;
}

/**
 Parses exact canonical GitHub repository HTTPS URL.
 
 @param value - User-supplied `--repo` value.
 
 @returns Canonical owner, name, and unchanged URL.
 
 @throws {@link RepositorySelectionError} when value differs from required shape.
 
 @example
 ```ts
 parseRepositoryUrl('https://github.com/owner/repository');
 ```
 */
export function parseRepositoryUrl(value: string,): GitHubRepository {
  if (!value.startsWith(GITHUB_URL_PREFIX,)) {
    throw new RepositorySelectionError(
      'repository must use https://github.com/OWNER/NAME',
    );
  }
  /**
   Owner/name portion after fixed host prefix.
   */
  const suffix = value.slice(GITHUB_URL_PREFIX.length,);
  /**
   Exact path segments without URL parser normalization.
   */
  const segments = suffix.split('/',);
  if (segments.length !== 2) {
    throw new RepositorySelectionError(
      'repository must contain exactly OWNER/NAME with no trailing slash',
    );
  }
  /**
   Candidate owner segment.
   */
  const [owner, name,] = segments;
  if ((owner === undefined)
    || (name === undefined)
    || (!everyCharacter({
      value: owner,
      accepts: isOwnerCharacter,
    }))
    || (!everyCharacter({
      value: name,
      accepts: isRepositoryCharacter,
    }))
    || owner.startsWith('-',)
    || owner.endsWith('-',)
    || name.endsWith('.git',))
    {
    throw new RepositorySelectionError(
      'repository must use canonical https://github.com/OWNER/NAME characters',
    );
  }
  return {
    owner,
    name,
    url: value,
  };
}


/**
 Converts supported Git remote URL to canonical repository URL.
 
 @param remote - Origin URL read from Git configuration.
 
 @returns Canonical repository identity.
 
 @throws {@link RepositorySelectionError} when origin is not unambiguous GitHub remote.
 */
function parseOriginRemote(remote: string,): GitHubRepository {
  /**
   Trimmed configured origin URL.
   */
  const value = remote.trim();
  /**
   Owner/name suffix extracted from supported remote grammar.
   */
  const suffix = value.startsWith(GITHUB_URL_PREFIX,)
    ? value.slice(GITHUB_URL_PREFIX.length,)
    : value.startsWith('git@github.com:',)
      ? value.slice('git@github.com:'.length,)
      : value.startsWith('ssh://git@github.com/',)
        ? value.slice('ssh://git@github.com/'.length,)
        : '';
  if (suffix === '') {
    throw new RepositorySelectionError(
      'Git origin is not a supported github.com remote; pass --repo https://github.com/OWNER/NAME',
    );
  }
  /**
   Owner/name without clone-only `.git` suffix.
   */
  const canonicalSuffix = suffix.endsWith('.git',)
    ? suffix.slice(
      0,
      -'.git'.length,
    )
    : suffix;
  return parseRepositoryUrl(`${GITHUB_URL_PREFIX}${canonicalSuffix}`,);
}

/**
 Runs one Git inspection command and converts process failures to selection errors.
 
 @param cwd - Current working directory.
 
 @param arguments - Exact Git argument vector.
 
 @param runProcess - Bounded process implementation.
 
 @returns Captured Git standard output.
 
 @throws {@link RepositorySelectionError} when Git command fails.
 */
async function runGit({
  cwd,
  arguments: commandArguments,
  runProcess,
}: {
  readonly cwd: string;
  readonly arguments: readonly string[];
  readonly runProcess: BoundedProcessRunner;
},): Promise<string> {
  try {
    /**
     Captured Git inspection result.
     */
    const result = await runProcess({
      file: 'git',
      arguments: commandArguments,
      cwd,
    },);
    return result.stdout;
  }
  catch (error: unknown) {
    throw new RepositorySelectionError(
      `repository inference failed: ${String(error,)}; pass --repo https://github.com/OWNER/NAME`,
    );
  }
}

/**
 Selects explicit repository or infers origin only from exact worktree root.
 
 @param explicitUrl - Optional canonical URL from `--repo`.
 
 @param cwd - Process working directory.
 
 @param runProcess - Bounded process implementation.
 
 @returns Canonical destination repository.
 
 @throws {@link RepositorySelectionError} when inference is unavailable or cwd is subdirectory.
 
 @example
 ```ts
 await selectRepository({ cwd: process.cwd() });
 ```
 */
export async function selectRepository({
  explicitUrl,
  cwd,
  runProcess = runBoundedProcess,
}: {
  readonly explicitUrl?: string;
  readonly cwd: string;
  readonly runProcess?: BoundedProcessRunner;
},): Promise<GitHubRepository> {
  if (explicitUrl !== undefined) {
    return parseRepositoryUrl(explicitUrl,);
  }
  /**
   Git-reported worktree top-level path.
   */
  const root = (await runGit({
    cwd,
    arguments: [
      'rev-parse',
      '--show-toplevel',
    ],
    runProcess,
  },)).trim();
  if (resolve(root,) !== resolve(cwd,)) {
    throw new RepositorySelectionError(
      'repository inference requires running at exact Git worktree root; rerun there or pass --repo https://github.com/OWNER/NAME',
    );
  }
  /**
   Configured origin remote URL.
   */
  const origin = await runGit({
    cwd,
    arguments: [
      'remote',
      'get-url',
      'origin',
    ],
    runProcess,
  },);
  return parseOriginRemote(origin,);
}
