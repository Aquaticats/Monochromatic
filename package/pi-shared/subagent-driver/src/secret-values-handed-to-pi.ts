/**
 How a run's secret values reach the extension Pi loads, and how the driver knows the masking is armed before it lets a run work.

 The values travel on a descriptor of Pi's own: the driver writes them into a pipe it opens as {@link SECRET_VALUES_FD} in the
 process it starts, and the extension reads that pipe once while it loads and closes it. Nothing writes them to disk, nothing puts
 them in Pi's environment (a command would inherit it), and nothing puts them on Pi's command line (every process could read it).

 What tells the driver the masking is armed is the extension's own command in Pi's command list: Pi loads its extensions before
 it takes a prompt, so a prompt that comes back with a command list holding the extension's command was taken by a Pi whose
 `tool_result` handler is in place. The command's description holds how many values were armed, so a driver whose extension armed
 fewer than it handed over refuses the run instead of working unmasked.

 @module
 */

import type { SecretValue, } from './secret-value-masking.ts';
import {
  isRecord,
  recordsIn,
  textOf,
} from './read-pi-fields.ts';
import { siblingCommandEntry, } from './sibling-command-entry.ts';

//region Constants

/**
 The descriptor Pi reads the values from while the extension loads. Pi's own standard input, output and error are 0, 1 and 2, so
 this is the first descriptor free for the driver to open.
 */
export const SECRET_VALUES_FD = 3;

/**
 Set in Pi's environment, holding the number of the descriptor the driver opened the values on.

 Pi inherits the descriptor, and so does every process it starts, so the number is what says which descriptor is the handover's:
 a Pi started without it has no handover to read, and its extension refuses to arm rather than reading a descriptor of somebody
 else's. The number is not a value of the secret environment and says nothing about one.
 */
export const SECRET_VALUES_DESCRIPTOR_VARIABLE = 'MONOCHROMATIC_SECRET_VALUE_DESCRIPTOR';

/**
 The name of the command the extension registers, which is what the driver looks for in Pi's command list.
 */
export const MASKING_ARMED_COMMAND = 'secret-value-masking';

/**
 The entry of this package that Pi loads as the extension.
 */
export const MASKING_EXTENSION_ENTRY_NAME = 'secret-value-masking-extension';

//endregion Constants

//region The payload

/**
 The text the driver writes into the pipe, and the extension reads back.

 It holds the name of each value and the value itself. Both live only in these two processes, and each side keeps them for
 masking and for nothing else.
 */
export type SecretValuesHandover = {
  readonly values: readonly SecretValue[];
};

/**
 Says what one value's name and value must be for the other side to read them.

 @param payload - what the driver wrote

 @returns the values, checked one by one

 @throws SecretValuesHandoverError when the text is not a handover of values, which is what a driver and an extension of two
 different versions would write and read

 @example
 ```ts
 const values = valuesOfSecretValuesHandover({ payload: '{"values":[]}', },);
 ```
 */
export function valuesOfSecretValuesHandover({ payload, }: { readonly payload: string },): readonly SecretValue[] {
  /**
   What the payload parsed to.
   */
  const parsed = parsedHandover({ payload, },);
  if ((!isRecord(parsed,)) || (!Array.isArray(parsed.values,))) {
    throw new SecretValuesHandoverError('the secret values handed to Pi hold no list of values',);
  }
  return parsed.values
    .map(function checkValue(value,) {
      if ((!isRecord(value,)) || ((typeof value.name) !== 'string')
        || ((typeof value.value) !== 'string')) {
        throw new SecretValuesHandoverError('one of the secret values handed to Pi holds no name or no value',);
      }
      return {
        name: value.name,
        value: value.value,
      };
    },);
}

/**
 Parses the text the driver wrote, which is one JSON record and nothing else.

 @param payload - what the driver wrote

 @returns what it parsed to

 @throws SecretValuesHandoverError when it is not JSON
 */
function parsedHandover({ payload, }: { readonly payload: string },): unknown {
  try {
    return JSON.parse(payload,);
  }
  catch (error) {
    throw new SecretValuesHandoverError(
      `the secret values handed to Pi are not the JSON this extension reads: ${Error.isError(error,) ? error.name : 'a failure'}`,
    );
  }
}

/**
 Reads the descriptor number the driver put in Pi's environment, which is the one descriptor holding the values.

 @param environment - Pi's environment

 @returns the descriptor to read

 @throws SecretValuesHandoverError when the run handed no values over, which is how a Pi whose driver opened no descriptor stops
 instead of reading one that belongs to somebody else

 @example
 ```ts
 const fd = descriptorOfSecretValues({ environment: process.env, },);
 ```
 */
export function descriptorOfSecretValues({
  environment,
}: {
  readonly environment: Readonly<NodeJS.ProcessEnv>;
},): number {
  /**
   What the driver put in the environment, of any shape.
   */
  const named = environment[SECRET_VALUES_DESCRIPTOR_VARIABLE];
  /**
   The number the text names.
   */
  const descriptor = Number(named,);
  if ((named === undefined) || (!Number.isInteger(descriptor,))
    || (descriptor < SECRET_VALUES_FD)) {
    throw new SecretValuesHandoverError(
      `this Pi was handed no descriptor of secret values, so its masking cannot be in place: ${SECRET_VALUES_DESCRIPTOR_VARIABLE} names no descriptor`,
    );
  }
  return descriptor;
}

/**
 Builds the text the driver writes into the pipe.

 @param values - the values read from this machine

 @returns the text, which is the only form the values take outside the driver until Pi's extension reads them

 @example
 ```ts
 const payload = secretValuesHandover({ values, },);
 ```
 */
export function secretValuesHandover({ values, }: { readonly values: readonly SecretValue[] },): string {
  return JSON.stringify({ values, } satisfies SecretValuesHandover,);
}

/**
 A handover this extension cannot read.
 */
export class SecretValuesHandoverError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'SecretValuesHandoverError';
}

//endregion The payload

//region Saying which values are armed

/**
 The description the extension registers its command with, which says how many values are masked.

 @param count - how many values the extension armed with

 @returns the description, which the driver compares with what it handed over

 @example
 ```ts
 const description = maskingArmedDescription({ count: 41, },);
 ```
 */
export function maskingArmedDescription({ count, }: { readonly count: number },): string {
  return `masked ${String(count,)} secret value(s) of this machine in every tool result`;
}

/**
 Says whether the Pi of a run reports the masking armed with every value the driver handed it.

 @param commands - the command list Pi answered with, of any shape

 @param count - how many values the driver handed over

 @returns whether the extension is loaded and armed with all of them

 @example
 ```ts
 const armed = maskingIsArmedIn({ commands, count: 41, },);
 ```
 */
export function maskingIsArmedIn({
  commands,
  count,
}: {
  readonly commands: unknown;
  readonly count: number;
},): boolean {
  /**
   What the extension says of itself.
   */
  const armed = maskingArmedDescription({ count, },);
  return recordsIn(commands,)
    .some(function isTheMaskingCommand(command,) {
      return (textOf(command.name,) === MASKING_ARMED_COMMAND) && (textOf(command.description,) === armed);
    },);
}

//endregion Saying which values are armed

//region Pi's command line

/**
 Builds the words that load the masking extension in the Pi of one run.

 @param folder - the folder the driver's own entry is in, which the extension's entry sits beside

 @returns the words Pi is started with

 @example
 ```ts
 const words = await maskingExtensionArguments({ folder: import.meta.dirname, },);
 ```
 */
export async function maskingExtensionArguments({ folder, }: { readonly folder: string },): Promise<readonly string[]> {
  return [
    '--extension',
    await siblingCommandEntry({
      folder,
      command: MASKING_EXTENSION_ENTRY_NAME,
    },),
  ];
}

//endregion Pi's command line
