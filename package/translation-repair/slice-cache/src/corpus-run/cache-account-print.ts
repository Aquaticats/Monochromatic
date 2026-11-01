import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type SourceCommit,
  unaccountedCommits,
  utcMinutes,
  type VersionSetting,
} from './cache-account-commits.ts';
import { citedHash, } from './cache-account-read.ts';

//region Cache account print
// HOW THE PRE-LAUNCH CACHE CHECK READS ON A TERMINAL (ledger M28): hashes,
// times and commit subjects, which are this package's own.

/**
 Prints the report: each version's setting, then the commits no account
 names.

 @param sources - pathspec of the package's source

 @param settings - every version with the commit that set it

 @param earliest - setting furthest back

 @param commits - non-test source commits since it

 @example
 ```ts
 printCacheAudit({ sources, settings, earliest, commits, },);
 ```
 */
export function printCacheAudit(
  {
    sources,
    settings,
    earliest,
    commits,
  }: {
    readonly sources: string;
    readonly settings: readonly VersionSetting[];
    readonly earliest: VersionSetting;
    readonly commits: readonly SourceCommit[];
  },
): void {
  /**
   Commits no account names.
   */
  const unnamed = unaccountedCommits({
    settings,
    commits,
  },);
  /**
   Commit the earliest setting was made in.
   */
  const { commit: earliestCommit, } = earliest;
  console.log(
    `cache-account-audit: ${String(settings.length,)} cache ${
      wordForCount({
        count: settings.length,
        one: 'version',
        many: 'versions',
      },)
    } under ${sources}`,
  );
  for (const {
    version,
    commit,
  } of settings) {
    console.log(
      `  ${version.declaration} (${version.path}): set in ${citedHash({ hash: commit.hash, },)} `
        + `at ${utcMinutes({ seconds: commit.seconds, },)}`,
    );
  }
  console.log(
    `source commits since ${citedHash({ hash: earliestCommit.hash, },)}: ${String(commits.length,)}, `
      + `named by an account: ${String(commits.length - unnamed.length,)}, by none: ${String(unnamed.length,)}`,
  );
  for (const {
    commit,
    setBefore,
  } of unnamed) {
    console.log(
      `  ${citedHash({ hash: commit.hash, },)} ${utcMinutes({ seconds: commit.seconds, },)} `
        + `[rides inside: ${setBefore.join(', ',)}] ${commit.subject}`,
    );
  }
}

//endregion Cache account print
