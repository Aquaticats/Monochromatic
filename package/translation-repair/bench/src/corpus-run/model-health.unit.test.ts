/**
 Boundary test for the model health command.

 The command asks every model on the roster a question, so what is checked here
 is the one thing only the built command can show: launched with no provider
 key it refuses as stated, exits 6 and prints nothing to stdout, before any
 call. Its procedures have their own suites.

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

await describe({
  name: 'model-health as built',
  children: [
    it({
      name: 'REFUSES as stated and exits 6 when launched with no provider key, before any call',
      fn: async () => {
        // The keys are withheld by `runBuiltCommand`, which builds the child's
        // whole environment from the parent's without any `_API_KEY` variable.
        const run = await runBuiltCommand({
          command: 'model-health',
          packageEntry: BENCH_ENTRY,
          args: [],
          env: {},
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'model-health: TRANSLATION_REPAIR_NEURALWATT_API_KEY, TRANSLATION_REPAIR_SYNTHETIC_API_KEY, '
            + 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY, '
            + 'TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY, DEEPSEEK_API_KEY, TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY, TRANSLATION_REPAIR_CLAUDE_API_KEY '
            + 'or TRANSLATION_REPAIR_OPENROUTER_API_KEY is not set; '
            + 'run under mise so sops injects it\n',
        },);
      },
    },),
    it({
      name: 'REFUSES a command-line argument the command does not read, exits 6 and prints the refusal alone',
      fn: async () => {
        const run = await runBuiltCommand({
          command: 'model-health',
          packageEntry: BENCH_ENTRY,
          args: ['--bogus',],
          env: {},
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'model-health: --bogus is not a flag this command reads. Usage: model-health\n',
        },);
      },
    },),
  ],
},);
