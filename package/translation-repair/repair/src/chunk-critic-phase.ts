import type {
  FanOutMode,
  StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  claimFilersOf,
  describeClaimFiling,
} from './claim-filers.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type ClaimAttribution,
  retainAttributions,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import {
  computeIssueClaimId,
  type IssueClaim,
  type RepairDocument,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  nonTranslationVotesStand,
  screenNonTranslationVotes,
} from '@monochromatic-dev/translation-repair-archive/ts';
import { runCriticStage, } from './repair-stages.ts';

//region Chunk critic phase
// Critics, then the deterministic screen over their non-translation votes.
// Gathered here because the two are never useful apart: a vote count that has
// not been screened is not something any caller may act on, and keeping them
// adjacent in one module is what stops a later caller reading
// `nonTranslationVotes` straight off the critic result and blocking a slice on
// votes the evidence already contradicted.

/**
 Everything the critic phase decided for one chunk.

 @example
 ```ts
 const phase = await runChunkCriticPhase({ ... },);
 if (phase.votesStand) l.warn('non-translation votes stand; carried as evidence',);
 ```
 */
export type ChunkCriticPhase = {
  /**
   Validated claims surviving the screen; empty when contradicted votes took
   their claims down with them.
   */
  readonly claims: readonly IssueClaim[];

  /**
   Critics reporting critical non-translation at wire level, before screening.
   */
  readonly nonTranslationVotes: number;

  /**
   Whether deterministic evidence contradicted those votes.
   */
  readonly contradicted: boolean;

  /**
   Whether votes met the block threshold uncontradicted. Evidence only since
   question 3, answer B (2026-08-16): the slice proceeds, and the votes ride
   on the outcome as `nonTranslationStanding` for the judges and the
   document-level report.
   */
  readonly votesStand: boolean;

  /**
   Critics heard, for the caller's degradation accounting.
   */
  readonly heardCritics: number;

  /**
   WHICH critics answered, sorted by model id. Attribution counts a critic's
   hits; this is every critic whose answer was heard, so a rate per answer can
   be computed rather than only a tally. A critic asked and never heard is not
   here. Unfiltered by screening, since being heard is independent of whether
   the claims survived.
   */
  readonly heardCriticIds: readonly RosterModelId[];

  /**
   Which critics raised each SURVIVING claim, keyed by deterministic claim id.
   Already filtered to the screened claims, unlike the pre-screening list
   `runCriticStage` returns, so a critic is never credited with a claim the
   screen threw away. Calibration only; adjudication never sees it.
   */
  readonly claimAttributions: readonly ClaimAttribution[];

  /**
   Critic findings plus the contradiction record when votes fell.
   */
  readonly findings: readonly string[];
};

/**
 Runs the critics over one chunk pair and screens their non-translation votes.

 @param client - injected model client

 @param criticModelIds - critic fan-out electorate

 @param sourceText - original chunk text

 @param targetText - translation chunk text

 @param documents - parsed chunk pair claims anchor against

 @param identityContext - declared names from both sides' front matter

 @param referenceContext - what the original's cited pages say (class
 thirty-five)

 @param sliceIndex - chunk position, for the dismissal warning

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - pipeline logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Screened claims plus the vote accounting

 @example
 ```ts
 const phase = await runChunkCriticPhase({ ... },);
 ```
 */
export async function runChunkCriticPhase(
  {
    client,
    criticModelIds,
    sourceText,
    targetText,
    documents,
    identityContext,
    referenceContext,
    neighbouringIncumbentText,
    neighbouringSourceText,
    sliceIndex,
    signal,
    perCallTimeoutMs,
    l,
    fanOut,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly criticModelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly targetText: string;
    readonly documents: {
      readonly source: RepairDocument;
      readonly target: RepairDocument;
    };
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly neighbouringIncumbentText?: string;
    readonly neighbouringSourceText?: string;
    readonly sliceIndex: number;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  }>,
): Promise<ChunkCriticPhase> {
  /**
   Critic fan-out result.
   */
  const critic = await runCriticStage({
    client,
    criticModelIds,
    sourceText,
    targetText,
    documents,
    ...(identityContext === undefined ? {} : { identityContext, }),
    ...((referenceContext === undefined) ? {} : { referenceContext, }),
    ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
    ...((neighbouringIncumbentText === undefined) ? {} : { neighbouringIncumbentText, }),
    signal,
    perCallTimeoutMs,
    l,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((fanOut === undefined) ? {} : { fanOut, }),
    stageSettings,
  },);

  /**
   Vote screening against deterministic evidence; contradicted votes fall
   together with their claims.
   */
  const screening = screenNonTranslationVotes({
    votes: critic.nonTranslationVotes,
    claims: critic.claims,
  },);
  if (screening.contradicted) {
    l.warn(
      `chunk ${String(sliceIndex,)}: ${
        String(critic.nonTranslationVotes,)
      } non-translation ${
        wordForCount({
          count: critic.nonTranslationVotes,
          one: 'vote',
          many: 'votes',
        },)
      } dismissed: ${screening.findings
        .join('; ',)}`,
    );
  }

  /**
   Identities of the claims screening left standing. Attribution is collected
   before the screen runs, so anything dropped here must lose its attribution
   too, or a critic keeps credit for a claim the pipeline discarded.
   */
  const survivingClaimIds = new Set(
    screening.claims
      .map(function toClaimId(claim,) {
        return computeIssueClaimId({ claim, },);
      },),
  );

  /**
   Surviving attribution, the filers the log lines and the artifact name.
   */
  const claimAttributions = retainAttributions({
    attributions: critic.claimAttributions,
    claimIds: survivingClaimIds,
  },);

  /**
   Filers per surviving claim id.
   */
  const filers = claimFilersOf({ attributions: claimAttributions, },);
  // ONE LINE PER CLAIM, WITH ITS FILERS (owner, 2026-09-24). The stage's
  // summary line counts claims; a reader chasing one claim needs the critic
  // that filed it, its category, severity and summary, and had to join the
  // artifact's attribution record by claim id to get there.
  for (const claim of screening.claims) {
    l.info(describeClaimFiling({
      sliceIndex,
      claimId: computeIssueClaimId({ claim, },),
      claim,
      filers,
    },),);
  }

  return {
    claims: screening.claims,
    nonTranslationVotes: critic.nonTranslationVotes,
    contradicted: screening.contradicted,
    votesStand: nonTranslationVotesStand({
      votes: critic.nonTranslationVotes,
      contradicted: screening.contradicted,
    },),
    heardCritics: critic.heardCritics,
    heardCriticIds: critic.heardCriticIds,
    claimAttributions,
    findings: [
      ...critic.findings,
      ...screening.findings,
    ],
  };
}

//endregion Chunk critic phase
