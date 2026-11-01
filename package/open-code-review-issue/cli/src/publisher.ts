/**
 Serial paced GitHub Issue publication.
 
 @module
 */

import { setTimeout as delay, } from 'node:timers/promises';

import { createIssueWithRetry, } from './create-with-retry.ts';
import type {
  GitHubApiClient,
  GitHubRepository,
} from './github-model.ts';
import type { RenderedIssue, } from './issue-model.ts';
import {
  PublicationInterruptedError,
  PublicationStoppedError,
} from './publication-error.ts';
import type {
  CreatedIssue,
  PublicationResult,
  PublicationStopCheck,
  PublicationWait,
} from './publisher-model.ts';

/**
 Minimum interval between mutative requests.
 */
const MUTATION_INTERVAL_MS = 1_000;

/**
 Default abortable-independent delay implementation.
 
 @param milliseconds - Delay duration.
 
 */
async function defaultWait(milliseconds: number,): Promise<void> {
  await delay(milliseconds,);
}

/**
 Default publication stop check.
 
 @returns False until signal-aware caller injects another check.
 */
function neverStop(): boolean {
  return false;
}

/**
 Creates rendered Issues serially with provider-aligned pacing.
 
 @param repository - Canonical destination identity.
 
 @param issues - Authorized rendered Issues in publication order.
 
 @param api - Authenticated GitHub API client.
 
 @param wait - Injectable delay implementation.
 
 @param now - Injectable epoch-millisecond clock for rate resets.
 
 @param shouldStop - First-interrupt state check at every serial boundary.
 
 @returns Complete successful creation identities.
 
 @example
 ```ts
 await publishIssues({ repository, issues, api });
 ```
 */
export async function publishIssues({
  repository,
  issues,
  api,
  wait = defaultWait,
  now = Date.now,
  shouldStop = neverStop,
}: {
  readonly repository: GitHubRepository;
  readonly issues: readonly RenderedIssue[];
  readonly api: GitHubApiClient;
  readonly wait?: PublicationWait;
  readonly now?: () => number;
  readonly shouldStop?: PublicationStopCheck;
},): Promise<PublicationResult> {
  /**
   Mutable result list scoped to this serial ownership boundary.
   */
  const created: CreatedIssue[] = [];
  for (const issue of issues) {
    if (shouldStop()) {
      throw new PublicationInterruptedError({
        created: [...created,],
        position: issue.position,
      },);
    }
    try {
      if (created.length > 0) {
        // oxlint-disable-next-line eslint/no-await-in-loop -- Issue N+1 mutation waits for Issue N pacing boundary.
        await wait(MUTATION_INTERVAL_MS,);
      }
      /**
       Confirmed created or reconciled Issue for current position.
       */
      // oxlint-disable-next-line eslint/no-await-in-loop -- create-only publication is deliberately serial and stops at first failure.
      const createdIssue = await createIssueWithRetry({
        repository,
        issue,
        api,
        wait,
        now,
      },);
      created.push(createdIssue,);
    }
    catch (error: unknown) {
      if (shouldStop()) {
        throw new PublicationInterruptedError({
          created: [...created,],
          position: issue.position,
        },);
      }
      throw new PublicationStoppedError({
        created: [...created,],
        position: issue.position,
        cause: error,
      },);
    }
    if (shouldStop()) {
      throw new PublicationInterruptedError({
        created: [...created,],
        position: issue.position,
      },);
    }
  }
  return { created, };
}

export { IssuePublicationError, } from './publication-error.ts';
