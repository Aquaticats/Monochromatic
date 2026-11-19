/**
 Verify discovery, request-time registry authentication and the actual session record.
 Invoked only by verify-extension.ts, in a disposable offline home.
 @module
 */

import assert from 'node:assert/strict';
import { fileURLToPath, } from 'node:url';
import {
  InMemoryCredentialStore,
  type Model,
  type Api,
  type Provider,
} from '@earendil-works/pi-ai';
import {
  createAgentSession,
  DefaultResourceLoader,
  ModelRuntime,
  SessionManager,
  SettingsManager,
} from '@earendil-works/pi-coding-agent';
import {
  immediate,
  MEMBER_KEY,
  PHYSICAL_MODELS,
} from './stand-in-pi-fixture.ts';

const cwd = process.cwd();
const credentials = new InMemoryCredentialStore();
const modelRuntime = await ModelRuntime.create({
  credentials,
  modelsPath: null,
  refreshOnCreate: false,
  allowModelNetwork: false,
},);
const calls: string[] = [];

/**
 Register a real Pi provider object with a scripted, no-network transport.
 
 @param model - exact fixture member to register
 
 @param refused - whether that member reports a rate limit
 */
async function registerStandIn({
  model,
  refused,
}: {
  model: Model<Api>;
  refused: boolean;
},): Promise<void> {
  const provider: Provider = {
    id: model.provider,
    name: model.provider,
    getModels() { return [model,]; },
    auth: {
      apiKey: {
        name: 'Stand-in credential',
        resolve({ credential, }) {
          return Promise.resolve(credential?.key === undefined
            ? undefined
            : {
              auth: { apiKey: credential.key, },
              source: 'test',
            },);
        },
      },
    },
    stream: () => { throw new Error('Unexpected API-specific request',); },
    streamSimple(
      physical,
      _context,
      options,
    ) {
      if (options?.apiKey !== MEMBER_KEY)
        throw new Error('Pi registry did not resolve the stand-in credential',);
      calls.push(physical.provider,);
      return immediate({
        model: physical,
        ...(refused ? { errorMessage: 'HTTP 429: Stand-in capacity', } : {}),
      },);
    },
  };
  modelRuntime.registerNativeProvider(provider,);
  await modelRuntime.setRuntimeApiKey(
    model.provider,
    MEMBER_KEY,
  );
}

const [head, fallback,] = PHYSICAL_MODELS;
assert.ok(head,);
assert.ok(fallback,);
await registerStandIn({
  model: head,
  refused: true,
},);
await registerStandIn({
  model: fallback,
  refused: false,
},);

const providerId = 'rate-limit-fallback';
const settingsManager = SettingsManager.inMemory({
  defaultProvider: providerId,
  defaultModel: 'writer',
  retry: { enabled: false, },
},);
const loader = new DefaultResourceLoader({
  cwd,
  agentDir: cwd,
  settingsManager,
  noExtensions: true,
  noSkills: true,
  noPromptTemplates: true,
  noThemes: true,
  noContextFiles: true,
  disabledBuiltinExtensions: ['mcp',],
  additionalExtensionPaths: [fileURLToPath(new URL(
    '../dist/final/node/index.mjs',
    import.meta.url,
  ),),],
},);
await loader.reload();
const sessionManager = SessionManager.inMemory(cwd,);
const {
  session,
  extensionsResult,
} = await createAgentSession({
  cwd,
  agentDir: cwd,
  modelRuntime,
  resourceLoader: loader,
  settingsManager,
  sessionManager,
  noTools: 'all',
},);
assert.equal(
  extensionsResult.errors
    .length,
  0,
);
try {
  await session.bindExtensions({
    onError() { throw new Error('Pi reported an extension error',); },
  },);
  const writer = modelRuntime.getModel(
    providerId,
    'writer',
  );
  assert.ok(
    writer,
    'Pi did not discover the built pool extension',
  );
  assert.ok(
    modelRuntime.getModel(
      providerId,
      'reviewer',
    ),
    'Pi did not discover reviewer',
  );
  await session.setModel(writer,);
  assert.equal(
    session.model
      ?.provider,
    providerId,
  );
  await session.prompt('Stand-in rate-limit check',);
  assert.deepEqual(
    calls,
    [
      head.provider,
      fallback.provider,
    ],
  );
  const answers = sessionManager.getBranch()
    .flatMap(entry =>
      (entry.type === 'message') && (entry.message
        .role
        === 'assistant') ? [entry.message,] : [],);
  assert.equal(
    answers.length,
    1,
  );
  assert.equal(
    answers[0]
      ?.provider,
    fallback.provider,
  );
  assert.equal(
    answers[0]
      ?.model,
    fallback.id,
  );
  assert.equal(
    answers[0]
      ?.stopReason,
    'stop',
  );
  assert.equal(
    session.model
      ?.provider,
    providerId,
  );
  process.stdout
    .write('PASS Pi session: built catalog, registry-authenticated 429 handoff, physical member in assistant record, pool still selected\n',);
} finally {
  session.dispose();
}
