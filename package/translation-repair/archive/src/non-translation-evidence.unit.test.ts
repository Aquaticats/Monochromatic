/**
 Tests for the deterministic non-translation contradiction check and
 the vote screening built on it:
 votes below threshold never contradict, content-critique claims
 anchored target-side count toward the floor, missing-translation
 categories plus source-only anchors never count, and dismissed votes
 take their non-translation claims along with a finding.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { ChunkRepairOutcome, } from '@monochromatic-dev/translation-repair-assembly';
import type {
  AdjudicatedIssue,
  IssueClaim,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  assessNonTranslationDominance,
  assessNonTranslationEvidence,
  NON_TRANSLATION_BLOCK_VOTES,
  NON_TRANSLATION_CONTRADICTION_MIN,
  nonTranslationVotesStand,
  screenNonTranslationVotes,
  sliceAnchorsTranslation,
} from '../dist/final/node/index.mjs';

/**
 Builds one claim with chosen category and span side.
 */
function catClaim(
  {
    category,
    side,
    summary,
  }: {
    readonly category: IssueClaim['category'];
    readonly side: 'source' | 'target';
    readonly summary: string;
  },
): IssueClaim {
  return {
    category,
    severity: 'major',
    summary,
    spans: [
      {
        side,
        nodeId: 'block/0',
        nodeHash: 'hash/whisker',
        startOffset: 0,
        endOffset: 4,
        quotedText: side === 'target' ? 'purr' : '呼噜',
      },
    ],
  };
}

/**
 Content-critique claims at exactly the contradiction floor.
 */
const FLOOR_CLAIMS: readonly IssueClaim[] = [
  ...Array.from({ length: NON_TRANSLATION_CONTRADICTION_MIN, },)
    .keys(),
]
  .map(function toClaim(index,) {
    return catClaim({
      category: 'accuracy/mistranslation',
      side: 'target',
      summary: `Purring nuance ${String(index,)} is rendered as hissing.`,
    },);
  },);

/**
 Non-translation claim dismissed together with contradicted votes.
 */
const NON_TRANSLATION_CLAIM: IssueClaim = catClaim({
  category: 'accuracy/non-translation',
  side: 'target',
  summary: 'Pages look unrelated.',
},);

/**
 Wraps one claim as an adjudicated issue with the chosen decision, for
 exercising the anchor probe over a slice's settled issues.
 */
function catIssue(
  {
    claim,
    status,
  }: {
    readonly claim: IssueClaim;
    readonly status: AdjudicatedIssue['status'];
  },
): AdjudicatedIssue {
  return {
    issueId: `issue/${status}/${claim.category}`,
    status,
    severity: claim.severity,
    claims: [
      {
        claimId: 'claim/whisker',
        claim,
      },
    ],
    tallies: {},
  };
}

/**
 Builds one settled slice outcome carrying the given issues and standing
 verdict; only the fields the anchor probe reads carry meaning.
 */
function catOutcome(
  {
    issues,
    nonTranslationStanding,
  }: {
    readonly issues: readonly AdjudicatedIssue[];
    readonly nonTranslationStanding: boolean;
  },
): ChunkRepairOutcome {
  return {
    sliceIndex: 0,
    repairedText: 'purr',
    changed: false,
    issues,
    resolvedIssueIds: [],
    candidateResolvedIssueIds: [],
    // No checker round in this fixture, so nothing was said about any issue.
    checkerReadings: {},
    recheckReadings: {},
    repairRegions: [],
    accuracyPatchSelected: false,
    refined: false,
    rounds: [],
    droppedDeclaredNames: [],
    // Hand-written fixture text, so nothing here has a model author.
    authorship: {
      perIssue: {},
      everyIssue: [],
    },
    nonTranslationVotes: nonTranslationStanding ? NON_TRANSLATION_BLOCK_VOTES : 0,
    nonTranslationContradicted: false,
    nonTranslationStanding,
    heardCritics: 7,
    heardCriticIds: [],
    claimAttributions: [],
    findings: [],
    silentStages: [],
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: assessNonTranslationEvidence.name,
      children: [
        it({
          name: 'contradicts votes at threshold when content critique reaches the floor',
          fn: async () => {
            const evidence = assessNonTranslationEvidence({
              votes: NON_TRANSLATION_BLOCK_VOTES,
              claims: FLOOR_CLAIMS,
            },);
            expect(evidence.contradicted,).toBe(true,);
            expect(evidence.contradictionClaimCount,).toBe(NON_TRANSLATION_CONTRADICTION_MIN,);
          },
        },),

        it({
          name: 'never contradicts votes below the block threshold',
          fn: async () => {
            const evidence = assessNonTranslationEvidence({
              votes: NON_TRANSLATION_BLOCK_VOTES - 1,
              claims: FLOOR_CLAIMS,
            },);
            expect(evidence.contradicted,).toBe(false,);
            expect(evidence.contradictionClaimCount,).toBe(NON_TRANSLATION_CONTRADICTION_MIN,);
          },
        },),

        it({
          name: 'stays uncontradicted one claim under the floor',
          fn: async () => {
            const evidence = assessNonTranslationEvidence({
              votes: NON_TRANSLATION_BLOCK_VOTES,
              claims: FLOOR_CLAIMS.slice(1,),
            },);
            expect(evidence.contradicted,).toBe(false,);
            expect(evidence.contradictionClaimCount,).toBe(NON_TRANSLATION_CONTRADICTION_MIN - 1,);
          },
        },),

        it({
          name: 'excludes missing-translation categories even when target-anchored',
          fn: async () => {
            /**
             Claims whose categories evidence missing translation.
             */
            const missingTranslationClaims: readonly IssueClaim[] = [
              catClaim({
                category: 'accuracy/omission',
                side: 'target',
                summary: 'Napping sentence has no counterpart.',
              },),
              catClaim({
                category: 'accuracy/untranslated',
                side: 'target',
                summary: 'Whisker paragraph stays untranslated.',
              },),
              NON_TRANSLATION_CLAIM,
            ];
            const evidence = assessNonTranslationEvidence({
              votes: NON_TRANSLATION_BLOCK_VOTES,
              claims: [
                ...missingTranslationClaims,
                ...FLOOR_CLAIMS.slice(1,),
              ],
            },);
            expect(evidence.contradicted,).toBe(false,);
            expect(evidence.contradictionClaimCount,).toBe(NON_TRANSLATION_CONTRADICTION_MIN - 1,);
          },
        },),

        it({
          name: 'excludes source-only claims from contradiction',
          fn: async () => {
            /**
             Content-critique claims anchored only in source text.
             */
            const sourceOnlyClaims: readonly IssueClaim[] = [
              ...Array.from({ length: NON_TRANSLATION_CONTRADICTION_MIN, },)
                .keys(),
            ]
              .map(function toClaim(index,) {
                return catClaim({
                  category: 'style/awkward-phrasing',
                  side: 'source',
                  summary: `Sunbeam clause ${String(index,)} reads stiffly.`,
                },);
              },);
            const evidence = assessNonTranslationEvidence({
              votes: NON_TRANSLATION_BLOCK_VOTES,
              claims: sourceOnlyClaims,
            },);
            expect(evidence.contradicted,).toBe(false,);
            expect(evidence.contradictionClaimCount,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: assessNonTranslationDominance.name,
      children: [
        it({
          name: 'blocks when standing slices dominate and none anchors translation',
          fn: async () => {
            const dominance = assessNonTranslationDominance({
              slices: [
                { targetChars: 900, votesStand: true, anchorsTranslation: false, },
                { targetChars: 200, votesStand: false, anchorsTranslation: false, },
              ],
            },);
            expect(dominance.blocked,).toBe(true,);
            expect(dominance.standingChars,).toBe(900,);
            expect(dominance.totalChars,).toBe(1_100,);
          },
        },),

        it({
          name: 'never blocks when a clean-translation anchor survives a standing majority (Mio regression)',
          fn: async () => {
            const dominance = assessNonTranslationDominance({
              slices: [
                { targetChars: 900, votesStand: true, anchorsTranslation: false, },
                { targetChars: 200, votesStand: false, anchorsTranslation: true, },
              ],
            },);
            // Standing chars dominate (900 of 1100), yet a confirmed
            // translation anchor proves the pair is a translation with
            // asymmetric extra content, so repair must proceed.
            expect(dominance.blocked,).toBe(false,);
            expect(dominance.standingChars,).toBe(900,);
          },
        },),

        it({
          name: 'leaves a minority standing region to per-slice degradation',
          fn: async () => {
            const dominance = assessNonTranslationDominance({
              slices: [
                { targetChars: 300, votesStand: true, anchorsTranslation: false, },
                { targetChars: 900, votesStand: false, anchorsTranslation: false, },
              ],
            },);
            expect(dominance.blocked,).toBe(false,);
            expect(dominance.standingChars,).toBe(300,);
          },
        },),

        it({
          name: 'never blocks an empty slice list',
          fn: async () => {
            const dominance = assessNonTranslationDominance({ slices: [], },);
            expect(dominance.blocked,).toBe(false,);
            expect(dominance.totalChars,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: sliceAnchorsTranslation.name,
      children: [
        it({
          name: 'anchors on a non-standing slice with an accepted target content critique',
          fn: async () => {
            const outcome = catOutcome({
              issues: [
                catIssue({
                  claim: catClaim({
                    category: 'accuracy/mistranslation',
                    side: 'target',
                    summary: 'A purr is rendered as a hiss.',
                  },),
                  status: 'accepted',
                },),
              ],
              nonTranslationStanding: false,
            },);
            expect(sliceAnchorsTranslation({ outcome, },),).toBe(true,);
          },
        },),

        it({
          name: 'does not anchor when the slice is a standing non-translation',
          fn: async () => {
            const outcome = catOutcome({
              issues: [
                catIssue({
                  claim: catClaim({
                    category: 'accuracy/mistranslation',
                    side: 'target',
                    summary: 'A purr is rendered as a hiss.',
                  },),
                  status: 'accepted',
                },),
              ],
              nonTranslationStanding: true,
            },);
            expect(sliceAnchorsTranslation({ outcome, },),).toBe(false,);
          },
        },),

        it({
          name: 'does not anchor on a missing-translation leaf even target-side',
          fn: async () => {
            const outcome = catOutcome({
              issues: [
                catIssue({
                  claim: catClaim({
                    category: 'accuracy/omission',
                    side: 'target',
                    summary: 'A whole whisker of content is dropped.',
                  },),
                  status: 'accepted',
                },),
              ],
              nonTranslationStanding: false,
            },);
            expect(sliceAnchorsTranslation({ outcome, },),).toBe(false,);
          },
        },),

        it({
          name: 'does not anchor on a source-only content critique',
          fn: async () => {
            const outcome = catOutcome({
              issues: [
                catIssue({
                  claim: catClaim({
                    category: 'accuracy/mistranslation',
                    side: 'source',
                    summary: 'The source phrasing itself is odd.',
                  },),
                  status: 'accepted',
                },),
              ],
              nonTranslationStanding: false,
            },);
            expect(sliceAnchorsTranslation({ outcome, },),).toBe(false,);
          },
        },),

        it({
          name: 'does not anchor on a rejected content critique',
          fn: async () => {
            const outcome = catOutcome({
              issues: [
                catIssue({
                  claim: catClaim({
                    category: 'accuracy/mistranslation',
                    side: 'target',
                    summary: 'A purr is rendered as a hiss.',
                  },),
                  status: 'rejected',
                },),
              ],
              nonTranslationStanding: false,
            },);
            expect(sliceAnchorsTranslation({ outcome, },),).toBe(false,);
          },
        },),

        it({
          name: 'does not anchor on a non-standing slice with no accepted issues',
          fn: async () => {
            const outcome = catOutcome({
              issues: [],
              nonTranslationStanding: false,
            },);
            expect(sliceAnchorsTranslation({ outcome, },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: screenNonTranslationVotes.name,
      children: [
        it({
          name: 'dismisses contradicted votes with their claims and a finding',
          fn: async () => {
            const screening = screenNonTranslationVotes({
              votes: NON_TRANSLATION_BLOCK_VOTES,
              claims: [
                NON_TRANSLATION_CLAIM,
                ...FLOOR_CLAIMS,
              ],
            },);
            expect(screening.contradicted,).toBe(true,);
            expect(screening.claims,).toEqual(FLOOR_CLAIMS,);
            expect(screening.findings,).toHaveLength(1,);
            expect(screening.findings[0],).toContain('non-translation votes contradicted',);
          },
        },),

        it({
          name: 'passes claims through untouched while votes stand',
          fn: async () => {
            /**
             Claims under the floor, so votes stand.
             */
            const standingClaims: readonly IssueClaim[] = [
              NON_TRANSLATION_CLAIM,
              ...FLOOR_CLAIMS.slice(1,),
            ];
            const screening = screenNonTranslationVotes({
              votes: NON_TRANSLATION_BLOCK_VOTES,
              claims: standingClaims,
            },);
            expect(screening.contradicted,).toBe(false,);
            expect(screening.claims,).toEqual(standingClaims,);
            expect(screening.findings,).toHaveLength(0,);
          },
        },),
      ],
    },),

    describe({
      name: nonTranslationVotesStand.name,
      children: [
        it({
          // AkiraComplex regression: a faithful translation drew two
          // non-translation votes from only two of seven critics heard on an
          // English-epigraph slice. Two votes are one under the floor, so the
          // slice must not stand and the document must not block.
          name: 'does not stand one vote under the floor (AkiraComplex false block)',
          fn: async () => {
            expect(nonTranslationVotesStand({
              votes: NON_TRANSLATION_BLOCK_VOTES - 1,
              contradicted: false,
            },),).toBe(false,);
          },
        },),
        it({
          name: 'stands at the floor when uncontradicted',
          fn: async () => {
            expect(nonTranslationVotesStand({
              votes: NON_TRANSLATION_BLOCK_VOTES,
              contradicted: false,
            },),).toBe(true,);
          },
        },),
        it({
          name: 'does not stand at the floor once contradicted',
          fn: async () => {
            expect(nonTranslationVotesStand({
              votes: NON_TRANSLATION_BLOCK_VOTES,
              contradicted: true,
            },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
