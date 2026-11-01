/**
 Tests for the distinct replacements offered for one envelope.

 Provenance is the point: a duplicate proposal merges its author into the
 survivor rather than being dropped, so a judge that wrote the words is
 discounted for them however many others wrote the same. Fixtures are
 cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  EditableEnvelope,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import type { CandidateProducer, } from '@monochromatic-dev/translation-repair-house-rule';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  collectEnvelopeProposals,
  type EditorCandidate,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Text the envelope covers.
 */
const BASE_TEXT = 'The cat is doing the sleeping on the windowsill.';

/**
 Envelope under decision.
 */
const ENVELOPE: EditableEnvelope = {
  envelopeId: 'envelope/cat',
  startOffset: 0,
  endOffset: BASE_TEXT.length,
  baseText: BASE_TEXT,
  baseHash: hashContent({ content: BASE_TEXT, },),
  issueIds: ['adjudicated/cat-tense',],
};

/**
 Editor outcome proposing one replacement for the envelope, or nothing.

 @param modelId - editor

 @param newText - its replacement, absent when it left the envelope alone

 @returns Candidate as the stage records it

 @example
 ```ts
 const candidate = proposing({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, newText: 'The cat sleeps.', },);
 ```
 */
function proposing(
  {
    modelId,
    newText,
  }: {
    readonly modelId: RosterModelId;
    readonly newText?: string;
  },
): EditorCandidate {
  return {
    modelId,
    patch: {
      patchedText: newText ?? BASE_TEXT,
      applied: (newText === undefined)
        ? []
        : [
          {
            envelopeId: ENVELOPE.envelopeId,
            baseHash: ENVELOPE.baseHash,
            newText,
          },
        ],
      rejected: [],
    },
  };
}

/**
 Every model a producer credits.

 @param producer - who a candidate is credited to

 @returns Model ids, in the producer's own order

 @example
 ```ts
 const models = creditedTo({ producer, },);
 ```
 */
function creditedTo({ producer, }: { readonly producer: CandidateProducer; },): readonly string[] {
  if (producer.kind === 'model')
    return [producer.modelId,];
  if (producer.kind === 'composite')
    return producer.contributors;
  return producer.matched;
}

await describe({
  name: collectEnvelopeProposals.name,
  children: [
    it({
      name: 'MERGES a duplicate proposal into the first one, crediting both writers, so neither votes at '
        + 'full weight for words it wrote',
      fn: async () => {
        const proposals = collectEnvelopeProposals({
          candidates: [
            proposing({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              newText: 'The cat sleeps on the windowsill.',
            },),
            proposing({
              modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
              newText: 'The cat sleeps on the windowsill.',
            },),
          ],
          envelope: ENVELOPE,
        },);

        expect(proposals.length,).toBe(1,);
        expect([...creditedTo({ producer: proposals[0]?.producer ?? { kind: 'incumbent', matched: [], }, },),]
          .toSorted(),).toEqual([
            SEAT_SYNTHETIC_VISION_WITHHELD,
            SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          ],);
      },
    },),

    it({
      name: 'keeps distinct replacements apart, in roster order, each credited to its writer alone',
      fn: async () => {
        const proposals = collectEnvelopeProposals({
          candidates: [
            proposing({
              modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
              newText: 'The cat sleeps on the windowsill.',
            },),
            proposing({
              modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
              newText: 'The cat naps on the windowsill.',
            },),
          ],
          envelope: ENVELOPE,
        },);

        expect(proposals.map(function toText(proposal,): string {
          return proposal.rendered;
        },),).toEqual([
          'The cat sleeps on the windowsill.',
          'The cat naps on the windowsill.',
        ],);
        expect(proposals.map(function toProducer(proposal,): readonly string[] {
          return creditedTo({ producer: proposal.producer, },);
        },),).toEqual([
          [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
          [SEAT_SYNTHETIC_VISION_WITHHELD,],
        ],);
      },
    },),

    it({
      name: 'contributes nothing for a model that left the envelope alone',
      fn: async () => {
        const proposals = collectEnvelopeProposals({
          candidates: [
            proposing({ modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR, },),
            proposing({
              modelId: SEAT_SYNTHETIC_VISION_WITHHELD,
              newText: 'The cat naps on the windowsill.',
            },),
          ],
          envelope: ENVELOPE,
        },);

        expect(proposals.length,).toBe(1,);
        expect(proposals[0]?.value.newText,).toBe('The cat naps on the windowsill.',);
      },
    },),
  ],
},);
