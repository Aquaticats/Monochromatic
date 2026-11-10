/**
 Unit tests for the pnpr publish steps' GitHub ID token request.

 GitHub answers the ID token endpoint with an HTML proxy page when its own
 upstream fails, so the request must report the HTTP status rather than fail on
 the parse of a body it will never read.

 @module
 */

import {
  describe,
  expect,
  it,
  type TestContext,
} from '@monochromatic-dev/test-harness-core/ts';

import { requestPublishToken, } from '../dist/final/node/index.mjs';

//region Fixtures

/**
 OIDC audience the cases request a token for.
 */
const AUDIENCE = 'https://pnpr.example.test';

/**
 Environment variable holding the GitHub ID token request endpoint.
 */
const REQUEST_URL_ENV = 'ACTIONS_ID_TOKEN_REQUEST_URL';

/**
 Environment variable holding the bearer for that endpoint.
 */
const REQUEST_TOKEN_ENV = 'ACTIONS_ID_TOKEN_REQUEST_TOKEN';

/**
 Environment variable holding a local token that skips the request entirely.
 */
const OVERRIDE_ENV = 'PNPR_PUBLISH_TOKEN';

/**
 Request endpoint the cases answer instead of GitHub.
 */
const REQUEST_URL = 'https://actions.example.test/id-token?api-version=2.0';

/**
 Bearer the request must carry to that endpoint.
 */
const REQUEST_TOKEN = 'github-request-token';

/**
 Prefix of the registry credential the publish run forwards to pnpr.
 */
const WORKLOAD_PREFIX = 'pnpr_workload_';

/**
 Local override the cases install, already carrying the credential prefix the caller adds.
 */
const LOCAL_OVERRIDE = `${WORKLOAD_PREFIX}local-override`;

/**
 Override value the request treats as unset, so it still asks GitHub.
 */
const EMPTY_OVERRIDE = '';

/**
 Body of a proxy error page, which is not JSON.
 */
const PROXY_ERROR_PAGE = '<html><body>502 Bad Gateway</body></html>';

/**
 HTTP status the proxy error page carries.
 */
const PROXY_ERROR_STATUS = 502;

/**
 Token value a successful GitHub response carries.
 */
const GITHUB_TOKEN_VALUE = 'id-token-value';

/**
 Request variables GitHub sets inside Actions.
 */
const GITHUB_REQUEST_VARIABLES: readonly (readonly [string, string])[] = [
  [REQUEST_URL_ENV, REQUEST_URL,],
  [REQUEST_TOKEN_ENV, REQUEST_TOKEN,],
];

/**
 Every variable a case may install, so clearing one case's environment is one list.
 */
const TOKEN_ENVIRONMENT_NAMES = [
  REQUEST_URL_ENV,
  REQUEST_TOKEN_ENV,
  OVERRIDE_ENV,
] as const;

/**
 Captures the error an async call rejects with.

 @param call - Operation expected to reject.

 @returns The rejection value, or `undefined` when the call resolved.

 @example
 ```ts
 const caught = await caughtBy(async () => { throw new Error('x'); });
 ```
 */
async function caughtBy(call: () => Promise<unknown>,): Promise<unknown> {
  try {
    await call();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Narrows a rejection to its message, failing the test when it is not an `Error`.

 @param caught - Rejection value from {@link caughtBy}.

 @returns The error message.

 @example
 ```ts
 expect(errorMessage(await caughtBy(() => requestPublishToken({ audience })))).toBe('...');
 ```
 */
function errorMessage(caught: unknown,): string {
  if (!Error.isError(caught,))
    throw new Error(`expected an Error, received ${String(caught,)}`,);
  return caught.message;
}

/**
 Puts one case's token environment in place, clearing whatever an earlier case left.

 The cases share one process and every variable below is process-global, so the
 suite runs them one at a time and each case installs its whole environment.

 @param variables - Values to set, keyed by variable name; a variable left out stays unset.

 @example
 ```ts
 installTokenEnvironment([...GITHUB_REQUEST_VARIABLES, [OVERRIDE_ENV, LOCAL_OVERRIDE]]);
 ```
 */
function installTokenEnvironment(variables: readonly (readonly [string, string])[],): void {
  for (const name of TOKEN_ENVIRONMENT_NAMES) {
    // `process.env` is an exotic object whose delete trap is the only way to unset a variable, so a computed name is the API here.
    Reflect.deleteProperty(
      process.env,
      name,
    );
  }
  for (const [name, value,] of variables)
    process.env[name] = value;
}

//endregion Fixtures

await describe({
  name: requestPublishToken.name,
  // Every case replaces `globalThis.fetch` and the process environment, which the next case would see.
  concurrency: 1,
  children: [
    it({
      name: 'reports the HTTP status of a non-JSON error body instead of a parse error',
      fn: async ({ sinon, }: TestContext,) => {
        sinon.stub(globalThis, 'fetch',)
          .resolves(new Response(
            PROXY_ERROR_PAGE,
            {
              status: PROXY_ERROR_STATUS,
              headers: { 'content-type': 'text/html', },
            },
          ),);
        installTokenEnvironment(GITHUB_REQUEST_VARIABLES,);
        const caught = await caughtBy(async () => {
          await requestPublishToken({ audience: AUDIENCE, },);
        },);
        expect(caught,).not.toBeInstanceOf(SyntaxError,);
        expect(errorMessage(caught,),).toBe(`GitHub ID token request failed with HTTP ${PROXY_ERROR_STATUS}`,);
      },
    },),
    it({
      name: 'returns the workload credential from a GitHub ID token and masks it before printing',
      fn: async ({ sinon, }: TestContext,) => {
        const printed = sinon.stub(console, 'log',);
        sinon.stub(globalThis, 'fetch',)
          .resolves(Response.json({ value: GITHUB_TOKEN_VALUE, },),);
        installTokenEnvironment(GITHUB_REQUEST_VARIABLES,);
        expect(await requestPublishToken({ audience: AUDIENCE, },),).toBe(`${WORKLOAD_PREFIX}${GITHUB_TOKEN_VALUE}`,);
        expect(printed.calledWith(`::add-mask::${GITHUB_TOKEN_VALUE}`,),).toBe(true,);
      },
    },),
    it({
      name: 'throws when an accepted response carries no string value',
      fn: async ({ sinon, }: TestContext,) => {
        sinon.stub(globalThis, 'fetch',)
          .resolves(Response.json({ value: [GITHUB_TOKEN_VALUE,], },),);
        installTokenEnvironment(GITHUB_REQUEST_VARIABLES,);
        const caught = await caughtBy(async () => {
          await requestPublishToken({ audience: AUDIENCE, },);
        },);
        expect(errorMessage(caught,),).toBe('GitHub ID token response carries no string value',);
      },
    },),
    it({
      name: 'throws outside Actions without requesting a token when the request variables are absent',
      fn: async ({ sinon, }: TestContext,) => {
        const requested = sinon.stub(globalThis, 'fetch',);
        installTokenEnvironment([],);
        const caught = await caughtBy(async () => {
          await requestPublishToken({ audience: AUDIENCE, },);
        },);
        expect(errorMessage(caught,),).toContain('no GitHub OIDC request environment',);
        expect(requested.called,).toBe(false,);
      },
    },),
    it({
      name: 'returns the local override without requesting a token',
      fn: async ({ sinon, }: TestContext,) => {
        const requested = sinon.stub(globalThis, 'fetch',);
        installTokenEnvironment([
          ...GITHUB_REQUEST_VARIABLES,
          [OVERRIDE_ENV, LOCAL_OVERRIDE,],
        ],);
        expect(await requestPublishToken({ audience: AUDIENCE, },),).toBe(LOCAL_OVERRIDE,);
        expect(requested.called,).toBe(false,);
      },
    },),
    it({
      name: 'ignores an empty local override and requests a token',
      fn: async ({ sinon, }: TestContext,) => {
        sinon.stub(globalThis, 'fetch',)
          .resolves(Response.json({ value: GITHUB_TOKEN_VALUE, },),);
        installTokenEnvironment([
          ...GITHUB_REQUEST_VARIABLES,
          [OVERRIDE_ENV, EMPTY_OVERRIDE,],
        ],);
        expect(await requestPublishToken({ audience: AUDIENCE, },),).toBe(`${WORKLOAD_PREFIX}${GITHUB_TOKEN_VALUE}`,);
      },
    },),
  ],
},);
