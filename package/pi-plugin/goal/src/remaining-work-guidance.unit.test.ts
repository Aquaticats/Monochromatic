/**
 Built-artifact tests for denial guidance that keeps goal work going.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildBudgetedGoalReviewPrompt,
  buildGoalReviewSystemPrompt,
  canReachUser,
  PRODUCTIVE_REMAINING_WORK,
  remainingWorkViolations,
  requiredProductiveRemainingWork,
} from '../dist/final/node/index.mjs';

/**
 Guidance produced by the measured Collatz runs.
 */
const DEAD_END_GUIDANCE = 'Explain that a proof is currently unknown and ask the user which alternative task they want.';

/**
 Guidance needing one detail only the user holds.
 */
const USER_DETAIL_GUIDANCE = 'Ask the user which of the two configuration files owns the migration table.';

/**
 Private review wording that must never authorize asking an absent user.
 */
const REACHABLE_ONLY_SENTENCE = 'must directly instruct the primary model to use ask_user_question';

await describe({
  name: canReachUser.name,
  children: [
    it({
      name: 'reaches a person only in the terminal session',
      fn: async () => {
        expect(canReachUser('tui',),).toBe(true,);
        for (const mode of ['rpc', 'json', 'print',] as const)
          expect(canReachUser(mode,),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: remainingWorkViolations.name,
  children: [
    it({
      name: 'reports dead ends in stable order regardless of wording order',
      fn: async () => {
        expect(remainingWorkViolations({
          remainingWork: DEAD_END_GUIDANCE,
          userReachable: false,
        },),).toEqual([
          'asks_user',
          'declares_impossible',
          'hands_back_objective',
        ],);
        expect(remainingWorkViolations({
          remainingWork: 'Stop working: the objective cannot be proven.',
          userReachable: true,
        },),).toEqual([
          'declares_impossible',
          'stops_work',
        ],);
      },
    },),
    it({
      name: 'accepts user questions only when a person can answer',
      fn: async () => {
        expect(remainingWorkViolations({
          remainingWork: USER_DETAIL_GUIDANCE,
          userReachable: true,
        },),).toEqual([],);
        expect(remainingWorkViolations({
          remainingWork: USER_DETAIL_GUIDANCE,
          userReachable: false,
        },),).toEqual(['asks_user',],);
      },
    },),
    it({
      name: 'accepts concrete work instructions',
      fn: async () => {
        expect(remainingWorkViolations({
          remainingWork: 'Check every starting value below 1000 and record the longest chain.',
          userReachable: false,
        },),).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: requiredProductiveRemainingWork.name,
  children: [
    it({
      name: 'keeps concrete work instructions unchanged',
      fn: async () => {
        /** Concrete guidance shared by both session modes. */
        const concrete = 'Check every starting value below 1000 and record the longest chain.';
        for (const userReachable of [true, false,]) {
          expect(requiredProductiveRemainingWork({
            remainingWork: concrete,
            userReachable,
          },),).toEqual({
            remainingWork: concrete,
            violations: [],
          },);
        }
      },
    },),
    it({
      name: 'replaces stopped, impossible, and handed-back work',
      fn: async () => {
        for (const userReachable of [true, false,]) {
          /** Replaced guidance and recorded defects. */
          const guidance = requiredProductiveRemainingWork({
            remainingWork: DEAD_END_GUIDANCE,
            userReachable,
          },);
          expect(guidance.remainingWork,).toBe(PRODUCTIVE_REMAINING_WORK,);
          expect(guidance.violations,).toContain('declares_impossible',);
          expect(guidance.violations,).toContain('hands_back_objective',);
        }
        expect(requiredProductiveRemainingWork({
          remainingWork: DEAD_END_GUIDANCE,
          userReachable: false,
        },).violations,).toContain('asks_user',);
        expect(requiredProductiveRemainingWork({
          remainingWork: DEAD_END_GUIDANCE,
          userReachable: true,
        },).violations,).not.toContain('asks_user',);
      },
    },),
    it({
      name: 'replaces a user question this session cannot deliver',
      fn: async () => {
        expect(requiredProductiveRemainingWork({
          remainingWork: USER_DETAIL_GUIDANCE,
          userReachable: false,
        },),).toEqual({
          remainingWork: PRODUCTIVE_REMAINING_WORK,
          violations: ['asks_user',],
        },);
        expect(requiredProductiveRemainingWork({
          remainingWork: USER_DETAIL_GUIDANCE,
          userReachable: true,
        },).remainingWork,).toBe(USER_DETAIL_GUIDANCE,);
      },
    },),
    it({
      name: 'replacement guidance satisfies the policy it enforces',
      fn: async () => {
        for (const userReachable of [true, false,]) {
          expect(remainingWorkViolations({
            remainingWork: PRODUCTIVE_REMAINING_WORK,
            userReachable,
          },),).toEqual([],);
        }
      },
    },),
  ],
},);

await describe({
  name: buildGoalReviewSystemPrompt.name,
  children: [
    it({
      name: 'demands a differing concrete attempt in every mode',
      fn: async () => {
        for (const userReachable of [true, false,]) {
          /** Mode-specific reviewer rubric. */
          const systemPrompt = buildGoalReviewSystemPrompt({ userReachable, },);
          expect(systemPrompt,).toContain('one concrete next attempt that differs from the attempts visible in the evidence',);
          expect(systemPrompt,).toContain('is never a reason to stop',);
          expect(systemPrompt,).toContain('must never tell the primary model to stop working',);
          expect(systemPrompt,).not.toContain(REACHABLE_ONLY_SENTENCE,);
        }
      },
    },),
    it({
      name: 'allows a user question only when a person can answer',
      fn: async () => {
        expect(buildGoalReviewSystemPrompt({ userReachable: true, },),).toContain('use ask_user_question once',);
        expect(buildGoalReviewSystemPrompt({ userReachable: false, },),).toContain('No person can answer in this session',);
        expect(buildGoalReviewSystemPrompt({ userReachable: false, },),).not.toContain('use ask_user_question once',);
      },
    },),
  ],
},);

await describe({
  name: buildBudgetedGoalReviewPrompt.name,
  children: [
    it({
      name: 'uses the mode-specific rubric without offering an absent user',
      fn: async () => {
        /** Reviewer request built for a session without a person. */
        const prompt = buildBudgetedGoalReviewPrompt({
          evidence: {
            objective: 'Prove the Collatz conjecture',
            transcriptChunks: ['Assistant:\nNo proof yet.',],
          },
          contextWindow: 128_000,
          userReachable: false,
        },);
        expect(prompt.systemPrompt,).toBe(buildGoalReviewSystemPrompt({ userReachable: false, },),);
        expect(prompt.systemPrompt,).not.toContain(REACHABLE_ONLY_SENTENCE,);
        expect(prompt.userContent,).toContain('Prove the Collatz conjecture',);
      },
    },),
  ],
},);
