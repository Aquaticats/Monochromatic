/**
 Tests generation-six final body polish artifact reader and the gate reader
 it calls: every not-run reason, a changed and an unchanged polish against
 each shipping role its gate may record, an unchanged polish with no gate,
 a naturalness review bound to the text the polish ships, ballot names
 narrowed to polish choices, and each refusal checked by the path and reason
 its message carries, so a case refused by another check fails. Texts and
 model ids are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import { parseConsolidationPolish, } from '../../dist/final/node/index.mjs';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Artifact path every case reads the polish at.
 */
const POLISH_PATH = 'artifact.consolidation.slices[0].polish';

/**
 One ballot for the polished wording.
 */
const POLISHED_BALLOT = {
  choice: 'polished',
  unsupported: [],
  unsupportedRaw: [],
  dropped: [],
  droppedRaw: [],
  reason: 'equally faithful and more idiomatic',
} as const;

/**
 Settled polish record fixture.
 */
const SETTLED = {
  kind: 'settled',
  baseText: 'The cat faced life proactively.',
  proposedText: 'The cat maintained a positive outlook on life.',
  text: 'The cat maintained a positive outlook on life.',
  changed: true,
  refinersHeard: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  contributors: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  roundCount: 1,
  gate: {
    choice: 'polished',
    ships: 'polished',
    ballots: [POLISHED_BALLOT,],
    usable: 1,
    findings: [],
  },
  findings: [],
} as const;

/**
 The settled polish kept at its base wording, its gate choosing the given
 candidate and shipping base.

 @param choice - panel choice the gate records

 @returns Polish value
 */
function unchangedPolish({ choice, }: { readonly choice: 'base' | 'neither'; },) {
  return {
    ...SETTLED,
    text: SETTLED.baseText,
    changed: false,
    gate: {
      ...SETTLED.gate,
      choice,
      ships: 'base',
      ballots: [{
        ...POLISHED_BALLOT,
        choice,
      },],
    },
  };
}

/**
 Reads a polish expecting the refusal of one check, named by the path and
 reason its message carries.

 @param value - polish as an artifact carries it

 @param says - `: expected <reason>` text after the path, the path suffix first
 */
function expectRefused(
  {
    value,
    says,
  }: {
    readonly value: unknown;
    readonly says: string;
  },
): void {
  /**
   The read, repeated for each check.
   */
  const read = () =>
    parseConsolidationPolish({
      value,
      path: POLISH_PATH,
    },);
  expect(read,).toThrow(ArtifactParseError,);
  expect(read,).toThrow(`at ${POLISH_PATH}${says}`,);
}

await describe({
  name: parseConsolidationPolish.name,
  children: [
    it({
      name: 'READS SETTLED POLISH and each reason a polish did not run',
      fn: async () => {
        expect(parseConsolidationPolish({
          value: SETTLED,
          path: POLISH_PATH,
        },),).toEqual(SETTLED,);
        for (const reason of ['front-matter', 'not-configured', 'unsafe-baseline',]) {
          expect(parseConsolidationPolish({
            value: {
              kind: 'not-run',
              reason,
            },
            path: POLISH_PATH,
          },),).toEqual({
            kind: 'not-run',
            reason,
          },);
        }
      },
    },),

    it({
      name: 'READS AN UNCHANGED POLISH whose gate chose base or neither and ships base',
      fn: async () => {
        for (const choice of ['base', 'neither',] as const) {
          expect(parseConsolidationPolish({
            value: unchangedPolish({ choice, },),
            path: POLISH_PATH,
          },),).toEqual(unchangedPolish({ choice, },),);
        }
      },
    },),

    it({
      name: 'READS AN UNCHANGED POLISH WITH NO GATE, which only a changed polish needs',
      fn: async () => {
        /**
         Settled fields except the gate.
         */
        const {
          gate: unusedGate,
          ...withoutGate
        } = SETTLED;
        expect(unusedGate,).not.toBeUndefined();
        /**
         The polish kept at its base wording, with no gate.
         */
        const unchanged = {
          ...withoutGate,
          text: SETTLED.baseText,
          changed: false,
        };
        expect(parseConsolidationPolish({
          value: unchanged,
          path: POLISH_PATH,
        },),).toEqual(unchanged,);
      },
    },),

    it({
      name: 'READS A POLISH WITH ITS NATURALNESS REVIEW where the generation carries one, and refuses a review of '
        + 'other text than the polish ships',
      fn: async () => {
        /**
         One reviewer accepting the whole passage.
         */
        const accepting = (modelId: string,) => ({
          modelId,
          status: 'acceptable',
          findings: [],
          reason: 'whole passage is publication-ready',
        });
        /**
         A no-correction review of the given text by two accepting seats.
         */
        const reviewOf = (text: string,) => ({
          correctionCount: 0,
          rounds: [{
            candidateDigest: hashContent({ content: text, },),
            paragraphCount: 1,
            seats: [accepting('hf:cat/Cat-A',), accepting('hf:cat/Cat-B',),],
            usable: 2,
            verdict: 'acceptable',
            findings: [],
          },],
        });
        /**
         The settled polish with a review of the text it ships.
         */
        const reviewed = {
          ...SETTLED,
          review: reviewOf(SETTLED.text,),
        };
        expect(parseConsolidationPolish({
          value: reviewed,
          path: POLISH_PATH,
          reviewRequired: true,
        },),).toEqual(reviewed,);
        /**
         The read of a review of the base wording instead.
         */
        const readOther = () =>
          parseConsolidationPolish({
            value: {
              ...SETTLED,
              review: reviewOf(SETTLED.baseText,),
            },
            path: POLISH_PATH,
            reviewRequired: true,
          },);
        expect(readOther,).toThrow(ArtifactParseError,);
        expect(readOther,).toThrow(`at ${POLISH_PATH}.review.rounds[0].candidateDigest: expected SHA-256 of final polish text`,);
      },
    },),

    it({
      name: 'READS BALLOT NAMES a voter marked unsupported or dropped as polish choices, and refuses a name that is '
        + 'none',
      fn: async () => {
        /**
         The settled polish with its one ballot's unsupported and dropped names.
         */
        const marking = {
          ...SETTLED,
          gate: {
            ...SETTLED.gate,
            ballots: [{
              ...POLISHED_BALLOT,
              unsupported: ['base',],
              unsupportedRaw: ['B',],
              dropped: ['neither',],
              droppedRaw: ['C',],
            },],
          },
        };
        expect(parseConsolidationPolish({
          value: marking,
          path: POLISH_PATH,
        },),).toEqual(marking,);
        expectRefused({
          value: {
            ...marking,
            gate: {
              ...marking.gate,
              ballots: [{
                ...marking.gate.ballots[0],
                unsupported: ['purr',],
              },],
            },
          },
          says: '.gate.ballots[0].unsupported: expected one of polished, base, neither',
        },);
        expectRefused({
          value: {
            ...marking,
            gate: {
              ...marking.gate,
              ballots: [{
                ...marking.gate.ballots[0],
                dropped: ['purr',],
              },],
            },
          },
          says: '.gate.ballots[0].dropped: expected one of polished, base, neither',
        },);
      },
    },),

    it({
      name: 'REFUSES A KIND or a not-run reason it does not know',
      fn: async () => {
        expectRefused({
          value: {
            ...SETTLED,
            kind: 'napping',
          },
          says: '.kind: expected one of settled, not-run',
        },);
        expectRefused({
          value: {
            kind: 'not-run',
            reason: 'too-sleepy',
          },
          says: '.reason: expected one of front-matter, not-configured, unsafe-baseline',
        },);
      },
    },),

    it({
      name: 'REFUSES CHANGED FLAG disagreeing with final text',
      fn: async () => {
        expectRefused({
          value: {
            ...SETTLED,
            changed: false,
          },
          says: '.changed: expected true, which is whether final text differs from baseText.',
        },);
      },
    },),

    it({
      name: 'REFUSES CHANGED POLISH WITHOUT FINAL GATE',
      fn: async () => {
        /**
         Settled fields except required changed-polish gate.
         */
        const {
          gate: unusedGate,
          ...withoutGate
        } = SETTLED;
        expect(unusedGate,).not.toBeUndefined();
        expectRefused({
          value: withoutGate,
          says: '.gate: expected final gate approving every changed polish',
        },);
      },
    },),

    it({
      name: 'REFUSES A GATE whose shipping role is not the one its choice implies, or is neither role, or whose '
        + 'choice is none it knows',
      fn: async () => {
        expectRefused({
          value: {
            ...SETTLED,
            gate: {
              ...SETTLED.gate,
              choice: 'polished',
              ships: 'base',
            },
          },
          says: '.gate.ships: expected polished, derived from gate choice',
        },);
        expectRefused({
          value: {
            ...SETTLED,
            gate: {
              ...SETTLED.gate,
              ships: 'both',
            },
          },
          says: '.gate.ships: expected one of polished, base',
        },);
        expectRefused({
          value: {
            ...SETTLED,
            gate: {
              ...SETTLED.gate,
              choice: 'purr',
            },
          },
          says: '.gate.choice: expected one of polished, base, neither',
        },);
      },
    },),

    it({
      name: 'REFUSES A CHANGED POLISH its gate kept at base or that ships other than the proposed text, and an '
        + 'unchanged polish its gate shipped polished',
      fn: async () => {
        expectRefused({
          value: {
            ...SETTLED,
            gate: {
              ...SETTLED.gate,
              choice: 'base',
              ships: 'base',
            },
          },
          says: ': expected changed polish whose gate ships polished proposedText',
        },);
        expectRefused({
          value: {
            ...SETTLED,
            text: 'The cat kept a sunny outlook on life.',
          },
          says: ': expected changed polish whose gate ships polished proposedText',
        },);
        expectRefused({
          value: {
            ...SETTLED,
            text: SETTLED.baseText,
            changed: false,
          },
          says: ': expected unchanged polish whose gate keeps base',
        },);
      },
    },),

    it({
      name: 'REFUSES GATE USABLE COUNT disagreeing with ballots',
      fn: async () => {
        expectRefused({
          value: {
            ...SETTLED,
            gate: {
              ...SETTLED.gate,
              usable: 2,
            },
          },
          says: '.gate.usable: expected 1 matching stored ballot',
        },);
      },
    },),

    it({
      name: 'REFUSES UNKNOWN POLISH KEY under exact schema',
      fn: async () => {
        expectRefused({
          value: {
            ...SETTLED,
            extra: true,
          },
          says: '.extra: expected no key here beyond kind, baseText,',
        },);
      },
    },),
  ],
},);
