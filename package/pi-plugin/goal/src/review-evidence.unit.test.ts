/**
 Built-artifact tests for goal evidence and reviewer contract.
 
 @module
 */

import type {
  SessionEntry,
  SessionMessageEntry,
} from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ESTIMATED_CHARACTERS_PER_TOKEN,
  buildBudgetedGoalReviewPrompt,
  buildGoalReviewEvidence,
  parseGoalReviewVerdict,
  reduceGoalEvents,
  remainingWorkDescribesReview,
  ReviewerContextTooLargeError,
  truncateTranscript,
  type ActiveGoalState,
  type BudgetedGoalReviewPrompt,
  type GoalSettlementReviewRequest,
} from '../dist/final/node/index.mjs';

/** Stable start timestamp. */
const STARTED_AT = '2026-08-26T00:00:00.000Z';

/** Stable completion timestamp. */
const COMPLETED_AT = '2026-08-26T00:01:00.000Z';

/** Exact incident answer sentinel. */
const INCIDENT_ANSWER = '67 is prime in these five ways:';

/** Candidate context window used by reviewer rubric fixtures. */
const REVIEWER_PROMPT_CONTEXT_WINDOW = 20_000;

/** Interactive human-input clause promising the user question tool. */
const INTERACTIVE_HUMAN_INPUT_CLAUSE = 'must directly instruct the primary model to use ask_user_question';

/** Non-interactive human-input clause forbidding user questions. */
const NON_INTERACTIVE_HUMAN_INPUT_CLAUSE = 'The user cannot be asked in this session';

/** Hard-work clause demanding a fresh next attempt. */
const CONCRETE_NEXT_ATTEMPT_CLAUSE = 'one concrete next attempt';

/**
 Build candidate reviewer prompt for one session capability.
 
 @param humanInputAvailable - whether the fixture session can reach the user
 
 @returns budgeted reviewer prompt over the shared evidence fixture
 */
function reviewerPromptFixture(humanInputAvailable: boolean,): BudgetedGoalReviewPrompt {
  return buildBudgetedGoalReviewPrompt({
    evidence: {
      objective: 'Explain why 67 is prime in five ways',
      transcriptChunks: ['Assistant:\n67 is prime in these five ways:',],
    },
    contextWindow: REVIEWER_PROMPT_CONTEXT_WINDOW,
    humanInputAvailable,
  },);
}

/**
 Build active evidence-test goal.
 
 @returns active goal state
 */
function evidenceGoal(): ActiveGoalState {
  /** Reduced active fixture. */
  const goal = reduceGoalEvents([{
    kind: 'run_started',
    runId: 'run-1',
    generationId: 'generation-1',
    objective: 'Explain why 67 is prime in five ways',
    startedAt: STARTED_AT,
    startBoundary: 'leaf-before-start',
    continuationSequence: 0,
    transitionedAt: STARTED_AT,
  },],);
  if (goal.phase !== 'active')
    throw new Error('expected active evidence fixture',);
  return goal;
}

/**
 Build captured settlement request.
 
 @returns request bound to active goal
 */
function evidenceRequest(): GoalSettlementReviewRequest {
  return {
    goal: evidenceGoal(),
    runtimeEpoch: 'runtime-1',
    branchLeafId: 'answer',
    settlementSequence: 0,
  };
}

/**
 Build finalized assistant text fixture.
 
 @param text - visible assistant output
 
 @returns finalized session message
 */
function assistantTextMessage(text: string,): SessionMessageEntry['message'] {
  return {
    role: 'assistant',
    content: [{ type: 'text', text, },],
    api: 'openai-completions',
    provider: 'primary',
    model: 'primary-model',
    usage: {
      input: 1,
      output: 1,
      cacheRead: 0,
      cacheWrite: 0,
      totalTokens: 2,
      cost: {
        input: 0,
        output: 0,
        cacheRead: 0,
        cacheWrite: 0,
        total: 0,
      },
    },
    stopReason: 'stop',
    timestamp: 1,
  };
}

await describe({
  name: buildGoalReviewEvidence.name,
  children: [
    it({
      name: 'includes finalized incident answer and excludes pre-goal and private state',
      fn: async () => {
        /** Selected active branch with task and finalized answer evidence. */
        const branch: SessionEntry[] = [
          {
            type: 'message',
            id: 'pre-goal',
            parentId: null,
            timestamp: STARTED_AT,
            message: {
              role: 'user',
              content: 'secret pre-goal history',
              timestamp: 0,
            },
          },
          {
            type: 'custom',
            customType: 'goal:state',
            id: 'start',
            parentId: 'pre-goal',
            timestamp: STARTED_AT,
            data: {
              kind: 'run_started',
              runId: 'run-1',
              generationId: 'generation-1',
              objective: 'Explain why 67 is prime in five ways',
              startedAt: STARTED_AT,
              startBoundary: 'leaf-before-start',
              continuationSequence: 0,
              transitionedAt: STARTED_AT,
            },
          },
          {
            type: 'custom_message',
            customType: 'goal',
            id: 'task-context',
            parentId: 'start',
            timestamp: STARTED_AT,
            content: 'User objective: explain primality',
            display: true,
            details: {
              runId: 'run-1',
              generationId: 'generation-1',
              continuationSequence: 0,
              marker: 'marker-1',
              kind: 'kickoff',
            },
          },
          {
            type: 'message',
            id: 'answer',
            parentId: 'task-context',
            timestamp: COMPLETED_AT,
            message: assistantTextMessage(
              `${INCIDENT_ANSWER}\n1. Trial division\n2. Sieve\n3. Wilson\n4. Lucas\n5. Residues`,
            ),
          },
        ];
        /** Serialized settlement evidence. */
        const evidence = buildGoalReviewEvidence({
          branch,
          request: evidenceRequest(),
        },);
        const transcript = evidence.transcriptChunks.join('\n',);
        expect(transcript,).toContain(INCIDENT_ANSWER,);
        expect(transcript,).toContain('User objective: explain primality',);
        expect(transcript,).not.toContain('secret pre-goal history',);
        expect(transcript,).not.toContain('generationId',);
        expect(evidence,).not.toHaveProperty('summary',);
      },
    },),
    it({
      name: 'keeps advisor exploration subordinate when finalized answer is absent',
      fn: async () => {
        /** Branch containing only private state and abandoned exploratory evidence. */
        const branch: SessionEntry[] = [
          {
            type: 'custom',
            customType: 'goal:state',
            id: 'start',
            parentId: null,
            timestamp: STARTED_AT,
            data: {
              kind: 'run_started',
              runId: 'run-1',
              generationId: 'generation-1',
              objective: 'Explain why 67 is prime in five ways',
              startedAt: STARTED_AT,
              startBoundary: 'leaf-before-start',
              continuationSequence: 0,
              transitionedAt: STARTED_AT,
            },
          },
          {
            type: 'message',
            id: 'advisor',
            parentId: 'start',
            timestamp: COMPLETED_AT,
            message: {
              role: 'toolResult',
              toolCallId: 'advisor-call',
              toolName: 'advisor',
              content: [{ type: 'text', text: 'Use an elliptic-curve method.', },],
              isError: false,
              timestamp: 1,
            },
          },
          {
            type: 'message',
            id: 'points',
            parentId: 'advisor',
            timestamp: COMPLETED_AT,
            message: {
              role: 'toolResult',
              toolCallId: 'bash-call',
              toolName: 'bash',
              content: [{ type: 'text', text: 'points 0', },],
              isError: false,
              timestamp: 2,
            },
          },
        ];
        const evidence = buildGoalReviewEvidence({
          branch,
          request: evidenceRequest(),
        },);
        expect(evidence.objective,).toBe('Explain why 67 is prime in five ways',);
        expect(evidence.transcriptChunks.join('\n',),).toContain('points 0',);
        expect(evidence.transcriptChunks.join('\n',),).not.toContain(INCIDENT_ANSWER,);
      },
    },),
  ],
},);

await describe({
  name: truncateTranscript.name,
  children: [
    it({
      name: 'retains full transcript and newest bounded evidence',
      fn: async () => {
        expect(truncateTranscript({
          chunks: ['old', 'new',],
          maximumCharacters: 100,
        },),).toEqual({
          transcript: 'old\n\n---\n\nnew',
          truncated: false,
        },);
        /** Truncated transcript retaining newest evidence. */
        const truncated = truncateTranscript({
          chunks: ['old evidence '.repeat(20,), 'new evidence',],
          maximumCharacters: 80,
        },);
        expect(truncated.truncated,).toBe(true,);
        expect(truncated.transcript,).toContain('new evidence',);
        expect(truncated.transcript,).not.toContain('old evidence old evidence',);
      },
    },),
    it({
      name: 'keeps objective outside truncation and rejects undersized model',
      fn: async () => {
        /** Candidate-specific bounded prompt. */
        const prompt = buildBudgetedGoalReviewPrompt({
          evidence: {
            objective: 'Exact objective',
            transcriptChunks: ['x'.repeat(20_000,),],
          },
          contextWindow: 20_000,
          humanInputAvailable: true,
        },);
        expect(prompt.userContent,).toContain('Exact objective',);
        expect(prompt.userContent,).not.toContain('Completion summary',);
        expect(prompt.transcriptTruncated,).toBe(true,);
        expect(() => buildBudgetedGoalReviewPrompt({
          evidence: {
            objective: 'Objective',
            transcriptChunks: [],
          },
          contextWindow: 100,
          humanInputAvailable: true,
        },),).toThrow(ReviewerContextTooLargeError,);
      },
    },),
  ],
},);

await describe({
  name: parseGoalReviewVerdict.name,
  children: [
    it({
      name: 'accepts exact binary verdicts and rejects meta denial guidance',
      fn: async () => {
        expect(parseGoalReviewVerdict({
          approved: false,
          rationale: 'Integration test absent.',
          remaining_work: ' Add the integration test. ',
        },),).toEqual({
          approved: false,
          rationale: 'Integration test absent.',
          remainingWork: 'Add the integration test.',
        },);
        expect(parseGoalReviewVerdict({
          approved: true,
          rationale: 'Every requirement is supported.',
          remaining_work: '',
        },).approved,).toBe(true,);
        for (const leakedGuidance of [
          'The reviewer could not verify integration tests.',
          'A reviewer needs another result.',
          'An independent review found missing verification.',
          'Goal mode requires finalized output.',
          'The stop hook needs a tool result.',
          'The stop-hook needs a tool result.',
        ]) {
          expect(remainingWorkDescribesReview(leakedGuidance,),).toBe(true,);
        }
        expect(() => parseGoalReviewVerdict({
          approved: false,
          rationale: 'Missing work.',
          remaining_work: 'This review needs more evidence.',
        },),).toThrow();
        expect(() => parseGoalReviewVerdict({
          approved: true,
          rationale: 'Done.',
          remaining_work: 'Do more.',
        },),).toThrow();
      },
    },),
  ],
},);

await describe({
  name: buildBudgetedGoalReviewPrompt.name,
  children: [
    it({
      name: 'offers user questions only to sessions that can reach the user',
      fn: async () => {
        /** Reviewer prompt for an interactive session. */
        const interactive = reviewerPromptFixture(true,);
        /** Reviewer prompt for a session that cannot ask the user. */
        const nonInteractive = reviewerPromptFixture(false,);
        expect(interactive.systemPrompt,).toContain(INTERACTIVE_HUMAN_INPUT_CLAUSE,);
        expect(nonInteractive.systemPrompt,).not.toContain(INTERACTIVE_HUMAN_INPUT_CLAUSE,);
        expect(nonInteractive.systemPrompt,).toContain(NON_INTERACTIVE_HUMAN_INPUT_CLAUSE,);
      },
    },),
    it({
      name: 'demands one concrete next attempt in both session capabilities',
      fn: async () => {
        expect(reviewerPromptFixture(true,).systemPrompt,).toContain(CONCRETE_NEXT_ATTEMPT_CLAUSE,);
        expect(reviewerPromptFixture(false,).systemPrompt,).toContain(CONCRETE_NEXT_ATTEMPT_CLAUSE,);
      },
    },),
    it({
      name: 'derives the token estimate from the prompt the session receives',
      fn: async () => {
        /** Reviewer prompt for an interactive session. */
        const interactive = reviewerPromptFixture(true,);
        /** Reviewer prompt for a session that cannot ask the user. */
        const nonInteractive = reviewerPromptFixture(false,);
        /** Serialized request characters behind each token estimate. */
        const interactiveCharacters = interactive.systemPrompt.length
          + interactive.userContent.length;
        const nonInteractiveCharacters = nonInteractive.systemPrompt.length
          + nonInteractive.userContent.length;
        expect(interactive.transcriptTruncated,).toBe(false,);
        expect(nonInteractive.transcriptTruncated,).toBe(false,);
        expect(nonInteractive.userContent,).toBe(interactive.userContent,);
        expect(interactive.estimatedInputTokens,).toBe(
          Math.ceil(interactiveCharacters / ESTIMATED_CHARACTERS_PER_TOKEN,),
        );
        expect(nonInteractive.estimatedInputTokens,).toBe(
          Math.ceil(nonInteractiveCharacters / ESTIMATED_CHARACTERS_PER_TOKEN,),
        );
        expect(nonInteractiveCharacters,).toBeGreaterThan(interactiveCharacters,);
        expect(nonInteractive.estimatedInputTokens,).toBeGreaterThan(interactive.estimatedInputTokens,);
      },
    },),
  ],
},);
