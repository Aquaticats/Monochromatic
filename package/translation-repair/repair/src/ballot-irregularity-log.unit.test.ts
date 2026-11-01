/**
 Tests that panel and checker ballot irregularities reach the log, naming the
 model that cast the ballot (ledger L12).

 WHY. Both stages fold each ballot's irregularities (a duplicated verdict, a
 number outside the sheet, an unknown vote) into their findings, so the
 artifact records them, and never log one: the audit counted 10 on
 TianqiChen66616, 14 on XingZ6014 and 26 on shihai4h2 in the artifacts, and
 none in the logs.
 The finding strings name no model, so the log line is also the only place
 that says whose ballot it was.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { UNATTRIBUTED_TEXT, } from '@monochromatic-dev/translation-repair-house-rule';
import { runPanelStage, } from '@monochromatic-dev/translation-repair-adjudication';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { aggregateClaims, } from '@monochromatic-dev/translation-repair-shared-contract';
import { runCheckerStage, } from '../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Invented original.
 */
const SOURCE_TEXT = '猫猫在中午打盹。';

/**
 Invented archive rendering with one mistranslation.
 */
const TARGET_TEXT = 'The cat hunts at noon.';

/**
 Voices of both stages; the first casts the irregular ballot.
 */
const VOICES = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
] as const;

/**
 The voice whose ballot repeats an entry.
 */
const [IRREGULAR,] = VOICES;

/**
 Client answering both stages, the irregular voice repeating its first entry.
 */
const CLIENT: ModelClient = {
  chatText: async () => {
    throw new Error('chatText unused by these stages',);
  },
  chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
    /**
     Stage name from the structured-output constraint.
     */
    const stage = request.responseFormat?.json_schema.name ?? '';
    /**
     Whether this voice repeats its first entry.
     */
    const repeats = request.modelId === IRREGULAR;
    /**
     Scripted wire reply for the stage.
     */
    const scripted: unknown = (stage === 'panel_ballot')
      ? {
        verdicts: [
          { claim: 1, reason: 'The quoted TRANSLATION says hunting.', vote: 'supported', },
          ...(repeats ? [{ claim: 1, reason: 'Again.', vote: 'supported', },] : []),
        ],
      }
      : {
        checks: [
          { issue: 1, verdict: 'fixed', },
          ...(repeats ? [{ issue: 1, verdict: 'fixed', },] : []),
        ],
      };
    if (!request.validate(scripted,))
      throw new Error(`stub script failed the ${stage} guard`,);
    return {
      kind: 'ok',
      value: scripted,
      rawText: JSON.stringify(scripted,),
    };
  },
  quotas: async () => {
    throw new Error('quotas unused by these stages',);
  },
};

/**
 The one claim the panel judges.
 */
const CLAIM = {
  category: 'accuracy/mistranslation' as const,
  severity: 'major' as const,
  summary: 'Napping is rendered as hunting.',
  spans: [
    {
      side: 'target' as const,
      nodeId: 'block/1',
      nodeHash: hashContent({ content: TARGET_TEXT, },),
      startOffset: TARGET_TEXT.indexOf('hunts',),
      endOffset: TARGET_TEXT.indexOf('hunts',) + 'hunts'.length,
      quotedText: 'hunts',
    },
  ],
};

/**
 Whether some line names the irregular voice and the irregularity.

 @param lines - lines a stage wrote

 @param irregularity - finding text the line must carry

 @returns Whether such a line exists

 @example
 ```ts
 const logged = namesVoice({ lines, irregularity: 'duplicate-check (1)', },);
 ```
 */
function namesVoice({ lines, irregularity, }: { readonly lines: readonly string[]; readonly irregularity: string; },): boolean {
  return lines.some(function names(line,) {
    return line.includes(IRREGULAR,) && line.includes(irregularity,);
  },);
}

await describe({
  name: 'ballot irregularities reach the log (ledger L12)',
  children: [
    it({
      name: 'the checker stage LOGS a duplicated check with the checker that cast it',
      fn: async () => {
        const { logger, lines, } = capturingLoggerPair();
        await runCheckerStage({
          client: CLIENT,
          checkerModelIds: [...VOICES,].slice(0, 3,),
          sourceText: SOURCE_TEXT,
          patchedText: 'The cat naps at noon.',
          issues: [
            {
              issueId: 'issue/nap',
              status: 'accepted',
              severity: 'major',
              claims: [{ claimId: 'claim/nap', claim: CLAIM, },],
              tallies: {},
            },
          ],
          authorship: UNATTRIBUTED_TEXT,
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l: logger,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(namesVoice({ lines, irregularity: 'duplicate-check (1)', },),).toBe(true,);
      },
    },),
    it({
      name: 'the panel stage LOGS a duplicated verdict with the panelist that cast it',
      fn: async () => {
        const { logger, lines, } = capturingLoggerPair();
        await runPanelStage({
          client: CLIENT,
          // THREE, so every panelist sits on the packet: a packet seats three,
          // and one left off it casts no ballot to be irregular.
          panelModelIds: [...VOICES,].slice(0, 3,),
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          clusters: aggregateClaims({ claims: [CLAIM,], },).clusters,
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l: logger,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(namesVoice({ lines, irregularity: 'duplicate-verdict (1)', },),).toBe(true,);
      },
    },),
  ],
},);
