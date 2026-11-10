/**
 Real AgentSession interruption and ordinary-tool regression.
 
 @module
 */

import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  type AgentToolResult,
  createAgentSession,
  DefaultResourceLoader,
  type ExtensionAPI,
  type ExtensionFactory,
  ModelRuntime,
  SessionManager,
  SettingsManager,
} from '@earendil-works/pi-coding-agent';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { Type, } from 'typebox';

import { registerInterruptionProvider, } from './pi-runtime-verifier-provider.ts';
import {
  goalContinuationMessageCount,
  goalEventCount,
  goalEventKinds,
} from './pi-runtime-verifier-session.ts';

//region Expected call counts

/**
 Expected provider calls before post-clear tool round.
 */
const EXPECTED_PRE_CLEAR_PROVIDER_CALLS = 5;

/**
 Expected provider calls through post-clear tools and final abort.
 */
const EXPECTED_PROVIDER_CALLS = 7;

//endregion Expected call counts

//region Echo contracts

/**
 Verification echo details proving custom callback execution.
 */
type VerificationEchoDetails = {
  readonly verified: true;
};

//endregion Echo contracts

//region Echo registration

/**
 Register custom echo tool used by real AgentSession regression.
 
 @param pi - Pi extension registration API
 
 @param observedValues - caller-owned output capture
 
 @mutates pi - pi.registerTool installs verification_echo in disposable runtime
 
 @mutates observedValues - tool callback appends each executed phase value
 
 @example
 ```ts
 registerVerificationEcho({ pi, observedValues: [] });
 ```
 */
function registerVerificationEcho(
  {
    pi,
    observedValues,
  }: {
    readonly pi: ForeignBorrowed<ExtensionAPI>;
    readonly observedValues: string[];
  },
): void {
  pi.registerTool({
    name: 'verification_echo',
    label: 'Verification Echo',
    description: 'Return supplied disposable verification value',
    parameters: Type.Object({ value: Type.String(), }),
    // oxlint-disable-next-line typescript/require-await, eslint/require-await -- Pi tool contract is asynchronous while fixture computation is synchronous.
    async execute(
      _callId,
      params: Readonly<{ value: string; }>,
    ): Promise<AgentToolResult<VerificationEchoDetails>> {
      observedValues.push(params.value,);
      return {
        content: [{
          type: 'text',
          text: params.value,
        },],
        details: { verified: true, },
      };
    },
  },);
}

/**
 Create extension factory capturing custom echo executions.
 
 @param observedValues - caller-owned output capture
 
 @returns Pi extension factory
 
 @mutates observedValues - returned factory's tool callback appends phase values
 
 @example
 ```ts
 createVerificationEchoFactory([]);
 ```
 */
function createVerificationEchoFactory(
  observedValues: string[],
): ExtensionFactory {
  return function registerVerificationEchoFactory(pi,): void {
    registerVerificationEcho({
      pi,
      observedValues,
    },);
  };
}

//endregion Echo registration

//region Ordinary tool verification

/**
 Exercise ordinary tools through real AgentSession after aborted and errored goal turns.
 
 @param packageDirectory - repository-owned goal package directory
 
 @param agentDirectory - disposable global Pi directory
 
 @param workspaceDirectory - disposable tool filesystem
 
 @param sessionDirectory - disposable persisted sessions
 
 @returns successful tool names and interruption phases
 
 @throws when real provider, lifecycle, tool execution, or filesystem effects differ
 
 @example
 ```ts
 await verifyOrdinaryToolsAfterAbort({ packageDirectory, agentDirectory, workspaceDirectory, sessionDirectory });
 ```
 */
async function verifyOrdinaryToolsAfterAbort(
  {
    packageDirectory,
    agentDirectory,
    workspaceDirectory,
    sessionDirectory,
  }: {
    readonly packageDirectory: string;
    readonly agentDirectory: string;
    readonly workspaceDirectory: string;
    readonly sessionDirectory: string;
  },
): Promise<readonly string[]> {
  await Promise.all([
    writeFile(
      join(
        workspaceDirectory,
        'read.txt',
      ),
      'readable fixture',
    ),
    writeFile(
      join(
        workspaceDirectory,
        'edit.txt',
      ),
      'before edit',
    ),
  ],);
  /**
   Custom-tool values captured after each interruption.
   */
  const observedEchoes: string[] = [];
  /**
   In-memory settings exclude real global and project configuration.
   */
  const settingsManager = SettingsManager.inMemory({
    compaction: { enabled: false, },
    retry: {
      enabled: false,
      provider: { maxRetries: 0, },
    },
  },);
  /**
   Loader uses built goal artifact plus harmless custom tool.
   */
  const resourceLoader = new DefaultResourceLoader({
    cwd: workspaceDirectory,
    agentDir: agentDirectory,
    settingsManager,
    additionalExtensionPaths: [join(
      packageDirectory,
      'dist/final/node/index.mjs',
    ),],
    extensionFactories: [createVerificationEchoFactory(observedEchoes,),],
  },);
  await resourceLoader.reload();
  /**
   Model registry isolated from real credentials and custom model files.
   */
  const modelRuntime = await ModelRuntime.create({
    authPath: join(
      agentDirectory,
      'auth.json',
    ),
    modelsPath: join(
      agentDirectory,
      'models.json',
    ),
  },);
  /**
   Scripted provider stages used by real AgentSession agent loop.
   */
  const provider = registerInterruptionProvider(modelRuntime,);
  /**
   Real session state persisted only in disposable directory.
   */
  const sessionManager = SessionManager.create(
    workspaceDirectory,
    sessionDirectory,
  );
  /**
   Real AgentSession owns extension lifecycle and wrapped tools.
   */
  const {
    session,
    extensionsResult,
  } = await createAgentSession({
    cwd: workspaceDirectory,
    agentDir: agentDirectory,
    model: provider.model,
    tools: [
      'read',
      'bash',
      'edit',
      'write',
      'verification_echo',
    ],
    resourceLoader,
    modelRuntime,
    settingsManager,
    sessionManager,
  },);
  /**
   Session cleanup owner covering every assertion failure.
   */
  using sessionOwner = {
    [Symbol.dispose](): void {
      session.dispose();
    },
  };
  void sessionOwner;
  if (extensionsResult.errors
    .length
    > 0)
    throw new Error(`AgentSession extension errors: ${JSON.stringify(extensionsResult.errors,)}`,);

  /**
   Command promise whose kickoff enters first scripted provider turn.
   */
  const goalStart = session.prompt('/goal Verify interruption recovery',);
  await provider.firstTurnStarted;
  await session.abort();
  await goalStart;
  if (provider.invocationCount() !== 1)
    throw new Error('aborted goal turn emitted automatic continuation',);
  if (goalEventCount({
    sessionManager,
    kind: 'run_started',
  },) !== 1)
    throw new Error('real AgentSession goal start count differed before replacement',);
  if ((goalEventCount({
    sessionManager,
    kind: 'continuation_issued',
  },) !== 0)
    || (goalContinuationMessageCount(sessionManager,) !== 0))
    throw new Error('aborted goal turn persisted continuation effect',);

  /**
   Replacement kickoff drives post-abort tools and terminal model error.
   */
  const recoveryRun = session.prompt('/goal Replacement interruption recovery',);
  await provider.errorTurnStarted;
  await recoveryRun;
  await session.waitForIdle();
  if (goalEventCount({
    sessionManager,
    kind: 'run_started',
  },) !== 2)
    throw new Error('real AgentSession replacement did not persist both run starts',);
  /**
   Persisted unavailable-review events after settled model error.
   */
  const unavailableReviewEvents = goalEventCount({
    sessionManager,
    kind: 'review_unavailable',
  },);
  if (unavailableReviewEvents !== 1) {
    /**
     Selected-branch goal event kinds for focused failure diagnostics.
     */
    const selectedGoalEventKinds = goalEventKinds(sessionManager.getBranch(),);
    /**
     All-session goal event kinds detecting active-leaf displacement.
     */
    const allGoalEventKinds = goalEventKinds(sessionManager.getEntries(),);
    throw new Error(
      `settled model error unavailable-review count ${unavailableReviewEvents}; selected kinds ${selectedGoalEventKinds.join(', ')}; all kinds ${allGoalEventKinds.join(', ')}`,
    );
  }
  if ((goalEventCount({
    sessionManager,
    kind: 'continuation_issued',
  },) !== 0)
    || (goalContinuationMessageCount(sessionManager,) !== 0))
    throw new Error('review-unavailable error emitted primary continuation',);

  /**
   Ordinary user turn executes every tool after terminal errored goal.
   */
  const postErrorRun = session.prompt('Exercise tools after goal error.',);
  await provider.finalTurnStarted;
  await session.abort();
  await postErrorRun;
  await session.waitForIdle();
  if (provider.invocationCount() !== EXPECTED_PRE_CLEAR_PROVIDER_CALLS)
    throw new Error(`unexpected pre-clear provider calls: ${provider.invocationCount()}`,);
  if (goalContinuationMessageCount(sessionManager,) !== 0)
    throw new Error('terminal goal emitted continuation during post-error tools',);

  await session.prompt('/goal clear',);
  if (goalEventCount({
    sessionManager,
    kind: 'run_cleared',
  },) !== 1)
    throw new Error('real AgentSession goal clear did not persist',);
  /**
   User turn driving ordinary tools after clear.
   */
  const postClearRun = session.prompt('Exercise tools after clear.',);
  await provider.clearFinalTurnStarted;
  await session.abort();
  await postClearRun;
  await session.waitForIdle();
  if (provider.invocationCount() !== EXPECTED_PROVIDER_CALLS)
    throw new Error(`unexpected scripted provider calls: ${provider.invocationCount()}`,);
  if ((goalEventCount({
    sessionManager,
    kind: 'continuation_issued',
  },) !== 0)
    || (goalContinuationMessageCount(sessionManager,) !== 0))
    throw new Error('cleared goal emitted continuation during ordinary tools',);
  /**
   Final edited fixture after post-error tool round.
   */
  const edited = await readFile(
    join(
      workspaceDirectory,
      'edit.txt',
    ),
    'utf8',
  );
  /**
   Final written fixture after post-error tool round.
   */
  const written = await readFile(
    join(
      workspaceDirectory,
      'write.txt',
    ),
    'utf8',
  );
  if ((edited !== 'after clear edit') || (written !== 'after clear write'))
    throw new Error('real AgentSession edit or write tools missed post-clear execution',);
  if (JSON.stringify(observedEchoes,) !== JSON.stringify([
    'abort',
    'error',
    'clear',
  ],))
    throw new Error(`custom tool phases differ: ${JSON.stringify(observedEchoes,)}`,);
  return [
    'read-after-abort-error-clear',
    'bash-after-abort-error-clear',
    'edit-after-abort-error-clear',
    'write-after-abort-error-clear',
    'verification_echo-after-abort-error-clear',
  ];
}

//endregion Ordinary tool verification

export { verifyOrdinaryToolsAfterAbort, };
