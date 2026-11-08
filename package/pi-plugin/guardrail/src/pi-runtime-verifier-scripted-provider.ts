/**
 Scripted provider that plays the Bash tool calls one real Pi session makes.
 
 The script comes from Pi's own faux provider, so nothing reaches the network and every run sees the same calls.
 Pi's agent loop, its Bash tool, and the loaded guardrail extension do the rest.
 
 @module
 */

import {
  createFauxCore,
  fauxAssistantMessage,
  fauxToolCall,
} from '@earendil-works/pi-ai';
import type {
  Model,
  Api,
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
 Name of the tool every scripted call reaches.
 */
const BASH_TOOL_NAME = 'bash';

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
 One scripted Bash tool call.
 */
type ScriptedBashCall = {
  /**
   Shell command the model asks the Bash tool to run.
   */
  readonly command: string;
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
 
 @returns registered model and the queue's remaining count
 
 @mutates modelRuntime - registers one provider with one model and reads it back
 
 @mutates the returned provider's queue - queues one tool-call response per command plus closing text
 
 @example
 ```typescript
 const provider = registerScriptedGitGuardProvider({ modelRuntime, calls: [{ command: 'git status' }] });
 ```
 */
function registerScriptedGitGuardProvider({
  modelRuntime,
  calls,
}: {
  readonly modelRuntime: ForeignBorrowed<ModelRuntime>;
  readonly calls: readonly ScriptedBashCall[];
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
        fauxToolCall(BASH_TOOL_NAME, { command: call.command, },),
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
    responsesLeft() {
      return faux.getPendingResponseCount();
    },
  };
}

//endregion Registration

export {
  CLOSING_TEXT,
  registerScriptedGitGuardProvider,
};
export type {
  ScriptedBashCall,
  ScriptedGitGuardProvider,
};
