import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  type PriorIssueDisclosure,
  type RepairRegion,
  runIntroducedDefectProbe,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { RelabelCase, } from './probe-relabel-case.ts';
import {
  relabelCaseLines,
  relabelClaimLines,
  relabelCounts,
} from './probe-relabel-print.ts';
import { singleRegionTally, } from './probe-single-tally.ts';
import {
  RUN_MODELS,
  RUN_PER_CALL_TIMEOUT_MS,
} from '@monochromatic-dev/translation-repair-run/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Probe relabel pair
// Asks the introduced-defect probe about one region under the three
// conditions the relabel runner compares, and prints what each said.

/**
 Disclosure the other prompt sends, for the arm that measures it.

 @param production - disclosure production sends

 @returns Disclosure production does not send

 @example
 ```ts
 const other = otherDisclosure({ production: 'withheld', },); // 'rendered'
 ```
 */
export function otherDisclosure({ production, }: { readonly production: PriorIssueDisclosure; },): PriorIssueDisclosure {
  return (production === 'rendered') ? 'withheld' : 'rendered';
}

/**
 Runs one probe call over one region and returns a printable tally.

 @param region - region under test

 @param issues - accepted issues the region was cut for; empty when nothing is known

 @param disclosure - whether the list is written into the prompt or only known to the screen

 @param sourceText - slice original

 @param baselineText - slice translation before replacement

 @param client - client this one call asks through

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns One line of counts

 @example
 ```ts
 const line = await probeOnce({ region, issues: [], disclosure: 'withheld', sourceText, baselineText, client, stageSettings, },);
 ```
 */
async function probeOnce(
  {
    region,
    issues,
    disclosure,
    sourceText,
    baselineText,
    client,
    stageSettings,
  }: {
    readonly region: RepairRegion;
    readonly issues: readonly AdjudicatedIssue[];
    readonly disclosure: PriorIssueDisclosure;
    readonly sourceText: string;
    readonly baselineText: string;
    readonly client: ModelClient;
    readonly stageSettings: StageSettings;
  },
): Promise<string> {
  /**
   Report for this single region.
   */
  const report = await runIntroducedDefectProbe({
    client,
    proberModelIds: RUN_MODELS.checkerModelIds,
    sourceText,
    baselineText,
    regions: [region,],
    issues,
    identityContext: '',
    disclosure,
    signal: new AbortController().signal,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l: tagged({ tag: 'probe-relabel', },),
    stageSettings,
  },);

  /**
   Screened tally of the single region.
   */
  const tally = singleRegionTally({ report, },);
  for (const line of relabelClaimLines({ tally, },))
    console.log(line,);

  return relabelCounts({
    report,
    tally,
  },);
}

/**
 Probes one case under both conditions and prints the pair.

 @param relabelCase - rebuilt damaged-region case

 @param production - disclosure production sends

 @param newClient - builds the client each of the three probes asks through:
 ONE FRESH CLIENT PER PROBE, since a client reuses the reply to a prompt it
 has already been asked, and a control region asked twice with no issues sends
 the same prompt twice

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @example
 ```ts
 await probePair({ relabelCase, production: 'withheld', newClient, stageSettings, },);
 ```
 */
export async function probePair(
  {
    relabelCase,
    production,
    newClient,
    stageSettings,
  }: {
    readonly relabelCase: RelabelCase;
    readonly production: PriorIssueDisclosure;
    readonly newClient: () => ModelClient;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  for (const line of relabelCaseLines({ relabelCase, },))
    console.log(line,);

  /**
   Disclosure the other prompt sends.
   */
  const other = otherDisclosure({ production, },);

  /**
   Production condition: the list under the disclosure the pass sends.
   */
  const productionLine = await probeOnce({
    region: relabelCase.region,
    issues: relabelCase.issues,
    disclosure: production,
    sourceText: relabelCase.sourceText,
    baselineText: relabelCase.baselineText,
    client: newClient(),
    stageSettings,
  },);
  console.log(`  issues-${production} ${productionLine}`,);

  /**
   The other prompt: the same list under the disclosure production does not send.
   */
  const otherPrompt = await probeOnce({
    region: relabelCase.region,
    issues: relabelCase.issues,
    disclosure: other,
    sourceText: relabelCase.sourceText,
    baselineText: relabelCase.baselineText,
    client: newClient(),
    stageSettings,
  },);
  console.log(`  issues-${other} ${otherPrompt}`,);

  /**
   Counterfactual: nothing known, so nothing rendered and nothing screened.

   WITHHELD WHATEVER PRODUCTION SENDS: under the rendered disclosure an empty
   list still writes its heading into the prompt, and an arm that sends the
   heading is not an arm with no list known.
   */
  const absent = await probeOnce({
    region: relabelCase.region,
    issues: [],
    disclosure: 'withheld',
    sourceText: relabelCase.sourceText,
    baselineText: relabelCase.baselineText,
    client: newClient(),
    stageSettings,
  },);
  console.log(`  issues-absent ${absent}`,);
}

//endregion Probe relabel pair
