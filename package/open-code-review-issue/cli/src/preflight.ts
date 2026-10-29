/**
 Destination label and source-link preflight.
 
 @module
 */

import type {
  GitHubApiClient,
  GitHubRepository,
  PublicationPreflight,
} from './github-model.ts';
import { isRecord, } from './json-record.ts';

/**
 Successful GitHub read status.
 */
const HTTP_OK = 200;

/**
 Confirmed missing resource status.
 */
const HTTP_NOT_FOUND = 404;

/**
 Reports terminal destination preflight failure.
 */
export class PublicationPreflightError extends Error {
  /**
   Creates preflight failure.
   
   @param message - Safe destination operation diagnostic.
   
   @example
   ```ts
   const error = new PublicationPreflightError('label lookup failed');
   ```
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'PublicationPreflightError';
  }
}

/**
 Builds shared repository REST endpoint prefix.
 
 @param repository - Canonical destination identity.
 
 @returns Owner/name endpoint prefix.
 
 @example
 ```ts
 repositoryEndpoint(repository); // 'repos/owner/name'
 ```
 */
function repositoryEndpoint(repository: GitHubRepository,): string {
  return `repos/${repository.owner}/${repository.name}`;
}

/**
 Looks up existing needs-triage label without creating one.
 
 @param repository - Canonical destination identity.
 
 @param api - Authenticated GitHub API client.
 
 @returns True for existing label and false only for confirmed 404.
 
 @throws {@link PublicationPreflightError} for every other status.
 
 @example
 ```ts
 await lookupNeedsTriageLabel({ repository, api });
 ```
 */
async function lookupNeedsTriageLabel({
  repository,
  api,
}: {
  readonly repository: GitHubRepository;
  readonly api: GitHubApiClient;
},): Promise<boolean> {
  /**
   Existing-label lookup response.
   */
  const response = await api({
    method: 'GET',
    endpoint: `${repositoryEndpoint(repository,)}/labels/needs-triage`,
  },);
  if (response.status === HTTP_OK) {
    return true;
  }
  if (response.status === HTTP_NOT_FOUND) {
    return false;
  }
  throw new PublicationPreflightError(
    `needs-triage label lookup failed with HTTP ${String(response.status,)}`,
  );
}

/**
 Verifies input head exists as exact destination commit.
 
 @param repository - Canonical destination identity.
 
 @param resolvedHead - OCR resolved head commit.
 
 @param api - Authenticated GitHub API client.
 
 @returns Optional commit-pinned source-link property.
 
 @throws {@link PublicationPreflightError} for statuses other than 200 or 404.
 
 @example
 ```ts
 await verifySourceLink({ repository, resolvedHead: 'abc', api });
 ```
 */
async function verifySourceLink({
  repository,
  resolvedHead,
  api,
}: {
  readonly repository: GitHubRepository;
  readonly resolvedHead: string;
  readonly api: GitHubApiClient;
},): Promise<Pick<PublicationPreflight, 'sourceLink'>> {
  /**
   Destination commit lookup response.
   */
  const response = await api({
    method: 'GET',
    endpoint: `${repositoryEndpoint(repository,)}/commits/${encodeURIComponent(resolvedHead,)}`,
  },);
  if (response.status === HTTP_NOT_FOUND) {
    return {};
  }
  if (response.status !== HTTP_OK) {
    throw new PublicationPreflightError(
      `source commit lookup failed with HTTP ${String(response.status,)}`,
    );
  }
  if ((!isRecord(response.body,))
    || (response.body
      .sha
      !== resolvedHead))
    {
    return {};
  }
  return {
    sourceLink: {
      repository: `${repository.owner}/${repository.name}`,
      commit: resolvedHead,
    },
  };
}

/**
 Resolves destination facts before rendering or mutation.
 
 @param repository - Canonical destination identity.
 
 @param resolvedHead - Optional OCR resolved head commit.
 
 @param api - Authenticated GitHub API client.
 
 @returns Existing-label and verified-source behavior.
 
 @example
 ```ts
 await preflightPublication({ repository, api });
 ```
 */
export async function preflightPublication({
  repository,
  resolvedHead,
  api,
}: {
  readonly repository: GitHubRepository;
  readonly resolvedHead?: string;
  readonly api: GitHubApiClient;
},): Promise<PublicationPreflight> {
  /**
   Confirmed existing-label state.
   */
  const needsTriageLabel = await lookupNeedsTriageLabel({
    repository,
    api,
  });
  if (resolvedHead === undefined) {
    return { needsTriageLabel, };
  }
  return {
    needsTriageLabel,
    ...await verifySourceLink({
      repository,
      resolvedHead,
      api,
    }),
  };
}
