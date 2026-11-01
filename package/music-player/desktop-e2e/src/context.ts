/**
 The context every scenario receives: the configuration, the recorder, the private sink and the scratch library.
 
 @example
 ```ts
 import { startContextSession, } from './context.ts';
 
 await using session = await startContextSession({ context, options, },);
 ```
 
 @packageDocumentation
 */

import type { RunConfig, } from './config.ts';
import type { Recorder, } from './recorder.ts';
import type { NullSink, } from './pipewire.ts';
import {
  type Session,
  startSession,
} from './session.ts';
import type { SessionOptions, } from './session-launch.ts';

/**
 What a scenario needs to run and record checks.
 */
export type RunContext = {
  /**
   Scratch directory of this run.
   */
  readonly base: string;
  readonly config: RunConfig;
  /**
   The scratch library the app scans.
   */
  readonly library: string;
  readonly recorder: Recorder;
  /**
   Process id of the runner, which makes unit names unique.
   */
  readonly runId: number;
  readonly sink: NullSink;
};

/**
 Starts a session inside a run's context.
 
 @param context - the run's context
 
 @param options - window and appearance of the session
 
 @returns the running session
 
 @throws CheckFailureError when the session cannot start
 
 @example
 ```ts
 await using session = await startContextSession({ context, options, },);
 ```
 */
export async function startContextSession(
  {
    context,
    options,
  }: {
    readonly context: RunContext;
    readonly options: SessionOptions;
  },
): Promise<Session> {
  return await startSession({
    options,
    base: context.base,
    config: context.config,
    sinkName: context.sink
      .name,
    runId: context.runId,
  },);
}
