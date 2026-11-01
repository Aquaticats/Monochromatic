/**
 Discovered command, abort, process, clear, and branch runtime scenarios.
 
 @module
 */

import {
  emitGoalEvent,
  getGoalCommand,
  requireCondition,
  requireCount,
  settleGoalRun,
} from './pi-runtime-verifier-access.ts';
import {
  createGoalRuntimeHarness,
  type GoalRuntimeHarness,
} from './pi-runtime-verifier-harness.ts';

/**
 Kickoff messages after start and immediate replacement.
 */
const REPLACEMENT_MESSAGE_COUNT = 2;

/**
 Loaded lifecycle verification result retained for exhaustion scenario.
 */
type GoalFlowResult = {
  readonly harness: GoalRuntimeHarness;
  readonly summary: string;
};

/**
 Check captured primary task content for private protocol absence.
 
 @param content - untrusted custom-message content
 
 @returns whether content is plain task text without protocol terms
 
 @example
 ```ts
 taskContentHasNoProtocol('User objective: test');
 ```
 */
function taskContentHasNoProtocol(content: unknown,): boolean {
  if ((typeof content) !== 'string')
    return false;
  return (!content.includes('goal_id'))
    && (!content.includes('review'))
    && (!content.includes('goal_complete'));
}

/**
 Exercise discovered command, lifecycle, and selected-branch reconstruction.
 
 @param packageDirectory - built package discovered through manifest
 
 @param agentDirectory - disposable Pi global directory
 
 @param sessionDirectory - disposable persisted sessions
 
 @returns runtime harness positioned on active replacement branch
 
 @throws when lifecycle invariant differs
 
 @example
 ```ts
 await verifyDiscoveredGoalFlow({ packageDirectory, agentDirectory, sessionDirectory });
 ```
 */
async function verifyDiscoveredGoalFlow(
  {
    packageDirectory,
    agentDirectory,
    sessionDirectory,
  }: {
    readonly packageDirectory: string;
    readonly agentDirectory: string;
    readonly sessionDirectory: string;
  },
): Promise<GoalFlowResult> {
  /**
   Real package discovery with disposable session.
   */
  const harness = await createGoalRuntimeHarness({
    packageDirectory,
    agentDirectory,
    sessionDirectory,
  },);
  requireCondition({
    condition: harness.extension
      .tools
      .size
      === 0,
    message: 'discovered package exposed primary-model tools',
  },);
  requireCondition({
    condition: !harness.extension
      .handlers
      .has('tool_call',),
    message: 'discovered package registered forbidden tool_call blocker',
  },);
  /**
   Loaded slash command under test.
   */
  const command = getGoalCommand(harness,);
  await command({
    args: 'First disposable objective',
    context: harness.context,
  },);
  /**
   Leaf retaining first start and kickoff.
   */
  const firstLeaf = harness.sessionManager
    .getLeafId();
  if (firstLeaf === null)
    throw new Error('first goal did not persist session leaf',);
  await command({
    args: 'Replacement disposable objective',
    context: harness.context,
  },);
  /**
   Leaf retaining replacement before review.
   */
  const replacementLeaf = harness.sessionManager
    .getLeafId();
  if (replacementLeaf === null)
    throw new Error('replacement goal did not persist session leaf',);
  requireCount({
    actual: harness.messages
      .length,
    expected: REPLACEMENT_MESSAGE_COUNT,
    message: 'start and replacement task kickoffs',
  },);
  requireCondition({
    condition: harness.messages
      .every(function containsNoProtocol(message,) {
        return taskContentHasNoProtocol(message.content,);
      },),
    message: 'task kickoff exposed harness protocol',
  },);

  /**
   Explicit abort leaves active state without review or continuation.
   */
  await settleGoalRun({
    harness,
    stopReason: 'aborted',
  },);
  requireCount({
    actual: harness.messages
      .length,
    expected: REPLACEMENT_MESSAGE_COUNT,
    message: 'abort task messages',
  },);

  /**
   Live process suppresses settlement review.
   */
  await emitGoalEvent({
    harness,
    type: 'tool_result',
    event: {
      type: 'tool_result',
      toolCallId: 'process-start',
      toolName: 'process',
      input: { action: 'start', },
      content: [],
      isError: false,
      details: {
        action: 'start',
        success: true,
        process: {
          id: 'proc_runtime',
          status: 'running',
        },
      },
    },
  },);
  await settleGoalRun({
    harness,
    stopReason: 'stop',
  },);
  requireCount({
    actual: harness.messages
      .length,
    expected: REPLACEMENT_MESSAGE_COUNT,
    message: 'live process settlement suppression',
  },);
  await emitGoalEvent({
    harness,
    type: 'tool_result',
    event: {
      type: 'tool_result',
      toolCallId: 'process-list',
      toolName: 'process',
      input: { action: 'list', },
      content: [],
      isError: false,
      details: {
        action: 'list',
        success: true,
        processes: [],
      },
    },
  },);

  /**
   Compaction restores state without triggering task turn.
   */
  await emitGoalEvent({
    harness,
    type: 'session_compact',
    event: {
      type: 'session_compact',
      compactionEntry: {},
      fromExtension: false,
      reason: 'overflow',
      willRetry: true,
    },
  },);
  requireCount({
    actual: harness.messages
      .length,
    expected: REPLACEMENT_MESSAGE_COUNT,
    message: 'compaction task messages',
  },);

  /**
   Clear invalidates delayed settlement review.
   */
  await emitGoalEvent({
    harness,
    type: 'agent_end',
    event: {
      type: 'agent_end',
      messages: [{
        role: 'assistant',
        stopReason: 'stop',
      },],
    },
  },);
  await command({
    args: 'clear',
    context: harness.context,
  },);
  await emitGoalEvent({
    harness,
    type: 'agent_settled',
    event: { type: 'agent_settled', },
  },);
  requireCount({
    actual: harness.messages
      .length,
    expected: REPLACEMENT_MESSAGE_COUNT,
    message: 'clear invalidated settlement',
  },);

  /**
   Start active run retained for exhaustion scenario.
   */
  await command({
    args: 'Reloaded disposable objective',
    context: harness.context,
  },);
  /**
   Task message count before restoration.
   */
  const beforeRestore = harness.messages
    .length;
  await emitGoalEvent({
    harness,
    type: 'session_start',
    event: {
      type: 'session_start',
      reason: 'resume',
    },
  },);
  requireCount({
    actual: harness.messages
      .length,
    expected: beforeRestore,
    message: 'session restoration task messages',
  },);
  harness.sessionManager
    .branch(firstLeaf,);
  await emitGoalEvent({
    harness,
    type: 'session_tree',
    event: { type: 'session_tree', },
  },);
  requireCondition({
    condition: harness.statuses
      .at(-1,)
      ?.includes('First',)
      === true,
    message: 'tree reconstruction used abandoned state',
  },);
  harness.sessionManager
    .branch(replacementLeaf,);
  await emitGoalEvent({
    harness,
    type: 'session_tree',
    event: { type: 'session_tree', },
  },);
  return {
    harness,
    summary: 'manifest discovery, zero tools, task-only context, process gating, abort, compaction, clear, and branch reconstruction',
  };
}

export { verifyDiscoveredGoalFlow, };
export type { GoalFlowResult, };
