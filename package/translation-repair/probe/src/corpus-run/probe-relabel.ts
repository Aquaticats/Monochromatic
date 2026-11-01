import {
  createProbeRunClient,
  readRunCorpusPinSetting,
  reportingRefusals,
  resolveRunsDir,
} from '@monochromatic-dev/translation-repair-run/ts';
import { PRODUCTION_PRIOR_ISSUE_DISCLOSURE, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import { gatherRelabelCases, } from './probe-relabel-case.ts';
import { gatherControlCases, } from './probe-relabel-control.ts';
import { runProbeRelabel, } from './probe-relabel-run.ts';
import { readStageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Probe relabel
// Asks the introduced-defect probe about regions a HUMAN read as damaged, three
// times each: under the disclosure production sends, under the other prompt,
// and with no accepted issue known at all.
//
// The cat fixtures established that the probe reports damage 3/3 on a deleted
// clause the source supports, loses one voice of three when a false accepted
// issue names that clause, and stays correctly silent when the deletion really
// is licensed. So the instrument works and the labelling effect is modest,
// which leaves the corpus result unexplained: 2438 of 2571 prober verdicts
// found nothing, including on every one of these regions.
//
// The first run answered that: withholding the list took the damaged regions
// from 0 of 15 prober verdicts raising anything to 7 of 15, with every region
// drawing at least one admissible claim. The fixture understated the effect
// because it carried ONE prior issue and these regions carry six to seventeen.
//
// That run was made while the probe still rendered the list by default.
// Production has since withheld it and moved the excusing to the screen
// (`introduced-defect-wire.ts`), and for a while the two arms here relied on
// that default, so both sent the same prompt and differed only in the screen,
// under labels that said otherwise. Every arm now names its
// disclosure, and the third arm, no list at all, separates the screen's effect
// from the prompt's.
//
// That alone proves nothing, which is why the control arm exists. Every damaged
// region is damaged by construction, so no claim raised there could be wrong,
// and an unlabelled prober reporting the PRE-EXISTING defect would look exactly
// like one finding the damage. Regions from the same entries that the reader did
// NOT flag separate the two: a withheld arm equally loud there is re-reporting
// old defects and the finding collapses.
//
// Reads corpus text through git at the pinned commit and writes nothing.

/**
 Hands the run's directory, pin, disclosure and client builder to the relabel.

 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  /**
   Corpus clone and commit this run reads, after any environment
   override, read inside the refusal boundary (ledger B302).
   */
  const { pin, } = readRunCorpusPinSetting({ env: process.env, },);
  await runProbeRelabel({
    dir: await resolveRunsDir(),
    pin,
    production: PRODUCTION_PRIOR_ISSUE_DISCLOSURE,
    newClient: createProbeRunClient,
    gather: {
      damaged: gatherRelabelCases,
      controls: gatherControlCases,
    },
    stageSettings: readStageSettings({ env: process.env, },),
  },);
}

// Guarded so this runs only when INVOKED. Unguarded it ran on IMPORT, so
// anything pulling this module into the bundle performed the whole task as a
// side effect of loading the library: for the probing scripts that means live
// model calls, and for every one of them it means writing files.
if (import.meta.main)
  await reportingRefusals({
    what: 'probe-relabel',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Probe relabel
