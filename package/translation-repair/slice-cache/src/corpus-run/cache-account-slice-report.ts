import { homedir, } from 'node:os';
import {
  dirname,
  join,
} from 'node:path';

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  reachOfRoot,
  type RootReading,
  RUNBOOK_RUNS_LEVELS,
  runbookRootReading,
  type UnresolvedRoot,
} from './cache-account-search-reach.ts';
import {
  MS_PER_SECOND,
  utcMinutes,
  type VersionSetting,
} from './cache-account-commits.ts';
import {
  RUNS_SEARCH_DEPTH,
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
// hands this the settings it read. A WARNING FOLLOWS WHENEVER THE RUNBOOK'S RUNS ROOT IS NOT REACHED by the roots
// searched, whatever the count: a stale record under the default roots gives a non-zero count that says nothing of the
// runs a launch that follows the runbook writes. A root reaches it by real path, within the depth the search walks.

/**
 Logger for the report's progress lines, apart from the report on stdout.
 */
const sliceReportLog = contextRoot({ tag: 'cache-account-slice-report', },);

/**
 Where a launch that follows the runbook writes its runs.

 @param home - home directory the runbook's runs root sits under

 @returns Runbook's runs root

 @example
 ```ts
 const root = runbookRootOf({ home: '/cats/home', },); // '/cats/home/temp/agent'
 ```
 */
function runbookRootOf({ home, }: { readonly home: string; },): string {
  return join(
    home,
    'temp',
    'agent',
  );
}

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
  const runbookRoot = runbookRootOf({ home, },);
  return 'no slice-cache record was read, so the newest record and the versions set after it say nothing of the '
    + `caches a launch would resume. Roots searched: ${searched.join(', ',)}. A launch that follows the runbook `
    + `writes its runs under ${runbookRoot}; pass --runs-under ${runbookRoot} after -- to read them.`;
}

/**
 The clause that names the roots the check could not resolve, which the warning ends with.

 @param unresolved - roots the check could not resolve to a real path, each with the filesystem's code

 @returns Clause with its leading space, or nothing where every root resolved

 @example
 ```ts
 const words = unresolvedRootWords({ unresolved: [{ root: '/cats/file/below', code: 'ENOTDIR', },], },);
 ```
 */
function unresolvedRootWords({ unresolved, }: { readonly unresolved: readonly UnresolvedRoot[]; },): string {
  if (unresolved.length === 0)
    return '';
  /**
   Each root with its code.
   */
  const named = unresolved.map(function withCode(
    {
      root,
      code,
    },
  ): string {
    return `${root} (${code})`;
  },);
  return ` Roots the check could not resolve, and so counted as not reaching it: ${named.join(', ',)}.`;
}

/**
 Words for records read when no searched root reaches the runbook's runs root.

 THE COUNT IS TRUE OF THE ROOTS SEARCHED AND SAYS NOTHING OF THE REST. A stale record under the default roots gives a
 non-zero count while the runs of a launch that follows the runbook, under `temp/agent` of the home directory, were
 never read.

 @param searched - every root the search covered, in the order the account printed them

 @param runbookRoot - where a launch that follows the runbook writes its runs

 @param unresolved - roots the check could not resolve to a real path, each with the filesystem's code

 @returns Sentence, without a prefix, for the log and for stdout alike

 @example
 ```ts
 const words = uncoveredRootWords({
   searched: ['/cats/runs',],
   runbookRoot: '/cats/home/temp/agent',
   unresolved: [],
 },);
 ```
 */
function uncoveredRootWords(
  {
    searched,
    runbookRoot,
    unresolved,
  }: {
    readonly searched: readonly string[];
    readonly runbookRoot: string;
    readonly unresolved: readonly UnresolvedRoot[];
  },
): string {
  /**
   The sentence about the roots that were read.
   */
  const reached = `the runbook's runs root ${runbookRoot} is not reached by the roots searched: `
    + `${searched.join(', ',)}. A search reaches runs directories at most ${String(RUNS_SEARCH_DEPTH,)} levels below `
    + `a root, and the runbook's runs sit up to ${String(RUNBOOK_RUNS_LEVELS,)} levels below its root; `
    + 'it enters no hidden directory or node_modules, and a directory counts by its real path. The newest record and '
    + 'the versions set after it speak only for those roots, '
    + `and a launch that follows the runbook writes its runs under ${runbookRoot}; pass --runs-under ${runbookRoot} `
    + 'after -- to read them.';
  return `${reached}${unresolvedRootWords({ unresolved, },)}`;
}

/**
 Words for records read when the runbook's runs root cannot be resolved, so no searched root can be compared with it.

 THE FAILURE IS THE RUNBOOK ROOT'S, NOT THE SEARCHED ROOTS'. Comparing a root with the runbook's runs root resolves
 both, so a runbook root behind a file, a loop of links or a directory nobody may enter fails every comparison; the
 warning names that one failure and does not blame the roots searched.

 @param searched - every root the search covered, in the order the account printed them

 @param runbookRoot - where a launch that follows the runbook writes its runs

 @param code - filesystem code of the failure

 @returns Sentence, without a prefix, for the log and for stdout alike

 @example
 ```ts
 const words = unresolvedRunbookRootWords({
   searched: ['/cats/runs',],
   runbookRoot: '/cats/file/temp/agent',
   code: 'ENOTDIR',
 },);
 ```
 */
function unresolvedRunbookRootWords(
  {
    searched,
    runbookRoot,
    code,
  }: {
    readonly searched: readonly string[];
    readonly runbookRoot: string;
    readonly code: string;
  },
): string {
  return `the runbook's runs root ${runbookRoot} could not be resolved (${code}), so none of the roots searched could `
    + `be compared with it: ${searched.join(', ',)}. The newest record and the versions set after it speak only for `
    + `those roots, and a launch that follows the runbook writes its runs under ${runbookRoot}; pass --runs-under `
    + `${runbookRoot} after -- to read them.`;
}

/**
 What the account's count does not say, for the warning that follows it.

 The empty case keeps its own sentence; a count above zero warns only when no searched root reaches the runbook's runs
 root (`searchReaches`).

 @param count - records read

 @param searched - every root the search covered, in the order the account printed them

 @param home - home directory the runbook's runs root sits under

 @param l - logger the roots the check could not resolve are written to

 @returns Sentence alone, or none where the count says all it should

 @example
 ```ts
 const words = await warningWordsOf({ count: 1, searched: ['/cats/runs',], home: '/cats/home', l, },);
 ```
 */
async function warningWordsOf(
  {
    count,
    searched,
    home,
    l,
  }: {
    readonly count: number;
    readonly searched: readonly string[];
    readonly home: string;
    readonly l: Logger;
  },
): Promise<readonly string[]> {
  if (count === 0) {
    return [
      noRecordWords({
        searched,
        home,
      },),
    ];
  }
  /**
   Where a launch that follows the runbook writes its runs.
   */
  const runbookRoot = runbookRootOf({ home, },);
  /**
   Logger tagged with this function's name.
   */
  const wl = tagged({
    l,
    tag: warningWordsOf.name,
  },);
  /**
   What the check found for the runbook's runs root, resolved once for every root searched.
   */
  const runbookReading = await runbookRootReading({
    runbookRoot,
    l: wl,
  },);
  if (runbookReading.kind === 'unresolved') {
    return [
      unresolvedRunbookRootWords({
        searched,
        runbookRoot,
        code: runbookReading.code,
      },),
    ];
  }
  /**
   What the check found for each root searched, in the order searched.
   */
  const readings = await allInInputOrder({
    members: searched.map(async function readReachOf(root,): Promise<RootReading> {
      return await reachOfRoot({
        root,
        runbookRoot,
        runbookReal: runbookReading.real,
        l: wl,
      },);
    },),
  },);
  if (readings.some(function reaches({ kind, },): boolean {
    return kind === 'reached';
  },))
    return [];
  return [
    uncoveredRootWords({
      searched,
      runbookRoot,
      unresolved: readings.flatMap(function unresolvedIn(reading,): readonly UnresolvedRoot[] {
        if (reading.kind !== 'unresolved')
          return [];
        return [{
          root: reading.root,
          code: reading.code,
        },];
      },),
    },),
  ];
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

 @param l - logger the warnings go to as well

 @param home - home directory the runbook's runs root sits under

 @example
 ```ts
 await printSliceCacheAccount({ account, searched, unlisted, settings, l, home, },);
 ```
 */
async function printSliceCacheAccount(
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
): Promise<void> {
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
  /**
   What the count does not say, none where it says all it should.
   */
  const warnings = await warningWordsOf({
    count,
    searched,
    home,
    l,
  },);
  for (const words of warnings) {
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

 @param runsDir - runs directory a pass would use now, which
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
  await printSliceCacheAccount({
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
