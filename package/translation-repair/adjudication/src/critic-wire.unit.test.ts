/**
 Tests for critic wire guarding and quote-to-anchor resolution.
 Fixtures are cat-themed invention only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  type CriticIssueWire,
  isCriticReportWire,
  resolveCriticIssue,
} from '../dist/final/node/index.mjs';

/**
 Parsed pair resolutions anchor against.
 */
const DOCUMENTS = {
  source: parseDocument({
    text: '---\nname: 小猫-whiskers\n---\n\n## 简介\n\n猫猫喜欢晒太阳，也喜欢追蝴蝶。\n\n猫猫会打呼噜。\n',
  },),
  target: parseDocument({
    text: '---\nname: 小猫-whiskers\n---\n\n## Introduction\n\nThe cat likes to nap in the sun.\n\nThe cat purrs.\n',
  },),
} as const;

/**
 Fully valid wire issue used as the base for corruptions.
 */
const VALID_WIRE: CriticIssueWire = {
  category: 'accuracy/omission',
  severity: 'major',
  summary: '追蝴蝶那句没有翻译。',
  sourceQuote: '也喜欢追蝴蝶',
  targetQuote: 'The cat likes to nap in the sun.',
};

/**
 Reads the failure reason of one corrupted wire.

 @param wire - corrupted wire under test

 @returns Failure reason, empty when resolution unexpectedly succeeded

 @example
 ```ts
 reasonOf({ ...VALID_WIRE, category: 'accuracy/vibes', },);
 ```
 */
function reasonOf(wire: CriticIssueWire,): string {
  /** Resolution of the corrupted wire. */
  const resolution = resolveCriticIssue({
    wire,
    documents: DOCUMENTS,
  },);
  return resolution.resolved
    ? ''
    : resolution.reason;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isCriticReportWire.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'accepts well-formed reports including empty ones',
          fn: async () => {
            expect(isCriticReportWire({ issues: [], },),).toBe(true,);
            expect(isCriticReportWire({ issues: [VALID_WIRE,], },),).toBe(true,);
            expect(isCriticReportWire({
              issues: [{ category: 'x', severity: 'y', summary: 'z', },],
            },),).toBe(true,);
          },
        },),

        it({
          name: 'rejects malformed reports',
          fn: async () => {
            expect(isCriticReportWire({},),).toBe(false,);
            expect(isCriticReportWire({ issues: 'many', },),).toBe(false,);
            expect(isCriticReportWire({ issues: [{ category: 1, },], },),).toBe(false,);
            expect(isCriticReportWire({
              issues: [{ category: 'x', severity: 'y', summary: 'z', targetQuote: 7, },],
            },),).toBe(false,);
          },
        },),

        it({
          name: 'rejects a non-record report, a non-record issue, and issues with '
            + 'non-string severity, summary, or sourceQuote',
          fn: async () => {
            expect(isCriticReportWire(null,),).toBe(false,);
            expect(isCriticReportWire({ issues: [7,], },),).toBe(false,);
            expect(isCriticReportWire({
              issues: [{ category: 'x', severity: 1, summary: 'z', },],
            },),).toBe(false,);
            expect(isCriticReportWire({
              issues: [{ category: 'x', severity: 'y', summary: 1, },],
            },),).toBe(false,);
            expect(isCriticReportWire({
              issues: [{ category: 'x', severity: 'y', summary: 'z', sourceQuote: 1, },],
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: resolveCriticIssue.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'resolves dual quotes into a validated anchored claim',
          fn: async () => {
            /** Resolution of the fully valid wire issue. */
            const resolution = resolveCriticIssue({
              wire: VALID_WIRE,
              documents: DOCUMENTS,
            },);
            expect(resolution.resolved,).toBe(true,);
            if (resolution.resolved) {
              expect(resolution.claim.spans,).toHaveLength(2,);
              expect(resolution.claim.spans[0]?.side,).toBe('source',);
              expect(resolution.claim.spans[1]?.side,).toBe('target',);
              expect(resolution.claim.spans[1]?.quotedText,)
                .toBe('The cat likes to nap in the sun.',);
            }
          },
        },),

        it({
          name: 'fails closed vocabularies, missing quotes, and empty quotes',
          fn: async () => {
            expect(reasonOf({ ...VALID_WIRE, category: 'accuracy/vibes', },),)
              .toContain('unknown-category',);
            expect(reasonOf({ ...VALID_WIRE, severity: 'apocalyptic', },),)
              .toContain('unknown-severity',);
            expect(reasonOf({
              category: 'accuracy/omission',
              severity: 'major',
              summary: 'no anchors at all',
            },),).toBe('no-quotes',);
            expect(reasonOf({
              category: 'accuracy/omission',
              severity: 'major',
              summary: 'empty target evidence',
              targetQuote: '',
            },),).toContain('empty-quote',);
          },
        },),

        it({
          name: 'fails absent, ambiguous, and outside-block quotes',
          fn: async () => {
            expect(reasonOf({ ...VALID_WIRE, targetQuote: 'The dog barks.', },),)
              .toBe('quote-not-found (target) needle=14 chars, 3 Latin tokens',);
            expect(reasonOf({ ...VALID_WIRE, targetQuote: 'The cat', },),)
              .toBe('ambiguous-quote (target)',);
            // Front matter text locates uniquely but belongs to no block node.
            expect(reasonOf({
              ...VALID_WIRE,
              targetQuote: 'name: 小猫-whiskers',
            },),).toBe('quote-outside-blocks (target)',);
          },
        },),

        it({
          name: 'splits block-crossing quotes into per-node spans',
          fn: async () => {
            /** Resolution of a quote spanning two paragraphs. */
            const resolution = resolveCriticIssue({
              wire: {
                category: VALID_WIRE.category,
                severity: VALID_WIRE.severity,
                summary: VALID_WIRE.summary,
                targetQuote: 'in the sun.\n\nThe cat purrs.',
              },
              documents: DOCUMENTS,
            },);

            expect(resolution.resolved,).toBe(true,);
            if (resolution.resolved) {
              expect(resolution.claim.spans,).toHaveLength(2,);
              expect(resolution.claim.spans[0]?.quotedText,).toBe('in the sun.',);
              expect(resolution.claim.spans[1]?.quotedText,).toBe('The cat purrs.',);
              expect(resolution.claim.spans[0]?.nodeId,)
                .not
                .toBe(resolution.claim.spans[1]?.nodeId,);
            }
          },
        },),

        it({
          name: 'resolves a target quote copied from the text without its notes between translators, '
            + 'across the place a note was cut from, as spans that leave the note out',
          fn: async () => {
            /** Target whose paragraph holds a note between translators at a line end. */
            const targetText = '## Introduction\n\nThe cat likes the sun.{/* 这句不太对劲 */}\nThen it naps on the cushion.\n';
            /** Pair whose translation holds the note. */
            const documents = {
              source: DOCUMENTS.source,
              target: parseDocument({ text: targetText, },),
            } as const;
            /** Resolution of a quote across the cut note, the original holding no such note. */
            const resolution = resolveCriticIssue({
              wire: {
                category: VALID_WIRE.category,
                severity: VALID_WIRE.severity,
                summary: VALID_WIRE.summary,
                targetQuote: 'The cat likes the sun.\nThen it naps on the cushion.',
              },
              documents,
              original: DOCUMENTS.source.text,
            },);
            expect(resolution.resolved,).toBe(true,);
            if (resolution.resolved) {
              expect(
                resolution.claim.spans.map(function quotedOf(span,): string {
                  return span.quotedText;
                },),
              ).toStrictEqual(['The cat likes the sun.', 'Then it naps on the cushion.',],);
            }
          },
        },),

        it({
          name: 'refuses the same quote when no original is given, since the note then stands in the text it reads',
          fn: async () => {
            /** Target whose paragraph holds a note between translators at a line end. */
            const targetText = '## Introduction\n\nThe cat likes the sun.{/* 这句不太对劲 */}\nThen it naps on the cushion.\n';
            /** Resolution with no original: the text is read as it stands. */
            const resolution = resolveCriticIssue({
              wire: {
                category: VALID_WIRE.category,
                severity: VALID_WIRE.severity,
                summary: VALID_WIRE.summary,
                targetQuote: 'The cat likes the sun.\nThen it naps on the cushion.',
              },
              documents: {
                source: DOCUMENTS.source,
                target: parseDocument({ text: targetText, },),
              },
            },);
            expect(resolution.resolved,).toBe(false,);
          },
        },),

        it({
          name: 'rescues punctuation-variant quotes with canonical document bytes',
          fn: async () => {
            /** Pair whose translation uses curly punctuation. */
            const documents = {
              source: DOCUMENTS.source,
              target: parseDocument({
                text: '## Introduction\n\nThe cat’s sunbeam is “warm” today.\n',
              },),
            } as const;
            /** Resolution of an ASCII-punctuation quote. */
            const resolution = resolveCriticIssue({
              wire: {
                category: VALID_WIRE.category,
                severity: VALID_WIRE.severity,
                summary: VALID_WIRE.summary,
                targetQuote: 'The cat\'s sunbeam is "warm" today.',
              },
              documents,
            },);

            expect(resolution.resolved,).toBe(true,);
            if (resolution.resolved) {
              // The anchor carries the document's canonical curly bytes.
              expect(resolution.claim.spans[0]?.quotedText,)
                .toBe('The cat’s sunbeam is “warm” today.',);
            }
          },
        },),

        it({
          name: 'rescues corner-bracket variants on the source side',
          fn: async () => {
            /** Resolution of a zh quote closed with a curly quote instead of 」. */
            const resolution = resolveCriticIssue({
              wire: {
                category: VALID_WIRE.category,
                severity: VALID_WIRE.severity,
                summary: VALID_WIRE.summary,
                sourceQuote: '猫猫会打呼噜。”',
              },
              documents: {
                source: parseDocument({
                  text: '## 简介\n\n她说：「猫猫会打呼噜。」\n\n猫猫也追蝴蝶。\n',
                },),
                target: DOCUMENTS.target,
              },
            },);

            expect(resolution.resolved,).toBe(true,);
            if (resolution.resolved) {
              // The anchor carries the document's corner-bracket bytes.
              expect(resolution.claim.spans[0]?.quotedText,).toBe('猫猫会打呼噜。」',);
            }
          },
        },),

        it({
          name: 'remaps known leaves reported under the wrong family',
          fn: async () => {
            /** Resolution of a family-slipped category. */
            const resolution = resolveCriticIssue({
              wire: {
                ...VALID_WIRE,
                category: 'fluency/awkward-phrasing',
              },
              documents: DOCUMENTS,
            },);
            expect(resolution.resolved,).toBe(true,);
            if (resolution.resolved)
              expect(resolution.claim.category,).toBe('style/awkward-phrasing',);
          },
        },),

        it({
          name: 'refuses a claim when caller-supplied documents list two nodes '
            + 'sharing one id, naming the repeated id rather than a stale hash',
          fn: async () => {
            /** Decoy paragraph sharing the real node's id but listed first. */
            const decoyText = 'Decoy paragraph nobody quotes.';
            /** Paragraph the quote actually lives in. */
            const realText = 'The cat sleeps on the windowsill.';
            /** Target text holding both paragraphs. */
            const targetText = `${decoyText}\n\n${realText}\n`;
            /** Offset of the decoy paragraph within targetText. */
            const decoyStart = targetText.indexOf(decoyText,);
            /** Offset of the real paragraph within targetText. */
            const realStart = targetText.indexOf(realText,);

            /**
             Resolution against a target side whose nodes list the decoy
             before the real node even though both carry id 'block/0':
             `bindQuoteRegion` anchors the quote to the overlapping (real)
             node, and `validateSpanAnchor` finds two nodes under that id.
             It used to take the decoy, the one listed first, and report a
             stale hash (ledger B129).
             */
            const resolution = resolveCriticIssue({
              wire: {
                category: 'accuracy/omission',
                severity: 'major',
                summary: 'duplicate node id disagreement',
                targetQuote: realText,
              },
              documents: {
                source: DOCUMENTS.source,
                target: {
                  text: targetText,
                  nodes: [
                    {
                      id: 'block/0',
                      zone: 'body',
                      kind: 'paragraph',
                      text: decoyText,
                      startOffset: decoyStart,
                      endOffset: decoyStart + decoyText.length,
                      contentHash: hashContent({ content: decoyText, },),
                    },
                    {
                      id: 'block/0',
                      zone: 'body',
                      kind: 'paragraph',
                      text: realText,
                      startOffset: realStart,
                      endOffset: realStart + realText.length,
                      contentHash: hashContent({ content: realText, },),
                    },
                  ],
                },
              },
            },);

            expect(resolution.resolved,).toBe(false,);
            if (!resolution.resolved)
              expect(resolution.reason,).toBe('anchor-validation (repeated-node-id)',);
          },
        },),
      ],
    },),
  ],
},);
