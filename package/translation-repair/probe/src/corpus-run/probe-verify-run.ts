import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  gatherRelabelCases,
  RelabelCase,
} from './probe-relabel-case.ts';
import type { gatherControlCases, } from './probe-relabel-control.ts';
import { collectFlagged, } from './probe-verify-collect.ts';
import {
  formatVerifyManifest,
  formatVerifySheet,
} from './probe-verify-sheet.ts';
import {
  assertSheetPairFree,
  writeSheetPair,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Probe verify run
// Gathers the damaged and control regions, probes each with the issues
// withheld, and writes the blind sheet and its manifest, which is the whole of
// the probe verify runner once the process has handed it a runs directory, a
// pin and a way to build a client.

/**
 Name of the sample manifest the damaged positions index into, in the runs
 directory.
 */
const MANIFEST_NAME = 'sample-manifest-milestone-three-precision-round-three.json';

/**
 Name of the blind sheet, in the runs directory.
 */
const SHEET_NAME = 'probe-verify-sheet.md';

/**
 Name of the scoring manifest, in the runs directory.
 */
const MANIFEST_OUT_NAME = 'probe-verify-manifest.json';

/**
 Opening line saying how many regions of each set will be probed.

 @param damaged - regions a human read as damaged

 @param controls - regions from the same entries that nobody read

 @returns The line, without a newline

 @example
 ```ts
 console.log(verifyOpening({ damaged, controls, },),);
 ```
 */
export function verifyOpening(
  {
    damaged,
    controls,
  }: {
    readonly damaged: readonly RelabelCase[];
    readonly controls: readonly RelabelCase[];
  },
): string {
  return `VERIFY probing ${String(damaged.length,)} damaged and ${
    String(controls.length,)
  } control ${
    wordForCount({
      count: controls.length,
      one: 'region',
      many: 'regions',
    },)
  }, issues withheld`;
}

/**
 Line saying how many items the sheet holds and where it was written.

 @param count - items on the sheet

 @param dir - directory the sheet landed in

 @returns The line, without a newline

 @example
 ```ts
 console.log(verifyWrote({ count: 2, dir: '/runs', },),);
 ```
 */
export function verifyWrote(
  {
    count,
    dir,
  }: {
    readonly count: number;
    readonly dir: string;
  },
): string {
  return `VERIFY wrote ${String(count,)} ${
    wordForCount({
      count,
      one: 'item',
      many: 'items',
    },)
  } to ${dir}/${SHEET_NAME}`;
}

/**
 Note closing the run, which keeps the grader away from the answer.
 */
export const VERIFY_BLIND_NOTE: string = 'NOTE the sheet is blind and its manifest is not. Grade the sheet without '
  + 'opening the manifest, or the answer stops meaning anything.';

/**
 Builds the blind verification sheet and its scoring manifest.

 @param dir - runs directory holding the sample manifest and the settled
 artifacts the gatherers read, and receiving the pair

 @param pin - corpus commit the regions' pages are read at

 @param newClient - builds the client each region is asked through

 @param gather - the two gatherers, which read the settled artifacts and the
 corpus pages: `gatherRelabelCases` and `gatherControlCases` in a run

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @throws {@link StatedRefusalError} when the sheet or its manifest is already in
 the directory, before anything is gathered or asked

 @throws Whatever a gatherer raises, a sample manifest that will not read
 included, since nothing has been asked by then

 @example
 ```ts
 await runProbeVerify({
   dir,
   pin,
   newClient: createProbeRunClient,
   gather: { damaged: gatherRelabelCases, controls: gatherControlCases, },
   stageSettings,
 },);
 ```
 */
export async function runProbeVerify(
  {
    dir,
    pin,
    newClient,
    gather,
    stageSettings,
  }: {
    readonly dir: string;
    readonly pin: CorpusPin;
    readonly newClient: () => ModelClient;
    readonly gather: {
      readonly damaged: typeof gatherRelabelCases;
      readonly controls: typeof gatherControlCases;
    };
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  // BEFORE ANY REGION IS ASKED: the pair is refused at the write when either
  // file is already there, and by then every region has been paid for.
  await assertSheetPairFree({
    dir,
    sheetName: SHEET_NAME,
    manifestName: MANIFEST_OUT_NAME,
  },);

  /**
   Manifest the damaged positions index into.
   */
  const manifestPath = `${dir}/${MANIFEST_NAME}`;

  /**
   Regions a human read as damaged.
   */
  const damaged = await gather.damaged({
    manifestPath,
    pin,
    runsDir: dir,
  },);

  /**
   Regions from the same entries that nobody read.
   */
  const controls = await gather.controls({
    manifestPath,
    damaged,
    pin,
    runsDir: dir,
  },);
  console.log(verifyOpening({
    damaged,
    controls,
  },),);

  /**
   Flagged regions from both sets.
   */
  const items = [
    ...await collectFlagged({
      cases: damaged,
      kind: 'damaged',
      newClient,
      stageSettings,
    },),
    ...await collectFlagged({
      cases: controls,
      kind: 'control',
      newClient,
      stageSettings,
    },),
  ];

  await writeSheetPair({
    dir,
    sheetName: SHEET_NAME,
    manifestName: MANIFEST_OUT_NAME,
    sheet: formatVerifySheet({ items, },),
    manifest: formatVerifyManifest({ items, },),
  },);

  console.log(verifyWrote({
    count: items.length,
    dir,
  },),);
  console.log(VERIFY_BLIND_NOTE,);
}

//endregion Probe verify run
