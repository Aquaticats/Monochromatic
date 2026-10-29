import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  contextRoot,
  isMissingPathError,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  CACHE_ACCOUNT_COMMIT_FORMAT,
  cacheAccountCommitsOf,
  cacheAccountGitOutput,
} from './cache-account-git.ts';
import { printCacheAudit, } from './cache-account-print.ts';
import {
  type CacheVersion,
  cacheVersionsIn,
} from './cache-account-read.ts';
import { cacheVersionSetting, } from './cache-account-setting.ts';
import {
  auditedSourcesOf,
  preSplitSourcesOf,
  preSplitWithoutTestsOf,
  repositoryTopOf,
  sourcesPathspecOf,
  withoutTestsPathspecOf,
} from './cache-account-sources.ts';
import type { VersionSetting, } from './cache-account-commits.ts';
import { reportSliceCaches, } from './cache-account-slice-report.ts';
import {
  type CommandLineOf,
  taskPathOf,
} from '@monochromatic-dev/translation-repair-run/ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Cache account run
// THE PRE-LAUNCH CACHE VERSION CHECK (ledger M28), read-only: git log, git
// show and file reads, no write and no model. For every cache version constant
// the source declares, it finds the commit that set the current value, then
// lists every non-test source commit since the earliest of those that no
// version's account names, with the versions each one postdates. Each such
// commit rides inside those versions, and the check reads it and either says
// so in the account or moves the version.
//
// THEN THE SLICE CACHES (ledger M57, M81), in `cache-account-slice-report.ts`:
// the runs directories read, how many records they hold, the newest record and
// how many versions were set after it, which every account stated from a
// hand-written `find` until then.
//
// MOVED IN FROM A SCRATCH SCRIPT the accounts cited and nobody else could run.
// That script walked a list of seven constants typed in by hand, which is
// the M28 mistake waiting to recur, and named the worktree by an absolute
// path; this reads the constants out of the source and finds the repository
// from where it is run.
//
// RUN FROM THE PACKAGE DIRECTORY, which the mise task does:
// `mise run //package/translation-repair/slice-cache:cache-account-audit`.
// A package in the family's directory reads the `src` of every package of the
// family (`cache-account-sources.ts`), since a cache version lives in
// whichever package declares it. Prints hashes, times and commit subjects,
// which are this repository's own.


/**
 Logger for the audit's progress lines, apart from the report on stdout.
 */
const auditLog = contextRoot({ tag: 'cache-account-audit', },);

/**
 Reads one tracked source file out of the working tree.

 @param root - repository's top directory

 @param path - file's path from that directory, as git lists it

 @returns File's text

 @throws StatedRefusalError where git lists the file and the working tree
 holds none, which is a removal not yet committed

 @example
 ```ts
 const text = await sourceTextAt({ root, path: 'src/nap.ts', },);
 ```
 */
async function sourceTextAt(
  {
    root,
    path,
  }: {
    readonly root: string;
    readonly path: string;
  },
): Promise<string> {
  try {
    return await readFile(
      join(
        root,
        path,
      ),
      'utf8',
    );
  }
  catch (error) {
    if (!isMissingPathError({ error, },)) {
      throw error;
    }
    throw new StatedRefusalError({
      says: `${path} is tracked but missing from the working tree; restore it or commit its removal, then run the audit again.`,
      cause: error,
    },);
  }
}

/**
 Reads every cache version the package's source declares and prints the
 source commits no version account names, then the slice caches' account.

 Returns nothing: the report on stdout IS the output.

 @param line - command line, whose `--runs-under` names directories to
 search for runs directories beside the worktree's own

 @param packageDirectory - directory the command runs from, which the mise task
 makes the package's own; the audit reads that package's `src`, or the `src` of
 every package of the family where the package stands in the family's directory

 @param runsDir - runs directory a pass would use now, which
 `TRANSLATION_REPAIR_RUNS_DIR` may set anywhere

 @param home - home directory the runbook's runs root sits under, the process's own where absent

 @throws StatedRefusalError where the working directory holds no cache
 version, a value is uncommitted, or a tracked file is missing from the working
 tree

 @example
 ```ts
 await auditCacheAccounts({ line, packageDirectory, runsDir, },);
 ```
 */
export async function auditCacheAccounts(
  {
    line,
    packageDirectory,
    runsDir,
    home,
  }: {
    readonly line: CommandLineOf<'cache-account-audit'>;
    readonly packageDirectory: string;
    readonly runsDir: string;
    readonly home?: string;
  },
): Promise<void> {
  /**
   Repository's top directory.
   */
  const root = await repositoryTopOf({ packageDirectory, },);
  /**
   Pathspec of the source directories, and the tracked files other than tests
   in them.
   */
  const {
    sources,
    files,
  } = await auditedSourcesOf({ packageDirectory, },);
  /**
   Pathspec matching every file under the source directories.
   */
  const sourcesPathspec = sourcesPathspecOf({ sources, },);
  /**
   Pathspec leaving the source's tests out.
   */
  const withoutTests = withoutTestsPathspecOf({ sources, },);
  /**
   Each file's text, by path.
   */
  const entries = await allInInputOrder({
    members: files.map(async function read(path,): Promise<readonly [
      string,
      string,
    ]> {
      return [
        path,
        await sourceTextAt({
          root,
          path,
        },),
      ] as const;
    },),
  },);
  /**
   The same, looked up by path.
   */
  const texts = new Map(entries,);
  /**
   Every cache version constant the source declares.
   */
  const versions = [...texts,].flatMap(function declared([
    path,
    text,
  ]: ReadonlyEntry<string, string>,): readonly CacheVersion[] {
    return cacheVersionsIn({
      path,
      text,
    },);
  },);
  auditLog.info(
    `${String(files.length,)} source ${
      wordForCount({
        count: files.length,
        one: 'file',
        many: 'files',
      },)
    } under ${sources}, ${String(versions.length,)} cache ${
      wordForCount({
        count: versions.length,
        one: 'version',
        many: 'versions',
      },)
    }`,
  );
  if (versions.length === 0) {
    throw new StatedRefusalError({
      says: `no cache version is declared under ${sources}; run this from the package directory, as `
        + `mise run ${taskPathOf({ command: 'cache-account-audit', },)} does.`,
    },);
  }

  /**
   Every version with the commit that set it and the text its account is in.
   */
  const settings: readonly VersionSetting[] = await allInInputOrder({
    members: versions.map(async function set(version,): Promise<VersionSetting> {
      return {
        version,
        commit: await cacheVersionSetting({
          root,
          sources: sourcesPathspec,
          version,
        },),
        account: nonNullishOrThrow(texts.get(version.path,),),
      };
    },),
  },);
  /**
   The setting furthest back, from which every later source commit is read;
   there is one, since a version was declared.
   */
  const earliest = nonNullishOrThrow(settings.toSorted(function older(
    { commit: left, },
    { commit: right, },
  ): number {
    return left.seconds - right.seconds;
  },)[0],);
  /**
   Commit it was made in.
   */
  const { commit: earliestCommit, } = earliest;
  /**
   Non-test source commits since the earliest setting, newest first.
   */
  const commits = cacheAccountCommitsOf({
    output: await cacheAccountGitOutput({
      args: [
        '-C',
        root,
        'log',
        CACHE_ACCOUNT_COMMIT_FORMAT,
        `${earliestCommit.hash}..HEAD`,
        '--',
        sourcesPathspec,
        ...preSplitSourcesOf({ sources, },),
        withoutTests,
        ...preSplitWithoutTestsOf({ sources, },),
      ],
    },),
  },);
  printCacheAudit({
    sources,
    settings,
    earliest,
    commits,
  },);
  await reportSliceCaches({
    root,
    runsDir,
    searched: line.list('runs-under',),
    settings,
    ...((home === undefined) ? {} : { home, }),
  },);
}

//endregion Cache account run
