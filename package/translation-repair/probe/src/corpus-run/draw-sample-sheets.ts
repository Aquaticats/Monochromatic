import { writeFile, } from 'node:fs/promises';


import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  DEFAULT_PRECISION_BAR,
  DEFAULT_SAMPLE_SEED,
  formatGradingSheet,
  type GradingCandidate,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import { formatRepairSheet, } from '@monochromatic-dev/translation-repair-assembly/ts';
import { buildSampleManifest, } from '@monochromatic-dev/translation-repair-lane-contest/ts';
import type {
  ArtifactFileName,
  EligibleEntries,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  type DrawOutputs,
  trackDrawOutputs,
} from './draw-outputs.ts';
import { poolGeneration, } from '@monochromatic-dev/translation-repair-insertion/ts';
import { resolveSheetPath, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Draw sample sheets
// Resolves where a draw's three files go and writes them, all or none.

/**
 Where the three files of one draw landed.
 */
export type DrawPaths = {
  /**
   Detection grading sheet.
   */
  readonly outPath: string;

  /**
   Companion repair sheet.
   */
  readonly repairPath: string;

  /**
   Companion manifest.
   */
  readonly manifestPath: string;
};

/**
 Whether a caught value is the filesystem's answer that the path is taken.

 @param error - caught value

 @returns Whether it is an `EEXIST` of an exclusive create

 @example
 ```ts
 if (isPathTaken({ error, },)) outputs.release({ path, },);
 ```
 */
function isPathTaken({ error, }: { readonly error: unknown; },): boolean {
  return Error.isError(error,)
    && ('code' in error)
    && (error.code === 'EEXIST');
}

/**
 Writes one file of the draw, recording it first so a write that fails after
 creating it is cleaned up, and releasing it when the write found the path
 already taken, since that file is not this draw's to remove.

 @param outputs - tracker of the files this draw created

 @param path - file to write

 @param text - its whole text

 @param flag - `wx` for a final draw, `w` for a preliminary one

 @throws Whatever the write raised

 @example
 ```ts
 await landFile({ outputs, path, text, flag: 'wx', },);
 ```
 */
async function landFile(
  {
    outputs,
    path,
    text,
    flag,
  }: {
    readonly outputs: DrawOutputs;
    readonly path: string;
    readonly text: string;
    readonly flag: 'wx' | 'w';
  },
): Promise<void> {
  outputs.record({ path, },);
  try {
    await writeFile(
      path,
      text,
      { flag, },
    );
  } catch (error) {
    if (isPathTaken({ error, },))
      outputs.release({ path, },);
    throw error;
  }
}

/**
 Writes the detection sheet, the repair sheet and the manifest of one draw.

 @param runsDir - durable, gitignored output root the three files land in

 @param isFinal - whether this writes the gate sheets, created exclusively and
 removed again if any of the three fails to land

 @param drawSeed - seed the draw used, which is not the gate seed on a
 preliminary run

 @param sample - drawn sample

 @param eligible - pool the draw read, whose recorded pipeline the manifest names

 @param names - artifact file names the draw kept

 @param corpusSha - corpus commit the sheets and the manifest record

 @returns Where the three files landed

 @throws {@link GradedSheetExistsError} When a final sheet is already there,
 before any file is written

 @example
 ```ts
 const paths = await writeDrawSheets({ runsDir, isFinal, drawSeed, sample, eligible, names, corpusSha, },);
 ```
 */
export async function writeDrawSheets(
  {
    runsDir,
    isFinal,
    drawSeed,
    sample,
    eligible,
    names,
    corpusSha,
  }: {
    readonly runsDir: string;
    readonly isFinal: boolean;
    readonly drawSeed: string;
    readonly sample: readonly GradingCandidate[];
    readonly eligible: EligibleEntries;
    readonly names: readonly ArtifactFileName[];
    readonly corpusSha: string;
  },
): Promise<DrawPaths> {
  /**
   Write mode for this draw's outputs.

   Final outputs are created exclusively. `resolveSheetPath` already refuses a
   path that exists, but that check and this write are separate steps, so two
   draws racing each other can both see absence and both truncate. The whole
   purpose of the refusal is that human grades exist nowhere else, which makes
   the narrow race worth closing rather than reasoning about.
   */
  const writeFlag: 'wx' | 'w' = isFinal
    ? 'wx'
    : 'w';

  /**
   Banner marking a scratch draw, prepended to BOTH sheets so neither can be
   mistaken for the gate sheet on its contents alone.
   */
  const banner = isFinal
    ? ''
    : '> PRELIMINARY draw over whatever has settled so far, for validating the '
      + 'sheets and the pool, NOT for final grading. It is drawn with a '
      + 'different seed from the gate sheet, so it is not a preview of the gate '
      + 'draw; individual items can still coincide, since a different seed '
      + 'reorders the pool rather than excluding anything from it. The final '
      + 'draw shifts again as the pool grows.\n\n';

  // Both paths resolve BEFORE either file is written. Writing the detection
  // sheet first would leave it in place, and protected against overwrite, when
  // the repair path turns out to be refused.
  /**
   Output path, named after the draw seed so one round cannot target another
   round's sheet, and refused outright when a final sheet is already there.
   */
  const outPath = await resolveSheetPath({
    runsDir,
    seed: DEFAULT_SAMPLE_SEED,
    isFinal,
  },);

  /**
   Companion repair sheet path. The repair sheet is its own file rather than
   extra boxes on the detection sheet: a visible correction makes an alleged
   defect look more real, so folding the two together would change what the
   detection number measures and break comparison with the rounds already
   graded.
   */
  const repairPath = await resolveSheetPath({
    runsDir,
    seed: DEFAULT_SAMPLE_SEED,
    isFinal,
    kind: 'repair',
  },);

  /**
   Companion manifest path.

   Resolved with the sheets and BEFORE any write, never after. Every one of
   these throws when a final file already exists, which is the protection
   against overwriting graded work, and a path resolved after a write turns
   that protection into damage: the sheets would be replaced and the run would
   then abort, leaving a graded set half rewritten.
   */
  const manifestPath = await resolveSheetPath({
    runsDir,
    seed: DEFAULT_SAMPLE_SEED,
    isFinal,
    kind: 'manifest',
  },);

  /**
   Files this invocation creates, removed on the way out unless all three
   land.
   */
  await using outputs = trackDrawOutputs({ enabled: isFinal, },);

  // Built BEFORE either sheet, so one object is the source of the digest all
  // three files carry. Computing it twice would let the sheets and the manifest
  // disagree about the very thing that exists to prove they agree.
  /**
   Which built pipeline settled this pool.

   ONE DIGEST FOR THE WHOLE POOL, taken from the entries the draw actually
   kept. The pool refuses a mixed generation before a draw can reach it, so
   every kept entry carries the same digest and disagreement here would mean
   that guard had failed rather than that a choice was needed.
   */
  const generation = poolGeneration({
    eligible,
    names,
  },);

  /**
   What sat at each sheet position, and the fingerprint of this exact draw.
   */
  const manifest = buildSampleManifest({
    sample,
    seed: drawSeed,
    corpusSha,
    generation,
  },);

  /**
   Draw fingerprint printed into both sheet headers.

   Non-null because `buildSampleManifest` always computes one; the field is
   optional only so manifests written before the binding can still be read.
   */
  const drawDigest = nonNullishOrThrow(manifest.drawDigest,);

  // Recorded BEFORE each write, since a write can create the file and then
  // fail, and a path recorded only on success would leave that file behind.
  await landFile({
    outputs,
    path: outPath,
    text: `${banner}${
      formatGradingSheet({
        sample,
        seed: drawSeed,
        bar: DEFAULT_PRECISION_BAR,
        corpusSha,
        drawDigest,
      },)
    }`,
    flag: writeFlag,
  },);
  await landFile({
    outputs,
    path: repairPath,
    text: `${banner}${
      formatRepairSheet({
        sample,
        seed: drawSeed,
        corpusSha,
        drawDigest,
      },)
    }`,
    flag: writeFlag,
  },);

  // Written in the same breath as the sheets, because this is the only instant
  // the mapping exists. The sheets print no issue id, and re-running the draw
  // does not recover one: the draw is deterministic in its seed but not in its
  // POOL, which grows with every entry that settles. Without this file no human
  // grade can ever be joined to a machine verdict about the same item.
  await landFile({
    outputs,
    path: manifestPath,
    text: `${JSON.stringify(
      manifest,
      undefined,
      2,
    )}\n`,
    flag: writeFlag,
  },);

  // Every output landed, so the set is the complete record of one draw and
  // nothing is removed on the way out.
  outputs.commit();

  return {
    outPath,
    repairPath,
    manifestPath,
  };
}

//endregion Draw sample sheets
