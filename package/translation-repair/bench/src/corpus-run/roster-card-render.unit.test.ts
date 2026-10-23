/**
 Guards on the roster-card renderer: each provider's listing shape reads
 into the card's field names, a field the listing lacks prints as a
 question, and a listing without the model says so.

 Fixtures are cat-themed invention in the providers' listing shapes; no
 corpus content and no live provider appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  cardFieldsFrom,
  listingRowFor,
  NOT_LISTED,
  readAsk,
  renderProviderCard,
} from '../../dist/final/node/index.mjs';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';

/**
 A listing body in the `{ data: [...] }` shape, with one OpenRouter-shaped
 row and one Hyper-shaped row.
 */
const LISTING = {
  data: [
    {
      id: 'whiskers/mittens-9',
      context_length: 200_000,
      architecture: {
        input_modalities: [
          'text',
          'image',
        ],
      },
      pricing: {
        prompt: '0.0000003',
        completion: '0.0000012',
      },
      top_provider: { max_completion_tokens: 64_000, },
    },
    {
      id: 'mittens-9',
      context_window: 200_000,
      max_output_tokens: 32_000,
      capabilities: { vision: false, },
    },
    'not a row',
  ],
};

await describe({
  name: 'roster-card-render',
  children: [
    it({
      name: 'FINDS THE ROW BY SERVED ID under `data`, skips non-rows, and answers NOT_LISTED for an unknown id '
        + 'or a body without rows',
      fn: async () => {
        /**
         Row the fixture carries under the OpenRouter spelling.
         */
        const row = listingRowFor({
          body: LISTING,
          servedId: 'whiskers/mittens-9',
        },);
        expect((typeof row) === 'symbol',).toBe(false,);
        expect(listingRowFor({
          body: LISTING,
          servedId: 'tabby-0',
        },),).toBe(NOT_LISTED,);
        expect(listingRowFor({
          body: { data: 'nothing', },
          servedId: 'mittens-9',
        },),).toBe(NOT_LISTED,);
      },
    },),

    it({
      name: 'REFUSES A LISTING THAT CARRIES THE SERVED ID ON TWO ROWS THAT DIFFER, since the card would describe '
        + 'whichever came first, and READS the row where every row under the id says the same',
      fn: async () => {
        /**
         Row the listing carries twice, word for word.
         */
        const row = {
          id: 'mittens-9',
          context_window: 200_000,
          max_output_tokens: 32_000,
        };
        /**
         What reading a listing whose second row under the id carries another ceiling throws.
         */
        const refusal = caught(function readApart(): unknown {
          return listingRowFor({
            body: {
              data: [
                row,
                {
                  ...row,
                  max_output_tokens: 8_000,
                },
              ],
            },
            servedId: 'mittens-9',
          },);
        },);
        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          'StatedRefusalError: the listing carries more than one row under mittens-9 and they differ, so which of '
            + 'them the card would describe cannot be read; read the listing by hand',
        );
        expect(listingRowFor({
          body: {
            data: [
              row,
              { ...row, },
            ],
          },
          servedId: 'mittens-9',
        },),).toEqual(row,);
      },
    },),

    it({
      name: 'READS OPENROUTER FIELDS into the card: image input off the architecture, the top endpoint\'s '
        + 'ceiling, and prices scaled from per token to per million',
      fn: async () => {
        /**
         OpenRouter-shaped row.
         */
        const row = listingRowFor({
          body: LISTING,
          servedId: 'whiskers/mittens-9',
        },);
        if ((typeof row) === 'symbol')
          throw new Error('fixture row missing',);
        expect(cardFieldsFrom({
          provider: 'openrouter',
          row,
        },),).toEqual({
          readsImages: true,
          maxOutputLength: 64_000,
          contextLength: 200_000,
          promptPrice: 0.3,
          completionPrice: 1.2,
        },);
      },
    },),

    it({
      name: 'READS SYNTHETIC PRICES WRITTEN WITH A DOLLAR SIGN, as its listing writes them, into dollars per token',
      fn: async () => {
        expect(cardFieldsFrom({
          provider: 'synthetic',
          row: {
            id: 'hf:cat/Mittens-9',
            input_modalities: ['text',],
            context_length: 100_000,
            max_output_length: 8_000,
            pricing: {
              prompt: '$0.000003',
              completion: '$0.000015',
            },
          },
        },),).toEqual({
          readsImages: false,
          maxOutputLength: 8_000,
          contextLength: 100_000,
          promptPrice: 0.000003,
          completionPrice: 0.000015,
        },);
      },
    },),

    it({
      name: 'READS HYPER FIELDS into the card and leaves the prices NOT_LISTED, since Hyper bills in credits '
        + 'the card does not carry',
      fn: async () => {
        /**
         Hyper-shaped row.
         */
        const row = listingRowFor({
          body: LISTING,
          servedId: 'mittens-9',
        },);
        if ((typeof row) === 'symbol')
          throw new Error('fixture row missing',);
        expect(cardFieldsFrom({
          provider: 'hyper',
          row,
        },),).toEqual({
          readsImages: false,
          maxOutputLength: 32_000,
          contextLength: 200_000,
          promptPrice: NOT_LISTED,
          completionPrice: NOT_LISTED,
        },);
      },
    },),

    it({
      name: 'LEAVES NOT_LISTED a field written blank, in a radix or below zero, which `Number` read as a free '
        + 'model, a price of one dollar a token and a negative price (ledger B73)',
      fn: async () => {
        expect([
          cardFieldsFrom({
            provider: 'synthetic',
            row: {
              id: 'hf:cat/Mittens-9',
              input_modalities: ['text',],
              context_length: ' ',
              max_output_length: -1,
              pricing: {
                prompt: '$ ',
                completion: '$0x1',
              },
            },
          },),
          cardFieldsFrom({
            provider: 'openrouter',
            row: {
              id: 'whiskers/mittens-9',
              context_length: 200_000,
              architecture: { input_modalities: ['text',], },
              pricing: {
                prompt: '-1',
                completion: '-0.000001',
              },
            },
          },),
        ],).toEqual([
          {
            readsImages: false,
            maxOutputLength: NOT_LISTED,
            contextLength: NOT_LISTED,
            promptPrice: NOT_LISTED,
            completionPrice: NOT_LISTED,
          },
          {
            readsImages: false,
            maxOutputLength: 200_000,
            contextLength: 200_000,
            promptPrice: NOT_LISTED,
            completionPrice: NOT_LISTED,
          },
        ],);
      },
    },),

    it({
      name: 'LEAVES NOT_LISTED a number past the range a double holds, which `JSON.parse` reads as Infinity '
        + '(ledger B73)',
      fn: async () => {
        /**
         Row as `JSON.parse` decodes a listing that writes its window past a double's range.
         */
        const row: unknown = JSON.parse(
          '{"id":"hf:cat/Mittens-9","input_modalities":["text"],"context_length":1e400,"max_output_length":8000}',
        );
        if (((typeof row) !== 'object') || (row === null))
          throw new Error('fixture row did not decode to an object',);
        expect(cardFieldsFrom({
          provider: 'synthetic',
          row: row as Record<string, unknown>,
        },).contextLength,).toBe(NOT_LISTED,);
      },
    },),

    it({
      name: 'LEAVES NOT_LISTED a string of digits past the range a double holds, which `Number` reads as '
        + 'Infinity though every character is a digit (ledger B73)',
      fn: async () => {
        expect(cardFieldsFrom({
          provider: 'synthetic',
          row: {
            id: 'hf:cat/Mittens-9',
            input_modalities: ['text',],
            context_length: `1${'0'.repeat(400,)}`,
            max_output_length: 8_000,
            pricing: {
              prompt: `$1${'0'.repeat(400,)}`,
              completion: '$0.000015',
            },
          },
        },),).toEqual({
          readsImages: false,
          maxOutputLength: 8_000,
          contextLength: NOT_LISTED,
          promptPrice: NOT_LISTED,
          completionPrice: 0.000015,
        },);
      },
    },),

    it({
      name: 'READS a string field only as JSON writes an unsigned number: digits, a fraction with digits on both '
        + 'sides of its point, and an exponent with digits, as a catalogue in OpenRouter\'s shape writes '
        + '`0.042e-6`; a second point, a point missing a digit on one side and an exponent missing its digits '
        + 'stay NOT_LISTED (ledger B73)',
      fn: async () => {
        expect([
          cardFieldsFrom({
            provider: 'synthetic',
            row: {
              id: 'hf:cat/Mittens-9',
              input_modalities: ['text',],
              context_length: '1.2.3',
              max_output_length: '8000.',
              pricing: {
                prompt: '$.5',
                completion: '$1e',
              },
            },
          },),
          cardFieldsFrom({
            provider: 'synthetic',
            row: {
              id: 'hf:cat/Mittens-9',
              input_modalities: ['text',],
              context_length: 'e5',
              max_output_length: '1e+',
              pricing: {
                prompt: '$1.5e3.2',
                completion: '$1e5e5',
              },
            },
          },),
          cardFieldsFrom({
            provider: 'synthetic',
            row: {
              id: 'hf:cat/Mittens-9',
              input_modalities: ['text',],
              context_length: '1.28E5',
              max_output_length: '8000',
              pricing: {
                prompt: '$0.042e-6',
                completion: '$1.5e-7',
              },
            },
          },),
          cardFieldsFrom({
            provider: 'synthetic',
            row: {
              id: 'hf:cat/Mittens-9',
              input_modalities: ['text',],
              context_length: '128000',
              max_output_length: '8000',
              pricing: {
                prompt: '$0.5',
                completion: '$7',
              },
            },
          },),
        ],).toEqual([
          {
            readsImages: false,
            maxOutputLength: NOT_LISTED,
            contextLength: NOT_LISTED,
            promptPrice: NOT_LISTED,
            completionPrice: NOT_LISTED,
          },
          {
            readsImages: false,
            maxOutputLength: NOT_LISTED,
            contextLength: NOT_LISTED,
            promptPrice: NOT_LISTED,
            completionPrice: NOT_LISTED,
          },
          {
            readsImages: false,
            maxOutputLength: 8_000,
            contextLength: 128_000,
            promptPrice: 4.2e-8,
            completionPrice: 1.5e-7,
          },
          {
            readsImages: false,
            maxOutputLength: 8_000,
            contextLength: 128_000,
            promptPrice: 0.5,
            completionPrice: 7,
          },
        ],);
      },
    },),

    it({
      name: 'RENDERS the provider side with the listing\'s values and a question for each field it lacks',
      fn: async () => {
        /**
         Rendered OpenRouter side.
         */
        const rendered = renderProviderCard({
          provider: 'openrouter',
          servedId: 'whiskers/mittens-9',
          fields: {
            readsImages: true,
            maxOutputLength: 64_000,
            contextLength: 200_000,
            promptPrice: 0.3,
            completionPrice: NOT_LISTED,
          },
        },);
        expect(rendered,).toContain(`    openrouter: {\n      id: 'whiskers/mittens-9',\n      readsImages: true,\n      maxOutputLength: 64000,`,);
        expect(rendered,).toContain('      promptUsdPerMillion: 0.3,',);
        expect(rendered,).toContain('      completionUsdPerMillion: /* not in the listing:',);
        expect(rendered,).toContain(`      ignoredEndpoints: [],\n      preferredEndpoints: [],\n      rawCharsPerToken: 'unmeasured',\n    },`,);
        /**
         Rendered Bedrock side, whose listing says nothing a card needs.
         */
        const bedrock = renderProviderCard({
          provider: 'bedrock',
          servedId: 'cat.mittens-9',
          fields: {
            readsImages: NOT_LISTED,
            maxOutputLength: NOT_LISTED,
            contextLength: NOT_LISTED,
            promptPrice: NOT_LISTED,
            completionPrice: NOT_LISTED,
          },
        },);
        expect(bedrock,).toContain('      readsImages: /* not in the listing: send it a picture and measure */,',);
        expect(bedrock,).toContain(`      route: /* 'openai-v1' | 'v1', measured */,`,);
      },
    },),

    it({
      name: 'READS THE ASK off the command line and refuses a missing id, an empty one, or a provider that is not '
        + 'one of the four, saying which part is wrong and quoting what was typed (ledger B75)',
      fn: async () => {
        expect(readAsk({
          line: lineOf({
            command: 'roster-card',
            typed: [
              'hyper',
              'mittens-9',
            ],
          },),
        },),).toEqual({
          provider: 'hyper',
          servedId: 'mittens-9',
        },);
        expect(function noId(): void {
          readAsk({
            line: lineOf({
              command: 'roster-card',
              typed: ['hyper',],
            },),
          },);
        },).toThrow('this command needs <served id>',);
        expect(function emptyId(): void {
          readAsk({
            line: lineOf({
              command: 'roster-card',
              typed: [
                'hyper',
                '',
              ],
            },),
          },);
        },).toThrow('roster-card\'s served id is written empty',);
        expect(function notAProvider(): void {
          readAsk({
            line: lineOf({
              command: 'roster-card',
              typed: [
                'cat nip',
                'mittens-9',
              ],
            },),
          },);
        },).toThrow(
          'roster-card\'s provider is one of synthetic, hyper, openrouter, bedrock, neuralwatt, deepseek, xiaomi, anthropic, '
            + 'and "cat nip" is none of them',
        );
        expect(function bothWrong(): void {
          readAsk({
            line: lineOf({
              command: 'roster-card',
              typed: [
                'Hyper',
                '',
              ],
            },),
          },);
        },).toThrow(
          'roster-card\'s provider is one of synthetic, hyper, openrouter, bedrock, neuralwatt, deepseek, xiaomi, anthropic, '
            + 'and "Hyper" is none of them; '
            + 'roster-card\'s served id is written empty',
        );
      },
    },),

    it({
      name: 'READS THE BODY ITSELF where it is the row array, and LEAVES THE IMAGE INPUT NOT_LISTED where the '
        + 'modalities field is no array',
      fn: async () => {
        /**
         Row named under the served id, and one whose modalities field is no
         array.
         */
        const row = listingRowFor({
          body: [{ id: 'm', },],
          servedId: 'm',
        },);
        expect(row,).toEqual({ id: 'm', },);
        expect(cardFieldsFrom({
          provider: 'openrouter',
          row: {
            id: 'm',
            architecture: { input_modalities: 'images', },
          },
        },),).toEqual({
          readsImages: NOT_LISTED,
          maxOutputLength: NOT_LISTED,
          contextLength: NOT_LISTED,
          promptPrice: NOT_LISTED,
          completionPrice: NOT_LISTED,
        },);
      },
    },),

    it({
      name: 'READS A NEURALWATT LISTING: limits and vision under metadata, a null output ceiling falling back to the '
        + 'context window, and no price on the card',
      fn: async () => {
        expect(cardFieldsFrom({
          provider: 'neuralwatt',
          row: {
            id: 'm',
            metadata: {
              capabilities: { vision: true, },
              limits: { max_context_length: 1_048_560, max_output_tokens: 393_216, },
            },
          },
        },),).toEqual({
          readsImages: true,
          maxOutputLength: 393_216,
          contextLength: 1_048_560,
          promptPrice: NOT_LISTED,
          completionPrice: NOT_LISTED,
        },);
        expect(cardFieldsFrom({
          provider: 'neuralwatt',
          row: {
            id: 'm',
            metadata: {
              capabilities: { vision: 'sometimes', },
              limits: { max_context_length: 1_048_560, max_output_tokens: null, },
            },
          },
        },),).toEqual({
          readsImages: NOT_LISTED,
          maxOutputLength: 1_048_560,
          contextLength: 1_048_560,
          promptPrice: NOT_LISTED,
          completionPrice: NOT_LISTED,
        },);
        expect(renderProviderCard({
          provider: 'neuralwatt',
          servedId: 'm',
          fields: {
            readsImages: true,
            maxOutputLength: 393_216,
            contextLength: 1_048_560,
            promptPrice: NOT_LISTED,
            completionPrice: NOT_LISTED,
          },
        },),).toContain("    neuralwatt: {\n      id: 'm',\n      readsImages: true,\n      maxOutputLength: 393216,\n      contextLength: 1048560,\n    },",);
      },
    },),

    it({
      name: 'READS the vision flag where it is a boolean and NOT_LISTED where it is anything else, and '
        + 'reads a bedrock listing as the bare fields it names',
      fn: async () => {
        expect(cardFieldsFrom({
          provider: 'hyper',
          row: {
            id: 'm',
            capabilities: { vision: true, },
          },
        },).readsImages,).toBe(true,);
        expect(cardFieldsFrom({
          provider: 'hyper',
          row: {
            id: 'm',
            capabilities: { vision: 'sometimes', },
          },
        },).readsImages,).toBe(NOT_LISTED,);
        const bedrock = cardFieldsFrom({
          provider: 'bedrock',
          row: {
            id: 'm',
            max_output_tokens: 64_000,
            context_length: 200_000,
          },
        },);
        expect(bedrock,).toEqual({
          readsImages: NOT_LISTED,
          maxOutputLength: 64_000,
          contextLength: 200_000,
          promptPrice: NOT_LISTED,
          completionPrice: NOT_LISTED,
        },);
      },
    },),
  ],
},);
