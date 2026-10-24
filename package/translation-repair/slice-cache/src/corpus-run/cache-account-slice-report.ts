import { homedir, } from 'node:os';
import {
  dirname,
  join,
} from 'node:path';

import { caughtValueText, } from '@monochromatic-dev/module-caught-value/ts';
import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import { failureName, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  contextRoot,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { searchReaches, } from './cache-account-search-reach.ts';
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
 How many levels below the runbook's runs root its runs directories sit, at the most: `<root>/<run>` and
 `<root>/<run>/runs`.
 */
const RUNBOOK_RUNS_LEVELS = 2;

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
 A searched root whose real path could not be resolved, and the filesystem's code.

 @example
 ```ts
 const unresolved: UnresolvedRoot = { root: '/cats/file/below', code: 'ENOTDIR', };
 ```
 */
type UnresolvedRoot = {
  /**
   The root as it was given.
   */
  readonly root: string;

  /**
   Filesystem code, such as `ENOTDIR`, or the class name of a failure that carries none.
   */
  readonly code: string;
};

/**
 What the check found for one searched root.

 @example
 ```ts
 const reading: RootReading = { kind: 'unresolved', root: '/cats/sealed/below', code: 'EACCES', };
 ```
 */
type RootReading =
  | {
    /**
     The root reaches the runbook's runs root.
     */
    readonly kind: 'reached';
  }
    | {
    /**
     The root was resolved and does not reach it.
     */
      readonly kind: 'not-reached';
    }
    | {
    /**
     The root could not be resolved, so it counts as not reaching it.
     */
      readonly kind: 'unresolved';

    /**
     The root as it was given.
     */
      readonly root: string;

    /**
     Filesystem code of the failure, or its class name where it carries none.
     */
      readonly code: string;
    };

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
 Whether one searched root reaches the runbook's runs root, a root the filesystem refuses to resolve counting as not.

 THE REFUSAL DOES NOT END THE AUDIT. The walk reports a directory it cannot list and goes on, and a root named on the
 command line can sit behind a file, a loop of links or a directory the process may not enter; the check logs the
 failure with the root and says so in the warning, so the newest record and the versions set after it are still
 printed.

 @param root - searched root

 @param runbookRoot - where a launch that follows the runbook writes its runs

 @param l - logger the refusal is written to

 @returns That the root reaches it, that it does not, or that it could not be resolved and why

 @example
 ```ts
 const reading = await reachOfRoot({ root: '/cats/runs', runbookRoot: '/cats/home/temp/agent', l, },);
 ```
 */
async function reachOfRoot(
  {
    root,
    runbookRoot,
    l,
  }: {
    readonly root: string;
    readonly runbookRoot: string;
    readonly l: Logger;
  },
): Promise<RootReading> {
  /**
   Logger tagged with this function's name.
   */
  const rl = tagged({
    l,
    tag: reachOfRoot.name,
  },);
  try {
    if (await searchReaches({
      root,
      directory: runbookRoot,
      runsLevels: RUNBOOK_RUNS_LEVELS,
    },))
      return { kind: 'reached', };
    return { kind: 'not-reached', };
  }
  catch (error) {
    /**
     The filesystem's code for the refusal, which the warning on stdout names.
     */
    const code = failureName({ error, },);
    // THE LOG LINE KEEPS THE FAILURE'S TEXT, which the warning on stdout leaves out: the text of a failed real-path
    // lookup names the path it stopped at, an ancestor of the root, and so tells which file or link on the way is to
    // blame. The line already prints the root and the runbook's runs root, so the text discloses no path the line
    // does not hold. The logger takes one string, so the caught value is rendered into it.
    rl.warn(
      `${root} could not be compared with the runbook's runs root ${runbookRoot} (${code}); `
        + `it counts as not reaching it. The failure was: ${caughtValueText(error,)}`,
    );
    return {
      kind: 'unresolved',
      root,
      code,
    };
  }
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
   What the check found for each root searched, in the order searched.
   */
  const readings = await allInInputOrder({
    members: searched.map(async function readReachOf(root,): Promise<RootReading> {
      return await reachOfRoot({
        root,
        runbookRoot,
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
