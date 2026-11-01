/**
 gh invocation and attempt builders for GitHub URL plans.
 
 @module
 */

import {
  EMPTY_SEGMENT,
  GH_API_SUBCOMMAND,
  GH_COMMENTS_FLAG,
  GH_DIFF_ACCEPT_HEADER,
  GH_DIRECTORY_LISTING_JQ,
  GH_HEADER_FLAG,
  GH_JQ_FLAG,
  GH_RAW_ACCEPT_HEADER,
  URL_SLASH,
} from './github-url-plan-constants.ts';
import type {
  GhFetchAttempt,
  GhInvocation,
  GitHubFetchRequestKind,
  GitHubRefPathSplit,
  PlannedGitHubFetch,
  RepositoryContentContext,
  UnplannedGitHubFetch,
} from './github-fetch-types.ts';
import { decodeUrlText, } from './github-url-validation.ts';

//region Plan shapes

/**
 Build one plan running a single required gh invocation.
 
 @param kind - recognized GitHub request kind
 
 @param label - attempt name used in diagnostics
 
 @param args - gh argument vector
 
 @returns planned result holding one attempt
 
 @example
 ```ts
 singleAttemptPlan({ kind: 'gist', label: 'gist abc123', args: ['gist', 'view', 'abc123'] });
 ```
 */
function singleAttemptPlan(
  {
    kind,
    label,
    args,
  }: {
    readonly kind: GitHubFetchRequestKind;
    readonly label: string;
    readonly args: readonly string[];
  },
): PlannedGitHubFetch {
  return planned({
    kind,
    attempts: [
      {
        label,
        invocations: [
          requiredInvocation(args,),
        ],
      },
    ],
  },);
}

/**
 Build one plan reading a thread body and tolerating a failed or empty comment read.
 
 @param kind - recognized GitHub request kind
 
 @param label - attempt name used in diagnostics
 
 @param viewArgs - gh arguments printing thread metadata and body
 
 @returns planned result holding one attempt with two invocations
 
 @example
 ```ts
 threadAttemptPlan({ kind: 'issue-thread', label: 'issue cli/cli#1', viewArgs: ['issue', 'view', 'https://github.com/cli/cli/issues/1'] });
 ```
 */
function threadAttemptPlan(
  {
    kind,
    label,
    viewArgs,
  }: {
    readonly kind: GitHubFetchRequestKind;
    readonly label: string;
    readonly viewArgs: readonly string[];
  },
): PlannedGitHubFetch {
  return planned({
    kind,
    attempts: [
      {
        label,
        invocations: [
          requiredInvocation(viewArgs,),
          optionalInvocation([
            ...viewArgs,
            GH_COMMENTS_FLAG,
          ],),
        ],
      },
    ],
  },);
}

/**
 Build one plan rendering one commit or comparison as a unified diff.
 
 @param kind - recognized GitHub request kind
 
 @param label - attempt name used in diagnostics
 
 @param endpoint - REST endpoint returning the diff
 
 @returns planned result holding one diff attempt
 
 @example
 ```ts
 diffAttemptPlan({ kind: 'commit-diff', label: 'commit abc diff', endpoint: '/repos/cli/cli/commits/abc' });
 ```
 */
function diffAttemptPlan(
  {
    kind,
    label,
    endpoint,
  }: {
    readonly kind: GitHubFetchRequestKind;
    readonly label: string;
    readonly endpoint: string;
  },
): PlannedGitHubFetch {
  return singleAttemptPlan({
    kind,
    label,
    args: [
      GH_API_SUBCOMMAND,
      GH_HEADER_FLAG,
      GH_DIFF_ACCEPT_HEADER,
      endpoint,
    ],
  },);
}

/**
 Build one attempt per reference and path split candidate.
 
 @param owner - repository owner login
 
 @param repo - repository name
 
 @param splits - ordered split candidates
 
 @param listing - whether the response is a directory listing rather than file bytes
 
 @returns attempts in split candidate order, shortest reference first
 
 @example
 ```ts
 contentAttempts({ owner: 'cli', repo: 'cli', splits: [{ ref: 'trunk', path: 'README.md' }], listing: false });
 ```
 */
function contentAttempts(
  {
    owner,
    repo,
    splits,
    listing,
  }: RepositoryContentContext,
): readonly GhFetchAttempt[] {
  return splits.map(function mapSplit(
    split: GitHubRefPathSplit,
    index: number,
  ): GhFetchAttempt {
    /**
     Repository path description used in the attempt label.
     */
    const pathLabel = split.path === EMPTY_SEGMENT
      ? 'repository root'
      : `path ${split.path}`;
    return {
      label: `${listing ? 'directory listing' : 'file content'} attempt ${String(index + 1,)} of ${String(splits.length,)} at ref ${split.ref} ${pathLabel}`,
      invocations: [
        requiredInvocation(contentsArgs({
          owner,
          repo,
          split,
          listing,
        },),),
      ],
    };
  },);
}

//endregion Plan shapes

//region Argument builders

/**
 Build gh arguments for one repository contents read.
 
 @param owner - repository owner login
 
 @param repo - repository name
 
 @param split - reference and path candidate
 
 @param listing - whether to project a directory listing instead of file bytes
 
 @returns gh argument vector
 
 @example
 ```ts
 contentsArgs({ owner: 'cli', repo: 'cli', split: { ref: 'trunk', path: 'docs' }, listing: true });
 ```
 */
function contentsArgs(
  {
    owner,
    repo,
    split,
    listing,
  }: {
    readonly owner: string;
    readonly repo: string;
    readonly split: GitHubRefPathSplit;
    readonly listing: boolean;
  },
): readonly string[] {
  /**
   Reference decoded from URL path form and re-encoded as one whole query value.
   
   A path segment keeps `+` and `&` literally,
   while a query value would read `+` as a space and `&` as a parameter separator.
   */
  const referenceQueryValue = encodeURIComponent(decodeUrlText(split.ref,),);
  /**
   Contents endpoint shared by both response shapes.
   */
  const endpoint = `${URL_SLASH}repos/${owner}/${repo}/contents/${split.path}?ref=${referenceQueryValue}`;
  return listing
    ? [
      GH_API_SUBCOMMAND,
      endpoint,
      GH_JQ_FLAG,
      GH_DIRECTORY_LISTING_JQ,
    ]
    : [
      GH_API_SUBCOMMAND,
      GH_HEADER_FLAG,
      GH_RAW_ACCEPT_HEADER,
      endpoint,
    ];
}

/**
 Build one invocation whose failure fails its attempt.
 
 @param args - gh argument vector
 
 @returns required invocation
 
 @example
 ```ts
 requiredInvocation(['repo', 'view', 'cli/cli']);
 ```
 */
function requiredInvocation(args: readonly string[],): GhInvocation {
  return {
    args,
    required: true,
  };
}

/**
 Build one invocation whose failure only shortens its attempt output.
 
 @param args - gh argument vector
 
 @returns optional invocation
 
 @example
 ```ts
 optionalInvocation(['issue', 'view', 'https://github.com/cli/cli/issues/1', '--comments']);
 ```
 */
function optionalInvocation(args: readonly string[],): GhInvocation {
  return {
    args,
    required: false,
  };
}

//endregion Argument builders

//region Result constructors

/**
 Build one successful plan.
 
 @param kind - recognized GitHub request kind
 
 @param attempts - attempts in preference order
 
 @returns planned result
 
 @example
 ```ts
 planned({ kind: 'gist', attempts: [] });
 ```
 */
function planned(
  {
    kind,
    attempts,
  }: {
    readonly kind: GitHubFetchRequestKind;
    readonly attempts: readonly GhFetchAttempt[];
  },
): PlannedGitHubFetch {
  return {
    planned: true,
    kind,
    attempts,
  };
}

/**
 Build one refusal to plan gh attempts.
 
 @param reason - safe explanation forwarded into fallback diagnostics
 
 @returns unplanned result
 
 @example
 ```ts
 unplanned('GitHub section /wiki/ has no gh equivalent');
 ```
 */
function unplanned(reason: string,): UnplannedGitHubFetch {
  return {
    planned: false,
    reason,
  };
}

//endregion Result constructors

export {
  contentAttempts,
  diffAttemptPlan,
  planned,
  requiredInvocation,
  singleAttemptPlan,
  threadAttemptPlan,
  unplanned,
};
