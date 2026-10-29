/**
 Complete CLI orchestration without process exit side effects.
 
 @module
 */

import { caughtValueText, } from '@monochromatic-dev/module-caught-value/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { runAppliedPublication, } from './applied-run.ts';
import {
  selectApplyPlan,
  SecurityAuthorityError,
} from './authority.ts';
import type { RunCliArguments, } from './cli-args.ts';
import {
  writeAppliedResult,
  writeCancellation,
  writeInteractiveSummary,
  writeJson,
} from './cli-output.ts';
import {
  checkGitHubCliVersion,
  createGitHubApiClient,
} from './github-client.ts';
import type {
  GitHubApiClient,
  GitHubRepository,
} from './github-model.ts';
import type { NormalizedInput, } from './model.ts';
import {
  buildNonInteractivePreview,
  buildPublicationPlan,
} from './plan.ts';
import type { PublicationPlan, } from './plan-model.ts';
import { preflightPublication, } from './preflight.ts';
import { promptForExplicitDecision, } from './interactive-prompts.ts';
import { selectInteractiveIssues, } from './interactive-selection.ts';
import type { PromptStreams, } from './interactive-model.ts';
import { loadCliInput, } from './input-source.ts';
import { selectRepository, } from './repository.ts';

/**
 Successful or clean-cancellation exit status.
 */
const EXIT_SUCCESS = 0;

/**
 Handled runtime or publication failure status.
 */
const EXIT_RUNTIME_FAILURE = 1;

/**
 Tagged root logger; messages never include finding content or paths.
 */
const l = tagged({ tag: 'run-cli', },);

/**
 Standard streams with optional TTY capability evidence.
 */
export type CliStreams = {
  readonly stdin: NodeJS.ReadableStream & { readonly isTTY?: boolean; };
  readonly stdout: NodeJS.WritableStream & { readonly isTTY?: boolean; };
  readonly stderr: NodeJS.WritableStream;
};

/**
 Reports handled runtime input or environment failure.
 */
export class CliRuntimeError extends Error {
  /**
   Creates runtime failure.
   
   @param message - User-facing runtime evidence and remediation.
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'CliRuntimeError';
  }
}

/**
 Projects process streams to Inquirer input/output names.
 
 @param streams - Process standard streams.
 
 @returns Explicit prompt streams.
 */
function promptStreams(streams: CliStreams,): PromptStreams {
  return {
    input: streams.stdin,
    output: streams.stdout,
  };
}

/**
 Requires visible and responsive streams for every interactive flow.
 
 @param streams - Process standard streams.
 
 @throws {@link CliRuntimeError} when stdin or stdout is not a TTY.
 */
function assertInteractiveTty(streams: CliStreams,): void {
  if ((streams.stdin
    .isTTY
    !== true) || (streams.stdout
      .isTTY
      !== true)) {
    throw new CliRuntimeError(
      'interactive mode requires TTY standard input and TTY standard output',
    );
  }
}

/**
 Builds destination-aware plan after GitHub preflight.
 
 @param input - Atomically normalized OCR input.
 
 @param repository - Canonical destination identity.
 
 @param api - Authenticated GitHub API client.
 
 @returns Complete internal publication plan.
 */
async function preparePlan({
  input,
  repository,
  api,
}: {
  readonly input: NormalizedInput;
  readonly repository: GitHubRepository;
  readonly api: GitHubApiClient;
},): Promise<PublicationPlan> {
  /**
   Existing-label and verified-source destination facts.
   */
  const preflight = await preflightPublication({
    repository,
    ...(input.resolvedHead === undefined ? {} : { resolvedHead: input.resolvedHead, }),
    api,
  },);
  return buildPublicationPlan({
    input,
    repository: repository.url,
    needsTriageLabel: preflight.needsTriageLabel,
    ...(preflight.sourceLink === undefined ? {} : { sourceLink: preflight.sourceLink, }),
  },);
}

/**
 Executes interactive selection, disclosure, and final confirmation.
 
 @returns Clean cancellation, success, or publication failure status.
 */
async function runInteractive({
  plan,
  repository,
  api,
  streams,
}: {
  readonly plan: PublicationPlan;
  readonly repository: GitHubRepository;
  readonly api: GitHubApiClient;
  readonly streams: CliStreams;
},): Promise<number> {
  /**
   Selected ordinary and individually confirmed security Issues.
   */
  const selection = await selectInteractiveIssues({
    plan,
    streams: promptStreams(streams,),
  },);
  if (selection.issues
    .length
    === 0) {
    writeCancellation(streams.stdout,);
    return EXIT_SUCCESS;
  }
  writeInteractiveSummary({
    output: streams.stdout,
    repository,
    issues: selection.issues,
    withheldCount: selection.withheldPositions
      .length,
  },);
  /**
   Explicit final batch mutation decision with no default.
   */
  const confirmed = await promptForExplicitDecision({
    message: 'Create these public GitHub Issues? Type yes or no',
    streams: promptStreams(streams,),
  },);
  if (!confirmed) {
    writeCancellation(streams.stdout,);
    return EXIT_SUCCESS;
  }
  return runAppliedPublication({
    mode: 'interactive',
    repository,
    issues: selection.issues,
    withheldSecurityPositions: selection.withheldPositions,
    api,
    streams,
  },);
}

/**
 Executes exact preview or explicitly authorized non-interactive publication.
 
 @returns Preview, success, or handled publication failure status.
 */
async function runNonInteractive({
  command,
  plan,
  repository,
  api,
  streams,
}: {
  readonly command: RunCliArguments;
  readonly plan: PublicationPlan;
  readonly repository: GitHubRepository;
  readonly api: GitHubApiClient;
  readonly streams: CliStreams;
},): Promise<number> {
  if (command.applyAuthority === undefined) {
    writeJson({
      output: streams.stdout,
      value: buildNonInteractivePreview(plan,),
    });
    return EXIT_SUCCESS;
  }
  try {
    /**
     Issues and redacted security positions authorized by invocation.
     */
    const selection = selectApplyPlan({
      plan,
      authority: command.applyAuthority,
    },);
    return await runAppliedPublication({
      mode: 'non-interactive',
      repository,
      issues: selection.issues,
      withheldSecurityPositions: selection.withheldPositions,
      api,
      streams,
    },);
  }
  catch (error: unknown) {
    if (!(error instanceof SecurityAuthorityError)) {
      throw error;
    }
    writeAppliedResult({
      output: streams.stdout,
      result: {
        outcome: 'failed',
        repository: repository.url,
        created: [],
        withheldSecurityPositions: buildNonInteractivePreview(plan,)
          .security
          .positions,
        failure: {
          message: error.message,
        },
      },
    },);
    return EXIT_RUNTIME_FAILURE;
  }
}

/**
 Executes input, GitHub preflight, planning, and selected mode for known repository.
 
 @returns Settled mode status.
 */
async function executeRepositoryRun({
  command,
  cwd,
  streams,
  repository,
}: {
  readonly command: RunCliArguments;
  readonly cwd: string;
  readonly streams: CliStreams;
  readonly repository: GitHubRepository;
},): Promise<number> {
  /**
   Validated OCR input loaded before GitHub operations.
   */
  const input = await loadCliInput({ input: command.input, });
  if (input.findings
    .length
    === 0) {
    throw new CliRuntimeError('OCR input contains no findings to publish',);
  }
  await checkGitHubCliVersion({ cwd, });
  /**
   Authenticated GitHub API process client.
   */
  const api = createGitHubApiClient({ cwd, });
  /**
   Destination-aware complete publication plan.
   */
  const plan = await preparePlan({
    input,
    repository,
    api,
  });
  l.debug(`prepared ${String(plan.issues
    .length,)} Issue(s) in ${command.mode} mode`,);
  return command.mode === 'interactive'
    ? runInteractive({
      plan,
      repository,
      api,
      streams,
    })
    : runNonInteractive({
      command,
      plan,
      repository,
      api,
      streams,
    });
}

/**
 Executes one validated run command through input, preflight, and selected mode.
 
 @returns Settled mode status.
 
 @example
 ```ts
 await executeRun({ command, cwd: process.cwd(), streams });
 ```
 */
export async function executeRun({
  command,
  cwd,
  streams,
}: {
  readonly command: RunCliArguments;
  readonly cwd: string;
  readonly streams: CliStreams;
},): Promise<number> {
  if (command.mode === 'interactive') {
    assertInteractiveTty(streams,);
  }
  /**
   Canonical explicit or inferred destination.
   */
  const repository = await selectRepository({
    ...(command.repositoryUrl === undefined ? {} : { explicitUrl: command.repositoryUrl, }),
    cwd,
  },);
  try {
    return await executeRepositoryRun({
      command,
      cwd,
      streams,
      repository,
    },);
  }
  catch (error: unknown) {
    if ((command.mode !== 'non-interactive')
      || (command.applyAuthority === undefined))
      {
      throw error;
    }
    /**
     Safe pre-publication applied-run failure message.
     */
    const message = caughtValueText(error,);
    l.error(message,);
    writeAppliedResult({
      output: streams.stdout,
      result: {
        outcome: 'failed',
        repository: repository.url,
        created: [],
        withheldSecurityPositions: [],
        failure: { message, },
      },
    },);
    return EXIT_RUNTIME_FAILURE;
  }
}
