import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { BenchSeating, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  attestCitedReferences,
  type ReferenceAttestation,
} from '@monochromatic-dev/translation-repair-reference/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Pass reference attestation
// THE PREPARATION'S ATTESTATION ROUND, split out of `preparePassEntry` so a
// test can drive the roster it is asked of (ledger X12): no fixture reaches
// that function's cited references, which it reads off the network.

/**
 Attestation of an original that links nowhere: nothing asked, nothing said.
 */
const NO_ATTESTATION: ReferenceAttestation = {
  details: [],
  lines: [],
  findings: [],
};

/**
 Archive details a reference states, attested by the roster the hook hands
 over (class thirty-seven, 2026-09-16; ledger X12, 2026-09-28): the repair
 lane screens addition claims against them before the panel, and every sheet
 reads them as ATTESTED lines under the references.

 THE HOOK IS READ ONLY WHEN THE ROUND IS ASKED, since an original linking
 nowhere asks nobody and so has no bench to re-seat.

 @param client - injected model client

 @param modelIds - roster the preparation started on

 @param beforeItem - hook handing the round its roster, `keepBench` for a
 caller with none; required so no caller drops it by omission

 @param sourceText - original

 @param archiveText - archive as the preparation reads it

 @param referenceLines - what the pages the original links say, empty when it
 links nowhere

 @param signal - entry deadline and caller abort

 @param exchangeTimeoutMs - per-call bound

 @param l - entry logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Attested details, their sheet lines and the round's findings

 @example
 ```ts
 const attestation = await attestPassReferences({ client, modelIds, beforeItem, sourceText, archiveText, referenceLines, signal, exchangeTimeoutMs, l, stageSettings, },);
 ```
 */
export async function attestPassReferences(
  {
    client,
    modelIds,
    beforeItem,
    sourceText,
    archiveText,
    referenceLines,
    signal,
    exchangeTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly beforeItem: () => Promise<BenchSeating>;
    readonly sourceText: string;
    readonly archiveText: string;
    readonly referenceLines: string;
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<ReferenceAttestation> {
  if (referenceLines === '') {
    l.debug(`${attestPassReferences.name}: the original links nowhere, so nobody is asked`,);
    return NO_ATTESTATION;
  }
  /**
   Seating the hook hands the round: a roster read under a hold, or none,
   which keeps the one the preparation started on.
   */
  const seating = await beforeItem();
  /**
   Roster the round is asked of.
   */
  const roster = seating.modelIds ?? modelIds;
  l.debug(`${attestPassReferences.name}: attesting on ${String(roster.length,)} ${
    wordForCount({
      count: roster.length,
      one: 'seat',
      many: 'seats',
    },)
  }`,);
  return await attestCitedReferences({
    client,
    modelIds: roster,
    sourceText,
    archiveText,
    referenceContext: referenceLines,
    signal,
    exchangeTimeoutMs,
    l,
    stageSettings,
  },);
}

//endregion Pass reference attestation
