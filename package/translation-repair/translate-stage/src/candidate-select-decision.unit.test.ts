/**
 Tests for the select ballot as a typed question (2026-09-18): the question
 carries the task, the rules, the evidence and the numbered candidates, the
 decline is option zero, and a decision seat's answer lands on the ballot
 beside the written ones with its distribution as the reason. Cat-themed
 invention.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  NO_TYPED_ANSWER,
  selectDecision,
  TYPED_BALLOT_REASON,
} from '@monochromatic-dev/translation-repair-adjudication';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  type Candidate,
  FULL_VOTE_WEIGHT,
} from '@monochromatic-dev/translation-repair-house-rule';
import { decideBestCandidate, } from '../dist/final/node/index.mjs';
import { SEAT_OPENROUTER_DECISIONS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the stage writes its progress to.
 */
const l = tagged({ tag: 'candidate-select-decision-test', },);

/**
 Two written judges beside the decision seat.
 */
const CHAT_JUDGES: readonly RosterModelId[] = [
  'hf:cat/Judge-A',
  'hf:cat/Judge-B',
].map(function toId(id,) {
  return id as unknown as RosterModelId;
},);

/**
 Candidates as the stage renders them.
 */
const CANDIDATES: readonly Candidate<{ readonly text: string; }>[] = [
  {
    producer: { kind: 'composite', contributors: [], },
    value: { text: 'The cat naps on the sill.', },
    rendered: 'The cat naps on the sill.',
  },
  {
    producer: { kind: 'composite', contributors: [], },
    value: { text: 'The cat dozes on the windowsill.', },
    rendered: 'The cat dozes on the windowsill.',
  },
];

await describe({
  name: selectDecision.name,
  children: [
    it({
      name: 'PHRASES the ballot as one choice over the decline and the numbered candidates, with the '
        + 'rules and evidence in the state, and READS a choice into a ballot carrying its distribution',
      fn: async () => {
        const decision = selectDecision({
          task: 'Pick the best rendering.',
          criteria: [
            'faithful to the original',
            'natural English',
          ],
          evidence: [{ label: 'ORIGINAL (Chinese)', text: '猫在窗台上打盹。', },],
          rendered: CANDIDATES.map(function toRendered(candidate,) {
            return candidate.rendered;
          },),
        },);
        // The properties this case exists for, not the whole state: the
        // house rules joined it on 2026-09-27 (`decision-seat-policy.unit.test.ts`).
        expect(decision.state,).toMatchObject({
          task: 'Pick the best rendering.',
          criteria: {
            '1': 'faithful to the original',
            '2': 'natural English',
          },
          evidence: [{ label: 'ORIGINAL (Chinese)', text: '猫在窗台上打盹。', },],
          candidates: {
            '1': 'The cat naps on the sill.',
            '2': 'The cat dozes on the windowsill.',
          },
        },);
        expect(Object.keys(decision.questions.best?.criteria ?? {},),).toEqual([
          '0',
          '1',
          '2',
        ],);
        expect(decision.read({
          best: {
            type: 'choice',
            choice: '2',
            probabilities: { '0': 0.1, '1': 0.2, '2': 0.7, },
            confidence: 0.6,
          },
        },),).toEqual({
          best: '2',
          assessment: `${TYPED_BALLOT_REASON}: probabilities {"0":0.1,"1":0.2,"2":0.7}, confidence 0.6`,
        },);
        expect(decision.read({},),).toBe(NO_TYPED_ANSWER,);
      },
    },),

    it({
      name: 'NAMES BOTH UNREPORTED when a choice carries neither a distribution nor a confidence, which the '
        + 'endpoint reports only when it chooses to, rather than writing an empty distribution nobody sent',
      fn: async () => {
        const decision = selectDecision({
          task: 'Pick the best rendering.',
          criteria: ['faithful to the original',],
          evidence: [{ label: 'ORIGINAL (Chinese)', text: '猫在窗台上打盹。', },],
          rendered: CANDIDATES.map(function toRendered(candidate,) {
            return candidate.rendered;
          },),
        },);
        expect(decision.read({
          best: {
            type: 'choice',
            choice: '2',
          },
        },),).toEqual({
          best: '2',
          assessment: `${TYPED_BALLOT_REASON}: probabilities unreported, confidence unreported`,
        },);
      },
    },),

    it({
      name: 'SEATS a decision seat beside written judges: its typed answer is one ballot at full '
        + 'weight and the contest counts it',
      fn: async () => {
        /**
         Client whose written judges pick candidate 2 and whose decision seat
         picks it too.
         */
        const client: ModelClient = {
          chatText: async () => {
            throw new Error('chatText unused',);
          },
          chatJson: async <ValueT,>(
            request: ChatJsonRequest<ValueT>,
          ): Promise<ChatJsonOutcome<ValueT>> => {
            if (request.modelId === SEAT_OPENROUTER_DECISIONS)
              throw new Error('a decision seat must never be asked for chat',);
            /**
             Ballot as the wire expects it.
             */
            const ballot: unknown = { best: 2, assessment: 'written', };
            if (!request.validate(ballot,))
              throw new Error('the fixture ballot failed the wire guard',);
            return {
              kind: 'ok',
              value: ballot,
              rawText: JSON.stringify(ballot,),
            };
          },
          quotas: async () => {
            throw new Error('quotas unused',);
          },
          decide: async (request,) => {
            expect(request.modelId,).toBe(SEAT_OPENROUTER_DECISIONS,);
            return {
              model: 'typesafe/jev-1.13',
              answers: {
                best: {
                  type: 'choice',
                  choice: '2',
                  probabilities: { '0': 0.01, '1': 0.04, '2': 0.95, },
                  confidence: 0.9,
                },
              },
            };
          },
        };
        const outcome = await decideBestCandidate({
          client,
          candidates: CANDIDATES,
          judgeModelIds: [
            ...CHAT_JUDGES,
            SEAT_OPENROUTER_DECISIONS,
          ],
          task: 'Pick the best rendering.',
          criteria: ['faithful to the original',],
          evidence: [{ label: 'ORIGINAL (Chinese)', text: '猫在窗台上打盹。', },],
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);
        expect(outcome.kind,).toBe('selected',);
        if (outcome.kind !== 'selected')
          throw new Error('selected by construction',);
        expect(outcome.selectedIndex,).toBe(2,);
        // Every written judge and the decision seat, none of them a contributor, at full weight.
        expect(outcome.voteWeight,).toBe((CHAT_JUDGES.length + 1) * FULL_VOTE_WEIGHT,);
        /**
         The decision seat's ballot.
         */
        const typed = outcome.ballots.find(function isTyped(ballot,): boolean {
          return ballot.modelId === SEAT_OPENROUTER_DECISIONS;
        },);
        expect(typed?.best,).toBe(2,);
        expect(typed?.reason.startsWith(TYPED_BALLOT_REASON,),).toBe(true,);
      },
    },),
  ],
},);
