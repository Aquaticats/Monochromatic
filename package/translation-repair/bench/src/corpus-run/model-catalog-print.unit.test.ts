/**
 Tests for the catalog drift report the model catalog command prints.

 WHAT THE REPORT OWES AN OPERATOR: the key is never rendered, a missing key is
 refused in words that name the fix before any request is made, a failed
 request says only the status, and a listing the provider got wrong is a
 refusal rather than a crash. The transport is scripted, so nothing here
 reaches a provider, and the key is an invented stand-in.

 Fixtures are cat-themed invention in the shape of provider ids. No corpus
 content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { CREDENTIAL_MARKER, } from '@monochromatic-dev/translation-repair-provider-stream';
import {
  CATALOG_MODEL_IDS,
  printModelCatalog,
} from '../../dist/final/node/index.mjs';
import { recordedTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts/recorded-transport.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

/**
 Invented stand-in key, never a real one.
 */
const KEY = 'whisker-key-7421';

/**
 Where the command asks for the listing.
 */
const MODELS_URL = 'https://api.synthetic.new/openai/v1/models';

/**
 Environment variable the command reads its key from.
 */
const KEY_VARIABLE = 'TRANSLATION_REPAIR_SYNTHETIC_API_KEY';

/**
 Every id the catalog lists, which a listing serving them all leaves none
 missing from.
 */
const SERVED_ALL = CATALOG_MODEL_IDS.map(function entryOf(id,): { readonly id: string; } {
  return { id, };
},);

/**
 Prints the report over a transport replying once.

 @param env - environment the command reads

 @param status - status the scripted provider answers

 @param bodyText - body the scripted provider answers

 @param sinon - case's own sandbox

 @returns Lines printed and exchanges the transport saw

 @example
 ```ts
 const { lines, } = await printReplying({ env, status: 200, bodyText: '{"data":[]}', sinon: ctx.sinon, },);
 ```
 */
async function printReplying(
  {
    env,
    status,
    bodyText,
    sinon,
  }: {
    readonly env: Readonly<Record<string, string>>;
    readonly status: number;
    readonly bodyText: string;
    readonly sinon: Parameters<typeof divertingConsoleLog>[0]['sinon'];
  },
): Promise<{
  readonly lines: readonly string[];
  readonly exchanges: ReturnType<typeof recordedTransport>['exchanges'];
}> {
  const {
    transport,
    exchanges,
  } = recordedTransport({
    replies: [
      {
        status,
        bodyText,
      },
    ],
  },);
  using capture = divertingConsoleLog({ sinon, },);
  await printModelCatalog({
    env,
    transport,
  },);
  return {
    lines: [...capture.lines,],
    exchanges,
  };
}

await describe({
  name: printModelCatalog.name,
  // ONE AT A TIME: its cases divert the process-wide `console.log`.
  concurrency: 1,
  children: [
    it({
      name: 'PRINTS the drift of the listing in three sections and asks the models URL with the key as a bearer token',
      fn: async (ctx) => {
        const [departed, ...rest] = CATALOG_MODEL_IDS;
        const [seated,] = rest;
        const seatedModel = (seated ?? '').replace(
          'hf:',
          '',
        );
        const { lines, exchanges, } = await printReplying({
          env: { [KEY_VARIABLE]: KEY, },
          status: 200,
          bodyText: JSON.stringify({
            data: [
              ...rest.map(function entryOf(id,): { readonly id: string; } {
                return { id, };
              },),
              {
                id: 'syn:tabby:text',
                hugging_face_id: 'cat/Tabby-1',
              },
              {
                id: 'syn:second-name:text',
                hugging_face_id: seatedModel,
              },
            ],
          },),
          sinon: ctx.sinon,
        },);

        expect(lines,).toEqual([
          [
            'MISSING from the provider but still in the catalog: 1',
            `  ${String(departed,)}  <- every call on this loses a voice to a 404`,
            'UNLISTED distinct models the provider serves: 1',
            '  syn:tabby:text  (cat/Tabby-1)',
            'ALIASES onto models already seated: 1',
            `  syn:second-name:text -> ${seatedModel}`,
          ].join('\n',),
        ],);
        expect(exchanges.map(function seen(exchange,): readonly string[] {
          return [
            exchange.url,
            exchange.method,
            exchange.headers.authorization ?? '',
          ];
        },),).toEqual([[
          MODELS_URL,
          'GET',
          `Bearer ${KEY}`,
        ],],);
      },
    },),
    it({
      name: 'PRINTS no drift in every section for a listing serving exactly the catalog',
      fn: async (ctx) => {
        const { lines, } = await printReplying({
          env: { [KEY_VARIABLE]: KEY, },
          status: 200,
          bodyText: JSON.stringify({ data: SERVED_ALL, },),
          sinon: ctx.sinon,
        },);

        expect(lines,).toEqual([
          [
            'MISSING from the provider but still in the catalog: 0',
            'UNLISTED distinct models the provider serves: 0',
            'ALIASES onto models already seated: 0',
          ].join('\n',),
        ],);
      },
    },),
    it({
      name: 'REFUSES as stated, naming the variable, when the key is not set, before any request',
      fn: async (ctx) => {
        const { transport, exchanges, } = recordedTransport({
          replies: [
            {
              status: 200,
              bodyText: '{"data":[]}',
            },
          ],
        },);
        using capture = divertingConsoleLog({ sinon: ctx.sinon, },);

        const refusal = await rejectionOf(async function printWithoutKey(): Promise<void> {
          await printModelCatalog({
            env: {},
            transport,
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: ${KEY_VARIABLE} is not set; run under mise so sops injects it`,
        );
        expect(exchanges,).toEqual([],);
        expect(capture.lines,).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES as stated, as for an absent key, when the variable is set to nothing',
      fn: async (ctx) => {
        const { transport, } = recordedTransport({
          replies: [
            {
              status: 200,
              bodyText: '{"data":[]}',
            },
          ],
        },);
        using capture = divertingConsoleLog({ sinon: ctx.sinon, },);

        const refusal = await rejectionOf(async function printWithEmptyKey(): Promise<void> {
          await printModelCatalog({
            env: { [KEY_VARIABLE]: '', },
            transport,
          },);
        },);

        expect(String(refusal,),).toBe(
          `StatedRefusalError: ${KEY_VARIABLE} is not set; run under mise so sops injects it`,
        );
        expect(capture.lines,).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES as stated with the status alone, and prints nothing, when the provider answers 404',
      fn: async (ctx) => {
        const refusal = await rejectionOf(async function printFailedRequest(): Promise<void> {
          await printReplying({
            env: { [KEY_VARIABLE]: KEY, },
            status: 404,
            bodyText: 'the cat is no longer supported',
            sinon: ctx.sinon,
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: ${MODELS_URL} answered 404. The reason phrase is dropped rather than repeated: `
            + 'it is the provider\'s wording, and this message promises to carry only ours.',
        );
      },
    },),
    it({
      name: 'MASKS a key the provider echoes in an id, so the printed report never holds it',
      fn: async (ctx) => {
        const { lines, } = await printReplying({
          env: { [KEY_VARIABLE]: KEY, },
          status: 200,
          bodyText: JSON.stringify({
            data: [
              ...SERVED_ALL,
              {
                id: KEY,
                hugging_face_id: 'cat/Echo-1',
              },
            ],
          },),
          sinon: ctx.sinon,
        },);

        expect(lines,).toEqual([
          [
            'MISSING from the provider but still in the catalog: 0',
            'UNLISTED distinct models the provider serves: 1',
            `  ${CREDENTIAL_MARKER}  (cat/Echo-1)`,
            'ALIASES onto models already seated: 0',
          ].join('\n',),
        ],);
      },
    },),
    it({
      name: 'REFUSES as stated, in words that name the shape read, a listing carrying no data array',
      fn: async (ctx) => {
        const refusal = await rejectionOf(async function printMalformedListing(): Promise<void> {
          await printReplying({
            env: { [KEY_VARIABLE]: KEY, },
            status: 200,
            bodyText: '{"data":5}',
            sinon: ctx.sinon,
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: ${MODELS_URL} answered a listing this report cannot read: it reads an object `
            + 'whose data is an array of entries that each carry a string id',
        );
      },
    },),
  ],
},);
