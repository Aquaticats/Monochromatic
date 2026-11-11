import {
  curlProgram,
  failureExitCode,
  ghProgram,
  millisecondsPerSecond,
} from './constants.ts';
import {
  foundValue,
  type Lookup,
  missingValue,
} from './lookup.ts';
import { cratesIoRoute } from './crates-io-version-label.ts';
import {
  describeDuration,
  expandTemplate,
  formatPublishResult,
  parseAssetNames,
  parseCratesIoCreatedAt,
  selectRunId,
} from './publish-results.ts';
import type {
  ProgramCall,
  ProgramResult,
} from './run-program.ts';
import {
  buildCratesIoArguments,
  buildReleaseViewArguments,
  buildRunListArguments,
  buildRunViewArguments,
} from './service-arguments.ts';
import {
  describeRunOutcome,
  isRunCompleted,
  jobsStillGoing,
  parseRunOutcome,
  type RunOutcome,
} from './run-outcome.ts';
import type { WaitOptions } from './wait-options.ts';

/**
 Runs one program call to completion and reports how it ended.
 */
export type WaitRunner = (call: ProgramCall) => Promise<ProgramResult>;

/**
 Everything the lookups need besides the attempt number.
 */
type LookInput = {
  readonly options: WaitOptions;
  readonly run: WaitRunner;
  readonly sleep: (milliseconds: number) => Promise<void>;
};

/**
 Repository argument of every `gh` call: the named repository, or none so `gh` reads the working directory.

 @param options - anything that may name the repository

 @returns an object to spread into a builder call

 @example
 ```ts
 repositoryScope(options);
 ```
 */
export function repositoryScope(options: { readonly repo?: string }): { readonly repo?: string } {
  return options.repo === undefined ? {} : { repo: options.repo };
}

/**
 Describe why a failed program call has no usable answer.

 @param result - finished call with a non-zero status

 @returns the call's error text, or its exit status when it printed none

 @example
 ```ts
 failureReason({ status: 1, stdout: '', stderr: 'not logged in' });
 ```
 */
export function failureReason(result: ProgramResult): string {
  return result.stderr
    .trim()
    === '' ? `gh exited with status ${String(result.status)}` : result.stderr
      .trim();
}

/**
 Look for the workflow run of the commit from one attempt on, sleeping between attempts.

 The attempts are a bounded chain (the attempt limit), each waiting for the previous one, so the recursion is as deep
 as the limit and no deeper.

 @param input - options, program runner and sleeper

 @param attempt - number of this attempt, from one

 @returns the run identifier, or a miss that carries the last reason

 @example
 ```ts
 await findRun({ input, attempt: 1 });
 ```
 */
async function findRun({
  input,
  attempt,
}: {
  readonly input: LookInput;
  readonly attempt: number;
}): Promise<Lookup<string>> {
  /**
   Parts of the input this attempt uses.
   */
  const {
    options,
    run,
    sleep
  } = input;
  /**
   Answer of `gh run list`.
   */
  const listing = await run({
    program: ghProgram,
    args: buildRunListArguments({
      workflow: options.workflow,
      limit: options.runLimit,
      ...repositoryScope(options)
    }),
    cwd: options.root,
  });
  /**
   The run of the commit, when the listing succeeded and holds it.
   */
  const lookup: Lookup<string> = listing.status === 0
    ? selectRunId({
      listing: listing.stdout,
      sha: options.sha
    })
    : missingValue(failureReason(listing));
  if (lookup.found || (attempt >= options.attempts))
    return lookup;
  await sleep(options.pollIntervalSeconds * millisecondsPerSecond);
  return await findRun({
    input,
    attempt: attempt + 1
  });
}

/**
 Turn the answer of `gh release view` into the asset names or the reason they are unknown.

 @param release - finished `gh release view` call

 @returns the asset names, or a miss that carries the `gh` error text

 @example
 ```ts
 releaseAssets(result);
 ```
 */
function releaseAssets(release: ProgramResult): Lookup<readonly string[]> {
  if (release.status === 0)
    return foundValue(parseAssetNames(release.stdout));
  return missingValue(failureReason(release));
}

/**
 The settings the check of a found publish run reads.
 */
export type PublishCheckInput = Pick<
  WaitOptions,
  | 'root' | 'crateName' | 'version' | 'repo' | 'workflow' | 'tagTemplate' | 'assetTemplates'
    | 'userAgent' | 'maxTimeSeconds' | 'pollIntervalSeconds' | 'followAttempts'
  >;

/**
 Everything one look at a run needs besides the look number.
 */
type FollowInput = {
  readonly options: Pick<WaitOptions, 'root' | 'repo' | 'pollIntervalSeconds' | 'followAttempts'>;
  readonly run: WaitRunner;
  readonly sleep: (milliseconds: number) => Promise<void>;
  readonly print: (line: string) => void;
  readonly workflow: string;
  readonly runId: string;
};

/**
 Say why a run that never ended was left.

 @param outcome - the last thing the run said about itself

 @param boundSeconds - how long the wait was allowed to follow it

 @returns text naming what the wait was still waiting for

 @example
 ```ts
 describeStillGoing({ outcome, boundSeconds: 2400 });
 ```
 */
function describeStillGoing({
  outcome,
  boundSeconds,
}: {
  readonly outcome: RunOutcome;
  readonly boundSeconds: number;
}): string {
  /**
   Jobs the run had not ended when the wait left it.
   */
  const going = jobsStillGoing(outcome);
  return going.length === 0
    ? `The run was still going after ${describeDuration(boundSeconds)}, with status ${JSON.stringify(outcome.status)}.`
    : `The run was still going after ${describeDuration(boundSeconds)}, waiting for ${going.join(', ')}.`;
}

/**
 Look at a run from one look on, sleeping between the looks, until the run says it ended.

 The answer of `gh run view` is the only verdict read here. A look that cannot be read costs one look, not the whole
 wait: a release must not be reported as failed because the GitHub API failed once while the run was still going.

 @param input - the run, the settings and the readers

 @param look - number of this look, from one

 @returns what the run said about itself once it ended, or a miss that says why it never did

 @example
 ```ts
 await followLook({ input: { options, run, sleep, print, workflow, runId }, look: 1 });
 ```
 */
async function followLook({
  input,
  look,
}: {
  readonly input: FollowInput;
  readonly look: number;
}): Promise<Lookup<RunOutcome>> {
  /**
   Parts of the input this look uses.
   */
  const {
    options,
    run,
    sleep,
    print,
    workflow,
    runId
  } = input;
  /**
   Answer of `gh run view`.
   */
  const view = await run({
    program: ghProgram,
    args: buildRunViewArguments({
      runId,
      ...repositoryScope(options)
    }),
    cwd: options.root,
  });
  /**
   What the run says about itself, or why this look read nothing.
   */
  const lookup: Lookup<RunOutcome> = view.status === 0
    ? parseRunOutcome(view.stdout)
    : missingValue(failureReason(view));
  if (lookup.found && isRunCompleted(lookup.value))
    return lookup;
  /**
   Seconds this wait may follow one run.
   */
  const boundSeconds = options.followAttempts * options.pollIntervalSeconds;
  if (look >= options.followAttempts)
    return lookup.found
      ? missingValue(describeStillGoing({
        outcome: lookup.value,
        boundSeconds
      }))
      : missingValue(`The run could not be read within ${describeDuration(boundSeconds)}: ${lookup.reason}`);
  print(
    lookup.found
      ? `${describeRunOutcome({
        workflow,
        runId,
        outcome: lookup.value
      })}; waiting up to ${describeDuration(boundSeconds)}`
      : `${workflow} run ${runId} could not be read (${lookup.reason}); waiting up to ${describeDuration(boundSeconds)}`,
  );
  await sleep(options.pollIntervalSeconds * millisecondsPerSecond);
  return await followLook({
    input,
    look: look + 1
  });
}

/**
 Follow one workflow run to its own end, bounded by the look limit and the poll interval.

 Every look at a run that has not ended prints what the run says about itself, so a run that takes a while says which
 jobs it is still running instead of looking stuck, and a look that could not be read costs one look rather than the
 whole wait. The caller prints the verdict, from the outcome this returns.

 @param options - the repository scope and the bounds of the wait

 @param run - runs a program call and reports its status and output

 @param sleep - waits the given milliseconds

 @param print - prints one line

 @param workflow - workflow file of the run, for the lines that name it

 @param runId - numeric identifier of the run

 @returns what the run said when it ended, or a miss that says why it never did

 @example
 ```ts
 await followRunOutcome({ options, run, sleep, print, workflow: 'cargo-publish.yml', runId: '77' });
 ```
 */
export async function followRunOutcome({
  options,
  run,
  sleep,
  print,
  workflow,
  runId,
}: {
  readonly options: FollowInput['options'];
  readonly run: WaitRunner;
  readonly sleep: (milliseconds: number) => Promise<void>;
  readonly print: (line: string) => void;
  readonly workflow: string;
  readonly runId: string;
}): Promise<Lookup<RunOutcome>> {
  return await followLook({
    input: {
      options,
      run,
      sleep,
      print,
      workflow,
      runId
    },
    look: 1
  });
}

/**
 Follow a publish workflow run to its own end, then check crates.io and the GitHub release.

 The run is the one that publishes the crate and the one whose jobs build and upload the archives, so the archives the
 release holds come from it, never from the release run that dispatched it.

 A run that is still going says which jobs it is still running, and the wait keeps following it until the look limit;
 a run that never ended is reported as still going rather than passed or failed.

 A version that never goes to crates.io (`-alpha.N`, `-beta.N` and every form the label rule does not name) is not
 looked up there: the report says crates.io was skipped and why, and the result depends on the workflow run and the
 GitHub release.

 A failed workflow run does not skip the checks, so the report shows everything that is and is not in place. The result
 is zero only when the run ended with success, crates.io has the version, and the release holds every expected asset.

 @param options - the crate, its version and the check settings

 @param run - runs a program call and reports its status and output

 @param sleep - waits the given milliseconds between two looks at the run

 @param runId - numeric identifier of the publish run

 @param print - prints one line

 @returns zero when the publish is complete, the failure status otherwise

 @example
 ```ts
 await checkPublishedRun({ options, run, sleep, runId: '77', print: console.log });
 ```
 */
export async function checkPublishedRun({
  options,
  run,
  sleep,
  runId,
  print,
}: {
  readonly options: PublishCheckInput;
  readonly run: WaitRunner;
  readonly sleep: (milliseconds: number) => Promise<void>;
  readonly runId: string;
  readonly print: (line: string) => void;
}): Promise<number> {
  /**
   What the run said about itself when it ended, or why it never did.
   */
  const outcome = await followRunOutcome({
    options,
    run,
    sleep,
    print,
    workflow: options.workflow,
    runId
  });
  /**
   Whether the label rule sends the version to crates.io, and why not when it does not.
   */
  const route = cratesIoRoute(options.version);
  /**
   Answer of crates.io, which is not asked for a version that never goes there.
   */
  const crates = route.goes
    ? await run({
      program: curlProgram,
      args: buildCratesIoArguments({
        crateName: options.crateName,
        version: options.version,
        userAgent: options.userAgent,
        maxTimeSeconds: options.maxTimeSeconds,
      }),
      cwd: options.root,
    })
    : {
      status: 0,
      stdout: '',
      stderr: ''
    };
  /**
   Tag of the release.
   */
  const tag = expandTemplate({
    template: options.tagTemplate,
    crateName: options.crateName,
    version: options.version
  });
  /**
   Answer of `gh release view`.
   */
  const release = await run({
    program: ghProgram,
    args: buildReleaseViewArguments({
      tag,
      ...repositoryScope(options)
    }),
    cwd: options.root,
  });
  /**
   What was found, and whether it is complete.
   */
  const report = formatPublishResult({
    workflow: options.workflow,
    runId,
    outcome,
    version: options.version,
    createdAt: parseCratesIoCreatedAt(crates.stdout),
    ...(route.goes ? {} : { skippedCratesIoReason: route.reason }),
    tag,
    expectedAssets: options.assetTemplates
      .map(function expandAsset(template) {
        return expandTemplate({
          template,
          crateName: options.crateName,
          version: options.version
        });
      }),
    release: releaseAssets(release),
  });
  for (const line of report.lines)
    print(line);
  return report.ok ? 0 : failureExitCode;
}

/**
 Wait for the publish workflow run of a commit, then check crates.io and the GitHub release.

 The wait is bounded by the attempt limit and the poll interval. A failed workflow run does not skip the checks, so the
 report shows everything that is and is not in place. The result is zero only when the workflow succeeded, crates.io
 has the version, and the release holds every expected asset.

 @param options - validated wait options

 @param run - runs a program call and reports its status and output

 @param sleep - waits the given milliseconds

 @param print - prints one line

 @returns zero when the publish is complete, the failure status otherwise

 @example
 ```ts
 await waitForPublish({ options, run, sleep, print: console.log });
 ```
 */
export async function waitForPublish({
  options,
  run,
  sleep,
  print,
}: {
  readonly options: WaitOptions;
  readonly run: WaitRunner;
  readonly sleep: (milliseconds: number) => Promise<void>;
  readonly print: (line: string) => void;
}): Promise<number> {
  /**
   The run of the commit, or why none turned up.
   */
  const runLookup = await findRun({
    input: {
      options,
      run,
      sleep
    },
    attempt: 1
  });
  if (!runLookup.found) {
    print(
      `no ${options.workflow} run for ${options.sha} within ${describeDuration(options.attempts * options.pollIntervalSeconds)}`,
    );
    print(`last result: ${runLookup.reason}`);
    return failureExitCode;
  }
  return await checkPublishedRun({
    options,
    run,
    sleep,
    runId: runLookup.value,
    print
  });
}
