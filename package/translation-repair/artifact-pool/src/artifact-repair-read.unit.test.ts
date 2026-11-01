/**
 Tests for reading repair provenance back out of a run artifact, including
 the one tolerance: artifacts written before repair recording existed.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ArtifactParseError,
  parseRecordRepair,
} from '../dist/final/node/index.mjs';

/**
 Builds one issue record wrapper carrying repair provenance.

 @param overrides - fields replacing the well-formed defaults

 @returns Record the reader parses

 @example
 ```ts
 const record = catRecord({},);
 ```
 */
function catRecord(
  overrides: Readonly<Record<string, unknown>>,
): Record<string, unknown> {
  return {
    sliceIndex: 0,
    resolved: true,
    repairDisposition: 'shipped',
    refined: false,
    repairRegions: [
      {
        envelopeId: 'envelope/nap',
        issueIds: ['adjudicated/nap',],
        before: 'The cat is doing the sleeping.',
        editorAfter: 'The cat is asleep.',
      },
    ],
    ...overrides,
  };
}

await describe({
  name: parseRecordRepair.name,
  children: [
    it({
      name: 'reads a recorded repair with its regions',
      fn: async () => {
        const reading = parseRecordRepair({
          record: catRecord({},),
          path: 'Kitten issues[0]',
        },);
        expect(reading.kind,).toBe('recorded',);
        if (reading.kind !== 'recorded')
          throw new Error('expected a recorded reading',);
        expect(reading.repair.disposition,).toBe('shipped',);
        expect(reading.repair.regions,).toHaveLength(1,);
        expect(reading.repair.regions[0]?.editorAfter,)
          .toBe('The cat is asleep.',);
        expect(reading.repair.finalSliceText,).toBeUndefined();
      },
    },),

    it({
      name: 'names the absence rather than inventing an empty repair when the '
        + 'artifact predates repair recording, so a legacy round can still be '
        + 'read for detection without entering any repair denominator',
      fn: async () => {
        /** Round-two shaped record: an issue wrapper with no repair fields. */
        const legacy = {
          sliceIndex: 0,
          resolved: true,
          issue: { issueId: 'adjudicated/nap', },
        };
        const reading = parseRecordRepair({
          record: legacy,
          path: 'Kitten issues[0]',
        },);
        expect(reading.kind,).toBe('unrecorded',);
      },
    },),

    it({
      name: 'refuses a PARTIALLY recorded repair rather than reading it as a '
        + 'legacy record, since a half-written repair is a malformed '
        + 'measurement and silently dropping it shortens the denominator',
      fn: async () => {
        /** Failure raised by the record carrying regions but no disposition. */
        let caught: unknown;
        try {
          /** Record with the disposition removed and everything else kept. */
          const partial = catRecord({},);
          delete partial.repairDisposition;
          parseRecordRepair({
            record: partial,
            path: 'Kitten issues[0]',
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(ArtifactParseError,);
        expect((caught as Error).message,).toContain('repairDisposition',);
      },
    },),

    it({
      name: 'reads the final slice text when refinement made the recorded '
        + 'replacement stale',
      fn: async () => {
        const reading = parseRecordRepair({
          record: catRecord({
            refined: true,
            finalSliceText: 'The cat naps in the sun.',
          },),
          path: 'Kitten issues[0]',
        },);
        if (reading.kind !== 'recorded')
          throw new Error('expected a recorded reading',);
        expect(reading.repair.refined,).toBe(true,);
        expect(reading.repair.finalSliceText,).toBe('The cat naps in the sun.',);
      },
    },),

    it({
      name: 'throws when a refined record carries no final slice text, since '
        + 'nothing then states the wording a grader must judge',
      fn: async () => {
        /** Failure raised by the missing final text. */
        let caught: unknown;
        try {
          parseRecordRepair({
            record: catRecord({ refined: true, },),
            path: 'Kitten issues[0]',
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(ArtifactParseError,);
        expect((caught as Error).message,).toContain('finalSliceText',);
      },
    },),

    it({
      name: 'reads a refined WITHDRAWN record that states no final wording, '
        + 'since a slice the document took back came back as the archive wrote '
        + 'it. Requiring the wording here would make every artifact carrying '
        + 'one unreadable, which is the whole measurement lost to a field that '
        + 'has nothing to say',
      fn: async () => {
        const reading = parseRecordRepair({
          record: catRecord({
            repairDisposition: 'withdrawn',
            refined: true,
          },),
          path: 'Kitten issues[0]',
        },);
        if (reading.kind !== 'recorded')
          throw new Error('expected a recorded reading',);
        expect(reading.repair.refined,).toBe(true,);
        expect(reading.repair.finalSliceText,).toBeUndefined();
      },
    },),

    it({
      name: 'still reads a final wording recorded under a disposition that '
        + 'shipped no targeted repair, because the naturalness lane runs '
        + 'whatever the accuracy selection decided and its rewrite does reach '
        + 'the reader',
      fn: async () => {
        const reading = parseRecordRepair({
          record: catRecord({
            repairDisposition: 'not-selected',
            refined: true,
            finalSliceText: 'The cat naps in the sun.',
          },),
          path: 'Kitten issues[0]',
        },);
        if (reading.kind !== 'recorded')
          throw new Error('expected a recorded reading',);
        expect(reading.repair.finalSliceText,).toBe('The cat naps in the sun.',);
      },
    },),

    it({
      name: 'throws on a final wording that is PRESENT but malformed even '
        + 'where none was required, since a field written as something other '
        + 'than text means writer and reader disagree wherever it appears',
      fn: async () => {
        /** Failure raised by the non-string final wording. */
        let caught: unknown;
        try {
          parseRecordRepair({
            record: catRecord({
              repairDisposition: 'withdrawn',
              refined: true,
              finalSliceText: null,
            },),
            path: 'Kitten issues[0]',
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(ArtifactParseError,);
        expect((caught as Error).message,).toContain('finalSliceText',);
      },
    },),

    it({
      name: 'throws on a malformed region rather than dropping it, because a '
        + 'silently short repair record biases what it measures',
      fn: async () => {
        /** Failure raised by the non-string replacement. */
        let caught: unknown;
        try {
          parseRecordRepair({
            record: catRecord({
              repairRegions: [
                {
                  envelopeId: 'envelope/nap',
                  issueIds: ['adjudicated/nap',],
                  before: 'The cat is doing the sleeping.',
                  editorAfter: 42,
                },
              ],
            },),
            path: 'Kitten issues[0]',
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(ArtifactParseError,);
        expect((caught as Error).message,).toContain('editorAfter',);
      },
    },),

    it({
      name: 'throws when a recorded repair has a non-string issue id on a '
        + 'region, which would break attribution to the sampled issue',
      fn: async () => {
        /** Failure raised by the non-string issue id. */
        let caught: unknown;
        try {
          parseRecordRepair({
            record: catRecord({
              repairRegions: [
                {
                  envelopeId: 'envelope/nap',
                  issueIds: [42,],
                  before: '',
                  editorAfter: 'The cat is asleep.',
                },
              ],
            },),
            path: 'Kitten issues[0]',
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(ArtifactParseError,);
        expect((caught as Error).message,).toContain('issueIds',);
      },
    },),
  ],
},);
