/**
 Tests for `coverage-control-probe` as an operator runs it: the built command
 in a child process that holds no provider key, over a throwaway corpus.

 THE COMMAND SPENDS QUOTA, so what is checked here is only what happens before
 any call: the key refusal, which the command makes before it reads its entry
 filter or the corpus, and a flag it does not read.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  ONE_ENTRY_CORPUS,
  runProbeOver,
} from '@monochromatic-dev/translation-repair-corpus-pass/ts/corpus-run/probes-b-built-command.test-fixture.ts';
import { COVERAGE_CENSUS_ENTRY, } from './built-entry.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 Built command under test.
 */
const COMMAND = 'coverage-control-probe';

/**
 Line the missing key refuses with.
 */
const KEY_REFUSAL = 'coverage-control-probe: TRANSLATION_REPAIR_NEURALWATT_API_KEY, TRANSLATION_REPAIR_SYNTHETIC_API_KEY, '
  + 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY, '
  + 'TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY, DEEPSEEK_API_KEY, TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY, TRANSLATION_REPAIR_CLAUDE_API_KEY or '
  + 'TRANSLATION_REPAIR_OPENROUTER_API_KEY is not set; '
  + 'run under mise so sops injects it\n';

await describe({
  name: 'coverage-control-probe as built',
  children: [
    it({
      name: 'REFUSES as stated and exits 6 for a named entry the corpus holds once no provider key is set, before any call',
      fn: async () => {
        /**
         What the command did.
         */
        const run = await runProbeOver({
          command: COMMAND,
          files: ONE_ENTRY_CORPUS,
          packageEntry: COVERAGE_CENSUS_ENTRY,
          args: ['--only', 'Mittens',],
        },);
        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: KEY_REFUSAL,
        },);
      },
    },),
    it({
      name: 'REFUSES for the missing key before it reads the entry filter, so an entry the corpus lacks is not yet named',
      fn: async () => {
        /**
         What the command did.
         */
        const run = await runProbeOver({
          command: COMMAND,
          files: ONE_ENTRY_CORPUS,
          packageEntry: COVERAGE_CENSUS_ENTRY,
          args: ['--only', 'Nobody',],
        },);
        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: KEY_REFUSAL,
        },);
      },
    },),
    it({
      name: 'REFUSES a flag it does not read, and exits 6 with its usage line',
      fn: async () => {
        /**
         What the command did.
         */
        const run = await runProbeOver({
          command: COMMAND,
          files: ONE_ENTRY_CORPUS,
          packageEntry: COVERAGE_CENSUS_ENTRY,
          args: ['--bogus',],
        },);
        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'coverage-control-probe: --bogus is not a flag this command reads. '
            + 'Usage: coverage-control-probe [--only <entry ids>]\n',
        },);
      },
    },),
  ],
},);
