import {
  mkdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import type { ChatTextReply, } from '@monochromatic-dev/translation-repair-provider/ts';
import { failureName, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  isJsonRecord,
  requireJsonSyntaxRefusal,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { isMissingPathError, } from '@monochromatic-dev/translation-repair-typography/ts';
import { isProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { writeFileAtomic, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Durable model-prompt payload store

/**
 On-disk payload format generation.
 */
const PROMPT_PAYLOAD_VERSION = 1;

/**
 Domain absence sentinel for prompt without durable payload.
 */
export const PROMPT_PAYLOAD_MISSING: unique symbol = Symbol('prompt-payload-missing',);

/**
 Raised when durable prompt payload cannot be trusted or written.

 NAMES WHAT REFUSED (ledger B69). The message is all a tally line prints, and
 one message for every check left an operator unable to tell a corrupted
 record from a format change from a full disk.

 WHAT TO DO ABOUT ONE. The record is `<digest>.json` in the run's prompt
 payload directory (`corpus-run/runs-layout.ts`). A read refusal stops the
 entry `INCOMPLETE`, and every later run stops the same way, since the record
 is read before any provider is asked: deleting the record lets the next run
 ask the provider again, at that call's cost, and a record another format
 version wrote reads again under a build of that version. A write refusal
 names the filesystem code; the reply was bought and not kept, so once the
 directory is writable again a rerun asks for it again.

 @example
 ```ts
 throw new PromptPayloadStoreError({
   promptDigest: 'abc',
   operation: 'read',
   reason: 'the record is not JSON',
 },);
 ```
 */
export class PromptPayloadStoreError extends Error {
  /**
   Declares message safe to forward: it carries the record's digest, the
   operation and what refused, named by a JSON path in the record, a
   filesystem code or a class name, and never a stored value.
   */
  readonly messageNamesOnly: true = true;

  /**
   What refused, named without quoting the record.
   */
  readonly reason: string;

  /**
   Constructs privacy-safe durable payload failure.

   @param promptDigest - model-plus-message digest naming record

   @param operation - failed store boundary

   @param reason - what refused, never quoting a stored value

   @param cause - underlying filesystem or parse failure, where there was one

   @example
   ```ts
   new PromptPayloadStoreError({ promptDigest, operation: 'write', reason, cause: error, });
   ```
   */
  public constructor(
    {
      promptDigest,
      operation,
      reason,
      cause,
    }: {
      readonly promptDigest: string;
      readonly operation: 'read' | 'write';
      readonly reason: string;
      readonly cause?: unknown;
    },
  ) {
    super(
      `prompt payload ${operation} failed for ${promptDigest}: ${reason}`,
      ...(cause === undefined ? [] : [{ cause, },]),
    );
    this.name = 'PromptPayloadStoreError';
    this.reason = reason;
  }
}

/**
 Durable raw payload operations used by prompt memoization.
 */
export type PromptPayloadStore = {
  /**
   Reads first completed payload for prompt identity.
   */
  readonly read: (args: { readonly promptDigest: string; },) => Promise<
    ChatTextReply | typeof PROMPT_PAYLOAD_MISSING
    >;

  /**
   Persists first completed payload before exposing it to caller.
   */
  readonly write: (args: {
    readonly promptDigest: string;
    readonly reply: ChatTextReply;
  },) => Promise<void>;
};

/**
 Reads payload text or domain absence sentinel.

 @param path - digest-derived payload path

 @param promptDigest - identity used in diagnostics

 @returns Stored text or missing sentinel

 @example
 ```ts
 const text = await readPayloadText({ path, promptDigest, });
 ```
 */
async function readPayloadText(
  {
    path,
    promptDigest,
  }: {
    readonly path: string;
    readonly promptDigest: string;
  },
): Promise<string | typeof PROMPT_PAYLOAD_MISSING> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if (isMissingPathError({ error, },))
      return PROMPT_PAYLOAD_MISSING;
    throw new PromptPayloadStoreError({
      promptDigest,
      operation: 'read',
      reason: `the record could not be read (${failureName({ error, },)})`,
      cause: error,
    },);
  }
}

/**
 Refuses a record that cannot be trusted, naming what refused.

 @param promptDigest - record identity

 @param reason - what refused, never quoting a stored value

 @throws {@link PromptPayloadStoreError} always

 @example
 ```ts
 invalidStoredPayload({ promptDigest, reason: 'the record is not a JSON object', },);
 ```
 */
function invalidStoredPayload(
  {
    promptDigest,
    reason,
  }: {
    readonly promptDigest: string;
    readonly reason: string;
  },
): never {
  throw new PromptPayloadStoreError({
    promptDigest,
    operation: 'read',
    reason,
  },);
}

/**
 Refuses a record holding a field the store would not have written, naming
 the field by its path in the record rather than quoting its value.

 @param promptDigest - record identity

 @param field - JSON path of field within record

 @param expected - what field must be for record to be trusted

 @throws {@link PromptPayloadStoreError} always

 @example
 ```ts
 invalidStoredField({ promptDigest, field: 'reply.text', expected: 'a string', },);
 ```
 */
function invalidStoredField(
  {
    promptDigest,
    field,
    expected,
  }: {
    readonly promptDigest: string;
    readonly field: string;
    readonly expected: string;
  },
): never {
  invalidStoredPayload({
    promptDigest,
    reason: `the record's ${field} is not ${expected}`,
  },);
}

/**
 Validates stored raw reply without admitting arbitrary disk bytes.

 @param value - parsed stored reply

 @param promptDigest - record identity for refusal

 @returns Trusted raw chat reply

 @throws {@link PromptPayloadStoreError} naming first field reply could not hold

 @example
 ```ts
 const reply = readStoredReply({ value: parsed.reply, promptDigest, },);
 ```
 */
function readStoredReply(
  {
    value,
    promptDigest,
  }: {
    readonly value: unknown;
    readonly promptDigest: string;
  },
): ChatTextReply {
  if (!isJsonRecord(value,)) {
    invalidStoredField({
      promptDigest,
      field: 'reply',
      expected: 'a JSON object',
    },);
  }
  /**
   Stored reply fields before primitive validation.
   */
  const {
    text,
    refusal,
    finishReason,
    usage,
    servedBy,
  } = value;
  if ((typeof text) !== 'string') {
    invalidStoredField({
      promptDigest,
      field: 'reply.text',
      expected: 'a string',
    },);
  }
  if ((refusal !== undefined) && ((typeof refusal) !== 'string')) {
    invalidStoredField({
      promptDigest,
      field: 'reply.refusal',
      expected: 'a string',
    },);
  }
  if ((finishReason !== undefined) && ((typeof finishReason) !== 'string')) {
    invalidStoredField({
      promptDigest,
      field: 'reply.finishReason',
      expected: 'a string',
    },);
  }
  // THE PROVIDER THAT SERVED IT IS REPLAYED TOO (ledger P9): without it a
  // resumed run could not re-ask a reply that could not be used elsewhere,
  // and would answer differently from the run it resumes. Payloads stored
  // before the tag existed carry none.
  if ((servedBy !== undefined) && (((typeof servedBy) !== 'string') || (!isProviderName(servedBy,)))) {
    invalidStoredField({
      promptDigest,
      field: 'reply.servedBy',
      expected: 'a provider name',
    },);
  }
  /**
   Validated reply without optional usage.
   */
  const reply = {
    text,
    ...((refusal === undefined) ? {} : { refusal, }),
    ...((finishReason === undefined) ? {} : { finishReason, }),
    ...((servedBy === undefined) ? {} : { servedBy, }),
  };
  if (usage === undefined)
    return reply;
  if (!isJsonRecord(usage,)) {
    invalidStoredField({
      promptDigest,
      field: 'reply.usage',
      expected: 'a JSON object',
    },);
  }
  /**
   Stored usage counts before numeric validation.
   */
  const {
    prompt_tokens: promptTokens,
    completion_tokens: completionTokens,
  } = usage;
  if ((typeof promptTokens) !== 'number') {
    invalidStoredField({
      promptDigest,
      field: 'reply.usage.prompt_tokens',
      expected: 'a number',
    },);
  }
  if ((typeof completionTokens) !== 'number') {
    invalidStoredField({
      promptDigest,
      field: 'reply.usage.completion_tokens',
      expected: 'a number',
    },);
  }
  return {
    ...reply,
    usage: {
      prompt_tokens: promptTokens,
      completion_tokens: completionTokens,
    },
  };
}

/**
 Parses a record's text, refusing text that is not JSON.

 @param text - record text, never quoted back

 @param promptDigest - record identity for refusal

 @returns Parsed record, of unknown shape for envelope checks

 @throws {@link PromptPayloadStoreError} where text is not JSON

 @example
 ```ts
 const parsed = parsedRecord({ text, promptDigest, },);
 ```
 */
function parsedRecord(
  {
    text,
    promptDigest,
  }: {
    readonly text: string;
    readonly promptDigest: string;
  },
): unknown {
  try {
    /**
     Record as parsed, before any shape is trusted.
     */
    const parsed: unknown = JSON.parse(text,);
    return parsed;
  }
  catch (error) {
    throw new PromptPayloadStoreError({
      promptDigest,
      operation: 'read',
      reason: 'the record is not JSON',
      cause: requireJsonSyntaxRefusal({ error, },),
    },);
  }
}

/**
 Opens privacy-sensitive prompt payload store beneath disposable run root.

 @param dir - directory dedicated to prompt payload records

 @returns Durable store keyed by canonical prompt digest

 @example
 ```ts
 const store = promptPayloadStore({ dir: '/tmp/run/prompt-cache', });
 ```
 */
export function promptPayloadStore(
  { dir, }: { readonly dir: string; },
): PromptPayloadStore {
  return {
    read: async function read(
      { promptDigest, },
    ): Promise<ChatTextReply | typeof PROMPT_PAYLOAD_MISSING> {
      /**
       Digest-derived record path.
       */
      const path = join(
        dir,
        `${promptDigest}.json`,
      );
      /**
       Stored JSON text or explicit absence.
       */
      const text = await readPayloadText({
        path,
        promptDigest,
      },);
      if (text === PROMPT_PAYLOAD_MISSING)
        return text;
      /**
       Parsed durable payload envelope.
       */
      const parsed = parsedRecord({
        text,
        promptDigest,
      },);
      if (!isJsonRecord(parsed,)) {
        invalidStoredPayload({
          promptDigest,
          reason: 'the record is not a JSON object',
        },);
      }
      if (parsed.version !== PROMPT_PAYLOAD_VERSION) {
        invalidStoredField({
          promptDigest,
          field: 'version',
          expected: `${String(PROMPT_PAYLOAD_VERSION,)}, the format this build reads`,
        },);
      }
      return readStoredReply({
        value: parsed.reply,
        promptDigest,
      },);
    },
    write: async function write(
      {
        promptDigest,
        reply,
      },
    ): Promise<void> {
      try {
        await mkdir(
          dir,
          { recursive: true, },
        );
        await writeFileAtomic({
          path: join(
            dir,
            `${promptDigest}.json`,
          ),
          text: `${JSON.stringify({
            version: PROMPT_PAYLOAD_VERSION,
            reply,
          },)}\n`,
        },);
      }
      catch (error) {
        throw new PromptPayloadStoreError({
          promptDigest,
          operation: 'write',
          reason: `the record could not be written (${failureName({ error, },)})`,
          cause: error,
        },);
      }
    },
  };
}

//endregion Durable model-prompt payload store
