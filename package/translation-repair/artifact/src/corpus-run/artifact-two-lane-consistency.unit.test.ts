/**
 Tests for what the consistency pass says about one slice of a recorded
 section: the text of a slice it changed, and nothing for any other.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ArtifactConsistency,
  CONSISTENCY_CANDIDATE_KINDS,
  CONSISTENCY_CARRY_DECISION_KINDS,
  CONSISTENCY_CARRY_NO_DECISION_REASONS,
  consistencyOverrideAt,
  NO_CONSISTENCY,
} from '../../dist/final/node/index.mjs';

/**
 A section whose pass changed the second slice.
 */
const CHANGED_SECOND: ArtifactConsistency = {
  groups: [],
  slices: [{ sliceIndex: 1, text: '## Two\n\nThe kitten naps on the sunny sill.', },],
  findings: [],
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: consistencyOverrideAt.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE TEXT OF A SLICE THE PASS CHANGED AND NOTHING FOR ANY OTHER, or for a pass that changed none',
          fn: async () => {
            expect([
              consistencyOverrideAt({ consistency: CHANGED_SECOND, sliceIndex: 1, },),
              consistencyOverrideAt({ consistency: CHANGED_SECOND, sliceIndex: 0, },),
              consistencyOverrideAt({ consistency: NO_CONSISTENCY, sliceIndex: 1, },),
            ],).toEqual([
              { kind: 'replaced', text: '## Two\n\nThe kitten naps on the sunny sill.', },
              { kind: 'untouched', },
              { kind: 'untouched', },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: 'the carry vocabularies the reader checks a record against',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAME EVERY KIND OF CANDIDATE, EVERY KIND OF DECISION AND EVERY REASON A CARRY CAN HAVE NO CHOICE, as records '
            + 'keyed by the types so that a member added to a type fails to compile until the record names it',
          fn: async () => {
            expect({
              candidates: Object.keys(CONSISTENCY_CANDIDATE_KINDS,),
              decisions: Object.keys(CONSISTENCY_CARRY_DECISION_KINDS,),
              reasons: Object.keys(CONSISTENCY_CARRY_NO_DECISION_REASONS,),
            },).toEqual({
              candidates: ['current', 'splice', 'writer',],
              decisions: ['chosen', 'none', 'failed',],
              reasons: ['no-alternative', 'no-quorum', 'no-majority',],
            },);
          },
        },),
      ],
    },),
  ],
},);
