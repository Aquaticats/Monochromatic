/**
 Maps Luna's reading of a final message to a stop decision.

 The table follows rules this repository already states rather than a fresh
 judgment: an announced next step, or open work the agent could do itself, is
 done now, a wait on the user goes through
 `AskUserQuestion`, and a wait on background work is refused only when no
 background task is running, because then nothing can resume the turn. A
 running task means Claude Code will wake the session itself, and whether it
 does so reliably is measured by the verdict log before this table widens.

 Pure by construction; the handler supplies every input.

 @module
 */

import type { BackgroundTask, } from '@monochromatic-dev/claude-code-plugin-hook-type/ts';

import { autoContinueReason, } from './auto-continue.ts';
import { hasRunningBackgroundTask, } from './continuation-progress.ts';
import type {
  Situation,
  SituationVerdict,
} from './luna-situation.ts';

/**
 Probability the chosen label must reach before the hook acts on it.

 Interim: no labeled ground truth exists for the 5.5 series because the
 restart nudges that served as labels are 8 of 439 stops and mostly pauses, so
 the value is revisited from the verdict log.
 */
const ACT_PROBABILITY = 0.6;

/**
 Outcome of reading Luna's verdict against hook state.
 */
type SituationAction =
  | {
    readonly kind: 'release';
    readonly why: string;
  }
    | {
      readonly kind: 'block';
      readonly why: string;
      readonly reason: readonly string[];
    };

/**
 Opening clause of every block reason from this table.

 It is the marker the depth and progress scans count, so every reason starts
 with it and the depth limit keeps bounding the chain.
 */
const LEAD = autoContinueReason()[0] ?? '';

/**
 Block reason for an agent that announced its own next step.
 */
const ANNOUNCED_REASON: readonly string[] = autoContinueReason();

/**
 Block reason for an agent that lists open work it could do itself and stops.
 */
const OPEN_WORK_REASON: readonly string[] = [
  LEAD,
  'Your final message lists open items, failures, or follow-ups that you can act on yourself.',
  'Do them now: file the issue, fix the failing test, or run the audit.',
  'Ask with the AskUserQuestion tool only for a choice that is the user\'s to make,',
  'and name what is left only after it is done.',
];

/**
 Block reason for an agent that asks the user for input in prose.
 */
const WAITS_ON_USER_REASON: readonly string[] = [
  LEAD,
  'Your final message asks the user for a decision, an authorization, or other input in prose.',
  'Ask it with the AskUserQuestion tool instead of ending your turn.',
  'That tool waits for their answer, which is what you actually need;',
  'a stopped turn only ends your work and waits to be restarted.',
];

/**
 Block reason for an agent that claims a background wait while none is running.
 */
const NOTHING_RUNNING_REASON: readonly string[] = [
  LEAD,
  'Your final message says you are waiting for background work, but no background task is running,',
  'so nothing will resume this turn.',
  'Perform the pending work now, or start the wait with a single command that ends at the result',
  'and keep the turn open until it returns.',
];

/**
 Action per situation, as a lookup so a new label is a type error until it is handled here.
 */
const ACTIONS: Readonly<Record<Situation, (state: { readonly backgroundTasks: readonly BackgroundTask[]; },) => SituationAction>> = {
  announces_own_next_step: function announces(): SituationAction {
    return {
      kind: 'block',
      why: 'announced next step',
      reason: ANNOUNCED_REASON,
    };
  },
  leaves_open_work: function leavesOpenWork(): SituationAction {
    return {
      kind: 'block',
      why: 'lists open work it could do itself',
      reason: OPEN_WORK_REASON,
    };
  },
  waits_on_user: function waitsOnUser(): SituationAction {
    return {
      kind: 'block',
      why: 'waits on user in prose',
      reason: WAITS_ON_USER_REASON,
    };
  },
  waits_on_background_work: function waitsOnBackground(
    { backgroundTasks, },
  ): SituationAction {
    return hasRunningBackgroundTask(backgroundTasks,)
      ? {
        kind: 'release',
        why: 'waits on background work and a background task is running',
      }
      : {
        kind: 'block',
        why: 'waits on background work and no background task is running',
        reason: NOTHING_RUNNING_REASON,
      };
  },
  pauses_for_handoff: function pauses(): SituationAction {
    return {
      kind: 'release',
      why: 'deliberate pause or handoff',
    };
  },
  work_closed: function closed(): SituationAction {
    return {
      kind: 'release',
      why: 'work closed',
    };
  },
};

/**
 Labels that block whatever the background state, in the order a tie is broken.
 */
const BLOCKING_LABELS = [
  'announces_own_next_step',
  'leaves_open_work',
  'waits_on_user',
] as const satisfies readonly Situation[];

/**
 One of the labels that block whatever the background state.
 */
type BlockingLabel = (typeof BLOCKING_LABELS)[number];

/**
 Block reason per always-blocking label, for a verdict split between them.
 */
const BLOCKING_REASONS: Readonly<Record<BlockingLabel, readonly string[]>> = {
  announces_own_next_step: ANNOUNCED_REASON,
  leaves_open_work: OPEN_WORK_REASON,
  waits_on_user: WAITS_ON_USER_REASON,
};

/**
 Combined probability of the always-blocking labels at which a split verdict blocks.

 Chosen by the owner on 2026-10-29. A stop scored `waits_on_user` 0.54 and `leaves_open_work` 0.37: neither label
 reached the cutoff, yet 0.95 of the mass said block. In the 832 logged released stops that have all six scores,
 a level of 0.9 fired never and 0.8 fired three times, so 0.9 adds no measured block.
 */
const SPLIT_BLOCK_PROBABILITY = 0.9;

/**
 Whole-percent value of a probability, so a sum such as 0.8999999999999999 compares as the 0.9 it stands for.

 @param probability - probability between zero and one

 @returns the probability as a whole percent

 @example
 ```ts
 wholePercent(0.3 + 0.3 + 0.3); // 90
 ```
 */
function wholePercent(probability: number,): number {
  return Math.round(probability * 100,);
}

/**
 Finds the always-blocking label with the largest probability.

 @param probabilities - Luna's probability for every label

 @returns the largest of the always-blocking labels, the first of them on a tie

 @example
 ```ts
 largestBlockingLabel({ ...scores, waits_on_user: 0.54, leaves_open_work: 0.37 }); // 'waits_on_user'
 ```
 */
function largestBlockingLabel(probabilities: SituationVerdict['probabilities'],): BlockingLabel {
  return BLOCKING_LABELS.reduce(function larger(
    best,
    label,
  ) {
    return (probabilities[label] > probabilities[best]) ? label : best;
  },);
}

/**
 Chooses the action for one verdict.

 A verdict whose top label is under the cutoff releases, unless the always-blocking labels together reach
 {@link SPLIT_BLOCK_PROBABILITY}: Luna is then sure the stop is premature and only unsure which kind, so the stop
 blocks with the reason of the largest of those labels.

 @param verdict - validated Luna reading

 @param backgroundTasks - `background_tasks` from the `Stop` event

 @returns action with a reason that names why, for the verdict log

 @example
 ```ts
 situationAction({ verdict, backgroundTasks: [] }).kind; // 'block'
 ```
 */
function situationAction(
  {
    verdict,
    backgroundTasks,
  }: {
    readonly verdict: SituationVerdict;
    readonly backgroundTasks: readonly BackgroundTask[];
  },
): SituationAction {
  if (verdict.probability < ACT_PROBABILITY) {
    /**
     Combined probability of the labels that block whatever the background state.
     */
    const blockingMass = BLOCKING_LABELS.reduce(
      function sum(
        running,
        label,
      ) {
        return running + verdict.probabilities[label];
      },
      0,
    );
    if (wholePercent(blockingMass,) >= wholePercent(SPLIT_BLOCK_PROBABILITY,)) {
      /**
       Label whose reason the split verdict borrows.
       */
      const largest = largestBlockingLabel(verdict.probabilities,);
      return {
        kind: 'block',
        why: `blocking labels together ${blockingMass.toFixed(2,)} with none at ${String(ACT_PROBABILITY,)}; largest ${largest}`,
        reason: BLOCKING_REASONS[largest],
      };
    }
    return {
      kind: 'release',
      why: `probability ${String(verdict.probability,)} for ${verdict.situation} is below ${String(ACT_PROBABILITY,)}`,
    };
  }
  return ACTIONS[verdict.situation]({ backgroundTasks, });
}

export type { SituationAction, };

export {
  ACT_PROBABILITY,
  SPLIT_BLOCK_PROBABILITY,
  situationAction,
};
