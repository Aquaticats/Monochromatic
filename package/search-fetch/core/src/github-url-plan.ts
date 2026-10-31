/**
 GitHub URL to gh invocation planner.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { GitHubFetchPlan, } from './github-fetch-types.ts';
import { planForHost, } from './github-section-plan.ts';
import {
  parseGitHubUrl,
  validatePathSegments,
} from './github-url-validation.ts';

/**
 Logger root for pi-search-fetch after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: githubPlanLogger, },);
 ```
 */
const githubPlanLogger = tagged({ tag: 'pi-search-fetch', },);

/**
 Module logger.
 */
const l = tagged({
  tag: 'github-url-plan',
  l: githubPlanLogger,
},);

//region Public API

/**
 Plan gh invocations for one fetched URL and log the routing decision.
 
 @param url - absolute URL requested by the fetch tool
 
 @returns planned gh attempts, or a safe reason why gh cannot serve the URL
 
 @example
 ```ts
 planGitHubFetch({ url: 'https://github.com/cli/cli/blob/trunk/README.md' });
 ```
 */
function planGitHubFetch({ url, }: { readonly url: string; }): GitHubFetchPlan {
  /**
   Logger tagged for this planning call.
   */
  const innerL = tagged({
    tag: planGitHubFetch.name,
    l,
  },);
  /**
   Plan built from URL host and path shape.
   */
  const plan = buildGitHubFetchPlan(url,);
  innerL.debug(
    plan.planned
      ? plannedDescription({
        url,
        kind: plan.kind,
        attemptCount: plan.attempts
          .length,
      },)
      : `no gh plan for ${url}: ${plan.reason}`,
  );
  return plan;
}

/**
 Build one debug description for a planned GitHub fetch.
 
 @param url - absolute URL requested by the fetch tool
 
 @param kind - recognized GitHub request kind
 
 @param attemptCount - planned attempt count
 
 @returns debug description
 
 @example
 ```ts
 plannedDescription({ url: 'https://github.com/cli/cli', kind: 'repository-home', attemptCount: 1 });
 ```
 */
function plannedDescription(
  {
    url,
    kind,
    attemptCount,
  }: {
    readonly url: string;
    readonly kind: string;
    readonly attemptCount: number;
  },
): string {
  return `planned gh ${kind} fetch for ${url} using ${String(attemptCount,)} attempt(s)`;
}

/**
 Build one GitHub fetch plan from URL host and path shape.
 
 @param url - absolute URL requested by the fetch tool
 
 @returns planned gh attempts, or a safe reason why gh cannot serve the URL
 
 @example
 ```ts
 buildGitHubFetchPlan('https://github.com/cli/cli');
 ```
 */
function buildGitHubFetchPlan(url: string,): GitHubFetchPlan {
  /**
   Parsed URL, normalized host, and path segments.
   */
  const parsed = parseGitHubUrl(url,);
  if (!parsed.parsed)
    return {
      planned: false,
      reason: parsed.reason,
    };

  /**
   Path segment validation result.
   */
  const segmentValidation = validatePathSegments(parsed.segments,);
  if (!segmentValidation.valid)
    return {
      planned: false,
      reason: segmentValidation.reason,
    };

  return planForHost({
    host: parsed.host,
    url: parsed.url,
    segments: parsed.segments,
  },);
}

//endregion Public API

export { planGitHubFetch, };
