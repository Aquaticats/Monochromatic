import {
  mkdir,
  mkdtemp,
  mkdtempDisposable,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { pathToFileURL, } from 'node:url';

import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { packageCacheDir, } from '@monochromatic-dev/translation-repair-reference/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { CommandLineOf, } from '@monochromatic-dev/translation-repair-run/ts';
import type { Baseline, } from './coverage-census-baseline-lines.ts';
import { requireCoverageBuild, } from './coverage-census-build.ts';
import {
  type CensusFamily,
  isFamilyBundleName,
  familySuiteRunsOf,
  type FamilySuiteRun,
  requireFamilyCoverageBuild,
} from './coverage-census-family.ts';
import { readUtf8Text, } from './coverage-census-read-text.ts';
import type {
  packageCommit,
  sourcesEditedSince,
} from './coverage-census-commit.ts';
import {
  readBaselineFile,
  readCensusArguments,
} from './coverage-census-input.ts';
import type { placeTally, } from './coverage-census-place.ts';
import { reportCensus, } from './coverage-census-reading.ts';
import type {
  runSuite,
  tallyCoverage,
} from './coverage-census-steps.ts';

//region Coverage census run
// LEDGER T8: WHICH PACKAGE CODE THE UNIT SUITE NEVER RUNS, measured rather
// than guessed from which names a test mentions. The mise task builds the
// package with a source map beside every chunk (`rolldown.coverage.config.ts`),
// runs the command (`coverage-census.ts`), then runs the normal build again.
// This runs the unit suite, or the test files named, under `NODE_V8_COVERAGE`,
// tallies the cold code over every process (`coverage-tally.ts`), maps it to
// source lines (`coverage-lines.ts`), prints the report and writes the census as
// JSON.
//
// An invariant throw is counted apart from the cold code. A stretch no test
// ran that is nothing but throws of a broken invariant, an `Error` whose
// message begins `unreachable:` or a class named `...InvariantError`, is a
// guard no honest input reaches, so no test should; the report prints such
// stretches' count beside the cold counts and lists each, the census file
// keeps them under `invariantThrows`, and no row, total or baseline reading
// counts one as cold (`coverage-census-invariant.ts`). A guard is never
// removed to close a stretch.
//
// A BATCH OF TESTS PROVES ITS REACH by running its own test files with
// `--baseline <an earlier census.json>` and `--source` for each source it
// claims: every claimed stretch must read as ran, and every claimed source the
// baseline holds no stretch in must be loaded by this run with no cold
// stretch left, since a baseline that never loaded a source proves nothing of
// it. No stretch may read as cold since then either, code the baseline ran
// that this run left cold (ledger B61). A claimed source edited since the baseline's commit is held to the same
// standard, this run loading it and leaving no cold stretch, since its
// baseline lines name other code now. The baseline must be of the current
// census format, taken from a tree matching its commit, and is read before the
// suite runs. A census written before invariant throws were counted apart
// holds them among its cold stretches and is refused by name
// (`readBaselineCensus`).
//
// SPENDS NO QUOTA, and the raw coverage (about 8 GB for the whole suite) is
// deleted once the census is written; the census and the suite's log stay in
// a directory the report names. Both sit under the package's cache directory
// (`~/.cache/translation-repair/coverage` unless XDG_CACHE_HOME says
// otherwise), ON DISK, because this host's `/tmp` is a 16 GB tmpfs held in
// memory, which the raw coverage alone would half fill. A suite that fails is
// refused rather than counted, since its failing tests skip code they would
// have run.
//
// THE FAMILY. Run from a package that has a coverage build config, the census
// measures every package beside it (`coverage-census-family.ts`): it reads each
// member's coverage build before it spends a suite, runs each member's unit
// suite in the member's directory under one coverage directory, and names
// sources from the family directory, as `<package>/src/<file>`. Run from a
// directory that is no such package, it measures that directory alone.
//
// RUN FROM THE PACKAGE DIRECTORY, which the task does:
// `mise run //package/translation-repair/coverage-census:coverage-census`.

/**
 The steps of the census that touch git, the suite, the coverage and the
 build, which a case scripts.

 @example
 ```ts
 const steps: CoverageCensusSteps = { packageCommit, runSuite, tallyCoverage, placeTally, sourcesEditedSince, };
 ```
 */
export type CoverageCensusSteps = {
  /**
   Asks git for the commit built from and whether the files match it.
   */
  readonly packageCommit: typeof packageCommit;

  /**
   Runs the unit suite or the named test files under coverage.
   */
  readonly runSuite: typeof runSuite;

  /**
   Reads the coverage files into a painted tally.
   */
  readonly tallyCoverage: typeof tallyCoverage;

  /**
   Places the tally on source lines through the build's maps.
   */
  readonly placeTally: typeof placeTally;

  /**
   Asks git which files changed since a commit.
   */
  readonly sourcesEditedSince: typeof sourcesEditedSince;
};

/**
 What one run of a suite answered.
 */
type SuiteAnswer = Awaited<ReturnType<CoverageCensusSteps['runSuite']>>;

/**
 Reads every earlier census the command line named, side by side.

 @param paths - census files named, in the order a refusal must be reported in

 @param readBaseline - reads one census file, passed in so a case scripts which of two reads ends first

 @returns Each census beside the path it came from, in the order named

 @throws Whatever the first file named, whose read failed, was refused with

 @example
 ```ts
 const baselines = await readBaselines({ paths: ['/cats/before.json',], readBaseline: readBaselineFile, },);
 ```
 */
export async function readBaselines(
  {
    paths,
    readBaseline,
  }: {
    readonly paths: readonly string[];
    readonly readBaseline: typeof readBaselineFile;
  },
): Promise<readonly Baseline[]> {
  return await allInInputOrder({
    members: paths.map(async function readOne(path,): Promise<Baseline> {
      return {
        path,
        census: await readBaseline({ path, },),
      };
    },),
  },);
}

/**
 Says a suite failed, naming the log to read.

 @param subject - which suite: the whole one, or one member's

 @param suite - what it answered

 @param logPath - its log

 @returns Refusal

 @example
 ```ts
 throw suiteFailure({ subject: 'the suite', suite, logPath, },);
 ```
 */
function suiteFailure(
  {
    subject,
    suite,
    logPath,
  }: {
    readonly subject: string;
    readonly suite: SuiteAnswer;
    readonly logPath: string;
  },
): StatedRefusalError {
  return new StatedRefusalError({
    says: `${subject} failed (exit ${String(suite.exitCode,)}, ${String(suite.failures,)} FAIL ${
      wordForCount({
        count: suite.failures,
        one: 'marker',
        many: 'markers',
      },)
    }); its log is `
      + `${logPath}; a census of a failing suite would count code its failing tests skipped`,
  },);
}

/**
 One member's suite and what it answered.

 @example
 ```ts
 const ran: MemberSuite = { run, suite: { passes: 3, failures: 0, exitCode: 0, }, logPath, };
 ```
 */
type MemberSuite = {
  /**
   The run asked for.
   */
  readonly run: FamilySuiteRun;

  /**
   What it answered.
   */
  readonly suite: SuiteAnswer;

  /**
   Where its output went.
   */
  readonly logPath: string;
};

/**
 Runs one member's suite.

 @param run - member's run

 @param reportDirectory - where the member's log goes

 @param coverageDirectory - where V8 writes coverage

 @param runSuite - runs one suite, passed in so a case scripts it

 @returns Run beside its log and what its suite answered

 @example
 ```ts
 const ran = await memberSuiteOf({ run, reportDirectory, coverageDirectory, runSuite, },);
 ```
 */
async function memberSuiteOf(
  {
    run,
    reportDirectory,
    coverageDirectory,
    runSuite,
  }: {
    readonly run: FamilySuiteRun;
    readonly reportDirectory: string;
    readonly coverageDirectory: string;
    readonly runSuite: CoverageCensusSteps['runSuite'];
  },
): Promise<MemberSuite> {
  /**
   Where this member's suite output goes.
   */
  const logPath = join(
    reportDirectory,
    `suite-${run.member}.log`,
  );
  return {
    run,
    suite: await runSuite({
      command: [
        'mise',
        'run',
        'test:unit',
        ...run.files,
      ],
      cwd: run.cwd,
      coverageDirectory,
      logPath,
    },),
    logPath,
  };
}

/**
 Runs each member's suite in turn, one at a time since all write coverage to
 one directory, each yielded as it ends so the first failure stops the rest.

 @param runs - each member's run

 @param reportDirectory - where each member's log goes

 @param coverageDirectory - where V8 writes coverage

 @param runSuite - runs one suite, passed in so a case scripts it

 @returns Each run beside its log and what its suite answered

 @example
 ```ts
 for await (const { run, suite, logPath, } of memberSuites({ runs, reportDirectory, coverageDirectory, runSuite, },)) report(suite,);
 ```
 */
async function* memberSuites(
  {
    runs,
    reportDirectory,
    coverageDirectory,
    runSuite,
  }: {
    readonly runs: readonly FamilySuiteRun[];
    readonly reportDirectory: string;
    readonly coverageDirectory: string;
    readonly runSuite: CoverageCensusSteps['runSuite'];
  },
): AsyncGenerator<MemberSuite> {
  for (const run of runs) {
    yield memberSuiteOf({
      run,
      reportDirectory,
      coverageDirectory,
      runSuite,
    },);
  }
}

/**
 Runs the unit suite of the package, or the test files named, and counts its
 passes.

 @param testFiles - test files named, none for the whole suite

 @param packageDirectory - package directory the suite runs in

 @param reportDirectory - where the suite's log goes

 @param coverageDirectory - where V8 writes coverage

 @param runSuite - runs one suite, passed in so a case scripts it

 @returns Passing markers the suite printed

 @throws StatedRefusalError where the suite failed or printed no pass marker

 @example
 ```ts
 const passes = await runPackageSuite({ testFiles: [], packageDirectory, reportDirectory, coverageDirectory, runSuite, },);
 ```
 */
async function runPackageSuite(
  {
    testFiles,
    packageDirectory,
    reportDirectory,
    coverageDirectory,
    runSuite,
  }: {
    readonly testFiles: readonly string[];
    readonly packageDirectory: string;
    readonly reportDirectory: string;
    readonly coverageDirectory: string;
    readonly runSuite: CoverageCensusSteps['runSuite'];
  },
): Promise<number> {
  /**
   Where the suite's output goes.
   */
  const logPath = join(
    reportDirectory,
    'suite.log',
  );
  /**
   The suite's result.
   */
  const suite = await runSuite({
    command: [
      'mise',
      'run',
      'test:unit',
      ...testFiles,
    ],
    cwd: packageDirectory,
    coverageDirectory,
    logPath,
  },);
  if ((suite.exitCode !== 0) || (suite.failures > 0))
    throw suiteFailure({
      subject: 'the suite',
      suite,
      logPath,
    },);
  // A CLEAN EXIT WITH NO PASS MARKER RAN NO TEST, whatever kept it from
  // starting one. Counted, it printed a census beside "0 passes" whose every
  // line is about a suite that never ran; until 2026-10-10 it was counted
  // (ledger B384).
  if (suite.passes === 0)
    throw new StatedRefusalError({
      says: `the suite printed no PASS marker, so it ran no test; its log is ${logPath}; a census of a suite `
        + 'that ran nothing would count every line of the package as code no test ran',
    },);
  return suite.passes;
}

/**
 Runs the unit suite of every member of the family, or of the members the named
 test files stand in, and adds up their passes.

 @param family - family

 @param testFiles - test files named, none for every member's whole suite

 @param reportDirectory - where each member's log goes

 @param coverageDirectory - where V8 writes coverage

 @param runSuite - runs one suite, passed in so a case scripts it

 @returns Passing markers the suites printed between them

 @throws StatedRefusalError where a test file named stands in no member, a
 member's suite failed, or no suite printed a pass marker

 @example
 ```ts
 const passes = await runFamilySuites({ family, testFiles: [], reportDirectory, coverageDirectory, runSuite, },);
 ```
 */
async function runFamilySuites(
  {
    family,
    testFiles,
    reportDirectory,
    coverageDirectory,
    runSuite,
  }: {
    readonly family: CensusFamily;
    readonly testFiles: readonly string[];
    readonly reportDirectory: string;
    readonly coverageDirectory: string;
    readonly runSuite: CoverageCensusSteps['runSuite'];
  },
): Promise<number> {
  /**
   Passing markers so far.
   */
  let passes = 0;
  for await (
    const {
      run,
      suite,
      logPath,
    } of memberSuites({
      runs: familySuiteRunsOf({
        family,
        testFiles,
      },),
      reportDirectory,
      coverageDirectory,
      runSuite,
    },)
  ) {
    if ((suite.exitCode !== 0) || (suite.failures > 0))
      throw suiteFailure({
        subject: `the suite of ${run.member}`,
        suite,
        logPath,
      },);
    passes += suite.passes;
  }
  if (passes === 0)
    throw new StatedRefusalError({
      says: `the suite printed no PASS marker, so it ran no test; its logs are in ${reportDirectory}; a census of a `
        + 'suite that ran nothing would count every line of every package as code no test ran',
    },);
  return passes;
}

/**
 Runs the census the command line asked for and prints its report.

 Returns nothing: the report on stdout IS the output, and the census file it
 names is the record.

 @param line - census's command line, read whole by `reportingRefusals`

 @param packageDirectory - package directory, where the mise task runs and
 whose `dist/final/node` the tests import

 @param family - family of packages beside this one that the census
 measures with it, none where the package is measured alone

 @param env - environment the cache directory is read from

 @param steps - steps that touch git, suite, coverage and
 build

 @param l - logger the progress lines go to, apart from the report on stdout

 @throws StatedRefusalError where --source is named with no --baseline, the
 build directory cannot be listed, holds no bundle or no source map, or is
 minified, or the suite failed or printed no pass marker

 @throws CensusBaselineError where a baseline named does not read as a census
 this command wrote

 @example
 ```ts
 await runCoverageCensus({ line, packageDirectory: process.cwd(), env: process.env, steps, l, },);
 ```
 */
export async function runCoverageCensus(
  {
    line,
    packageDirectory,
    family,
    env,
    steps,
    l,
  }: {
    readonly line: CommandLineOf<'coverage-census'>;
    readonly packageDirectory: string;
    readonly family?: CensusFamily;
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly steps: CoverageCensusSteps;
    readonly l: Logger;
  },
): Promise<void> {
  /**
   What was asked.
   */
  const asked = readCensusArguments({ line, },);
  /**
   Each earlier census named, read before the suite runs, so a file that
   does not read as one refuses at once rather than after the whole suite
   (ledger M67's control run spent a suite on one).
   */
  const baselines = await readBaselines({
    paths: asked.baseline,
    readBaseline: readBaselineFile,
  },);
  /**
   Directory sources are named from and git is asked about: the family's
   directory where the family is measured, else the package's.
   */
  const measuredDirectory = (family === undefined) ? packageDirectory : family.directory;
  /**
   Build directory the tests import, or the family's directory, from which
   each member's bundles are named by their path.
   */
  const distDirectory = (family === undefined)
    ? join(
      packageDirectory,
      'dist',
      'final',
      'node',
    )
    : family.directory;
  /**
   The build's bundles, split by whether a map stands beside each.
   */
  const bundleMaps = await ((family === undefined)
    ? requireCoverageBuild({
      distDirectory,
      l,
      readText: readUtf8Text,
    },)
    : requireFamilyCoverageBuild({
      family,
      l,
      readText: readUtf8Text,
    },));
  /**
   The commit and whether the files match it.
   */
  const {
    head,
    clean,
  } = await steps.packageCommit({ packageDirectory: measuredDirectory, },);
  /**
   The census's home on disk, under the package's cache directory.
   */
  const censusHome = join(
    packageCacheDir({ env, },),
    'coverage',
  );
  await mkdir(
    censusHome,
    { recursive: true, },
  );
  /**
   Where the census and the suite's log are kept.
   */
  const reportDirectory = await mkdtemp(join(
    censusHome,
    'census-',
  ),);
  /**
   Where V8 writes coverage, removed with everything in it when this returns
   or throws.
   */
  await using coverage = await mkdtempDisposable(join(
    censusHome,
    'raw-',
  ),);
  l.info(`at ${head}${clean ? '' : ' with uncommitted changes'}; coverage in ${coverage.path}, report in ${reportDirectory}`,);
  /**
   Passing markers the suite or suites printed.
   */
  const passes = (family === undefined)
    ? await runPackageSuite({
      testFiles: asked.testFiles,
      packageDirectory,
      reportDirectory,
      coverageDirectory: coverage.path,
      runSuite: steps.runSuite,
    },)
    : await runFamilySuites({
      family,
      testFiles: asked.testFiles,
      reportDirectory,
      coverageDirectory: coverage.path,
      runSuite: steps.runSuite,
    },);
  await reportCensus({
    asked,
    packageDirectory: measuredDirectory,
    distDirectory,
    bundleMaps,
    head,
    clean,
    passes,
    tally: await steps.tallyCoverage({
      coverageDirectory: coverage.path,
      bundleUrlPrefix: `${pathToFileURL(distDirectory,)
        .href}/`,
      ...((family === undefined) ? {} : { isBundleName: isFamilyBundleName, }),
    },),
    reportDirectory,
    baselines,
    place: steps.placeTally,
    editedSince: steps.sourcesEditedSince,
  },);
}

//endregion Coverage census run
