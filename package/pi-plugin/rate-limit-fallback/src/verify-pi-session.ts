/**
 Verify discovery, request-time registry authentication and the actual session record.
 Invoked only by mise.verify-extension.ts, in a disposable offline home.
 @module
 */

import assert from 'node:assert/strict';
import { fileURLToPath, } from 'node:url';
import {
  InMemoryCredentialStore,
  type AssistantMessage,
} from '@earendil-works/pi-ai';
import {
  createAgentSession,
  DefaultResourceLoader,
  ModelRuntime,
  SessionManager,
  SettingsManager,
} from '@earendil-works/pi-coding-agent';
import { WRITER_TARGETS, } from './member-order-fixture.ts';
import { verifyHeldModelArm, } from './verify-held-model.ts';
import {
  labelOfMember,
  REAL_LIMITS,
  registerWriterProvider,
} from './verify-stand-in-members.ts';
import {
  compactionCount,
  sendLongTurns,
} from './verify-session-turns.ts';

/**
 Destinations every request in this process asked for, which must stay empty.
 */
const fetchAttempts: string[] = [];

/**
 Refuse one outbound request and remember what it asked for.

 The extension reads quota only from an origin its model names, and every stand-in member names a
 local origin, so nothing here may ask a provider anything. Replacing the transport makes an edited
 origin fail this check instead of calling out.

 @param input - destination some code asked for

 @returns never returns: every request is refused

 @throws Error naming the destination, always
 */
function refuseProviderRequest(input: Parameters<typeof globalThis.fetch>[0],): Promise<Response> {
  /**
   Destination as a URL string, for the failure message and the record.
   */
  const url = (typeof input) === 'string'
    ? input
    : input instanceof URL ? input.href : input.url;
  fetchAttempts.push(url,);
  throw new Error(`Verification must not reach a provider endpoint: ${url}`,);
}

globalThis.fetch = refuseProviderRequest;

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
 Physical requests, including discarded attempts, captured by the fixture, as `provider/id`.
 */
const calls: string[] = [];

/**
 Members the first arm registered, which its streams answer for.
 */
const members = await registerWriterProvider({
  modelRuntime,
  calls,
  limits: REAL_LIMITS,
},);

/**
 Long turns the window check sends; five of about 34 thousand estimated tokens stay below the
 advertised window.
 */
const TRANSCRIPT_TURNS = 5;

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
  // Pi marks a keyless provider configured in a background availability pass; the command line
  // resolves availability before its first request, so this arm waits for the same state
  // (`@earendil-works/pi-coding-agent` dist/core/model-runtime.js, `registerNativeProvider`).
  await modelRuntime.refresh({ allowNetwork: false, },);
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
      labelOfMember(members.refusing,),
      labelOfMember(members.answering,),
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
  /**
   Member this arm registered as the answering one.
   */
  const { answering, } = members;
  assert.equal(
    answers[0]
      ?.provider,
    answering.provider,
  );
  assert.equal(
    answers[0]
      ?.model,
    answering.id,
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
  assert.deepEqual(
    fetchAttempts,
    [],
    'Verification asked a provider endpoint instead of staying on the stand-in origin',
  );
  // Pi compacts on a threshold taken from the selected model's window, so the pool must advertise
  // the smallest registered member's limits rather than zero. The writer pool also asks a real
  // built-in member, so the expectation is computed from the same registry the pool reads.
  /**
   Windows every registered writer member reports, as the registry resolves them.
   */
  const memberWindows: number[] = [];
  /**
   Output limits every registered writer member reports, as the registry resolves them.
   */
  const memberOutputLimits: number[] = [];
  for (const target of WRITER_TARGETS) {
    /**
     Catalog record Pi resolves for that member, or undefined when it is not registered.
     */
    const member = modelRuntime.getModel(
      target.provider,
      target.id,
    );
    if (member === undefined)
      continue;
    if (member.contextWindow > 0)
      memberWindows.push(member.contextWindow,);
    if (member.maxTokens > 0)
      memberOutputLimits.push(member.maxTokens,);
  }
  assert.ok(
    memberWindows.length > 0,
    'no writer member is registered, so there is no window to advertise',
  );
  assert.equal(
    writer.contextWindow,
    Math.min(...memberWindows,),
    'the pool must advertise the smallest registered member window',
  );
  assert.equal(
    writer.maxTokens,
    Math.min(...memberOutputLimits,),
    'the pool must advertise the smallest registered member output limit',
  );
  /**
   Reviewer pool entry Pi's registry carries after the bind.
   */
  const reviewerEntry = modelRuntime.getModel(
    providerId,
    'reviewer',
  );
  assert.equal(
    reviewerEntry?.contextWindow,
    Math.min(...memberWindows,),
    'the reviewer pool must advertise the smallest registered member window',
  );
  /**
   Member calls before the long transcript, so the loop can prove the pool kept answering.
   */
  const callsBeforeTranscript = calls.length;
  await sendLongTurns({
    session,
    turns: TRANSCRIPT_TURNS,
  },);
  assert.equal(
    compactionCount(sessionManager,),
    0,
    'Pi compacted a pool transcript whose advertised window fits it',
  );
  assert.ok(
    calls.length > callsBeforeTranscript,
    'the pool answered none of the long transcript turns',
  );
  await verifyHeldModelArm({
    cwd,
    modelRuntime,
    fetchAttempts,
  },);
  process.stdout
    .write('PASS Pi session: built catalog, registry-authenticated 429 handoff, physical member in assistant record, pool still selected, no threshold compaction on a long transcript, held model took the bound window\n',);
}
