import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { PriorIssueDisclosure, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { gatherRelabelCases, } from './probe-relabel-case.ts';
import type { gatherControlCases, } from './probe-relabel-control.ts';
import {
  otherDisclosure,
  probePair,
} from './probe-relabel-pair.ts';
import {
  relabelGathered,
  relabelNotes,
  relabelRebuilt,
} from './probe-relabel-print.ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Probe relabel run
// Rebuilds the damaged and the control regions and probes each under the
// three conditions, which is the whole of the probe relabel runner once the
// process has handed it a runs directory, a pin and a way to build a client.

/**
 Rebuilds every damaged-region case and probes each under both conditions.

 @param dir - runs directory holding the sample manifest and the settled
 artifacts the gatherers read

 @param pin - corpus commit the regions' pages are read at

 @param production - disclosure production sends

 @param newClient - builds the client each probe asks through

 @param gather - the two gatherers, which read the settled artifacts and the
 corpus pages: `gatherRelabelCases` and `gatherControlCases` in a run

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @throws Whatever a gatherer raises, a sample manifest that will not read
 included, since nothing has been asked by then

 @example
 ```ts
 await runProbeRelabel({
   dir,
   pin,
   production: 'withheld',
   newClient: createProbeRunClient,
   gather: { damaged: gatherRelabelCases, controls: gatherControlCases, },
   stageSettings,
 },);
 ```
 */
export async function runProbeRelabel(
  {
    dir,
    pin,
    production,
    newClient,
    gather,
    stageSettings,
  }: {
    readonly dir: string;
    readonly pin: CorpusPin;
    readonly production: PriorIssueDisclosure;
    readonly newClient: () => ModelClient;
    readonly gather: {
      readonly damaged: typeof gatherRelabelCases;
      readonly controls: typeof gatherControlCases;
    };
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  /**
   Manifest the damaged positions index into.
   */
  const manifestPath = `${dir}/sample-manifest-milestone-three-precision-round-three.json`;

  /**
   Damaged regions rebuilt from the round-three draw.
   */
  const cases = await gather.damaged({
    manifestPath,
    pin,
    runsDir: dir,
  },);
  console.log(relabelRebuilt({ count: cases.length, },),);

  /**
   Regions from the same entries that the reader did NOT flag.

   The arm that decides whether the damaged result means anything: every
   damaged region is damaged by construction, so no claim raised there could
   be wrong, and only unflagged regions can show whether the withheld arm is
   detecting damage or re-reporting the defect the region was cut for.
   */
  const controls = await gather.controls({
    manifestPath,
    damaged: cases,
    pin,
    runsDir: dir,
  },);
  console.log(relabelGathered({ count: controls.length, },),);

  // Sequential so this never competes with a running corpus pass for the
  // per-model stream slots.
  /* oxlint-disable no-await-in-loop -- sequential by design, see comment */
  for (const relabelCase of cases)
    await probePair({
      relabelCase,
      production,
      newClient,
      stageSettings,
    },);
  for (const relabelCase of controls)
    await probePair({
      relabelCase,
      production,
      newClient,
      stageSettings,
    },);
  /* oxlint-enable no-await-in-loop */

  for (
    const note of relabelNotes({
      production,
      other: otherDisclosure({ production, },),
    },)
  )
    console.log(note,);
}

//endregion Probe relabel run
