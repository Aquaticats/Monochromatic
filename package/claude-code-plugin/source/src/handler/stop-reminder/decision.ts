/**
 The pure stop decision: which reminders a final message raises.

 Split from `index.ts`, which owns the handler's ambient inputs, so the policy reads on its own and
 every branch is reachable in a test without touching process state.

 @module
 */

import type {
  StopInput,
  StopOutput,
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import type { ReadonlyDeep, } from 'type-fest';

import { autoContinueReason, } from './auto-continue.ts';
import {
  findParkedDecision,
  parkedDecisionLead,
  parkedDecisionReminder,
} from './parked-decision.ts';
import { alreadyRaised, } from './reminder-history.ts';
import {
  dismissalLead,
  questionLead,
  uncertaintyLead,
} from './reminder-lead.ts';
import type { SituationAction, } from './situation-action.ts';
import {
  findCategoricalDismissal,
  findTrailingQuestion,
  findUncertainty,
  NO_MATCH,
  stripNonProseRegions,
} from './uncertainty.ts';

/**
 Output returned by the stop-reminders handler.

 Either a typed {@link StopOutput} carrying a block decision and concatenated
 reasons, or the empty pass-through `{}`. Every {@link StopOutput} field is
 optional, so `{}` is itself a valid {@link StopOutput}; no separate empty type
 is needed.
 */
type StopRemindersOutput = StopOutput;

/**
 Detects uncertain language and trailing user-directed questions in Claude's
 final response, returning a `block` decision when either is present so Claude
 is reminded to investigate or to use `AskUserQuestion` instead.

 Decision tree:

 1. **Once-per-trigger gate**: the hedging, dismissal, trailing-question, and
    parked-decision detectors always run. On a first stop every trigger they
    find raises a reminder. Inside a blocked chain (`stop_hook_active` is
    `true`) a trigger raises its reminder only if this hook has not already
    raised it since the last human turn, so the agent can answer a block and
    finish. Switching these detectors off in a chain let a stop that asked
    "Do you want me to?" after one block go through (2026-10-29).
 2. **Prose extraction**: {@link stripNonProseRegions} strips code blocks,
    inline code, blockquotes, and quoted strings from `last_assistant_message`
    before scanning.
 3. **Uncertainty scan**: {@link findUncertainty} matches against hedging
    phrases (probably, maybe, I think, etc.); a hit contributes a reminder
    to gather evidence.
 4. **Trailing-question scan**: {@link findTrailingQuestion} looks for
    sentences ending in `?` in the last 500 characters; rhetorical/conditional
    prefixes are excluded.
 5. **Forced continuation**: unless a trailing question matched, and unless
    `forcedContinuationAllowed` is false, {@link autoContinueReason} is
    appended on every stop including stops inside a blocked chain. This is the
    one detector that re-arms, and it reads none of the response text.
    Claude Code does not reliably end the resulting chain, so the caller is
    responsible for bounding it; see `continuation-depth.ts`.
 6. **Result**: if any reasons accumulated, returns
    `\{ decision: 'block', reason: [reasons joined by space] \}`;
    otherwise `\{\}`.

 Pure by construction: every ambient input arrives as a parameter, so each
 branch is reachable in a test without touching process state. An earlier
 revision read the opt-in switch inside this function, which made the disabled
 branch testable only by mutating `process.env` and left those tests racing
 once the handler became asynchronous.

 @param event - parsed {@link StopInput} event from Claude Code

 @param forcedContinuationAllowed - whether to append the continuation reason

 @returns blocking output when reminders apply, otherwise `{}`

 @example
 ```ts
 stopRemindersDecision({ event, forcedContinuationAllowed: true });
 ```
 */
function stopRemindersDecision(
  {
    event,
    forcedContinuationAllowed,
    situation,
    notice,
    transcriptLines = [],
  }: {
    readonly event: ReadonlyDeep<StopInput>;
    readonly forcedContinuationAllowed: boolean;
    readonly situation?: SituationAction;
    readonly notice?: string;
    readonly transcriptLines?: readonly string[];
  },
): StopRemindersOutput {
  /**
   Final assistant message with code blocks, inline code, and quotes stripped before scanning.
   */
  const prose = stripNonProseRegions(event.last_assistant_message
    ?? '',);

  /**
   Whether the agent was already told about the trigger whose reminder opens with `lead`.

   Only a stop inside a blocked chain consults the transcript; a first stop always raises its triggers.
   Each trigger therefore blocks at most once per turn inside a chain, so a block can be answered.

   @param lead - opening sentence of the reminder about to be raised

   @returns whether this turn already carries that reminder
   */
  function toldBefore(lead: string,): boolean {
    return event.stop_hook_active
      && alreadyRaised({
        lead,
        transcriptLines,
      },);
  }

  /**
   First hedging-phrase hit, or `NO_MATCH`.
   */
  const hedge = findUncertainty(prose,);
  /**
   Hedging hit to remind about, or `NO_MATCH` when absent or already raised in this turn.
   */
  const match = ((hedge !== NO_MATCH) && toldBefore(uncertaintyLead(hedge.phrase,),))
    ? NO_MATCH
    : hedge;
  /**
   First uncited categorical-dismissal hit, or `NO_MATCH`.
   */
  const uncited = findCategoricalDismissal(prose,);
  /**
   Dismissal hit to remind about, or `NO_MATCH` when absent or already raised in this turn.
   */
  const dismissal = ((uncited !== NO_MATCH) && toldBefore(dismissalLead(uncited.phrase,),))
    ? NO_MATCH
    : uncited;
  /**
   Trailing user-directed question hit, or `NO_MATCH`.
   */
  const asked = findTrailingQuestion(prose,);
  /**
   Question hit to remind about, or `NO_MATCH` when absent or already raised in this turn.
   */
  const question = ((asked !== NO_MATCH) && toldBefore(questionLead(asked.sentence,),))
    ? NO_MATCH
    : asked;
  /**
   First phrase that leaves a choice with the user in prose, or `NO_MATCH`.
   */
  const offered = findParkedDecision(prose,);
  /**
   Parked-decision hit to remind about, or `NO_MATCH` when absent or already raised in this turn.
   */
  const parked = ((offered !== NO_MATCH) && toldBefore(parkedDecisionLead(offered.phrase,),))
    ? NO_MATCH
    : offered;

  /**
   Reminder lines accumulated across the detectors; joined into the final block reason.
   */
  const reasons: string[] = [];

  if (match !== NO_MATCH) {
    reasons.push(
      uncertaintyLead(match.phrase,),
      'Search for evidence, read the relevant code, or check documentation.',
      'Always research thoroughly before responding.',
      'If you have already investigated and the uncertainty is genuinely warranted,',
      'say so explicitly and continue with your response.',
      'This may be a false positive; use your judgement.',
    );
  }

  if (dismissal !== NO_MATCH) {
    reasons.push(
      dismissalLead(dismissal.phrase,),
      'Categorical dismissals are one rg/find/config-read/AGENTS.md-grep away from being verified.',
      'Cite the search result inline on the same line as the dismissal:',
      'a file path with an extension, a path:line reference, or the literal "AGENTS.md".',
      'If the dismissal was wrong, fold the now-relevant option back into the analysis.',
      'This may be a false positive; use your judgement.',
    );
  }

  if (question !== NO_MATCH) {
    reasons.push(
      questionLead(question.sentence,),
      'Use the AskUserQuestion tool to ask the user instead of ending your response with a question.',
      'The AskUserQuestion tool ensures the user sees and can respond to your question directly.',
      'Rephrase your question as an AskUserQuestion tool call and continue.',
    );
  }

  // A trailing question already routes the agent to the question tool, so a second reminder would repeat it.
  if ((parked !== NO_MATCH) && (question === NO_MATCH)) {
    reasons.push(...parkedDecisionReminder(parked.phrase,),);
  }

  // A trailing question takes precedence over forced continuation: instructing the
  // agent to resume work and to route its question through AskUserQuestion in the
  // same reason would be contradictory, and the question reason already refuses the stop.
  if (question === NO_MATCH) {
    // A situation verdict replaces the state-only reason: Luna read the message, so its
    // block reason says what was detected, and its release overrides the state default.
    if (situation !== undefined) {
      if (situation.kind === 'block') {
        reasons.push(...situation.reason,);
      }
    }
    else if (forcedContinuationAllowed) {
      reasons.push(...autoContinueReason(),);
    }
  }

  /**
   Notice for the user, present only when the judge could not answer.
   */
  const systemMessage = (notice === undefined)
    ? {}
    : { systemMessage: notice, };

  if (reasons.length
    > 0) {
    return {
      decision: 'block',
      reason: reasons.join(' ',),
      ...systemMessage,
    };
  }

  return { ...systemMessage, };
}

export type { StopRemindersOutput, };

export { stopRemindersDecision, };
