import {
  failureExitCode,
  ghProgram,
  millisecondsPerSecond,
  releaseWorkflow,
} from './constants.ts';
import type { AwaitOptions } from './await-options.ts';
import {
  type Lookup,
  missingValue,
} from './lookup.ts';
import { describeDuration } from './publish-results.ts';
import {
  type DispatchedRun,
  parseRunCreatedAt,
  selectDispatchedRun,
} from './run-results.ts';
import {
  buildDispatchedRunListArguments,
  buildRunViewArguments,
  buildRunWatchArguments,
} from './service-arguments.ts';
import {
  checkPublishedRun,
  failureReason,
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
    ? selectDispatchedRun({
      listing: listing.stdout,
      notBefore
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
 Read when the release run was created.

 @param options - validated await options

 @param run - runs a program call and reports its status and output

 @returns the creation time, or a miss that carries the reason `gh` or its answer gave

 @example
 ```ts
 await releaseRunCreatedAt({ options, run });
 ```
 */
async function releaseRunCreatedAt({
  options,
  run,
}: {
  readonly options: AwaitOptions;
  readonly run: WaitRunner;
}): Promise<Lookup<string>> {
  /**
   Answer of `gh run view`.
   */
  const view = await run({
    program: ghProgram,
    args: buildRunViewArguments({
      runId: options.releaseRunId,
      ...repositoryScope(options)
    }),
    cwd: options.root,
  });
  return view.status === 0 ? parseRunCreatedAt(view.stdout) : missingValue(failureReason(view));
}

/**
 Wait for a release workflow run, then for the publish run it started, then check crates.io and the GitHub release.

 The release run is watched to its end first. When it fails, the publish run is never looked for, because a failed
 release run starts none; the report names the run so its log can be read. When it succeeds, the publish run is the
 earliest manually started run of the publish workflow created at or after the release run, so another publish run
 cannot be taken for it.

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
   Outcome of watching the release run to its end.
   */
  const watched = await run({
    program: ghProgram,
    args: buildRunWatchArguments({
      runId: options.releaseRunId,
      ...repositoryScope(options)
    }),
    cwd: options.root,
  });
  print(`watch exit ${String(watched.status)}`);
  if (watched.status !== 0) {
    print(
      `RELEASE RUN FAILED: no publish was started. Read its log with: gh run view ${options.releaseRunId} --log-failed`,
    );
    return failureExitCode;
  }
  /**
   When the release run was created, or why that is unknown.
   */
  const createdAt = await releaseRunCreatedAt({
    options,
    run
  });
  if (!createdAt.found) {
    print(`RELEASE RUN UNREADABLE: ${createdAt.reason}`);
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
      notBefore: createdAt.value
    },
    attempt: 1
  });
  if (!publishRun.found) {
    print(
      `no ${options.workflow} run was started since ${createdAt.value} within ${describeDuration(options.attempts * options.pollIntervalSeconds)}`,
    );
    print(`last result: ${publishRun.reason}`);
    return failureExitCode;
  }
  print(`publish run built from ${publishRun.value
    .headSha}`);
  return await checkPublishedRun({
    options,
    run,
    runId: publishRun.value
      .runId,
    print
  });
}
