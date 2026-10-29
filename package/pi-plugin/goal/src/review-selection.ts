/**
 Scoped, distinct, authenticated reviewer selection and shared fallback transport.
 
 @module
 */

import type {
  Api,
  Model,
} from '@earendil-works/pi-ai';
import type { ExtensionContext, } from '@earendil-works/pi-coding-agent';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  buildCostRanking,
  canonicalSlug,
  resolveEffectiveScope,
} from '@monochromatic-dev/agent-harness-shared-model-selection/ts';
import { REVIEW_OUTPUT_TOKENS, } from './constants.ts';
import type {
  GoalReviewEvidence,
  GoalReviewerCandidate,
} from './completion-types.ts';
import {
  buildBudgetedGoalReviewPrompt,
  ReviewerContextTooLargeError,
} from './review-contract.ts';

/**
 Prompted scoped candidate before auth resolution.
 */
type PromptedReviewer = {
  readonly model: ForeignBorrowed<Model<Api>>;
  readonly canonicalSlug: string;
  readonly systemPrompt: string;
  readonly userContent: string;
  readonly transcriptTruncated: boolean;
  readonly estimatedInputTokens: number;
};

/**
 Authenticated pool in descending expected-call-cost order.
 */
type GoalReviewerPool = {
  readonly candidates: readonly GoalReviewerCandidate[];
  readonly diagnostics: readonly string[];
};

/**
 Candidate prompt construction outcome.
 */
type PromptedReviewerOutcome =
  | {
    readonly eligible: true;
    readonly prompted: PromptedReviewer;
  }
    | {
      readonly eligible: false;
      readonly diagnostic: string;
    };

/**
 Candidate authentication outcome.
 */
type AuthenticatedReviewerOutcome =
  | {
    readonly authenticated: true;
    readonly candidate: GoalReviewerCandidate;
  }
    | {
      readonly authenticated: false;
      readonly diagnostic: string;
    };

/**
 Candidate not present in finite authenticated reviewer pool.
 
 @example
 ```ts
 throw new NoEligibleGoalReviewerError();
 ```
 */
class NoEligibleGoalReviewerError extends Error {
  /**
   Create exhausted finite-pool marker.
   */
  constructor() {
    super('No distinct authenticated goal reviewer remains in effective model scope',);
    this.name = 'NoEligibleGoalReviewerError';
  }
}

/**
 Build model-specific prompts for context-eligible distinct scoped models.
 
 @param context - Pi context exposing effective model scope and active model
 
 @param evidence - serialized post-start evidence
 
 @returns prompted candidates and context diagnostics
 
 @mutates context - effective scope resolution invokes model registry and optional scope capabilities
 
 @example
 ```ts
 await promptedReviewers({ context, evidence });
 ```
 */
async function promptedReviewers(
  {
    context,
    evidence,
  }: {
    readonly context: ForeignBorrowed<ExtensionContext>;
    readonly evidence: GoalReviewEvidence;
  },
): Promise<{
  readonly prompted: readonly PromptedReviewer[];
  readonly diagnostics: readonly string[];
}> {
  if (context.model === undefined) {
    return {
      prompted: [],
      diagnostics: ['No active primary model is available for distinct-reviewer exclusion.',],
    };
  }
  /**
   Exact primary identity excluded from every reviewer stage.
   */
  const activeIdentity = canonicalSlug(context.model,);
  /**
   Effective scoped set following live, argv, settings, and available precedence.
   */
  const scope = await resolveEffectiveScope<Model<Api>>({ ctx: context, },);
  /**
   Prompt construction outcomes preserving ineligible diagnostics.
   */
  const outcomes = scope.entries
    .filter(function excludesPrimary(
      entry: ForeignBorrowed<(typeof scope.entries)[number]>,
    ) {
      return entry.canonicalSlug !== activeIdentity;
    },)
    .map(function buildCandidatePrompt(
      entry: ForeignBorrowed<(typeof scope.entries)[number]>,
    ): PromptedReviewerOutcome {
      try {
        /**
         Model-specific prompt and deterministic truncation.
         */
        const prompt = buildBudgetedGoalReviewPrompt({
          evidence,
          contextWindow: entry.model
            .contextWindow,
        },);
        return {
          eligible: true as const,
          prompted: {
            model: entry.model,
            canonicalSlug: entry.canonicalSlug,
            ...prompt,
          },
        };
      }
      catch (error) {
        if (!(error instanceof ReviewerContextTooLargeError))
          throw error;
        return {
          eligible: false as const,
          diagnostic: `${entry.canonicalSlug}: ${error.message}`,
        };
      }
    },);
  return {
    prompted: outcomes.flatMap(function keepPrompted(
      outcome: PromptedReviewerOutcome,
    ) {
      return outcome.eligible ? [outcome.prompted,] : [];
    },),
    diagnostics: outcomes.flatMap(function keepDiagnostic(
      outcome: PromptedReviewerOutcome,
    ) {
      return outcome.eligible ? [] : [outcome.diagnostic,];
    },),
  };
}

/**
 Sort prompted candidates by expected model-specific request cost.
 
 @param prompted - context-eligible scoped candidates
 
 @returns candidates in descending expected cost order
 
 @example
 ```ts
 rankPromptedReviewers(prompted);
 ```
 */
function rankPromptedReviewers(
  prompted: readonly PromptedReviewer[],
): readonly PromptedReviewer[] {
  if (prompted.length === 0)
    return [];
  /**
   Per-model input estimates keyed by canonical slug.
   */
  const estimates = new Map(
    prompted.map(function estimateEntry(candidate,) {
      return [
        candidate.canonicalSlug,
        candidate.estimatedInputTokens,
      ] as const;
    },),
  );
  /**
   Expected-cost ranking from shared Advisor-style selection policy.
   */
  const ranking = buildCostRanking({
    scope: {
      source: 'live',
      entries: prompted.map(function scopedCandidate(candidate,) {
        return {
          model: candidate.model,
          canonicalSlug: candidate.canonicalSlug,
        };
      },),
    },
    estimatedInputTokensBySlug: estimates,
    maxOutputTokens: REVIEW_OUTPUT_TOKENS,
    errorPrefix: 'goal reviewer selection',
  },);
  return ranking.map(function rankedCandidate(score,) {
    /**
     Prompted candidate matching ranked score identity.
     */
    const candidate = prompted.find(function matchesScore(candidateToCheck,) {
      return candidateToCheck.canonicalSlug === score.slug;
    },);
    if (candidate === undefined)
      throw new Error(`Ranked reviewer disappeared from prompted scope: ${score.slug}`,);
    return candidate;
  },);
}

/**
 Authenticate ranked reviewer candidates before bounded fallback orchestration.
 
 @param context - Pi model registry context
 
 @param prompted - candidates in expected-cost order
 
 @returns authenticated candidates and normalized auth failures
 
 @mutates context - model registry may execute command-backed authentication
 
 @example
 ```ts
 await authenticateReviewers({ context, prompted });
 ```
 */
async function authenticateReviewers(
  {
    context,
    prompted,
  }: {
    readonly context: ForeignBorrowed<ExtensionContext>;
    readonly prompted: readonly PromptedReviewer[];
  },
): Promise<GoalReviewerPool> {
  /**
   Parallel auth outcomes retain input order through Promise.all.
   */
  const outcomes: readonly AuthenticatedReviewerOutcome[] = await Promise.all(prompted.map(async function authenticate(candidate,) {
    try {
      /**
       Registry-resolved request credentials and headers.
       */
      const auth = await context.modelRegistry
        .getApiKeyAndHeaders(candidate.model,);
      if (!auth.ok) {
        return {
          authenticated: false as const,
          diagnostic: `${candidate.canonicalSlug}: authentication unavailable`,
        };
      }
      return {
        authenticated: true as const,
        candidate: {
          model: candidate.model,
          auth: {
            ...(auth.apiKey === undefined ? {} : { apiKey: auth.apiKey, }),
            ...(auth.headers === undefined ? {} : { headers: auth.headers, }),
          },
          systemPrompt: candidate.systemPrompt,
          userContent: candidate.userContent,
          transcriptTruncated: candidate.transcriptTruncated,
        },
      };
    }
    catch (error) {
      return {
        authenticated: false as const,
        diagnostic: `${candidate.canonicalSlug}: ${caughtValueText(error,)}`,
      };
    }
  },));
  return {
    candidates: outcomes.flatMap(function keepAuthenticated(
      outcome: AuthenticatedReviewerOutcome,
    ) {
      return outcome.authenticated ? [outcome.candidate,] : [];
    },),
    diagnostics: outcomes.flatMap(function keepAuthDiagnostic(
      outcome: AuthenticatedReviewerOutcome,
    ) {
      return outcome.authenticated ? [] : [outcome.diagnostic,];
    },),
  };
}

/**
 Resolve complete authenticated reviewer pool for one completion claim.
 
 @param context - Pi tool execution context
 
 @param evidence - serialized active-run evidence
 
 @returns ranked authenticated candidates and selection diagnostics
 
 @mutates context - scope and authentication capabilities may update registry state
 
 @example
 ```ts
 await resolveGoalReviewerPool({ context, evidence });
 ```
 */
async function resolveGoalReviewerPool(
  {
    context,
    evidence,
  }: {
    readonly context: ForeignBorrowed<ExtensionContext>;
    readonly evidence: GoalReviewEvidence;
  },
): Promise<GoalReviewerPool> {
  /**
   Context-eligible prompted candidates and diagnostics.
   */
  const promptResult = await promptedReviewers({
    context,
    evidence,
  },);
  /**
   Authenticated pool following expected-cost order.
   */
  const authResult = await authenticateReviewers({
    context,
    prompted: rankPromptedReviewers(promptResult.prompted,),
  },);
  return {
    candidates: authResult.candidates,
    diagnostics: [
      ...promptResult.diagnostics,
      ...authResult.diagnostics,
    ],
  };
}

export {
  authenticateReviewers,
  NoEligibleGoalReviewerError,
  promptedReviewers,
  rankPromptedReviewers,
  resolveGoalReviewerPool,
};
export type {
  GoalReviewerPool,
  PromptedReviewer,
};
