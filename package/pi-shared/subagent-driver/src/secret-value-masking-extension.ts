/**
 The extension a run's Pi loads before it takes a prompt: it masks every value of this machine's secret environment in each tool
 result, and again in the whole transcript on its way to the model.

 Two hooks, and why both:

 `tool_result` fires after a tool has run and before Pi emits the end of the call and the tool result message, so a value masked
 there never reaches the model, never reaches Pi's session file, and never reaches the run's feed, which reads the same event
 (`dist/core/agent-session.js:403` in Pi 1.5.2, and `finalizeExecutedToolCall` then `createToolResultMessage` in its bundle).

 `context_with_system` fires before every model request with the whole transcript, so a value that a session already held (a run
 resumed from a session of before this extension, or a value a command wrote into a file the agent read before the sources knew
 it) is masked on its way out as well. Pi's compaction and branch summarization build their own request from the session and do
 not run this hook (`dist/core/compaction/compaction.js:524`), which is named as a limit of this masking.

 What an agent reads in a tool result is a marker that names the variable, and never the value.

 @module
 */

import {
  closeSync,
  readFileSync,
  readSync,
} from 'node:fs';
import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { basename, } from 'node:path';
import type {
  ContextWithSystemEvent,
  ExtensionAPI,
  ToolResultEvent,
  ToolResultEventResult,
} from '@earendil-works/pi-coding-agent';
import type {
  ForeignBorrowed,
  ForeignHostCapability,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  createSecretValueMasking,
  type MaskedDeeply,
  type SecretValue,
  type SecretValueMasking,
} from './secret-value-masking.ts';
import {
  fieldsOf,
  textOf,
} from './read-pi-fields.ts';
import {
  descriptorOfSecretValues,
  MASKING_ARMED_COMMAND,
  maskingArmedDescription,
  SecretValuesHandoverError,
  valuesOfSecretValuesHandover,
} from './secret-values-handed-to-pi.ts';

//region Constants

/**
 What a tool result says instead of its own text when this extension could not mask it.
 */
const WITHHELD_RESULT_TEXT = 'this tool result is withheld: the driver could not mask the secret values in it, and a result that '
  + 'may hold one is not shown to a model';

/**
 How Pi names a file it keeps a tool's whole output in: `<prefix><random hex><extension>`, under the operating system's
 temporary folder (`dist/utils/output-files.js` in Pi 1.5.2).
 */
const SPARE_COPY_FILE_NAME_PREFIX = 'pi-output-';

/**
 The longest payload the descriptor may hold, which is a few kilobytes of names and values for every source of one machine, and far
 below what a pipe of its own would need before a writer had to wait.
 */
const LONGEST_HANDOVER_BYTES = 1_048_576;

//endregion Constants

//region Reading what the driver handed over

/**
 Reads the payload the driver wrote into the descriptor this process was started with, and closes the descriptor so no command
 started later can read what is left.

 The read is synchronous because it must finish, and the descriptor must be closed, before this process takes a prompt: a command
 Pi starts would inherit the descriptor, and every later reader of `/proc/self/fd` inside a run would find it.

 @param fd - the descriptor the driver opened

 @returns the payload as text

 @throws SecretValuesHandoverError when the descriptor is not there, or holds more than a handover of this size

 @example
 ```ts
 const payload = payloadHandedOver({ fd: 3, },);
 ```
 */
function payloadHandedOver({ fd, }: { readonly fd: number },): string {
  /**
   What reading the descriptor gave.
   */
  const read = readAndCloseTheDescriptor({ fd, },);
  if ('failure' in read) {
    throw new SecretValuesHandoverError(
      `the descriptor the secret values are handed over on could not be read, so this run would work with its masking off: ${Error.isError(read.failure,) ? read.failure
        .name : 'a failure'}`,
    );
  }
  return read.text;
}

/**
 Reads a descriptor's payload and closes it, whatever the read gave.

 @param fd - the descriptor to read and close

 @returns the payload, or what the read threw
 */
function readAndCloseTheDescriptor({ fd, }: { readonly fd: number },): { readonly text: string } | {
  readonly failure: unknown;
} {
  /**
   The payload's bytes, which is a pipe: one read gives whatever has arrived, so the reads go on until the writer's end closes.
   */
  const buffer = Buffer.alloc(LONGEST_HANDOVER_BYTES,);
  /**
   How many bytes have been read so far, which is the place in the buffer the next read writes to. The count is in a holder rather
   than a binding so the drain stays one expression of the loop's bound.
   */
  const read: { filled: number } = { filled: 0, };
  try {
    while (read.filled < buffer.length) {
      /**
       How many bytes this read gave, which is zero once the writer has closed its end.
       */
      /* oxlint-disable-next-line no-restricted-syntax/no-sync -- the descriptor must be drained before a prompt is taken. */
      const bytes = readSync(
        fd,
        buffer,
        read.filled,
        buffer.length - read.filled,
        null,
      );
      if (bytes === 0) {
        break;
      }
      read.filled += bytes;
    }
    closeTheDescriptor({ fd, },);
    return { text: buffer.subarray(
      0,
      read.filled,
    )
      .toString('utf8',), };
  }
  catch (error) {
    closeTheDescriptor({ fd, },);
    return { failure: error, };
  }
}

/**
 Closes the descriptor the values came on, saying nothing when it is already gone.

 @param fd - the descriptor to close
 */
function closeTheDescriptor({ fd, }: { readonly fd: number },): void {
  try {
    /* oxlint-disable-next-line no-restricted-syntax/no-sync -- the descriptor must be closed before a command can inherit it. */
    closeSync(fd,);
  }
  catch (error) {
    // A descriptor already gone is exactly what closing it wanted, and nothing about it is worth a run's standard error.
    if (!Error.isError(error,)) {
      throw error;
    }
  }
}

/**
 Reads the values the driver wrote into the descriptor, and closes it.

 @param fd - the descriptor the driver opened

 @returns the values, which are kept in this process for masking and nowhere else

 @throws SecretValuesHandoverError when the descriptor is not there or does not hold a handover, which is how a run started
 without a driver that read the values refuses to work

 @example
 ```ts
 const values = readTheSecretValuesHandedOver({ fd: 3, },);
 ```
 */
export function readTheSecretValuesHandedOver({ fd, }: { readonly fd: number },): readonly SecretValue[] {
  return valuesOfSecretValuesHandover({ payload: payloadHandedOver({ fd, },), },);
}

//endregion Reading what the driver handed over

//region Masking a tool result

/**
 Reads the path of the file Pi keeps a tool's whole output in, which it writes when it shows only part of that output.

 @param details - the tool result's own details, of any shape

 @returns the path, or nothing when the tool kept no such file
 */
function spareCopyOfToolOutput(details: unknown,): { readonly path: string } | { readonly absent: true } {
  /**
   The field Pi puts the path in.
   */
  const path = fieldsOf(details,)
    .fullOutputPath;
  if ((typeof path) !== 'string') {
    return { absent: true, };
  }
  return isSpareCopyOfToolOutput(path,)
    ? { path, }
    : { absent: true, };
}

/**
 Says whether a path is one of the files Pi keeps tool output in, which is the only kind of file this extension rewrites.

 @param path - the path the result's details name

 @returns whether Pi made that file for the output of a tool call
 */
function isSpareCopyOfToolOutput(path: string,): boolean {
  return basename(path,)
    .startsWith(SPARE_COPY_FILE_NAME_PREFIX,);
}

/**
 Masks the file Pi saved a truncated tool output in, so a value masked in what the model reads is not left in what Pi wrote.

 Pi writes those files with mode 0600, and a path that is not one of them is left alone: this code rewrites only a file Pi made
 for the result it is masking.

 @param path - the file Pi saved the output in

 @param masking - the masking
 */
async function maskSpareCopyOfToolOutput({
  path,
  masking,
}: {
  readonly path: string;
  readonly masking: SecretValueMasking;
},): Promise<void> {
  /**
   The file's text, or nothing when it has already gone.
   */
  const text = await textOfTheSpareCopy({ path, },);
  if ('absent' in text) {
    return;
  }
  /**
   The same text with every value masked.
   */
  const masked = masking.maskText(text.text,);
  if (masked === text.text) {
    return;
  }
  try {
    await writeFile(
      path,
      masked,
      { mode: 0o600, },
    );
  }
  catch (error) {
    process.stderr
      .write(`the whole output of a masked tool result could not be rewritten: ${Error.isError(error,) ? error.name : 'a failure'}\n`,);
  }
}

/**
 Reads the text of the file Pi saved a tool's whole output in.

 @param path - the file

 @returns its text, or that it is not there to read
 */
async function textOfTheSpareCopy({ path, }: { readonly path: string },): Promise<
  | { readonly text: string }
    | { readonly absent: true }
  > {
  try {
    return { text: await readFile(
      path,
      'utf8',
    ), };
  }
  catch (error) {
    if (!Error.isError(error,)) {
      throw error;
    }
    return { absent: true, };
  }
}

/**
 Masks the text parts of a tool result, keeping images and every other part as they were.

 @param content - the tool result's parts

 @param masking - the masking

 @returns the parts, and whether any of them changed
 */
function maskedToolResultParts({
  content,
  masking,
}: {
  readonly content: ForeignBorrowed<ToolResultEvent['content']>;
  readonly masking: SecretValueMasking;
},): {
  readonly parts: ForeignBorrowed<ToolResultEvent['content']>;
  readonly changed: boolean
} {
  /**
   Each part, with the text of it masked.
   */
  const maskedParts = content.map(function maskOnePart(part,) {
    return maskedToolResultPart({
      part,
      masking,
    },);
  },);
  /**
   Whether any part held a value.
   */
  const changed = maskedParts.some(function didChange(masked,) {
    return masked.changed;
  },);
  return {
    parts: changed ? maskedParts.map(function partOf(masked,) {
      return masked.part;
    },) : content,
    changed,
  };
}

/**
 Masks the text of one part of a tool result, keeping images and every other part as they were.

 @param part - the part

 @param masking - the masking

 @returns the part, and whether its text changed
 */
function maskedToolResultPart({
  part,
  masking,
}: {
  readonly part: ForeignBorrowed<ToolResultEvent['content'][number]>;
  readonly masking: SecretValueMasking;
},): {
  readonly part: ForeignBorrowed<ToolResultEvent['content'][number]>;
  readonly changed: boolean
} {
  if (part.type !== 'text') {
    return {
      part,
      changed: false,
    };
  }
  /**
   The text after masking.
   */
  const masked = masking.maskText(part.text,);
  if (masked === part.text) {
    return {
      part,
      changed: false,
    };
  }
  return {
    part: {
      ...part,
      text: masked,
    },
    changed: true,
  };
}

/**
 Masks one tool result, which is what Pi calls after the tool has run and before the result becomes a message.

 @param event - Pi's tool result event

 @param masking - the masking

 @returns the result with every value masked, or nothing when nothing was masked

 @example
 ```ts
 const patch = maskedToolResult({ event, masking, },);
 ```
 */
export function maskedToolResult({
  event,
  masking,
}: {
  readonly event: ForeignBorrowed<ToolResultEvent>;
  readonly masking: SecretValueMasking;
  // Pi's own hook takes `undefined` as "nothing changed", and a handler that returned an empty patch would drop the structured
  // content a tool declared, so the absence is mirrored from the installed interface rather than modelled here.
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- Pi's `tool_result` handler result is `T | undefined`.
},): ToolResultEventResult | undefined {
  /**
   The content, with the text of each part masked.
   */
  const content = maskedToolResultParts({
    content: event.content,
    masking,
  },);
  /**
   The structured content a tool declared an output schema for, masked field by field, when the tool declared any.
   */
  const structured = (event.structuredContent === undefined)
    ? undefined
    : masking.maskDeeply(event.structuredContent,);
  if ((!content.changed) && (structured?.changed !== true)) {
    return undefined;
  }
  return {
    content: content.parts,
    ...((structured === undefined) || (!structured.changed) ? {} : { structuredContent: structured.value, }),
  };
}

//endregion Masking a tool result

//region The extension

/**
 Writes one line about this extension's own failure to Pi's standard error, which holds no value.

 @param text - the line
 */
function writeToStandardError(text: string,): void {
  process.stderr
    .write(`${text}\n`,);
}

/**
 Answers a run that asks the command what the driver already checked before it let the run work.

 The handler is what Pi requires of a command, and its answer is the description the driver read.
 */
function reportTheMasking(): Promise<void> {
  return Promise.resolve();
}

/**
 What arming the masking needs.
 */
export type ArmingOfSecretValueMasking = {
  /**
   The extension API Pi hands its extensions, which owns what a handler can do with it.
   */
  readonly pi: ForeignHostCapability<ExtensionAPI>;

  /**
   The values the driver handed over.
   */
  readonly values: readonly SecretValue[];

  /**
   Where a failure of this extension is written, which is Pi's standard error and holds no value.
   */
  readonly noteFailure?: (text: string,) => void;
};

/**
 Arms the masking in one Pi: the two hooks, and the command whose presence tells the driver the masking is in place.

 @param pi - the extension API Pi hands its extensions

 @param values - the values the driver handed over

 @param noteFailure - where a failure of this extension is written

 @example
 ```ts
 armSecretValueMasking({ pi, values, },);
 ```
 */
export function armSecretValueMasking({
  pi,
  values,
  noteFailure = writeToStandardError,
}: ArmingOfSecretValueMasking,): void {
  /**
   The masking of every value handed over.
   */
  const masking = createSecretValueMasking({ values, },);

  /**
   Withholds a tool result this extension could not mask, rather than letting it through unmasked.

   @param event - Pi's tool result event

   @param failure - what the masking threw

   @returns the result Pi shows instead of the tool's own
   */
  function withheldResult({
    event,
    failure,
  }: {
    readonly event: ForeignBorrowed<ToolResultEvent>;
    readonly failure: unknown;
  },): ToolResultEventResult {
    noteFailure(`${MASKING_ARMED_COMMAND}: a tool result was withheld because it could not be masked: ${Error.isError(failure,) ? failure.name : 'a failure'}`,);
    /**
     The path Pi saved the result's whole output in, if it saved one.
     */
    const spareCopy = spareCopyOfToolOutput(event.details,);
    return {
      isError: true,
      content: [{
        type: 'text',
        text: ('absent' in spareCopy)
          ? WITHHELD_RESULT_TEXT
          : `${WITHHELD_RESULT_TEXT} (its whole output is in ${spareCopy.path})`,
      },],
    };
  }

  pi.on(
    'tool_result',
    async function maskOneToolResult(event,) {
      try {
        /**
         The result with every value masked.
         */
        const patch = maskedToolResult({
          event,
          masking,
        },);
        /**
         The path Pi saved the result's whole output in, if it saved one.
         */
        const spareCopy = spareCopyOfToolOutput(event.details,);
        if (!('absent' in spareCopy)) {
          await maskSpareCopyOfToolOutput({
            path: spareCopy.path,
            masking,
          },);
        }
        return patch;
      }
      catch (error) {
        return withheldResult({
          event,
          failure: error,
        },);
      }
    },
  );

  pi.on(
    'context_with_system',
    function maskTheTranscript(event: ForeignBorrowed<ContextWithSystemEvent>,) {
      try {
        /**
         The transcript with every string in it masked.
         */
        const messages = masking.maskDeeply(event.messages,);
        return messages.changed
          ? { messages: messages.value, }
          : undefined;
      }
      catch (error) {
        noteFailure(`${MASKING_ARMED_COMMAND}: the transcript could not be masked: ${Error.isError(error,) ? error.name : 'a failure'}`,);
        return undefined;
      }
    },
  );

  pi.registerCommand(
    MASKING_ARMED_COMMAND,
    {
      description: maskingArmedDescription({ count: masking.valueCount, },),
      handler: reportTheMasking,
    },
  );
}

/**
 The extension Pi loads for a run: it reads the values the driver handed over on its own descriptor, and arms the masking.

 @param pi - the extension API Pi hands its extensions

 @throws SecretValuesHandoverError when the run handed it no values, which stops a run whose masking would otherwise be off

 @example
 ```ts
 // Pi loads this file with `--extension` and calls what it exports with itself.
 ```
 */
export default function secretValueMasking(pi: ExtensionAPI,): void {
  armSecretValueMasking({
    pi,
    values: readTheSecretValuesHandedOver({
      fd: descriptorOfSecretValues({ environment: process.env, },),
    },),
  },);
}

//endregion The extension
