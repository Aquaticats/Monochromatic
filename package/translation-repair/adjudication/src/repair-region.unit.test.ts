/**
 Tests for attributing applied operations to the accepted issues their
 envelope was cut for.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { PatchOperation, } from '@monochromatic-dev/translation-repair-page-pass';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import type { EditableEnvelope, } from '@monochromatic-dev/translation-repair-shared-contract';
import { collectRepairRegions, } from '../dist/final/node/index.mjs';

/**
 Builds one envelope over the given base text and served issues.

 @param envelopeId - envelope identity operations name

 @param baseText - text occupying the envelope

 @param issueIds - accepted issues the envelope was cut for

 @returns Envelope the collector reads

 @example
 ```ts
 const envelope = catEnvelope({ envelopeId: 'envelope/nap', baseText: 'naps', issueIds: [], },);
 ```
 */
function catEnvelope(
  {
    envelopeId,
    baseText,
    issueIds,
  }: {
    readonly envelopeId: string;
    readonly baseText: string;
    readonly issueIds: readonly string[];
  },
): EditableEnvelope {
  return {
    envelopeId,
    startOffset: 0,
    endOffset: baseText.length,
    baseText,
    baseHash: hashContent({ content: baseText, },),
    issueIds,
  };
}

/**
 Builds one applied operation against an envelope.

 @param envelopeId - envelope the operation targets

 @param newText - replacement text

 @returns Operation the collector reads

 @example
 ```ts
 const operation = catOperation({ envelopeId: 'envelope/nap', newText: 'sleeps', },);
 ```
 */
function catOperation(
  {
    envelopeId,
    newText,
  }: {
    readonly envelopeId: string;
    readonly newText: string;
  },
): PatchOperation {
  return {
    envelopeId,
    baseHash: 'unread-by-the-collector',
    newText,
  };
}

await describe({
  name: collectRepairRegions.name,
  children: [
    it({
      name: 'records one region per applied operation, carrying the text it '
        + 'replaced and the text it wrote',
      fn: async () => {
        const regions = collectRepairRegions({
          envelopes: [
            catEnvelope({
              envelopeId: 'envelope/nap',
              baseText: 'The cat is doing the sleeping.',
              issueIds: ['adjudicated/nap',],
            },),
          ],
          applied: [
            catOperation({
              envelopeId: 'envelope/nap',
              newText: 'The cat is asleep.',
            },),
          ],
        },);
        expect(regions,).toHaveLength(1,);
        expect(regions[0]?.before,).toBe('The cat is doing the sleeping.',);
        expect(regions[0]?.editorAfter,).toBe('The cat is asleep.',);
        expect(regions[0]?.issueIds,).toEqual(['adjudicated/nap',],);
      },
    },),

    it({
      name: 'keeps every issue a merged envelope serves on the one region, '
        + 'which is what stops a shared edit reading as one issue\'s own repair',
      fn: async () => {
        // Envelope merging is why this matters: two accepted issues whose
        // target spans overlap become ONE envelope, so one replacement is
        // written for both and may fix only one of them.
        const regions = collectRepairRegions({
          envelopes: [
            catEnvelope({
              envelopeId: 'envelope/shared',
              baseText: 'The cat, she chase the butterfly.',
              issueIds: [
                'adjudicated/grammar',
                'adjudicated/omission',
              ],
            },),
          ],
          applied: [
            catOperation({
              envelopeId: 'envelope/shared',
              newText: 'The cat chases the butterfly in the garden.',
            },),
          ],
        },);
        expect(regions[0]?.issueIds,).toEqual([
          'adjudicated/grammar',
          'adjudicated/omission',
        ],);
      },
    },),

    it({
      name: 'records an insertion as an empty before, since a zero-width '
        + 'envelope is a real place with no text at it',
      fn: async () => {
        const regions = collectRepairRegions({
          envelopes: [
            catEnvelope({
              envelopeId: 'envelope/insert',
              baseText: '',
              issueIds: ['adjudicated/dropped',],
            },),
          ],
          applied: [
            catOperation({
              envelopeId: 'envelope/insert',
              newText: ' She purrs afterwards.',
            },),
          ],
        },);
        expect(regions[0]?.before,).toBe('',);
        expect(regions[0]?.editorAfter,).toBe(' She purrs afterwards.',);
      },
    },),

    it({
      name: 'records nothing when no operation survived the apply gate',
      fn: async () => {
        const regions = collectRepairRegions({
          envelopes: [
            catEnvelope({
              envelopeId: 'envelope/nap',
              baseText: 'naps',
              issueIds: ['adjudicated/nap',],
            },),
          ],
          applied: [],
        },);
        expect(regions,).toHaveLength(0,);
      },
    },),

    it({
      name: 'throws on an operation naming an unknown envelope rather than '
        + 'dropping it, because the apply gate already refuses those and a '
        + 'silent drop would understate what the pipeline wrote',
      fn: async () => {
        /** Failure raised by attributing an unattributable operation. */
        let caught: unknown;
        try {
          collectRepairRegions({
            envelopes: [],
            applied: [
              catOperation({
                envelopeId: 'envelope/ghost',
                newText: 'nothing anchors this',
              },),
            ],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
      },
    },),
  ],
},);
