/**
 Verify discovery, request-time registry authentication and the actual session record.
 Invoked only by verify-extension.ts, in a disposable offline home.
 @module
 */

import assert from 'node:assert/strict';
import { fileURLToPath, } from 'node:url';
import {
  InMemoryCredentialStore,
  type AssistantMessage,
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
  type ModelIdentity,
} from './stand-in-pi-fixture.ts';

/**
 Working directory in the disposable verification home.
 */
const cwd = process.cwd();
/**
 In-memory credential storage for stand-in providers only.
 */
const credentials = new InMemoryCredentialStore();
/**
 Real Pi registry runtime with remote catalog refresh disabled.
 */
const modelRuntime = await ModelRuntime.create({
  credentials,
  modelsPath: null,
  refreshOnCreate: false,
  allowModelNetwork: false,
},);
/**
 Physical requests, including discarded attempts, captured by the fixture.
 */
const calls: string[] = [];

/**
 First two writer members, registered as stand-ins before extension loading.
 */
const [head, fallback,] = PHYSICAL_MODELS;
assert.ok(head,);
assert.ok(fallback,);

/**
 Verified catalog records used by the registration helper.
 */
const members = {
  preferred: head,
  backup: fallback,
};

/**
 The pool has no need for the fixture's API-specific request path.

 @throws Error when verification reaches the wrong request path
 */
function streamUnsupported(): never {
  throw new Error('Unexpected API-specific request',);
}

/**
 Register one scripted Pi provider, in the same order as the two writer members.

 @param role - preferred member refuses; backup answers
 */
async function registerStandIn(role: 'preferred' | 'backup',): Promise<void> {
  /**
   The preferred member refuses once per handoff exercise.
   */
  const refused = role === 'preferred';
  /**
   Exact writable catalog record that Pi receives.
   */
  const model = members[role];
  /**
   Provider object registered with the host.
   */
  const provider: Provider = {
    id: model.provider,
    name: model.provider,
    getModels() { return [model,]; },
    auth: {
      apiKey: {
        name: 'Stand-in credential',
        resolve({ credential, }: { readonly credential?: { readonly key?: string; }; }) {
          return Promise.resolve(credential?.key === undefined
            ? undefined
            : {
              auth: { apiKey: credential.key, },
              source: 'test',
            },);
        },
      },
    },
    stream: streamUnsupported,
    streamSimple(
      physical: ModelIdentity,
      _context: unknown,
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

await registerStandIn('preferred',);
await registerStandIn('backup',);

/**
 Pool provider the verifier expects from Pi's extension loader.
 */
const providerId = 'rate-limit-fallback';
/**
 Isolated session settings, with no user configuration or outer retry.
 */
const settingsManager = SettingsManager.inMemory({
  defaultProvider: providerId,
  defaultModel: 'writer',
  retry: { enabled: false, },
},);
/**
 Pi resource loader pointed only at the built extension and disposable home.
 */
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
/**
 Host-owned session record used to check assistant provenance.
 */
const sessionManager = SessionManager.inMemory(cwd,);
/**
 Session and extension diagnostics returned by the real Pi SDK.
 */
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
{
  /**
   Dispose Pi's session and runtime bindings on every exit from the check.
   */
  using cleanup: Disposable = {
    [Symbol.dispose](): void { session.dispose(); },
  };
  await session.bindExtensions({
    onError() { throw new Error('Pi reported an extension error',); },
  },);
  /**
   Newly registered pool selected before the stand-in prompt.
   */
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
  /**
   Assistant entries from the active session branch, excluding non-message entries.
   */
  const answers: AssistantMessage[] = [];
  for (const entry of sessionManager.getBranch()) {
    if ((entry.type === 'message') && (entry.message
      .role
      === 'assistant'))
      answers.push(entry.message,);
  }
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
}
