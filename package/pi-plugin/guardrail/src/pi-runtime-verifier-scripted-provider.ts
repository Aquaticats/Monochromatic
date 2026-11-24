/**
 Scripted provider that plays the tool calls one real Pi session makes.
 
 The script comes from Pi's own faux provider, so nothing reaches the network and every run sees the same calls.
 Pi's agent loop, its Bash tool, and the loaded guardrail extension do the rest.
 
 @module
 */

import {
  type Api,
  createFauxCore,
  fauxAssistantMessage,
  fauxToolCall,
  type Model,
} from '@earendil-works/pi-ai';
import type { ModelRuntime, } from '@earendil-works/pi-coding-agent';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Constants

/**
 Provider identity, registered only inside the verification process.
 */
const PROVIDER_ID = 'pi-guardrail-runtime-verifier';

/**
 Model identity the verification session runs on.
 */
const MODEL_ID = 'absolute-git-sequence';

/**
 Closing text once every scripted call has run, which ends the run.
 */
const CLOSING_TEXT = 'scripted sequence finished';

/**
 Context and output limits of the scripted model, enough for three tool calls.
 */
const SCRIPTED_LIMITS = {
  contextWindow: 16_384,
  maxTokens: 4_096,
} as const;

//endregion Constants

//region Types

/**
 One scripted tool call.
 */
type ScriptedToolCall = {
  /**
   Tool the model asks Pi to run.
   */
  readonly toolName: string;
  /**
   Arguments the model passes to that tool, as Pi's own tool-call shape carries them.
   */
  readonly arguments: Parameters<typeof fauxToolCall>[1];
};

/**
 Registered scripted provider.
 */
type ScriptedGitGuardProvider = {
  /**
   Model the verification session runs on.
   */
  readonly model: Model<Api>;
  /**
   How many tool calls the script queued, which is how many tool results a full run leaves.
   */
  readonly callCount: number;
  /**
   How many scripted responses are left, so a run that stopped early says so.
   */
  readonly responsesLeft: () => number;
};

//endregion Types

//region Registration

/**
 Registers the scripted provider and queues one response per Bash command.
 
 @param modelRuntime - model registry of the verification session
 
 @param calls - Bash commands the scripted model asks for, in order
 
 @returns registered model, its command count, and the queue's remaining count
 
 @mutates modelRuntime - registers one provider with one model and reads it back
 
 @example
 ```typescript
 const provider = registerScriptedToolCallProvider({ modelRuntime, calls: [{ toolName: 'bash', arguments: { command: 'git status' } }] });
 ```
 */
function registerScriptedToolCallProvider({
  modelRuntime,
  calls,
}: {
  readonly modelRuntime: ForeignBorrowed<ModelRuntime>;
  readonly calls: readonly ScriptedToolCall[];
},): ScriptedGitGuardProvider {
  /**
   Pi's faux provider, which turns queued responses into ordinary model streams.
   */
  const faux = createFauxCore({
    api: PROVIDER_ID,
    provider: PROVIDER_ID,
  },);

  modelRuntime.registerProvider(
    PROVIDER_ID,
    {
      name: 'Pi guardrail runtime verifier',
      baseUrl: 'https://pi-guardrail-verifier.invalid',
      apiKey: 'disposable-verifier-key',
      api: PROVIDER_ID,
      models: [{
        id: MODEL_ID,
        name: 'Absolute Git sequence',
        reasoning: false,
        input: ['text',],
        cost: {
          input: 0,
          output: 0,
          cacheRead: 0,
          cacheWrite: 0,
        },
        ...SCRIPTED_LIMITS,
      },],
      streamSimple: faux.streamSimple,
    },
  );

  faux.setResponses([
    ...calls.map(function toolCallStep(call,) {
      return fauxAssistantMessage(
        fauxToolCall(
          call.toolName,
          call.arguments,
        ),
        { stopReason: 'toolUse', },
      );
    },),
    fauxAssistantMessage(CLOSING_TEXT,),
  ],);

  /**
   Registered model, selected without credential discovery.
   */
  const model = modelRuntime.getModel(
    PROVIDER_ID,
    MODEL_ID,
  );
  if (model === undefined)
    throw new Error('scripted guardrail model registration failed',);

  return {
    model,
    callCount: calls.length,
    responsesLeft() {
      return faux.getPendingResponseCount();
    },
  };
}

//endregion Registration

export {
  CLOSING_TEXT,
  registerScriptedToolCallProvider,
};
export type {
  ScriptedGitGuardProvider,
  ScriptedToolCall,
};
