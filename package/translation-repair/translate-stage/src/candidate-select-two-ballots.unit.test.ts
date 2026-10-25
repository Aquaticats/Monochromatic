/**
 Tests for the selection round's floor of two ballots, which replaced the minimum vote weight of two.

 THE OWNER'S RULINGS OF 2026-10-25, quoted where the rule is stated
 (`doc/design-commitments-votes-repairs-and-run-offs.md`): "floor of 3 - don't set a floor." and "Every weight
 floor": a leader named by two judges wins whatever the two ballots weigh, and one judge alone never decides.
 A producer's vote for its own candidate carries half a vote toward which candidate leads and still counts as one
 answering judge. Before the change a bench at quorum declined a leader drawing 1.5 from two ballots, a full vote and
 a self-vote, against the minimum of 2.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { Candidate, } from '@monochromatic-dev/translation-repair-house-rule';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  selectBestCandidate,
  selectionFanOut,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the stage writes its progress to.
 */
const l = tagged({ tag: 'candidate-select-two-ballots-test', },);

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
 Two competing candidates, the first written by the first judge.
 */
const CANDIDATES: readonly Candidate<string>[] = [
  {
    producer: { kind: 'model', modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, },
    value: 'first',
    rendered: 'The cat chases butterflies.',
  },
  {
    producer: { kind: 'model', modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER, },
    value: 'second',
    rendered: 'The cat loves chasing butterflies.',
  },
];

/**
 Client answering every ballot from a script: one-based candidate index per
 judge, zero to decline.

 @param ballots - ballot per judge id

 @returns Client usable by the selection stage

 @example
 ```ts
 const client = scriptedJudges({ ballots: { [JUDGES[0]]: 1, }, },);
 ```
 */
function scriptedJudges(
  { ballots, }: { readonly ballots: Readonly<Record<string, number>>; },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by candidate selection',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Scripted ballot for the judge that was asked.
       */
      const scripted: unknown = {
        best: ballots[request.modelId] ?? 0,
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

/**
 Runs one selection round over the fixture candidates with the whole bench asked.

 @param ballots - ballot per judge id

 @returns Verdict of the round

 @example
 ```ts
 const outcome = await runRound({ ballots: { [JUDGES[2]]: 1, }, },);
 ```
 */
async function runRound(
  { ballots, }: { readonly ballots: Readonly<Record<string, number>>; },
) {
  return selectBestCandidate({
    client: scriptedJudges({ ballots, },),
    candidates: CANDIDATES,
    judgeModelIds: JUDGES,
    fanOut: 'whole-bench',
    task: 'Pick one.',
    criteria: ['Faithful.',],
    evidence: [{ label: 'ORIGINAL', text: '猫猫喜欢追蝴蝶。', },],
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: 'selection floor of two ballots',
  children: [
    describe({
      name: selectBestCandidate.name,
      children: [
        it({
          name: 'SELECTS the leader two judges named on a bench at quorum, one a full vote and one the author\'s '
            + 'own vote, at 1.5 against no rival',
          fn: async () => {
            /** The author backs its own text; a disinterested judge backs it too; three decline. */
            const outcome = await runRound({
              ballots: {
                [JUDGES[0] as string]: 1,
                [JUDGES[2] as string]: 1,
              },
            },);
            expect(outcome.kind,).toBe('selected',);
            expect(outcome.kind === 'selected' ? outcome.selectedIndex : 0,).toBe(1,);
            expect(outcome.kind === 'selected' ? outcome.voteWeight : 0,).toBe(1.5,);
          },
        },),

        it({
          name: 'SELECTS the leader two judges named over a rival one judge named, at 1.5 against 1',
          fn: async () => {
            /** The second author and a disinterested judge back the second candidate; one judge backs the first. */
            const outcome = await runRound({
              ballots: {
                [JUDGES[1] as string]: 2,
                [JUDGES[2] as string]: 2,
                [JUDGES[3] as string]: 1,
              },
            },);
            expect(outcome.kind,).toBe('selected',);
            expect(outcome.kind === 'selected' ? outcome.selectedIndex : 0,).toBe(2,);
          },
        },),

        it({
          name: 'DECLINES a leader one judge alone named, the positive control: one judge alone never decides',
          fn: async () => {
            /** One judge names a candidate; four decline. */
            const outcome = await runRound({ ballots: { [JUDGES[2] as string]: 1, }, },);
            expect(outcome.kind,).toBe('declined',);
            expect(outcome.kind === 'declined' ? outcome.disposition : '',).toBe('indecision',);
          },
        },),
      ],
    },),

    describe({
      name: selectionFanOut.name,
      children: [
        it({
          name: 'ASKS the window on a four-seat bench, since two answering judges decide and the window holds three',
          fn: async () => {
            expect(selectionFanOut({ judgeCount: 4, },),).toBe('window',);
            expect(selectionFanOut({ judgeCount: 3, },),).toBe('window',);
          },
        },),
      ],
    },),
  ],
},);
