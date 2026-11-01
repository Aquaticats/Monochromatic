import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type SourceCommit,
  settingCommit,
} from './cache-account-commits.ts';
import {
  type CacheVersion,
  citedHash,
  declarationLineCounts,
} from './cache-account-read.ts';
import {
  CACHE_ACCOUNT_COMMIT_FORMAT,
  cacheAccountCommitsOf,
  cacheAccountGitOutput,
} from './cache-account-git.ts';
import { preSplitSourcesOf, } from './cache-account-sources.ts';

//region Cache account setting
// WHICH COMMIT SET A CACHE VERSION'S CURRENT VALUE (ledger M28): the newest
// commit that added its declaration on balance, found by git's pickaxe and
// then read against each candidate's diff. The pickaxe reads merges too
// (ledger B814): a merge that meets two changes taking one number writes the
// next in its resolution, and no ordinary commit holds that value.

/**
 Logger for the audit's progress lines, apart from the report on stdout.
 */
const auditLog = contextRoot({ tag: 'cache-account-audit', },);

/**
 Keeps each commit once, where it first stands: `git log -m` names a merge once
 for each parent its diff against differs.

 @param commits - commits as git printed them

 @returns Them in the same order, each hash once

 @example
 ```ts
 commitsOnceEach({ commits: [merge, merge, side,], },); // [merge, side]
 ```
 */
function commitsOnceEach({ commits, }: { readonly commits: readonly SourceCommit[]; },): readonly SourceCommit[] {
  /**
   Each hash with its commit; a map keeps where a key was first set, and a later naming carries the same commit.
   */
  const byHash = new Map(commits.map(function keyed(commit,): readonly [
    string,
    SourceCommit,
  ] {
    return [
      commit.hash,
      commit,
    ];
  },),);
  return Array.from(byHash.values(),);
}

/**
 Finds the commit that set one version's current value.

 @param root - repository's top directory

 @param sources - pathspec of the package's source; where it is the family's, the pre-split source directory is
 read beside it, so a version set before the split is found at its old commit and not at the move

 @param version - constant as the source declares it

 @returns That commit

 @throws StatedRefusalError where no commit adds the current declaration,
 which is an uncommitted value

 @example
 ```ts
 const commit = await cacheVersionSetting({ root, sources, version, },);
 ```
 */
export async function cacheVersionSetting(
  {
    root,
    sources,
    version,
  }: {
    readonly root: string;
    readonly sources: string;
    readonly version: CacheVersion;
  },
): Promise<SourceCommit> {
  /**
   Commits whose diffs change how often the declaration appears, newest first, merges among them; git names a
   merge once for each parent its diff against differs, so only its first naming is kept.
   A longer name or value holding it as a substring is sorted out by `declarationLineCounts` and `settingCommit`,
   as is a merge that only brought in what a side commit set, since its combined diff writes nothing of its own.
   */
  const commits = commitsOnceEach({
    commits: cacheAccountCommitsOf({
      output: await cacheAccountGitOutput({
        args: [
          '-C',
          root,
          'log',
          '-m',
          CACHE_ACCOUNT_COMMIT_FORMAT,
          '-S',
          version.declaration,
          '--',
          sources,
          ...preSplitSourcesOf({ sources, },),
        ],
      },),
    },),
  },);
  /**
   Each with the declarations of this value it added and removed.
   */
  const candidates = await allInInputOrder({
    members: commits.map(async function counted(commit,) {
      /**
       The commit's changes to the source, without context lines.
       */
      const diff = await cacheAccountGitOutput({
        args: [
          '-C',
          root,
          'show',
          '--format=',
          '--unified=0',
          commit.hash,
          '--',
          sources,
          ...preSplitSourcesOf({ sources, },),
        ],
      },);
      return {
        commit,
        ...declarationLineCounts({
          diff,
          version,
        },),
      };
    },),
  },);
  /**
   Newest commit that added the declaration on balance.
   */
  const setting = settingCommit({ candidates, },);
  if (setting.kind === 'uncommitted') {
    throw new StatedRefusalError({
      says: `${version.declaration} in ${version.path} is not in any commit; commit it, then run the audit again.`,
    },);
  }
  /**
   That commit.
   */
  const { commit: setAt, } = setting;
  auditLog.info(
    `${version.declaration}: set in ${citedHash({ hash: setAt.hash, },)}, `
      + `${String(commits.length,)} pickaxe ${
        wordForCount({
          count: commits.length,
          one: 'candidate',
          many: 'candidates',
        },)
      }`,
  );
  return setAt;
}

//endregion Cache account setting
