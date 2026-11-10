/**
 GitHub repository and API client contracts.
 
 @module
 */

import type { GitHubApiRequest, } from './github-api.ts';
import type { IncludedResponse, } from './github-response.ts';
import type { SourceLink, } from './issue-model.ts';

/**
 Canonical GitHub repository identity.
 */
export type GitHubRepository = {
  readonly owner: string;
  readonly name: string;
  readonly url: string;
};

/**
 Builds the REST endpoint prefix for a repository.

 @param repository - Canonical repository identity.

 @returns `repos/<owner>/<name>` prefix every repository REST path hangs off.

 @example
 ```ts
 repositoryEndpoint({ owner: 'aquaticat', name: 'Monochromatic', url: 'https://github.com/aquaticat/Monochromatic', }); // 'repos/aquaticat/Monochromatic'
 ```
 */
export function repositoryEndpoint(repository: GitHubRepository,): string {
  return `repos/${repository.owner}/${repository.name}`;
}

/**
 Injectable authenticated GitHub REST boundary.
 */
export type GitHubApiClient = (
  request: GitHubApiRequest,
) => Promise<IncludedResponse>;

/**
 Destination facts needed by deterministic publication planning.
 */
export type PublicationPreflight = {
  readonly needsTriageLabel: boolean;
  readonly sourceLink?: SourceLink;
};
