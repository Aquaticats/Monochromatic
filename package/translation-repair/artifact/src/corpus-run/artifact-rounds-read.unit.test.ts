/**
 Tests for reading judged repair rounds back out of a settled artifact.

 THE REFUSAL IS THE FEATURE, and half these cases pin it. Version 2 hands the
 lane result back unread, so everything this reader takes it has to check, and
 a model the roster no longer seats has to be NAMED rather than read as
 current. A standing that silently mixed two rosters would be worse than one
 that reported nothing.

 Content is cat-themed invention. Model ids are not: they come from the
 catalog, because the whole point of half these cases is which ids the roster
 holds.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  OffRosterModelError,
  readRepairRounds,
  RoundsNotRecordedError,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Model the roster seats today, used wherever a case is not about the roster.
 */
const SEATED = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 A second seated model, so a slate can carry two producers.
 */
const ALSO_SEATED = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

/**
 Model the roster held until 2026-08-24 and does not now.
 */
const DEPARTED = 'hf:zai-org/GLM-4.7-Flash';

/**
 Where every case reads from, so a message can be checked against a path.
 */
const PATH = 'Whiskerfold.lanes.repair.result';

/**
 Builds one ballot, defaulting everything a case does not care about.

 @param modelId - judge that cast it

 @param best - candidate it named

 @param weight - what the ballot counted for

 @param selfVote - whether the judge named its own writing

 @param reason - what the judge said, which every case but one leaves alone

 @returns Ballot as an artifact records one

 @example
 ```ts
 const ballot = ballotOf({ modelId: SEATED, best: 1, },);
 ```
 */
function ballotOf(
  {
    modelId,
    best,
    weight = 1,
    selfVote = false,
    reason = 'the tabby paragraph reads more like a cat and less like a filing',
  }: {
    readonly modelId: string;
    readonly best: number;
    readonly weight?: number;
    readonly selfVote?: boolean;
    readonly reason?: string;
  },
): Record<string, unknown> {
  return {
    modelId,
    best,
    reason,
    weight,
    selfVote,
  };
}

/**
 Builds one slate position around a producer.

 @param index - one-based position judges saw

 @param producer - who wrote it

 @returns Slate entry as an artifact records one

 @example
 ```ts
 const entry = slateOf({ index: 1, producer: { kind: 'model', modelId: SEATED, }, },);
 ```
 */
function slateOf(
  {
    index,
    producer,
  }: {
    readonly index: number;
    readonly producer: Record<string, unknown>;
  },
): Record<string, unknown> {
  return {
    index,
    rendered: 'The cat sat, and went on sitting, in the window she had chosen.',
    hash: `sha256-whisker-${String(index,)}`,
    producer,
  };
}

/**
 Builds what one slate position drew.

 @param index - one-based position

 @returns Weight entry as an artifact records one

 @example
 ```ts
 const drawn = drawnOf({ index: 1, },);
 ```
 */
function drawnOf(
  {
    index,
    weight = 1,
  }: {
    readonly index: number;
    readonly weight?: number;
  },
): Record<string, unknown> {
  return {
    index,
    ballots: 1,
    fullVotes: 1,
    selfVotes: 0,
    weight,
  };
}

/**
 Builds one round around a slate and its ballots.

 CARRIES EVERY FIELD THE LANE WRITES, including the ones a standing never
 reads. The reader validates the whole round, so a fixture carrying only what
 a standing needs would pin a reader looser than the one that ships.

 @param slate - candidates judges were shown

 @param ballots - ballots cast over them

 @param stage - stage that ran it

 @returns Round as an artifact records one

 @example
 ```ts
 const round = roundOf({ slate, ballots, },);
 ```
 */
function roundOf(
  {
    slate,
    ballots,
    stage = 'envelope',
    drawnWeight = 1,
  }: {
    readonly slate: readonly Record<string, unknown>[];
    readonly ballots: readonly Record<string, unknown>[];
    readonly stage?: string;
    readonly drawnWeight?: number;
  },
): Record<string, unknown> {
  return {
    kind: 'selected',
    stage,
    envelopeId: 'chunk',
    slate,
    ballots,
    tally: {
      judgesAvailable: 2,
      ballots: ballots.length,
      abstentions: 0,
      selfVotes: 0,
    },
    perCandidate: slate.map(function drawn(entry, index,): Record<string, unknown> {
      return drawnOf({
        index: Number(entry.index ?? (index + 1),),
        weight: drawnWeight,
      },);
    },),
    selectedIndex: 1,
    voteWeight: 1,
  };
}

/**
 Builds an adopted round as the lane writes it: a slate of one, the index of
 that one, a reason, and nothing a vote would have produced.

 @param slate - one entry

 @returns Round as its JSON would parse

 @example
 ```ts
 const round = adoptedRoundOf({ slate: [slateOf({ index: 1, },),], },);
 ```
 */
function adoptedRoundOf(
  { slate, }: { readonly slate: readonly Record<string, unknown>[]; },
): Record<string, unknown> {
  return {
    kind: 'adopted',
    stage: 'envelope',
    envelopeId: 'kept',
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
 Builds a round that decided nothing, which records two fields the other
 outcome does not.

 @param slate - candidates judges were shown

 @param ballots - ballots cast over them

 @returns Declining round as an artifact records one

 @example
 ```ts
 const round = declinedRoundOf({ slate, ballots, },);
 ```
 */
function declinedRoundOf(
  {
    slate,
    ballots,
  }: {
    readonly slate: readonly Record<string, unknown>[];
    readonly ballots: readonly Record<string, unknown>[];
  },
): Record<string, unknown> {
  /**
   Selecting round, whose six shared fields this reuses so the two fixtures
   cannot drift apart on them.
   */
  const selecting = roundOf({
    slate,
    ballots,
  },);

  // NAMED ONE BY ONE rather than spread and blanked, because a declining round
  // records neither `selectedIndex` nor `voteWeight` and a fixture carrying
  // them as blanks would let a reader that ignored `kind` pass.
  return {
    kind: 'declined',
    stage: selecting.stage,
    envelopeId: selecting.envelopeId,
    slate: selecting.slate,
    ballots: selecting.ballots,
    tally: selecting.tally,
    perCandidate: selecting.perCandidate,
    reason: 'judges split evenly across two candidates',
    disposition: 'indecision',
  };
}

/**
 Wraps rounds into the raw lane result shape the reader takes.

 @param chunks - rounds per chunk, in chunk order

 @returns Raw result as an artifact holds one

 @example
 ```ts
 const raw = rawOf({ chunks: [[round,],], },);
 ```
 */
function rawOf(
  { chunks, }: { readonly chunks: readonly (readonly Record<string, unknown>[])[]; },
): Record<string, unknown> {
  return {
    chunks: chunks.map(function toChunk(rounds, index,): Record<string, unknown> {
      return {
        sliceIndex: index,
        rounds,
      };
    },),
  };
}

/**
 A round every case can start from, produced by two seated models.
 */
const PLAIN_ROUND = roundOf({
  slate: [
    slateOf({
      index: 1,
      producer: {
        kind: 'model',
        modelId: SEATED,
      },
    },),
    slateOf({
      index: 2,
      producer: {
        kind: 'model',
        modelId: ALSO_SEATED,
      },
    },),
  ],
  ballots: [
    ballotOf({
      modelId: SEATED,
      best: 2,
    },),
    ballotOf({
      modelId: ALSO_SEATED,
      best: 2,
      weight: 0.5,
      selfVote: true,
    },),
  ],
},);

await describe({
  name: readRepairRounds.name,
  children: [
    it({
      name: 'reads a round the lane actually records',
      fn: async () => {
        /**
         Rounds one chunk produced.
         */
        const perChunk = readRepairRounds({
          raw: rawOf({ chunks: [[PLAIN_ROUND,],], },),
          path: PATH,
        },);

        expect(perChunk.length,).toBe(1,);
        expect(perChunk[0]?.length,).toBe(1,);
        expect(perChunk[0]?.[0]?.stage,).toBe('envelope',);
        expect(perChunk[0]?.[0]?.slate.length,).toBe(2,);
      },
    },),

    it({
      name: 'CARRIES WHAT EACH JUDGE SAID, in its own words and in ballot order, since a standing that '
        + 'reported a judge name where its reasoning belongs would read as a roster the pipeline never '
        + 'seated and would say nothing at all about why a candidate won',
      fn: async () => {
        /**
         Two ballots whose stated reasons differ from each other and from
         every other field on them, so no other value can stand in.
         */
        const spoken = readRepairRounds({
          raw: rawOf({
            chunks: [[
              roundOf({
                slate: [
                  slateOf({
                    index: 1,
                    producer: {
                      kind: 'model',
                      modelId: SEATED,
                    },
                  },),
                  slateOf({
                    index: 2,
                    producer: {
                      kind: 'model',
                      modelId: ALSO_SEATED,
                    },
                  },),
                ],
                ballots: [
                  ballotOf({
                    modelId: SEATED,
                    best: 2,
                    reason: 'the second keeps the cat in the window where the original leaves her',
                  },),
                  ballotOf({
                    modelId: ALSO_SEATED,
                    best: 1,
                    reason: 'the first says whiskers once rather than three times',
                  },),
                ],
              },),
            ],],
          },),
          path: PATH,
        },);

        /**
         Ballots of the only round the only chunk recorded.
         */
        const ballots = spoken[0]?.[0]?.ballots ?? [];

        expect(ballots.length,).toBe(2,);
        expect(ballots[0]?.reason,).toBe(
          'the second keeps the cat in the window where the original leaves her',
        );
        expect(ballots[1]?.reason,).toBe('the first says whiskers once rather than three times',);
        expect(ballots[0]?.modelId,).toBe(SEATED,);
        expect(ballots[1]?.modelId,).toBe(ALSO_SEATED,);
      },
    },),

    it({
      name: 'reads a DECLINING round, keeping what a selecting one has no room for',
      fn: async () => {
        /**
         A round where judges settled on nothing, which records a reason and a
         disposition in place of a winner and its weight.
         */
        const perChunk = readRepairRounds({
          raw: rawOf({
            chunks: [[
              declinedRoundOf({
                slate: [
                  slateOf({
                    index: 1,
                    producer: {
                      kind: 'model',
                      modelId: SEATED,
                    },
                  },),
                ],
                ballots: [
                  ballotOf({
                    modelId: ALSO_SEATED,
                    best: 1,
                  },),
                ],
              },),
            ],],
          },),
          path: PATH,
        },);

        /**
         The round itself, read out so both of its own fields can be asserted.
         */
        const round = perChunk[0]?.[0];

        expect(round?.kind,).toBe('declined',);
        expect(round?.perCandidate.length,).toBe(1,);
        expect((round?.kind === 'declined') ? round.disposition : 'not-declined',)
          .toBe('indecision',);
      },
    },),

    it({
      name: 'READS an adopted round, which carries its one slate entry and index and nothing a vote '
        + 'would have produced',
      fn: async () => {
        /**
         Rounds read back, one chunk holding one adopted round.
         */
        const perChunk = readRepairRounds({
          raw: rawOf({
            chunks: [[
              adoptedRoundOf({
                slate: [
                  slateOf({
                    index: 1,
                    producer: {
                      kind: 'model',
                      modelId: SEATED,
                    },
                  },),
                ],
              },),
            ],],
          },),
          path: PATH,
        },);
        /**
         The one round.
         */
        const round = perChunk[0]?.[0];
        expect(round?.kind,).toBe('adopted',);
        expect(round?.slate.length,).toBe(1,);
        expect((round?.kind === 'adopted') ? round.selectedIndex : 0,).toBe(1,);
      },
    },),
    it({
      name: 'REFUSES a round recording nothing about what each position drew',
      fn: async () => {
        /**
         What the reader threw when `perCandidate` was absent, which is the
         shape every round on disk carries and a partial reader would ignore.
         */
        const refusal = caught(function readsAPartialRound() {
          /**
           A well formed round with that one field taken back out.
           */
          const {
            perCandidate,
            ...withoutDraws
          } = roundOf({
            slate: [
              slateOf({
                index: 1,
                producer: {
                  kind: 'model',
                  modelId: SEATED,
                },
              },),
            ],
            ballots: [
              ballotOf({
                modelId: ALSO_SEATED,
                best: 1,
              },),
            ],
          },);

          expect(perCandidate,).toBeDefined();

          readRepairRounds({
            raw: rawOf({ chunks: [[withoutDraws,],], },),
            path: PATH,
          },);
        },);

        expect(refusal,).toBeInstanceOf(ArtifactParseError,);
        expect((refusal as Error).message,)
          .toContain(`${PATH}.chunks[0].rounds[0].perCandidate`,);
      },
    },),

    it({
      name: 'REFUSES a declining round naming a disposition the lane cannot produce',
      fn: async () => {
        /**
         What the reader threw on a disposition outside the two the selection
         can reach, which is how a hand-edited or foreign record reads.
         */
        const refusal = caught(function readsAnUnknownDisposition() {
          readRepairRounds({
            raw: rawOf({
              chunks: [[
                {
                  ...declinedRoundOf({
                    slate: [
                      slateOf({
                        index: 1,
                        producer: {
                          kind: 'model',
                          modelId: SEATED,
                        },
                      },),
                    ],
                    ballots: [
                      ballotOf({
                        modelId: ALSO_SEATED,
                        best: 1,
                      },),
                    ],
                  },),
                  disposition: 'adjourned',
                },
              ],],
            },),
            path: PATH,
          },);
        },);

        expect(refusal,).toBeInstanceOf(ArtifactParseError,);
        expect((refusal as Error).message,)
          .toContain(`${PATH}.chunks[0].rounds[0].disposition`,);
      },
    },),

    it({
      name: 'ACCEPTS a half weight and a declining ballot, which no count guard would',
      fn: async () => {
        /**
         A round whose second ballot is a self-vote and whose first names no
         candidate at all, both of which a count guard refuses.
         */
        const declining = readRepairRounds({
          raw: rawOf({
            chunks: [[
              roundOf({
                slate: [
                  slateOf({
                    index: 1,
                    producer: {
                      kind: 'model',
                      modelId: SEATED,
                    },
                  },),
                ],
                ballots: [
                  ballotOf({
                    modelId: ALSO_SEATED,
                    best: -1,
                    weight: 0,
                  },),
                  ballotOf({
                    modelId: SEATED,
                    best: 1,
                    weight: 0.5,
                    selfVote: true,
                  },),
                ],
              },),
            ],],
          },),
          path: PATH,
        },);

        expect(declining[0]?.[0]?.ballots[0]?.best,).toBe(-1,);
        expect(declining[0]?.[0]?.ballots[1]?.weight,).toBe(0.5,);
      },
    },),

    it({
      name: 'ACCEPTS a FRACTIONAL SUMMED WEIGHT for a candidate, which no count guard would, '
        + 'and which the ballot-side case beside this one does not reach',
      fn: async () => {
        // The "ACCEPTS a half weight and a declining ballot" case pins the
        // weight ONE judge carries. This pins
        // what a candidate DREW from all of them, read by a different guard on
        // a different field. Every other fixture here draws a whole 1, so
        // swapping `requireFinite` for `requireCount` on `perCandidate.weight`
        // passed the whole file while refusing any real round where a judge
        // voted on its own writing.
        /**
         A round whose one candidate drew a full vote and a self vote.
         */
        const drawn = readRepairRounds({
          raw: rawOf({
            chunks: [[
              roundOf({
                slate: [
                  slateOf({
                    index: 1,
                    producer: {
                      kind: 'model',
                      modelId: SEATED,
                    },
                  },),
                ],
                ballots: [
                  ballotOf({
                    modelId: ALSO_SEATED,
                    best: 1,
                    weight: 1,
                  },),
                  ballotOf({
                    modelId: SEATED,
                    best: 1,
                    weight: 0.5,
                    selfVote: true,
                  },),
                ],
                drawnWeight: 1.5,
              },),
            ],],
          },),
          path: PATH,
        },);

        expect(drawn[0]?.[0]?.perCandidate[0]?.weight,).toBe(1.5,);
      },
    },),

    it({
      name: 'keeps a chunk that judged nothing as its own empty list',
      fn: async () => {
        /**
         Two chunks, only the second of which asked anyone to write.
         */
        const perChunk = readRepairRounds({
          raw: rawOf({
            chunks: [
              [],
              [PLAIN_ROUND,],
            ],
          },),
          path: PATH,
        },);

        expect(perChunk.length,).toBe(2,);
        expect(perChunk[0]?.length,).toBe(0,);
        expect(perChunk[1]?.length,).toBe(1,);
      },
    },),

    it({
      name: 'reads a composite producer and an incumbent, not only a lone model',
      fn: async () => {
        /**
         A slate carrying all three provenance shapes at once.
         */
        const perChunk = readRepairRounds({
          raw: rawOf({
            chunks: [[
              roundOf({
                slate: [
                  slateOf({
                    index: 1,
                    producer: {
                      kind: 'composite',
                      contributors: [
                        SEATED,
                        ALSO_SEATED,
                      ],
                    },
                  },),
                  slateOf({
                    index: 2,
                    producer: {
                      kind: 'incumbent',
                      matched: [],
                    },
                  },),
                ],
                ballots: [
                  ballotOf({
                    modelId: SEATED,
                    best: 1,
                  },),
                ],
              },),
            ],],
          },),
          path: PATH,
        },);

        expect(perChunk[0]?.[0]?.slate[0]?.producer.kind,).toBe('composite',);
        expect(perChunk[0]?.[0]?.slate[1]?.producer.kind,).toBe('incumbent',);
      },
    },),

    it({
      name: 'REFUSES a candidate written by a model the roster no longer seats',
      fn: async () => {
        expect(function read() {
          readRepairRounds({
            raw: rawOf({
              chunks: [[
                roundOf({
                  slate: [
                    slateOf({
                      index: 1,
                      producer: {
                        kind: 'model',
                        modelId: DEPARTED,
                      },
                    },),
                  ],
                  ballots: [
                    ballotOf({
                      modelId: SEATED,
                      best: 1,
                    },),
                  ],
                },),
              ],],
            },),
            path: PATH,
          },);
        },).toThrow(OffRosterModelError,);
      },
    },),

    it({
      name: 'REFUSES a departed model inside a composite, where a slate check alone would miss it',
      fn: async () => {
        expect(function read() {
          readRepairRounds({
            raw: rawOf({
              chunks: [[
                roundOf({
                  slate: [
                    slateOf({
                      index: 1,
                      producer: {
                        kind: 'composite',
                        contributors: [
                          SEATED,
                          DEPARTED,
                        ],
                      },
                    },),
                  ],
                  ballots: [
                    ballotOf({
                      modelId: SEATED,
                      best: 1,
                    },),
                  ],
                },),
              ],],
            },),
            path: PATH,
          },);
        },).toThrow(OffRosterModelError,);
      },
    },),

    it({
      name: 'REFUSES a departed model that only ever judged, having written nothing',
      fn: async () => {
        expect(function read() {
          readRepairRounds({
            raw: rawOf({
              chunks: [[
                roundOf({
                  slate: [
                    slateOf({
                      index: 1,
                      producer: {
                        kind: 'model',
                        modelId: SEATED,
                      },
                    },),
                  ],
                  ballots: [
                    ballotOf({
                      modelId: DEPARTED,
                      best: 1,
                    },),
                  ],
                },),
              ],],
            },),
            path: PATH,
          },);
        },).toThrow(OffRosterModelError,);
      },
    },),

    it({
      name: 'names a result with no chunks an EARLIER SHAPE, not a malformed one',
      fn: async () => {
        /**
         What a repair result settled before the lane recorded rounds throws.
         It is a complete, correct record that cannot answer this question,
         and reporting it as a parse failure would call a healthy archive
         broken: 22 of 41 artifacts on disk are exactly this.
         */
        const refusal = caught(function readsAnEarlierShape() {
          readRepairRounds({
            raw: { repairedText: 'The cat, having sat, declined to comment.', },
            path: PATH,
          },);
        },);

        expect(refusal,).toBeInstanceOf(RoundsNotRecordedError,);
        expect(refusal,).not.toBeInstanceOf(ArtifactParseError,);
        expect((refusal as Error).message,).toContain(`${PATH}.chunks`,);
      },
    },),

    it({
      name: 'REFUSES chunks that are present and not an array, which IS malformed',
      fn: async () => {
        /**
         A result naming the field with something that cannot hold rounds,
         which is the case the earlier-shape answer must not swallow.
         */
        const refusal = caught(function readsAMalformedResult() {
          readRepairRounds({
            raw: { chunks: 'three of them', },
            path: PATH,
          },);
        },);

        expect(refusal,).toBeInstanceOf(ArtifactParseError,);
        expect(refusal,).not.toBeInstanceOf(RoundsNotRecordedError,);
      },
    },),

    it({
      name: 'REFUSES a stage the lane cannot have run',
      fn: async () => {
        expect(function read() {
          readRepairRounds({
            raw: rawOf({
              chunks: [[
                roundOf({
                  slate: [
                    slateOf({
                      index: 1,
                      producer: {
                        kind: 'model',
                        modelId: SEATED,
                      },
                    },),
                  ],
                  ballots: [
                    ballotOf({
                      modelId: SEATED,
                      best: 1,
                    },),
                  ],
                  stage: 'napping',
                },),
              ],],
            },),
            path: PATH,
          },);
        },).toThrow(ArtifactParseError,);
      },
    },),

    it({
      name: 'REFUSES a ballot whose weight is not a number, since a share divides by it',
      fn: async () => {
        expect(function read() {
          readRepairRounds({
            raw: rawOf({
              chunks: [[
                roundOf({
                  slate: [
                    slateOf({
                      index: 1,
                      producer: {
                        kind: 'model',
                        modelId: SEATED,
                      },
                    },),
                  ],
                  ballots: [
                    {
                      ...ballotOf({
                        modelId: SEATED,
                        best: 1,
                      },),
                      weight: 'a lot',
                    },
                  ],
                },),
              ],],
            },),
            path: PATH,
          },);
        },).toThrow(ArtifactParseError,);
      },
    },),

    it({
      name:
        'REFUSES a ballot weight that OVERFLOWED to infinity, which the check beside this one reads as '
          + 'a number: `1e400` is valid JSON, so a file can carry it, and a share dividing by an infinite '
          + 'weight is zero at every candidate rather than a refusal anyone would see',
      fn: async () => {
        /**
         Weight exactly as a file carries it. `1e400` is syntactically valid
         JSON and parses to `Infinity`, which is the only way a non-finite
         number reaches a reader that sees nothing but `JSON.parse` output.
         */
        const overflowed = JSON.parse('{ "weight": 1e400 }',) as { readonly weight: number; };

        /**
         What the reader threw, held so both its class and its reason can be
         asserted separately.
         */
        const refusalOfInfiniteWeight = caught(function readsAnOverflowedWeight() {
          readRepairRounds({
            raw: rawOf({
              chunks: [[
                roundOf({
                  slate: [
                    slateOf({
                      index: 1,
                      producer: {
                        kind: 'model',
                        modelId: SEATED,
                      },
                    },),
                  ],
                  ballots: [
                    {
                      ...ballotOf({
                        modelId: SEATED,
                        best: 1,
                      },),
                      weight: overflowed.weight,
                    },
                  ],
                },),
              ],],
            },),
            path: PATH,
          },);
        },);

        expect(refusalOfInfiniteWeight,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfInfiniteWeight as Error).message,).toContain('a finite number',);
      },
    },),

    it({
      name: 'names the exact path a departed model was read at',
      fn: async () => {
        /**
         What the reader threw, held so both the class and the path it names
         can be asserted separately.
         */
        const refusal = caught(function readsAnEarlierRoster() {
          readRepairRounds({
            raw: rawOf({
              chunks: [
                [],
                [
                  roundOf({
                    slate: [
                      slateOf({
                        index: 1,
                        producer: {
                          kind: 'model',
                          modelId: DEPARTED,
                        },
                      },),
                    ],
                    ballots: [
                      ballotOf({
                        modelId: SEATED,
                        best: 1,
                      },),
                    ],
                  },),
                ],
              ],
            },),
            path: PATH,
          },);
        },);

        expect(refusal,).toBeInstanceOf(OffRosterModelError,);
        expect((refusal as Error).message,)
          .toContain(`${PATH}.chunks[1].rounds[0].slate[0].producer.modelId`,);
        expect((refusal as Error).message,).toContain(DEPARTED,);
      },
    },),

    it({
      name: 'names the FIELD and not just the result when `chunks` is not a list, since an operator '
        + 'reading a refusal that stops at the result has been told which artifact is unreadable '
        + 'and not which of its fields to look at',
      fn: async () => {
        /**
         What the reader threw when handed a result whose `chunks` is a string.

         PRESENT BUT WRONG-SHAPED, which is a different answer from absent: a
         result carrying no `chunks` at all predates the field and is reported
         as {@link RoundsNotRecordedError} rather than as a parse failure.
         */
        const refusal = caught(function readsChunksThatAreNotAList() {
          readRepairRounds({
            raw: { chunks: 'recorded as prose rather than as a list', },
            path: PATH,
          },);
        },);

        expect(refusal,).toBeInstanceOf(ArtifactParseError,);
        expect(refusal,).not.toBeInstanceOf(RoundsNotRecordedError,);
        expect((refusal as Error).message,).toContain(`${PATH}.chunks`,);
      },
    },),
  ],
},);
