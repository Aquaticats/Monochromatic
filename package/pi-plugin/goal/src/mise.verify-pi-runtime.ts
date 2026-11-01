/**
 Exercises built goal package through real Pi discovery and runtime boundaries.
 
 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import { verifyDefaultCompletionExhaustion, } from './pi-runtime-verifier-completion.ts';
import { verifyDiscoveredGoalFlow, } from './pi-runtime-verifier-goal-flow.ts';
import { verifyInjectedReviewerOutcomes, } from './pi-runtime-verifier-review.ts';
import { verifyOrdinaryToolsAfterAbort, } from './pi-runtime-verifier-tools.ts';

/**
 Disposable async temporary directory owner.
 */
type AsyncTemporaryDirectory = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Create disposable directory removed through `await using`.
 
 @returns asynchronous temporary directory owner
 
 @example
 ```ts
 await using temporary = await createTemporaryDirectory();
 ```
 */
async function createTemporaryDirectory(): Promise<AsyncTemporaryDirectory> {
  /**
   Unique root containing every verifier-controlled resource.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    'pi-goal-runtime-',
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
 Run all disposable Pi consumer-boundary checks.
 
 @returns verification result text
 
 @example
 ```ts
 console.log(await verifyPiGoalRuntime());
 ```
 */
async function verifyPiGoalRuntime(): Promise<string> {
  /**
   Disposable root removed after every success or failure path.
   */
  await using temporary = await createTemporaryDirectory();
  /**
   Disposable Pi global directory.
   */
  const agentDirectory = join(
    temporary.path,
    'agent',
  );
  /**
   Disposable persisted session directory.
   */
  const sessionDirectory = join(
    temporary.path,
    'sessions',
  );
  /**
   Disposable ordinary-tool workspace.
   */
  const workspaceDirectory = join(
    temporary.path,
    'workspace',
  );
  await Promise.all([
    mkdir(
      agentDirectory,
      { recursive: true, },
    ),
    mkdir(
      sessionDirectory,
      { recursive: true, },
    ),
    mkdir(
      workspaceDirectory,
      { recursive: true, },
    ),
  ],);
  await writeFile(
    join(
      agentDirectory,
      'settings.json',
    ),
    '{}\n',
  );
  /**
   Goal package directory is task working directory.
   */
  const packageDirectory = process.cwd();
  /**
   Real-loader lifecycle flow and active harness for completion.
   */
  const flow = await verifyDiscoveredGoalFlow({
    packageDirectory,
    agentDirectory,
    sessionDirectory,
  },);
  /**
   Default registered noninteractive exhaustion result.
   */
  const exhaustion = await verifyDefaultCompletionExhaustion(flow.harness,);
  /**
   Deterministic injected reviewer denial and approval results.
   */
  const reviews = await verifyInjectedReviewerOutcomes();
  /**
   Real AgentSession ordinary-tool names executed after abort boundary.
   */
  const tools = await verifyOrdinaryToolsAfterAbort({
    packageDirectory,
    agentDirectory,
    workspaceDirectory,
    sessionDirectory: join(
      temporary.path,
      'tool-sessions',
    ),
  },);
  return `pi-goal Pi runtime verified: ${flow.summary}; ${exhaustion}; ${reviews}; tools ${tools.join(', ')}`;
}

console.log(await verifyPiGoalRuntime(),);
