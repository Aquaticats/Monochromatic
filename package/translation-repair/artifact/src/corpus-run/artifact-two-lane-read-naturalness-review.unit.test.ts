/**
 Tests absolute naturalness review recomputation, correction binding, and
 acceptance confirmation, through the one reader the package exports: every
 refusal of the review, round, seat, digest and confirmation readers is
 checked by the path and reason it names, so a case refused by another
 check fails. Model ids and texts are cat-themed invention.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  type ArtifactNaturalnessReviewSeat,
  parseNaturalnessReview,
} from '../../dist/final/node/index.mjs';

/**
 Artifact path every case reads the review at.
 */
const REVIEW_PATH = 'consolidation.slices[0].polish.review';

/**
 Exact final text review binds to.
 */
const FINAL_TEXT = 'The cat slept peacefully on the windowsill.';

/**
 Initial wording rejected before bounded corrections.
 */
const INITIAL_TEXT = 'The cat conducted peaceful sleeping on the windowsill.';

/**
 First correction whose exact review exposes another defect.
 */
const FIRST_CORRECTION_TEXT = 'The cat was sleeping peacefully upon the windowsill.';

/**
 One acceptable reviewer seat.

 @param modelId - invented distinct reviewer id

 @returns Schema-eight acceptable seat
 */
function acceptableSeat(
  { modelId, }: { readonly modelId: string; },
): ArtifactNaturalnessReviewSeat {
  return {
    modelId,
    status: 'acceptable',
    findings: [],
    reason: 'whole passage is publication-ready',
  };
}

/**
 One unavailable requested reviewer seat.

 @param modelId - invented distinct reviewer id

 @returns Accounted seat without usable verdict
 */
function unusableSeat(
  { modelId, }: { readonly modelId: string; },
): ArtifactNaturalnessReviewSeat {
  return {
    modelId,
    status: 'unusable',
    findings: [],
    reason: '',
  };
}

/**
 Valid no-correction review fixture.
 */
const REVIEW = {
  correctionCount: 0,
  rounds: [{
    candidateDigest: hashContent({ content: FINAL_TEXT, },),
    paragraphCount: 1,
    seats: [
      acceptableSeat({ modelId: 'hf:cat/Cat-A', },),
      acceptableSeat({ modelId: 'hf:cat/Cat-B', },),
    ],
    usable: 2,
    verdict: 'acceptable',
    findings: [],
  },],
} as const;

/**
 Builds one rejecting seat for located fixture finding.

 @param modelId - invented distinct reviewer id

 @param problem - actionable defect

 @returns Unacceptable seat with one paragraph finding
 */
function unacceptableSeat(
  {
    modelId,
    problem,
  }: {
    readonly modelId: string;
    readonly problem: string;
  },
): ArtifactNaturalnessReviewSeat {
  return {
    modelId,
    status: 'unacceptable',
    findings: [{ paragraph: 1, problem, },],
    reason: 'material defect remains',
  };
}

/**
 First exact review finding.
 */
const INITIAL_FINDINGS = [{ paragraph: 1, problem: 'Replace nominalized verb phrase.', },] as const;

/**
 Finding exposed by first correction.
 */
const SECOND_FINDINGS = [{ paragraph: 1, problem: 'Use ordinary location preposition.', },] as const;

/**
 Second correction whose review exposes one final defect.
 */
const SECOND_CORRECTION_TEXT = 'The cat slept peacefully upon the windowsill.';

/**
 Finding exposed by second correction.
 */
const THIRD_FINDINGS = [{ paragraph: 1, problem: 'Replace marked location preposition.', },] as const;

/**
 Valid schema-nine two-correction digest chain.
 */
const CHAINED_REVIEW = {
  correctionCount: 2,
  corrections: [
    {
      inputDigest: hashContent({ content: INITIAL_TEXT, },),
      findingsDigest: hashContent({ content: JSON.stringify(INITIAL_FINDINGS,), },),
      gatedTextDigest: hashContent({ content: FIRST_CORRECTION_TEXT, },),
    },
    {
      inputDigest: hashContent({ content: FIRST_CORRECTION_TEXT, },),
      findingsDigest: hashContent({ content: JSON.stringify(SECOND_FINDINGS,), },),
      gatedTextDigest: hashContent({ content: FINAL_TEXT, },),
    },
  ],
  rounds: [
    {
      candidateDigest: hashContent({ content: INITIAL_TEXT, },),
      candidateText: INITIAL_TEXT,
      paragraphCount: 1,
      paragraphDigests: [hashContent({ content: INITIAL_TEXT, },),],
      seats: [
        unacceptableSeat({ modelId: 'hf:cat/Cat-A', problem: INITIAL_FINDINGS[0].problem, },),
        acceptableSeat({ modelId: 'hf:cat/Cat-B', },),
      ],
      usable: 2,
      verdict: 'unacceptable',
      findings: INITIAL_FINDINGS,
    },
    {
      candidateDigest: hashContent({ content: FIRST_CORRECTION_TEXT, },),
      candidateText: FIRST_CORRECTION_TEXT,
      paragraphCount: 1,
      paragraphDigests: [hashContent({ content: FIRST_CORRECTION_TEXT, },),],
      seats: [
        unacceptableSeat({ modelId: 'hf:cat/Cat-A', problem: SECOND_FINDINGS[0].problem, },),
        acceptableSeat({ modelId: 'hf:cat/Cat-B', },),
      ],
      usable: 2,
      verdict: 'unacceptable',
      findings: SECOND_FINDINGS,
    },
    {
      candidateDigest: hashContent({ content: FINAL_TEXT, },),
      candidateText: FINAL_TEXT,
      paragraphCount: 1,
      paragraphDigests: [hashContent({ content: FINAL_TEXT, },),],
      seats: [
        acceptableSeat({ modelId: 'hf:cat/Cat-A', },),
        acceptableSeat({ modelId: 'hf:cat/Cat-B', },),
      ],
      usable: 2,
      verdict: 'acceptable',
      findings: [],
    },
  ],
} as const;

/**
 Valid schema-nine three-correction digest chain.
 */
const THREE_CORRECTION_REVIEW = {
  correctionCount: 3,
  corrections: [
    CHAINED_REVIEW.corrections[0],
    {
      inputDigest: hashContent({ content: FIRST_CORRECTION_TEXT, },),
      findingsDigest: hashContent({ content: JSON.stringify(SECOND_FINDINGS,), },),
      gatedTextDigest: hashContent({ content: SECOND_CORRECTION_TEXT, },),
    },
    {
      inputDigest: hashContent({ content: SECOND_CORRECTION_TEXT, },),
      findingsDigest: hashContent({ content: JSON.stringify(THIRD_FINDINGS,), },),
      gatedTextDigest: hashContent({ content: FINAL_TEXT, },),
    },
  ],
  rounds: [
    CHAINED_REVIEW.rounds[0],
    CHAINED_REVIEW.rounds[1],
    {
      candidateDigest: hashContent({ content: SECOND_CORRECTION_TEXT, },),
      candidateText: SECOND_CORRECTION_TEXT,
      paragraphCount: 1,
      paragraphDigests: [hashContent({ content: SECOND_CORRECTION_TEXT, },),],
      seats: [
        unacceptableSeat({ modelId: 'hf:cat/Cat-A', problem: THIRD_FINDINGS[0].problem, },),
        acceptableSeat({ modelId: 'hf:cat/Cat-B', },),
      ],
      usable: 2,
      verdict: 'unacceptable',
      findings: THIRD_FINDINGS,
    },
    CHAINED_REVIEW.rounds[2],
  ],
} as const;

/**
 Builds acceptable schema-nine reading of exact candidate.

 @param text - exact candidate independently reviewed

 @returns Candidate and paragraph-bound acceptable round
 */
function acceptableRound({ text, }: { readonly text: string; },) {
  return {
    candidateDigest: hashContent({ content: text, },),
    candidateText: text,
    paragraphCount: 1,
    paragraphDigests: [hashContent({ content: text, },),],
    seats: [
      acceptableSeat({ modelId: 'hf:cat/Cat-A', },),
      acceptableSeat({ modelId: 'hf:cat/Cat-B', },),
    ],
    usable: 2,
    verdict: 'acceptable' as const,
    findings: [],
  };
}

/**
 Schema-nine chain retaining earlier acceptances before decisive reviews.
 */
const CONFIRMED_CHAINED_REVIEW = {
  ...CHAINED_REVIEW,
  confirmations: [
    acceptableRound({ text: INITIAL_TEXT, },),
    acceptableRound({ text: FINAL_TEXT, },),
  ],
};

/**
 Schema-nine chain whose second correction returns to the text the first
 round reviewed, so one candidate stands in two decisive rounds, with the
 acceptance confirmation of that candidate.
 */
const REVISITING_REVIEW = {
  ...CHAINED_REVIEW,
  corrections: [
    CHAINED_REVIEW.corrections[0],
    {
      inputDigest: hashContent({ content: FIRST_CORRECTION_TEXT, },),
      findingsDigest: hashContent({ content: JSON.stringify(SECOND_FINDINGS,), },),
      gatedTextDigest: hashContent({ content: INITIAL_TEXT, },),
    },
  ],
  rounds: [
    CHAINED_REVIEW.rounds[0],
    CHAINED_REVIEW.rounds[1],
    acceptableRound({ text: INITIAL_TEXT, },),
  ],
  confirmations: [acceptableRound({ text: INITIAL_TEXT, },),],
};

/**
 Reads a review expecting the refusal of one check, named by the path and
 reason its message carries.

 @param value - review as an artifact carries it

 @param says - `at <path>: expected <reason>` text the refusal carries

 @param correctionChainRequired - whether to read under generation nine

 @param finalText - text the artifact says ships
 */
function expectRefused(
  {
    value,
    says,
    correctionChainRequired = false,
    finalText = FINAL_TEXT,
  }: {
    readonly value: unknown;
    readonly says: string;
    readonly correctionChainRequired?: boolean;
    readonly finalText?: string;
  },
): void {
  /**
   The read, repeated for each check.
   */
  const read = () =>
    parseNaturalnessReview({
      value,
      path: REVIEW_PATH,
      finalText,
      correctionChainRequired,
    },);
  expect(read,).toThrow(ArtifactParseError,);
  expect(read,).toThrow(`at ${REVIEW_PATH}${says}`,);
}

/**
 The no-correction review with its first round's first seat replaced.

 @param seat - seat as an artifact carries it

 @returns Review value
 */
function reviewWithSeat(seat: unknown,): unknown {
  return {
    ...REVIEW,
    rounds: [{
      ...REVIEW.rounds[0],
      seats: [seat, REVIEW.rounds[0].seats[1],],
    },],
  };
}

/**
 The two-correction chain with its first round's fields replaced.

 @param fields - fields written over the first round

 @returns Review value
 */
function chainWithFirstRound(fields: Readonly<Record<string, unknown>>,): unknown {
  return {
    ...CHAINED_REVIEW,
    rounds: [
      {
        ...CHAINED_REVIEW.rounds[0],
        ...fields,
      },
      CHAINED_REVIEW.rounds[1],
      CHAINED_REVIEW.rounds[2],
    ],
  };
}

await describe({
  name: parseNaturalnessReview.name,
  children: [
    it({
      name: 'ACCEPTS REVIEW whose recomputed final verdict and digest approve final text',
      fn: async () => {
        const parsed = parseNaturalnessReview({
          value: REVIEW,
          path: REVIEW_PATH,
          finalText: FINAL_TEXT,
        },);
        expect(parsed,).toEqual(REVIEW,);
      },
    },),

    it({
      name: 'ACCEPTS TWO CORRECTIONS only when every rejected-text, finding, paragraph, and gated-text digest links',
      fn: async () => {
        const parsed = parseNaturalnessReview({
          value: CHAINED_REVIEW,
          path: REVIEW_PATH,
          finalText: FINAL_TEXT,
          correctionChainRequired: true,
        },);
        expect(parsed,).toEqual(CHAINED_REVIEW,);
      },
    },),

    it({
      name: 'ACCEPTS THIRD CORRECTION when complete digest chain reaches final acceptance',
      fn: async () => {
        const parsed = parseNaturalnessReview({
          value: THREE_CORRECTION_REVIEW,
          path: REVIEW_PATH,
          finalText: FINAL_TEXT,
          correctionChainRequired: true,
        },);
        expect(parsed,).toEqual(THREE_CORRECTION_REVIEW,);
      },
    },),

    it({
      name: 'ACCEPTS CONFIRMATIONS bound to decisive candidates including acceptance later rejected',
      fn: async () => {
        const parsed = parseNaturalnessReview({
          value: CONFIRMED_CHAINED_REVIEW,
          path: REVIEW_PATH,
          finalText: FINAL_TEXT,
          correctionChainRequired: true,
        },);
        expect(parsed,).toEqual(CONFIRMED_CHAINED_REVIEW,);
      },
    },),

    it({
      name: 'REFUSES MISSING FINAL, REJECTED, DUPLICATE, UNBOUND, REORDERED, OR DIFFERENT-ROSTER CONFIRMATION, '
        + 'each by its own reason',
      fn: async () => {
        for (const [value, says,] of [
          [
            {
              ...CONFIRMED_CHAINED_REVIEW,
              confirmations: [CONFIRMED_CHAINED_REVIEW.confirmations[0],],
            },
            '.confirmations: expected earlier acceptable reading of exact final candidate',
          ],
          [
            {
              ...CONFIRMED_CHAINED_REVIEW,
              confirmations: [{
                ...CONFIRMED_CHAINED_REVIEW.confirmations[1],
                verdict: 'unacceptable',
                seats: [
                  unacceptableSeat({
                    modelId: 'hf:cat/Cat-A',
                    problem: 'Still awkward.',
                  },),
                  acceptableSeat({ modelId: 'hf:cat/Cat-B', },),
                ],
                findings: [{ paragraph: 1, problem: 'Still awkward.', },],
              },],
            },
            '.confirmations: expected acceptable earlier readings before decisive same-candidate review',
          ],
          [
            {
              ...CONFIRMED_CHAINED_REVIEW,
              confirmations: [
                CONFIRMED_CHAINED_REVIEW.confirmations[1],
                CONFIRMED_CHAINED_REVIEW.confirmations[1],
              ],
            },
            '.confirmations: expected at most one acceptance confirmation per reviewed candidate',
          ],
          [
            {
              ...CONFIRMED_CHAINED_REVIEW,
              confirmations: [acceptableRound({ text: 'The cat sat elsewhere.', },),],
            },
            '.confirmations: expected exact candidate and paragraph identities of one decisive review round',
          ],
          [
            {
              ...CONFIRMED_CHAINED_REVIEW,
              confirmations: CONFIRMED_CHAINED_REVIEW.confirmations.toReversed(),
            },
            '.confirmations: expected same candidate order as decisive review rounds',
          ],
          [
            {
              ...CONFIRMED_CHAINED_REVIEW,
              confirmations: [
                CONFIRMED_CHAINED_REVIEW.confirmations[0],
                {
                  ...CONFIRMED_CHAINED_REVIEW.confirmations[1],
                  seats: [
                    acceptableSeat({ modelId: 'hf:cat/Cat-A', },),
                    acceptableSeat({ modelId: 'hf:cat/Cat-C', },),
                  ],
                },
              ],
            },
            '.confirmations: expected same requested reviewer roster and quorum basis as decisive review',
          ],
        ] as const) {
          expectRefused({
            value,
            says,
            correctionChainRequired: true,
          },);
        }
      },
    },),

    it({
      name: 'REFUSES A CONFIRMATION THAT FITS TWO DECISIVE ROUNDS, since a chain that returns to a reviewed '
        + 'candidate leaves the confirmation naming neither round',
      fn: async () => {
        /**
         The read of the revisiting chain.
         */
        function readRevisiting(): unknown {
          return parseNaturalnessReview({
            value: REVISITING_REVIEW,
            path: REVIEW_PATH,
            finalText: INITIAL_TEXT,
            correctionChainRequired: true,
          },);
        }
        /** What the read threw. */
        const refusal = caught(readRevisiting,);
        expect(refusal,).toBeInstanceOf(ArtifactParseError,);
        expect(String(refusal,),).toBe(
          `ArtifactParseError: artifact parse failed at ${REVIEW_PATH}.confirmations[0]: expected `
            + 'a confirmation that fits one decisive review round, not several.',
        );
      },
    },),

    it({
      name: 'REFUSES A MUTATED CORRECTION, CANDIDATE OR PARAGRAPH DIGEST in a generation-nine chain, each by its own '
        + 'reason: extended candidate text, too few paragraph digests, a paragraph count the text does not have, a '
        + 'candidate digest of other text, a changed findings digest, and a changed final paragraph digest',
      fn: async () => {
        /**
         Paragraph digest of the first round's one paragraph.
         */
        const initialDigest = hashContent({ content: INITIAL_TEXT, },);
        for (const [value, says,] of [
          [
            chainWithFirstRound({ candidateText: `${INITIAL_TEXT} Extra sentence.`, },),
            '.rounds[0].paragraphDigests: expected SHA-256 digest of every structurally correctable reviewed paragraph',
          ],
          [
            chainWithFirstRound({ paragraphDigests: [], },),
            '.rounds[0].paragraphDigests: expected 1 reviewed paragraph digest',
          ],
          [
            chainWithFirstRound({
              paragraphCount: 2,
              paragraphDigests: [initialDigest, initialDigest,],
            },),
            '.rounds[0].paragraphCount: expected structurally correctable paragraph count of reviewed candidate text',
          ],
          [
            chainWithFirstRound({ candidateDigest: hashContent({ content: 'The cat napped elsewhere.', },), },),
            '.rounds[0].candidateDigest: expected SHA-256 of exact reviewed candidate text',
          ],
          [
            {
              ...CHAINED_REVIEW,
              corrections: [
                { ...CHAINED_REVIEW.corrections[0], findingsDigest: '0'.repeat(64,), },
                CHAINED_REVIEW.corrections[1],
              ],
            },
            '.corrections[0]: expected digest chain from rejected review through canonical findings to next gated text',
          ],
          [
            {
              ...CHAINED_REVIEW,
              rounds: [
                CHAINED_REVIEW.rounds[0],
                CHAINED_REVIEW.rounds[1],
                {
                  ...CHAINED_REVIEW.rounds[2],
                  paragraphDigests: ['0'.repeat(64,),],
                },
              ],
            },
            '.rounds[2].paragraphDigests: expected SHA-256 digest of every structurally correctable reviewed paragraph',
          ],
        ] as const) {
          expectRefused({
            value,
            says,
            correctionChainRequired: true,
          },);
        }
      },
    },),

    it({
      name: 'REFUSES A GENERATION-NINE CHAIN whose final candidate is not the text that ships, whose transitions '
        + 'number other than its correction count, or whose first round did not reject the text it corrected',
      fn: async () => {
        expectRefused({
          value: CHAINED_REVIEW,
          says: '.rounds[2].candidateText: expected exact final polish text',
          correctionChainRequired: true,
          finalText: 'The cat slept elsewhere.',
        },);
        expectRefused({
          value: {
            ...CHAINED_REVIEW,
            corrections: [CHAINED_REVIEW.corrections[0],],
          },
          says: '.corrections: expected 2 digest-bound correction transitions',
          correctionChainRequired: true,
        },);
        expectRefused({
          value: chainWithFirstRound(acceptableRound({ text: INITIAL_TEXT, },),),
          says: '.rounds: expected unacceptable review before every correction',
          correctionChainRequired: true,
        },);
      },
    },),

    it({
      name: 'REFUSES MUTATED COUNTS, VERDICT, DIGEST, MODEL IDS, PARAGRAPHS, CORRECTIONS, OR BELOW-HALF APPROVAL, '
        + 'each by its own reason, and more than one correction outside the digest chain',
      fn: async () => {
        for (const [value, says,] of [
          [
            {
              ...REVIEW,
              rounds: [{ ...REVIEW.rounds[0], usable: 1, },],
            },
            '.rounds[0].usable: expected 2 derived from seat statuses',
          ],
          [
            {
              ...REVIEW,
              rounds: [{ ...REVIEW.rounds[0], verdict: 'unacceptable', },],
            },
            '.rounds[0].verdict: expected acceptable, derived from seat statuses and quorum',
          ],
          [
            {
              ...REVIEW,
              rounds: [{ ...REVIEW.rounds[0], candidateDigest: '0'.repeat(64,), },],
            },
            '.rounds[0].candidateDigest: expected SHA-256 of final polish text',
          ],
          [
            {
              ...REVIEW,
              rounds: [{
                ...REVIEW.rounds[0],
                seats: [
                  acceptableSeat({ modelId: 'hf:cat/Cat-A', },),
                  acceptableSeat({ modelId: 'hf:cat/Cat-A', },),
                ],
              },],
            },
            '.rounds[0].seats: expected one status per unique reviewer model id',
          ],
          [
            {
              ...REVIEW,
              rounds: [{ ...REVIEW.rounds[0], paragraphCount: 2, },],
            },
            '.rounds[0].paragraphCount: expected structurally correctable paragraph count of final polish text',
          ],
          [
            {
              ...REVIEW,
              correctionCount: 1,
            },
            '.rounds: expected 2 rounds for correction count',
          ],
          [
            {
              ...REVIEW,
              correctionCount: 2,
            },
            '.correctionCount: expected zero or one legacy correction',
          ],
          [
            {
              ...REVIEW,
              rounds: [{
                ...REVIEW.rounds[0],
                seats: [
                  acceptableSeat({ modelId: 'hf:cat/Cat-A', },),
                  acceptableSeat({ modelId: 'hf:cat/Cat-B', },),
                  unusableSeat({ modelId: 'hf:cat/Cat-C', },),
                  unusableSeat({ modelId: 'hf:cat/Cat-D', },),
                  unusableSeat({ modelId: 'hf:cat/Cat-E', },),
                  unusableSeat({ modelId: 'hf:cat/Cat-F', },),
                ],
              },],
            },
            '.rounds[0].verdict: expected quorum-not-met, derived from seat statuses and quorum',
          ],
        ] as const) {
          expectRefused({
            value,
            says,
          },);
        }
      },
    },),

    it({
      name: 'REFUSES A ROUND whose findings name a paragraph it did not review, or whose stored findings are not '
        + 'its rejecting seats\' findings',
      fn: async () => {
        expectRefused({
          value: reviewWithSeat(unacceptableSeat({
            modelId: 'hf:cat/Cat-A',
            problem: 'Stored findings leave this out.',
          },),),
          says: '.rounds[0].findings: expected deduplicated unacceptable-seat findings in roster order',
        },);
        expectRefused({
          value: reviewWithSeat({
            ...unacceptableSeat({
              modelId: 'hf:cat/Cat-A',
              problem: 'Names a second paragraph.',
            },),
            findings: [{ paragraph: 2, problem: 'Names a second paragraph.', },],
          },),
          says: '.rounds[0].seats: expected findings naming existing reviewed paragraph',
        },);
      },
    },),

    it({
      name: 'REFUSES A SEAT with an unknown status, a finding at paragraph zero or with no problem, an acceptable '
        + 'seat with findings, an unacceptable seat without, and an unusable seat with a finding or a reason',
      fn: async () => {
        /**
         A rejecting seat, whose findings each case replaces.
         */
        const rejecting = unacceptableSeat({
          modelId: 'hf:cat/Cat-A',
          problem: 'Replace nominalized verb phrase.',
        },);
        for (const [seat, says,] of [
          [
            {
              ...rejecting,
              status: 'sleepy',
            },
            '.rounds[0].seats[0].status: expected one of acceptable, unacceptable, unusable',
          ],
          [
            {
              ...rejecting,
              findings: [{ paragraph: 0, problem: 'Paragraph zero.', },],
            },
            '.rounds[0].seats[0].findings[0].paragraph: expected one-based paragraph number',
          ],
          [
            {
              ...rejecting,
              findings: [{ paragraph: 1, problem: '', },],
            },
            '.rounds[0].seats[0].findings[0].problem: expected non-empty actionable defect',
          ],
          [
            {
              ...acceptableSeat({ modelId: 'hf:cat/Cat-A', },),
              findings: rejecting.findings,
            },
            '.rounds[0].seats[0].findings: expected empty findings for acceptable seat',
          ],
          [
            {
              ...rejecting,
              findings: [],
            },
            '.rounds[0].seats[0].findings: expected at least one finding for unacceptable seat',
          ],
          [
            {
              ...unusableSeat({ modelId: 'hf:cat/Cat-A', },),
              findings: rejecting.findings,
            },
            '.rounds[0].seats[0]: expected unusable seat with empty findings and reason',
          ],
          [
            {
              ...unusableSeat({ modelId: 'hf:cat/Cat-A', },),
              reason: 'timed out',
            },
            '.rounds[0].seats[0]: expected unusable seat with empty findings and reason',
          ],
        ] as const) {
          expectRefused({
            value: reviewWithSeat(seat,),
            says,
          },);
        }
      },
    },),

    it({
      name: 'REFUSES A CANDIDATE DIGEST that is not 64 lowercase hexadecimal characters: one short, and one in '
        + 'capitals',
      fn: async () => {
        for (const candidateDigest of ['a'.repeat(63,), 'A'.repeat(64,),]) {
          expectRefused({
            value: {
              ...REVIEW,
              rounds: [{
                ...REVIEW.rounds[0],
                candidateDigest,
              },],
            },
            says: '.rounds[0].candidateDigest: expected lowercase hexadecimal SHA-256 digest',
          },);
        }
      },
    },),

    it({
      name: 'COUNTS REVIEWED PARAGRAPHS AS THE WRITING GENERATION SHOWED THEM: a blockquote after a paragraph is '
        + 'a second reviewed paragraph where every body block was shown (generation ten), and not where only '
        + 'the refinable paragraphs were',
      fn: async () => {
        /**
         A paragraph, then a blockquote the polish may not edit.
         */
        const quotedText = `${FINAL_TEXT}\n\n> The cat dreamed of fish.`;
        /**
         A no-correction review of it counting both blocks.
         */
        const review = {
          correctionCount: 0,
          rounds: [{
            ...REVIEW.rounds[0],
            candidateDigest: hashContent({ content: quotedText, },),
            paragraphCount: 2,
          },],
        };
        expect(parseNaturalnessReview({
          value: review,
          path: REVIEW_PATH,
          finalText: quotedText,
          everyBodyBlockReviewed: true,
        },),).toEqual(review,);
        expectRefused({
          value: review,
          says: '.rounds[0].paragraphCount: expected structurally correctable paragraph count of final polish text',
          finalText: quotedText,
        },);
      },
    },),

    it({
      name: 'READS A ROUND CLOSED ON A SHORT BENCH by the seats it counted out of reach (ledger E3): five '
        + 'asked over a bench of eight, two refused, three accepting is acceptable only beside that count, '
        + 'and a count wider than the bench is refused, as is a bench smaller than the seats recorded',
      fn: async () => {
        /**
         The round as the stage writes it, before its count of seats out of reach.
         */
        const shortRound = {
          quorumOver: 8,
          candidateDigest: hashContent({ content: FINAL_TEXT, },),
          paragraphCount: 1,
          seats: [
            unusableSeat({ modelId: 'hf:cat/Cat-A', },),
            unusableSeat({ modelId: 'hf:cat/Cat-B', },),
            acceptableSeat({ modelId: 'hf:cat/Cat-C', },),
            acceptableSeat({ modelId: 'hf:cat/Cat-D', },),
            acceptableSeat({ modelId: 'hf:cat/Cat-E', },),
          ],
          usable: 3,
          verdict: 'acceptable',
          findings: [],
        };
        /**
         Review whose one round carries the given count, or none.

         @param unreachable - seats counted out of reach, absent as in a stored artifact

         @returns Review value as an artifact carries it
         */
        function reviewCounting(
          { unreachable, }: { readonly unreachable?: number; },
        ): unknown {
          return {
            correctionCount: 0,
            rounds: [{
              ...shortRound,
              ...((unreachable === undefined) ? {} : { unreachable, }),
            },],
          };
        }
        /**
         Reads a review under the generation that records its quorum basis.

         @param value - review to read

         @returns Parsed review
         */
        function read(value: unknown,): unknown {
          return parseNaturalnessReview({
            value,
            path: REVIEW_PATH,
            finalText: FINAL_TEXT,
            quorumBasisRequired: true,
          },);
        }

        /**
         The short round read back with its count.
         */
        const readBack = read(reviewCounting({ unreachable: 5, },),);
        expect(readBack,).toEqual(reviewCounting({ unreachable: 5, },),);
        expect(() => read(reviewCounting({}),),)
          .toThrow(`at ${REVIEW_PATH}.rounds[0].verdict: expected quorum-not-met, derived from seat statuses and quorum`,);
        expect(() => read(reviewCounting({ unreachable: 9, },),),)
          .toThrow(`at ${REVIEW_PATH}.rounds[0].unreachable: expected at most the 8 seats of the bench`,);
        expect(() =>
          read({
            correctionCount: 0,
            rounds: [{
              ...shortRound,
              quorumOver: 4,
            },],
          },)
          ,)
          .toThrow(`at ${REVIEW_PATH}.rounds[0].quorumOver: expected safe integer quorum basis covering every recorded seat`,);
      },
    },),

    it({
      name: 'READS A ROUND WHOSE LIVE SEATS ARE EXACTLY ITS QUORUM under the rule of its generation: seven seats with '
        + 'three out of reach and three of the four live seats accepting is acceptable under the last-seat grace of '
        + 'generation twenty and quorum-not-met under the rule of 2026-09-09 that nineteen was written under, and each '
        + 'generation refuses the verdict of the other',
      fn: async () => {
        /**
         Round over a bench of seven, three seats out of reach, the fourth
         live seat cut at the grace.

         @param verdict - verdict the round records

         @returns Review value as an artifact carries it
         */
        function reviewDeciding(
          { verdict, }: { readonly verdict: string; },
        ): unknown {
          return {
            correctionCount: 0,
            rounds: [{
              quorumOver: 7,
              unreachable: 3,
              candidateDigest: hashContent({ content: FINAL_TEXT, },),
              paragraphCount: 1,
              seats: [
                unusableSeat({ modelId: 'hf:cat/Cat-A', },),
                acceptableSeat({ modelId: 'hf:cat/Cat-B', },),
                acceptableSeat({ modelId: 'hf:cat/Cat-C', },),
                acceptableSeat({ modelId: 'hf:cat/Cat-D', },),
              ],
              usable: 3,
              verdict,
              findings: [],
            },],
          };
        }
        /**
         Reads a review under the given rule.

         @param value - review to read

         @param lastSeatGrace - whether the generation read has the last-seat grace

         @returns Parsed review
         */
        function readUnder(
          {
            value,
            lastSeatGrace,
          }: {
            readonly value: unknown;
            readonly lastSeatGrace: boolean;
          },
        ): unknown {
          return parseNaturalnessReview({
            value,
            path: REVIEW_PATH,
            finalText: FINAL_TEXT,
            quorumBasisRequired: true,
            lastSeatGrace,
          },);
        }

        expect(readUnder({
          value: reviewDeciding({ verdict: 'acceptable', },),
          lastSeatGrace: true,
        },),).toEqual(reviewDeciding({ verdict: 'acceptable', },),);
        expect(readUnder({
          value: reviewDeciding({ verdict: 'quorum-not-met', },),
          lastSeatGrace: false,
        },),).toEqual(reviewDeciding({ verdict: 'quorum-not-met', },),);
        expect(() =>
          readUnder({
            value: reviewDeciding({ verdict: 'quorum-not-met', },),
            lastSeatGrace: true,
          },)
          ,)
          .toThrow(`at ${REVIEW_PATH}.rounds[0].verdict: expected acceptable, derived from seat statuses and quorum`,);
        expect(() =>
          readUnder({
            value: reviewDeciding({ verdict: 'acceptable', },),
            lastSeatGrace: false,
          },)
          ,)
          .toThrow(`at ${REVIEW_PATH}.rounds[0].verdict: expected quorum-not-met, derived from seat statuses and quorum`,);
      },
    },),
  ],
},);
