/**
 The stand-in's part in the masking: it plays Pi's side of the handover, so a test sees what a run's own files hold when a tool
 result printed a value of this machine's secret environment.

 A real Pi loads the masking extension, which reads the descriptor the driver opened and registers a command reporting how many
 values it armed with. The stand-in does the same with the same module, so the driver's handover, the extension's arming and the
 order Pi runs its hooks in are all exercised by one fixture, and a scenario that prints a value shows what the run wrote.

 Nothing here writes a value to a log: the lines a test reads name the variables, and a value reaches only the records the run's
 own files are made of, which is what the tests assert on.

 @module
 */

import { closeSync, } from 'node:fs';
import type {
  ContextWithSystemEvent,
  ExtensionAPI,
  ToolResultEvent,
  ToolResultEventResult,
} from '@earendil-works/pi-coding-agent';
import {
  armSecretValueMasking,
  descriptorOfSecretValues,
  fieldsOf,
  MASKING_ARMED_COMMAND,
  type PiRecord,
  readTheSecretValuesHandedOver,
  SECRET_VALUES_FD,
  type SecretValue,
} from '../dist/final/node/index.mjs';
import {
  answer,
  emit,
} from './stand-in-pi-records-fixture.ts';

/**
 The scenario in which the stand-in plays a Pi whose extension did not load, so the masking is not in place.
 */
export const WITHOUT_MASKING_SCENARIO = 'without-masking-extension';

/**
 The scenario in which a tool result prints a value of this machine's secret environment.
 */
export const PRINTS_A_VALUE_SCENARIO = 'prints-a-secret-value';

/**
 What masking one tool result answered: the result Pi keeps, or that nothing was masked in it.
 */
export type MaskedToolResultOfStandIn =
  | { readonly result: ToolResultEventResult }
    | { readonly unchanged: true };

/**
 What the stand-in's Pi was handed, and what the masking registered with it.
 */
export type StandInMasking = {
  /**
   The values the driver handed over, which are read once and kept for the records a scenario writes.
   */
  readonly values: readonly SecretValue[];

  /**
   The command the extension registered, as Pi's command list answers it.
   */
  readonly commands: readonly PiRecord[];

  /**
   Masks one tool result the way Pi does, through the handler the extension registered. An answer of `unchanged` is what Pi takes
   for a result nothing was masked in.
   */
  readonly maskToolResult: (event: ToolResultEvent,) => Promise<MaskedToolResultOfStandIn>;

  /**
   Masks one transcript the way Pi does, through the handler the extension registered.
   */
  readonly maskTranscript: (event: ContextWithSystemEvent,) => unknown;
};

/**
 A stand-in that armed no masking, which is what a Pi whose extension did not load registers.
 */
const NO_MASKING: StandInMasking = {
  values: [],
  commands: [],
  maskToolResult: function maskNothing(): Promise<MaskedToolResultOfStandIn> {
    return Promise.resolve({ unchanged: true, },);
  },
  maskTranscript: function maskNoTranscript() {
    return undefined;
  },
};

/**
 What this stand-in's Pi armed, kept in a holder so every play reads one place without a binding that changes under it.
 */
const armed: { masking: StandInMasking } = { masking: NO_MASKING, };

/**
 What this stand-in's Pi armed.

 @returns the arming of the masking, or none when the scenario is a Pi whose extension did not load

 @example
 ```ts
 const masking = standInMasking();
 ```
 */
export function standInMasking(): StandInMasking {
  return armed.masking;
}

/**
 Reads what the driver handed over on the descriptor this process was started with, and closes it before any scenario runs.

 @param scenario - the behaviour the test asked for

 @returns the arming of the masking, or none when the scenario is a Pi whose extension did not load

 @example
 ```ts
 const masking = armTheStandInMasking({ scenario: 'normal', },);
 ```
 */
export function armTheStandInMasking({ scenario, }: { readonly scenario: string },): StandInMasking {
  if (scenario === WITHOUT_MASKING_SCENARIO) {
    closeTheDescriptorIfThereIsOne();
    armed.masking = NO_MASKING;
    return armed.masking;
  }
  /**
   The values handed over, which is an empty list when the driver handed over none.
   */
  const values = valuesHandedOver();
  /**
   Every handler the extension registered, by the event it asked for.
   */
  const handlers = new Map<string, (event: never,) => unknown>();
  /**
   Every command the extension registered.
   */
  const commands: PiRecord[] = [];
  /**
   The extension API this stand-in plays for it: it records what the extension asks Pi for and does nothing else.
   */
  /* oxlint-disable typescript/no-unsafe-type-assertion -- the stand-in plays Pi's own extension API, and the two members this
     extension registers are the only ones it could exercise; a fake of every method Pi has would be a second Pi. */
  const api = {
    on: function on(
      event: string,
      handler: (event: never,) => unknown,
    ) {
      handlers.set(
        event,
        handler,
      );
      return function off(): void {
        handlers.delete(event,);
      };
    },
    registerCommand: function registerCommand(
      name: string,
      options: { readonly description?: string },
    ) {
      commands.push({
        name,
        description: options.description,
        source: 'extension',
      },);
    },
  } as unknown as ExtensionAPI;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  armSecretValueMasking({
    pi: api,
    values,
  },);
  armed.masking = {
    values,
    commands,
    /* oxlint-disable typescript/no-unsafe-type-assertion -- this plays Pi's own hook pipeline, whose handler types are Pi's; the
       recorder below is typed for the two events this extension registers and nothing else. */
    maskToolResult: async function maskThroughTheExtension(event,): Promise<MaskedToolResultOfStandIn> {
      /**
       The handler the extension registered for Pi's tool result hook.
       */
      const handler = handlers.get('tool_result',);
      if (handler === undefined) {
        return { unchanged: true, };
      }
      /**
       The patch Pi's own hook answered with, which is what Pi's `tool_result` handler gives back.
       */
      const patch = await handler(event as never,);
      if (patch === undefined) {
        return { unchanged: true, };
      }
      return { result: patch as ToolResultEventResult, };
    },
    maskTranscript: function maskThroughTheExtension(event,): unknown {
      /**
       The handler the extension registered for Pi's transcript hook.
       */
      const handler = handlers.get('context_with_system',);
      return (handler === undefined) ? undefined : handler(event as never,);
    },
    /* oxlint-enable typescript/no-unsafe-type-assertion */
  };
  return armed.masking;
}

/**
 Says whether a command list holds the masking the extension registered, which is what the driver asks before it sends a prompt.

 @param masking - what the stand-in armed

 @returns the command list entry, when the extension registered one

 @example
 ```ts
 const commands = maskingCommandsOf({ masking, },);
 ```
 */
export function maskingCommandsOf({ masking, }: { readonly masking: StandInMasking },): readonly PiRecord[] {
  return masking.commands
    .filter(function isTheMaskingCommand(command,) {
      return command.name === MASKING_ARMED_COMMAND;
    },);
}

/**
 Writes the answer of a `get_commands` command, holding the commands the stand-in's Pi has.

 @param id - command's identifier

 @param masking - what the stand-in armed

 @example
 ```ts
 answerCommands({ id, masking, },);
 ```
 */
export function answerCommands({
  id,
  masking,
}: {
  readonly id: unknown;
  readonly masking: StandInMasking;
},): void {
  answer({
    id,
    data: {
      commands: [
        { name: 'bash', },
        { name: 'edit', },
        ...maskingCommandsOf({ masking, },),
      ],
    },
  },);
}

/**
 Plays a run whose tool result printed a value of this machine's secret environment, in the order Pi runs its hooks: the masking
 first, then the end of the call, then the message the session keeps.

 The text is the value as a command that prints `NAME=value` lines writes it, so a test reads what a real `mise env` leak would
 have left in the run's files.

 @param id - command's identifier

 @example
 ```ts
 await printOneSecretValue({ id, },);
 ```
 */
export async function printOneSecretValue({ id, }: { readonly id: unknown },): Promise<void> {
  /**
   What this stand-in's Pi armed.
   */
  const masking = standInMasking();
  answer({ id, },);
  emit({ type: 'agent_start', },);
  emit({ type: 'turn_start', },);
  emit({
    type: 'tool_execution_start',
    toolCallId: 'call-1',
    toolName: 'bash',
    args: { command: 'mise env', },
  },);
  /**
   The value the tool result prints, which is this machine's own value when the driver handed one over.
   */
  const [value,] = masking.values;
  /**
   What the command printed, which is a line naming the variable and holding its value.
   */
  const printed = (value === undefined) ? 'MISE_EXAMPLE_TOKEN=no value was handed over' : `${value.name}=${value.value}`;
  /**
   What Pi's masking made of the printed result.
   */
  const masked = await masking.maskToolResult({
    type: 'tool_result',
    toolCallId: 'call-1',
    toolName: 'bash',
    input: { command: 'mise env', },
    content: [{
      type: 'text',
      text: printed,
    },],
    isError: false,
    details: undefined,
  } as ToolResultEvent,);
  /**
   The text Pi keeps, which is what its masking returned and the tool's own content when nothing was masked.
   */
  const kept = (('unchanged' in masked) || (masked.result
    .content
    === undefined))
    ? printed
    : textOfContent({ content: masked.result
      .content, },);
  emit({
    type: 'tool_execution_end',
    toolCallId: 'call-1',
    toolName: 'bash',
    isError: false,
    result: {
      content: [{
        type: 'text',
        text: kept,
      },],
    },
  },);
  emit({
    type: 'message_end',
    message: {
      role: 'toolResult',
      toolCallId: 'call-1',
      toolName: 'bash',
      content: [{
        type: 'text',
        text: kept,
      },],
      isError: false,
      timestamp: Date.now(),
    },
  },);
  emit({
    type: 'agent_end',
    messages: [],
  },);
  emit({ type: 'agent_settled', },);
}

/**
 Joins the text parts of a tool result's content.

 @param content - the content Pi's masking returned

 @returns its text
 */
function textOfContent({ content, }: { readonly content: readonly unknown[] },): string {
  return content.map(function textOfPart(part,) {
    /**
     The part's text, when the part is text and not an image.
     */
    const {text} = fieldsOf(part,);
    return ((typeof text) === 'string') ? text : '';
  },)
    .join('',);
}

//region The descriptor

/**
 Reads the descriptor the driver opened, and closes it.

 @returns the values the driver handed over, or none when it opened no descriptor
 */
function valuesHandedOver(): readonly SecretValue[] {
  try {
    return readTheSecretValuesHandedOver({
      fd: descriptorOfSecretValues({ environment: process.env, },),
    },);
  }
  catch (error) {
    // A driver that handed nothing over is a run whose masking cannot be in place, which the stand-in plays rather than dies of.
    void error;
    return [];
  }
}

/**
 Closes the descriptor the driver opened, so a stand-in playing a Pi without the extension does not leave it open.

 This close is on the stand-in's own descriptor and happens before any scenario runs, which is what keeps a command from inheriting
 it.
 */
function closeTheDescriptorIfThereIsOne(): void {
  try {
    /**
     The descriptor the driver named, when it named one.
     */
    const fd = descriptorOfSecretValues({ environment: process.env, },);
    /* oxlint-disable-next-line no-restricted-syntax/no-sync -- the stand-in closes its own descriptor before playing anything. */
    closeSync(fd,);
  }
  catch (error) {
    // Nothing was handed over, which is what a scenario without the extension expects.
    void error;
  }
}

//endregion The descriptor
