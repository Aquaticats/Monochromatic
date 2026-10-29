import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { openRouterIdFor, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  type CommandLineOf,
  wholeNumberFlag,
  writtenOr,
} from '@monochromatic-dev/translation-repair-run/ts';
import {
  ROSTER_MODEL_IDS,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  rendersAsNothing,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';

//region Reader measure ask
// THE COMMAND LINE OF `reader-measure`, kept apart from the entry so the entry
// module exports nothing (an exported entry is bundled as a shared chunk and
// `import.meta.main` is then false in it; see `roster-card-ask.ts`).
//
// usage: mise run //package/translation-repair/bench:reader-measure -- --trusted <runs dir> --pictures <people dir>
//   [--trusted <runs dir> ...] [--model <seat id>] [--sample <count>]

/**
 Candidate measured when `--model` is left off: the Gemma size the owner chose
 to try on 2026-10-21.
 */
const DEFAULT_MODEL = 'gemma-4-26b-a4b-it';

/**
 Matched pictures asked when `--sample` is left off: the nine the 2026-09-08
 measurement used.
 */
const DEFAULT_SAMPLE = 9;

/**
 Environment variable the key is read from.
 */
const KEY_VARIABLE = 'TRANSLATION_REPAIR_OPENROUTER_API_KEY';

/**
 What the command measures, and with what.
 */
export type MeasureAsk = {
  readonly modelId: RosterModelId;
  readonly trustedDirs: readonly string[];
  readonly picturesDir: string;
  readonly sample: number;
  readonly apiKey: string;
};

/**
 Reads the measurement's inputs off the command line and environment.

 @param line - command line, read whole by `reportingRefusals`

 @param env - environment the key is read from

 @returns Model, directories, sample and key

 @throws {@link StatedRefusalError} When a trusted directory, the pictures
 directory or the key is missing, or the model is not a roster model that
 OpenRouter serves, naming each part that is wrong

 @example
 ```ts
 const ask = readMeasureAsk({ line, env: process.env, },);
 ```
 */
export function readMeasureAsk(
  {
    line,
    env,
  }: {
    readonly line: CommandLineOf<'reader-measure'>;
    readonly env: Readonly<NodeJS.ProcessEnv>;
  },
): MeasureAsk {
  /**
   Model as written.
   */
  const modelWritten = writtenOr({
    asked: line.flag('model',),
    unwritten: DEFAULT_MODEL,
  },);

  /**
   Roster model the text names, when it names one.
   */
  const rosterId = ROSTER_MODEL_IDS.find(function isNamed(candidate,): boolean {
    return candidate === modelWritten;
  },);

  /**
   Directories holding the trusted readings.
   */
  const trustedDirs = line.list('trusted',);

  /**
   Directory holding the people directories whose pictures are tried.
   */
  const picturesDir = writtenOr({
    asked: line.flag('pictures',),
    unwritten: '',
  },);

  /**
   Key as the environment holds it.
   */
  const apiKey = env[KEY_VARIABLE] ?? '';

  /**
   What is wrong with each part, empty when nothing is.
   */
  const problems = [
    ...((rosterId !== undefined)
      && openRouterIdFor({ modelId: rosterId, },)
        .served
      ? []
      : [`--model ${JSON.stringify(modelWritten,)} is not a roster model that OpenRouter serves`,]),
    ...((trustedDirs.length === 0)
      ? ['--trusted names no directory holding the readings the pipeline already trusts',]
      : []),
    ...(rendersAsNothing({ text: picturesDir, },) ? ['--pictures names no directory holding the pictures to try',] : []),
    ...(rendersAsNothing({ text: apiKey, },) ? [`${KEY_VARIABLE} is not set`,] : []),
  ];
  if (problems.length > 0) {
    throw new StatedRefusalError({
      says: `reader-measure cannot start: ${problems.join('; ',)}`,
    },);
  }
  return {
    modelId: nonNullishOrThrow(rosterId,),
    trustedDirs,
    picturesDir,
    sample: wholeNumberFlag({
      asked: line.flag('sample',),
      unwritten: DEFAULT_SAMPLE,
      leaveOffTo: `ask ${String(DEFAULT_SAMPLE,)} ${
        wordForCount({
          count: DEFAULT_SAMPLE,
          one: 'matched picture',
          many: 'matched pictures',
        },)
      }`,
    },),
    apiKey,
  };
}

//endregion Reader measure ask
