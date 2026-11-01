/**
 Tests the command line `reader-measure` reads: the model (Gemma by default),
 the trusted directories, the pictures directory, the sample size and the key,
 and the refusals that name each missing part.

 Fixtures are cat-themed invention. No provider is reached.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { readMeasureAsk, } from '../../dist/final/node/index.mjs';

await describe({
  name: readMeasureAsk.name,
  children: [
    it({
      name: 'READS THE MODEL (DEFAULT GEMMA), THE TRUSTED DIRECTORIES, THE PICTURES DIRECTORY, THE SAMPLE (DEFAULT 9) '
        + 'AND THE KEY',
      fn: async () => {
        const ask = readMeasureAsk({
          line: lineOf({
            command: 'reader-measure',
            typed: ['--trusted', '/runs/one', '--trusted', '/runs/two', '--pictures', '/people',],
          },),
          env: { TRANSLATION_REPAIR_OPENROUTER_API_KEY: 'whisker-key-7421', },
        },);
        expect(ask,).toEqual({
          modelId: 'gemma-4-26b-a4b-it',
          trustedDirs: ['/runs/one', '/runs/two',],
          picturesDir: '/people',
          sample: 9,
          apiKey: 'whisker-key-7421',
        },);
      },
    },),

    it({
      name: 'REFUSES A LINE WITH NO TRUSTED DIRECTORY, NO PICTURES DIRECTORY OR NO KEY, naming each missing part',
      fn: async () => {
        const refusal = await rejectionOf(async function readEmptyLine(): Promise<unknown> {
          await Promise.resolve();
          return readMeasureAsk({
            line: lineOf({ command: 'reader-measure', typed: [], },),
            env: {},
          },);
        },);
        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect((refusal as Error).message,).toContain('--trusted',);
        expect((refusal as Error).message,).toContain('--pictures',);
        expect((refusal as Error).message,).toContain('TRANSLATION_REPAIR_OPENROUTER_API_KEY',);
      },
    },),

    it({
      name: 'REFUSES A ROSTER MODEL OPENROUTER DOES NOT SERVE AND A NAME THAT IS NO ROSTER MODEL, since the '
        + 'measurement is of that route',
      fn: async () => {
        for (const model of ['google.gemma-4-e2b', 'tabby-9',]) {
          /* oxlint-disable no-await-in-loop -- two refusals, one after the other */
          const refusal = await rejectionOf(async function readUnservedModel(): Promise<unknown> {
            await Promise.resolve();
            return readMeasureAsk({
              line: lineOf({
                command: 'reader-measure',
                typed: ['--model', model, '--trusted', '/runs', '--pictures', '/people',],
              },),
              env: { TRANSLATION_REPAIR_OPENROUTER_API_KEY: 'whisker-key-7421', },
            },);
          },);
          /* oxlint-enable no-await-in-loop */
          expect(refusal,).toBeInstanceOf(StatedRefusalError,);
          expect((refusal as Error).message,).toContain('OpenRouter',);
        }
      },
    },),
  ],
},);
