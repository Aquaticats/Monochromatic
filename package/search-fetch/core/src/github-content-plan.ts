/**
 GitHub repository content plans for file, directory, and raw-host URLs.
 
 @module
 */

import {
  EMPTY_SEGMENT,
  URL_SLASH,
} from './github-url-plan-constants.ts';
import type {
  GitHubFetchPlan,
  GitHubRefPathSplit,
  RepositorySectionContext,
  RepositoryTarget,
} from './github-fetch-types.ts';
import {
  contentAttempts,
  planned,
  unplanned,
} from './gh-invocation-plan.ts';
import { validateTokenArgument, } from './github-url-validation.ts';

//region Content dispatch

/**
 Plan one raw.githubusercontent.com URL as repository file content.
 
 @param segments - validated path segments
 
 @returns gh plan for the raw file
 
 @example
 ```ts
 planRawContentHost(['cli', 'cli', 'trunk', 'README.md']);
 ```
 */
function planRawContentHost(segments: readonly string[],): GitHubFetchPlan {
  /**
   Repository target resolved from the leading two segments.
   */
  const target = repositoryTarget(segments,);
  if (!target.found)
    return unplanned(target.reason,);

  /**
   Reference and path split candidates, each keeping at least one path segment.
   */
  const splits = refPathSplits({
    segments: target.rest,
    minimumPathSegments: 1,
  },);
  if (splits.length === 0)
    return unplanned('Raw GitHub URL names no file path',);

  return planned({
    kind: 'file-content',
    attempts: contentAttempts({
      owner: target.owner,
      repo: target.repo,
      splits,
      listing: false,
    },),
  },);
}

/**
 Plan one blob, raw, or blame URL as repository file bytes.
 
 @param context - repository section context
 
 @returns gh plan for the file, or a reason when the URL names no file path
 
 @example
 ```ts
 planFileContentSection({ owner: 'cli', repo: 'cli', rest: ['trunk', 'README.md'] });
 ```
 */
function planFileContentSection(context: RepositorySectionContext,): GitHubFetchPlan {
  /**
   Reference and path split candidates, each keeping at least one path segment.
   */
  const splits = refPathSplits({
    segments: context.rest,
    minimumPathSegments: 1,
  },);
  if (splits.length === 0)
    return unplanned('GitHub file URL names no file path',);

  return planned({
    kind: 'file-content',
    attempts: contentAttempts({
      ...context,
      splits,
      listing: false,
    },),
  },);
}

/**
 Plan one tree URL as a repository directory listing.
 
 @param context - repository section context
 
 @returns gh plan for the directory, or a reason when the URL names no reference
 
 @example
 ```ts
 planDirectorySection({ owner: 'cli', repo: 'cli', rest: ['trunk', 'docs'] });
 ```
 */
function planDirectorySection(context: RepositorySectionContext,): GitHubFetchPlan {
  /**
   Reference and path split candidates, whose last entry may list the repository root.
   */
  const splits = refPathSplits({
    segments: context.rest,
    minimumPathSegments: 0,
  },);
  if (splits.length === 0)
    return unplanned('GitHub tree URL names no reference',);

  return planned({
    kind: 'directory-listing',
    attempts: contentAttempts({
      ...context,
      splits,
      listing: true,
    },),
  },);
}

//endregion Content dispatch

//region Content helpers

/**
 Resolve repository owner and name from leading path segments.
 
 @param segments - validated path segments
 
 @returns resolved repository target, or a safe rejection reason
 
 @example
 ```ts
 repositoryTarget(['cli', 'cli', 'blob', 'trunk', 'README.md']);
 ```
 */
function repositoryTarget(segments: readonly string[],): RepositoryTarget {
  /**
   Repository owner and name segments.
   */
  const [
    owner,
    repo,
  ] = segments;
  if ((owner === undefined) || (repo === undefined))
    return {
      found: false,
      reason: 'GitHub URL names no repository',
    };

  /**
   Owner argument validation result.
   */
  const ownerValidation = validateTokenArgument({
    value: owner,
    label: 'repository owner',
  },);
  if (!ownerValidation.safe)
    return {
      found: false,
      reason: ownerValidation.reason,
    };

  /**
   Repository argument validation result.
   */
  const repoValidation = validateTokenArgument({
    value: repo,
    label: 'repository name',
  },);
  if (!repoValidation.safe)
    return {
      found: false,
      reason: repoValidation.reason,
    };

  return {
    found: true,
    owner,
    repo,
    rest: segments.slice(2,),
  };
}

/**
 Build ordered reference and path split candidates for one repository path.
 
 @param segments - repository path segments following the section name
 
 @param minimumPathSegments - path segments that must survive the split
 
 @returns split candidates ordered from shortest to longest reference
 
 @example
 ```ts
 refPathSplits({ segments: ['8761', 'allow-items', 'go.mod'], minimumPathSegments: 1 });
 ```
 */
function refPathSplits(
  {
    segments,
    minimumPathSegments,
  }: {
    readonly segments: readonly string[];
    readonly minimumPathSegments: number;
  },
): readonly GitHubRefPathSplit[] {
  /**
   Longest reference that still leaves the required path segments.
   */
  const maximumRefSegments = Math.max(
    0,
    segments.length - minimumPathSegments,
  );
  return segments
    .slice(
    0,
    maximumRefSegments,
    )
    .map(function mapRefSegment(
      _ignoredSegment: string,
      index: number,
    ): GitHubRefPathSplit {
      /**
       Reference segment count for this candidate.
       */
      const refSegments = index + 1;
      return {
        ref: segments
          .slice(
          0,
          refSegments,
          )
          .join(URL_SLASH,),
        path: segments
          .slice(refSegments,)
          .join(URL_SLASH,),
      };
    },);
}

//endregion Content helpers

export {
  planDirectorySection,
  planFileContentSection,
  planRawContentHost,
  repositoryTarget,
};
