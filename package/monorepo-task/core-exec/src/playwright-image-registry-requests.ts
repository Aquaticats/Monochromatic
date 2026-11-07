/**
 The two requests the Playwright image workflow makes to `ghcr.io`, and how it reads the answers.

 The registry's own token flow is the one `podman login` and `podman pull` use: a request for a bearer token for the
 repository (with the workflow's token as the password of a basic-auth request, or with no credentials at all for the
 anonymous check), then a request for the manifest of the tag with that bearer token. Credentials go to `curl` through
 its standard input (`--config -`), never on a command line. `curl` runs as it did in the shell steps, so its retries,
 time limit, output and statuses are unchanged. Imports only `node:` modules and sibling files that do, because the
 workflow runs these steps before any dependency is installed.

 @module
 */

import { randomBytes, } from 'node:crypto';
import {
  open,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  jsonAt,
  rawJsonTextOrEmpty,
  requiredEnvironment,
  runProgram,
  withoutTrailingLineBreaks,
} from './github-actions-step-support.ts';

//region Names and constants

/**
 Media types a manifest request accepts, the ones `podman pull` accepts.
 */
const MANIFEST_MEDIA_TYPES = [
  'application/vnd.oci.image.manifest.v1+json',
  'application/vnd.oci.image.index.v1+json',
  'application/vnd.docker.distribution.manifest.v2+json',
  'application/vnd.docker.distribution.manifest.list.v2+json',
].join(', ',);

/**
 Host prefix removed from the image name to get the repository path in the registry.
 */
const REGISTRY_PREFIX = 'ghcr.io/';

/**
 Longest a request may take, in seconds, retries not counted.
 */
const REQUEST_MAX_SECONDS = '30';

/**
 How many times `curl` retries a transient failure.
 */
const REQUEST_RETRIES = '3';

/**
 Seconds `curl` waits between retries.
 */
const REQUEST_RETRY_DELAY_SECONDS = '2';

/**
 The `curl` options both requests share, with or without `--show-error` (the check that must never fail the run keeps
 `curl` quiet).

 @param showError - Whether `curl` prints its own message when a request fails

 @returns Options in the order the shell steps gave them

 @example
 ```ts
 const flags = curlFlags({ showError: true, },);
 ```
 */
export function curlFlags(
  {
    showError,
  }: {
    readonly showError: boolean;
  },
): readonly string[] {
  return [
    '--silent',
    ...showError ? ['--show-error',] : [],
    '--max-time',
    REQUEST_MAX_SECONDS,
    '--retry',
    REQUEST_RETRIES,
    '--retry-delay',
    REQUEST_RETRY_DELAY_SECONDS,
  ];
}

/**
 The repository path of an image in the registry.

 @param image - Image name with its host, such as `ghcr.io/owner/name`

 @returns The name without the `ghcr.io/` prefix when it has one

 @example
 ```ts
 const repository = repositoryOf('ghcr.io/aquaticats/monochromatic-playwright',);
 ```
 */
export function repositoryOf(image: string,): string {
  return image.startsWith(REGISTRY_PREFIX,) ? image.slice(REGISTRY_PREFIX.length,) : image;
}

/**
 Address of the token request for pulling a repository.

 @param repository - Repository path in the registry

 @returns The address

 @example
 ```ts
 const address = tokenAddress('aquaticats/monochromatic-playwright',);
 ```
 */
export function tokenAddress(repository: string,): string {
  return `https://ghcr.io/token?service=ghcr.io&scope=repository:${repository}:pull`;
}

/**
 Address of the manifest of a tag.

 @param repository - Repository path in the registry

 @param tag - Tag to ask for

 @returns The address

 @example
 ```ts
 const address = manifestAddress({ repository: 'aquaticats/monochromatic-playwright', tag: '0123456789abcdef', },);
 ```
 */
export function manifestAddress(
  {
    repository,
    tag,
  }: {
    readonly repository: string;
    readonly tag: string;
  },
): string {
  return `https://ghcr.io/v2/${repository}/manifests/${tag}`;
}

/**
 Random bytes in the name of the response body file, which become twice as many hex digits.
 */
const BODY_NAME_RANDOM_BYTES = 3;

/**
 Permission bits of the response body file: the owner reads and writes it, nobody else.
 */
const OWNER_ONLY_MODE = 0o600;

//endregion Names and constants

//region Making the requests

/**
 What a request left: how `curl` ended and the HTTP status it printed (`000` when nothing answered).
 */
export type RequestResult = {
  /**
   Exit status of `curl`; 0 when the registry answered with any status.
   */
  readonly curlStatus: number;
  /**
   The HTTP status `curl` printed through `--write-out`, without a trailing line break.
   */
  readonly httpStatus: string;
};

/**
 Makes one request with `curl` and reads the HTTP status it prints.

 @param flags - The options both requests share, from {@link curlFlags}

 @param input - Text for `curl --config -` on standard input (credentials or headers); no `--config -` when it is left out

 @param head - Whether to ask only for the headers

 @param output - File that receives the body

 @param address - What to request

 @returns How `curl` ended and the status it printed

 @example
 ```ts
 const { httpStatus, } = await request({ flags, output: '/dev/null', address, head: true, },);
 ```
 */
export async function request(
  {
    flags,
    input,
    head,
    output,
    address,
  }: {
    readonly flags: readonly string[];
    readonly input?: string;
    readonly head: boolean;
    readonly output: string;
    readonly address: string;
  },
): Promise<RequestResult> {
  /**
   The finished request, its standard output captured because that is where `--write-out` prints.
   */
  const result = await runProgram({
    file: 'curl',
    args: [
      ...flags,
      ...input === undefined ? [] : [
        '--config',
        '-',
      ],
      ...head ? ['--head',] : [],
      '--output',
      output,
      '--write-out',
      '%{http_code}',
      address,
    ],
    ...input === undefined ? {} : { input, },
    captureStdout: true,
    cwd: process.cwd(),
  },);
  return {
    curlStatus: result.status,
    httpStatus: withoutTrailingLineBreaks(result.stdout,),
  };
}

/**
 Credentials of the token request: the workflow's token as the password of a basic-auth request, in the form
 `curl --config -` reads.

 @param actor - User name sent with the token

 @param token - The workflow's token

 @returns Text for `curl`'s standard input

 @example
 ```ts
 const input = basicAuthConfig({ actor: 'octocat', token, },);
 ```
 */
export function basicAuthConfig(
  {
    actor,
    token,
  }: {
    readonly actor: string;
    readonly token: string;
  },
): string {
  return `user = "${actor}:${token}"\n`;
}

/**
 Headers of the manifest request: the bearer token and the accepted media types, in the form `curl --config -` reads.

 @param bearer - Bearer token the registry gave

 @returns Text for `curl`'s standard input

 @example
 ```ts
 const input = manifestHeadersConfig(bearer,);
 ```
 */
export function manifestHeadersConfig(bearer: string,): string {
  return `header = "Authorization: Bearer ${bearer}"\nheader = "Accept: ${MANIFEST_MEDIA_TYPES}"\n`;
}

/**
 Makes the file the token response is written into, owner-only, in the runner's temporary folder.

 @returns Path of the new, empty file

 @example
 ```ts
 const body = await makeBodyFile();
 ```
 */
export async function makeBodyFile(): Promise<string> {
  /**
   Where the response body goes; a random suffix keeps two steps from sharing a file.
   */
  const path = join(
    requiredEnvironment('RUNNER_TEMP',),
    `tool-${randomBytes(BODY_NAME_RANDOM_BYTES,)
      .toString('hex',)}`,
  );
  /**
   The new file, created exclusively.
   */
  const handle = await open(
    path,
    'wx',
    OWNER_ONLY_MODE,
  );
  await handle.close();
  return path;
}

//endregion Making the requests

//region Reading the answers

/**
 Reads a response body as JSON; text that is not JSON counts as no value.

 @param path - File the response was written into

 @returns The parsed value; `undefined` when the file is unreadable, empty or not JSON

 @example
 ```ts
 const body = await readResponse(path,);
 ```
 */
async function readResponse(path: string,): Promise<unknown> {
  try {
    return JSON.parse(await readFile(
      path,
      'utf8',
    ),) as unknown;
  }
  catch (error) {
    // A body that is missing, empty or not JSON has no token and no error code, which is how both callers read it.
    void error;
    return undefined;
  }
}

/**
 The bearer token of a token response: `jq --raw-output '.token // empty'`.

 @param path - File the response was written into

 @returns The token, empty when the response has none

 @example
 ```ts
 const token = await tokenOf(path,);
 ```
 */
export async function tokenOf(path: string,): Promise<string> {
  return rawJsonTextOrEmpty(jsonAt({
    value: await readResponse(path,),
    keys: ['token',],
  },),);
}

/**
 The registry's error code in a refusal: `jq --raw-output '.errors[0].code // empty'`.

 @param path - File the response was written into

 @returns The first error's code, empty when the response has none

 @example
 ```ts
 const code = await errorCodeOf(path,);
 ```
 */
export async function errorCodeOf(path: string,): Promise<string> {
  /**
   The `errors` entry of the response.
   */
  const errors = jsonAt({
    value: await readResponse(path,),
    keys: ['errors',],
  },);
  if (!Array.isArray(errors,))
    return '';
  return rawJsonTextOrEmpty(jsonAt({
    value: errors[0],
    keys: ['code',],
  },),);
}

//endregion Reading the answers
