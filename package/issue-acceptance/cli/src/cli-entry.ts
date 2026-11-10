/**
 Runs one parsed command and reports the exit status.

 The entry point owns the streams and the exit status;
 this module owns the work each subcommand does,
 so tests drive every subcommand with fakes.

 @module
 */

import { writeFile, } from 'node:fs/promises';

import {
  parseArguments,
  USAGE,
} from './cli-args.ts';
import { PlanError, } from './errors.ts';
import {
  createGitReader,
  type GitReader,
  type GitReaderOptions,
} from './git-reader.ts';
import {
  createIssueTracker,
  type IssueTracker,
  type IssueTrackerOptions,
} from './issue-tracker.ts';
import { readPlan, } from './plan-read.ts';
import {
  groupReadyBugs,
  READY_BUG_LABELS,
  renderReadyBugReport,
} from './ready-bugs.ts';
import { settlePlan, } from './settle.ts';
import { showIssues, } from './show.ts';

/**
 Status of a run that did everything it was asked.
 */
export const EXIT_OK: number = 0;

/**
 Status of a run whose checks or mutations failed.
 */
export const EXIT_FAILED: number = 1;

/**
 Status of a run that was invoked wrongly.
 */
export const EXIT_MISUSE: number = 2;

/**
 Where a run's output goes.
 */
export type CliStreams = {
  readonly write: (line: string,) => void;
  readonly errorWrite: (line: string,) => void;
};

/**
 One invocation, including the factories tests replace with fakes.
 */
export type RunCliRequest = {
  readonly arguments: readonly string[];
  readonly streams: CliStreams;
  readonly createReader?: (options: GitReaderOptions,) => GitReader;
  readonly createTracker?: (options: IssueTrackerOptions,) => IssueTracker;
};

/**
 The settle subcommand's arguments.
 */
export type RunSettleRequest = {
  readonly request: RunCliRequest;
  readonly planPath: string;
  readonly apply: boolean;
};

/**
 The show subcommand's arguments.
 */
export type RunShowRequest = {
  readonly request: RunCliRequest;
  readonly repository: string;
  readonly numbers: readonly number[];
  readonly mode: 'full' | 'acceptance';
};

/**
 The ready-bugs subcommand's arguments.
 */
export type RunReadyBugsRequest = {
  readonly request: RunCliRequest;
  readonly repository: string;
  readonly outPath?: string;
};

/**
 Runs the settle subcommand.
 
 @param request - Invocation whose streams receive the verdicts.
 
 @param planPath - Plan file to settle.
 
 @param apply - Whether to write on GitHub.
 
 @returns Exit status.
 
 @throws {@link PlanError} when the plan cannot be read or does not validate.
 
 @example
 ```ts
 const status = await runSettle({ request, planPath: 'plan.json', apply: false });
 ```
 */
async function runSettle({
  request,
  planPath,
  apply,
}: RunSettleRequest,): Promise<number> {
  /**
   Validated plan, read before any child process runs.
   */
  const plan = await readPlan({ path: planPath, },);
  /**
   Reader for the plan's checkout and branch.
   */
  const reader = (request.createReader ?? createGitReader)({
    checkout: plan.checkout,
    branch: plan.branch,
  },);
  /**
   Tracker for the plan's repository.
   */
  const tracker = (request.createTracker ?? createIssueTracker)({ repository: plan.repository, },);
  /**
   What the run decided and did.
   */
  const report = await settlePlan({
    plan,
    reader,
    tracker,
    apply,
    write: function writeLine(line,) {
      request.streams.write(`${line}\n`,);
    },
  },);
  return report.failed ? EXIT_FAILED : EXIT_OK;
}

/**
 Runs the show subcommand.
 
 @param request - Invocation whose streams receive the issue text.
 
 @param repository - Repository the issues live in.
 
 @param numbers - Issues to print.
 
 @param mode - Whether to print each issue whole or only its acceptance.
 
 @returns Exit status.
 
 @example
 ```ts
 const status = await runShow({ request, repository: 'OWNER/NAME', numbers: [1], mode: 'acceptance' });
 ```
 */
async function runShow({
  request,
  repository,
  numbers,
  mode,
}: RunShowRequest,): Promise<number> {
  /**
   Tracker the issues are read from.
   */
  const tracker = (request.createTracker ?? createIssueTracker)({ repository, },);
  await showIssues({
    numbers,
    tracker,
    mode,
    write: request.streams
      .write,
  },);
  return EXIT_OK;
}

/**
 Runs the ready-bugs subcommand.
 
 @param request - Invocation whose streams receive the grouping.
 
 @param repository - Repository the issues live in.
 
 @param outPath - File the grouping is also written to as JSON, when named.
 
 @returns Exit status.
 
 @example
 ```ts
 const status = await runReadyBugs({ request, repository: 'OWNER/NAME', outPath: undefined });
 ```
 */
async function runReadyBugs({
  request,
  repository,
  outPath,
}: RunReadyBugsRequest,): Promise<number> {
  /**
   Tracker the open issues are read from.
   */
  const tracker = (request.createTracker ?? createIssueTracker)({ repository, },);
  /**
   Issues grouped by the first package each names.
   */
  const report = groupReadyBugs({
    listings: await tracker.listOpenWithLabels(READY_BUG_LABELS,),
  },);
  for (const line of renderReadyBugReport({ report, },)) {
    request.streams
      .write(`${line}\n`,);
  }
  if (outPath !== undefined) {
    await writeFile(
      outPath,
      `${JSON.stringify(
        report,
        undefined,
        1,
      )}\n`,
    );
  }
  return EXIT_OK;
}

/**
 Runs one command line and reports the exit status.
 
 @param request - Tokens, streams, and optional factories.
 
 @returns The status the process should exit with.
 
 @throws {@link CliInvocationError} when the tokens do not form a command.
 
 @example
 ```ts
 const status = await runCli({ arguments: ['--help'], streams: { write, errorWrite } });
 ```
 */
export function runCli(request: RunCliRequest,): Promise<number> {
  /**
   Command the tokens form.
   */
  const command = parseArguments({ tokens: request.arguments, },);
  if (command.kind === 'help') {
    request.streams
      .write(`${USAGE}\n`,);
    return Promise.resolve(EXIT_OK,);
  }
  if (command.kind === 'settle') {
    return runSettle({
      request,
      planPath: command.planPath,
      apply: command.apply,
    },);
  }
  if (command.kind === 'show') {
    return runShow({
      request,
      repository: command.repository,
      numbers: command.numbers,
      mode: command.mode,
    },);
  }
  return runReadyBugs({
    request,
    repository: command.repository,
    ...(command.outPath === undefined ? {} : { outPath: command.outPath, }),
  },);
}
