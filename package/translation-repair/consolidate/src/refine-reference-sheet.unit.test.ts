/**
 Guards class forty-one (Mio25 slice 2, 2026-09-17): the refine stage's
 sheets carry the cited references the other deciding sheets carry, so a
 refiner and its judges, and the consolidation polish and its gate, read the
 same rule the slate gate read before them. Cat-themed invention throughout;
 no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { messageText, } from '@monochromatic-dev/translation-repair-provider';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import type { EditableEnvelope, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  buildRefineMessages,
  buildRefineSelectionContext,
  refineRunShape,
} from '@monochromatic-dev/translation-repair-repair';
import { buildConsolidationPolishGateMessages, } from '../dist/final/node/index.mjs';

/**
 Reference lines as the preparation renders them, one page.
 */
const REFERENCE_CONTEXT = 'REFERENCE 1 https://example.invalid/cat-diary: The cat has an older sister who also naps.';

/**
 Label every deciding sheet gives the references.
 */
const REFERENCE_HEADING = 'CITED REFERENCES, EVIDENCE ONLY';

/**
 One eligible paragraph of the text under refinement.

 @param baseText - paragraph wording

 @returns Envelope the refiner is shown

 @example
 ```ts
 const envelope = paragraph({ baseText: 'The cat naps.', },);
 ```
 */
function paragraph({ baseText, }: { readonly baseText: string; },): EditableEnvelope {
  return {
    envelopeId: 'paragraph/0',
    startOffset: 0,
    endOffset: baseText.length,
    baseText,
    baseHash: hashContent({ content: baseText, },),
    issueIds: [],
  };
}

/**
 Every user message of a sheet as one string.

 @param messages - sheet under test

 @returns User content joined

 @example
 ```ts
 const sheet = userText({ messages, },);
 ```
 */
function userText(
  { messages, }: { readonly messages: ReturnType<typeof buildConsolidationPolishGateMessages>; },
): string {
  return messages
    .filter(function isUser(message,) {
      return message.role === 'user';
    },)
    .map(function toContent(message,) {
      return messageText({ message, },);
    },)
    .join('\n',);
}

await describe({
  name: 'refine sheets carry the cited references (class forty-one)',
  children: [
    it({
      name: 'SHOWS the references and their rule to the refiner, and nothing when the original cites nowhere',
      fn: async () => {
        const withReferences = buildRefineMessages({
          sourceText: '猫有一个姐姐。',
          envelopes: [paragraph({ baseText: 'The cat has an older sister who also naps.', },),],
          referenceContext: REFERENCE_CONTEXT,
        },);
        const sheet = userText({ messages: withReferences.messages, },);
        expect(sheet,).toContain(REFERENCE_HEADING,);
        expect(sheet,).toContain('who also naps',);
        const without = buildRefineMessages({
          sourceText: '猫有一个姐姐。',
          envelopes: [paragraph({ baseText: 'The cat has an older sister.', },),],
        },);
        expect(userText({ messages: without.messages, },),).not.toContain(REFERENCE_HEADING,);
      },
    },),
    it({
      name: 'GIVES the refine judges the references as labelled evidence in both modes',
      fn: async () => {
        const comparative = buildRefineSelectionContext({
          mode: { kind: 'comparative', },
          sourceText: '猫有一个姐姐。',
          repairedText: 'The cat has an older sister who also naps.',
          referenceContext: REFERENCE_CONTEXT,
        },);
        const comparativeLabels = comparative.evidence.map(function toLabel(entry,) {
          return entry.label;
        },);
        expect(comparativeLabels.some(function isReference(label,) {
          return label.startsWith(REFERENCE_HEADING,);
        },),).toBe(true,);
        const correction = buildRefineSelectionContext({
          mode: {
            kind: 'objection-correction',
            groups: [{
              origin: 'consolidation gate',
              objections: ['Reads as a calque.',],
            },],
          },
          sourceText: '猫有一个姐姐。',
          repairedText: 'The cat has an older sister who also naps.',
          referenceContext: REFERENCE_CONTEXT,
        },);
        const correctionLabels = correction.evidence.map(function toLabel(entry,) {
          return entry.label;
        },);
        expect(correctionLabels.some(function isReference(label,) {
          return label.startsWith(REFERENCE_HEADING,);
        },),).toBe(true,);
        const silent = buildRefineSelectionContext({
          mode: { kind: 'comparative', },
          sourceText: '猫有一个姐姐。',
          repairedText: 'The cat has an older sister.',
        },);
        expect(silent.evidence.some(function isReference(entry,) {
          return entry.label.startsWith(REFERENCE_HEADING,);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'SHOWS the references to the polish gate beside the two candidates',
      fn: async () => {
        const sheet = userText({
          messages: buildConsolidationPolishGateMessages({
            subject: {
              sourceText: '猫有一个姐姐。',
              archiveText: 'The cat has an older sister who also naps.',
              baseText: 'The cat has an older sister who also naps.',
              polishedText: 'The cat has an older sister.',
              lineStructured: false,
              mode: { kind: 'comparative', },
              referenceContext: REFERENCE_CONTEXT,
            },
          },),
        },);
        expect(sheet,).toContain(REFERENCE_HEADING,);
        expect(sheet,).toContain('who also naps',);
      },
    },),
    it({
      name: 'SEPARATES refine cache keys by the references the sheets carried',
      fn: async () => {
        const roster = {
          refinerModelIds: ['minimax-m3',],
          judgeModelIds: ['minimax-m3', 'deepseek-v4.1-flash',],
          checkerModelIds: ['deepseek-v4.1-flash',],
        } as const;
        const withReferences = refineRunShape({
          ...roster,
          referenceContext: REFERENCE_CONTEXT,
        },);
        const without = refineRunShape(roster,);
        expect(withReferences,).not.toBe(without,);
      },
    },),
  ],
},);
