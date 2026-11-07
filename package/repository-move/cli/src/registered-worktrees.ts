/**
 The worktrees Git registers below the main checkout, each in its resolved spelling beside the spelling Git printed, and
 the text a stop prints when the Git registry and the filesystem's subvolume list disagree.

 @module
 */

import { execFile, } from 'node:child_process';
import { promisify, } from 'node:util';
import { canonicalPath, } from './canonical-path.ts';
import { MoveStopError, } from './errors.ts';

/**
 A folder named in its canonical spelling, with the spelling the other side gave it when that differs.
 */
export type SpelledPath = {
  readonly canonical: string;
  readonly given: string;
};

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 The generic `promisify(execFile)` form trips `strict-void-return`;
 see `doc/troubleshooting/oxlint-promisify-void-return.md`.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program and waits for it to end.
 */
const execFileAsync = promisifyExecFile(execFile,);

/**
 Git's own program, never the repository's wrapper.
 */
const GIT = '/usr/bin/git';

/**
 Names a folder for a person: the canonical path, plus the spelling it was given in when that is a different text.

 @param spelled - the folder in both spellings

 @param label - what the other spelling is called (`registered as`, `listed as`)

 @returns the text to print

 @example
 ```ts
 import { describeFolder, } from './registered-worktrees.ts';

 describeFolder({
   spelled: { canonical: '/var/mnt/encrypted/temp/a', given: '/var/home/user/temp/a', },
   label: 'listed as',
 },);
 // '/var/mnt/encrypted/temp/a (listed as /var/home/user/temp/a)'
 ```
 */
export function describeFolder(
  {
    spelled,
    label,
  }: {
    readonly spelled: SpelledPath;
    readonly label: string;
  },
): string {
  return spelled.canonical === spelled.given ? spelled.canonical : `${spelled.canonical} (${label} ${spelled.given})`;
}

/**
 Lists the worktrees Git registers below the main checkout, each in its canonical spelling beside the one Git printed.

 Git records a worktree by its resolved path, while the caller may name the main checkout through a link, so both sides
 are resolved before they are compared.

 @param mainPath - the main checkout, in any spelling

 @returns the worktrees in Git's order

 @throws MoveStopError when Git cannot list them

 @example
 ```ts
 import { listRegisteredWorktreeSpellings, } from './registered-worktrees.ts';

 const worktrees = await listRegisteredWorktreeSpellings('/var/home/user/Monochromatic',);
 ```
 */
export async function listRegisteredWorktreeSpellings(mainPath: string,): Promise<readonly SpelledPath[]> {
  try {
    /**
     The output of the listing command.
     */
    const listing = await execFileAsync(
      GIT,
      [
        '--no-optional-locks',
        '-C',
        mainPath,
        'worktree',
        'list',
        '--porcelain',
      ],
      { encoding: 'utf8', },
    );
    /**
     The main checkout in its canonical spelling.
     */
    const canonicalMain = await canonicalPath(mainPath,);
    /**
     The worktrees as Git printed them, each with its canonical spelling.
     */
    const spelled = await Promise.all(listing.stdout
      .split('\n',)
      .filter(function isWorktreeLine(line,) {
        return line.startsWith('worktree ',);
      },)
      .map(async function spell(line,): Promise<SpelledPath> {
        /**
         The path as Git printed it.
         */
        const given = line.slice('worktree '.length,);
        return {
          canonical: await canonicalPath(given,),
          given,
        };
      },),);
    return spelled.filter(function isBelowMain(entry,) {
      return entry.canonical
        .startsWith(`${canonicalMain}/`,);
    },);
  }
  catch (error) {
    throw new MoveStopError(
      `cannot list worktrees below ${mainPath}`,
      { cause: error, },
    );
  }
}

/**
 The stop text for an inventory that differs from the Git registry.

 @param unknownToGit - canonical paths the inventory lists and Git does not register

 @param notSubvolumes - canonical paths Git registers and the inventory does not list

 @param listedAs - the spelling the caller's mapping gave each inventory path, by canonical path

 @param registeredAs - the spelling Git printed for each registered path, by canonical path

 @returns the message, naming every path in both spellings where they differ

 @example
 ```ts
 import { describeDisagreement, } from './registered-worktrees.ts';

 const message = describeDisagreement({
   unknownToGit: ['/a/hidden'],
   notSubvolumes: [],
   listedAs: new Map([['/a/hidden', '/link/hidden',],],),
   registeredAs: new Map(),
 },);
 ```
 */
export function describeDisagreement(
  {
    unknownToGit,
    notSubvolumes,
    listedAs,
    registeredAs,
  }: {
    readonly unknownToGit: readonly string[];
    readonly notSubvolumes: readonly string[];
    readonly listedAs: ReadonlyMap<string, string>;
    readonly registeredAs: ReadonlyMap<string, string>;
  },
): string {
  /**
   The paths Git does not know, each named in both spellings.
   */
  const unknown = unknownToGit.map(function describe(path,) {
    return describeFolder({
      spelled: {
        canonical: path,
        given: listedAs.get(path,) ?? path,
      },
      label: 'listed as',
    },);
  },);
  /**
   The registered paths that are not subvolumes, each named in both spellings.
   */
  const unlisted = notSubvolumes.map(function describe(path,) {
    return describeFolder({
      spelled: {
        canonical: path,
        given: registeredAs.get(path,) ?? path,
      },
      label: 'registered as',
    },);
  },);
  return `inventory differs from the Git registry; subvolumes Git does not know: ${unknown.join(' ',) || 'none'}; `
    + `registered but not subvolumes: ${unlisted.join(' ',) || 'none'}. Give each an owner-approved disposition first.`;
}
