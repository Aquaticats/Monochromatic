/**
 GitHub record plans for issue, pull request, commit, comparison, and release URLs.
 
 @module
 */

import {
  EMPTY_SEGMENT,
  GH_COLOR_FLAG,
  GH_COLOR_NEVER,
  GH_REPO_FLAG,
  GITHUB_URL_PREFIX,
  ISSUES_SECTION,
  PULL_FILES_SECTION,
  PULL_SECTION,
  RELEASE_TAG_SECTION,
  URL_SLASH,
} from './github-url-plan-constants.ts';
import type {
  GitHubFetchPlan,
  RepositorySectionContext,
} from './github-fetch-types.ts';
import {
  diffAttemptPlan,
  singleAttemptPlan,
  threadAttemptPlan,
  unplanned,
} from './gh-invocation-plan.ts';
import {
  decodeUrlText,
  validateEndpointFragment,
  validatePositionalPathArgument,
  validateReferenceNumber,
} from './github-url-validation.ts';

//region Record planners

/**
 Plan one numbered issue URL as issue body plus comments.
 
 @param context - repository section context
 
 @returns gh plan for the issue thread
 
 @example
 ```ts
 planIssueSection({ owner: 'cli', repo: 'cli', rest: ['13118'] });
 ```
 */
function planIssueSection(context: RepositorySectionContext,): GitHubFetchPlan {
  /**
   Issue number segment.
   */
  const [
    reference,
  ] = context.rest;
  if (reference === undefined)
    return unplanned('GitHub issues URL names no issue number',);

  /**
   Issue number validation result.
   */
  const validation = validateReferenceNumber({
    value: reference,
    label: 'issue',
  },);
  if (!validation.safe)
    return unplanned(validation.reason,);

  return threadAttemptPlan({
    kind: 'issue-thread',
    label: `issue ${context.owner}/${context.repo}#${reference} body and comments`,
    viewArgs: [
      'issue',
      'view',
      `${GITHUB_URL_PREFIX}${context.owner}${URL_SLASH}${context.repo}${URL_SLASH}${ISSUES_SECTION}${URL_SLASH}${reference}`,
    ],
  },);
}

/**
 Plan one numbered pull request URL as body plus comments, or as a diff for changed-file URLs.
 
 @param context - repository section context
 
 @returns gh plan for the pull request thread or diff
 
 @example
 ```ts
 planPullRequestSection({ owner: 'cli', repo: 'cli', rest: ['14517', 'files'] });
 ```
 */
function planPullRequestSection(context: RepositorySectionContext,): GitHubFetchPlan {
  /**
   Pull request number and optional subsection segments.
   */
  const [
    reference,
    subsection,
  ] = context.rest;
  if (reference === undefined)
    return unplanned('GitHub pull URL names no pull request number',);

  /**
   Pull request number validation result.
   */
  const validation = validateReferenceNumber({
    value: reference,
    label: 'pull request',
  },);
  if (!validation.safe)
    return unplanned(validation.reason,);

  /**
   Canonical pull request URL accepted by `gh pr view` and `gh pr diff`.
   */
  const pullRequestUrl = `${GITHUB_URL_PREFIX}${context.owner}${URL_SLASH}${context.repo}${URL_SLASH}${PULL_SECTION}${URL_SLASH}${reference}`;
  if (subsection === undefined)
    return threadAttemptPlan({
      kind: 'pull-request-thread',
      label: `pull request ${context.owner}/${context.repo}#${reference} body and comments`,
      viewArgs: [
        'pr',
        'view',
        pullRequestUrl,
      ],
    },);
  if (subsection.toLowerCase() === PULL_FILES_SECTION)
    return singleAttemptPlan({
      kind: 'pull-request-diff',
      label: `pull request ${context.owner}/${context.repo}#${reference} changed files`,
      args: [
        'pr',
        'diff',
        pullRequestUrl,
        GH_COLOR_FLAG,
        GH_COLOR_NEVER,
      ],
    },);
  return unplanned(`GitHub pull request subsection ${URL_SLASH}${subsection}${URL_SLASH} has no gh equivalent`,);
}

/**
 Plan one commit URL as a unified diff.
 
 @param context - repository section context
 
 @returns gh plan for the commit diff
 
 @example
 ```ts
 planCommitSection({ owner: 'cli', repo: 'cli', rest: ['9b031151a825bda919203c5202876a725d637368'] });
 ```
 */
function planCommitSection(context: RepositorySectionContext,): GitHubFetchPlan {
  /**
   Commit reference joined from remaining segments.
   */
  const reference = context.rest
    .join(URL_SLASH,);
  /**
   Commit reference validation result.
   */
  const validation = validateEndpointFragment({
    value: reference,
    label: 'commit reference',
  },);
  if (!validation.safe)
    return unplanned(validation.reason,);

  return diffAttemptPlan({
    kind: 'commit-diff',
    label: `commit ${context.owner}/${context.repo}@${reference} diff`,
    endpoint: `${URL_SLASH}repos/${context.owner}/${context.repo}/commits/${reference}`,
  },);
}

/**
 Plan one comparison URL as a unified diff.
 
 @param context - repository section context
 
 @returns gh plan for the comparison diff
 
 @example
 ```ts
 planCompareSection({ owner: 'cli', repo: 'cli', rest: ['v2.100.0...v2.101.0'] });
 ```
 */
function planCompareSection(context: RepositorySectionContext,): GitHubFetchPlan {
  /**
   Comparison specification joined from remaining segments.
   */
  const comparison = context.rest
    .join(URL_SLASH,);
  /**
   Comparison specification validation result.
   */
  const validation = validateEndpointFragment({
    value: comparison,
    label: 'comparison specification',
  },);
  if (!validation.safe)
    return unplanned(validation.reason,);

  return diffAttemptPlan({
    kind: 'comparison-diff',
    label: `comparison ${context.owner}/${context.repo} ${comparison} diff`,
    endpoint: `${URL_SLASH}repos/${context.owner}/${context.repo}/compare/${comparison}`,
  },);
}

/**
 Plan one releases URL as release notes for a tag, or as the release index.
 
 @param context - repository section context
 
 @returns gh plan for the release tag or release index
 
 @example
 ```ts
 planReleasesSection({ owner: 'cli', repo: 'cli', rest: ['tag', 'v2.101.0'] });
 ```
 */
function planReleasesSection(context: RepositorySectionContext,): GitHubFetchPlan {
  /**
   Repository argument shared by both release forms.
   */
  const repositoryArgument = `${context.owner}${URL_SLASH}${context.repo}`;
  if (context.rest
    .length
    === 0)
    return singleAttemptPlan({
      kind: 'release-index',
      label: `release index for ${repositoryArgument}`,
      args: [
        'release',
        'list',
        GH_REPO_FLAG,
        repositoryArgument,
      ],
    },);

  /**
   Release subsection and tag segments.
   */
  const [
    subsection = EMPTY_SEGMENT,
    ...tagSegments
  ] = context.rest;
  if (subsection.toLowerCase() !== RELEASE_TAG_SECTION)
    return unplanned(`GitHub releases subsection ${URL_SLASH}${subsection}${URL_SLASH} has no gh equivalent`,);

  /**
   Release tag joined from remaining segments and decoded for use as one argv value.
   */
  const tag = decodeUrlText(tagSegments.join(URL_SLASH,),);
  /**
   Release tag validation result, checked after decoding so a decoded dash cannot become a flag.
   */
  const validation = validatePositionalPathArgument({
    value: tag,
    label: 'release tag',
  },);
  if (!validation.safe)
    return unplanned(validation.reason,);

  return singleAttemptPlan({
    kind: 'release-notes',
    label: `release ${tag} notes for ${repositoryArgument}`,
    args: [
      'release',
      'view',
      tag,
      GH_REPO_FLAG,
      repositoryArgument,
    ],
  },);
}

//endregion Record planners

export {
  planCommitSection,
  planCompareSection,
  planIssueSection,
  planPullRequestSection,
  planReleasesSection,
};
