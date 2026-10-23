import { homedir, } from 'node:os';
import {
  dirname,
  join,
} from 'node:path';

import type { Logger, } from '@monochromatic-dev/module-logger/ts';
import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  MS_PER_SECOND,
  utcMinutes,
  type VersionSetting,
} from './cache-account-commits.ts';
import {
  type RunsDirSearch,
  runsDirsIn,
  runsDirsUnder,
  type SliceCacheAccount,
  sliceCacheAccount,
  type UnlistedDir,
} from './cache-account-slices.ts';
import { defaultRunsDirIn, } from '@monochromatic-dev/translation-repair-run/ts';

//region Slice cache report
// THE SLICE CACHES' PART OF THE PRE-LAUNCH CACHE CHECK (ledger M57, M81): the
// runs directories read, how many records they hold, the directories the
// search could not list, the newest record and how many versions were set
// after it, which every account stated from a hand-written `find` until then.
// The entry (`cache-account-audit.ts`) prints the version accounts first and
// hands this the settings it read.

/**
 Logger for the report's progress lines, apart from the report on stdout.
 */
const sliceReportLog = contextRoot({ tag: 'cache-account-slice-report', },);

/**
 Words for a zero count, so the absence of records is never read as an empty cache.

 THE ZERO IS NOT A FINDING ABOUT THE CACHES. The audit reads the roots it names, and a launch that follows the
 runbook writes its runs under `temp/agent` of the home directory, which the default roots do not include; the
 lead's run on 2026-10-23 read 0 records there and 13,711 with `--runs-under` naming that directory.

 @param searched - every root the search covered, in the order the account printed them

 @param home - home directory the runbook's runs root sits under

 @returns Sentence, without a prefix, for the log and for stdout alike

 @example
 ```ts
 const words = noRecordWords({ searched: ['/cats/runs',], home: '/cats/home', },);
 ```
 */
function noRecordWords(
  {
    searched,
    home,
  }: {
    readonly searched: readonly string[];
    readonly home: string;
  },
): string {
  /**
   Where the runbook's launches write their runs.
   */
  const runbookRoot = join(
    home,
    'temp',
    'agent',
  );
  return 'no slice-cache record was read, so the newest record and the versions set after it say nothing of the '
    + `caches a launch would resume. Roots searched: ${searched.join(', ',)}. A launch that follows the runbook `
    + `writes its runs under ${runbookRoot}; pass --runs-under ${runbookRoot} after -- to read them.`;
}

/**
 Prints the slice caches' account: the runs directories read, how many
 records they hold (the control), the directories the search could not list,
 the newest record, and how many versions were set after it.

 @param account - what the slice caches hold

 @param searched - where runs directories were looked for, so a reader sees
 which runs the count can include

 @param unlisted - directories the search could not list, whose runs the
 count leaves out

 @param settings - every version with the commit that set it

 @param l - logger the zero-record warning goes to as well

 @param home - home directory the runbook's runs root sits under

 @example
 ```ts
 printSliceCacheAccount({ account, searched, unlisted, settings, l, home, },);
 ```
 */
function printSliceCacheAccount(
  {
    account,
    searched,
    unlisted,
    settings,
    l,
    home,
  }: {
    readonly account: SliceCacheAccount;
    readonly searched: readonly string[];
    readonly unlisted: readonly UnlistedDir[];
    readonly settings: readonly VersionSetting[];
    readonly l: Logger;
    readonly home: string;
  },
): void {
  /**
   Runs directories read, records read, and the one written last.
   */
  const {
    runsDirs,
    count,
    newest,
  } = account;
  console.log(
    `slice-cache records under ${String(runsDirs.length,)} ${wordForCount({
      count: runsDirs.length,
      one: 'runs directory',
      many: 'runs directories',
    },)} found in ${searched.join(', ',)}: ${String(count,)}`,
  );
  console.log(
    `directories the search could not list, whose runs the count leaves out: ${String(unlisted.length,)}`,
  );
  for (const {
    dir,
    reason,
  } of unlisted)
    console.log(`  ${dir} (${reason})`,);
  if (count === 0) {
    /**
     What a zero count does and does not say.
     */
    const words = noRecordWords({
      searched,
      home,
    },);
    l.warn(words,);
    console.log(`WARNING: ${words}`,);
  }
  if (newest.kind === 'none') {
    console.log('newest slice-cache record: none',);
    return;
  }
  /**
   The record written last.
   */
  const { record, } = newest;
  /**
   Its time in whole seconds, as commit times are kept.
   */
  const newestSeconds = Math.floor(record.modifiedMs / MS_PER_SECOND,);
  /**
   Versions whose current value was set after it.
   */
  const setAfter = settings.filter(function laterThanNewest({ commit, },): boolean {
    return commit.seconds > newestSeconds;
  },);
  console.log(`newest slice-cache record: ${utcMinutes({ seconds: newestSeconds, },)} ${record.path}`,);
  console.log(`cache versions set after it: ${String(setAfter.length,)} of ${String(settings.length,)}`,);
}

/**
 Reads the slice caches under every runs directory the audit can name and
 prints their account.

 Returns nothing: the report on stdout IS the output.

 @param root - repository's top directory, whose dependency tree holds the
 default runs directory

 @param runsDir - the runs directory a pass would use now, which
 `TRANSLATION_REPAIR_RUNS_DIR` may set anywhere

 @param searched - directories named on the command line to search for runs
 directories beside the worktree's own

 @param settings - every version with the commit that set it

 @param l - logger for the progress lines and the zero-record warning, the audit's own where absent

 @param home - home directory the runbook's runs root sits under, the process's own where absent

 @throws What a listing or `stat` raises inside a runs directory for any
 failure but an absent path

 @example
 ```ts
 await reportSliceCaches({ root, runsDir, searched: line.list('runs-under',), settings, },);
 ```
 */
export async function reportSliceCaches(
  {
    root,
    runsDir,
    searched,
    settings,
    l = sliceReportLog,
    home = homedir(),
  }: {
    readonly root: string;
    readonly runsDir: string;
    readonly searched: readonly string[];
    readonly settings: readonly VersionSetting[];
    readonly l?: Logger;
    readonly home?: string;
  },
): Promise<void> {
  /**
   Directory the default runs directory sits in, where hand-set ones beside
   it carry its name as a prefix.
   */
  const runsParent = dirname(defaultRunsDirIn({ worktreeRoot: root, },),);
  /**
   What the search found under each searched directory.
   */
  const searches = await Promise.all(searched.map(async function runsUnder(searchRoot,): Promise<RunsDirSearch> {
    return await runsDirsUnder({ root: searchRoot, },);
  },),);
  /**
   Runs directories found under them, beside the worktree's own and the
   one a pass would use now, each once.
   */
  const runsDirs = [...new Set([
    ...await runsDirsIn({ parent: runsParent, },),
    runsDir,
    ...searches.flatMap(function foundIn({ found, },): readonly string[] {
      return found;
    },),
  ],),];
  /**
   Directories the searches could not list.
   */
  const unlisted = searches.flatMap(function unlistedIn({ unlisted: skipped, },): readonly UnlistedDir[] {
    return skipped;
  },);
  l.info(
    `runs directories to read: ${String(runsDirs.length,)}; directories not listed: ${String(unlisted.length,)}`,
  );
  /**
   What their slice caches hold.
   */
  const account = await sliceCacheAccount({ runsDirs, },);
  l.info(`slice-cache records read: ${String(account.count,)}`,);
  printSliceCacheAccount({
    account,
    searched: [
      runsParent,
      runsDir,
      ...searched,
    ],
    unlisted,
    settings,
    l,
    home,
  },);
}

//endregion Slice cache report
