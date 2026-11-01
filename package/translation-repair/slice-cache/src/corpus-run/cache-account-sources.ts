import { cacheAccountGitOutput, } from './cache-account-git.ts';

//region Cache account sources
// WHICH SOURCE FILES THE PRE-LAUNCH CACHE CHECK READS (ledger M28). Before the
// translation-repair package was split, every cache version constant lived
// under the one package's `src`, which the check listed with `git ls-files`. A
// constant now lives in whichever package of the family declares it, so a
// package that stands in the family's directory reads the `src` of every
// package beside it, and any other package reads its own.
//
// HISTORY DID NOT START AT THE SPLIT. Until the family was split out, every
// source stood under the one package's `src`, so a version set before the split
// was set at `package/module/translation-repair/src`, and a history query that
// names only the family's `src` takes the commit that moved the file for the
// commit that set it. The history queries add that directory
// (`preSplitSourcesOf`); the listing of tracked files does not, since the files
// stand in the family now.

/**
 Directory from the repository's top that holds the family's packages, ending
 in a slash.
 */
const FAMILY_DIRECTORY = 'package/translation-repair/';

/**
 Directory from the repository's top where every source of the family stood
 before the split, without a trailing slash.
 */
const PRE_SPLIT_SOURCES = 'package/module/translation-repair/src';

/**
 The source files the check reads.
 */
export type AuditedSources = {
  /**
   Pathspec of the source directories as the report names them: the package's
   own `src`, or `src` of every package of the family.
   */
  readonly sources: string;

  /**
   Tracked source files other than tests, as git lists them from the
   repository's top.
   */
  readonly files: readonly string[];
};

/**
 Finds the repository's top directory.

 @param packageDirectory - directory the command runs from, inside the repository

 @returns The repository's top directory

 @example
 ```ts
 const root = await repositoryTopOf({ packageDirectory, },);
 ```
 */
export async function repositoryTopOf({ packageDirectory, }: { readonly packageDirectory: string; },): Promise<string> {
  return (await cacheAccountGitOutput({
    args: [
      '-C',
      packageDirectory,
      'rev-parse',
      '--show-toplevel',
    ],
  },)).trim();
}

/**
 Names the source directories a package reads: its own `src`, or `src` of
 every package of the family where the package stands in the family's
 directory.

 @param prefix - the package's directory from the repository's top, ending in
 a slash, or empty at the top

 @returns Pathspec of the source directories, which the report prints

 @example
 ```ts
 sourcesOfPrefix({ prefix: 'package/translation-repair/stage/', },); // 'package/translation-repair/*\/src'
 ```
 */
export function sourcesOfPrefix({ prefix, }: { readonly prefix: string; },): string {
  return prefix.startsWith(FAMILY_DIRECTORY,) ? `${FAMILY_DIRECTORY}*/src` : `${prefix}src`;
}

/**
 Writes the pathspec that matches every file under the source directories.

 @param sources - pathspec of the source directories, as {@link sourcesOfPrefix} names them

 @returns A pathspec git reads as the directories' whole contents

 @example
 ```ts
 sourcesPathspecOf({ sources: 'package/translation-repair/*\/src', },); // ':(glob)package/translation-repair/*\/src/**'
 ```
 */
export function sourcesPathspecOf({ sources, }: { readonly sources: string; },): string {
  return sources.includes('*',) ? `:(glob)${sources}/**` : sources;
}

/**
 Names the pre-split source directory for a history query to read beside the
 current sources, where those are the family's.

 @param sources - pathspec of the source directories, in the form of {@link sourcesOfPrefix} or of
 {@link sourcesPathspecOf}

 @returns The pre-split directory where the sources are the family's, none where they are one package's own, whose
 sources never stood there

 @example
 ```ts
 preSplitSourcesOf({ sources: 'package/translation-repair/*\/src', },); // ['package/module/translation-repair/src',]
 ```
 */
export function preSplitSourcesOf({ sources, }: { readonly sources: string; },): readonly string[] {
  return sources.includes(FAMILY_DIRECTORY,) ? [PRE_SPLIT_SOURCES,] : [];
}

/**
 Writes the pathspec that leaves the pre-split source directory's tests out.

 @param sources - pathspec of the source directories, in the form of {@link sourcesOfPrefix} or of
 {@link sourcesPathspecOf}

 @returns An exclusion git reads after the source pathspecs where the sources are the family's, none where they are
 one package's own

 @example
 ```ts
 preSplitWithoutTestsOf({ sources: 'src', },); // []
 ```
 */
export function preSplitWithoutTestsOf({ sources, }: { readonly sources: string; },): readonly string[] {
  return preSplitSourcesOf({ sources, },)
    .map(function withoutTests(directory,): string {
      return withoutTestsPathspecOf({ sources: directory, },);
    },);
}

/**
 Writes the pathspec that leaves the source directories' tests out.

 @param sources - pathspec of the source directories, as {@link sourcesOfPrefix} names them

 @returns An exclusion git reads after the source pathspec

 @example
 ```ts
 const withoutTests = withoutTestsPathspecOf({ sources: 'src', },); // ':(exclude,glob)src/**\/*.test.ts'
 ```
 */
export function withoutTestsPathspecOf({ sources, }: { readonly sources: string; },): string {
  return `:(exclude,glob)${sources}/**/*.test.ts`;
}

/**
 Lists the tracked source files the check reads, tests left out.

 @param packageDirectory - directory the command runs from, which the mise
 task makes the package's own

 @returns The pathspec naming the source directories and the files in them

 @example
 ```ts
 const { sources, files, } = await auditedSourcesOf({ packageDirectory, },);
 ```
 */
export async function auditedSourcesOf({ packageDirectory, }: { readonly packageDirectory: string; },): Promise<AuditedSources> {
  /**
   Package's directory from the repository's top.
   */
  const prefix = (await cacheAccountGitOutput({
    args: [
      '-C',
      packageDirectory,
      'rev-parse',
      '--show-prefix',
    ],
  },)).trim();
  /**
   Pathspec of the source directories.
   */
  const sources = sourcesOfPrefix({ prefix, },);
  return {
    sources,
    files: (await cacheAccountGitOutput({
      args: [
        '-C',
        await repositoryTopOf({ packageDirectory, },),
        'ls-files',
        '--',
        sourcesPathspecOf({ sources, },),
        withoutTestsPathspecOf({ sources, },),
      ],
    },))
      .split('\n',)
      .filter(function listed(path,): boolean {
        return path !== '';
      },),
  };
}

//endregion Cache account sources
