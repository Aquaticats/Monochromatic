/**
 Guarded faux-provider fixture for the busy-agent delivery test.

 A disposable Pi host loads this file beside the built extension. It registers a
 scripted model that keeps calling one slow tool, so the agent stays busy across
 several tool batches, and then gives a fixed final reply. It refuses to load
 unless the test runner set its guard variable, so it can never register a fake
 model in a real session.

 @module
 */

import {
  fauxAssistantMessage,
  fauxProvider,
  fauxToolCall,
  type AssistantMessage,
} from '@earendil-works/pi-ai';
import type { ExtensionAPI, } from '@earendil-works/pi-coding-agent';
import { wait, } from '@monochromatic-dev/async-time-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Constants

/**
 Environment variable that must be `1` for this fixture to load.
 */
export const FIXTURE_GUARD_VARIABLE = 'BASH_POKE_BUSY_AGENT_FIXTURE';

/**
 Environment variable naming how many tool batches the scripted model runs.
 */
export const FIXTURE_STEPS_VARIABLE = 'BASH_POKE_BUSY_AGENT_STEPS';

/**
 Environment variable naming how long each scripted tool call takes, in milliseconds.
 */
export const FIXTURE_STEP_MS_VARIABLE = 'BASH_POKE_BUSY_AGENT_STEP_MS';

/**
 Provider name the host is started with.
 */
export const FIXTURE_PROVIDER = 'fixture-busy-agent';

/**
 Model identifier the host is started with.
 */
export const FIXTURE_MODEL = 'busy';

/**
 Name of the slow tool the scripted model calls.
 */
export const FIXTURE_TOOL_NAME = 'wait_step';

/**
 Text of the scripted model's final reply, which ends a run with no tool call.
 */
export const FIXTURE_FINAL_TEXT = 'RUN_COMPLETE';

/**
 Text of every reply queued after the final one.
 */
export const FIXTURE_SPARE_TEXT = 'SPARE_REPLY';

/**
 Replies queued after the final one, so a turn the test did not plan for,
 such as the extra turn a follow-up adds, gets a recognizable reply instead of
 the faux provider's "no more responses" error.
 */
const SPARE_REPLY_COUNT = 3;

/**
 Text each scripted tool call returns.
 */
const TOOL_RESULT_TEXT = 'waited';

//endregion Constants

//region Environment

/**
 Reads a positive integer from the environment the test runner prepared.

 @param name - environment variable to read

 @returns parsed value

 @throws when the variable is absent or not a positive integer

 @example
 ```ts
 readPositiveInteger({ name: FIXTURE_STEPS_VARIABLE, },);
 ```
 */
function readPositiveInteger({ name, }: { readonly name: string; }, ): number {
  /**
   Raw text, absent when the runner did not set it.
   */
  const raw = process.env[name];
  if (raw === undefined)
    throw new Error(`${name} is not set; the busy-agent test runner sets it`, );

  /**
   Parsed value, checked before use.
   */
  const value = Number(raw, );
  if ((!Number.isInteger(value, )) || (value < 1))
    throw new Error(`${name} must be a positive integer, got ${raw}`, );
  return value;
}

//endregion Environment

//region Fixture entry

/**
 Registers the scripted model and its slow tool in a disposable Pi host.

 @param pi - Pi extension API of the disposable host

 @mutates pi - one provider and one tool are registered in the host

 @throws when the guard variable is not set or a step setting is invalid

 @example
 ```ts
 // Loaded only by busy-agent-delivery.unit.test.ts through --extension.
 ```
 */
export default function busyAgentHostFixture(pi: ForeignBorrowed<ExtensionAPI>, ): void {
  if (process.env[FIXTURE_GUARD_VARIABLE] !== '1')
    throw new Error(`busy-agent fixture loads only when ${FIXTURE_GUARD_VARIABLE}=1`, );

  /**
   Tool batches before the final reply.
   */
  const steps = readPositiveInteger({ name: FIXTURE_STEPS_VARIABLE, }, );

  /**
   Milliseconds each tool call takes.
   */
  const stepMs = readPositiveInteger({ name: FIXTURE_STEP_MS_VARIABLE, }, );

  /**
   Scripted model with no network transport.
   */
  const model = fauxProvider({
    provider: FIXTURE_PROVIDER,
    api: `${FIXTURE_PROVIDER}-api`,
    models: [{ id: FIXTURE_MODEL, }, ],
  }, );

  /**
   One tool-calling reply per batch.
   */
  const toolReplies: readonly AssistantMessage[] = Array.from(
    { length: steps, },
    function toolReply(
      _unused: unknown,
      index: number,
    ): AssistantMessage {
      return fauxAssistantMessage(fauxToolCall(
        FIXTURE_TOOL_NAME,
        {},
        { id: `${FIXTURE_TOOL_NAME}-${String(index, )}`, },
      ), );
    },
  );

  /**
   Replies for turns the test did not plan for.
   */
  const spareReplies: readonly AssistantMessage[] = Array.from(
    { length: SPARE_REPLY_COUNT, },
    function spareReply(): AssistantMessage {
      return fauxAssistantMessage(FIXTURE_SPARE_TEXT, );
    },
  );

  model.setResponses([
    ...toolReplies,
    fauxAssistantMessage(FIXTURE_FINAL_TEXT, ),
    ...spareReplies,
  ], );
  pi.registerProvider(model.provider, );
  pi.registerTool({
    name: FIXTURE_TOOL_NAME,
    label: FIXTURE_TOOL_NAME,
    description: 'Waits a fixed time so the scripted agent stays busy.',
    parameters: {
      type: 'object',
      properties: {},
    },
    async execute() {
      await wait(stepMs, );
      return {
        content: [
          {
            type: 'text',
            text: TOOL_RESULT_TEXT,
          },
        ],
        details: {},
      };
    },
  }, );
}

//endregion Fixture entry
