/**
 Applied publication execution and final output projection.
 
 @module
 */

import type { AppliedResult, } from './cli-result.ts';
import {
  writeAppliedResult,
  writeCreatedIssues,
} from './cli-output.ts';
import type {
  GitHubApiClient,
  GitHubRepository,
} from './github-model.ts';
import { createPublicationInterruptControl, } from './interrupt.ts';
import type { RenderedIssue, } from './issue-model.ts';
import type { InputPosition, } from './model.ts';
import {
  AmbiguousReconciliationError,
  PublicationInterruptedError,
  PublicationStoppedError,
} from './publication-error.ts';
import { publishIssues, } from './publisher.ts';

/**
 Standard streams used after publication is authorized.
 */
export type AppliedStreams = {
  readonly stdout: NodeJS.WritableStream;
  readonly stderr: NodeJS.WritableStream;
};

/**
 Extracts multiple reconciliation URLs from nested stopping failure.
 
 @param cause - Underlying positioned stopping cause.
 
 @returns Optional matching URL property for machine result.
 */
function matchingUrlMetadata(
  cause: unknown,
): { readonly matchingUrls?: readonly string[]; } {
  return cause instanceof AmbiguousReconciliationError
    ? { matchingUrls: cause.urls, }
    : {};
}

/**
 Executes authorized publication and writes mode-specific final result.
 
 @param mode - Interactive human or non-interactive machine mode.
 
 @param repository - Canonical destination identity.
 
 @param issues - Fully authorized rendered Issues.
 
 @param withheldSecurityPositions - Security findings excluded from creation.
 
 @param api - Authenticated GitHub API client.
 
 @param streams - Standard output and diagnostic streams.
 
 @returns Zero for complete success or one for handled failure/interruption.
 
 @example
 ```ts
 await runAppliedPublication({ mode: 'non-interactive', repository, issues, api, streams });
 ```
 */
export async function runAppliedPublication({
  mode,
  repository,
  issues,
  withheldSecurityPositions,
  api,
  streams,
}: {
  readonly mode: 'interactive' | 'non-interactive';
  readonly repository: GitHubRepository;
  readonly issues: readonly RenderedIssue[];
  readonly withheldSecurityPositions: readonly InputPosition[];
  readonly api: GitHubApiClient;
  readonly streams: AppliedStreams;
},): Promise<number> {
  /**
   Two-stage signal control scoped to publication lifecycle.
   */
  using interrupts = createPublicationInterruptControl({});
  try {
    /**
     Complete successful serial publication result.
     */
    const result = await publishIssues({
      repository,
      issues,
      api,
      wait: interrupts.wait,
      shouldStop: interrupts.shouldStop,
    },);
    if (mode === 'interactive') {
      writeCreatedIssues({
        output: streams.stdout,
        created: result.created,
      });
    }
    else {
      writeAppliedResult({
        output: streams.stdout,
        result: {
          outcome: 'success',
          repository: repository.url,
          created: result.created,
          withheldSecurityPositions,
        },
      },);
    }
    return 0;
  }
  catch (error: unknown) {
    if (error instanceof PublicationInterruptedError) {
      if (mode === 'interactive') {
        streams.stderr
          .write(`${error.message}\n`,);
        writeCreatedIssues({
          output: streams.stdout,
          created: error.created,
        });
      }
      else {
        writeAppliedResult({
          output: streams.stdout,
          result: {
            outcome: 'interrupted',
            repository: repository.url,
            created: error.created,
            withheldSecurityPositions,
            failure: {
              position: error.position,
              message: error.message,
            },
          },
        },);
      }
      return 1;
    }
    if (error instanceof PublicationStoppedError) {
      if (mode === 'interactive') {
        streams.stderr
          .write(`${error.message}\n`,);
        writeCreatedIssues({
          output: streams.stdout,
          created: error.created,
        });
      }
      else {
        writeAppliedResult({
          output: streams.stdout,
          result: {
            outcome: 'failed',
            repository: repository.url,
            created: error.created,
            withheldSecurityPositions,
            failure: {
              position: error.position,
              message: error.message,
              ...matchingUrlMetadata(error.cause,),
            },
          },
        },);
      }
      return 1;
    }
    throw error;
  }
}
