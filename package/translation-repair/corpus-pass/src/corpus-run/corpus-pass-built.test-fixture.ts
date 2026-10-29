import { join, } from 'node:path';

import { isIsoStampText, } from '@monochromatic-dev/translation-repair-markup';
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
 The key of a clock time field of a pass line, with the space before it.
 */
const CLOCK_TIME_KEY = ' at=';

/**
 Characters of an instant in the spelling the logger writes, `2026-10-22T08:10:11.123Z`.
 */
const INSTANT_LENGTH = 24;

/**
 Splits what the pass printed into the clock time fields it carried and the text around them, in one pass over the
 text, a field being the key followed by an instant in the logger's spelling.

 @param text - what the pass printed

 @returns Text without the ` at=<instant>` fields, and the instants in the order they were printed

 @example
 ```ts
 const { rest, times, } = splitClockTimes({ text: 'DONE processed=0 of pending=0 at=2026-10-22T08:10:11.123Z', },);
 ```
 */
function splitClockTimes({ text, }: { readonly text: string; },): {
  readonly rest: string;
  readonly times: readonly string[];
} {
  /**
   Parts of the text between the keys, the first being the part before the first key.
   */
  const [
    head = '',
    ...tails
  ] = text.split(CLOCK_TIME_KEY,);

  /**
   Text kept, in order, from the part before the first key on.
   */
  const kept: string[] = [head,];

  /**
   Instants found, in order.
   */
  const times: string[] = [];
  for (const tail of tails) {
    /**
     What follows the key, as far as an instant is long.
     */
    const instant = tail.slice(
      0,
      INSTANT_LENGTH,
    );
    if (isIsoStampText({ text: instant, },)) {
      times.push(instant,);
      kept.push(tail.slice(INSTANT_LENGTH,),);
    }
    else {
      kept.push(`${CLOCK_TIME_KEY}${tail}`,);
    }
  }
  return {
    rest: kept.join('',),
    times,
  };
}

/**
 The clock times the pass lines in a text carry, in the order they are printed.

 @param text - what the pass printed

 @returns Instants, without their `at=` key

 @example
 ```ts
 const times = clockTimesIn({ text: 'DONE processed=0 of pending=0 at=2026-10-22T08:10:11.123Z', },);
 ```
 */
function clockTimesIn({ text, }: { readonly text: string; },): readonly string[] {
  /**
   The instants the text carries.
   */
  const { times, } = splitClockTimes({ text, },);
  return times;
}

/**
 What the pass printed with the clock time fields taken out, so a case run in a child process, whose clock it cannot
 fix, compares the rest of each line whole.

 @param text - what the pass printed

 @returns Text without the ` at=<instant>` fields

 @example
 ```ts
 const rest = withoutClockTimes({ text: 'DONE processed=0 of pending=0 at=2026-10-22T08:10:11.123Z', },);
 ```
 */
function withoutClockTimes({ text, }: { readonly text: string; },): string {
  /**
   The text around the fields.
   */
  const { rest, } = splitClockTimes({ text, },);
  return rest;
}

/**
 What the pass wrote to each stream and how it exited, with the places it ran in.
 */
type PassRun = {
  /**
   Exit code of the child.
   */
  readonly code: number;

  /**
   Everything the child wrote to stdout, with the clock time fields taken out (`clockTimes` holds them).
   */
  readonly stdout: string;

  /**
   Everything the child wrote to stderr.
   */
  readonly stderr: string;

  /**
   The clock times the child's pass lines carried, in the order it printed them.
   */
  readonly clockTimes: readonly string[];
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
  /**
   What the child wrote, as it wrote it.
   */
  const written = await runBuiltCommand({
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
  return {
    code: written.code,
    stdout: withoutClockTimes({ text: written.stdout, },),
    stderr: written.stderr,
    clockTimes: clockTimesIn({ text: written.stdout, },),
  };
}

/**
 The START line this build prints for the given counts and ceiling.

 @param pending - entries the pass would run

 @param hardMs - per-entry ceiling in milliseconds

 @returns Whole line, without its newline

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
  } done=0 soft=259200000ms hard=${String(hardMs,)}ms hard-scope=per-attempt call=360000ms claude-call=460000ms`;
}

/**
 The pin line the pass prints for a clone the case made.

 @param clone - throwaway corpus clone and its commit

 @returns Whole line, without its newline

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
