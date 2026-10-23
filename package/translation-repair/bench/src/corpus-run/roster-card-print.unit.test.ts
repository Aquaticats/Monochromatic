/**
 Tests for the card fragment the roster card command prints.

 WHAT THE FRAGMENT OWES THE PERSON PASTING IT: the provenance line names the
 provider, the served id, the listing it was read from and the date, the three
 steps that follow name the list of that provider, and a model the listing does
 not carry is a refusal that names the URL. The transport is scripted and the
 date handed in, so nothing here reaches a provider or reads a clock.

 Fixtures are cat-themed invention in the providers' listing shapes. No corpus
 content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { printRosterCard, } from '../../dist/final/node/index.mjs';
import { recordedTransport, } from '@monochromatic-dev/translation-repair-provider-stream/ts/recorded-transport.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

/**
 Invented stand-in key, never a real one.
 */
const KEY = 'whisker-key-7421';

/**
 The instant every case hands the command, whose date the fragment names.
 */
const NOW = new Date('2026-10-06T23:59:59.000Z',);

/**
 Listing of one Hyper-shaped row.
 */
const HYPER_LISTING = JSON.stringify({
  data: [
    {
      id: 'mittens-9',
      context_window: 200_000,
      max_output_tokens: 32_000,
      capabilities: { vision: false, },
    },
  ],
},);

/**
 Prints the fragment over a transport replying once.

 @param provider - provider asked for

 @param servedId - served id asked for

 @param status - status the scripted provider answers

 @param bodyText - body the scripted provider answers

 @param sinon - the case's own sandbox

 @returns Lines printed

 @example
 ```ts
 const lines = await printReplying({ provider: 'hyper', servedId: 'mittens-9', status: 200, bodyText, sinon: ctx.sinon, },);
 ```
 */
async function printReplying(
  {
    provider,
    servedId,
    status,
    bodyText,
    sinon,
  }: {
    readonly provider: string;
    readonly servedId: string;
    readonly status: number;
    readonly bodyText: string;
    readonly sinon: Parameters<typeof divertingConsoleLog>[0]['sinon'];
  },
): Promise<readonly string[]> {
  const { transport, } = recordedTransport({
    replies: [
      {
        status,
        bodyText,
      },
    ],
  },);
  using capture = divertingConsoleLog({ sinon, },);
  await printRosterCard({
    line: lineOf({
      command: 'roster-card',
      typed: [
        provider,
        servedId,
      ],
    },),
    env: {
      TRANSLATION_REPAIR_SYNTHETIC_API_KEY: KEY,
      TRANSLATION_REPAIR_CHARM_HYPER_API_KEY: KEY,
      TRANSLATION_REPAIR_OPENROUTER_API_KEY: KEY,
      TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY: KEY,
      TRANSLATION_REPAIR_NEURALWATT_API_KEY: KEY,
    },
    transport,
    now: NOW,
  },);
  return [...capture.lines,];
}

/**
 The three steps and the closing note every fragment ends with, after the
 name of the list the served id goes on.

 @param list - constant in `roster-id.ts` the served id goes on

 @param servedId - served id asked for

 @returns Lines of the footer

 @example
 ```ts
 const footer = footerFor({ list: 'HYPER_SERVED_IDS', servedId: 'mittens-9', },);
 ```
 */
function footerFor(
  {
    list,
    servedId,
  }: {
    readonly list: string;
    readonly servedId: string;
  },
): readonly string[] {
  return [
    `// 1. Add '${servedId}' to ${list} in roster-id.ts.`,
    '// 2. If this is a new model, add its roster id (the first serving provider\'s',
    '//    spelling: Synthetic, else Hyper, else Bedrock, else OpenRouter) to',
    '//    ROSTER_MODEL_IDS and write its card in model-cards.ts with',
    "//    completionCap: 'pooled-p99' and holds ['judge-unmeasured',",
    "//    'writer-unmeasured', 'reader-unmeasured'] until each is measured.",
    '// 3. Run the package checks; model-cards.unit.test.ts holds the lists to the cards.',
    '// A field the provider listing does not carry is printed as a question to answer by hand.',
  ];
}

await describe({
  name: printRosterCard.name,
  // ONE AT A TIME: its cases divert the process-wide `console.log`.
  concurrency: 1,
  children: [
    it({
      name: 'PRINTS the provenance line, the card fragment and the three steps for a model the listing carries',
      fn: async (ctx) => {
        const lines = await printReplying({
          provider: 'hyper',
          servedId: 'mittens-9',
          status: 200,
          bodyText: HYPER_LISTING,
          sinon: ctx.sinon,
        },);

        expect(lines,).toEqual([
          [
            '// hyper side for mittens-9, read from https://hyper.charm.land/v1/models on 2026-10-06',
            '    hyper: {',
            "      id: 'mittens-9',",
            '      readsImages: false,',
            '      maxOutputLength: 32000,',
            '    },',
            '',
            ...footerFor({
              list: 'HYPER_SERVED_IDS',
              servedId: 'mittens-9',
            },),
          ].join('\n',),
        ],);
      },
    },),
    it({
      name: 'NAMES the list of the provider asked for in the first step',
      fn: async (ctx) => {
        const lines = await printReplying({
          provider: 'bedrock',
          servedId: 'cat.mittens-9',
          status: 200,
          bodyText: JSON.stringify({ data: [{ id: 'cat.mittens-9', },], },),
          sinon: ctx.sinon,
        },);

        expect(lines,).toEqual([
          [
            '// bedrock side for cat.mittens-9, read from https://bedrock-mantle.us-east-1.api.aws/v1/models '
              + 'on 2026-10-06',
            '    bedrock: {',
            "      id: 'cat.mittens-9',",
            '      readsImages: /* not in the listing: send it a picture and measure */,',
            '      maxOutputLength: /* not in the listing: the model card page */,',
            '      contextLength: /* not in the listing: the model card page */,',
            "      route: /* 'openai-v1' | 'v1', measured */,",
            "      streamEnd: /* 'done-sentinel' | 'usage-chunk', measured */,",
            '      promptUsdPerMillion: /* the public pricing page */,',
            '      completionUsdPerMillion: /* the public pricing page */,',
            '    },',
            '',
            ...footerFor({
              list: 'BEDROCK_SERVED_IDS',
              servedId: 'cat.mittens-9',
            },),
          ].join('\n',),
        ],);
      },
    },),
    it({
      name: 'PRINTS THE NEURALWATT SIDE off a listing that states its limits under metadata, with the context window '
        + 'standing in where the listing states no output ceiling',
      fn: async (ctx) => {
        const lines = await printReplying({
          provider: 'neuralwatt',
          servedId: 'cat-nap-flash',
          status: 200,
          bodyText: JSON.stringify({
            data: [
              {
                id: 'cat-nap-flash',
                max_model_len: 1_048_560,
                metadata: {
                  capabilities: { vision: true, },
                  limits: { max_context_length: 1_048_560, max_output_tokens: null, },
                },
              },
            ],
          },),
          sinon: ctx.sinon,
        },);

        expect(lines,).toEqual([
          [
            '// neuralwatt side for cat-nap-flash, read from https://api.neuralwatt.com/v1/models on 2026-10-06',
            '    neuralwatt: {',
            "      id: 'cat-nap-flash',",
            '      readsImages: true,',
            '      maxOutputLength: 1048560,',
            '      contextLength: 1048560,',
            '    },',
            '',
            ...footerFor({
              list: 'NEURALWATT_SERVED_IDS',
              servedId: 'cat-nap-flash',
            },),
          ].join('\n',),
        ],);
      },
    },),
    it({
      name: 'NAMES the listing URL and the served-id list of each provider on the provenance line and the first step',
      fn: async (ctx) => {
        /**
         Provenance line and first step the fragment of each provider carries.
         */
        const named: string[][] = [];
        for (const provider of [
          'synthetic',
          'hyper',
          'openrouter',
          'bedrock',
          'neuralwatt',
        ]) {
          /* oxlint-disable no-await-in-loop -- one provider at a time on purpose: each case diverts the process-wide console.log */
          const [fragment = '',] = await printReplying({
            provider,
            servedId: 'cat/mittens-9',
            status: 200,
            bodyText: JSON.stringify({ data: [{ id: 'cat/mittens-9', },], },),
            sinon: ctx.sinon,
          },);
          /* oxlint-enable no-await-in-loop */
          named.push(fragment.split('\n',)
            .filter(function isNamingLine(line,): boolean {
              return line.startsWith(`// ${provider}`,) || line.startsWith('// 1.',);
            },),);
        }

        expect(named,).toEqual([
          [
            '// synthetic side for cat/mittens-9, read from https://api.synthetic.new/openai/v1/models on 2026-10-06',
            "// 1. Add 'cat/mittens-9' to SYNTHETIC_SERVED_IDS in roster-id.ts.",
          ],
          [
            '// hyper side for cat/mittens-9, read from https://hyper.charm.land/v1/models on 2026-10-06',
            "// 1. Add 'cat/mittens-9' to HYPER_SERVED_IDS in roster-id.ts.",
          ],
          [
            '// openrouter side for cat/mittens-9, read from https://openrouter.ai/api/v1/models on 2026-10-06',
            "// 1. Add 'cat/mittens-9' to OPENROUTER_SERVED_IDS in roster-id.ts.",
          ],
          [
            '// bedrock side for cat/mittens-9, read from https://bedrock-mantle.us-east-1.api.aws/v1/models '
              + 'on 2026-10-06',
            "// 1. Add 'cat/mittens-9' to BEDROCK_SERVED_IDS in roster-id.ts.",
          ],
          [
            '// neuralwatt side for cat/mittens-9, read from https://api.neuralwatt.com/v1/models on 2026-10-06',
            "// 1. Add 'cat/mittens-9' to NEURALWATT_SERVED_IDS in roster-id.ts.",
          ],
        ],);
      },
    },),
    it({
      name: 'REFUSES as stated, naming the listing URL and the id, when the listing carries no row under the id',
      fn: async (ctx) => {
        const refusal = await rejectionOf(async function printUnlisted(): Promise<void> {
          await printReplying({
            provider: 'hyper',
            servedId: 'tabby-0',
            status: 200,
            bodyText: HYPER_LISTING,
            sinon: ctx.sinon,
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          'StatedRefusalError: https://hyper.charm.land/v1/models lists no model under tabby-0',
        );
      },
    },),
    it({
      name: 'REFUSES as stated, before any request, a provider that is none of the four',
      fn: async (ctx) => {
        const refusal = await rejectionOf(async function printUnknownProvider(): Promise<void> {
          await printReplying({
            provider: 'bogus',
            servedId: 'mittens-9',
            status: 200,
            bodyText: HYPER_LISTING,
            sinon: ctx.sinon,
          },);
        },);

        expect(String(refusal,),).toBe(
          'StatedRefusalError: roster-card\'s provider is one of synthetic, hyper, openrouter, bedrock, neuralwatt, '
            + 'deepseek, xiaomi, anthropic, '
            + 'and "bogus" is none of them',
        );
      },
    },),
    it({
      name: 'REFUSES as stated, before any request, the Claude key: its card sides are fixed by the owner\'s lock to '
        + 'two models and its listing needs the x-api-key header, which the bearer reader here does not send',
      fn: async (ctx) => {
        const refusal = await rejectionOf(async function printClaudeCard(): Promise<void> {
          await printReplying({
            provider: 'anthropic',
            servedId: 'claude-sonnet-5-5',
            status: 200,
            bodyText: HYPER_LISTING,
            sinon: ctx.sinon,
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toContain('roster-card does not read the Claude key',);
        expect(String(refusal,),).toContain('claude-card-sides.ts',);
      },
    },),
    it({
      name: 'PRINTS nothing when the provider answers with a failure status',
      fn: async (ctx) => {
        const refusal = await rejectionOf(async function printFailedRequest(): Promise<void> {
          await printReplying({
            provider: 'synthetic',
            servedId: 'mittens-9',
            status: 401,
            bodyText: `bad key ${KEY}`,
            sinon: ctx.sinon,
          },);
        },);

        expect(String(refusal,),).toBe(
          'StatedRefusalError: https://api.synthetic.new/openai/v1/models answered 401; the reason phrase is the '
            + 'provider\'s wording and is dropped',
        );
      },
    },),
  ],
},);
