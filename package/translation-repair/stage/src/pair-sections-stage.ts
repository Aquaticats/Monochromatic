import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  JsonSchemaResponseFormat,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';
import {
  isSectionPairingWire,
  readSectionPairing,
} from './pair-sections-read.ts';
import {
  buildSectionPairingMessages,
  type NumberedSection,
  requireSectionPairingRefusal,
  type SectionPair,
} from './pair-sections-wire.ts';
import { agreePairs, } from './pair-agreement.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { FanOutMode, } from './stage-fanout-window.ts';
import { shortBenchStageFinding, } from './stage-reachable-quorum.ts';
import { runWindowedRounds, } from './stage-windowed-rounds.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { StageSettings, } from './stage-settings.ts';

//region Section pairing stage
// ASKS THE ROSTER WHICH SECTION RENDERS WHICH, and keeps only what enough of
// them agree on. `pair-blocks-stage.ts` does the same job one scale down, and
// the two share their rules deliberately: no single model decides, and
// agreement is counted per correspondence rather than per reply.
//
// ASKED ONLY WHERE THE DETERMINISTIC ALIGNER REFUSED. Measured over the pinned
// corpus, 85 of 92 entries have equal section shape and never reach the aligner,
// and 5 of the remaining 7 align with no refusal. Two entries are ever asked.

/**
 Voices that must name a correspondence before it is kept.

 TWO, matching the block stage, and for its reason: a pairing one model
 invented is the risk, and a correspondence two models reached independently
 is not plausibly coincidence when each is choosing from every section on the
 other side.
 */
const AGREEMENT_NEEDED = 2;

/**
 Schema the reply must satisfy before it reaches the reader.
 Exported so the family's schema scan (`corpus-pass/src/claude-schemas.unit.test.ts`) reads the schema the stage sends.
 */
export const SECTION_PAIRING_RESPONSE_FORMAT: JsonSchemaResponseFormat = {
  type: 'json_schema',
  json_schema: {
    name: 'section_pairing',
    schema: {
      type: 'object',
      properties: {
        pairs: {
          type: 'array',
          items: {
            type: 'object',
            properties: {
              source: { type: 'integer', },
              target: { type: 'integer', },
            },
            required: [
              'source',
              'target',
            ],
            additionalProperties: false,
          },
        },
      },
      required: ['pairs',],
      additionalProperties: false,
    },
  },
};

/**
 What the roster settled on for one document's sections.

 @example
 ```ts
 const outcome: SectionPairingOutcome = { pairs: [], heard: 0, usable: 0, findings: [], };
 ```
 */
export type SectionPairingOutcome = {
  /**
   Correspondences enough voices named, in document order.
   */
  readonly pairs: readonly SectionPair[];

  /**
   Voices that answered at all.
   */
  readonly heard: number;

  /**
   Voices whose answer survived the reader.
   */
  readonly usable: number;

  /**
   What went wrong, in scorecard-stable wording.
   */
  readonly findings: readonly string[];
};

/**
 One document's settled section pairing as the cache stores it.

 THE FINDINGS ARE HALF THE RECORD, for the reason `PairedSectionRecord` gives
 at block scale: a resumed run makes no calls, so anything this round reported
 the first time is reported by nothing on the second unless it was stored.

 @example
 ```ts
 const settled: PairedDocumentRecord = { pairs: [], findings: [], };
 ```
 */
export type PairedDocumentRecord = {
  /**
   Correspondences the roster agreed on, in document order.
   */
  readonly pairs: readonly SectionPair[];

  /**
   Findings this round contributed, in the order a cold run emitted them.
   */
  readonly findings: readonly string[];
};

/**
 Reads every heard reply, keeping the usable ones and reporting the rest.

 @param outcomes - one round result per voice

 @param sourceCount - original sections the sheet numbered

 @param targetCount - translation sections the sheet numbered

 @param findings - accumulator an unusable reply appends its notice to

 @param l - stage logger

 @returns Pairings that survived the reader, one per usable voice

 @throws Error when a reader raises anything other than a
 `SectionPairingError`, since that is a defect rather than a bad reply

 @example
 ```ts
 const pairings = readUsablePairings({ outcomes, sourceCount, targetCount, findings, l, },);
 ```
 */
function readUsablePairings(
  {
    outcomes,
    sourceCount,
    targetCount,
    findings,
    l,
  }: {
    readonly outcomes: Awaited<ReturnType<typeof runWindowedRounds>>['outcomes'];
    readonly sourceCount: number;
    readonly targetCount: number;
    readonly findings: string[];
    readonly l: Logger;
  },
): readonly (readonly SectionPair[])[] {
  /**
   Pairings that survived the reader, one per usable voice.
   */
  const pairings: (readonly SectionPair[])[] = [];
  for (const outcome of outcomes) {
    /**
     This voice's reply, heard or lost.
     */
    const { voice, } = outcome;
    if (!voice.heard)
      continue;
    try {
      pairings.push(readSectionPairing({
        value: voice.value,
        sourceCount,
        targetCount,
      },),);
    }
    catch (error) {
      /**
       Why the reply cannot be used; anything else propagates.
       */
      const refusal = requireSectionPairingRefusal({ error, },);
      // A REPLY THAT CANNOT BE USED IS A LOST VOICE, not a stage failure: the
      // rest of the roster may still agree on a pairing, and refusing the whole
      // document because one model answered badly is the failure that once
      // discarded translated slices.
      findings.push(`section-pairing unusable (${outcome.modelId}: ${refusal.message})`,);
      l.warn(`${outcome.modelId} returned an unusable section pairing: ${refusal.message}`,);
    }
  }
  return pairings;
}

/**
 Asks the roster to pair two documents' sections and keeps the agreed ones.

 REFUSES RATHER THAN GUESSES. When no voice answers usably the outcome carries
 no pairs and says why, and the caller keeps the deterministic aligner's own
 refusals rather than proceeding on one model's word, by the standing rule:
 a wrong pairing is worse than no pairing, because it manufactures issues
 rather than skipping work.

 @param client - injected model client

 @param modelIds - roster to ask

 @param sourceSections - original sections in document order

 @param targetSections - translation sections in document order

 @param signal - caller's steering

 @param exchangeTimeoutMs - per-call bound

 @param l - driver logger

 @param fanOut - seats a round asks: the window of quorum plus one by
 default, or the whole bench a fixture scripting every seat asks for

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns What the roster agreed on, with what it lost

 @example
 ```ts
 const outcome = await pairSectionsWithRoster({ client, modelIds, sourceSections, targetSections, signal, exchangeTimeoutMs, l, stageSettings, },);
 ```
 */
export async function pairSectionsWithRoster(
  {
    client,
    modelIds,
    sourceSections,
    targetSections,
    signal,
    exchangeTimeoutMs,
    l,
    fanOut,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly sourceSections: readonly NumberedSection[];
    readonly targetSections: readonly NumberedSection[];
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  }>,
): Promise<SectionPairingOutcome> {
  /**
   Logger tagged with this stage.
   */
  const pl = tagged({
    tag: pairSectionsWithRoster.name,
    l,
  },);

  /**
   Every voice's reply, heard or lost.
   */
  const {
    outcomes,
    quorum,
  } = await runWindowedRounds({
    client,
    modelIds,
    messages: buildSectionPairingMessages({
      sourceSections,
      targetSections,
    },),
    signal,
    exchangeTimeoutMs,
    responseFormat: SECTION_PAIRING_RESPONSE_FORMAT,
    validate: isSectionPairingWire,
    stage: 'section-pairing',
    l: pl,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((fanOut === undefined) ? {} : { fanOut, }),
    graceMs: stageSettings
      .grace
      .roundMs,
  },);

  /**
   Voices that arrived and validated in shape.
   */
  const heard = outcomes
    .filter(function wasHeard(outcome,) {
      /**
       This voice's reply, heard or lost.
       */
      const { voice, } = outcome;
      return voice.heard;
    },)
    .length;

  /**
   Findings accumulated while reading replies, opening with a short bench
   where the router left one, as the gathers say it (ledger X8).
   */
  const findings: string[] = quorum.short
    ? [shortBenchStageFinding({
      stage: 'section-pairing',
      quorum,
      benchSize: modelIds.length,
    },),]
    : [];

  /**
   Pairings that survived the reader, one per usable voice.
   */
  const pairings = readUsablePairings({
    outcomes,
    sourceCount: sourceSections.length,
    targetCount: targetSections.length,
    findings,
    l: pl,
  },);

  if (pairings.length === 0) {
    findings.push(
      `section-pairing no-usable-voice (${String(heard,)} heard of ${String(modelIds.length,)})`,
    );
    return {
      pairs: [],
      heard,
      usable: 0,
      findings,
    };
  }

  /**
   Pairs the roster agreed on, counted over every usable voice's pairs and
   kept strictly increasing on both sides.
   */
  const agreement = agreePairs({
    pairings,
    needed: AGREEMENT_NEEDED,
    pairingShape: 'one-to-one',
  },);
  /**
   What agreement dropped, in its own words.
   */
  const { findings: dropped, } = agreement;

  /**
   The same, in this stage's vocabulary.
   */
  const prefixed = dropped.map(function prefix(finding,): string {
    return `section-pairing ${finding}`;
  },);
  findings.push(...prefixed,);

  /**
   Pairs that survived agreement and ordering.
   */
  const agreed = agreement.pairs;
  pl.info(
    `paired ${String(agreed.length,)} of ${String(sourceSections.length,)} original and ${
      String(targetSections.length,)
    } translation ${
      wordForCount({
        count: targetSections.length,
        one: 'section',
        many: 'sections',
      },)
    }, from ${String(pairings.length,)} usable ${
      wordForCount({
        count: pairings.length,
        one: 'voice',
        many: 'voices',
      },)
    } of ${String(heard,)} heard`,
  );
  return {
    pairs: agreed,
    heard,
    usable: pairings.length,
    findings,
  };
}

//endregion Section pairing stage
