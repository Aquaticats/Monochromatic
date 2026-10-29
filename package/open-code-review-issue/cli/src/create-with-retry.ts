/**
 Retry and reconciliation loop for one GitHub Issue creation.
 
 @module
 */

import {
  GitHubProcessError,
  GitHubProcessTimeoutError,
} from './github-process.ts';
import type {
  GitHubApiClient,
  GitHubRepository,
} from './github-model.ts';
import type { IncludedResponse, } from './github-response.ts';
import { isRecord, } from './json-record.ts';
import type { RenderedIssue, } from './issue-model.ts';
import { IssuePublicationError, } from './publication-error.ts';
import type {
  CreatedIssue,
  PublicationWait,
} from './publisher-model.ts';
import {
  readHighWater,
  reconcileCreate,
} from './reconcile.ts';

/**
 Successful GitHub Issue creation status.
 */
const HTTP_CREATED = 201;

/**
 Too-many-requests status.
 */
const HTTP_TOO_MANY_REQUESTS = 429;

/**
 Forbidden status used by exhausted GitHub rate limit.
 */
const HTTP_FORBIDDEN = 403;

/**
 Lowest server-error status.
 */
const HTTP_SERVER_ERROR_MINIMUM = 500;

/**
 Highest server-error status.
 */
const HTTP_SERVER_ERROR_MAXIMUM = 599;

/**
 Maximum retries after initial create request.
 */
const MAXIMUM_RETRIES = 3;

/**
 Base network and server retry delay.
 */
const TRANSIENT_RETRY_BASE_MS = 1_000;

/**
 Base rate retry delay when headers are absent.
 */
const RATE_RETRY_BASE_MS = 60_000;

/**
 One create attempt response or process failure.
 */
type CreateAttempt =
  | {
    readonly kind: 'response';
    readonly response: IncludedResponse;
  }
    | {
      readonly kind: 'process-failure';
      readonly error: GitHubProcessError | GitHubProcessTimeoutError;
    };

/**
 Creates one raw Issue request and captures retryable process failures.
 
 @param repository - Canonical destination identity.
 
 @param issue - Complete rendered Issue request.
 
 @param api - Authenticated GitHub API client.
 
 @returns HTTP response or retryable process failure.
 */
async function attemptCreate({
  repository,
  issue,
  api,
}: {
  readonly repository: GitHubRepository;
  readonly issue: RenderedIssue;
  readonly api: GitHubApiClient;
},): Promise<CreateAttempt> {
  try {
    return {
      kind: 'response',
      response: await api({
        method: 'POST',
        endpoint: `repos/${repository.owner}/${repository.name}/issues`,
        body: {
          title: issue.title,
          body: issue.body,
          labels: issue.labels,
        },
      },),
    };
  }
  catch (error: unknown) {
    if ((error instanceof GitHubProcessError)
      || (error instanceof GitHubProcessTimeoutError))
      {
      return {
        kind: 'process-failure',
        error,
      };
    }
    throw error;
  }
}

/**
 Parses successful Issue create response.
 
 @param issue - Requested Issue carrying input position.
 
 @param response - GitHub create response.
 
 @returns Confirmed created Issue identity.
 */
function parseCreatedIssue({
  issue,
  response,
}: {
  readonly issue: RenderedIssue;
  readonly response: IncludedResponse;
},): CreatedIssue {
  if ((response.status !== HTTP_CREATED)
    || (!isRecord(response.body,))
    || ((typeof response.body
      .number) !== 'number')
    || (!Number.isInteger(response.body
      .number,))
    || (response.body
      .number
      < 1)
    || ((typeof response.body
      .html_url) !== 'string'))
    {
    throw new IssuePublicationError(`Issue creation failed with HTTP ${String(response.status,)}`,);
  }
  return {
    position: issue.position,
    number: response.body
      .number,
    url: response.body
      .html_url,
  };
}

/**
 Determines whether response represents rate-limit rejection.
 
 @param response - GitHub create response.
 
 @returns Whether response belongs to rate-limit retry class.
 */
function isRateLimit(response: IncludedResponse,): boolean {
  return (response.status === HTTP_TOO_MANY_REQUESTS)
    || ((response.status === HTTP_FORBIDDEN)
      && ((response.headers['x-ratelimit-remaining'] === '0')
        || (response.headers['retry-after'] !== undefined)));
}

/**
 Determines whether response is ambiguous server failure.
 
 @param response - GitHub create response.
 
 @returns Whether response belongs to server retry class.
 */
function isServerError(response: IncludedResponse,): boolean {
  return (response.status >= HTTP_SERVER_ERROR_MINIMUM)
    && (response.status <= HTTP_SERVER_ERROR_MAXIMUM);
}

/**
 Reads positive numeric header as milliseconds.
 
 @param value - Header numeric text.
 
 @param multiplier - Unit conversion multiplier.
 
 @returns Positive converted value or zero for invalid header.
 */
function headerSeconds({
  value,
  multiplier,
}: {
  readonly value: string;
  readonly multiplier: number;
},): number {
  if (value === undefined) {
    return 0;
  }
  /**
   Numeric header candidate.
   */
  const parsed = Number(value,);
  return Number.isFinite(parsed,) && (parsed > 0) ? parsed * multiplier : 0;
}

/**
 Calculates header-aware rate delay for one retry index.
 
 @param response - Rate-limited GitHub response.
 
 @param retryIndex - Zero-based retry index.
 
 @param now - Epoch-millisecond clock.
 
 @returns Header delay or exponential fallback.
 */
function rateDelay({
  response,
  retryIndex,
  now,
}: {
  readonly response: IncludedResponse;
  readonly retryIndex: number;
  readonly now: () => number;
},): number {
  /**
   Raw retry-after header when supplied.
   */
  const retryAfterHeader = response.headers['retry-after'];
  if (retryAfterHeader !== undefined) {
    /**
     Retry-after delay converted from seconds.
     */
    const retryAfter = headerSeconds({
      value: retryAfterHeader,
      multiplier: 1_000,
    });
    if (retryAfter > 0) {
      return retryAfter;
    }
  }
  /**
   Raw absolute reset header when supplied.
   */
  const resetHeader = response.headers['x-ratelimit-reset'];
  if (resetHeader !== undefined) {
    /**
     Absolute reset instant converted from seconds.
     */
    const reset = headerSeconds({
      value: resetHeader,
      multiplier: 1_000,
    });
    if (reset > 0) {
      return Math.max(
        0,
        reset - now(),
      );
    }
  }
  return RATE_RETRY_BASE_MS * (2 ** retryIndex);
}

/**
 Creates one Issue with three bounded retries and ambiguity reconciliation.
 
 @param repository - Canonical destination identity.
 
 @param issue - Complete rendered Issue request.
 
 @param api - Authenticated GitHub API client.
 
 @param wait - Retry delay implementation.
 
 @param now - Epoch-millisecond clock.
 
 @returns Confirmed created or reconciled Issue identity.
 
 @example
 ```ts
 await createIssueWithRetry({ repository, issue, api, wait, now: Date.now });
 ```
 */
export async function createIssueWithRetry({
  repository,
  issue,
  api,
  wait,
  now,
}: {
  readonly repository: GitHubRepository;
  readonly issue: RenderedIssue;
  readonly api: GitHubApiClient;
  readonly wait: PublicationWait;
  readonly now: () => number;
},): Promise<CreatedIssue> {
  /**
   Shared number boundary recorded before first create attempt.
   */
  const highWater = await readHighWater({
    repository,
    api,
  });
  for (let attemptIndex = 0; attemptIndex <= MAXIMUM_RETRIES; attemptIndex += 1) {
    /**
     Current create response or process-level failure.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- retry attempt N+1 depends on attempt N response and reconciliation.
    const attempt = await attemptCreate({
      repository,
      issue,
      api,
    });
    if ((attempt.kind === 'response') && (attempt.response
      .status
      === HTTP_CREATED)) {
      return parseCreatedIssue({
        issue,
        response: attempt.response,
      });
    }
    /**
     Whether current failure may have created remote Issue.
     */
    const ambiguous = (attempt.kind === 'process-failure')
      || isServerError(attempt.response,);
    if (ambiguous) {
      /**
       Exact post-failure reconciliation result.
       */
      // oxlint-disable-next-line eslint/no-await-in-loop -- reconciliation must settle before retry decision for this attempt.
      const reconciliation = await reconcileCreate({
        repository,
        issue,
        highWater,
        api,
      },);
      if (reconciliation.kind === 'match') {
        return reconciliation.created;
      }
    }
    /**
     Whether current response qualifies for rate-aware retry.
     */
    const rateLimited = (attempt.kind === 'response') && isRateLimit(attempt.response,);
    /**
     Whether current failure belongs to settled retry classes.
     */
    const retryable = ambiguous || rateLimited;
    if ((!retryable) || (attemptIndex === MAXIMUM_RETRIES)) {
      /**
       Safe terminal status summary without Issue content.
       */
      const status = attempt.kind === 'response'
        ? `HTTP ${String(attempt.response
          .status,)}`
        : attempt.error
          .message;
      throw new IssuePublicationError(
        `Issue creation stopped after ${String(attemptIndex + 1,)} attempt(s): ${status}`,
      );
    }
    /**
     Settled delay before next mutative retry.
     */
    const retryDelay = rateLimited && (attempt.kind === 'response')
      ? rateDelay({
        response: attempt.response,
        retryIndex: attemptIndex,
        now,
      })
      : TRANSIENT_RETRY_BASE_MS * (2 ** attemptIndex);
    // oxlint-disable-next-line eslint/no-await-in-loop -- retry N+1 must wait for retry N backoff and mutation pacing.
    await wait(Math.max(
      TRANSIENT_RETRY_BASE_MS,
      retryDelay,
    ),);
  }
  throw new IssuePublicationError('Issue creation retry loop reached an impossible state',);
}
