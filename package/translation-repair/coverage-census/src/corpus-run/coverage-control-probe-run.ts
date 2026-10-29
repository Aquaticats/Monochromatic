import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { CorpusPin, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  type CommandLineOf,
  idListFlag,
} from '@monochromatic-dev/translation-repair-run/ts';
import { gatherCases, } from './coverage-control-probe-cases.ts';
import {
  controlLines,
  offeringLine,
} from './coverage-control-probe-lines.ts';
import type {
  CoverageControlCase,
  CoverageControlResult,
} from './coverage-control.ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Coverage control probe run
// Can the coverage roster vote absence at all?
//
// The block-scale reading it produced is ninety-six answers with not one vote
// for absence. Read as a fact about the corpus, that says the translations
// carry everything. Read as a fact about the instrument, it says the evidence
// rule admits any non-empty quote, so absence is unreachable and the count
// means nothing. This runner separates the two.
//
// It asks the roster about real passages, DELETES THE SPANS THE ROSTER ITSELF
// ANCHORED ON, and asks again. Nothing else can choose the damage: coverage
// candidates are exactly the passages the aligners refuse to pair, so no
// pairing exists to say which target text renders one.
//
// SPENDS QUOTA, two roster rounds per case. Point `TRANSLATION_REPAIR_RUNS_DIR`
// at a throwaway directory.

/**
 Runs the control and reports what it found.

 @param line - control's command line, read whole by `reportingRefusals`

 @param pin - corpus clone and commit every read resolves against

 @param newClient - builds the client every call goes through, called before
 the entry filter is read, so a missing key is refused first

 @param holds - control itself: asks the roster, deletes what it anchored
 on, and asks again

 @param roster - models asked

 @param exchangeTimeoutMs - deadline per exchange

 @param l - logger the control logs through

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @throws StatedRefusalError when no entry offered a single passage to ask
 about, since a run that measured nothing must not be reported as one that
 measured a null

 @example
 ```ts
 await runCoverageControl({ line, pin, newClient, holds, roster, exchangeTimeoutMs, l, stageSettings, },);
 ```
 */
export async function runCoverageControl(
  {
    line,
    pin,
    newClient,
    holds,
    roster,
    exchangeTimeoutMs,
    l,
    stageSettings,
  }: {
    readonly line: CommandLineOf<'coverage-control-probe'>;
    readonly pin: CorpusPin;
    readonly newClient: () => ModelClient;
    readonly holds: (input: {
      readonly client: ModelClient;
      readonly cases: readonly CoverageControlCase[];
      readonly modelIds: readonly RosterModelId[];
      readonly signal: AbortSignal;
      readonly exchangeTimeoutMs: number;
      readonly l: Logger;
      readonly stageSettings: StageSettings;
    },) => Promise<CoverageControlResult>;
    readonly roster: readonly RosterModelId[];
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  /**
   Client every call goes through.
   */
  const client = newClient();

  /**
   Cancellation shared by every call, never fired.
   */
  const { signal, } = new AbortController();

  /**
   Entries the caller named.
   */
  const onlyIds = idListFlag({
    asked: line.flag('only',),
    naming: 'entry id',
  },);

  /**
   Passages to try.
   */
  const cases = await gatherCases({
    onlyIds,
    pin,
  },);

  if (cases.length === 0)
    throw new StatedRefusalError({
      says: 'coverage control probe refused: no walked entry offered a passage the aligners '
        + 'declined to pair, so there was nothing to ask the roster about',
    },);

  console.log(offeringLine({
    caseCount: cases.length,
    rosterSize: roster.length,
  },),);
  for (
    const printed of controlLines({
      control: await holds({
        client,
        cases,
        modelIds: roster,
        signal,
        exchangeTimeoutMs,
        l,
        stageSettings,
      },),
    },)
  )
    console.log(printed,);
}

//endregion Coverage control probe run
