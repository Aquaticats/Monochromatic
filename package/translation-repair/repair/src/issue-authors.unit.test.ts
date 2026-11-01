/**
 Tests for who wrote the text a checker stage is about to judge, read off the
 producer the editor stage recorded.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  RepairJudgedRound,
  RepairSlateEntry,
} from '@monochromatic-dev/translation-repair-assembly';
import type { PatchOperation, } from '@monochromatic-dev/translation-repair-page-pass';
import {
  appliedIssuesByEnvelope,
  collectIssueAuthors,
  type EditorStageResult,
  NOBODY_WROTE_IT,
  type ShippedProducer,
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
 Issue the first envelope serves.
 */
const WHISKER = 'adjudicated/whisker';

/**
 Issue the second envelope serves.
 */
const PAW = 'adjudicated/paw';

/**
 Model that wrote the text that ships unless a case says otherwise.
 */
const AUTHOR: RosterModelId = SEAT_HYPER_OPENROUTER_VISION_EDITOR;

/**
 Second model, for rivals and for candidates that lose.
 */
const HELPER: RosterModelId = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

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
 Builds the record of an envelope whose sole proposal was adopted without a
 vote.

 @param envelopeId - envelope adopted

 @param slate - its one entry

 @returns Adopted round

 @example
 ```ts
 const round = adoptedRound({ envelopeId: 'kept', slate: [entry,], },);
 ```
 */
function adoptedRound(
  {
    envelopeId,
    slate,
  }: {
    readonly envelopeId: string;
    readonly slate: readonly RepairSlateEntry[];
  },
): RepairJudgedRound {
  return {
    kind: 'adopted',
    stage: 'envelope',
    envelopeId,
    slate,
    ballots: [],
    tally: {
      judgesAvailable: 0,
      ballots: 0,
      abstentions: 0,
      selfVotes: 0,
    },
    perCandidate: [],
    selectedIndex: 1,
    reason: 'sole proposal, adopted without a vote',
  };
}

/**
 Builds the whole-chunk round as it looks when judges could not rank anything,
 which is the round that records ballots and names no winner.
 */
function declinedChunkRound(slate: readonly RepairSlateEntry[],): RepairJudgedRound {
  return {
    kind: 'declined',
    stage: 'chunk-patch',
    envelopeId: 'chunk',
    slate,
    ballots: [],
    tally: {
      judgesAvailable: 3,
      ballots: 3,
      abstentions: 3,
      selfVotes: 0,
    },
    perCandidate: [],
    reason: 'no candidate drew quorum',
    disposition: 'indecision',
  };
}

/**
 Builds the record of an envelope whose judges refused every proposal, which
 names no winner.

 @param envelopeId - envelope refused

 @param slate - proposals the judges refused

 @returns Declined round

 @example
 ```ts
 const round = declinedEnvelopeRound({ envelopeId: 'dropped', slate: [entry,], },);
 ```
 */
function declinedEnvelopeRound(
  {
    envelopeId,
    slate,
  }: {
    readonly envelopeId: string;
    readonly slate: readonly RepairSlateEntry[];
  },
): RepairJudgedRound {
  return {
    kind: 'declined',
    stage: 'envelope',
    envelopeId,
    slate,
    ballots: [],
    tally: {
      judgesAvailable: 3,
      ballots: 3,
      abstentions: 0,
      selfVotes: 0,
    },
    perCandidate: [],
    reason: 'every judge refused every proposal',
    disposition: 'rejection',
  };
}

/**
 Builds the editor stage result the authorship read takes.
 */
function editorOf(
  {
    rounds,
    applied,
    shippedProducer,
  }: {
    readonly rounds: readonly RepairJudgedRound[];
    readonly applied: readonly PatchOperation[];
    readonly shippedProducer: ShippedProducer;
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
    shippedProducer,
    // These fixtures read authorship only, so no gate is re-applied to them.
    preservation: { mode: 'skip', },
  };
}

/**
 The one envelope every case in this file repairs, and the issue it serves.
 */
const KEPT = [
  envelopeOf({
    envelopeId: 'kept',
    issueIds: [WHISKER,],
  },),
];

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: appliedIssuesByEnvelope.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES ONLY THE ENVELOPES WHOSE OPERATION SURVIVED THE GATE, so an envelope the gate '
            + 'refused puts no text into the candidate and the issues it named keep whole votes',
          fn: async function rejectedEnvelopesAreAbsent() {
            expect(appliedIssuesByEnvelope({
              envelopes: [
                envelopeOf({
                  envelopeId: 'kept',
                  issueIds: [WHISKER,],
                },),
                envelopeOf({
                  envelopeId: 'refused',
                  issueIds: [PAW,],
                },),
              ],
              applied: [operationOf('kept',),],
            },),).toEqual({ kept: [WHISKER,], },);
          },
        },),

        it({
          name: 'SKIPS an operation naming an envelope the slate no longer carries, rather than '
            + 'refusing: an unknown envelope contributes no authorship, which is the safe direction',
          fn: async function unknownEnvelopesAreSkipped() {
            expect(appliedIssuesByEnvelope({
              envelopes: [],
              applied: [operationOf('vanished',),],
            },),).toEqual({},);
          },
        },),
      ],
    },),

    describe({
      name: collectIssueAuthors.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE MODEL THAT WROTE THE WHOLE SHIPPED CHUNK for every issue, because one '
            + 'model wrote all of it and every creditable issue sits in text it produced',
          fn: async function aLoneWinnerAnswersForEveryIssue() {
            expect(collectIssueAuthors({
              editor: editorOf({
                applied: [operationOf('kept',),],
                rounds: [],
                shippedProducer: {
                  kind: 'model',
                  modelId: AUTHOR,
                },
              },),
              envelopes: KEPT,
            },),).toEqual({
              perIssue: {},
              everyIssue: [AUTHOR,],
            },);
          },
        },),

        it({
          name: 'NAMES THE EDITOR WHOSE REPAIR SHIPS AFTER THE JUDGES DECLINED TO RANK ANYTHING. This '
            + 'is the case a reader of the rounds cannot answer: the round records ballots and no '
            + 'winner, yet a real editor wrote the text, and leaving it unnamed lets that editor '
            + 'certify its own work at full weight',
          fn: async function theIndecisionFallbackHasAnAuthor() {
            expect(collectIssueAuthors({
              editor: editorOf({
                applied: [operationOf('kept',),],
                rounds: [
                  declinedChunkRound([
                    slateEntryOf({
                      index: 1,
                      modelId: AUTHOR,
                    },),
                    slateEntryOf({
                      index: 2,
                      modelId: HELPER,
                    },),
                  ],),
                ],
                shippedProducer: {
                  kind: 'model',
                  modelId: AUTHOR,
                },
              },),
              envelopes: KEPT,
            },),).toEqual({
              perIssue: {},
              everyIssue: [AUTHOR,],
            },);
          },
        },),

        it({
          name: 'IGNORES AN ENVELOPE WINNER WHOSE TEXT LOST TO A RIVAL WHOLE-CHUNK PROPOSAL, because '
            + 'the composite it was assembled into never shipped, so it is judging someone else\'s '
            + 'text and keeps a whole vote',
          fn: async function envelopeWinnersThatLostAreNotAuthors() {
            expect(collectIssueAuthors({
              editor: editorOf({
                applied: [operationOf('kept',),],
                rounds: [
                  selectedRound({
                    envelopeId: 'kept',
                    slate: [
                      slateEntryOf({
                        index: 1,
                        modelId: HELPER,
                      },),
                    ],
                    selectedIndex: 1,
                  },),
                ],
                shippedProducer: {
                  kind: 'model',
                  modelId: AUTHOR,
                },
              },),
              envelopes: KEPT,
            },),).toEqual({
              perIssue: {},
              everyIssue: [AUTHOR,],
            },);
          },
        },),

        it({
          name: 'NAMES THE AUTHOR OF AN ENVELOPE ADOPTED WITHOUT A VOTE, so a checker who wrote that '
            + 'text is discounted on it like any other winner',
          fn: async function adoptedEnvelopesKeepTheirAuthors() {
            expect(collectIssueAuthors({
              editor: editorOf({
                applied: [operationOf('kept',),],
                rounds: [
                  adoptedRound({
                    envelopeId: 'kept',
                    slate: [
                      slateEntryOf({
                        index: 1,
                        modelId: HELPER,
                      },),
                    ],
                  },),
                ],
                shippedProducer: {
                  kind: 'composite',
                  contributors: [HELPER,],
                },
              },),
              envelopes: KEPT,
            },),).toEqual({
              perIssue: { [WHISKER]: [HELPER,], },
              everyIssue: [],
            },);
          },
        },),
        it({
          name: 'READS PAST AN ENVELOPE THE JUDGES REFUSED when the composite ships, since it names no winner '
            + 'and wrote none of the composite, and names the authors of the envelope that was kept',
          fn: async function refusedEnvelopesAreNotRead() {
            expect(collectIssueAuthors({
              editor: editorOf({
                applied: [operationOf('kept',),],
                rounds: [
                  adoptedRound({
                    envelopeId: 'kept',
                    slate: [
                      slateEntryOf({
                        index: 1,
                        modelId: AUTHOR,
                      },),
                    ],
                  },),
                  declinedEnvelopeRound({
                    envelopeId: 'dropped',
                    slate: [
                      slateEntryOf({
                        index: 1,
                        modelId: HELPER,
                      },),
                    ],
                  },),
                ],
                shippedProducer: {
                  kind: 'composite',
                  contributors: [AUTHOR,],
                },
              },),
              envelopes: KEPT,
            },),).toEqual({
              perIssue: { [WHISKER]: [AUTHOR,], },
              everyIssue: [],
            },);
          },
        },),
        it({
          name: 'NAMES NOBODY WHEN THE UNTOUCHED TRANSLATION SHIPS, since no model wrote it and no '
            + 'checker can be certifying its own work',
          fn: async function nothingShippedMeansNoAuthors() {
            expect(collectIssueAuthors({
              editor: editorOf({
                applied: [],
                rounds: [],
                shippedProducer: NOBODY_WROTE_IT,
              },),
              envelopes: KEPT,
            },),).toEqual({
              perIssue: {},
              everyIssue: [],
            },);
          },
        },),
      ],
    },),
  ],
},);
