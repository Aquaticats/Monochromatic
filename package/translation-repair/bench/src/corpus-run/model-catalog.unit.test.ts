/**
 Boundary test for the model catalog command.

 The command asks the provider what it serves, so what is checked here is the
 one thing only the built command can show: launched with no provider key it
 refuses as stated, exits 6 and prints nothing to stdout, before any request.
 Its procedures have their own suites.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { BENCH_ENTRY, } from './built-entry.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

await describe({
  name: 'model-catalog as built',
  children: [
    it({
      name: 'REFUSES as stated and exits 6 when launched with no provider key, before any request',
      fn: async () => {
        // The keys are withheld by `runBuiltCommand`, which builds the child's
        // whole environment from the parent's without any `_API_KEY` variable.
        const run = await runBuiltCommand({
          command: 'model-catalog',
          packageEntry: BENCH_ENTRY,
          args: [],
          env: {},
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'model-catalog: TRANSLATION_REPAIR_SYNTHETIC_API_KEY is not set; run under mise so sops injects it\n',
        },);
      },
    },),
    it({
      name: 'REFUSES an argument the command does not read, exits 6 and prints the refusal alone',
      fn: async () => {
        const run = await runBuiltCommand({
          command: 'model-catalog',
          packageEntry: BENCH_ENTRY,
          args: ['extra',],
          env: {},
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'model-catalog: this command takes no argument but its flags, and was given "extra". '
            + 'Usage: model-catalog\n',
        },);
      },
    },),
  ],
},);
