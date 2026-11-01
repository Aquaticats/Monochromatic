/**
 Production GitHub CLI client composition.
 
 @module
 */

import {
  runGitHubApi,
  type GitHubApiRequest,
} from './github-api.ts';
import type { GitHubApiClient, } from './github-model.ts';
import {
  runBoundedProcess,
  type BoundedProcessRunner,
} from './github-process.ts';
import type { IncludedResponse, } from './github-response.ts';
import {
  parseGitHubCliVersion,
  type GitHubCliVersion,
} from './github-version.ts';

/**
 Runs and validates installed GitHub CLI version before API use.
 
 @param cwd - Explicit child working directory.
 
 @param runProcess - Injectable bounded process implementation.
 
 @returns Parsed supported GitHub CLI version.
 
 @example
 ```ts
 await checkGitHubCliVersion({ cwd: process.cwd() });
 ```
 */
export async function checkGitHubCliVersion({
  cwd,
  runProcess = runBoundedProcess,
}: {
  readonly cwd: string;
  readonly runProcess?: BoundedProcessRunner;
},): Promise<GitHubCliVersion> {
  /**
   Captured `gh --version` process result.
   */
  const result = await runProcess({
    file: 'gh',
    arguments: ['--version',],
    cwd,
  },);
  return parseGitHubCliVersion({ stdout: result.stdout, },);
}

/**
 Creates authenticated GitHub API function bound to cwd and process runner.
 
 @param cwd - Explicit child working directory.
 
 @param runProcess - Injectable bounded process implementation.
 
 @returns API function forwarding every request through private process boundary.
 
 @example
 ```ts
 const api = createGitHubApiClient({ cwd: process.cwd() });
 ```
 */
export function createGitHubApiClient({
  cwd,
  runProcess = runBoundedProcess,
}: {
  readonly cwd: string;
  readonly runProcess?: BoundedProcessRunner;
},): GitHubApiClient {
  /**
   Bound API operation.
   
   @param request - One non-paginated GitHub REST request.
   
   @returns Parsed status, headers, and body.
   */
  function api(request: GitHubApiRequest,): Promise<IncludedResponse> {
    return runGitHubApi({
      request,
      cwd,
      runProcess,
    },);
  }
  return api;
}
