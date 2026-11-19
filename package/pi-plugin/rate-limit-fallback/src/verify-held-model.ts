/**
 Held-model arm: the pool resolved before extensions bind takes the registry's window once it binds.
 @module
 */

import assert from 'node:assert/strict';
import { fileURLToPath, } from 'node:url';
import {
  createAgentSessionFromServices,
  createAgentSessionServices,
  type ModelRuntime,
  SessionManager,
  SettingsManager,
} from '@earendil-works/pi-coding-agent';
import {
  HELD_LIMITS,
  registerWriterProvider,
} from './verify-stand-in-members.ts';
import {
  compactionCount,
  sendLongTurns,
} from './verify-session-turns.ts';

/**
 Provider id the built pool registers under.
 */
const PROVIDER_ID = 'rate-limit-fallback';

/**
 Long turns the held arm sends; two stay below this arm's compaction threshold.
 */
const HELD_TRANSCRIPT_TURNS = 2;

/**
 Verify the window a session holds when Pi resolves the pool before extensions bind.

 `--provider=rate-limit-fallback --model=writer` resolves the model before `session_start`, so the
 session holds the catalog entry the pool carried then. The extension registers the provider again
 inside `session_start`, once the registry is known, and this arm requires the held entry to carry
 the members' smallest window afterwards instead of the provisional one.

 @param cwd - disposable verification home

 @param modelRuntime - real registry the extension reads

 @param fetchAttempts - destinations this process asked for, which must stay empty

 @throws AssertionError when the held model kept the provisional window or the transcript compacted

 @example
 ```ts
 await verifyHeldModelArm({ cwd, modelRuntime, fetchAttempts });
 ```
 */
export async function verifyHeldModelArm({
  cwd,
  modelRuntime,
  fetchAttempts,
}: {
  readonly cwd: string;
  readonly modelRuntime: ModelRuntime;
  readonly fetchAttempts: readonly string[];
},): Promise<void> {
  await registerWriterProvider({
    modelRuntime,
    calls: [],
    limits: HELD_LIMITS,
  },);
  /**
   Settings that make the pool the default model, resolved before extensions bind.
   */
  const settingsManager = SettingsManager.inMemory({
    defaultProvider: PROVIDER_ID,
    defaultModel: 'writer',
    retry: { enabled: false, },
  },);
  /**
   Pi's session services, built before any extension binds.
   */
  const services = await createAgentSessionServices({
    cwd,
    agentDir: cwd,
    modelRuntime,
    settingsManager,
    resourceLoaderOptions: {
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
    },
  },);
  /**
   This arm's own session record, so its compaction entries are read apart from the first arm's.
   */
  const sessionManager = SessionManager.inMemory(cwd,);
  /**
   Session built through the same services, so it holds the pool entry resolved before bind.
   */
  const { session, } = await createAgentSessionFromServices({
    services,
    sessionManager,
    noTools: 'all',
  },);
  /**
   Dispose this arm's session on every exit from the check.
   */
  using cleanup: Disposable = {
    [Symbol.dispose](): void { session.dispose(); },
  };
  assert.equal(
    session.model
      ?.provider,
    PROVIDER_ID,
    'the held arm did not start on the pool',
  );
  assert.equal(
    session.model
      ?.id,
    'writer',
  );
  await session.bindExtensions({
    onError() { throw new Error('Pi reported an extension error',); },
  },);
  /**
   Pool entry Pi's registry carries after this arm's session bound its registry.
   */
  const boundEntry = modelRuntime.getModel(
    PROVIDER_ID,
    'writer',
  );
  assert.equal(
    boundEntry?.contextWindow,
    HELD_LIMITS.contextWindow,
    'the registry must carry the window the held arm members report',
  );
  assert.equal(
    session.model
      ?.contextWindow,
    HELD_LIMITS.contextWindow,
    'the held pool model kept the provisional window after the bind',
  );
  await sendLongTurns({
    session,
    turns: HELD_TRANSCRIPT_TURNS,
  },);
  assert.equal(
    compactionCount(sessionManager,),
    0,
    'Pi compacted the held pool transcript whose window fits it',
  );
  assert.deepEqual(
    fetchAttempts,
    [],
    'the held arm asked a provider endpoint instead of staying on the stand-in origin',
  );
}
