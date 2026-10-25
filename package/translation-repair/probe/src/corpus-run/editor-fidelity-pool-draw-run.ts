import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type { Logger, } from '@monochromatic-dev/module-logger/ts';
import {
  type CommandLineOf,
  type FlagValue,
  wholeNumberFlag,
} from '@monochromatic-dev/translation-repair-run/ts';
import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  EDITOR_DAMAGE_KINDS,
  editorFidelityTrials,
} from '../editor-fidelity-trials.ts';
import {
  DEFAULT_POOL_SEED,
  POOL_DRAWN_ON,
  POOL_TRIALS_WANTED,
  SPECS_PER_PART,
} from '../editor-fidelity-pool-rule.ts';
import {
  type PoolFile,
  poolSourceFiles,
} from '../editor-fidelity-pool-source.ts';
import type {
  FidelityReferenceSpec,
  ReviewedFidelityReference,
} from '../fidelity-reference-model.ts';
import {
  type EntryPages,
  drawPool,
  type PoolDraw,
} from './editor-fidelity-pool-selection.ts';
import { scoreableVerdict, } from './editor-fidelity-pool-verdict.ts';

//region Editor fidelity pool draw run
// THE RUN OF THE POOL DRAW, over parts it is handed: the listing of the clone's entries, the reader of an entry's pages,
// the reader and writer of the drawn files, the committed specifications and the hash-verifying reference reader. Nothing
// here touches the clone, the disk or a provider itself, so every line it prints and every refusal it makes is tested over
// scripted parts. `editor-fidelity-pool-draw.ts` hands it the real ones.

/**
 What the run is handed.
 */
export type PoolDrawRunInput = {
  /**
   The command line, read whole.
   */
  readonly line: CommandLineOf<'editor-fidelity-pool-draw'>;

  /**
   Corpus pin the pages are read at.
   */
  readonly pin: CorpusPin;

  /**
   Lists every entry id of the clone.
   */
  readonly listEntries: () => Promise<readonly string[]>;

  /**
   Reads the two pages of an entry.
   */
  readonly readEntry: (input: { readonly entryId: string; },) => Promise<EntryPages>;

  /**
   Reads the files of a directory that the draw writes, by name.
   */
  readonly readFiles: (input: { readonly dir: string; },) => Promise<readonly PoolFile[]>;

  /**
   Writes files into a directory.
   */
  readonly writeFiles: (input: {
    readonly dir: string;
    readonly files: readonly PoolFile[]
  },) => Promise<void>;

  /**
   Specifications the committed pool holds, reviewed ones first, which a check replays against the clone.
   */
  readonly committed: readonly FidelityReferenceSpec[];

  /**
   Entries of the reviewed references, which the draw leaves out.
   */
  readonly reviewedEntryIds: readonly string[];

  /**
   Trials the reviewed references hold, which count toward the trials wanted.
   */
  readonly reviewedTrials: number;

  /**
   Reads specifications against the clone and verifies every hash.
   */
  readonly readReferences: (
    input: {
      readonly pin: CorpusPin;
      readonly specs: readonly FidelityReferenceSpec[]
    },
  ) => Promise<readonly ReviewedFidelityReference[]>;

  /**
   Where the run writes its lines.
   */
  readonly log: Logger;
};

/**
 Writes a count with the noun it takes.

 @param count - how many

 @param one - noun at exactly one

 @param many - noun at any other count

 @returns The count, a space and the noun

 @example
 ```ts
 const written = counted({ count: 2, one: 'trial', many: 'trials', },);
 ```
 */
function counted(
  {
    count,
    one,
    many,
  }: {
    readonly count: number;
    readonly one: string;
    readonly many: string;
  },
): string {
  return `${String(count,)} ${wordForCount({
    count,
    one,
    many,
  },)}`;
}

/**
 Reads the seed the operator wrote, else the seed of the pool the fourth round froze.

 @param asked - what `--seed` carried

 @returns The seed

 @example
 ```ts
 const seed = seedOf({ asked: line.flag('seed',), },);
 ```
 */
function seedOf({ asked, }: { readonly asked: FlagValue; },): string {
  return (asked.kind === 'written') ? asked.value : DEFAULT_POOL_SEED;
}

/**
 Says the smallest, middle and largest of the sheet sizes drawn.

 @param sheetChars - characters of each sheet taken

 @returns The line, without a newline

 @example
 ```ts
 const line = sheetLine({ sheetChars: [900, 1_100, 1_400], },);
 ```
 */
function sheetLine({ sheetChars, }: { readonly sheetChars: readonly number[]; },): string {
  /**
   The sizes, smallest first.
   */
  const sorted = sheetChars.toSorted(function ascending(
    left,
    right,
  ): number {
    return left - right;
  },);
  return `draw: sheet chars min ${String(sorted[0] ?? 0,)}, median ${String(sorted[Math.floor(sorted.length / 2,)] ?? 0,)}, max ${
    String(sorted.at(-1,) ?? 0,)
  }`;
}

/**
 Logs what a draw took and what it left out.

 @param log - where the lines go

 @param draw - the draw

 @param seed - seed the draw was made under

 @param want - trials the pool should hold

 @param reviewedTrials - trials the reviewed references hold

 @param listed - entries the clone listed

 @example
 ```ts
 logDraw({ log, draw, seed: 'round-4', want: 40, reviewedTrials: 4, listed: 120, },);
 ```
 */
function logDraw(
  {
    log,
    draw,
    seed,
    want,
    reviewedTrials,
    listed,
  }: {
    readonly log: Logger;
    readonly draw: PoolDraw;
    readonly seed: string;
    readonly want: number;
    readonly reviewedTrials: number;
    readonly listed: number;
  },
): void {
  log.info(`draw: seed \`${seed}\`, ${counted({
    count: want,
    one: 'trial',
    many: 'trials',
  },)} wanted (${String(reviewedTrials,)} reviewed counted), ${
    counted({
      count: listed,
      one: 'entry',
      many: 'entries',
    },)
  } listed`,);
  log.info(`draw: eligible entries ${String(draw.eligibleEntries,)}, drawn entries ${String(draw.drawnEntries
    .length,)}, trials ${
    String(draw.trials,)
  } (insertion ${String(draw.kinds
    .insertion,)}, alteration ${String(draw.kinds
      .alteration,)})`,);
  log.info(sheetLine({ sheetChars: draw.sheetChars, },),);
  Object.keys(draw.skipped,)
    .toSorted(function byCodePoints(
      left,
      right,
    ): number {
      return compareCodePoints({
        left,
        right,
      },);
    },)
    .forEach(function say(reason,): void {
      log.info(`draw: left out ${String(draw.skipped[reason],)} for ${reason}`,);
    },);
}

/**
 Names the files whose text differs from the fresh draw, or that one side lacks.

 @param fresh - files the draw gives

 @param onDisk - files found in the directory

 @returns The names, sorted

 @example
 ```ts
 const differing = differingNames({ fresh, onDisk, },);
 ```
 */
function differingNames(
  {
    fresh,
    onDisk,
  }: {
    readonly fresh: readonly PoolFile[];
    readonly onDisk: readonly PoolFile[];
  },
): readonly string[] {
  /**
   Names on either side.
   */
  const names = new Set([
    ...fresh,
    ...onDisk,
  ].map(function nameOf(file,): string {
    return file.name;
  },),);
  return [...names,].filter(function differs(name,): boolean {
    return fresh.find(function isIt(file,): boolean {
      return file.name === name;
    },)
      ?.text
      !== onDisk.find(function isIt(file,): boolean {
        return file.name === name;
      },)
        ?.text;
  },)
    .toSorted(function byCodePoints(
      left,
      right,
    ): number {
      return compareCodePoints({
        left,
        right,
      },);
    },);
}

/**
 Replays the committed specifications against the clone: reads every reference through the hash-verifying reader, builds
 its trials and runs the scoreability check on each.

 @param input - what the run was handed

 @example
 ```ts
 await replayCommitted({ input, },);
 ```
 */
async function replayCommitted({ input, }: { readonly input: PoolDrawRunInput; },): Promise<void> {
  /**
   References read and verified.
   */
  const references = await input.readReferences({
    pin: input.pin,
    specs: input.committed,
  },);
  /**
   Trials built from them.
   */
  const trials = editorFidelityTrials({
    references,
    damageKinds: EDITOR_DAMAGE_KINDS,
  },);
  /**
   One verdict per trial, the empty text for a trial that is scoreable.
   */
  const verdicts = await allInInputOrder({
    members: trials.map(function verdictOf(trial,): Promise<string> {
      return scoreableVerdict({ trial, },);
    },),
  },);
  /**
   Trials that failed the check, each with its reason.
   */
  const failing = trials.flatMap(function failed(
    trial,
    index,
  ): readonly string[] {
    return verdicts[index] === ''
      ? []
      : [`${trial.spec
        .id} ${trial.damageKind}: ${verdicts[index] ?? ''}`,];
  },);
  input.log
    .info(`replay: ${String(references.length,)} ${
    wordForCount({
      count: references.length,
      one: 'reference',
      many: 'references',
    },)
  } read and verified, ${String(trials.length,)} ${
    wordForCount({
      count: trials.length,
      one: 'trial',
      many: 'trials',
    },)
  }, ${String(trials.length - failing.length,)} scoreable`,);
  failing.forEach(function say(reason,): void {
    input.log
      .warn(`replay: not scoreable: ${reason}`,);
  },);
}

/**
 Runs the pool draw: draws, then writes the parts to `--out`, or with `--check` compares them with the files there and
 replays the committed pool against the clone.

 @param input - what the run is handed

 @throws StatedRefusalError when `--check` has no `--out` to compare with, or the files there differ from a fresh draw

 @example
 ```ts
 await runEditorFidelityPoolDraw({ line, pin, listEntries, readEntry, readFiles, writeFiles, committed, reviewedEntryIds, reviewedTrials, readReferences, log, },);
 ```
 */
export async function runEditorFidelityPoolDraw(input: PoolDrawRunInput,): Promise<void> {
  /**
   The command line, read whole.
   */
  const { line, } = input;
  /**
   Where the run writes its lines.
   */
  const { log, } = input;
  /**
   Directory the parts are written to or checked in.
   */
  const out = line.flag('out',);
  if (line.switched('check',) && (out.kind === 'unwritten')) {
    throw new StatedRefusalError({
      says: '--check compares the draw with the files in --out, and --out was not written; write the directory the '
        + 'drawn files sit in',
    },);
  }
  /**
   Seed of the draw.
   */
  const seed = seedOf({ asked: line.flag('seed',), },);
  /**
   Trials the pool should hold.
   */
  const want = wholeNumberFlag({
    asked: line.flag('want',),
    unwritten: POOL_TRIALS_WANTED,
    leaveOffTo: `draw ${counted({
      count: POOL_TRIALS_WANTED,
      one: 'trial',
      many: 'trials',
    },)}`,
  },);
  /**
   Entries of the clone.
   */
  const entryIds = await input.listEntries();
  /**
   What the draw took.
   */
  const draw = await drawPool({
    entryIds,
    readEntry: input.readEntry,
    seed,
    want,
    reviewedTrials: input.reviewedTrials,
    reviewedEntryIds: input.reviewedEntryIds,
    corpusSha: input.pin
      .commitSha,
    reviewedOn: POOL_DRAWN_ON,
  },);
  logDraw({
    log,
    draw,
    seed,
    want,
    reviewedTrials: input.reviewedTrials,
    listed: entryIds.length,
  },);
  if (out.kind === 'unwritten')
    return;
  /**
   The files of the draw.
   */
  const files = poolSourceFiles({
    specs: draw.specs,
    seed,
    perPart: SPECS_PER_PART,
  },);
  if (!line.switched('check',)) {
    await input.writeFiles({
      dir: out.value,
      files,
    },);
    log.info(`wrote ${String(files.length,)} ${
      wordForCount({
        count: files.length,
        one: 'file',
        many: 'files',
      },)
    } to ${out.value}`,);
    return;
  }
  /**
   Names of the files that are not what a fresh draw gives.
   */
  const differing = differingNames({
    fresh: files,
    onDisk: await input.readFiles({ dir: out.value, },),
  },);
  if (differing.length > 0) {
    throw new StatedRefusalError({
      says: `the files in ${out.value} are not what a fresh draw of seed \`${seed}\` gives: ${
        differing.join(', ',)
      } differ or are missing; run the draw with --out ${out.value} and without --check to write them again, or draw with the seed the files were written under`,
    },);
  }
  log.info(`check: ${counted({
    count: files.length,
    one: 'file',
    many: 'files',
  },)} equal a fresh draw`,);
  if (input.committed
    .length
    > 0)
    await replayCommitted({ input, },);
}

//endregion Editor fidelity pool draw run
