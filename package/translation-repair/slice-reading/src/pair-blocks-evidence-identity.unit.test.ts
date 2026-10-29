import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  BlockPairingWire,
  RoundOutcome,
} from '@monochromatic-dev/translation-repair-stage';
import { createSyntheticClient, } from '@monochromatic-dev/translation-repair-provider';
import {
  pairBlocksWithRoster,
  PairingEvidenceError,
  readBlockPairingOutcomes,
} from '../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

const roster = [SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD, SEAT_HYPER_OPENROUTER_VISION_EDITOR,] as const;
const l = tagged({ tag: 'pairing-evidence-identity-test', },);
const wire: BlockPairingWire = { pairs: [{ source: 0, target: 0, },], };
const heard = (modelId: RosterModelId): RoundOutcome<BlockPairingWire> => ({ modelId, voice: { heard: true, value: wire, }, });
const missing = (modelId: RosterModelId): RoundOutcome<BlockPairingWire> => ({ modelId, voice: { heard: false, answered: false, unreachable: false, }, });
const invalid: readonly {
  readonly name: string;
  readonly modelIds: readonly RosterModelId[];
  readonly outcomes: readonly RoundOutcome<BlockPairingWire>[];
}[] = [
  { name: 'one identity cannot manufacture two endorsements', modelIds: roster, outcomes: [heard(roster[0]), heard(roster[0]),], },
  { name: 'unconfigured outcomes are not electorate members', modelIds: roster, outcomes: [heard(SEAT_SYNTHETIC_TEXT_EVERYWHERE),], },
  { name: 'reordered outcomes are not a final roster-ordered record', modelIds: roster, outcomes: [heard(roster[1]), heard(roster[0]),], },
  { name: 'duplicate missing outcomes are still duplicate seat evidence', modelIds: roster, outcomes: [missing(roster[0]), missing(roster[0]),], },
  { name: 'an empty electorate is not a pairing configuration', modelIds: [], outcomes: [], },
  { name: 'a duplicated electorate is not independent configuration', modelIds: [roster[0], roster[0],], outcomes: [heard(roster[0]),], },
];

await describe({
  name: 'pairing evidence identity',
  children: [
    ...invalid.map(test => it({
      name: test.name,
      fn: async () => {
        expect(() => readBlockPairingOutcomes({ ...test, sourceCount: 2, targetCount: 2, l, },),).toThrow(PairingEvidenceError,);
      },
    },)),
    it({
      name: 'accepts a sparse ordered subset without inventing the spared seat',
      fn: async () => {
        const outcomes = [heard(roster[0]), heard(roster[2]),];
        const result = readBlockPairingOutcomes({ outcomes, modelIds: roster, sourceCount: 2, targetCount: 2, l, },);
        expect(result.pairs,).toEqual(wire.pairs,);
        // The two asked seats are heard; the spared third is neither heard nor counted as a ballot.
        expect(result.heard,).toBe(2,);
        expect(result.usable,).toBe(2,);
      },
    },),
    it({
      name: 'a single configured identity remains insufficient for two endorsements',
      fn: async () => {
        const result = readBlockPairingOutcomes({ outcomes: [heard(roster[0]),], modelIds: [roster[0],], sourceCount: 2, targetCount: 2, l, },);
        expect(result.usable,).toBe(1,);
        expect(result.pairs,).toEqual([],);
      },
    },),
    ...[
      { name: 'duplicated', modelIds: [roster[0], roster[0],], },
      { name: 'empty', modelIds: [], },
    ].map(test => it({
      name: `refuses ${test.name} live electorate before any model transport`,
      fn: async () => {
        let calls = 0;
        const client = createSyntheticClient({
          apiKey: 'fixture-key',
          transport: async () => {
            calls += 1;
            return { status: 200, bodyText: `data: ${JSON.stringify({ choices: [{ index: 0, delta: { content: JSON.stringify(wire,), }, },], },)}\n\ndata: [DONE]\n\n`, };
          },
        },);
        let caught: unknown;
        try {
          await pairBlocksWithRoster({ client, modelIds: test.modelIds, sourceBlocks: [{ index: 0, text: '猫睡了。', },],
            targetBlocks: [{ index: 0, text: 'The cat slept.', },], signal: new AbortController().signal,
            exchangeTimeoutMs: HANG_STOP_MS, l, fanOut: 'whole-bench', stageSettings: SCRIPTED_STAGE_SETTINGS, },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(PairingEvidenceError,);
        expect(calls,).toBe(0,);
      },
    },)),
  ],
},);
