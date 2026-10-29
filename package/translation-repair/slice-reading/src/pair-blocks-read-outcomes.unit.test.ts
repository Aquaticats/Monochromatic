import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import type {
  BlockPair,
  BlockPairingWire,
  RoundOutcome,
} from '@monochromatic-dev/translation-repair-stage';
import { readBlockPairingOutcomes, } from '../dist/final/node/index.mjs';
import {
  SEAT_OPENROUTER_ONLY,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

const roster = [SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD, SEAT_HYPER_OPENROUTER_VISION_EDITOR,] as const;
const l = tagged({ tag: 'pairing-outcome-reading-test', },);

await describe({
  name: readBlockPairingOutcomes.name,
  children: [
    it({
      name: 'keeps independently endorsed relations, not singleton relations or absent ballots',
      fn: async () => {
        const outcomes: readonly RoundOutcome<BlockPairingWire>[] = [
          { modelId: roster[0], voice: { heard: true, value: { pairs: [{ source: 0, target: 0, }, { source: 1, target: 1, },], }, }, },
          { modelId: roster[1], voice: { heard: true, value: { pairs: [{ source: 0, target: 0, },], }, }, },
          { modelId: roster[2], voice: { heard: false, answered: false, unreachable: true, }, },
        ];
        const result = readBlockPairingOutcomes({ outcomes, modelIds: roster, sourceCount: 2, targetCount: 2, l, },);
        expect(result.pairs,).toEqual([{ source: 0, target: 0, },],);
        expect(result.heard,).toBe(2,);
        expect(result.usable,).toBe(2,);
        expect(readBlockPairingOutcomes({ outcomes: structuredClone(outcomes,), modelIds: roster, sourceCount: 2, targetCount: 2, l, },),).toEqual(result,);
      },
    },),
    it({
      name: 'reads schema-valid replies naming blocks out of range as unusable rather than accepting their relations',
      fn: async () => {
        const outcomes: readonly RoundOutcome<BlockPairingWire>[] = roster.map(modelId => ({
          modelId,
          voice: { heard: true, value: { pairs: [{ source: 9, target: 0, },], }, },
        }));
        const result = readBlockPairingOutcomes({ outcomes, modelIds: roster, sourceCount: 2, targetCount: 2, l, },);
        expect(result.pairs,).toEqual([],);
        expect(result.heard,).toBe(3,);
        expect(result.usable,).toBe(0,);
        expect(result.cacheEligible,).toBe(false,);
        expect(result.findings.filter(finding => finding.includes('unusable')),).toHaveLength(3,);
      },
    },),
    it({
      name: 'reads an unreadable reply as unheard and counts configured seats, not asked ones, in the finding',
      fn: async () => {
        const outcomes: readonly RoundOutcome<BlockPairingWire>[] = [
          { modelId: roster[1], voice: { heard: false, answered: true, unreachable: false, unreadable: 'off-shape', }, },
        ];
        const result = readBlockPairingOutcomes({ outcomes, modelIds: roster, sourceCount: 2, targetCount: 2, l, },);
        expect(result.pairs,).toEqual([],);
        expect(result.heard,).toBe(0,);
        expect(result.usable,).toBe(0,);
        expect(result.findings,).toEqual(['block-pairing no-usable-voice (0 heard of 3)',],);
      },
    },),
    it({
      name: 'exempts definition blocks from the order rule only when told which blocks are definitions',
      fn: async () => {
        const outcomes: readonly RoundOutcome<BlockPairingWire>[] = roster.map(modelId => ({
          modelId,
          voice: { heard: true, value: { pairs: [{ source: 0, target: 1, }, { source: 1, target: 0, },], }, },
        }));
        const ordinary = readBlockPairingOutcomes({ outcomes, modelIds: roster, sourceCount: 2, targetCount: 2, l, },);
        const definitions = readBlockPairingOutcomes({ outcomes, modelIds: roster, sourceCount: 2, targetCount: 2,
          freeOrder: { source: new Set([0, 1,],), target: new Set([0, 1,],), }, l, },);
        expect(ordinary.usable,).toBe(0,);
        expect(definitions.usable,).toBe(3,);
        expect(definitions.cacheEligible,).toBe(false,);
      },
    },),
    it({
      name: 'keeps the two-voice relation threshold distinct from a larger electorate quorum',
      fn: async () => {
        const largerRoster = [...roster, SEAT_SYNTHETIC_TEXT_EVERYWHERE, SEAT_OPENROUTER_ONLY,] as const;
        const outcomes: readonly RoundOutcome<BlockPairingWire>[] = largerRoster.map((modelId, index) => ({
          modelId,
          voice: { heard: true, value: { pairs: index < 2 ? [{ source: 0, target: 0, },] : [], }, },
        }));
        const result = readBlockPairingOutcomes({ outcomes, modelIds: largerRoster, sourceCount: 2, targetCount: 2, l, },);
        expect(result.pairs,).toEqual([{ source: 0, target: 0, },],);
        expect(result.usable,).toBe(5,);
      },
    },),
    it({
      name: 'reads an empty reply list as no usable voice over the configured seats',
      fn: async () => {
        const result = readBlockPairingOutcomes({ outcomes: [], modelIds: roster, sourceCount: 2, targetCount: 2, l, },);
        expect(result.heard,).toBe(0,);
        expect(result.cacheEligible,).toBe(false,);
        expect(result.findings,).toEqual(['block-pairing no-usable-voice (0 heard of 3)',],);
      },
    },),
    it({
      name: 'propagates unexpected reader errors instead of treating infrastructure failures as unusable votes',
      fn: async () => {
        const failure = new Error('fixture wire access failed',);
        const value: BlockPairingWire = {
          get pairs(): readonly BlockPair[] {
            throw failure;
          },
        };
        let caught: unknown;
        try {
          readBlockPairingOutcomes({
            outcomes: [{ modelId: roster[0], voice: { heard: true, value, }, },],
            modelIds: roster, sourceCount: 2, targetCount: 2, l,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBe(failure,);
      },
    },),
  ],
},);
