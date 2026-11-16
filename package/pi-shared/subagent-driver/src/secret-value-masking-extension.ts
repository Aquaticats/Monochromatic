/**
 The extension a run's Pi loads before it takes a prompt: it masks every value of this machine's secret environment in each tool
 result, and again in the whole transcript on its way to the model.

 Three hooks, and why each:

 `tool_result` fires after a tool has run and before Pi emits the end of the call and the tool result message, so a value masked
 there never reaches the model, never reaches Pi's session file, and never reaches the run's feed, which reads the same event
 (`dist/core/agent-session.js:403` in Pi 1.5.2, and `finalizeExecutedToolCall` then `createToolResultMessage` in its bundle).

 `message_end` fires before Pi keeps a message in its session (`dist/core/agent-session.js:994`, then the append at `:814` and the
 write at `dist/core/session-manager.js:813`), so the first prompt, a steer, an inbox note, an assistant message with its tool
 calls, and every other message of a run are masked before the session file holds them. That is also what keeps Pi's compaction
 and branch-summarization requests free of values: both build their own text from the session and run no context hook
 (`dist/core/compaction/compaction.js:494`).

 `context_with_system` fires before every model request with the whole transcript, so a value that a session already held (a run
 resumed from a session of before this extension) is masked on its way out as well.

 What an agent reads in a tool result is a marker that names the variable, and never the value.

 @module
 */

import {
  closeSync,
  readFileSync,
  readSync,
} from 'node:fs';
import type {
  ContextWithSystemEvent,
  ExtensionAPI,
  MessageEndEvent,
  ToolResultEvent,
  ToolResultEventResult,
} from '@earendil-works/pi-coding-agent';
import type {
  ForeignBorrowed,
  ForeignHostCapability,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  createSecretValueMasking,
  type SecretValue,
  type SecretValueMasking,
} from './secret-value-masking.ts';
import {
  maskSpareCopyOfToolOutput,
  spareCopyOfToolOutput,
} from './pi-output-file-of-a-tool-result.ts';
import { fieldsOf, } from './read-pi-fields.ts';
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

//region Withheld in place

/**
 A message that stands where one this extension could not mask would have stood, of the same role, so Pi keeps it instead.

 @param event - Pi's message event

 @param cause - why the message could not be masked

 @returns a message of the same role, holding nothing of the original
 */
function withheldMessage({
  event,
  cause,
}: {
  readonly event: ForeignBorrowed<MessageEndEvent>;
  readonly cause: string;
},): MessageEndEvent['message'] {
  // Pi rejects a `message_end` replacement whose role differs from the message it has, and would then keep the original, so the
  // replacement keeps the role and carries nothing else of it: no content, no tool call, no usage.
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Pi's message types carry a role-specific shape this replacement cannot satisfy field for field.
  return {
    role: event.message
      .role,
    content: `[this message was withheld: the driver could not mask the secret values in it (${cause})]`,
    timestamp: event.message
      .timestamp,
  } as MessageEndEvent['message'];
}

/**
 A message that stands where a transcript this extension could not mask would have stood.

 @param cause - why the transcript could not be masked

 @returns one message holding nothing of the original
 */
function withheldUserMessage({ cause, }: { readonly cause: string },): {
  readonly role: 'user';
  readonly content: string;
  readonly timestamp: number
} {
  return {
    role: 'user' as const,
    content: `[this conversation was withheld: the driver could not mask the secret values in it (${cause})]`,
    timestamp: Date.now(),
  };
}

//endregion Withheld in place

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

 Every field Pi copies into the message is masked as the data it serialises to: the content, the details a tool declared, and the
 structured content a tool declared an output schema for. A field that could not be masked withholds the whole result, because
 Pi reads an omitted `details` as unchanged and would then keep the tool's own.

 @param event - Pi's tool result event

 @param masking - the masking

 @returns the result as data, or that it must be withheld

 @example
 ```ts
 const answer = maskedToolResult({ event, masking, },);
 ```
 */
export function maskedToolResult({
  event,
  masking,
}: {
  readonly event: ForeignBorrowed<ToolResultEvent>;
  readonly masking: SecretValueMasking;
},): { readonly patch: ToolResultEventResult } | {
  readonly withheld: true;
  readonly cause: string
} {
  /**
   The content as data, which is one answer for keys, methods, cycles and every other thing an object can do.
   */
  const content = masking.maskData(event.content,);
  if ('withheld' in content) {
    return {
      withheld: true,
      cause: content.cause,
    };
  }
  /**
   The details as data, when the tool declared any.
   */
  const details = (event.details === undefined) ? undefined : masking.maskData(event.details,);
  if ((details !== undefined) && ('withheld' in details)) {
    return {
      withheld: true,
      cause: details.cause,
    };
  }
  /**
   The structured content as data, when the tool declared any.
   */
  const structured = (event.structuredContent === undefined)
    ? undefined
    : masking.maskData(event.structuredContent,);
  if ((structured !== undefined) && ('withheld' in structured)) {
    return {
      withheld: true,
      cause: structured.cause,
    };
  }
  return {
    patch: {
      // The data is Pin's own content shape, re-parsed from the serialisation it came from; a value whose shape changed was
      // withheld above, so what reaches here is the shape the tool declared.
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Pi's content type is the shape the round trip proved.
      content: content.value as ToolResultEvent['content'],
      // A field the tool never declared is left out, so Pi keeps what it had; a field it declared is returned as its masked data.
      ...((details === undefined) ? {} : { details: (details as { readonly value: unknown }).value, }),
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Pi's output schema value is what the round trip proved.
      ...((structured === undefined) ? {} : { structuredContent: (structured as { readonly value: unknown }).value as never, }),
    },
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

  /**
   The folder Pi writes the files it keeps a tool's whole output in, which a test names for its own disposable folder.
   */
  readonly piOutputFolder?: string;
};

/**
 Arms the masking in one Pi: the two hooks, and the command whose presence tells the driver the masking is in place.

 @param pi - the extension API Pi hands its extensions

 @param values - the values the driver handed over

 @param noteFailure - where a failure of this extension is written

 @param piOutputFolder - the folder Pi writes its own output files in, when a caller names one

 @example
 ```ts
 armSecretValueMasking({ pi, values, },);
 ```
 */
export function armSecretValueMasking({
  pi,
  values,
  noteFailure = writeToStandardError,
  piOutputFolder,
}: ArmingOfSecretValueMasking,): void {
  /**
   The masking of every value handed over.
   */
  const masking = createSecretValueMasking({ values, },);

  /**
   Withholds a tool result this extension could not mask, rather than letting it through unmasked.

   @param event - Pi's tool result event

   @param cause - why the result could not be masked

   @returns the result Pi shows instead of the tool's own
   */
  function withheldResult({
    event,
    cause,
  }: {
    readonly event: ForeignBorrowed<ToolResultEvent>;
    readonly cause: string;
  },): ToolResultEventResult {
    noteFailure(`${MASKING_ARMED_COMMAND}: a tool result was withheld because it could not be masked: ${cause}`,);
    /**
     The path Pi saved the result's whole output in, if it saved one.
     */
    const spareCopy = spareCopyOfToolOutput({
      details: event.details,
      ...(piOutputFolder === undefined ? {} : { piOutputFolder, }),
    },);
    return {
      isError: true,
      content: [{
        type: 'text',
        text: ('absent' in spareCopy)
          ? WITHHELD_RESULT_TEXT
          : `${WITHHELD_RESULT_TEXT} (its whole output is in ${spareCopy.path})`,
      },],
      // Pi reads an omitted `details` as unchanged and would keep the tool's own, so the withheld result says explicitly that it
      // has none.
      details: { withheld: true, },
    };
  }

  pi.on(
    'tool_result',
    async function maskOneToolResult(event,) {
      try {
        /**
         What masking the result found: the patch Pi keeps, or that the result must be withheld.
         */
        const answer = maskedToolResult({
          event,
          masking,
        },);
        if ('withheld' in answer) {
          return withheldResult({
            event,
            cause: answer.cause,
          },);
        }
        /**
         The path Pi saved the result's whole output in, if it saved one.
         */
        const spareCopy = spareCopyOfToolOutput({
          details: event.details,
          ...(piOutputFolder === undefined ? {} : { piOutputFolder, }),
        },);
        if (!('absent' in spareCopy)) {
          /**
           Why the file kept what Pi wrote, when it did.
           */
          const rewritten = await maskSpareCopyOfToolOutput({
            path: spareCopy.path,
            masking,
          },);
          if (rewritten.problem !== undefined) {
            noteFailure(rewritten.problem,);
          }
        }
        return answer.patch;
      }
      catch (error) {
        return withheldResult({
          event,
          cause: Error.isError(error,) ? error.name : 'a failure',
        },);
      }
    },
  );

  pi.on(
    'context_with_system',
    function maskTheTranscript(event: ForeignBorrowed<ContextWithSystemEvent>,) {
      /**
       The transcript as data, which is what the request will serialise.
       */
      const messages = masking.maskData(event.messages,);
      if ('withheld' in messages) {
        noteFailure(`${MASKING_ARMED_COMMAND}: the transcript could not be masked (${messages.cause}), so the request carries none of it`,);
        // Nothing of the transcript is sent: a request with a marker in it holds no value, and Pi fails a request it cannot build
        // rather than sending the original.
        return { messages: [withheldUserMessage({ cause: messages.cause, },),], };
      }
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Pi's message shape is the one the round trip proved.
      return { messages: messages.value as ContextWithSystemEvent['messages'], };
    },
  );

  pi.on(
    'message_end',
    function maskTheMessageBeforePiKeepsIt(event: ForeignBorrowed<MessageEndEvent>,) {
      /**
       The message as data, which is what Pi will write to its session.
       */
      const message = masking.maskData(event.message,);
      if ('withheld' in message) {
        noteFailure(`${MASKING_ARMED_COMMAND}: a message could not be masked (${message.cause}), so a message that says so is kept in its place`,);
        return { message: withheldMessage({
          event,
          cause: message.cause,
        },), };
      }
      // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- Pi's message shape is the one the round trip proved.
      return { message: message.value as MessageEndEvent['message'], };
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
