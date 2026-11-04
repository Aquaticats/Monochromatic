#!/usr/bin/env node
/**
 Container entry point of the concurrent-commit end-to-end suite.
 Run only through `mise run //package/git-policy/cli:test:e2e:concurrent:native`,
 which starts it inside a bounded,
 network-isolated Podman container with the native executable.

 Environment:
 `E2E_WRAPPER` names the wrapper whose `git` shadows the real Git:
 `native` (the default and the only value);
 `E2E_SEED` replays a seed;
 `E2E_SCENARIOS` and `E2E_GIT_VERSIONS` are comma-separated filters (the Git filter names the installed Git's release).
 Output is terminal report text (AGENTS.md TLG exception):
 workspace packages,
 including the tagged logger,
 are not installed in the container.

 @module
 */

import { execFile, } from 'node:child_process';
import { randomInt, } from 'node:crypto';
import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { promisify, } from 'node:util';

import { assertCommitShapeTrace, } from './commit-shape-trace-fixture.ts';
import { WRAPPER_BIN, } from './repository-fixture.ts';
import {
  formatResult,
  runPassed,
  summarize,
} from './report-fixture.ts';
import { SCENARIO_CATALOG, } from './scenario-catalog-fixture.ts';
import {
  runScenario,
  type ScenarioResult,
} from './scenario-model-fixture.ts';
import {
  parseSeed,
  SEED_MODULUS,
} from './seeded-random-fixture.ts';
import {
  verifyWrapperExecutable,
  WRAPPER_KIND,
} from './wrapper-selection-fixture.ts';

//region Configuration

/**
 The Git the image installs from its distribution's package: the only Git the suite runs, whatever release that is
 (owner decision, 2026-11-04: the 2.55.0 source build and its scenarios are dropped and the image's current Git is used).
 */
const INSTALLED_GIT = '/usr/bin/git';

/**
 Reads the release of the installed Git.

 @returns the dotted release, such as `2.56.0`

 @throws Error when the installed Git's `--version` output has no `git version <release>` line

 @example
 ```ts
 await installedGitVersion(); // => '2.56.0'
 ```
 */
async function installedGitVersion(): Promise<string> {
  /**
   Output of the installed Git's version command.
   */
  const { stdout, } = await promisify(execFile,)(
    INSTALLED_GIT,
    ['--version',],
  );
  /**
   The release in the version line.
   */
  const release = /^git version (?<release>\d+(?:\.\d+)+)/u.exec(stdout,)?.groups?.['release'];
  if (release === undefined)
    throw new Error(`${INSTALLED_GIT} printed no release in its version line: ${stdout.trim()}`,);
  return release;
}

/**
 Container work root.
 */
const WORK_ROOT = '/work';


/**
 Splits a comma-separated filter.

 @param text - filter text

 @returns entries, empty for no filter

 @example
 ```ts
 splitFilter('a,b'); // => ['a', 'b']
 ```
 */
function splitFilter(text: string,): readonly string[] {
  return text.split(',',)
    .map(function trim(entry,) {
      return entry.trim();
    },)
    .filter(function nonEmpty(entry,) {
      return entry !== '';
    },);
}

//endregion Configuration

//region Run

/**
 Error raised when any scenario failed or errored.
 */
class ConcurrentSuiteError extends Error {
  /**
   Creates the suite failure.

   @param message - failure summary

   @example
   ```ts
   throw new ConcurrentSuiteError('3 scenario runs failed');
   ```
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'ConcurrentSuiteError';
  }
}

/**
 Runs the filtered catalog on every selected Git version and prints the report.

 @throws {@link ConcurrentSuiteError} when a scenario failed or errored

 @example
 ```ts
 await runSuite();
 ```
 */
async function runSuite(): Promise<void> {
  /**
   Seed from the task argument, or a fresh one.
   */
  const {
    seed,
    generated,
  } = parseSeed({
    text: process.env
      .E2E_SEED
      ?? '',
    fresh() {
      return randomInt(
        0,
        SEED_MODULUS,
      );
    },
  },);
  /**
   Scenario name filter.
   */
  const scenarioFilter = splitFilter(process.env
    .E2E_SCENARIOS
    ?? '',);
  /**
   Git version filter.
   */
  const versionFilter = splitFilter(process.env
    .E2E_GIT_VERSIONS
    ?? '',);
  console.log(await verifyWrapperExecutable({
    kind: WRAPPER_KIND,
    bin: WRAPPER_BIN,
  },),);
  console.log(`seed=${String(seed,)}${generated ? ' (generated)' : ''}`,);
  console.log(`replay: mise run //package/git-policy/cli:test:e2e:concurrent:native --seed ${String(seed,)}`,);
  /**
   Committed trace.
   */
  const trace = assertCommitShapeTrace(
    JSON.parse(await readFile(
      join(
        import.meta.dirname,
        'commit-shape-trace.json',
      ),
      'utf8',
    ),),
  );
  /**
   Selected scenarios.
   */
  const scenarios = SCENARIO_CATALOG.filter(function selected(definition,) {
    return (scenarioFilter.length === 0) || scenarioFilter.includes(definition.name,);
  },);
  /**
   Selected Git versions.
   */
  const versions = [await installedGitVersion(),].filter(function selected(version,) {
    return (versionFilter.length === 0) || versionFilter.includes(version,);
  },);
  /**
   Results, run strictly one scenario at a time so interleavings stay inside a scenario; each scenario starts and stops
   the commit service its commits land through.
   */
  const results = await versions.flatMap(function versionRuns(gitVersion,) {
    return scenarios.map(function scenarioRun(definition,) {
      return {
        gitVersion,
        definition,
      };
    },);
  },)
    .reduce<Promise<readonly ScenarioResult[]>>(
    async function next(
      previous,
      {
        gitVersion,
        definition,
      },
    ) {
    /**
     Results so far.
     */
      const done = await previous;
    /**
     This run's result.
     */
      const result = await runScenario({
        definition,
        gitVersion,
        seed,
        workRoot: WORK_ROOT,
        trace,
      },);
      console.log(formatResult(result,)
        .join('\n',),);
      return [
        ...done,
        result,
      ];
    },
    Promise.resolve([],),
    );
  console.log(summarize(results,)
    .join('\n',),);
  console.log(`seed=${String(seed,)}`,);
  if (!runPassed(results,)) {
    throw new ConcurrentSuiteError(`${String(results.filter(function failed(result,) {
      return (result.status === 'fail') || (result.status === 'error');
    },)
      .length,)} of ${String(results.length,)} scenario runs failed; replay with --seed ${String(seed,)}`,);
  }
}

//endregion Run

await runSuite();
