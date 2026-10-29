/**
 Goal reviewer transport orchestration over ranked authenticated pool.
 
 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { canonicalSlug, } from '@monochromatic-dev/agent-harness-shared-model-selection/ts';
import {
  ReviewUnavailableError,
  runStructuredJsonRetries,
  runStructuredToolRequest,
  type ScriptedStructuredReviewTransport,
  structuredReviewSignal,
} from '@monochromatic-dev/pi-shared-model-review/ts';

import {
  REVIEW_OUTPUT_TOKENS,
  REVIEW_TIMEOUT_MS,
} from './constants.ts';
import type {
  GoalSettlementReview,
  GoalSettlementReviewer,
  GoalReviewerCandidate,
  GoalReviewVerdict,
} from './completion-types.ts';
import { buildGoalReviewEvidence, } from './review-context.ts';
import {
  buildGoalJsonRetryPrompt,
  GOAL_REVIEW_TOOL,
  GOAL_REVIEW_TOOL_NAME,
  parseGoalReviewVerdict,
} from './review-contract.ts';
import {
  type GoalReviewerPool,
  resolveGoalReviewerPool,
} from './review-selection.ts';

/**
 Reviewer orchestration logger.
 */
const reviewRunnerLogger = tagged({ tag: 'pi-goal-review-runner', },);

/**
 Run one production structured reviewer attempt.
 
 @param candidate - authenticated reviewer and model-specific prompt
 
 @param signal - optional tool cancellation signal
 
 @param testTransport - optional deterministic data-only provider seam
 
 @returns strict reviewer verdict
 
 @mutates candidate - provider consumes model and auth data
 
 @mutates signal - composed cancellation can retain caller signal
 
 @mutates testTransport - deterministic seam advances script and records snapshots
 
 @example
 ```ts
 await runGoalReviewerAttempt({ candidate });
 ```
 */
async function runGoalReviewerAttempt(
  {
    candidate,
    signal: callerSignal,
    testTransport,
  }: {
    readonly candidate: ForeignBorrowed<GoalReviewerCandidate>;
    readonly signal?: AbortSignal;
    readonly testTransport?: ForeignBorrowed<ScriptedStructuredReviewTransport>;
  },
): Promise<GoalReviewVerdict> {
  /**
   Candidate prompt reused by initial request and retry builder.
   */
  const prompt = {
    systemPrompt: candidate.systemPrompt,
    userContent: candidate.userContent,
  };
  /**
   Complete candidate deadline shared across all requests.
   */
  const signal = structuredReviewSignal({
    timeoutMs: REVIEW_TIMEOUT_MS,
    ...(callerSignal === undefined ? {} : { signal: callerSignal, }),
  },);
  /**
   Initial forced-tool provider result.
   */
  const initial = await runStructuredToolRequest({
    model: candidate.model,
    auth: candidate.auth,
    prompt,
    signal,
    toolName: GOAL_REVIEW_TOOL_NAME,
    tool: GOAL_REVIEW_TOOL,
    maxOutputTokens: REVIEW_OUTPUT_TOKENS,
    ...(testTransport === undefined ? {} : { testTransport, }),
  },);
  if (initial.kind === 'toolCall')
    return parseGoalReviewVerdict(initial.arguments,);
  /**
   Goal-specific direct-JSON retry prompt.
   */
  const retryPrompt = buildGoalJsonRetryPrompt({
    initialPrompt: prompt,
    firstAttemptTextContent: initial.textContent,
  },);
  /**
   Unknown direct-JSON value retained only until strict parsing.
   */
  const value = await runStructuredJsonRetries({
    model: candidate.model,
    auth: candidate.auth,
    prompt: retryPrompt,
    signal,
    expectedToolName: GOAL_REVIEW_TOOL_NAME,
    maxOutputTokens: REVIEW_OUTPUT_TOKENS,
    ...(testTransport === undefined ? {} : { testTransport, }),
  },);
  return parseGoalReviewVerdict(value,);
}

/**
 Successful fallback paired with candidate audit metadata.
 
 @example
 ```ts
 const result: GoalFallbackSuccess = { candidate, identity: 'provider/model', verdict };
 ```
 */
type GoalFallbackSuccess = {
  /**
   Candidate returning valid verdict.
   */
  readonly candidate: GoalReviewerCandidate;
  /**
   Canonical reviewer identity.
   */
  readonly identity: string;
  /**
   Strict reviewer verdict.
   */
  readonly verdict: GoalReviewVerdict;
};

/**
 Run one concrete goal fallback attempt and record candidate-labeled failure.
 
 @param candidate - authenticated prompted reviewer
 
 @param signal - optional caller cancellation
 
 @param testTransport - optional deterministic provider script
 
 @param diagnostics - local complete failure audit
 
 @returns valid labeled verdict
 
 @mutates candidate - provider consumes model and auth data
 
 @mutates signal - composed cancellation can retain caller signal
 
 @mutates testTransport - deterministic seam advances script and records snapshots
 
 @mutates diagnostics - records normalized contender failure
 
 @example
 ```ts
 await runGoalFallbackAttempt({ candidate, diagnostics });
 ```
 */
async function runGoalFallbackAttempt(
  {
    candidate,
    signal,
    testTransport,
    diagnostics,
  }: {
    readonly candidate: ForeignBorrowed<GoalReviewerCandidate>;
    readonly signal?: AbortSignal;
    readonly testTransport?: ForeignBorrowed<ScriptedStructuredReviewTransport>;
    readonly diagnostics: string[];
  },
): Promise<GoalFallbackSuccess> {
  /**
   Canonical reviewer identity.
   */
  const identity = canonicalSlug(candidate.model,);
  reviewRunnerLogger.debug(`starting fallback goal reviewer ${identity}`,);
  try {
    return {
      candidate,
      identity,
      verdict: await runGoalReviewerAttempt({
        candidate,
        ...(signal === undefined ? {} : { signal, }),
        ...(testTransport === undefined ? {} : { testTransport, }),
      },),
    };
  }
  catch (error) {
    /**
     Candidate-labeled normalized failure.
     */
    const diagnostic = `${identity}: ${caughtValueText(error,)}`;
    diagnostics.push(diagnostic,);
    reviewRunnerLogger.error(`fallback goal reviewer failed: ${diagnostic}`,);
    throw new Error(
      diagnostic,
      { cause: error, },
    );
  }
}

/**
 Run initial reviewer and distinct bounded concurrent fallbacks from one pool.
 
 @param pool - expected-cost-ranked authenticated candidates
 
 @param signal - optional tool cancellation signal
 
 @param testTransport - optional deterministic data-only provider seam
 
 @returns first valid verdict with winning reviewer audit
 
 @mutates pool - provider attempts consume candidate model and auth data
 
 @mutates signal - candidate attempt can retain caller cancellation signal
 
 @mutates testTransport - deterministic seam advances script and records snapshots
 
 @throws {@link ReviewUnavailableError} when pool is empty or every attempt fails
 
 @example
 ```ts
 await runGoalReviewerPool({ pool });
 ```
 */
async function runGoalReviewerPool(
  {
    pool,
    signal,
    testTransport,
  }: {
    readonly pool: ForeignBorrowed<GoalReviewerPool>;
    readonly signal?: AbortSignal;
    readonly testTransport?: ForeignBorrowed<ScriptedStructuredReviewTransport>;
  },
): Promise<GoalSettlementReview> {
  /**
   Ranked candidates and selection diagnostics.
   */
  const {
    candidates,
    diagnostics: selectionDiagnostics,
  } = pool;
  /**
   Initial and bounded fallback candidates in expected-cost order.
   */
  const [
    firstCandidate,
    firstFallback,
    secondFallback,
  ] = candidates;
  if (firstCandidate === undefined) {
    throw new ReviewUnavailableError({
      attemptedCandidateIdentities: [],
      diagnostics: selectionDiagnostics.length === 0
        ? ['No distinct authenticated reviewer is eligible.',]
        : selectionDiagnostics,
    },);
  }
  /**
   Initial reviewer identity.
   */
  const firstIdentity = canonicalSlug(firstCandidate.model,);
  /**
   Candidate identities whose transports started.
   */
  const attemptedReviewerIdentities: string[] = [firstIdentity,];
  /**
   Complete selection and transport failure audit.
   */
  const diagnostics = [...selectionDiagnostics,];
  reviewRunnerLogger.debug(
    `selected initial goal reviewer ${firstIdentity} from ${candidates.length} authenticated candidates`,
  );
  try {
    return {
      verdict: await runGoalReviewerAttempt({
        candidate: firstCandidate,
        ...(signal === undefined ? {} : { signal, }),
        ...(testTransport === undefined ? {} : { testTransport, }),
      },),
      reviewerIdentity: firstIdentity,
      attemptedReviewerIdentities,
      transcriptTruncated: firstCandidate.transcriptTruncated,
    };
  }
  catch (error) {
    diagnostics.push(`${firstIdentity}: ${caughtValueText(error,)}`,);
    reviewRunnerLogger.error(`initial goal reviewer failed: ${diagnostics.at(-1)}`,);
  }

  if (firstFallback === undefined) {
    throw new ReviewUnavailableError({
      attemptedCandidateIdentities: attemptedReviewerIdentities,
      diagnostics: [
        ...diagnostics,
        'no distinct fallback reviewer is available',
      ],
    },);
  }
  attemptedReviewerIdentities.push(canonicalSlug(firstFallback.model,),);
  /**
   Concurrent fallback attempts started before first await.
   */
  const fallbackAttempts: Promise<GoalFallbackSuccess>[] = [
    runGoalFallbackAttempt({
      candidate: firstFallback,
      diagnostics,
      ...(signal === undefined ? {} : { signal, }),
      ...(testTransport === undefined ? {} : { testTransport, }),
    },),
  ];
  if (secondFallback !== undefined) {
    attemptedReviewerIdentities.push(canonicalSlug(secondFallback.model,),);
    fallbackAttempts.push(runGoalFallbackAttempt({
      candidate: secondFallback,
      diagnostics,
      ...(signal === undefined ? {} : { signal, }),
      ...(testTransport === undefined ? {} : { testTransport, }),
    },),);
  }
  try {
    /**
     First fulfilled strict verdict; rejected transports do not settle race.
     */
    const winner = await Promise.any(fallbackAttempts,);
    return {
      verdict: winner.verdict,
      reviewerIdentity: winner.identity,
      attemptedReviewerIdentities,
      transcriptTruncated: winner.candidate
        .transcriptTruncated,
    };
  }
  catch (error) {
    throw new ReviewUnavailableError({
      attemptedCandidateIdentities: attemptedReviewerIdentities,
      diagnostics,
      cause: error,
    },);
  }
}

/**
 Production settlement reviewer building active-branch evidence and scoped pool.
 
 @param request - captured active settlement
 
 @param context - current Pi extension context
 
 @param signal - optional review cancellation signal
 
 @returns first valid verdict and reviewer audit
 
 @mutates context - branch, scope resolution, and auth can change Pi-owned state
 
 @mutates signal - shared attempt cancellation can retain caller signal
 
 @throws {@link ReviewUnavailableError} when every eligible attempt fails
 
 @example
 ```ts
 await reviewGoalSettlement({ request, context });
 ```
 */
async function reviewGoalSettlement(
  {
    request,
    context,
    signal,
  }: Parameters<GoalSettlementReviewer>[0],
): Promise<GoalSettlementReview> {
  /**
   Selected active branch captured before reviewer awaits.
   */
  const branch = context.sessionManager
    .getBranch();
  /**
   Finalized post-start evidence including settled assistant output.
   */
  const evidence = buildGoalReviewEvidence({
    branch,
    request,
  },);
  /**
   Ranked authenticated reviewer pool.
   */
  const pool = await resolveGoalReviewerPool({
    context,
    evidence,
  },);
  return await runGoalReviewerPool({
    pool,
    ...(signal === undefined ? {} : { signal, }),
  },);
}

export {
  reviewGoalSettlement,
  runGoalReviewerAttempt,
  runGoalReviewerPool,
};
