/**
 Exercises the deployed Playwright image workflow steps (`playwright-image-tag.ts`, `playwright-image-tag-exists.ts`,
 `playwright-image-build.ts`, `playwright-image-push.ts` and `playwright-image-anonymous-pull-check.ts`) through
 subprocesses, as `.github/workflows/playwright-image.yml` starts them.

 The source files are run, not built copies: the workflow runs them with the runner's Node and no installed dependencies,
 so the checked-in TypeScript is the artifact that ships. `curl` and `podman` are stand-ins that answer from a scenario
 and record what they were asked; the registry is never reached. Every secret here is a fake.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  createStepFixture,
  type CurlRule,
  type StepFixture,
  type StepRun,
} from './github-actions-step-fixture.ts';

//region Fixtures: the registry's answers and the step environment

/**
 Folder of the steps under test.
 */
const SOURCE_FOLDER = import.meta.dirname;

/**
 Fake secret the runner holds as the workflow's token; no output of a step may show it.
 */
const FAKE_SECRET = 'FAKE_TOKEN_123';

/**
 Fake bearer token the stand-in registry hands out; only the mask line may show it.
 */
const FAKE_BEARER = 'FAKE_BEARER_456';

/**
 Image the workflow publishes.
 */
const IMAGE = 'ghcr.io/aquaticats/monochromatic-playwright';

/**
 Tag under test.
 */
const TAG = '0123456789abcdef';

/**
 User name sent with the token.
 */
const ACTOR = 'fake-actor';

/**
 Address fragment of the token request.
 */
const TOKEN_ADDRESS = 'https://ghcr.io/token?service=ghcr.io&scope=repository:aquaticats/monochromatic-playwright:pull';

/**
 Address fragment of the manifest request.
 */
const MANIFEST_ADDRESS = `https://ghcr.io/v2/aquaticats/monochromatic-playwright/manifests/${TAG}`;

/**
 Rules for a registry that gives a bearer token and then answers the manifest request with a status.

 @param manifestStatus - HTTP status of the manifest request

 @returns Rules for the stand-in `curl`

 @example
 ```ts
 await fixture.answerCurl(registry(200,),);
 ```
 */
function registry(manifestStatus: number,): readonly CurlRule[] {
  return [
    { urlIncludes: TOKEN_ADDRESS, status: 200, body: JSON.stringify({ token: FAKE_BEARER, },), },
    { urlIncludes: MANIFEST_ADDRESS, status: manifestStatus, },
  ];
}

/**
 Runs one of the steps with the workflow's environment.

 @param fixture - Runner to run in

 @param program - File name of the step under test

 @param extra - More environment variables

 @returns What the step left behind

 @example
 ```ts
 const run = await runStepProgram({ fixture, program: 'playwright-image-tag.ts', },);
 ```
 */
async function runStepProgram(
  {
    fixture,
    program,
    extra = {},
  }: {
    readonly fixture: StepFixture;
    readonly program: string;
    readonly extra?: Readonly<Record<string, string>>;
  },
): Promise<StepRun> {
  return await fixture.run({
    command: [
      process.execPath,
      join(SOURCE_FOLDER, program,),
    ],
    env: {
      IMAGE,
      TAG,
      GITHUB_TOKEN: FAKE_SECRET,
      GITHUB_ACTOR_NAME: ACTOR,
      REPOSITORY_URL: 'https://github.com/Example/Repository',
      ...extra,
    },
  },);
}

/**
 Checks that the fake workflow token is in no stream or runner file of a run.

 @param run - What the step left behind

 @example
 ```ts
 expectNoSecret(run,);
 ```
 */
function expectNoSecret(run: StepRun,): void {
  expect(JSON.stringify(run,),).not.toContain(FAKE_SECRET,);
}

/**
 Reads the JSON lines a stand-in recorded.

 @param fixture - Runner whose records to read

 @param name - Record file name

 @returns One parsed value per line

 @example
 ```ts
 const calls = await recordsOf({ fixture, name: 'podman-calls.jsonl', },);
 ```
 */
async function recordsOf(
  {
    fixture,
    name,
  }: {
    readonly fixture: StepFixture;
    readonly name: string;
  },
): Promise<readonly unknown[]> {
  return (await readFile(join(fixture.records, name,), 'utf8',))
    .split('\n',)
    .filter(function isLine(line: string,) {
      return line !== '';
    },)
    .map(function parse(line: string,) {
      return JSON.parse(line,) as unknown;
    },);
}

/**
 What the response body file is called in the records, in place of its random name.
 */
const BODY_FILE = '<body file>';

/**
 Replaces the random name of the response body file in the recorded `curl` arguments.

 @param calls - Recorded argument lists

 @returns The same lists with the body file's path replaced by {@link BODY_FILE}

 @example
 ```ts
 const calls = withBodyFileNamed(await recordsOf({ fixture, name: 'curl-calls.jsonl', },),);
 ```
 */
function withBodyFileNamed(calls: readonly unknown[],): readonly unknown[] {
  return calls.map(function named(call: unknown,) {
    return (call as readonly string[]).map(function bodyOnly(argument: string,) {
      return argument.includes('/tool-',) ? BODY_FILE : argument;
    },);
  },);
}

/**
 Reads the whole text of a stand-in's record file.

 @param fixture - Runner whose records to read

 @param name - Record file name

 @returns The file's text

 @example
 ```ts
 const text = await recordText({ fixture, name: 'curl-config-input.log', },);
 ```
 */
async function recordText(
  {
    fixture,
    name,
  }: {
    readonly fixture: StepFixture;
    readonly name: string;
  },
): Promise<string> {
  return await readFile(join(fixture.records, name,), 'utf8',);
}

//endregion Fixtures: the registry's answers and the step environment

await describe({
  name: 'playwright image steps',
  children: [
    describe({
      name: 'compute the tag',
      children: [
        it({
          name: 'publishes the first 16 hex digits of the Dockerfile SHA-256 and prints them',
          fn: async function publishesTag() {
            await using fixture = await createStepFixture();
            await writeFile(join(fixture.checkout, 'playwright.Dockerfile',), 'FROM scratch\n',);
            const run = await runStepProgram({ fixture, program: 'playwright-image-tag.ts', },);
            /**
             What `sha256sum playwright.Dockerfile | cut --characters=1-16` prints.
             */
            const expected = await fixture.run({
              command: [
                '/usr/bin/sha256sum',
                'playwright.Dockerfile',
              ],
              env: {},
            },);
            /**
             The expected tag.
             */
            const tag = expected.stdout.slice(0, 16,);
            expect(run.status,).toBe(0,);
            expect(run.githubOutput,).toBe(`value=${tag}\n`,);
            expect(run.stdout,).toBe(`tag ${tag}\n`,);
          },
        },),
        it({
          name: 'ends with status 1 when there is no Dockerfile, and publishes nothing',
          fn: async function missingDockerfile() {
            await using fixture = await createStepFixture();
            const run = await runStepProgram({ fixture, program: 'playwright-image-tag.ts', },);
            expect(run.status,).toBe(1,);
            expect(run.githubOutput,).toBe('',);
            expect(run.stderr,).toContain('sha256sum: playwright.Dockerfile',);
          },
        },),
      ],
    },),
    describe({
      name: 'check whether the tag exists',
      children: [
        it({
          name: 'reports an existing tag as present, masks the bearer token and keeps credentials off the command line',
          fn: async function presentTag() {
            await using fixture = await createStepFixture();
            await fixture.answerCurl(registry(200,),);
            const run = await runStepProgram({ fixture, program: 'playwright-image-tag-exists.ts', },);
            expect(run.status,).toBe(0,);
            expect(run.githubOutput,).toBe('present=true\n',);
            expect(run.stdout,).toBe(`::add-mask::${FAKE_BEARER}\n${IMAGE}:${TAG} exists (HTTP 200); nothing to build\n`,);
            expect(run.stderr,).toBe('',);
            expect(JSON.stringify(run,).replaceAll(`::add-mask::${FAKE_BEARER}`, '',),).not.toContain(FAKE_BEARER,);
            expectNoSecret(run,);
            /**
             Arguments of the two requests, in the order the shell step gave them.
             */
            const calls = withBodyFileNamed(await recordsOf({ fixture, name: 'curl-calls.jsonl', },),);
            expect(calls,).toEqual([
              [
                '--silent',
                '--show-error',
                '--max-time',
                '30',
                '--retry',
                '3',
                '--retry-delay',
                '2',
                '--config',
                '-',
                '--output',
                BODY_FILE,
                '--write-out',
                '%{http_code}',
                TOKEN_ADDRESS,
              ],
              [
                '--silent',
                '--show-error',
                '--max-time',
                '30',
                '--retry',
                '3',
                '--retry-delay',
                '2',
                '--config',
                '-',
                '--head',
                '--output',
                '/dev/null',
                '--write-out',
                '%{http_code}',
                MANIFEST_ADDRESS,
              ],
            ],);
            expect(JSON.stringify(calls,),).not.toContain(FAKE_SECRET,);
            expect(JSON.stringify(calls,),).not.toContain(FAKE_BEARER,);
            expect(await recordText({ fixture, name: 'curl-config-input.log', },),).toBe(
              `user = "${ACTOR}:${FAKE_SECRET}"\n---\nheader = "Authorization: Bearer ${FAKE_BEARER}"\nheader = "Accept: application/vnd.oci.image.manifest.v1+json, application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.v2+json, application/vnd.docker.distribution.manifest.list.v2+json"\n---\n`,
            );
          },
        },),
        it({
          name: 'reports a missing tag (HTTP 404) as absent and succeeds so the build runs',
          fn: async function absentTag() {
            await using fixture = await createStepFixture();
            await fixture.answerCurl(registry(404,),);
            const run = await runStepProgram({ fixture, program: 'playwright-image-tag-exists.ts', },);
            expect(run.status,).toBe(0,);
            expect(run.githubOutput,).toBe('present=false\n',);
            expect(run.stdout,).toBe(`::add-mask::${FAKE_BEARER}\n${IMAGE}:${TAG} is absent (HTTP 404 for the tag); building it\n`,);
          },
        },),
        it({
          name: 'reports a refused token (HTTP 403) as absent without asking for the manifest',
          fn: async function refusedToken() {
            await using fixture = await createStepFixture();
            await fixture.answerCurl([{ urlIncludes: TOKEN_ADDRESS, status: 403, },],);
            const run = await runStepProgram({ fixture, program: 'playwright-image-tag-exists.ts', },);
            expect(run.status,).toBe(0,);
            expect(run.githubOutput,).toBe('present=false\n',);
            expect(run.stdout,).toBe(
              `${IMAGE}:${TAG} is absent (HTTP 403 for the token: the package does not exist yet, or it exists and this token has no access to it; the build runs, and a push this token may not make is refused); building it\n`,
            );
            expect(await recordsOf({ fixture, name: 'curl-calls.jsonl', },),).toHaveLength(1,);
          },
        },),
        it({
          name: 'fails the run, without an output, when the token request answers anything else',
          fn: async function tokenOther() {
            await using fixture = await createStepFixture();
            await fixture.answerCurl([{ urlIncludes: TOKEN_ADDRESS, status: 500, },],);
            const run = await runStepProgram({ fixture, program: 'playwright-image-tag-exists.ts', },);
            expect(run.status,).toBe(1,);
            expect(run.githubOutput,).toBe('',);
            expect(run.stdout,).toContain('::error title=Cannot tell whether the tag exists::The token request answered HTTP 500. Nothing was built or pushed',);
          },
        },),
        ...[
          { name: 'has no token field', body: '{}', },
          { name: 'is not JSON', body: 'not json', },
          { name: 'is empty', body: '', },
          { name: 'has an empty token', body: '{"token":""}', },
          { name: 'has a null token', body: '{"token":null}', },
        ].map(function noTokenCase({ name, body, }: { readonly name: string; readonly body: string; },) {
          return it({
            name: `fails the run when the token response ${name}`,
            fn: async function noToken() {
              await using fixture = await createStepFixture();
              await fixture.answerCurl([{ urlIncludes: TOKEN_ADDRESS, status: 200, body, },],);
              const run = await runStepProgram({ fixture, program: 'playwright-image-tag-exists.ts', },);
              expect(run.status,).toBe(1,);
              expect(run.githubOutput,).toBe('',);
              expect(run.stdout,).toContain('::error title=Cannot tell whether the tag exists::The token response had no token. Nothing',);
              expect(run.stdout,).not.toContain('::add-mask::',);
            },
          },);
        },),
        it({
          name: 'fails the run when the manifest request answers neither 200 nor 404',
          fn: async function manifestOther() {
            await using fixture = await createStepFixture();
            await fixture.answerCurl(registry(503,),);
            const run = await runStepProgram({ fixture, program: 'playwright-image-tag-exists.ts', },);
            expect(run.status,).toBe(1,);
            expect(run.githubOutput,).toBe('',);
            expect(run.stdout,).toContain('::error title=Cannot tell whether the tag exists::The manifest request answered HTTP 503. Nothing');
          },
        },),
        it({
          name: 'ends with the status of curl when a request cannot be made, so the build and push are skipped',
          fn: async function transportFault() {
            await using fixture = await createStepFixture();
            await fixture.answerCurl([{ urlIncludes: TOKEN_ADDRESS, status: 0, exit: 28, },],);
            const first = await runStepProgram({ fixture, program: 'playwright-image-tag-exists.ts', },);
            expect(first.status,).toBe(28,);
            expect(first.stderr,).toBe('curl: (28) fake transport failure\n',);
            expect(first.githubOutput,).toBe('',);
            await fixture.answerCurl([
              { urlIncludes: TOKEN_ADDRESS, status: 200, body: JSON.stringify({ token: FAKE_BEARER, },), },
              { urlIncludes: MANIFEST_ADDRESS, status: 0, exit: 6, },
            ],);
            const second = await runStepProgram({ fixture, program: 'playwright-image-tag-exists.ts', },);
            expect(second.status,).toBe(6,);
            expect(second.githubOutput,).toBe('',);
          },
        },),
      ],
    },),
    describe({
      name: 'build the image',
      children: [
        it({
          name: 'prints the podman version, then builds with the labels and the tag',
          fn: async function builds() {
            await using fixture = await createStepFixture();
            const run = await runStepProgram({ fixture, program: 'playwright-image-build.ts', },);
            expect(run.status,).toBe(0,);
            expect(run.stdout,).toBe('podman version 9.9.9-fake\nSTEP 1/1: FROM fake\n',);
            expect(await recordsOf({ fixture, name: 'podman-calls.jsonl', },),).toEqual([
              ['--version',],
              [
                'build',
                '--file',
                'playwright.Dockerfile',
                '--label',
                'org.opencontainers.image.source=https://github.com/Example/Repository',
                '--label',
                'org.opencontainers.image.description=Browsers and system libraries for the Playwright tests of the Monochromatic monorepo (playwright.Dockerfile)',
                '--tag',
                `${IMAGE}:${TAG}`,
                '.',
              ],
            ],);
            expectNoSecret(run,);
          },
        },),
        it({
          name: 'ends with the status of a failing build',
          fn: async function buildFails() {
            await using fixture = await createStepFixture();
            const run = await runStepProgram({
              fixture,
              program: 'playwright-image-build.ts',
              extra: { FAKE_PODMAN_BUILD_EXIT: '125', },
            },);
            expect(run.status,).toBe(125,);
          },
        },),
      ],
    },),
    describe({
      name: 'push the image',
      children: [
        it({
          name: 'logs in with the token on standard input only, pushes, and publishes the digest',
          fn: async function pushes() {
            await using fixture = await createStepFixture();
            const run = await runStepProgram({ fixture, program: 'playwright-image-push.ts', },);
            /**
             The digest the stand-in `podman` writes when it is not told otherwise.
             */
            const digest = `sha256:${'a'.repeat(64,)}`;
            expect(run.status,).toBe(0,);
            expect(run.githubOutput,).toBe(`digest=${digest}\n`,);
            expect(run.stdout,).toBe(
              `Login Succeeded!\nWriting manifest to image destination\npushed ${IMAGE}:${TAG} as ${digest}\n`,
            );
            expect(await recordsOf({ fixture, name: 'podman-calls.jsonl', },),).toEqual([
              [
                'login',
                'ghcr.io',
                '--username',
                ACTOR,
                '--password-stdin',
              ],
              [
                'push',
                '--digestfile',
                join(fixture.runnerTemp, 'playwright-image.digest',),
                `${IMAGE}:${TAG}`,
              ],
            ],);
            expect(await recordText({ fixture, name: 'podman-login-input.log', },),).toBe(`${FAKE_SECRET}---\n`,);
            expectNoSecret(run,);
          },
        },),
        it({
          name: 'drops the line break after a digest',
          fn: async function trimsDigest() {
            await using fixture = await createStepFixture();
            const run = await runStepProgram({
              fixture,
              program: 'playwright-image-push.ts',
              extra: { FAKE_PODMAN_DIGEST: `sha256:${'b'.repeat(64,)}\n\n`, },
            },);
            expect(run.status,).toBe(0,);
            expect(run.githubOutput,).toBe(`digest=sha256:${'b'.repeat(64,)}\n`,);
          },
        },),
        ...[
          'garbage',
          '',
          'sha512:abc',
        ].map(function badDigestCase(digest: string,) {
          return it({
            name: `fails and publishes nothing when podman wrote ${JSON.stringify(digest,)} as the digest`,
            fn: async function badDigest() {
              await using fixture = await createStepFixture();
              const run = await runStepProgram({
                fixture,
                program: 'playwright-image-push.ts',
                extra: { FAKE_PODMAN_DIGEST: digest, },
              },);
              expect(run.status,).toBe(1,);
              expect(run.githubOutput,).toBe('',);
              expect(run.stdout,).toContain(
                `::error title=Unexpected digest::podman wrote '${digest}' as the digest of the pushed image\n`,
              );
            },
          },);
        },),
        it({
          name: 'ends with the status of a failing push and publishes nothing',
          fn: async function pushFails() {
            await using fixture = await createStepFixture();
            const run = await runStepProgram({
              fixture,
              program: 'playwright-image-push.ts',
              extra: { FAKE_PODMAN_PUSH_EXIT: '125', },
            },);
            expect(run.status,).toBe(125,);
            expect(run.githubOutput,).toBe('',);
          },
        },),
      ],
    },),
    describe({
      name: 'check that anyone can pull it',
      children: [
        it({
          name: 'records a public image in the summary and says anyone can pull it',
          fn: async function publicImage() {
            await using fixture = await createStepFixture();
            await fixture.answerCurl(registry(200,),);
            const run = await runStepProgram({ fixture, program: 'playwright-image-anonymous-pull-check.ts', },);
            expect(run.status,).toBe(0,);
            expect(run.githubSummary,).toBe(`## Playwright image\n- image: \`${IMAGE}:${TAG}\`\n- anonymous pull check: HTTP 200\n`,);
            expect(run.stdout,).toBe(`::add-mask::${FAKE_BEARER}\nanyone can pull ${IMAGE}:${TAG}\n`,);
            expectNoSecret(run,);
            /**
             The first request carries no credentials: no `--config -`, so nothing is read from standard input.
             */
            const calls = withBodyFileNamed(await recordsOf({ fixture, name: 'curl-calls.jsonl', },),);
            expect(calls[0],).toEqual([
              '--silent',
              '--max-time',
              '30',
              '--retry',
              '3',
              '--retry-delay',
              '2',
              '--output',
              BODY_FILE,
              '--write-out',
              '%{http_code}',
              TOKEN_ADDRESS,
            ],);
            expect(await recordText({ fixture, name: 'curl-config-input.log', },),).not.toContain('user =',);
          },
        },),
        it({
          name: 'warns, with the registry error code, when the anonymous token is refused',
          fn: async function privateImage() {
            await using fixture = await createStepFixture();
            await fixture.answerCurl([
              { urlIncludes: TOKEN_ADDRESS, status: 403, body: '{"errors":[{"code":"DENIED","message":"denied"}]}', },
            ],);
            const run = await runStepProgram({ fixture, program: 'playwright-image-anonymous-pull-check.ts', },);
            expect(run.status,).toBe(0,);
            expect(run.githubSummary,).toContain('- anonymous pull check: HTTP 403 (DENIED)\n',);
            expect(run.stdout,).toContain(
              `::warning title=Playwright image is not public::An anonymous request for ${IMAGE}:${TAG} answered HTTP 403 (DENIED), so the package is private or the tag is missing.`,
            );
          },
        },),
        ...[
          '',
          'not json',
          '{}',
          '{"errors":{}}',
          '{"errors":[]}',
          '{"errors":["text"]}',
          '{"errors":[{"code":null}]}',
        ].map(function noCodeCase(body: string,) {
          return it({
            name: `shows only the status when the refusal body is ${JSON.stringify(body,)}`,
            fn: async function refusalWithoutCode() {
              await using fixture = await createStepFixture();
              await fixture.answerCurl([{ urlIncludes: TOKEN_ADDRESS, status: 401, body, },],);
              const run = await runStepProgram({ fixture, program: 'playwright-image-anonymous-pull-check.ts', },);
              expect(run.status,).toBe(0,);
              expect(run.githubSummary,).toBe(
                `## Playwright image\n- image: \`${IMAGE}:${TAG}\`\n- anonymous pull check: HTTP 401\n`,
              );
            },
          },);
        },),
        it({
          name: 'warns with the manifest status when the tag is missing, and never fails the run',
          fn: async function missingTag() {
            await using fixture = await createStepFixture();
            await fixture.answerCurl(registry(404,),);
            const run = await runStepProgram({ fixture, program: 'playwright-image-anonymous-pull-check.ts', },);
            expect(run.status,).toBe(0,);
            expect(run.githubSummary,).toContain('- anonymous pull check: HTTP 404\n',);
            expect(run.stdout,).toContain('::warning title=Playwright image is not public::An anonymous request for',);
          },
        },),
        it({
          name: 'reports 000 when nothing answers, and still succeeds',
          fn: async function nothingAnswers() {
            await using fixture = await createStepFixture();
            await fixture.answerCurl([{ urlIncludes: TOKEN_ADDRESS, status: 0, exit: 28, },],);
            const first = await runStepProgram({ fixture, program: 'playwright-image-anonymous-pull-check.ts', },);
            expect(first.status,).toBe(0,);
            expect(first.githubSummary,).toContain('- anonymous pull check: HTTP 000\n',);
            await fixture.answerCurl([
              { urlIncludes: TOKEN_ADDRESS, status: 200, body: JSON.stringify({ token: FAKE_BEARER, },), },
              { urlIncludes: MANIFEST_ADDRESS, status: 0, exit: 6, },
            ],);
            const second = await runStepProgram({ fixture, program: 'playwright-image-anonymous-pull-check.ts', },);
            expect(second.status,).toBe(0,);
            expect(second.githubSummary,).toContain('- anonymous pull check: HTTP 000\n',);
          },
        },),
        it({
          name: 'reads a token answer without a token as the token status alone',
          fn: async function tokenWithoutToken() {
            await using fixture = await createStepFixture();
            await fixture.answerCurl([{ urlIncludes: TOKEN_ADDRESS, status: 200, body: '{}', },],);
            const run = await runStepProgram({ fixture, program: 'playwright-image-anonymous-pull-check.ts', },);
            expect(run.status,).toBe(0,);
            expect(run.githubSummary,).toContain('- anonymous pull check: HTTP 200\n',);
            expect(run.stdout,).toBe(`anyone can pull ${IMAGE}:${TAG}\n`,);
          },
        },),
      ],
    },),
  ],
},);
