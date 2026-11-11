/**
 Tests for the Qwen Token Plan Individual credential seam:
 the resolution chain order,
 its trimming rules,
 and the api-key auth object pi resolves per request.

 @module
 */

import {
  QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type {
  ApiKeyCredential,
  AuthContext,
  Credential,
  OAuthCredential,
} from '@earendil-works/pi-ai';
import type {
  Logger,
} from '@monochromatic-dev/logger-core/ts';
import {
  createQwenTokenPlanIndividualApiKeyAuth,
  isApiKeyCredential,
  resolveQwenTokenPlanIndividualCredential,
  storedCredentialSources,
  QWEN_TOKEN_PLAN_INDIVIDUAL_BUILTIN_CREDENTIAL_ENV,
  QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_GUIDANCE,
  QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_LABEL,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 One captured log record.
 */
type LogRecord = {
  readonly level: string;
  readonly message: string;
};

/**
 Logger capturing every message, so tests read what the seam reported.

 @returns the logger and the records it captured
 */
function capturingLogger(): {
  readonly logger: Logger;
  readonly records: LogRecord[];
} {
  /**
   Records the logger captured, in order.
   */
  const records: LogRecord[] = [];
  /**
   Capturing implementation of the logger shape.
   */
  function capture(level: string, message: string,): void {
    records.push({
      level,
      message,
    },);
  }
  return {
    logger: {
      debug: function debug(message: string,): void {
        capture('debug', message,);
      },
      error: function error(message: string,): void {
        capture('error', message,);
      },
      fatal: function fatal(message: string,): void {
        capture('fatal', message,);
      },
      flush: function flush(): Promise<void> {
        return Promise.resolve();
      },
      info: function info(message: string,): void {
        capture('info', message,);
      },
      trace: function trace(message: string,): void {
        capture('trace', message,);
      },
      warn: function warn(message: string,): void {
        capture('warn', message,);
      },
    },
    records,
  };
}

/**
 Auth context serving a fixed environment slice.

 @param environment - values the env seam answers with

 @returns the auth context
 */
function fixtureAuthContext(environment: NodeJS.ProcessEnv,): AuthContext {
  return {
    env: function env(name: string,) {
      return Promise.resolve(environment[name]);
    },
    fileExists: function fileExists(): Promise<boolean> {
      return Promise.resolve(false,);
    },
  };
}

/**
 Stored api-key credential fixture.

 @param key - key the credential carries

 @returns the credential record
 */
function apiKeyCredential(key: string,): ApiKeyCredential {
  return {
    type: 'api_key',
    key,
  };
}

//endregion Fixtures

//region Resolution chain

await describe({
  name: resolveQwenTokenPlanIndividualCredential.name,
  children: [
    it({
      name: 'prefers the pi stored credential over both environment homes',
      fn: async () => {
        expect(resolveQwenTokenPlanIndividualCredential({
          sources: {
            credentialKey: 'stored-key',
            individualEnvironmentKey: 'individual-key',
            builtinEnvironmentKey: 'builtin-key',
          },
        },),)
          .toEqual({
            key: 'stored-key',
            source: 'pi-credential',
          },);
      },
    },),
    it({
      name: 'prefers the individual environment name over the builtin fallback',
      fn: async () => {
        expect(resolveQwenTokenPlanIndividualCredential({
          sources: {
            individualEnvironmentKey: 'individual-key',
            builtinEnvironmentKey: 'builtin-key',
          },
        },),)
          .toEqual({
            key: 'individual-key',
            source: 'individual-environment',
          },);
      },
    },),
    it({
      name: 'falls back to the builtin environment name last',
      fn: async () => {
        expect(resolveQwenTokenPlanIndividualCredential({
          sources: { builtinEnvironmentKey: 'builtin-key', },
        },),)
          .toEqual({
            key: 'builtin-key',
            source: 'builtin-environment',
          },);
      },
    },),
    it({
      name: 'reports absence when no home carries a key',
      fn: async () => {
        expect(resolveQwenTokenPlanIndividualCredential({ sources: {}, },),)
          .toEqual({ source: 'none', },);
      },
    },),
    it({
      name: 'treats whitespace-only home values as absent and trims real keys',
      fn: async () => {
        expect(resolveQwenTokenPlanIndividualCredential({
          sources: {
            credentialKey: '   ',
            individualEnvironmentKey: '\t',
            builtinEnvironmentKey: '  builtin-key  ',
          },
        },),)
          .toEqual({
            key: 'builtin-key',
            source: 'builtin-environment',
          },);
      },
    },),
    it({
      name: 'treats an empty string home value as absent',
      fn: async () => {
        expect(resolveQwenTokenPlanIndividualCredential({
          sources: {
            credentialKey: '',
            individualEnvironmentKey: 'individual-key',
          },
        },),)
          .toEqual({
            key: 'individual-key',
            source: 'individual-environment',
          },);
      },
    },),
  ],
},);

//endregion Resolution chain

//region Stored credential narrowing

await describe({
  name: storedCredentialSources.name,
  children: [
    it({
      name: 'fills the sources fragment with the key of an api-key credential',
      fn: async () => {
        expect(storedCredentialSources({
          credential: apiKeyCredential('stored-key',),
        },),)
          .toEqual({ credentialKey: 'stored-key', },);
      },
    },),
    it({
      name: 'carries no key when pi resolved no credential',
      fn: async () => {
        expect(storedCredentialSources({},),)
          .toEqual({},);
      },
    },),
    it({
      name: 'reads nothing from an oauth credential',
      fn: async () => {
        /**
         OAuth record whose index signature would answer `key` with a
         non-string.
         */
        const credential: OAuthCredential = {
          type: 'oauth',
          refresh: 'refresh-token',
          access: 'access-token',
          expires: 0,
        };
        expect(isApiKeyCredential(credential,),).toBe(false);
        expect(storedCredentialSources({ credential, },),)
          .toEqual({},);
      },
    },),
  ],
},);

//endregion Stored credential narrowing

//region Pi auth object

await describe({
  name: createQwenTokenPlanIndividualApiKeyAuth.name,
  children: [
    it({
      name: 'names the credential exactly as the builtin provider does',
      fn: async () => {
        const { logger, } = capturingLogger();
        const auth = createQwenTokenPlanIndividualApiKeyAuth({ logger, },);
        expect(auth.name,)
          .toBe(QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_LABEL);
      },
    },),
    it({
      name: 'resolves the stored credential first and labels its home',
      fn: async () => {
        const { logger, } = capturingLogger();
        const auth = createQwenTokenPlanIndividualApiKeyAuth({ logger, },);
        /**
         Resolved auth the seam reported.
         */
        const resolved = await auth.resolve({
          ctx: fixtureAuthContext({
            [QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME]: 'individual-key',
            [QWEN_TOKEN_PLAN_INDIVIDUAL_BUILTIN_CREDENTIAL_ENV]: 'builtin-key',
          },),
          credential: apiKeyCredential('stored-key',),
          signal: new AbortController().signal,
        },);
        expect(resolved,).toEqual({
          auth: { apiKey: 'stored-key', },
          source: QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_LABEL,
        },);
      },
    },),
    it({
      name: 'resolves the individual environment name and labels it',
      fn: async () => {
        const { logger, } = capturingLogger();
        const auth = createQwenTokenPlanIndividualApiKeyAuth({ logger, },);
        /**
         Resolved auth the seam reported.
         */
        const resolved = await auth.resolve({
          ctx: fixtureAuthContext({
            [QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME]: 'individual-key',
            [QWEN_TOKEN_PLAN_INDIVIDUAL_BUILTIN_CREDENTIAL_ENV]: 'builtin-key',
          },),
          signal: new AbortController().signal,
        },);
        expect(resolved,).toEqual({
          auth: { apiKey: 'individual-key', },
          source: QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME,
        },);
      },
    },),
    it({
      name: 'resolves the builtin environment name and labels it',
      fn: async () => {
        const { logger, } = capturingLogger();
        const auth = createQwenTokenPlanIndividualApiKeyAuth({ logger, },);
        /**
         Resolved auth the seam reported.
         */
        const resolved = await auth.resolve({
          ctx: fixtureAuthContext({
            [QWEN_TOKEN_PLAN_INDIVIDUAL_BUILTIN_CREDENTIAL_ENV]: 'builtin-key',
          },),
          signal: new AbortController().signal,
        },);
        expect(resolved,).toEqual({
          auth: { apiKey: 'builtin-key', },
          source: QWEN_TOKEN_PLAN_INDIVIDUAL_BUILTIN_CREDENTIAL_ENV,
        },);
      },
    },),
    it({
      name: 'reports nothing configured and logs the guidance when no home answers',
      fn: async () => {
        const { logger, records, } = capturingLogger();
        const auth = createQwenTokenPlanIndividualApiKeyAuth({ logger, },);
        /**
         Resolved auth the seam reported.
         */
        const resolved = await auth.resolve({
          ctx: fixtureAuthContext({}),
          signal: new AbortController().signal,
        },);
        expect(resolved,).toBe(undefined);
        expect(records.some(function loggedGuidance(record,) {
          return record.message.includes(QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_GUIDANCE,);
        },),)
          .toBe(true);
      },
    },),
    it({
      name: 'never logs the resolved key',
      fn: async () => {
        const { logger, records, } = capturingLogger();
        const auth = createQwenTokenPlanIndividualApiKeyAuth({ logger, },);
        await auth.resolve({
          ctx: fixtureAuthContext({}),
          credential: apiKeyCredential('super-secret-key',),
          signal: new AbortController().signal,
        },);
        expect(records.some(function leaked(record,) {
          return record.message.includes('super-secret-key',);
        },),)
          .toBe(false);
      },
    },),
  ],
},);

//endregion Pi auth object
