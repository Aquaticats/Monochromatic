/**
 Tests for `sentinel-probe` as an operator runs it: the built command in a
 child process that holds no provider key, over a throwaway corpus.

 THE COMMAND SPENDS QUOTA, so what is checked here is only what happens before
 any call: the order of its refusals (an id the corpus lacks is refused before
 the key is asked for), the key refusal itself, and a flag it does not read.

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
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  corpusEnvOf,
  makeProbeCorpus,
} from '@monochromatic-dev/translation-repair-corpus-pass/ts/corpus-run/probes-b-built-command.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 Line the missing key refuses with.
 */
const KEY_REFUSAL = 'sentinel-probe: TRANSLATION_REPAIR_NEURALWATT_API_KEY, TRANSLATION_REPAIR_SYNTHETIC_API_KEY, '
  + 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY, '
  + 'TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY, DEEPSEEK_API_KEY, TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY, TRANSLATION_REPAIR_CLAUDE_API_KEY or '
  + 'TRANSLATION_REPAIR_OPENROUTER_API_KEY is not set; '
  + 'run under mise so sops injects it\n';

await describe({
  name: 'sentinel-probe as built',
  children: [
    it({
      name: 'REFUSES as stated and exits 6 for a named id the corpus holds once no provider key is set, before any call',
      fn: async () => {
        await using corpus = await makeProbeCorpus({ files: { 'people/mittens/page.md': 'Mittens naps.\n', }, },);
        await using runs = await scratchDir({ prefix: 'sentinel-probe-runs-', },);
        const run = await runBuiltCommand({
          command: 'sentinel-probe',
          packageEntry: BENCH_ENTRY,
          args: ['mittens',],
          env: {
            ...corpusEnvOf({ corpus, },),
            TRANSLATION_REPAIR_RUNS_DIR: runs.path,
          },
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: KEY_REFUSAL,
        },);
      },
    },),
    it({
      name: 'REFUSES the default sentinels the corpus lacks before the key is asked for, and exits 6',
      fn: async () => {
        await using corpus = await makeProbeCorpus({ files: { 'people/mittens/page.md': 'Mittens naps.\n', }, },);
        await using runs = await scratchDir({ prefix: 'sentinel-probe-runs-', },);
        const run = await runBuiltCommand({
          command: 'sentinel-probe',
          packageEntry: BENCH_ENTRY,
          args: [],
          env: {
            ...corpusEnvOf({ corpus, },),
            TRANSLATION_REPAIR_RUNS_DIR: runs.path,
          },
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'sentinel-probe: sentinel-probe asks for "Anilovr", "Aniloviraw", which the corpus at the pin does not hold\n',
        },);
      },
    },),
    it({
      name: 'REFUSES a flag it does not read, and exits 6 with its usage line',
      fn: async () => {
        await using corpus = await makeProbeCorpus({ files: { 'people/mittens/page.md': 'Mittens naps.\n', }, },);
        await using runs = await scratchDir({ prefix: 'sentinel-probe-runs-', },);
        const run = await runBuiltCommand({
          command: 'sentinel-probe',
          packageEntry: BENCH_ENTRY,
          args: ['--bogus',],
          env: {
            ...corpusEnvOf({ corpus, },),
            TRANSLATION_REPAIR_RUNS_DIR: runs.path,
          },
        },);

        expect(run,).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'sentinel-probe: --bogus is not a flag this command reads. Usage: sentinel-probe [<entry id> ...]\n',
        },);
      },
    },),
  ],
},);
