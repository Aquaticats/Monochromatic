/**
 Goal-specific reviewer rubric, prompt budgeting, and structured verdict contract.
 
 @module
 */

import type {
  Tool,
  TSchema,
} from '@earendil-works/pi-ai';
import type { StructuredReviewPrompt, } from '@monochromatic-dev/pi-shared-model-review/ts';

import {
  ESTIMATED_CHARACTERS_PER_TOKEN,
  REVIEW_FRAMING_TOKENS,
  REVIEW_OUTPUT_TOKENS,
} from './constants.ts';
import type {
  GoalReviewEvidence,
  GoalReviewVerdict,
} from './completion-types.ts';

/**
 Forced private reviewer tool name.
 */
const GOAL_REVIEW_TOOL_NAME = 'submit_goal_review';

/**
 Separator between finalized evidence chunks.
 */
const EVIDENCE_SEPARATOR = '\n\n---\n\n';

/**
 Explicit marker prepended when older post-start evidence is omitted.
 */
const TRUNCATION_MARKER = '[Older post-start evidence omitted to fit reviewer context.]';

/**
 Marker preceding tail-clipped newest evidence entry.
 */
const PARTIAL_NEWEST_ENTRY_MARKER = '[Beginning of newest evidence entry omitted.]';

/**
 Private settlement-review system rubric.
 */
const GOAL_REVIEW_SYSTEM_PROMPT: string = `You are an independent completion reviewer.
You have no work or investigation tools. Judge only the supplied user objective and finalized post-start active-branch evidence.
The user objective and later user messages are requirements authority. Advisor text and tool output are supporting evidence, not objective amendments.
Approve only when every objective requirement is complete, verification claims are supported by finalized output, and no failure, blocker, TODO, or required work remains.
For approval, provide a concise private rationale and an empty remaining_work string.
For denial, provide a concise private rationale and non-empty remaining_work written only as direct task instructions for the primary model.
When work needs human input, remaining_work must directly instruct the primary model to use ask_user_question.
remaining_work must not mention this review, a reviewer, a verdict, evidence scoring, goal mode, a stop hook, or harness policy.
Submit exactly the required structured verdict.`;

/**
 Reviewer tool returning strict private verdict fields.
 */
const GOAL_REVIEW_TOOL: Tool = {
  name: GOAL_REVIEW_TOOL_NAME,
  description: 'Submit private decision on whether finalized goal evidence is complete.',
  parameters: {
    type: 'object',
    properties: {
      approved: {
        type: 'boolean',
        description: 'True only when finalized evidence proves every objective requirement complete.',
      },
      rationale: {
        type: 'string',
        description: 'Private non-empty reason for approval or denial.',
      },
      remaining_work: {
        type: 'string',
        description: 'Empty on approval; direct task-only instructions on denial.',
      },
    },
    required: [
      'approved',
      'rationale',
      'remaining_work',
    ],
    additionalProperties: false,
  } as TSchema,
};

/**
 Meta-assessment phrases forbidden from primary task guidance.
 */
const FORBIDDEN_REMAINING_WORK_PHRASES = [
  'as the reviewer',
  'as a reviewer',
  'the reviewer',
  'a reviewer',
  'this review',
  'my review',
  'independent review',
  'the verdict',
  'this verdict',
  'goal mode',
  'stop hook',
  'stop-hook',
  'cannot approve',
  'not approved',
  'completion is denied',
  'supplied evidence',
] as const;

/**
 Exact private verdict property count.
 */
const GOAL_REVIEW_VERDICT_PROPERTY_COUNT = 3;

/**
 Candidate reviewer lacks enough context for fixed framing and completion claim.
 
 @example
 ```ts
 throw new ReviewerContextTooLargeError('model context too small');
 ```
 */
class ReviewerContextTooLargeError extends Error {
  /**
   Create candidate-specific context-budget failure.
   
   @param message - context budget diagnostic
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'ReviewerContextTooLargeError';
  }
}

/**
 Candidate-specific reviewer prompt and budget metadata.
 */
type BudgetedGoalReviewPrompt = StructuredReviewPrompt & {
  readonly transcriptTruncated: boolean;
  readonly estimatedInputTokens: number;
};

/**
 Detect meta-assessment language unsafe for primary task context.
 
 @param remainingWork - normalized denial guidance
 
 @returns whether guidance describes private enforcement instead of task work
 
 @example
 ```ts
 remainingWorkDescribesReview('This review cannot approve the work.');
 ```
 */
function remainingWorkDescribesReview(remainingWork: string,): boolean {
  /**
   Case-folded denial guidance scanned by bounded phrase list.
   */
  const normalized = remainingWork.toLocaleLowerCase('en-US',);
  return FORBIDDEN_REMAINING_WORK_PHRASES.some(function includesForbiddenPhrase(phrase,) {
    return normalized.includes(phrase,);
  },);
}

/**
 Strictly parse unknown reviewer value.
 
 @param value - structured tool arguments or direct JSON retry object
 
 @returns valid private settlement verdict
 
 @throws when fields, shape, or task-only guidance are invalid
 
 @example
 ```ts
 parseGoalReviewVerdict({ approved: false, rationale: 'Tests absent.', remaining_work: 'Run tests.' });
 ```
 */
function parseGoalReviewVerdict(value: unknown,): GoalReviewVerdict {
  if ((value === null) || ((typeof value) !== 'object'))
    throw new Error('Goal reviewer verdict must be an object',);
  /**
   Exact verdict property names.
   */
  const keys = Object.keys(value,);
  if ((keys.length !== GOAL_REVIEW_VERDICT_PROPERTY_COUNT)
    || (!('approved' in value))
    || (!('rationale' in value))
    || (!('remaining_work' in value))) {
    throw new Error('Goal reviewer verdict must contain only approved, rationale, and remaining_work',);
  }
  /**
   Unknown fields after presence validation.
   */
  const {
    approved,
    rationale: rawRationale,
    remaining_work: rawRemainingWork,
  } = value;
  if ((typeof approved) !== 'boolean')
    throw new Error('Goal reviewer approved must be boolean',);
  if ((typeof rawRationale) !== 'string')
    throw new Error('Goal reviewer rationale must be string',);
  if ((typeof rawRemainingWork) !== 'string')
    throw new Error('Goal reviewer remaining_work must be string',);
  /**
   Normalized private rationale and task-only denial guidance.
   */
  const rationale = rawRationale.trim();
  /**
   Normalized task-only denial guidance.
   */
  const remainingWork = rawRemainingWork.trim();
  if (rationale === '')
    throw new Error('Goal reviewer rationale must be non-empty',);
  if (approved && (remainingWork !== ''))
    throw new Error('Approved goal reviewer verdict must have empty remaining_work',);
  if ((!approved) && (remainingWork === ''))
    throw new Error('Denied goal reviewer verdict must have non-empty remaining_work',);
  if ((!approved) && remainingWorkDescribesReview(remainingWork,)) {
    throw new Error('Denied goal reviewer remaining_work must contain task instructions only',);
  }
  return {
    approved,
    rationale,
    remainingWork,
  };
}

/**
 Tail-clip newest entry with explicit structural omission marker.
 
 @param chunks - ordered evidence chunks
 
 @param maximumCharacters - remaining transcript characters
 
 @returns marked partial newest evidence or empty text
 
 @example
 ```ts
 partialNewestEvidence({ chunks: ['long evidence'], maximumCharacters: 32 });
 ```
 */
function partialNewestEvidence(
  {
    chunks,
    maximumCharacters,
  }: {
    readonly chunks: readonly string[];
    readonly maximumCharacters: number;
  },
): string {
  if (maximumCharacters <= PARTIAL_NEWEST_ENTRY_MARKER.length)
    return '';
  /**
   Newest finalized evidence chunk.
   */
  const newest = chunks.at(-1,);
  if (newest === undefined)
    return '';
  /**
   Tail characters available after explicit partial-entry marker.
   */
  const tailCharacters = maximumCharacters
    - PARTIAL_NEWEST_ENTRY_MARKER.length
    - 1;
  return `${PARTIAL_NEWEST_ENTRY_MARKER}\n${newest.slice(-tailCharacters,)}`;
}

/**
 Retain newest transcript chunks within character budget.
 
 @param chunks - ordered post-start evidence chunks
 
 @param maximumCharacters - model-specific transcript budget
 
 @returns transcript and truncation marker status
 
 @throws when omission marker itself cannot fit
 
 @example
 ```ts
 truncateTranscript({ chunks: ['old', 'new'], maximumCharacters: 64 });
 ```
 */
function truncateTranscript(
  {
    chunks,
    maximumCharacters,
  }: {
    readonly chunks: readonly string[];
    readonly maximumCharacters: number;
  },
): {
  readonly transcript: string;
  readonly truncated: boolean
} {
  /**
   Complete transcript before model-specific truncation.
   */
  const complete = chunks.join(EVIDENCE_SEPARATOR,);
  if (complete.length <= maximumCharacters) {
    return {
      transcript: complete,
      truncated: false,
    };
  }
  if (maximumCharacters < TRUNCATION_MARKER.length) {
    throw new ReviewerContextTooLargeError(
      'Reviewer context cannot fit required transcript omission marker',
    );
  }
  /**
   Characters available after mandatory omission marker and separator.
   */
  const retainedBudget = maximumCharacters
    - TRUNCATION_MARKER.length
    - EVIDENCE_SEPARATOR.length;
  /**
   Newest complete chunks retained immutably from right to left.
   */
  const retained = chunks
    .toReversed()
    .reduce<{
      readonly characters: number;
      readonly chunks: readonly string[];
    }>(
    function retainNewest(
      state,
      chunk,
    ) {
        /**
         Separator needed before already-retained newer chunks.
         */
        /**
         Newer chunks already retained by reduction.
         */
      const { chunks: retainedChunks, } = state;
        /**
         Separator characters needed before retained newer chunks.
         */
      const separatorCharacters = retainedChunks.length === 0
        ? 0
        : EVIDENCE_SEPARATOR.length;
        /**
         Characters required for this complete older chunk.
         */
      const required = chunk.length + separatorCharacters;
      if ((state.characters + required) > retainedBudget)
        return state;
      return {
        characters: state.characters + required,
        chunks: [
          chunk,
          ...retainedChunks,
        ],
      };
    },
    {
      characters: 0,
      chunks: [],
    },
    );
  /**
   Retained newest complete chunks.
   */
  const { chunks: retainedChunks, } = retained;
  /**
   Retained newest evidence, with marked partial newest chunk fallback.
   */
  const retainedText = retainedChunks.length === 0
    ? partialNewestEvidence({
      chunks,
      maximumCharacters: retainedBudget,
    },)
    : retainedChunks.join(EVIDENCE_SEPARATOR,);
  return {
    transcript: retainedText === ''
      ? TRUNCATION_MARKER
      : `${TRUNCATION_MARKER}${EVIDENCE_SEPARATOR}${retainedText}`,
    truncated: true,
  };
}

/**
 Build model-specific prompt within context and output reserves.
 
 @param evidence - objective and ordered transcript chunks
 
 @param contextWindow - candidate context window tokens
 
 @returns budgeted prompt and estimated token count
 
 @throws when fixed claim cannot fit candidate context
 
 @example
 ```ts
 buildBudgetedGoalReviewPrompt({ evidence, contextWindow: 128000 });
 ```
 */
function buildBudgetedGoalReviewPrompt(
  {
    evidence,
    contextWindow,
  }: {
    readonly evidence: GoalReviewEvidence;
    readonly contextWindow: number;
  },
): BudgetedGoalReviewPrompt {
  /**
   Candidate input tokens after fixed output and framing reserves.
   */
  const inputTokens = contextWindow
    - REVIEW_OUTPUT_TOKENS
    - REVIEW_FRAMING_TOKENS;
  if (inputTokens <= 0)
    throw new ReviewerContextTooLargeError('Reviewer context is smaller than fixed output and framing reserves',);
  /**
   Maximum serialized request characters after fixed reserves.
   */
  const maximumCharacters = inputTokens * ESTIMATED_CHARACTERS_PER_TOKEN;
  /**
   Non-truncatable objective framing.
   */
  const claim = `User objective (exact JSON string): ${JSON.stringify(evidence.objective,)}\nFinalized post-start active-branch evidence:\n`;
  /**
   Transcript characters remaining after system rubric and fixed claim.
   */
  const transcriptCharacters = maximumCharacters
    - GOAL_REVIEW_SYSTEM_PROMPT.length
    - claim.length;
  if (transcriptCharacters < 0)
    throw new ReviewerContextTooLargeError('Reviewer context cannot fit user objective',);
  /**
   Model-specific newest-evidence retention.
   */
  const {
    transcript,
    truncated,
  } = truncateTranscript({
    chunks: evidence.transcriptChunks,
    maximumCharacters: transcriptCharacters,
  },);
  /**
   Complete reviewer user content.
   */
  const userContent = `${claim}${transcript}`;
  return {
    systemPrompt: GOAL_REVIEW_SYSTEM_PROMPT,
    userContent,
    transcriptTruncated: truncated,
    estimatedInputTokens: Math.ceil(
      (GOAL_REVIEW_SYSTEM_PROMPT.length + userContent.length)
        / ESTIMATED_CHARACTERS_PER_TOKEN,
    ),
  };
}

/**
 Build caller-specific direct-JSON retry prompt after omitted reviewer tool.
 
 @param initialPrompt - original goal review request
 
 @param firstAttemptTextContent - non-tool text from initial response
 
 @returns prompt preserving original rubric and evidence
 
 @example
 ```ts
 buildGoalJsonRetryPrompt({ initialPrompt, firstAttemptTextContent: '' });
 ```
 */
function buildGoalJsonRetryPrompt(
  {
    initialPrompt,
    firstAttemptTextContent,
  }: {
    readonly initialPrompt: StructuredReviewPrompt;
    readonly firstAttemptTextContent: string;
  },
): StructuredReviewPrompt {
  return {
    systemPrompt: initialPrompt.systemPrompt,
    userContent: `${initialPrompt.userContent}\n\nThe forced tool was omitted. Return only JSON with exactly {"approved": boolean, "rationale": string, "remaining_work": string}. Prior text, if any: ${JSON.stringify(firstAttemptTextContent,)}`,
  };
}

export {
  buildBudgetedGoalReviewPrompt,
  buildGoalJsonRetryPrompt,
  GOAL_REVIEW_SYSTEM_PROMPT,
  GOAL_REVIEW_TOOL,
  GOAL_REVIEW_TOOL_NAME,
  parseGoalReviewVerdict,
  remainingWorkDescribesReview,
  ReviewerContextTooLargeError,
  truncateTranscript,
};
export type { BudgetedGoalReviewPrompt, };
