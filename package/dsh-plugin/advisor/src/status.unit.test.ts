/**
 Unit tests for the DSH `/advisor status` text.
 */

import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { SessionId, } from '@deepseek-ai/dsh-session';

import {
  DEFAULT_CONFIG,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';

import {
  buildDshAdvisorCatalog,
  buildDshAdvisorStatus,
  type DshAdvisorLlmSurface,
} from '../dist/final/node/index.mjs';

/**
 Configured Advisor output requirement.
 */
const ADVISOR_OUTPUT_TOKENS = 100;

/**
 Advertised route identity.
 */
type FixtureRoute = {
  /**
   Provider route id.
   */
  readonly provider: string;
  /**
   Model id on that provider route.
   */
  readonly id: string;
  /**
   Display name of the model.
   */
  readonly name: string;
};

/**
 Status config fixture carrying source metadata.
 */
const advisorConfig = {
  ...DEFAULT_CONFIG,
  maxAdvisorOutputTokens: ADVISOR_OUTPUT_TOKENS,
  source: {
    globalPath: '/home/test/.pi/agent/extensions/dsh-advisor.json',
    projectPath: '/repo/.pi/extensions/dsh-advisor.json',
    globalLoaded: false,
    projectLoaded: false,
  },
} as unknown as Parameters<typeof buildDshAdvisorStatus>[0]['config'];

/**
 Build one scripted LLM surface advertising the given routes.

 @param models - advertised route identities

 @returns LLM surface whose catalog lists the given routes

 @example
 ```ts
 const llm = scriptedLlmSurface([{ provider: 'fake', id: 'reviewer', name: 'Reviewer', }]);
 ```
 */
function scriptedLlmSurface(
  models: readonly FixtureRoute[],
): DshAdvisorLlmSurface {
  return {
    listProviders: function listProviders() {
      return [{
        id: 'fake',
        name: 'Fake',
      },];
    },
    listModels: async function listModels() {
      return [...models,];
    },
    resolveModelInfo: async function resolveModelInfo(
      provider: string,
      id: string,
    ) {
      /**
       Advertised route matching the requested identity.
       */
      const model = models.find(function matches(candidate: FixtureRoute,): boolean {
        return (candidate.provider === provider) && (candidate.id === id);
      },);
      return {
        provider,
        id,
        name: model?.name ?? id,
        context: { contextWindow: 100_000, },
        defaultMaxTokens: 1_000,
      };
    },
  } as unknown as DshAdvisorLlmSurface;
}

/**
 Disposable sandbox directory standing in for both home and working directory.
 */
type TempDirectory = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Create a disposable sandbox directory for settings-free scope resolution.

 @returns directory removed when async disposal completes

 @example
 ```ts
 await using tempDirectory = await createTempDirectory();
 ```
 */
async function createTempDirectory(): Promise<TempDirectory> {
  /**
   Absolute sandbox root unique to the current probe.
   */
  const path = await mkdtemp(join(
    tmpdir(),
    'dsh-advisor-status-',
  ),);

  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Build `/advisor status` text for the given advertised routes.

 @param models - advertised route identities

 @returns status text

 @example
 ```ts
 const text = await statusFor([{ provider: 'fake', id: 'reviewer', name: 'Reviewer', }]);
 ```
 */
async function statusFor(
  models: readonly FixtureRoute[],
): Promise<string> {
  await using tempDirectory = await createTempDirectory();
  return await buildDshAdvisorStatus({
    catalog: await buildDshAdvisorCatalog({
      llm: scriptedLlmSurface(models,),
      sessionId: SessionId('session-status-test',),
      maxAdvisorOutputTokens: ADVISOR_OUTPUT_TOKENS,
    },),
    providerIds: ['fake',],
    cwd: tempDirectory.path,
    config: advisorConfig,
    enabled: true,
    transcriptDegradationReason: '',
    home: tempDirectory.path,
  },);
}

await describe({
  name: buildDshAdvisorStatus.name,
  children: [
    it({
      name: 'reports the scoped-model count instead of the slug list',
      fn: async () => {
        const text = await statusFor([
          {
            provider: 'fake',
            id: 'reviewer',
            name: 'Reviewer',
          },
          {
            provider: 'fake',
            id: 'limited-reviewer',
            name: 'Limited Reviewer',
          },
        ],);
        expect(text,).toContain('Scoped models: 2',);
        expect(text,).not.toContain('fake/reviewer, fake/limited-reviewer',);
        expect(text,).toContain(
          `Advisor request budget: ${String(ADVISOR_OUTPUT_TOKENS,)} output tokens, clamped to each route's advertised maximum`,
        );
      },
    },),
    it({
      name: 'reports none when no routes are advertised',
      fn: async () => {
        const text = await statusFor([],);
        expect(text,).toContain('Scoped models: none',);
      },
    },),
  ],
},);
