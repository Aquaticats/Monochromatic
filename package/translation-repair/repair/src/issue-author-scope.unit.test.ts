/**
 Tests for how far one author's credit reaches: the per-envelope split a
 shipped composite earns, and the refiners layered on top of it.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { RepairJudgedRound, } from '@monochromatic-dev/translation-repair-assembly';
import type { PatchOperation, } from '@monochromatic-dev/translation-repair-page-pass';
import type { CandidateProducer, } from '@monochromatic-dev/translation-repair-house-rule';
import {
  collectIssueAuthors,
  type EditorStageResult,
} from '../dist/final/node/index.mjs';
import {
  envelopeOf,
  selectedRound,
  slateEntryOf,
} from '@monochromatic-dev/translation-repair-assembly/ts/issue-author-fixtures.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Issue the cases in this file credit.
 */
const WHISKER = 'adjudicated/whisker';

/**
 Model that wins the envelope rounds in this file unless a case says otherwise.
 */
const AUTHOR: RosterModelId = SEAT_HYPER_OPENROUTER_VISION_EDITOR;

/**
 Second model, for composites and for candidates that lose.
 */
const HELPER: RosterModelId = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

/**
 What ships in every `collectIssueAuthors` case here.

 A COMPOSITE, deliberately: it is the one shipped patch whose parts have
 different authors, so it is the only producer for which the envelope rounds
 are consulted at all.
 */
const COMPOSITE: CandidateProducer = {
  kind: 'composite',
  contributors: [
    AUTHOR,
    HELPER,
  ],
};

/**
 Builds an applied patch operation against one envelope.
 */
function operationOf(envelopeId: string,): PatchOperation {
  return {
    envelopeId,
    baseHash: `hash-${envelopeId}`,
    newText: 'the cat dozes',
  };
}

/**
 Builds the editor stage result, with the composite as the shipped producer.
 */
function compositeShipped(
  {
    rounds,
    applied,
  }: {
    readonly rounds: readonly RepairJudgedRound[];
    readonly applied: readonly PatchOperation[];
  },
): EditorStageResult {
  return {
    patch: {
      patchedText: 'the cat dozes',
      applied,
      rejected: [],
    },
    heardEditors: 3,
    rounds,
    findings: [],
    shippedProducer: COMPOSITE,
    // These fixtures read authorship only, so no gate is re-applied to them.
    preservation: { mode: 'skip', },
  };
}

await describe({
  name: collectIssueAuthors.name,
  children: [
    it({
      name: 'JOINS THE WINNER BY THE NUMBER JUDGES WERE SHOWN, NOT BY ARRAY POSITION. Numbers are '
        + 'ONE-BASED, so the winner numbered 1 sits at position 0 and indexing the array by '
        + 'selectedIndex silently names the LOSER instead. This is the shape that discriminates: '
        + 'it fails with a wrong answer rather than with a throw',
      fn: async function oneBasedNumbersAreNotArrayPositions() {
        expect(collectIssueAuthors({
          editor: compositeShipped({
            applied: [operationOf('kept',),],
            rounds: [
              selectedRound({
                envelopeId: 'kept',
                slate: [
                  slateEntryOf({
                    index: 1,
                    modelId: AUTHOR,
                  },),
                  slateEntryOf({
                    index: 2,
                    modelId: HELPER,
                  },),
                ],
                selectedIndex: 1,
              },),
            ],
          },),
          envelopes: [
            envelopeOf({
              envelopeId: 'kept',
              issueIds: [WHISKER,],
            },),
          ],
        },),).toEqual({
          perIssue: { [WHISKER]: [AUTHOR,], },
          everyIssue: [],
        },);
      },
    },),

    it({
      name: 'MERGES THE AUTHORS OF TWO ENVELOPES SERVING ONE ISSUE, so neither writer of that '
        + 'issue text keeps a whole vote on it',
      fn: async function twoEnvelopesMergeOntoOneIssue() {
        expect(collectIssueAuthors({
          editor: compositeShipped({
            applied: [
              operationOf('front',),
              operationOf('back',),
            ],
            rounds: [
              selectedRound({
                envelopeId: 'front',
                slate: [
                  slateEntryOf({
                    index: 1,
                    modelId: AUTHOR,
                  },),
                ],
                selectedIndex: 1,
              },),
              selectedRound({
                envelopeId: 'back',
                slate: [
                  slateEntryOf({
                    index: 1,
                    modelId: HELPER,
                  },),
                ],
                selectedIndex: 1,
              },),
            ],
          },),
          envelopes: [
            envelopeOf({
              envelopeId: 'front',
              issueIds: [WHISKER,],
            },),
            envelopeOf({
              envelopeId: 'back',
              issueIds: [WHISKER,],
            },),
          ],
        },),).toEqual({
          perIssue: {
            [WHISKER]: [
              AUTHOR,
              HELPER,
            ],
          },
          everyIssue: [],
        },);
      },
    },),

    it({
      name: 'NAMES EVERY CONTRIBUTOR TO A COMPOSITE THAT WON AN ENVELOPE ROUND, because each of '
        + 'them has a stake in the text that composite put in',
      fn: async function compositesNameEveryContributor() {
        expect(collectIssueAuthors({
          editor: compositeShipped({
            applied: [operationOf('kept',),],
            rounds: [
              selectedRound({
                envelopeId: 'kept',
                slate: [
                  {
                    index: 1,
                    rendered: 'assembled',
                    hash: 'slate-1',
                    producer: COMPOSITE,
                  },
                ],
                selectedIndex: 1,
              },),
            ],
          },),
          envelopes: [
            envelopeOf({
              envelopeId: 'kept',
              issueIds: [WHISKER,],
            },),
          ],
        },),).toEqual({
          perIssue: {
            [WHISKER]: [
              AUTHOR,
              HELPER,
            ],
          },
          everyIssue: [],
        },);
      },
    },),
  ],
},);
