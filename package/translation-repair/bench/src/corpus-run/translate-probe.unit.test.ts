/**
 Tests for `translate-probe` as an operator runs it: the built command in a
 child process that holds no provider key, over a throwaway corpus.

 THE COMMAND SPENDS QUOTA, so what is checked here is what happens before any
 call and without one: the entry the command is fixed on, an entry with
 nothing to carve, the flags it does not read, and the key it cannot find, which is a refusal
 of the command and not one slice's failure.

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
  runProbeOver,
} from '@monochromatic-dev/translation-repair-corpus-pass/ts/corpus-run/probes-b-built-command.test-fixture.ts';

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
const REFUSED_AS_STATED = 6;

/**
 Entry the command is fixed on.
 */
const ENTRY = 'XingZ60';

/**
 Original of the fixture entry: three blocks.
 */
const SOURCE_PAGE = '# 星\n\n星睡觉。\n\n星吃鱼。\n';

/**
 Translation of the fixture entry: two blocks.
 */
const TARGET_PAGE = '# Star\n\nStar naps.\n';

await describe({
  name: 'translate-probe as built',
  children: [
    it({
      name: 'RUNS TO ITS END on an entry with nothing to carve, saying so, and exits 0',
      fn: async () => {
        expect(await runProbeOver({
          command: 'translate-probe',
          packageEntry: BENCH_ENTRY,
          files: {
            [`people/${ENTRY}/page.md`]: '',
            [`people/${ENTRY}/page.en.md`]: '',
          },
          args: [],
        },),).toEqual({
          code: 0,
          stdout: 'TRANSLATE no aligned section carries source blocks\n',
          stderr: '',
        },);
      },
    },),
    it({
      name: 'REFUSES as stated and exits 6 when the corpus lacks the entry it is fixed on',
      fn: async () => {
        await using corpus = await makeProbeCorpus({ files: { 'people/Mittens/page.md': 'x\n', }, },);
        await using runs = await scratchDir({ prefix: 'translate-probe-runs-', },);
        expect(await runBuiltCommand({
          command: 'translate-probe',
          packageEntry: BENCH_ENTRY,
          args: [],
          env: {
            ...corpusEnvOf({ corpus, },),
            TRANSLATION_REPAIR_RUNS_DIR: runs.path,
          },
        },),).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: `translate-probe: corpus read failed for ${corpus.commitSha}:people/${ENTRY}/page.md (missing-object); `
            + 'the commit has no such path: check the path, or pin a commit that has it.\n',
        },);
      },
    },),
    it({
      name: 'REFUSES a flag it does not read, and exits 6 with its usage line',
      fn: async () => {
        expect(await runProbeOver({
          command: 'translate-probe',
          packageEntry: BENCH_ENTRY,
          files: {
            [`people/${ENTRY}/page.md`]: SOURCE_PAGE,
            [`people/${ENTRY}/page.en.md`]: TARGET_PAGE,
          },
          args: ['--bogus',],
        },),).toEqual({
          code: REFUSED_AS_STATED,
          stdout: '',
          stderr: 'translate-probe: --bogus is not a flag this command reads. Usage: translate-probe\n',
        },);
      },
    },),
    it({
      name: 'PRINTS the section it chose and its slice, then REFUSES as stated for want of a key and exits 6, before any call',
      fn: async () => {
        expect(await runProbeOver({
          command: 'translate-probe',
          packageEntry: BENCH_ENTRY,
          files: {
            [`people/${ENTRY}/page.md`]: SOURCE_PAGE,
            [`people/${ENTRY}/page.en.md`]: TARGET_PAGE,
          },
          args: [],
        },),).toEqual({
          code: REFUSED_AS_STATED,
          stdout: 'TRANSLATE XingZ60: source 3 blocks / 15 chars, target 2 blocks / 18 chars, coverage 0.667\n'
            + 'TRANSLATE section subdivides into 1 slice; probing the first 1\n'
            + '\n--- slice: 15 source chars, 18 target chars ---\n'
            + 'SOURCE: # 星\n\n星睡觉。\n\n星吃鱼。\n',
          stderr: 'translate-probe: TRANSLATION_REPAIR_NEURALWATT_API_KEY, TRANSLATION_REPAIR_SYNTHETIC_API_KEY, '
            + 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY, '
            + 'TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY, DEEPSEEK_API_KEY, TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY, TRANSLATION_REPAIR_CLAUDE_API_KEY '
            + 'or TRANSLATION_REPAIR_OPENROUTER_API_KEY is not set; '
            + 'run under mise so sops injects it\n',
        },);
      },
    },),
  ],
},);
