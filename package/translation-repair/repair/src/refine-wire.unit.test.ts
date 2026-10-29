/**
 Tests for the refinement wire guard and its binding step.

 Both read untrusted model output. `isRefineReportWire` is the only thing
 standing between a malformed reply and code that assumes `rewrites` is an
 array of well-shaped rewrites, and `resolveRefineRewrites` decides what a
 miscounted paragraph number does. Neither had a test.

 The binding step's stated contract is that it drops rather than throws: a
 rewriter miscounting its own list says nothing about the paragraphs it got
 right. So the cases in this file check that a bad item is recorded AND dropped,
 while its well-formed neighbours survive.

 Fixtures are cat-themed invention.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { EditableEnvelope, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  isRefineReportWire,
  resolveRefineRewrites,
} from '../dist/final/node/index.mjs';

/**
 Builds an eligible paragraph envelope.

 @param envelopeId - handle the operation carries

 @param startOffset - absolute start of the paragraph

 @param baseText - paragraph text the rewrite replaces

 @returns Envelope in prompt numbering order

 @example
 ```ts
 const envelope = paragraph({ envelopeId: 'envelope/0', startOffset: 0, baseText: 'The cat naps.', },);
 ```
 */
function paragraph(
  {
    envelopeId,
    startOffset,
    baseText,
  }: {
    readonly envelopeId: string;
    readonly startOffset: number;
    readonly baseText: string;
  },
): EditableEnvelope {
  return {
    envelopeId,
    startOffset,
    endOffset: startOffset + baseText.length,
    baseText,
    baseHash: `hash/${envelopeId}`,
    issueIds: [],
  };
}

/**
 Two eligible paragraphs, which makes paragraph 3 out of range.
 */
const ENVELOPES: readonly EditableEnvelope[] = [
  paragraph({
    envelopeId: 'envelope/0',
    startOffset: 0,
    baseText: 'The cat is doing the sleeping.',
  },),
  paragraph({
    envelopeId: 'envelope/1',
    startOffset: 32,
    baseText: 'She is doing the chasing of butterflies.',
  },),
];

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isRefineReportWire.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'accepts an empty rewrites array, since proposing nothing is a '
            + 'real answer meaning no paragraph was worth changing',
          fn: async () => {
            expect(isRefineReportWire({ rewrites: [], },),).toBe(true,);
          },
        },),

        it({
          name: 'accepts well-formed rewrites and ignores extra keys the model '
            + 'volunteered, since a chatty reply is still a usable one',
          fn: async () => {
            expect(
              isRefineReportWire({
                rewrites: [
                  {
                    paragraph: 1,
                    newText: 'The cat sleeps.',
                  },
                ],
                commentary: 'I also fixed the tone.',
              },),
            ).toBe(true,);
          },
        },),

        it({
          name: 'refuses a rewrites value that is not an array, notably the single '
            + 'object a model sends when it had exactly one rewrite and forgot to '
            + 'wrap it',
          fn: async () => {
            expect(
              isRefineReportWire({
                rewrites: {
                  paragraph: 1,
                  newText: 'The cat sleeps.',
                },
              },),
            ).toBe(false,);
            expect(isRefineReportWire({ rewrites: null, },),).toBe(false,);
            expect(isRefineReportWire({},),).toBe(false,);
          },
        },),

        it({
          name: 'refuses the whole report when ANY single rewrite is malformed, '
            + 'rather than keeping the good ones: the guard is a type predicate, '
            + 'so admitting a report means every element is safe to read',
          fn: async () => {
            expect(
              isRefineReportWire({
                rewrites: [
                  {
                    paragraph: 1,
                    newText: 'The cat sleeps.',
                  },
                  {
                    paragraph: 2,
                    newText: 42,
                  },
                ],
              },),
            ).toBe(false,);
          },
        },),

        it({
          name: 'refuses a rewrite that is no record at all, so no field is read off a number or a '
            + 'string',
          fn: async () => {
            expect(isRefineReportWire({ rewrites: [5,], },),).toBe(false,);
            expect(isRefineReportWire({ rewrites: ['The cat sleeps.',], },),).toBe(false,);
            expect(isRefineReportWire({ rewrites: [null,], },),).toBe(false,);
          },
        },),

        it({
          name: 'refuses a zero or negative paragraph number, because numbering is '
            + 'one-based and a zero would silently bind to the paragraph before '
            + 'the first',
          fn: async () => {
            for (const paragraphNumber of [
              0,
              -1,
            ])
              expect(
                isRefineReportWire({
                  rewrites: [
                    {
                      paragraph: paragraphNumber,
                      newText: 'The cat sleeps.',
                    },
                  ],
                },),
              ).toBe(false,);
          },
        },),

        it({
          name: 'refuses a fractional or non-finite paragraph number, which would '
            + 'index nothing while still passing a bare typeof check',
          fn: async () => {
            for (const paragraphNumber of [
              1.5,
              Number.NaN,
              Number.POSITIVE_INFINITY,
            ])
              expect(
                isRefineReportWire({
                  rewrites: [
                    {
                      paragraph: paragraphNumber,
                      newText: 'The cat sleeps.',
                    },
                  ],
                },),
              ).toBe(false,);
          },
        },),

        it({
          name: 'refuses a numeric string paragraph, the shape a model produces '
            + 'when it quotes every field',
          fn: async () => {
            expect(
              isRefineReportWire({
                rewrites: [
                  {
                    paragraph: '1',
                    newText: 'The cat sleeps.',
                  },
                ],
              },),
            ).toBe(false,);
          },
        },),

        it({
          name: 'refuses a non-record entirely, including null and an array, so a '
            + 'bare list of rewrites cannot pass as a report',
          fn: async () => {
            for (const value of [
              null,
              undefined,
              'rewrites',
              7,
              [
                {
                  paragraph: 1,
                  newText: 'The cat sleeps.',
                },
              ],
            ])
              expect(isRefineReportWire(value,),).toBe(false,);
          },
        },),

        it({
          name: 'ACCEPTS an empty replacement string, since deleting a paragraph '
            + 'is a legitimate proposal and the eligibility gate, not the wire '
            + 'guard, is what decides whether it may ship',
          fn: async () => {
            expect(
              isRefineReportWire({
                rewrites: [
                  {
                    paragraph: 1,
                    newText: '',
                  },
                ],
              },),
            ).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: resolveRefineRewrites.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'binds each rewrite to the paragraph its one-based number names, '
            + 'carrying that envelope\'s id and base hash so the patch gate can '
            + 'detect a stale base',
          fn: async () => {
            /**
             Resolution over both paragraphs, named out of order on purpose.
             */
            const { operations, findings, } = resolveRefineRewrites({
              wire: {
                rewrites: [
                  {
                    paragraph: 2,
                    newText: 'She chases butterflies.',
                  },
                  {
                    paragraph: 1,
                    newText: 'The cat sleeps.',
                  },
                ],
              },
              envelopes: ENVELOPES,
            },);

            expect(findings,).toStrictEqual([],);
            // Wire order, not envelope order.
            expect(operations.map(function toId(operation,) {
              return operation.envelopeId;
            },),).toStrictEqual([
              'envelope/1',
              'envelope/0',
            ],);
            expect(operations[0]?.baseHash,).toBe('hash/envelope/1',);
            expect(operations[0]?.newText,).toBe('She chases butterflies.',);
          },
        },),

        it({
          name: 'RECORDS AND DROPS a paragraph outside the sheet while keeping its '
            + 'well-formed neighbours, which is the stated contract: a rewriter '
            + 'miscounting its own list says nothing about the paragraphs it got '
            + 'right',
          fn: async () => {
            /**
             Resolution where the first rewrite names a paragraph that is not there.
             */
            const { operations, findings, } = resolveRefineRewrites({
              wire: {
                rewrites: [
                  {
                    paragraph: 3,
                    newText: 'The kitten pounces.',
                  },
                  {
                    paragraph: 1,
                    newText: 'The cat sleeps.',
                  },
                ],
              },
              envelopes: ENVELOPES,
            },);

            expect(findings,).toStrictEqual(['refine-unknown-paragraph (3)',],);
            expect(operations,).toStrictEqual([
              {
                envelopeId: 'envelope/0',
                baseHash: 'hash/envelope/0',
                newText: 'The cat sleeps.',
              },
            ],);
          },
        },),

        it({
          name: 'keeps the FIRST rewrite of a repeated paragraph and records the '
            + 'repeat, so two proposals for one paragraph cannot both apply and '
            + 'produce overlapping operations',
          fn: async () => {
            /**
             Resolution where paragraph 1 is named twice.
             */
            const { operations, findings, } = resolveRefineRewrites({
              wire: {
                rewrites: [
                  {
                    paragraph: 1,
                    newText: 'The cat sleeps.',
                  },
                  {
                    paragraph: 1,
                    newText: 'The cat naps.',
                  },
                ],
              },
              envelopes: ENVELOPES,
            },);

            expect(findings,).toStrictEqual(['refine-duplicate-paragraph (1)',],);
            expect(operations.length,).toBe(1,);
            expect(operations[0]?.newText,).toBe('The cat sleeps.',);
          },
        },),

        it({
          name: 'records every irregularity rather than stopping at the first, so '
            + 'one scorecard reading shows the whole shape of a bad reply',
          fn: async () => {
            /**
             Resolution mixing two unknown paragraphs and a duplicate.
             */
            const { operations, findings, } = resolveRefineRewrites({
              wire: {
                rewrites: [
                  {
                    paragraph: 9,
                    newText: 'The kitten pounces.',
                  },
                  {
                    paragraph: 1,
                    newText: 'The cat sleeps.',
                  },
                  {
                    paragraph: 7,
                    newText: 'The kitten purrs.',
                  },
                  {
                    paragraph: 1,
                    newText: 'The cat naps.',
                  },
                ],
              },
              envelopes: ENVELOPES,
            },);

            expect(findings,).toStrictEqual([
              'refine-unknown-paragraph (9)',
              'refine-unknown-paragraph (7)',
              'refine-duplicate-paragraph (1)',
            ],);
            expect(operations,).toStrictEqual([
              {
                envelopeId: 'envelope/0',
                baseHash: 'hash/envelope/0',
                newText: 'The cat sleeps.',
              },
            ],);
          },
        },),

        it({
          name: 'returns nothing at all, without findings, for a reply proposing '
            + 'no rewrites, so a rewriter that saw nothing to improve is not '
            + 'reported as an irregularity',
          fn: async () => {
            /**
             Resolution over an empty proposal list.
             */
            const { operations, findings, } = resolveRefineRewrites({
              wire: { rewrites: [], },
              envelopes: ENVELOPES,
            },);

            expect(operations,).toStrictEqual([],);
            expect(findings,).toStrictEqual([],);
          },
        },),

        it({
          name: 'drops every rewrite when no paragraph was eligible, recording one '
            + 'finding each rather than binding to whatever happened to be at '
            + 'index zero of an empty list',
          fn: async () => {
            /**
             Resolution against no eligible paragraphs at all.
             */
            const { operations, findings, } = resolveRefineRewrites({
              wire: {
                rewrites: [
                  {
                    paragraph: 1,
                    newText: 'The cat sleeps.',
                  },
                ],
              },
              envelopes: [],
            },);

            expect(operations,).toStrictEqual([],);
            expect(findings,).toStrictEqual(['refine-unknown-paragraph (1)',],);
          },
        },),

        it({
          name: 'FOLDS an invisible variant out of a rewrite at intake and names it, so the judges see '
            + 'the bytes that ship',
          fn: async () => {
            const { operations, findings, } = resolveRefineRewrites({
              wire: {
                rewrites: [
                  {
                    paragraph: 1,
                    newText: 'The cat\u00A0sleeps.',
                  },
                ],
              },
              envelopes: ENVELOPES,
            },);

            expect(operations[0]?.newText,).toBe('The cat sleeps.',);
            expect(findings,).toStrictEqual(['invisible-variant-folded (U+00A0 x1)',],);
          },
        },),
      ],
    },),
  ],
},);
