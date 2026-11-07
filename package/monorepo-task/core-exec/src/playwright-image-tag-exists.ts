#!/usr/bin/env node

/**
 The "Check whether the tag exists" step of `.github/workflows/playwright-image.yml`: asks `ghcr.io` whether
 `<IMAGE>:<TAG>` is already published, and writes `present=true` or `present=false` as the `present` output.

 A tag names one content, so an existing tag is never rebuilt or replaced. The step fails the run when it cannot tell (a
 registry error, a timeout) and treats only a missing tag, or a package the registry grants no token for, as absent, so a
 transient fault never leads to an overwrite. It uses the registry's own token flow, the one `podman login` and
 `podman pull` use: the workflow's token is the password of a basic-auth request and the answer is a bearer token for this
 repository. The credentials go through `curl`'s standard input and not its command line, and the bearer token is masked in
 the log before anything else uses it.

 Reads `IMAGE`, `TAG`, `GITHUB_TOKEN`, `GITHUB_ACTOR_NAME` and `RUNNER_TEMP` from the environment and appends to
 `GITHUB_OUTPUT`. Imports only `node:` modules and sibling files that do, because the workflow runs it before any
 dependency is installed.

 @example
 ```bash
 node package/monorepo-task/core-exec/src/playwright-image-tag-exists.ts
 ```
 */

import {
  basicAuthConfig,
  curlFlags,
  makeBodyFile,
  manifestAddress,
  manifestHeadersConfig,
  repositoryOf,
  request,
  tokenAddress,
  tokenOf,
} from './playwright-image-registry-requests.ts';
import {
  appendToRunnerFile,
  requiredEnvironment,
  runStep,
  say,
  StepExitError,
} from './github-actions-step-support.ts';

/**
 What the registry may answer that this step reads as "the tag is not there".
 */
const HTTP_FORBIDDEN = '403';

/**
 The answer for a token request that succeeded.
 */
const HTTP_OK = '200';

/**
 The answer for a manifest that does not exist.
 */
const HTTP_NOT_FOUND = '404';

/**
 Ends the step with the verdict "absent", which lets the build and the push run.

 @param reason - Why the tag counts as absent

 @example
 ```ts
 await reportAbsent('HTTP 404 for the tag',);
 ```
 */
async function reportAbsent(reason: string,): Promise<void> {
  await appendToRunnerFile({
    variable: 'GITHUB_OUTPUT',
    text: 'present=false\n',
  },);
  say(`${requiredEnvironment('IMAGE',)}:${requiredEnvironment('TAG',)} is absent (${reason}); building it`,);
}

/**
 Ends the step with an error: it cannot tell, so nothing is built or pushed.

 @param reason - What the registry answered

 @throws {@link StepExitError} Always, with status 1

 @example
 ```ts
 reportUnknown('The token request answered HTTP 500',);
 ```
 */
function reportUnknown(reason: string,): never {
  say(
    `::error title=Cannot tell whether the tag exists::${reason}. Nothing was built or pushed, so no existing tag was replaced. After a registry fault one rerun of the job is right. On the first run, before the package exists, the registry answer for a missing package was never observed with the workflow token, so a rerun would fail the same way and the status handling of this step has to be edited (doc/runbook/publish-playwright-image.md).`,
  );
  throw new StepExitError(1,);
}

/**
 Asks the registry for a bearer token with the workflow's token.

 @param repository - Repository path in the registry

 @param flags - Options for `curl`

 @returns The bearer token, or nothing when the registry grants this token nothing for the package (the tag then counts as
   absent and the verdict is already reported)

 @throws {@link StepExitError} With `curl`'s status when the request fails, or status 1 when the answer cannot be read

 @example
 ```ts
 const bearer = await requestBearer({ repository, flags, },);
 ```
 */
async function requestBearer(
  {
    repository,
    flags,
  }: {
    readonly repository: string;
    readonly flags: readonly string[];
  },
): Promise<{
  readonly granted: true;
  readonly bearer: string
} | { readonly granted: false; }> {
  /**
   File the token response is written into.
   */
  const body = await makeBodyFile();
  /**
   How the token request ended.
   */
  const answer = await request({
    flags,
    input: basicAuthConfig({
      actor: requiredEnvironment('GITHUB_ACTOR_NAME',),
      token: requiredEnvironment('GITHUB_TOKEN',),
    },),
    head: false,
    output: body,
    address: tokenAddress(repository,),
  },);
  // A transport fault (no connection, a timeout after the retries) ends the step with curl's own status and message, so the
  // build and the push are skipped.
  if (answer.curlStatus !== 0)
    throw new StepExitError(answer.curlStatus,);
  if (answer.httpStatus === HTTP_FORBIDDEN) {
    // A 403 from the token endpoint means the registry grants this token nothing for the package: the package does not
    // exist yet, or it exists and the workflow token has no access to it. Both go on to the build; in the second reading the
    // push is refused, so no existing tag is replaced.
    await reportAbsent(
      'HTTP 403 for the token: the package does not exist yet, or it exists and this token has no access to it; the build runs, and a push this token may not make is refused',
    );
    return { granted: false, };
  }
  if (answer.httpStatus !== HTTP_OK)
    reportUnknown(`The token request answered HTTP ${answer.httpStatus}`,);
  /**
   The token in the response.
   */
  const bearer = await tokenOf(body,);
  if (bearer === '')
    reportUnknown('The token response had no token',);
  return {
    granted: true,
    bearer,
  };
}

/**
 Decides whether the tag exists and reports it.

 @throws {@link StepExitError} With status 1 when it cannot tell, or with `curl`'s status when a request fails

 @example
 ```ts
 await checkTag();
 ```
 */
async function checkTag(): Promise<void> {
  /**
   The image, with its host.
   */
  const image = requiredEnvironment('IMAGE',);
  /**
   The tag to look for.
   */
  const tag = requiredEnvironment('TAG',);
  /**
   Repository path in the registry.
   */
  const repository = repositoryOf(image,);
  /**
   Options for every request of this step.
   */
  const flags = curlFlags({ showError: true, },);
  /**
   Whether the registry gave a bearer token.
   */
  const bearer = await requestBearer({
    repository,
    flags,
  },);
  if (!bearer.granted)
    return;
  // The token is masked in the log before the next request uses it.
  say(`::add-mask::${bearer.bearer}`,);
  /**
   How the manifest request ended.
   */
  const manifest = await request({
    flags,
    input: manifestHeadersConfig(bearer.bearer,),
    head: true,
    output: '/dev/null',
    address: manifestAddress({
      repository,
      tag,
    },),
  },);
  if (manifest.curlStatus !== 0)
    throw new StepExitError(manifest.curlStatus,);
  if (manifest.httpStatus === HTTP_OK) {
    await appendToRunnerFile({
      variable: 'GITHUB_OUTPUT',
      text: 'present=true\n',
    },);
    say(`${image}:${tag} exists (HTTP ${manifest.httpStatus}); nothing to build`,);
    return;
  }
  if (manifest.httpStatus === HTTP_NOT_FOUND) {
    await reportAbsent('HTTP 404 for the tag',);
    return;
  }
  reportUnknown(`The manifest request answered HTTP ${manifest.httpStatus}`,);
}

await runStep(checkTag,);
