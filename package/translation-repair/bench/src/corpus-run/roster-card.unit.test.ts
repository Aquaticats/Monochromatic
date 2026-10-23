/**
 Boundary test for the roster card command.

 The command reads a provider's live listing, so what is checked here is what
 only the built command can show: launched with no provider key it refuses as
 stated, exits 6 and prints nothing to stdout, before any request, and it
 names the key variable of the provider asked for. Its procedures have their
 own suites.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { BENCH_ENTRY, } from './built-entry.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 Key variable each provider's listing is read with.
 */
const KEY_VARIABLE_OF: Readonly<Record<string, string>> = {
  synthetic: 'TRANSLATION_REPAIR_SYNTHETIC_API_KEY',
  hyper: 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY',
  openrouter: 'TRANSLATION_REPAIR_OPENROUTER_API_KEY',
  bedrock: 'TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY',
  neuralwatt: 'TRANSLATION_REPAIR_NEURALWATT_API_KEY',
};

await describe({
  name: 'roster-card as built',
  children: [
    ...Object.entries(KEY_VARIABLE_OF,)
      .map(function keylessCase([provider, variable,],) {
        return it({
          name: `REFUSES as stated and exits 6, naming the key of ${provider}, when that key is not set`,
          fn: async () => {
            // The keys are withheld by `runBuiltCommand`, which builds the child's
            // whole environment from the parent's without any `_API_KEY` variable.
            const run = await runBuiltCommand({
              command: 'roster-card',
              packageEntry: BENCH_ENTRY,
              args: [
                provider,
                'hf:cat/Mittens-1',
              ],
              env: {},
            },);

            expect(run,).toEqual({
              code: REFUSED_AS_STATED,
              stdout: '',
              stderr: `roster-card: ${variable} is not set; run under mise so sops injects it\n`,
            },);
          },
        },);
      },),
    it({
      name: 'REFUSES a provider that is none of the four, quoting what was typed, before any key is read',
      fn: async () => {
        const run = await runBuiltCommand({
          command: 'roster-card',
          packageEntry: BENCH_ENTRY,
          args: [
            'bogus',
            'hf:cat/Mittens-1',
          ],
          env: {},
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'roster-card: roster-card\'s provider is one of synthetic, hyper, openrouter, bedrock, neuralwatt, deepseek, xiaomi, anthropic, '
            + 'and "bogus" is none of them\n',
        },);
      },
    },),
    it({
      name: 'REFUSES a flag the command does not read, exits 6 and prints the refusal alone',
      fn: async () => {
        const run = await runBuiltCommand({
          command: 'roster-card',
          packageEntry: BENCH_ENTRY,
          args: [
            'synthetic',
            'hf:cat/Mittens-1',
            '--flag',
          ],
          env: {},
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'roster-card: --flag is not a flag this command reads. '
            + 'Usage: roster-card <synthetic|hyper|openrouter|bedrock|neuralwatt|deepseek|xiaomi|anthropic> <served id>\n',
        },);
      },
    },),
    it({
      name: 'REFUSES a command line with no provider and no served id, exits 6 and says what it needs',
      fn: async () => {
        const run = await runBuiltCommand({
          command: 'roster-card',
          packageEntry: BENCH_ENTRY,
          args: [],
          env: {},
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'roster-card: this command needs <synthetic|hyper|openrouter|bedrock|neuralwatt|deepseek|xiaomi|anthropic> <served id>. '
            + 'Usage: roster-card <synthetic|hyper|openrouter|bedrock|neuralwatt|deepseek|xiaomi|anthropic> <served id>\n',
        },);
      },
    },),
  ],
},);
