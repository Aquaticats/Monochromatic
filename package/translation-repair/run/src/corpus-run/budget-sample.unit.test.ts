/**
 Boundary test for the budget sample command.

 The command reads every provider's meter, so what is checked here is what
 only the built command can show with no provider key in its environment: it
 refuses as stated, names each variable and whether it is present, exits 6 and
 prints nothing to stdout, before any request. Its procedures have their own
 suites.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { RUN_ENTRY, } from './built-entry.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

await describe({
  name: 'budget-sample as built',
  children: [
    it({
      name: 'REFUSES as stated and exits 6, naming every key variable as absent, when launched with no provider key',
      fn: async () => {
        // The keys are withheld by `runBuiltCommand`, which builds the child's
        // whole environment from the parent's without any `_API_KEY` variable.
        const run = await runBuiltCommand({
          command: 'budget-sample',
          packageEntry: RUN_ENTRY,
          args: [],
          env: {},
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'budget-sample: every provider key must be set to sample availability, and at least one is not: '
            + 'TRANSLATION_REPAIR_NEURALWATT_API_KEY is absent, TRANSLATION_REPAIR_SYNTHETIC_API_KEY is absent, '
            + 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY is absent, '
            + 'TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY is absent, TRANSLATION_REPAIR_OPENROUTER_API_KEY is absent. '
            + 'Run under mise so sops injects them. A sample of some providers is not recorded, because the record '
            + 'is read as a statement about all of them and a missing column would be indistinguishable from a '
            + 'provider that answered.\n',
        },);
      },
    },),
    it({
      name: 'REFUSES an argument the command does not read, exits 6 and prints the refusal alone',
      fn: async () => {
        const run = await runBuiltCommand({
          command: 'budget-sample',
          packageEntry: RUN_ENTRY,
          args: ['extra',],
          env: {},
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'budget-sample: this command takes no argument but its flags, and was given "extra". '
            + 'Usage: budget-sample\n',
        },);
      },
    },),
  ],
},);
