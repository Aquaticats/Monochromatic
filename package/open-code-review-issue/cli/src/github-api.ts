/**
 GitHub REST transport through bounded `gh api --include` subprocesses.
 
 @module
 */

import {
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  GitHubProcessError,
  runBoundedProcess,
  type BoundedProcessRunner,
} from './github-process.ts';
import {
  IncludedResponseError,
  parseIncludedResponse,
  type IncludedResponse,
} from './github-response.ts';

/**
 HTTP methods used by adapter GitHub boundary.
 */
export type GitHubApiMethod = 'GET' | 'POST' | 'PATCH' | 'DELETE';

/**
 One non-paginated GitHub REST request.
 */
export type GitHubApiRequest = {
  readonly method: GitHubApiMethod;
  readonly endpoint: string;
  readonly body?: Readonly<Record<string, unknown>>;
};

/**
 Parses included output from successful or HTTP-error process result.
 
 @param runProcess - Bounded process implementation.
 
 @param arguments - Exact GitHub CLI argument vector.
 
 @param cwd - Explicit child working directory.
 
 @returns Parsed included response including non-success HTTP status.
 
 @throws {@link GitHubProcessError} when process output is not HTTP response.
 
 @throws {@link IncludedResponseError} when successful output is malformed.
 
 @example
 ```ts
 await executeApi({ runProcess, arguments: ['api'], cwd: process.cwd() });
 ```
 */
async function executeApi({
  runProcess,
  arguments: commandArguments,
  cwd,
}: {
  readonly runProcess: BoundedProcessRunner;
  readonly arguments: readonly string[];
  readonly cwd: string;
},): Promise<IncludedResponse> {
  try {
    /**
     Successful GitHub CLI process result.
     */
    const result = await runProcess({
      file: 'gh',
      arguments: commandArguments,
      cwd,
    },);
    return parseIncludedResponse({ stdout: result.stdout, },);
  }
  catch (error: unknown) {
    if ((!(error instanceof GitHubProcessError)) || (error.stdout === '')) {
      throw error;
    }
    try {
      return parseIncludedResponse({ stdout: error.stdout, },);
    }
    catch (parseError: unknown) {
      throw new GitHubProcessError({
        message: `${error.message}; included output could not be parsed: ${String(parseError,)}`,
        stdout: error.stdout,
        stderr: error.stderr,
        ...(error.exitCode === undefined ? {} : { exitCode: error.exitCode, }),
      },);
    }
  }
}

/**
 Builds base non-paginated GitHub CLI argument vector.
 
 @param request - REST method and endpoint.
 
 @returns Exact arguments before optional input path.
 
 @example
 ```ts
 apiArguments({ method: 'GET', endpoint: 'repos/owner/repo' });
 ```
 */
function apiArguments(request: GitHubApiRequest,): readonly string[] {
  return [
    'api',
    '--include',
    '--method',
    request.method,
    request.endpoint,
  ];
}

/**
 Executes body request through private named JSON file.
 
 @param request - REST request carrying body.
 
 @param cwd - Explicit child working directory.
 
 @param runProcess - Bounded process implementation.
 
 @returns Parsed included response.
 
 @example
 ```ts
 await executeBodyRequest({ request, cwd: process.cwd(), runProcess });
 ```
 */
async function executeBodyRequest({
  request,
  cwd,
  runProcess,
}: {
  readonly request: GitHubApiRequest & {
    readonly body: Readonly<Record<string, unknown>>;
  };
  readonly cwd: string;
  readonly runProcess: BoundedProcessRunner;
},): Promise<IncludedResponse> {
  /**
   Private disposable directory owning request file cleanup.
   */
  await using directory = await mkdtempDisposable(join(
    tmpdir(),
    'ocr-issue-gh-',
  ),);
  /**
   Private named JSON input path.
   */
  const inputPath = join(
    directory.path,
    'request.json',
  );
  await writeFile(
    inputPath,
    JSON.stringify(request.body,),
    {
      encoding: 'utf8',
      mode: 0o600,
      flag: 'wx',
    },
  );
  // Await before async disposal so request file remains present through child completion.
  return await executeApi({
    runProcess,
    arguments: [
      ...apiArguments(request,),
      '--input',
      inputPath,
    ],
    cwd,
  },);
}

/**
 Executes one GitHub REST request through fixed process boundary.
 
 @param request - Method, endpoint, and optional JSON body.
 
 @param cwd - Explicit child working directory.
 
 @param runProcess - Injectable process boundary used by artifact tests.
 
 @returns Parsed HTTP status, headers, and JSON body.
 
 @example
 ```ts
 await runGitHubApi({ request: { method: 'GET', endpoint: 'user' }, cwd: process.cwd() });
 ```
 */
export function runGitHubApi({
  request,
  cwd,
  runProcess = runBoundedProcess,
}: {
  readonly request: GitHubApiRequest;
  readonly cwd: string;
  readonly runProcess?: BoundedProcessRunner;
},): Promise<IncludedResponse> {
  if (request.body !== undefined) {
    return executeBodyRequest({
      request: {
        ...request,
        body: request.body,
      },
      cwd,
      runProcess,
    },);
  }
  return executeApi({
    runProcess,
    arguments: apiArguments(request,),
    cwd,
  },);
}
