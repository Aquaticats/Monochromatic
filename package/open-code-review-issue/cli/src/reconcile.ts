/**
 Ambiguous GitHub Issue creation reconciliation.
 
 @module
 */

import type {
  GitHubApiClient,
  GitHubRepository,
} from './github-model.ts';
import { isRecord, } from './json-record.ts';
import type { RenderedIssue, } from './issue-model.ts';
import {
  AmbiguousReconciliationError,
  IssuePublicationError,
} from './publication-error.ts';
import type { CreatedIssue, } from './publisher-model.ts';

/**
 Successful GitHub read status.
 */
const HTTP_OK = 200;

/**
 Confirmed missing Issue status.
 */
const HTTP_NOT_FOUND = 404;

/**
 Reconciliation result proving match or absence.
 */
export type ReconciliationResult =
  | { readonly kind: 'none'; }
    | {
      readonly kind: 'match';
      readonly created: CreatedIssue;
    };

/**
 Builds repository REST endpoint prefix.
 
 @param repository - Canonical destination identity.
 
 @returns REST owner/name prefix.
 */
function repositoryEndpoint(repository: GitHubRepository,): string {
  return `repos/${repository.owner}/${repository.name}`;
}

/**
 Reads greatest current shared Issue or pull-request number.
 
 @param repository - Canonical destination identity.
 
 @param api - Authenticated GitHub API client.
 
 @returns Current high-water number or zero for empty repository.
 
 @throws {@link IssuePublicationError} when response is not valid list.
 
 @example
 ```ts
 await readHighWater({ repository, api });
 ```
 */
export async function readHighWater({
  repository,
  api,
}: {
  readonly repository: GitHubRepository;
  readonly api: GitHubApiClient;
},): Promise<number> {
  /**
   Most-recent Issue list response.
   */
  const response = await api({
    method: 'GET',
    endpoint: `${repositoryEndpoint(repository,)}/issues?state=all&sort=created&direction=desc&per_page=1`,
  },);
  if ((response.status !== HTTP_OK) || (!Array.isArray(response.body,))) {
    throw new IssuePublicationError(
      `Issue high-water lookup failed with HTTP ${String(response.status,)}`,
    );
  }
  /**
   Narrowed untrusted list values.
   */
  const items: readonly unknown[] = response.body;
  return items
    .reduce<number>(
    function greatestNumber(
      greatest,
      item,
    ): number {
      if ((!isRecord(item,))
        || ((typeof item.number) !== 'number')
        || (!Number.isInteger(item.number,))
        || (item.number < 1))
        {
        throw new IssuePublicationError('Issue high-water response contains invalid number',);
      }
      return Math.max(
        greatest,
        item.number,
      );
    },
    0,
    );
}

/**
 Reads one Issue or pull request and compares exact generated content.
 
 @param repository - Canonical destination identity.
 
 @param issue - Generated request being reconciled.
 
 @param number - Candidate shared Issue number.
 
 @param api - Authenticated GitHub API client.
 
 @returns Matching created identity or no-match object.
 
 @throws {@link IssuePublicationError} when candidate read fails.
 */
async function compareCandidate({
  repository,
  issue,
  number,
  api,
}: {
  readonly repository: GitHubRepository;
  readonly issue: RenderedIssue;
  readonly number: number;
  readonly api: GitHubApiClient;
},): Promise<ReconciliationResult> {
  /**
   Candidate Issue-or-pull-request response.
   */
  const response = await api({
    method: 'GET',
    endpoint: `${repositoryEndpoint(repository,)}/issues/${String(number,)}`,
  },);
  if (response.status === HTTP_NOT_FOUND) {
    return { kind: 'none', };
  }
  if ((response.status !== HTTP_OK) || (!isRecord(response.body,))) {
    throw new IssuePublicationError(
      `reconciliation read for number ${String(number,)} failed with HTTP ${String(response.status,)}`,
    );
  }
  if ((response.body
    .title
    !== issue.title)
    || (response.body
      .body
      !== issue.body)
    || ((typeof response.body
      .html_url) !== 'string'))
    {
    return { kind: 'none', };
  }
  return {
    kind: 'match',
    created: {
      position: issue.position,
      number,
      url: response.body
        .html_url,
    },
  };
}

/**
 Scans every number above pre-request high-water for exact request match.
 
 @param repository - Canonical destination identity.
 
 @param issue - Generated request being reconciled.
 
 @param highWater - Greatest number recorded before create request.
 
 @param api - Authenticated GitHub API client.
 
 @returns No match or one confirmed created Issue.
 
 @throws {@link AmbiguousReconciliationError} when multiple exact matches exist.
 
 @throws {@link IssuePublicationError} when any owning read fails.
 
 @example
 ```ts
 await reconcileCreate({ repository, issue, highWater: 0, api });
 ```
 */
export async function reconcileCreate({
  repository,
  issue,
  highWater,
  api,
}: {
  readonly repository: GitHubRepository;
  readonly issue: RenderedIssue;
  readonly highWater: number;
  readonly api: GitHubApiClient;
},): Promise<ReconciliationResult> {
  /**
   Greatest number visible after ambiguous failure.
   */
  const currentHighWater = await readHighWater({
    repository,
    api,
  });
  /**
   Exact matches accumulated for multiple-match terminal handling.
   */
  const matches: CreatedIssue[] = [];
  for (let number = highWater + 1; number <= currentHighWater; number += 1) {
    /**
     Exact-comparison result for current candidate number.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- every newer number must be checked in ascending order before deciding match count.
    const result = await compareCandidate({
      repository,
      issue,
      number,
      api,
    },);
    if (result.kind === 'match') {
      matches.push(result.created,);
    }
  }
  if (matches.length === 0) {
    return { kind: 'none', };
  }
  if (matches.length === 1) {
    /**
     Sole confirmed exact match.
     */
    const [created,] = matches;
    if (created !== undefined) {
      return {
        kind: 'match',
        created,
      };
    }
  }
  throw new AmbiguousReconciliationError({
    urls: matches.map(function matchUrl(match,): string {
      return match.url;
    },),
  },);
}
