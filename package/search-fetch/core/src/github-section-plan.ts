/**
 GitHub host and repository section dispatch for gh fetch plans.
 
 @module
 */

import {
  API_GITHUB_HOST,
  BLOB_SECTION,
  BLAME_SECTION,
  COMMIT_SECTION,
  COMPARE_SECTION,
  GH_API_SUBCOMMAND,
  GIST_GITHUB_HOST,
  ISSUES_SECTION,
  PULL_SECTION,
  RAW_GITHUB_CONTENT_HOST,
  RAW_SECTION,
  RELEASES_SECTION,
  SECTION_ARGUMENT_OFFSET,
  TREE_SECTION,
  URL_SLASH,
} from './github-url-plan-constants.ts';
import type {
  GitHubFetchPlan,
  RepositorySectionContext,
} from './github-fetch-types.ts';
import {
  singleAttemptPlan,
  unplanned,
} from './gh-invocation-plan.ts';
import {
  validateEndpointFragment,
  validateTokenArgument,
} from './github-url-validation.ts';
import {
  planDirectorySection,
  planFileContentSection,
  planRawContentHost,
  repositoryTarget,
} from './github-content-plan.ts';
import {
  planCommitSection,
  planCompareSection,
  planIssueSection,
  planPullRequestSection,
  planReleasesSection,
} from './github-record-plan.ts';

//region Host dispatch

/**
 Plan one mapped GitHub host.
 
 @param host - normalized mapped host
 
 @param url - parsed URL
 
 @param segments - validated path segments
 
 @returns gh plan for the host
 
 @example
 ```ts
 planForHost({ host: 'gist.github.com', url: new URL('https://gist.github.com/abc123'), segments: ['abc123'] });
 ```
 */
function planForHost(
  {
    host,
    url,
    segments,
  }: {
    readonly host: string;
    readonly url: URL;
    readonly segments: readonly string[];
  },
): GitHubFetchPlan {
  if (host === GIST_GITHUB_HOST)
    return planGist(segments,);
  if (host === API_GITHUB_HOST)
    return planApiEndpoint(url,);
  if (host === RAW_GITHUB_CONTENT_HOST)
    return planRawContentHost(segments,);
  return planRepositoryPath(segments,);
}

/**
 Plan one gist URL from its last path segment.
 
 @param segments - validated path segments
 
 @returns gh plan for the gist
 
 @example
 ```ts
 planGist(['Aquaticat', 'a6570831a0e0a6ca2c86919b54b3c4d9']);
 ```
 */
function planGist(segments: readonly string[],): GitHubFetchPlan {
  /**
   Gist identifier segment.
   */
  const gistId = segments.at(-1,);
  if (gistId === undefined)
    return unplanned('Gist URL names no gist identifier',);

  /**
   Gist identifier validation result.
   */
  const validation = validateTokenArgument({
    value: gistId,
    label: 'gist identifier',
  },);
  if (!validation.safe)
    return unplanned(validation.reason,);

  return singleAttemptPlan({
    kind: 'gist',
    label: `gist ${gistId}`,
    args: [
      'gist',
      'view',
      gistId,
    ],
  },);
}

/**
 Plan one GitHub REST API URL as a direct `gh api` call.
 
 @param url - parsed api.github.com URL
 
 @returns gh plan for the API endpoint
 
 @example
 ```ts
 planApiEndpoint(new URL('https://api.github.com/repos/cli/cli'));
 ```
 */
function planApiEndpoint(url: URL,): GitHubFetchPlan {
  /**
   API endpoint path plus query string.
   */
  const endpoint = `${url.pathname}${url.search}`;
  /**
   Endpoint validation result.
   */
  const validation = validateEndpointFragment({
    value: endpoint,
    label: 'GitHub API endpoint',
  },);
  if (!validation.safe)
    return unplanned(validation.reason,);

  return singleAttemptPlan({
    kind: 'api-endpoint',
    label: `api endpoint ${endpoint}`,
    args: [
      GH_API_SUBCOMMAND,
      endpoint,
    ],
  },);
}

//endregion Host dispatch

//region Repository dispatch

/**
 Plan one repository path on the GitHub host.
 
 @param segments - validated path segments
 
 @returns gh plan for the repository path
 
 @example
 ```ts
 planRepositoryPath(['cli', 'cli', 'blob', 'trunk', 'README.md']);
 ```
 */
function planRepositoryPath(segments: readonly string[],): GitHubFetchPlan {
  /**
   Owner, name, and section segments, each absent when the URL is shorter.
   */
  const [
    owner,
    repo,
    section,
  ] = segments;
  if (owner === undefined)
    return unplanned('GitHub root URL names no repository',);
  if (repo === undefined)
    return unplanned(`GitHub owner URL ${URL_SLASH}${owner} has no gh equivalent`,);

  /**
   Repository target resolved from the leading two segments.
   */
  const target = repositoryTarget(segments,);
  if (!target.found)
    return unplanned(target.reason,);

  if (section === undefined)
    return singleAttemptPlan({
      kind: 'repository-home',
      label: `repository ${target.owner}/${target.repo} overview and README`,
      args: [
        'repo',
        'view',
        `${target.owner}${URL_SLASH}${target.repo}`,
      ],
    },);

  return planRepositorySection({
    owner: target.owner,
    repo: target.repo,
    section,
    rest: segments.slice(SECTION_ARGUMENT_OFFSET,),
  },);
}

/**
 Plan one repository section.
 
 @param owner - repository owner login
 
 @param repo - repository name
 
 @param section - section name following the repository
 
 @param rest - path segments after the section name
 
 @returns gh plan for the section
 
 @example
 ```ts
 planRepositorySection({ owner: 'cli', repo: 'cli', section: 'tree', rest: ['trunk', 'docs'] });
 ```
 */
function planRepositorySection(
  {
    owner,
    repo,
    section,
    rest,
  }: RepositorySectionContext & { readonly section: string; },
): GitHubFetchPlan {
  /**
   Lowercase section name used for matching.
   */
  const lowered = section.toLowerCase();
  /**
   Section context shared by every section planner.
   */
  const context: RepositorySectionContext = {
    owner,
    repo,
    rest,
  };
  if ((lowered === BLOB_SECTION) || (lowered === RAW_SECTION)
    || (lowered === BLAME_SECTION))
    return planFileContentSection(context,);
  if (lowered === TREE_SECTION)
    return planDirectorySection(context,);
  if (lowered === ISSUES_SECTION)
    return planIssueSection(context,);
  if (lowered === PULL_SECTION)
    return planPullRequestSection(context,);
  if (lowered === COMMIT_SECTION)
    return planCommitSection(context,);
  if (lowered === COMPARE_SECTION)
    return planCompareSection(context,);
  if (lowered === RELEASES_SECTION)
    return planReleasesSection(context,);
  return unplanned(`GitHub section ${URL_SLASH}${section}${URL_SLASH} has no gh equivalent`,);
}

//endregion Repository dispatch

export {
  planForHost,
};
