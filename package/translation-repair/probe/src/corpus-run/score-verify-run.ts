import { parseGradedSheet, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { readSheetText, } from './score-sheet-text.ts';
import { readVerifyManifest, } from './score-verify-manifest.ts';
import {
  tallyByKind,
  verifyKindLines,
} from './score-verify-tally.ts';

//region Score verify run
// Joins the blind verification grades to the manifest that says which set each
// item came from, and reports what the unlabelled probe's flags are worth.
//
// The join is BY POSITION, the same as every other sheet here, because the
// sheet prints no ids on purpose. The manifest is written in the same instant
// as the sheet by `probe-verify.ts`, from one ordering, so position is exact
// rather than assumed.
//
// What the numbers mean. A flag on a region a reader already called damaged is
// the probe agreeing with a known answer, which measures little. A flag on a
// region nobody had read is the whole question: graded Y it is damage the
// sample missed, and graded N it is an invention, and the split between them is
// the precision any gating decision has to live with.

/**
 Reports what the graded sheet says about the unlabelled probe.

 @param runsDir - run artifact root holding the sheet and its manifest

 @param basename - which sheet to score, so one scorer serves every sheet this
 formatter writes rather than each sheet growing its own

 @throws {@link StatedRefusalError} when the sheet cannot be read or its length differs from the
 manifest's

 @example
 ```ts
 await printVerifyScore({ runsDir, basename: 'probe-verify', },);
 ```
 */
export async function printVerifyScore(
  {
    runsDir,
    basename,
  }: {
    readonly runsDir: string;
    readonly basename: string;
  },
): Promise<void> {
  /**
   Graded sheet items, in sheet order.
   */
  const graded = parseGradedSheet({
    text: await readSheetText({
      path: `${runsDir}/${basename}-sheet.md`,
      label: 'graded sheet',
      remedy: 'name the runs directory that holds it with TRANSLATION_REPAIR_RUNS_DIR, or the sheet with '
        + 'VERIFY_SHEET_BASENAME',
    },),
  },);

  /**
   Manifest rows, in the same order.
   */
  const manifest = await readVerifyManifest({
    path: `${runsDir}/${basename}-manifest.json`,
  },);

  /**
   Tally per set, which is empty only where the sheet and the manifest are.
   */
  const tallies = tallyByKind({
    graded,
    manifest,
  },);
  if (tallies.size === 0)
    console.log('NONE the sheet and the manifest hold no item, so there is no set to report',);
  for (const line of verifyKindLines({ tallies, },))
    console.log(line,);

  console.log(
    'NOTE read the labels as the drawing task meant them. On the verification '
      + 'sheet they name what a READER already believed, so the control line is '
      + 'the one that decides gating. On the damage sample they name what the '
      + 'PROBE said, so probe-flagged carries its precision and probe-silent '
      + 'carries its MISSES: a Y there is damage the probe did not see.',
  );
}

//endregion Score verify run
