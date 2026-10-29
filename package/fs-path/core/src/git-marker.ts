/**
 Git administrative marker validation.
 
 Mirrors Git's repository signatures without starting a Git subprocess.
 
 @module
 */

import { resolve, } from '#posix-path';
import {
  ABSENT,
  type RootFilesystem,
} from './root-filesystem-contract.ts';
import type { RootMatcherArgs, } from './root-marker-contract.ts';

/**
 Gitfile prefix required by Git's `read_gitfile_gently`.
 */
const GIT_DIRECTORY_PREFIX = 'gitdir: ';

/**
 Maximum gitfile character count accepted before validation.
 */
const MAX_GIT_FILE_CHARACTERS = 1_048_576;

/**
 SHA-1 hexadecimal length.
 */
const SHA_ONE_LENGTH = 40;

/**
 SHA-256 hexadecimal length.
 */
const SHA_TWO_LENGTH = 64;

/**
 ASCII hexadecimal characters accepted in detached HEAD IDs.
 */
const HEX_CHARACTERS: ReadonlySet<string> = new Set('0123456789abcdefABCDEF',);

/**
 ASCII carriage return code point.
 */
const CARRIAGE_RETURN_CODE_POINT = 13;

/**
 ASCII line feed code point.
 */
const LINE_FEED_CODE_POINT = 10;

/**
 Removes trailing line endings the same way Git reads gitfiles and commondir.
 
 @param value - text whose trailing CR and LF characters should be removed
 
 @returns text without trailing line endings
 
 @example
 ```ts
 trimTrailingLineEndings('path\r\n');
 ```
 */
function trimTrailingLineEndings(value: string,): string {
  for (let index = value.length - 1; index >= 0; index -= 1) {
    /**
     Last retained code point candidate.
     */
    const codePoint = value.codePointAt(index,);
    if ((codePoint !== CARRIAGE_RETURN_CODE_POINT)
      && (codePoint !== LINE_FEED_CODE_POINT)) {
      return value.slice(
        0,
        index + 1,
      );
    }
  }
  return '';
}

/**
 Checks hexadecimal object-ID text without regular expressions.
 
 @param value - candidate detached HEAD value
 
 @returns whether every character is an ASCII hexadecimal digit
 
 @example
 ```ts
 isHexObjectId('a'.repeat(40));
 ```
 */
function isHexObjectId(value: string,): boolean {
  if ((value.length !== SHA_ONE_LENGTH) && (value.length !== SHA_TWO_LENGTH))
    return false;
  for (const character of value) {
    if (!HEX_CHARACTERS.has(character,))
      return false;
  }
  return true;
}

/**
 Validates symbolic or detached HEAD text.
 
 @param content - exact HEAD file text
 
 @returns whether Git accepts this broad HEAD shape
 
 @example
 ```ts
 isValidHead('ref: refs/heads/main\n');
 ```
 */
function isValidHead(content: string,): boolean {
  /**
   HEAD text without Git-ignored trailing line endings.
   */
  const value = trimTrailingLineEndings(content,);
  if (value.startsWith('ref:',))
    return value.slice('ref:'.length,)
      .trimStart()
      .startsWith('refs/',);
  return isHexObjectId(value,);
}

/**
 Validates regular-file and symbolic-link HEAD forms.
 
 @param headPath - candidate HEAD path
 
 @param fs - root-discovery filesystem adapter
 
 @returns whether HEAD has a Git-supported shape
 
 @example
 ```ts
 await isValidHeadPath({ headPath: '/repo/.git/HEAD', fs });
 ```
 */
async function isValidHeadPath({
  headPath,
  fs,
}: {
  readonly headPath: string;
  readonly fs: RootFilesystem;
},): Promise<boolean> {
  /**
   Symbolic HEAD target accepted when it names refs namespace.
   */
  const symbolicHead = await fs.readSymbolicLink(headPath,);
  if (symbolicHead !== ABSENT)
    return symbolicHead.startsWith('refs/',);
  if (!await fs.isFile(headPath,))
    return false;
  /**
   Regular HEAD content.
   */
  const head = await fs.readTextFile(headPath,);
  return (head !== ABSENT) && isValidHead(head,);
}

/**
 Parses a Git path payload after a required prefix.
 
 @param content - exact file text
 
 @param prefix - required syntax prefix
 
 @returns path payload or absence sentinel when malformed
 
 @example
 ```ts
 parsePathPayload({ content: 'gitdir: target\n', prefix: 'gitdir: ' });
 ```
 */
function parsePathPayload({
  content,
  prefix,
}: {
  readonly content: string;
  readonly prefix: string;
},): string | typeof ABSENT {
  if ((content.length > MAX_GIT_FILE_CHARACTERS) || (!content.startsWith(prefix,)))
    return ABSENT;
  /**
   Path payload without Git-ignored trailing line endings.
   */
  const path = trimTrailingLineEndings(content.slice(prefix.length,),);
  if ((path === '') || path.includes('\0',))
    return ABSENT;
  return path;
}

/**
 Resolves common administrative directory for ordinary and linked worktrees.
 
 @param gitDirectory - candidate Git administrative directory
 
 @param fs - root-discovery filesystem adapter
 
 @returns common directory or absence sentinel when commondir is malformed
 
 @example
 ```ts
 await resolveCommonDirectory({ gitDirectory: '/repo/.git', fs });
 ```
 */
async function resolveCommonDirectory({
  gitDirectory,
  fs,
}: {
  readonly gitDirectory: string;
  readonly fs: RootFilesystem;
},): Promise<string | typeof ABSENT> {
  /**
   Optional linked-worktree common directory pointer path.
   */
  const commonDirectoryPath = `${gitDirectory}/commondir`;
  if (!await fs.exists(commonDirectoryPath,))
    return gitDirectory;
  if (!await fs.isFile(commonDirectoryPath,))
    return ABSENT;
  /**
   Linked-worktree common directory pointer content.
   */
  const content = await fs.readTextFile(commonDirectoryPath,);
  if (content === ABSENT)
    return ABSENT;
  /**
   Parsed common directory path.
   */
  const path = parsePathPayload({
    content,
    prefix: '',
  },);
  if (path === ABSENT)
    return ABSENT;
  return resolve([
    gitDirectory,
    path,
  ],);
}

/**
 Checks Git's HEAD, objects, and refs repository signatures.
 
 @param gitDirectory - candidate Git administrative directory
 
 @param fs - root-discovery filesystem adapter
 
 @returns whether candidate is usable as a Git directory
 
 @example
 ```ts
 await isValidGitDirectory({ gitDirectory: '/repo/.git', fs });
 ```
 */
async function isValidGitDirectory({
  gitDirectory,
  fs,
}: {
  readonly gitDirectory: string;
  readonly fs: RootFilesystem;
},): Promise<boolean> {
  if (!await fs.isDirectory(gitDirectory,))
    return false;
  /**
   Candidate HEAD path.
   */
  const headPath = `${gitDirectory}/HEAD`;
  if (!await isValidHeadPath({
    headPath,
    fs,
  },))
    return false;
  /**
   Directory that must own objects and refs.
   */
  const commonDirectory = await resolveCommonDirectory({
    gitDirectory,
    fs,
  },);
  if (commonDirectory === ABSENT)
    return false;
  /**
   Required common-directory signatures.
   */
  const [hasObjects, hasRefs,] = await Promise.all([
    fs.isDirectory(`${commonDirectory}/objects`,),
    fs.isDirectory(`${commonDirectory}/refs`,),
  ],);
  return hasObjects && hasRefs;
}

/**
 Checks whether candidate directory contains a usable `.git` directory or gitfile.
 
 @param dir - candidate worktree root
 
 @param fs - root-discovery filesystem adapter
 
 @returns whether Git would recognize candidate marker
 
 @example
 ```ts
 await matchesValidGitMarker({ dir: '/repo', fs });
 ```
 */
export async function matchesValidGitMarker({
  dir,
  fs,
}: RootMatcherArgs,): Promise<boolean> {
  /**
   Candidate worktree marker path.
   */
  const markerPath = `${dir}/.git`;
  if (await fs.isDirectory(markerPath,))
    return isValidGitDirectory({
      gitDirectory: markerPath,
      fs,
    },);
  if (!await fs.isFile(markerPath,))
    return false;
  /**
   Candidate gitfile content when marker is not a directory.
   */
  const content = await fs.readTextFile(markerPath,);
  if (content === ABSENT)
    return false;
  /**
   Path declared by valid gitfile syntax.
   */
  const gitDirectoryPath = parsePathPayload({
    content,
    prefix: GIT_DIRECTORY_PREFIX,
  },);
  if (gitDirectoryPath === ABSENT)
    return false;
  /**
   Runtime-native absolute administrative directory path.
   */
  const gitDirectory = resolve([
    dir,
    gitDirectoryPath,
  ],);
  return isValidGitDirectory({
    gitDirectory,
    fs,
  },);
}
