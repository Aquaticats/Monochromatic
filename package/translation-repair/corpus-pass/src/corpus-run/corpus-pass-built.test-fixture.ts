import { join, } from 'node:path';

import {
  PROVIDERS_LEFT_OFF,
  readHeadSha,
} from '@monochromatic-dev/translation-repair-run';
import { digestPipeline, } from '@monochromatic-dev/translation-repair-glossary';
import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { passProvidersOffLines, } from '../../dist/final/node/index.mjs';
import type { CorpusCloneEntry, } from './corpus-pass-clone.test-fixture.ts';

//region Corpus pass built command fixture
// What the as-built suites of `corpus-pass` share: the child's environment, the
// lines the pass prints, and the invented corpora they run over. Every child
// goes through `runBuiltWithoutKeys`, which removes every `_API_KEY` variable
// and every `TRANSLATION_REPAIR_` variable before the case adds its scratch
// locations.

/**
 URL of the built index of this package, which the built pass sits beside.
 */
const CORPUS_PASS_ENTRY = import.meta.resolve('@monochromatic-dev/translation-repair-corpus-pass',);

/**
 Exit code `reportingRefusals` sets for a stated refusal.
 */
export const REFUSED_AS_STATED: number = 6;

/**
 Directory the built commands, and so the pipeline digest, live in.
 */
const BUILT_DIRECTORY = join(
  import.meta.dirname,
  '../../dist/final/node',
);

/**
 Line the pass prints when the built-in writer window applies, which is every
 launch that does not set its own.
 */
export const WRITER_GRACE_BUILT_IN: string = 'WRITER GRACE built in: writer rounds (editor, refiner, translate, '
  + 'produceConsolidations, archive-block-review) abandon stragglers 180000ms after quorum rather than the '
  + '120000ms every other round waits, the owner\'s decision of 2026-09-06; '
  + 'TRANSLATION_REPAIR_WRITER_GRACE_MS moves it for one launch';

/**
 Lines the pass prints to say which providers it leaves off, which every launch past the required-provider gate prints
 before it builds the run client.
 */
export const PROVIDERS_OFF_LINES: readonly string[] = passProvidersOffLines({ off: PROVIDERS_LEFT_OFF, },);

/**
 Refusal the run client raises while no provider key is set.
 */
export const NO_KEY_REFUSAL: string = 'corpus-pass: TRANSLATION_REPAIR_NEURALWATT_API_KEY, TRANSLATION_REPAIR_SYNTHETIC_API_KEY, '
  + 'TRANSLATION_REPAIR_CHARM_HYPER_API_KEY, TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY, DEEPSEEK_API_KEY, '
  + 'TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY, TRANSLATION_REPAIR_CLAUDE_API_KEY '
  + 'or TRANSLATION_REPAIR_OPENROUTER_API_KEY is not set; '
  + 'run under mise so sops injects it\n';

/**
 Entries of the invented corpus: two complete pairs.
 */
export const ENTRIES: readonly CorpusCloneEntry[] = [
  {
    id: 'tabby',
    sourceText: '猫睡觉。\n',
    targetText: 'The cat naps.\n',
  },
  {
    id: 'biscuit',
    sourceText: '饼干猫。\n',
    targetText: 'Biscuit naps.\n',
  },
];

/**
 Entries of a corpus where one cat has no English page.
 */
export const ENTRIES_WITH_A_GAP: readonly CorpusCloneEntry[] = [
  ...ENTRIES,
  {
    id: 'mittens',
    sourceText: '手套猫。\n',
  },
];

/**
 What the pass wrote to each stream and how it exited, with the places it ran in.
 */
type PassRun = {
  /**
   Exit code of the child.
   */
  readonly code: number;

  /**
   Everything the child wrote to stdout.
   */
  readonly stdout: string;

  /**
   Everything the child wrote to stderr.
   */
  readonly stderr: string;
};

/**
 Runs the built pass with every key and every package location variable
 withheld, then pointed at the scratch locations the case names.

 @param args - arguments after the command

 @param runsDir - scratch runs directory

 @param cacheDir - scratch lookup cache directory

 @param clone - throwaway corpus clone and its commit

 @param settings - further `TRANSLATION_REPAIR_` variables the case sets

 @returns Exit code and both streams

 @example
 ```ts
 const run = await runPass({ args: [], runsDir, cacheDir, clone, settings: {}, },);
 ```
 */
export async function runPass(
  {
    args,
    runsDir,
    cacheDir,
    clone,
    settings,
  }: {
    readonly args: readonly string[];
    readonly runsDir: string;
    readonly cacheDir: string;
    readonly clone: {
      readonly cloneDir: string;
      readonly commitSha: string;
    };
    readonly settings: Readonly<Record<string, string>>;
  },
): Promise<PassRun> {
  return await runBuiltCommand({
    command: 'corpus-pass',
    packageEntry: CORPUS_PASS_ENTRY,
    args,
    env: {
      TRANSLATION_REPAIR_RUNS_DIR: runsDir,
      TRANSLATION_REPAIR_LOOKUP_CACHE_DIR: cacheDir,
      TRANSLATION_REPAIR_CORPUS_CLONE_DIR: clone.cloneDir,
      TRANSLATION_REPAIR_CORPUS_COMMIT: clone.commitSha,
      ...settings,
    },
  },);
}

/**
 The START line this build prints for the given counts and ceiling.

 @param pending - entries the pass would run

 @param hardMs - per-entry ceiling in milliseconds

 @returns The whole line, without its newline

 @example
 ```ts
 const line = await startLine({ pending: 1, hardMs: 25_200_000, },);
 ```
 */
export async function startLine(
  {
    pending,
    hardMs,
  }: {
    readonly pending: number;
    readonly hardMs: number;
  },
): Promise<string> {
  /**
   The digest and file count the child takes over its own directory.
   */
  const {
    digest,
    fileCount,
  } = await digestPipeline({ dir: BUILT_DIRECTORY, },);
  return `START tip=${await readHeadSha()} pipeline=${digest} files=${String(fileCount,)} pending=${
    String(pending,)
  } done=0 soft=259200000ms hard=${String(hardMs,)}ms call=360000ms claude-call=460000ms`;
}

/**
 The pin line the pass prints for a clone the case made.

 @param clone - throwaway corpus clone and its commit

 @returns The whole line, without its newline

 @example
 ```ts
 const line = pinLine({ clone, },);
 ```
 */
export function pinLine(
  { clone, }: {
    readonly clone: {
      readonly cloneDir: string;
      readonly commitSha: string;
    };
  },
): string {
  return `CORPUS PIN OVERRIDDEN: clone ${clone.cloneDir} from TRANSLATION_REPAIR_CORPUS_CLONE_DIR, commit ${
    clone.commitSha
  } from TRANSLATION_REPAIR_CORPUS_COMMIT`;
}

//endregion Corpus pass built command fixture
