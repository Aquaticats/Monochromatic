/**
 Scripted tool batches for real AgentSession interruption verification.
 
 @module
 */

import type {
  Api,
  JsonObject,
  Model,
} from '@earendil-works/pi-ai';

//region Scripted provider contracts

/**
 Script stage notifications and observations exposed to driver.
 */
type ScriptedProvider = {
  readonly model: Model<Api>;
  readonly firstTurnStarted: Promise<void>;
  readonly errorTurnStarted: Promise<void>;
  readonly finalTurnStarted: Promise<void>;
  readonly clearFinalTurnStarted: Promise<void>;
  readonly invocationCount: () => number;
};

/**
 Tool call specification for one scripted assistant batch.
 */
type ScriptedToolCall = {
  readonly name: string;
  // Pi AI 0.87 types ToolCall.arguments as JsonObject.
  readonly arguments: JsonObject;
};

//endregion Scripted provider contracts

//region Interruption tool calls

/**
 Build scripted ordinary-tool batch for one interruption boundary.
 
 @param phase - interruption whose recovery is under test
 
 @returns ordered built-in and custom tool calls
 
 @example
 ```ts
 interruptionToolCalls('abort');
 ```
 */
function interruptionToolCalls(
  phase: 'abort' | 'clear' | 'error',
): readonly ScriptedToolCall[] {
  /**
   Whether first recovery round follows user abort.
   */
  const afterAbort = phase === 'abort';
  /**
   Whether second recovery round follows model error.
   */
  const afterError = phase === 'error';
  return [
    {
      name: 'read',
      arguments: { path: 'read.txt', },
    },
    {
      name: 'bash',
      arguments: { command: 'pwd', },
    },
    {
      name: 'edit',
      arguments: {
        path: 'edit.txt',
        oldText: afterAbort
          ? 'before edit'
          : afterError
            ? 'after abort edit'
            : 'after error edit',
        newText: afterAbort
          ? 'after abort edit'
          : afterError
            ? 'after error edit'
            : 'after clear edit',
      },
    },
    {
      name: 'write',
      arguments: {
        path: 'write.txt',
        content: afterAbort
          ? 'after abort write'
          : afterError
            ? 'after error write'
            : 'after clear write',
      },
    },
    {
      name: 'verification_echo',
      arguments: { value: phase, },
    },
  ];
}

//endregion Interruption tool calls

export { interruptionToolCalls, };
export type {
  ScriptedProvider,
  ScriptedToolCall,
};
