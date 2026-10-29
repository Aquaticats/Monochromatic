/**
 Tests durable prompt payload validation and cross-client replay.

 @module
 */

import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  ChatTextRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  modelPromptDigest,
  promptPayloadStore,
  PromptPayloadStoreError,
  promptUniqueClient,
} from '@monochromatic-dev/translation-repair-stage';
import { tallyErrorText, } from '../dist/final/node/index.mjs';
import { SEAT_SYNTHETIC_VISION_WITHHELD, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

/**
 Exact request replayed across separate client instances.
 */
const REQUEST: ChatTextRequest = {
  modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
  messages: [{ role: 'user', content: 'Read one cat sentence.', },],
  signal: AbortSignal.timeout(HANG_STOP_MS,),
};

/**
 Digest the refusal cases store their records under.
 */
const PROMPT_DIGEST = modelPromptDigest({ request: REQUEST, },);

/**
 Reply every refusal case starts from before breaking one field.
 */
const SLEPT = 'The cat slept.';

/**
 Format version the store writes, read off a record it wrote, so a fixture
 carries the store's own version rather than a copied number.

 @returns Version of a freshly written record

 @throws Error when the written record carries no numeric version

 @example
 ```ts
 const version = await writtenVersion();
 ```
 */
async function writtenVersion(): Promise<number> {
  await using dir = await scratchDir({ prefix: 'prompt-payload-store-', },);
  await promptPayloadStore({ dir: dir.path, },).write({
    promptDigest: PROMPT_DIGEST,
    reply: { text: SLEPT, },
  },);
  /**
   Record the store wrote, parsed.
   */
  const written: unknown = JSON.parse(await readFile(
    join(
      dir.path,
      `${PROMPT_DIGEST}.json`,
    ),
    'utf8',
  ),);
  if (((typeof written) !== 'object') || (written === null) || (!('version' in written)))
    throw new Error('the store wrote a record without a version',);
  /**
   Version field of the written record.
   */
  const { version, } = written;
  if ((typeof version) !== 'number')
    throw new Error('the store wrote a version that is not a number',);
  return version;
}

/**
 Store's own format version.
 */
const VERSION = await writtenVersion();

/**
 Record text holding one stored reply under the store's own version.

 @param reply - stored reply, one field broken per case

 @returns Record text as the store would find it

 @example
 ```ts
 const record = storedReply({ reply: { text: 7, }, },);
 ```
 */
function storedReply({ reply, }: { readonly reply: unknown; },): string {
  return JSON.stringify({
    version: VERSION,
    reply,
  },);
}

/**
 Message a refused read carries: the record's digest, then what refused.

 @param reason - what the store names as refused

 @returns Whole message expected

 @example
 ```ts
 const message = readRefusal({ reason: 'the record is not a JSON object', },);
 ```
 */
function readRefusal({ reason, }: { readonly reason: string; },): string {
  return `prompt payload read failed for ${PROMPT_DIGEST}: ${reason}`;
}

/**
 What a store call refused with, held so a case can assert its class and the
 text a tally line prints for it.

 @param pending - store call expected to refuse

 @returns Refusal, unchanged

 @throws Error when the call answers instead of refusing

 @example
 ```ts
 const refusal = await refusalOf(store.read({ promptDigest, },),);
 ```
 */
async function refusalOf(pending: Promise<unknown>,): Promise<unknown> {
  try {
    await pending;
  }
  catch (error) {
    return error;
  }
  throw new Error('expected the store to refuse, but it answered',);
}

/**
 One stored record the store must refuse, and what the refusal must name.
 */
type RefusedRecord = {
  /**
   What the case pins.
   */
  readonly name: string;

  /**
   Record text found at the digest's path.
   */
  readonly record: string;

  /**
   What the refusal names as refused.
   */
  readonly reason: string;
};

/**
 Every check a stored record can fail, one record failing each.
 */
const REFUSED_RECORDS: readonly RefusedRecord[] = [
  {
    name: 'text that is not JSON',
    record: '{"version":',
    reason: 'the record is not JSON',
  },
  {
    // A string rather than an array: the record guard lets an array through
    // to be probed for fields (`json-guard.ts` `isJsonRecord`).
    name: 'JSON that is not an object',
    record: JSON.stringify(SLEPT,),
    reason: 'the record is not a JSON object',
  },
  {
    name: 'a format version this build does not read, without quoting the stored one',
    record: JSON.stringify({
      version: VERSION + 1,
      reply: { text: SLEPT, },
    },),
    reason: `the record's version is not ${String(VERSION,)}, the format this build reads`,
  },
  {
    name: 'a reply that is not an object',
    record: storedReply({ reply: SLEPT, },),
    reason: 'the record\'s reply is not a JSON object',
  },
  {
    name: 'reply text that is not a string',
    record: storedReply({ reply: { text: 7, }, },),
    reason: 'the record\'s reply.text is not a string',
  },
  {
    name: 'a refusal that is not a string',
    record: storedReply({ reply: { text: '', refusal: 7, }, },),
    reason: 'the record\'s reply.refusal is not a string',
  },
  {
    name: 'a finish reason that is not a string',
    record: storedReply({ reply: { text: SLEPT, finishReason: 7, }, },),
    reason: 'the record\'s reply.finishReason is not a string',
  },
  {
    name: 'a serving provider that is not a string',
    record: storedReply({ reply: { text: SLEPT, servedBy: 7, }, },),
    reason: 'the record\'s reply.servedBy is not a provider name',
  },
  {
    name: 'a serving provider no provider is called',
    record: storedReply({ reply: { text: SLEPT, servedBy: 'catflap', }, },),
    reason: 'the record\'s reply.servedBy is not a provider name',
  },
  {
    name: 'usage that is not an object',
    record: storedReply({ reply: { text: SLEPT, usage: 7, }, },),
    reason: 'the record\'s reply.usage is not a JSON object',
  },
  {
    name: 'a prompt token count that is not a number',
    record: storedReply({
      reply: {
        text: SLEPT,
        usage: { prompt_tokens: '4', completion_tokens: 5, },
      },
    },),
    reason: 'the record\'s reply.usage.prompt_tokens is not a number',
  },
  {
    name: 'a completion token count that is not a number',
    record: storedReply({
      reply: {
        text: SLEPT,
        usage: { prompt_tokens: 4, completion_tokens: '5', },
      },
    },),
    reason: 'the record\'s reply.usage.completion_tokens is not a number',
  },
];

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: promptPayloadStore.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REPLAYS COMPLETED PAYLOAD across client instances without provider call',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'prompt-payload-store-', },);
            const store = promptPayloadStore({ dir: dir.path, },);
            /** Provider calls across both client instances. */
            let providerCalls = 0;
            const firstInner: ModelClient = {
              chatText: async () => {
                providerCalls += 1;
                return {
                  text: 'The cat slept.',
                  finishReason: 'stop',
                  usage: {
                    prompt_tokens: 4,
                    completion_tokens: 5,
                  },
                };
              },
              chatJson: async () => {
                throw new Error('chatJson bypassed by prompt payload reader',);
              },
              quotas: async () => {
                throw new Error('quotas unused by prompt payload fixture',);
              },
            };
            const first = promptUniqueClient({
              inner: firstInner,
              store,
            },);
            const initial = await first.chatText(REQUEST,);

            const resumedInner: ModelClient = {
              chatText: async () => {
                providerCalls += 1;
                throw new Error('resumed client must not call provider',);
              },
              chatJson: async () => {
                throw new Error('chatJson bypassed by prompt payload reader',);
              },
              quotas: async () => {
                throw new Error('quotas unused by prompt payload fixture',);
              },
            };
            const resumed = promptUniqueClient({
              inner: resumedInner,
              store,
            },);
            expect(await resumed.chatText(REQUEST,),).toEqual(initial,);
            expect(providerCalls,).toBe(1,);
          },
        },),

        it({
          name: 'REPLAYS WHICH PROVIDER SERVED A PAYLOAD, since a resumed run that lost it could not re-ask '
            + 'elsewhere and would answer differently from the run it resumes (ledger P9)',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'prompt-payload-store-', },);
            const store = promptPayloadStore({ dir: dir.path, },);
            const first = promptUniqueClient({
              inner: {
                chatText: async () => (
                  {
                    text: 'The cat slept.',
                    servedBy: 'hyper',
                  }
                ),
                chatJson: async () => {
                  throw new Error('chatJson bypassed by prompt payload reader',);
                },
                quotas: async () => {
                  throw new Error('quotas unused by prompt payload fixture',);
                },
              },
              store,
            },);
            await first.chatText(REQUEST,);

            const resumed = promptUniqueClient({
              inner: {
                chatText: async () => {
                  throw new Error('resumed client must not call provider',);
                },
                chatJson: async () => {
                  throw new Error('chatJson bypassed by prompt payload reader',);
                },
                quotas: async () => {
                  throw new Error('quotas unused by prompt payload fixture',);
                },
              },
              store,
            },);
            expect((await resumed.chatText(REQUEST,)).servedBy,).toBe('hyper',);
          },
        },),

        it({
          name: 'REPLAYS A STORED REFUSAL, where the provider declined in words rather than answered',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'prompt-payload-store-', },);
            const store = promptPayloadStore({ dir: dir.path, },);
            const promptDigest = modelPromptDigest({ request: REQUEST, },);
            const reply = {
              text: '',
              refusal: 'The cat declines to judge.',
              finishReason: 'stop',
            };
            await store.write({ promptDigest, reply, },);
            expect(await store.read({ promptDigest, },),).toStrictEqual(reply,);
          },
        },),

        it({
          name: 'REFUSES CORRUPTED DURABLE PAYLOAD rather than recalling provider',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'prompt-payload-store-', },);
            const promptDigest = modelPromptDigest({ request: REQUEST, },);
            await writeFile(
              join(
                dir.path,
                `${promptDigest}.json`,
              ),
              '{"version":1,"reply":{"text":7}}\n',
            );
            const store = promptPayloadStore({ dir: dir.path, },);
            let providerCalls = 0;
            const inner: ModelClient = {
              chatText: async () => {
                providerCalls += 1;
                return { text: 'must not run', };
              },
              chatJson: async () => {
                throw new Error('chatJson bypassed by prompt payload reader',);
              },
              quotas: async () => {
                throw new Error('quotas unused by prompt payload fixture',);
              },
            };
            const client = promptUniqueClient({ inner, store, },);
            let caught: unknown;
            try {
              await client.chatText(REQUEST,);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(PromptPayloadStoreError,);
            expect(providerCalls,).toBe(0,);
          },
        },),
      ],
    },),

    // EACH REFUSAL NAMES WHAT REFUSED (ledger B69). The error's message is all the
    // tally line prints, so one message for every check left an operator unable
    // to tell a corrupted record from a format change from a full disk, and let a
    // case pass whichever check fired.
    describe({
      name: 'promptPayloadStore refusals, each naming what refused (ledger B69)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        ...REFUSED_RECORDS.map(function toCase({ name, record, reason, },) {
          return it({
            name: `refuses ${name}, naming it`,
            fn: async () => {
              await using dir = await scratchDir({ prefix: 'prompt-payload-store-', },);
              await writeFile(
                join(
                  dir.path,
                  `${PROMPT_DIGEST}.json`,
                ),
                `${record}\n`,
              );
              const refusal = await refusalOf(promptPayloadStore({ dir: dir.path, },).read({ promptDigest: PROMPT_DIGEST, },),);
              expect(refusal,).toBeInstanceOf(PromptPayloadStoreError,);
              // WHAT THE TALLY LINE PRINTS, whole: the reason survives its cap.
              expect(tallyErrorText({ error: refusal, },),).toBe(readRefusal({ reason, },),);
            },
          },);
        },),

        it({
          name: 'names the filesystem code when the record cannot be read',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'prompt-payload-store-', },);
            // A directory where the record belongs fails the read as a record the
            // run may not open would.
            await mkdir(join(
              dir.path,
              `${PROMPT_DIGEST}.json`,
            ),);
            const refusal = await refusalOf(promptPayloadStore({ dir: dir.path, },).read({ promptDigest: PROMPT_DIGEST, },),);
            expect(refusal,).toBeInstanceOf(PromptPayloadStoreError,);
            expect(tallyErrorText({ error: refusal, },),).toBe(readRefusal({ reason: 'the record could not be read (EISDIR)', },),);
          },
        },),

        it({
          name: 'names the filesystem code when the record cannot be written',
          fn: async () => {
            await using dir = await scratchDir({ prefix: 'prompt-payload-store-', },);
            /**
             Store directory path, taken by a file.
             */
            const storeDir = join(
              dir.path,
              'prompt-payloads',
            );
            await writeFile(
              storeDir,
              '',
            );
            const refusal = await refusalOf(promptPayloadStore({ dir: storeDir, },).write({
              promptDigest: PROMPT_DIGEST,
              reply: { text: SLEPT, },
            },),);
            const message = `prompt payload write failed for ${PROMPT_DIGEST}: the record could not be written (EEXIST)`;
            expect(refusal,).toBeInstanceOf(PromptPayloadStoreError,);
            expect(tallyErrorText({ error: refusal, },),).toBe(message,);
          },
        },),
      ],
    },),
  ],
},);
