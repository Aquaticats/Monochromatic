#!/usr/bin/env node

/**
 The "Check that anyone can pull it" step of `.github/workflows/playwright-image.yml`: asks `ghcr.io` for the manifest of
 `<IMAGE>:<TAG>` the way a person without a login (or `prepare:playwright`) would, records the answer in the job summary,
 and warns when the package is private or the tag is missing.

 Advisory only: a fault here is reported as the status below (`000` when nothing answered) and never fails a run that has
 already published the image. No credentials of ours are sent. Reads `IMAGE`, `TAG` and `RUNNER_TEMP` from the
 environment and appends to `GITHUB_STEP_SUMMARY`. Imports only `node:` modules and sibling files that do, because the
 workflow runs it before any dependency is installed.

 @example
 ```bash
 node package/monorepo-task/core-exec/src/playwright-image-anonymous-pull-check.ts
 ```
 */

import {
  curlFlags,
  errorCodeOf,
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
} from './github-actions-step-support.ts';

/**
 The answer of a request that succeeded.
 */
const HTTP_OK = '200';

/**
 Asks for the manifest with a bearer token, reporting what the registry answered.

 @param repository - Repository path in the registry

 @param tag - Tag to ask for

 @param bearer - The anonymous bearer token

 @param flags - Options for `curl`

 @returns The HTTP status, as `curl` printed it

 @example
 ```ts
 const answer = await askManifest({ repository, tag, bearer, flags, },);
 ```
 */
async function askManifest(
  {
    repository,
    tag,
    bearer,
    flags,
  }: {
    readonly repository: string;
    readonly tag: string;
    readonly bearer: string;
    readonly flags: readonly string[];
  },
): Promise<string> {
  say(`::add-mask::${bearer}`,);
  return (await request({
    flags,
    input: manifestHeadersConfig(bearer,),
    head: true,
    output: '/dev/null',
    address: manifestAddress({
      repository,
      tag,
    },),
  },)).httpStatus;
}

/**
 Finds what the anonymous request answers, as a person without a login would see it.

 @param repository - Repository path in the registry

 @param tag - Tag to ask for

 @returns The text the summary and the warning show: the HTTP status, with the registry's error code after it when the
   first request was refused with one

 @example
 ```ts
 const answer = await anonymousAnswer({ repository, tag, },);
 ```
 */
async function anonymousAnswer(
  {
    repository,
    tag,
  }: {
    readonly repository: string;
    readonly tag: string;
  },
): Promise<string> {
  /**
   Options for every request of this step; `curl` stays quiet.
   */
  const flags = curlFlags({ showError: false, },);
  /**
   File the token response is written into.
   */
  const body = await makeBodyFile();
  /**
   How the token request ended; a failure only shows in its status.
   */
  const tokenAnswer = await request({
    flags,
    head: false,
    output: body,
    address: tokenAddress(repository,),
  },);
  if (tokenAnswer.httpStatus === HTTP_OK) {
    /**
     The bearer token of the anonymous request, empty when the response has none.
     */
    const bearer = await tokenOf(body,);
    if (bearer === '')
      return tokenAnswer.httpStatus;
    return await askManifest({
      repository,
      tag,
      bearer,
      flags,
    },);
  }
  /**
   What the first refused request answered: the registry's error code for a private package.
   */
  const code = await errorCodeOf(body,);
  return code === '' ? tokenAnswer.httpStatus : `${tokenAnswer.httpStatus} (${code})`;
}

/**
 Records the anonymous answer in the job summary and warns when anyone cannot pull the image.

 @example
 ```ts
 await checkAnonymousPull();
 ```
 */
async function checkAnonymousPull(): Promise<void> {
  /**
   The image, with its host.
   */
  const image = requiredEnvironment('IMAGE',);
  /**
   The tag to look for.
   */
  const tag = requiredEnvironment('TAG',);
  /**
   What the registry answered anonymously.
   */
  const answer = await anonymousAnswer({
    repository: repositoryOf(image,),
    tag,
  },);
  await appendToRunnerFile({
    variable: 'GITHUB_STEP_SUMMARY',
    text: `## Playwright image\n- image: \`${image}:${tag}\`\n- anonymous pull check: HTTP ${answer}\n`,
  },);
  if (answer === HTTP_OK) {
    say(`anyone can pull ${image}:${tag}`,);
    return;
  }
  say(
    `::warning title=Playwright image is not public::An anonymous request for ${image}:${tag} answered HTTP ${answer}, so the package is private or the tag is missing. The owner makes the package public once: open it from https://github.com/orgs/Aquaticats/packages, then Package settings, Danger Zone, Change visibility, Public. Until then prepare:playwright falls back to building playwright.Dockerfile locally.`,
  );
}

await runStep(checkAnonymousPull,);
