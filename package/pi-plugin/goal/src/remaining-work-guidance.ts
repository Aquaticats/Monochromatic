/**
 Denial guidance policy keeping the primary model at work and the user reachable only when real.
 
 A truthful statement that the objective is hard,
 unsolved,
 or unfinished never ends work.
 Denial guidance is replaced when it would stop work,
 claim the objective impossible,
 hand the objective back,
 or ask a user that this session cannot reach.
 
 @module
 */

import type { ExtensionContext, } from '@earendil-works/pi-coding-agent';

/**
 Denial guidance defect that would end goal work before the objective is met.
 */
type RemainingWorkViolation =
  | 'asks_user'
    | 'declares_impossible'
    | 'hands_back_objective'
    | 'stops_work';

/**
 Assessed denial guidance together with every policy defect it carried.
 */
type ProductiveRemainingWork = {
  /**
   Guidance safe to hand to the primary model.
   */
  readonly remainingWork: string;
  /**
   Defects found in requested guidance before replacement.
   */
  readonly violations: readonly RemainingWorkViolation[];
};

/**
 Guidance sent when requested denial guidance would halt work.
 */
const PRODUCTIVE_REMAINING_WORK = 'Continue the current objective without stopping. Choose one concrete next step that the recorded attempts have not tried: change the approach, obtain the missing evidence, or reduce the objective to a smaller case that can be checked, then report its exact result. Report what the step produced instead of stopping to describe the difficulty, and do not switch to some other objective.';

/**
 Phrases instructing the primary model to end work instead of attempting more.
 */
const STOP_WORK_PHRASES = [
  'stop working',
  'stop the work',
  'stop here',
  'do not continue',
  'don\'t continue',
  'no further work',
  'nothing further',
  'wait for the user',
  'wait for a user',
  'wait for their answer',
] as const;

/**
 Phrases presenting difficulty, openness, or absence of a known result as a dead end.
 */
const IMPOSSIBILITY_PHRASES = [
  'is impossible',
  'impossible to prove',
  'impossible to solve',
  'cannot be proven',
  'cannot be proved',
  'can\'t be proven',
  'can\'t be proved',
  'cannot be solved',
  'can\'t be solved',
  'is unsolvable',
  'is unsolved',
  'remains unsolved',
  'is not solvable',
  'open problem',
  'open question',
  'no proof is known',
  'no known proof',
  'no proof exists',
  'currently unknown',
  'not currently known',
] as const;

/**
 Phrases asking for a different objective instead of continuing this one.
 */
const HAND_BACK_PHRASES = [
  'ask for a different objective',
  'ask for another objective',
  'request a different objective',
  'request a different task',
  'request a new objective',
  'choose an alternative task',
  'choose a different task',
  'alternative task',
  'different task',
  'suggest another task',
  'pick another task',
  'hand the objective back',
  'hand back the objective',
] as const;

/**
 Phrases asking a human for input that already stops or narrows the objective.
 */
const ASK_USER_PHRASES = [
  'ask_user_question',
  'ask the user',
  'ask your user',
  'ask the human',
  'ask the operator',
  'ask a question',
  'ask them',
  'ask him',
  'ask her',
  'request clarification',
  'clarify with the user',
  'check with the user',
  'confirm with the user',
  'consult the user',
  'let the user decide',
  'user to choose',
  'which alternative',
] as const;

/**
 Report whether human decisions can reach someone in this session mode.
 
 Only the terminal session can answer a question; every other mode runs without a person present.
 
 @param mode - Pi extension session mode
 
 @returns whether asking the user can succeed
 
 @example
 ```ts
 canReachUser('rpc');
 ```
 */
function canReachUser(mode: ExtensionContext['mode'],): boolean {
  return mode === 'tui';
}

/**
 Detect whether case-folded guidance contains any banned phrase.
 
 @param phrases - lower-case phrases whose presence is defective
 
 @param normalized - case-folded denial guidance
 
 @returns whether one phrase is present
 
 @example
 ```ts
 containsAnyPhrase({ phrases: ['stop working'], normalized: 'stop working now' });
 ```
 */
function containsAnyPhrase(
  {
    phrases,
    normalized,
  }: {
    readonly phrases: readonly string[];
    readonly normalized: string;
  },
): boolean {
  return phrases.some(function includesPhrase(phrase,) {
    return normalized.includes(phrase,);
  },);
}

/**
 Find every policy defect in requested denial guidance.
 
 Asking the user is defective only when no human can answer;
 the other three defects end work in every session mode.
 
 @param remainingWork - normalized denial guidance
 
 @param userReachable - whether this session can reach a human
 
 @returns defective categories in stable order
 
 @example
 ```ts
 remainingWorkViolations({ remainingWork: 'Stop working.', userReachable: true });
 ```
 */
function remainingWorkViolations(
  {
    remainingWork,
    userReachable,
  }: {
    readonly remainingWork: string;
    readonly userReachable: boolean;
  },
): readonly RemainingWorkViolation[] {
  /**
   Case-folded denial guidance scanned by bounded phrase lists.
   */
  const normalized = remainingWork.toLocaleLowerCase('en-US',);
  /**
   Detected defects in stable declaration order.
   */
  const violations: RemainingWorkViolation[] = [];
  if ((!userReachable)
    && containsAnyPhrase({
      phrases: ASK_USER_PHRASES,
      normalized,
    },)) {
    violations.push('asks_user',);
  }
  if (containsAnyPhrase({
    phrases: IMPOSSIBILITY_PHRASES,
    normalized,
  },))
    violations.push('declares_impossible',);
  if (containsAnyPhrase({
    phrases: HAND_BACK_PHRASES,
    normalized,
  },))
    violations.push('hands_back_objective',);
  if (containsAnyPhrase({
    phrases: STOP_WORK_PHRASES,
    normalized,
  },))
    violations.push('stops_work',);
  return violations;
}

/**
 Keep requested denial guidance or replace it with a concrete attempt instruction.
 
 @param remainingWork - normalized denial guidance from a model reviewer
 
 @param userReachable - whether this session can reach a human
 
 @returns guidance for the primary model plus every replaced defect
 
 @example
 ```ts
 requiredProductiveRemainingWork({ remainingWork: 'Ask the user for a different task.', userReachable: true });
 ```
 */
function requiredProductiveRemainingWork(
  {
    remainingWork,
    userReachable,
  }: {
    readonly remainingWork: string;
    readonly userReachable: boolean;
  },
): ProductiveRemainingWork {
  /**
   Policy defects in requested guidance.
   */
  const violations = remainingWorkViolations({
    remainingWork,
    userReachable,
  },);
  if (violations.length === 0) {
    return {
      remainingWork,
      violations,
    };
  }
  return {
    remainingWork: PRODUCTIVE_REMAINING_WORK,
    violations,
  };
}

export {
  canReachUser,
  PRODUCTIVE_REMAINING_WORK,
  remainingWorkViolations,
  requiredProductiveRemainingWork,
};
export type {
  ProductiveRemainingWork,
  RemainingWorkViolation,
};
