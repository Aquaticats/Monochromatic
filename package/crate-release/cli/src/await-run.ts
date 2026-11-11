import {
  failureExitCode,
  ghProgram,
  millisecondsPerSecond,
  releaseWorkflow,
} from './constants.ts';
import type { AwaitOptions } from './await-options.ts';
import {
  foundValue,
  type Lookup,
  missingValue,
} from './lookup.ts';
import { CrateReleaseError } from './error.ts';
import { readPackageVersion } from './package-version.ts';
import { describeDuration } from './publish-results.ts';
import { selectPublishRun } from './publish-run-selection.ts';
import {
  describeRunFailures,
  describeRunOutcome,
  runPassed,
} from './run-outcome.ts';
import type { DispatchedRun } from './run-results.ts';
import {
  buildDispatchedRunListArguments,
  buildManifestContentsArguments,
} from './service-arguments.ts';
import {
  checkPublishedRun,
  failureReason,
  followRunOutcome,
  repositoryScope,
  type WaitRunner,
} from './wait-run.ts';

/**
 Everything the look for the publish run needs besides the attempt number.
 */
type LookInput = {
  readonly options: AwaitOptions;
  readonly run: WaitRunner;
  readonly sleep: (milliseconds: number) => Promise<void>;
  readonly notBefore: string;
};

/**
 Build the `gh api` arguments that read the crate's manifest at a commit.

 @param options - validated await options

 @param sha - commit to read the manifest at

 @returns the arguments, or a miss that carries the refusal when the commit is not a hash the API could have written

 @example
 ```ts
 manifestReadArguments({ options, sha: 'abc1234' });
 ```
 */
function manifestReadArguments({
  options,
  sha,
}: {
  readonly options: AwaitOptions;
  readonly sha: string;
}): Lookup<readonly string[]> {
  try {
    return foundValue(buildManifestContentsArguments({
      manifestPath: options.manifestPath,
      sha,
      ...repositoryScope(options)
    }));
  }
  catch (error) {
    if (error instanceof CrateReleaseError)
      return missingValue(error.message);
    throw error;
  }
}

/**
 Read the version a commit holds in the crate's manifest.

 @param options - validated await options

 @param run - runs a program call and reports its status and output

 @param sha - commit to read the manifest at

 @returns the version, or a miss that carries the reason `gh` or the manifest gave

 @example
 ```ts
 await readManifestVersion({ options, run, sha: 'abc1234' });
 ```
 */
async function readManifestVersion({
  options,
  run,
  sha,
}: {
  readonly options: AwaitOptions;
  readonly run: WaitRunner;
  readonly sha: string;
}): Promise<Lookup<string>> {
  /**
   The arguments of the read, or why the commit is not a hash the API could have written.
   */
  const args = manifestReadArguments({
    options,
    sha
  });
  if (!args.found)
    return args;
  /**
   Answer of `gh api`.
   */
  const contents = await run({
    program: ghProgram,
    args: args.value,
    cwd: options.root,
  });
  return contents.status === 0 ? readPackageVersion(contents.stdout) : missingValue(failureReason(contents));
}

/**
 Look for the publish run that the release run started, from one attempt on, sleeping between attempts.

 The attempts are a bounded chain (the attempt limit), each waiting for the previous one, so the recursion is as deep
 as the limit and no deeper.

 @param input - options, program runner, sleeper and the creation time of the release run

 @param attempt - number of this attempt, from one

 @returns the run, or a miss that carries the last reason

 @example
 ```ts
 await findPublishRun({ input, attempt: 1 });
 ```
 */
async function findPublishRun({
  input,
  attempt,
}: {
  readonly input: LookInput;
  readonly attempt: number;
}): Promise<Lookup<DispatchedRun>> {
  /**
   Parts of the input this attempt uses.
   */
  const {
    options,
    run,
    sleep,
    notBefore
  } = input;
  /**
   Answer of `gh run list`.
   */
  const listing = await run({
    program: ghProgram,
    args: buildDispatchedRunListArguments({
      workflow: options.workflow,
      limit: options.runLimit,
      ...repositoryScope(options)
    }),
    cwd: options.root,
  });
  /**
   The publish run, when the listing succeeded and holds it.
   */
  const lookup: Lookup<DispatchedRun> = listing.status === 0
    ? await selectPublishRun({
      listing: listing.stdout,
      notBefore,
      version: options.version,
      manifestPath: options.manifestPath,
      readVersion: async function readVersionAt(sha) {
        return await readManifestVersion({
          options,
          run,
          sha
        });
      }
    })
    : missingValue(failureReason(listing));
  if (lookup.found || (attempt >= options.attempts))
    return lookup;
  await sleep(options.pollIntervalSeconds * millisecondsPerSecond);
  return await findPublishRun({
    input,
    attempt: attempt + 1
  });
}

/**
 Wait for a release workflow run, then for the publish run it started, then check crates.io and the GitHub release.

 The release run is followed to its own end first, and the verdict is what the run says about itself, never the exit
 status of a watcher: `gh run watch --exit-status` exits 1 for its own failures too. When the run failed, the publish run
 is never looked for, because a failed release run starts none; the report names the failing job and step and the run
 whose log names it. When it succeeded, the publish run is the
 earliest manually started run of the publish workflow that was created at or after the release run and was built from
 a commit whose crate manifest holds the version being released (see `selectPublishRun`), so a publish run that
 someone else dispatched in the same window on another commit cannot be taken for it.

 @param options - validated await options

 @param run - runs a program call and reports its status and output

 @param sleep - waits the given milliseconds

 @param print - prints one line

 @returns zero when the whole release is published, the failure status otherwise

 @example
 ```ts
 await awaitRelease({ options, run, sleep, print: console.log });
 ```
 */
export async function awaitRelease({
  options,
  run,
  sleep,
  print,
}: {
  readonly options: AwaitOptions;
  readonly run: WaitRunner;
  readonly sleep: (milliseconds: number) => Promise<void>;
  readonly print: (line: string) => void;
}): Promise<number> {
  print(`${releaseWorkflow} run ${options.releaseRunId}`);
  /**
   What the release run says about itself: its creation time, and how it ended once it has.
   */
  const release = await followRunOutcome({
    options,
    run,
    sleep,
    print,
    workflow: releaseWorkflow,
    runId: options.releaseRunId
  });
  if (!release.found) {
    print(`RELEASE RUN NOT FINISHED: ${release.reason}`);
    return failureExitCode;
  }
  print(describeRunOutcome({
    workflow: releaseWorkflow,
    runId: options.releaseRunId,
    outcome: release.value
  }));
  if (!runPassed(release.value)) {
    for (const line of describeRunFailures(release.value))
      print(line);
    print(
      `RELEASE RUN FAILED: no publish was started. Read its log with: gh run view ${options.releaseRunId} --log-failed`,
    );
    return failureExitCode;
  }
  /**
   The publish run the release run started, or why none turned up.
   */
  const publishRun = await findPublishRun({
    input: {
      options,
      run,
      sleep,
      notBefore: release.value.createdAt
    },
    attempt: 1
  });
  if (!publishRun.found) {
    print(
      `no ${options.workflow} run was started since ${release.value.createdAt} within ${describeDuration(options.attempts * options.pollIntervalSeconds)}`,
    );
    print(`last result: ${publishRun.reason}`);
    return failureExitCode;
  }
  print(`publish run built from ${publishRun.value
    .headSha}`);
  return await checkPublishedRun({
    options,
    run,
    sleep,
    runId: publishRun.value
      .runId,
    print
  });
}
