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
  buildRunWatchArguments,
} from './service-arguments.ts';
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
  'root' | 'crateName' | 'version' | 'repo' | 'tagTemplate' | 'assetTemplates' | 'userAgent' | 'maxTimeSeconds'
  >;

/**
 Watch a publish workflow run to its end, then check crates.io and the GitHub release.

 A version that never goes to crates.io (`-alpha.N`, `-beta.N` and every form the label rule does not name) is not
 looked up there: the report says crates.io was skipped and why, and the result depends on the workflow run and the
 GitHub release.

 A failed workflow run does not skip the checks, so the report shows everything that is and is not in place. The result
 is zero only when the workflow succeeded, crates.io has the version, and the release holds every expected asset.

 @param options - the crate, its version and the check settings

 @param run - runs a program call and reports its status and output

 @param runId - numeric identifier of the publish run

 @param print - prints one line

 @returns zero when the publish is complete, the failure status otherwise

 @example
 ```ts
 await checkPublishedRun({ options, run, runId: '77', print: console.log });
 ```
 */
export async function checkPublishedRun({
  options,
  run,
  runId,
  print,
}: {
  readonly options: PublishCheckInput;
  readonly run: WaitRunner;
  readonly runId: string;
  readonly print: (line: string) => void;
}): Promise<number> {
  print(`run ${runId}`);
  /**
   Outcome of watching the run to its end.
   */
  const watched = await run({
    program: ghProgram,
    args: buildRunWatchArguments({
      runId,
      ...repositoryScope(options)
    }),
    cwd: options.root,
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
    watchStatus: watched.status,
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
    runId: runLookup.value,
    print
  });
}
