/**
 Test for the weight of a ballot cast by the model that answered for a judge whose reasoning runaway was re-asked (ledger
 B1142, the lead's ruling of 2026-10-31 that a re-ask must not give one model two voices and that own-text half weight
 needs the true author).

 A BALLOT IS WEIGHED BY THE MODEL THAT CAST IT. The seat that ran away casts none; the model asked in its place, when it
 wrote one of the candidates, backs its own text at half weight, as any judge that is also a producer does. Weighed
 under the seat, the same ballot would have counted in full, and a model would have judged its own text at full weight.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { Candidate, } from '@monochromatic-dev/translation-repair-house-rule';
import {
  alternateModelsFor,
  type ChatJsonOutcome,
  type ChatJsonRequest,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { selectBestCandidate, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

//region Alternate judge tests

/**
 Judges, five so the bench reaches quorum with every seat reachable.
 */
const JUDGES: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

/**
 Judge seat whose reasoning runaway is re-asked of another model.
 */
const RUNAWAY_SEAT = JUDGES[0] ?? SEAT_HYPER_OPENROUTER_VISION_EDITOR;

/**
 The model the re-ask of that seat goes to: the first of the order that no seat of the bench holds.
 */
const [ALTERNATE,] = alternateModelsFor({
  modelId: RUNAWAY_SEAT,
  role: 'wide-judge',
  bench: JUDGES,
},).alternates;

/**
 Client whose first seat runs away and whose every other model backs the first candidate.

 @returns Client usable by the selection stage

 @example
 ```ts
 const client = runawayThenBackers();
 ```
 */
function runawayThenBackers(): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by candidate selection',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      if (request.modelId === RUNAWAY_SEAT) {
        return {
          kind: 'schema-mismatch',
          rawText: '',
          reason: 'truncated-completion',
          detail: 'provider reported a truncating completion',
        };
      }
      /**
       Ballot naming the first candidate.
       */
      const scripted: unknown = {
        best: 1,
        assessment: 'scripted',
      };
      if (!request.validate(scripted,))
        throw new Error('stub script failed the ballot guard',);
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by candidate selection',);
    },
  };
}

await describe({
  name: `${selectBestCandidate.name} (an alternate judge)`,
  concurrency: 1,
  children: [
    it({
      name: 'WEIGHS THE BALLOT OF THE MODEL THAT ANSWERED FOR A RUNAWAY SEAT AS THAT MODEL\'S OWN: it backs the candidate '
        + 'it wrote at half weight, and the seat that ran away casts none',
      fn: async () => {
        if (ALTERNATE === undefined)
          throw new Error('the wide-judge role has a model left to answer a runaway',);

        /**
         Two candidates, the first written by the model that answers for the runaway seat.
         */
        const candidates: readonly Candidate<string>[] = [
          {
            producer: { kind: 'model', modelId: ALTERNATE, },
            value: 'first',
            rendered: 'The cat chases butterflies.',
          },
          {
            producer: { kind: 'model', modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, },
            value: 'second',
            rendered: 'The cat loves chasing butterflies.',
          },
        ];
        const outcome = await selectBestCandidate({
          client: runawayThenBackers(),
          candidates,
          judgeModelIds: JUDGES,
          role: 'wide-judge',
          fanOut: 'whole-bench',
          task: 'Pick one.',
          criteria: ['Faithful.',],
          evidence: [{ label: 'ORIGINAL', text: '猫猫喜欢追蝴蝶。', },],
          signal: new AbortController().signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'candidate-select-alternate-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome.ballots.map(function modelOf(ballot,): string {
          return ballot.modelId;
        },),).not.toContain(RUNAWAY_SEAT,);
        expect(outcome.ballots
          .filter(function isAlternate(ballot,): boolean {
            return ballot.modelId === ALTERNATE;
          },)
          .map(function weighed(ballot,) {
            return [ballot.selfVote, ballot.weight,];
          },),).toEqual([[true, 0.5,],],);
      },
    },),
  ],
},);

//endregion Alternate judge tests
