import type {
  StopInput,
  StopOutput,
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import { tagged, } from '@monochromatic-dev/module-logger/ts';
import type { ReadonlyDeep, } from 'type-fest';

import {
  AUTO_CONTINUE_ENV,
  autoContinueEnabled,
  autoContinueReason,
  UNSET_SETTING,
} from './auto-continue.ts';
import {
  continuationDepth,
  MAX_DEPTH_ENV,
  maxContinuationDepth,
  readTranscriptTail,
} from './continuation-depth.ts';
import {
  hasRunningBackgroundTask,
  hasRunningSubagent,
  workedSinceLastForcedContinuation,
} from './continuation-progress.ts';
import { taskListState, } from './continuation-tasks.ts';
import { readLunaKey, } from './luna-key.ts';
import {
  askLunaSituation,
  LUNA_SWITCH_ENV,
  lunaEnabled,
} from './luna-situation.ts';
import {
  alreadyRaised,
  findParkedDecision,
  parkedDecisionReminder,
} from './parked-decision.ts';
import { hasQueuedTaskNotification, } from './queued-notification.ts';
import {
  type SituationAction,
  situationAction,
} from './situation-action.ts';
import {
  appendVerdict,
  verdictLogPath,
} from './verdict-log.ts';
import {
  findCategoricalDismissal,
  findTrailingQuestion,
  findUncertainty,
  NO_MATCH,
  stripNonProseRegions,
} from './uncertainty.ts';

/**
 Module logger for the stop-reminders handler.
 */
const logger = tagged({ tag: 'stop-reminders', },);

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
 
 1. **Response-quality gate**: the hedging, dismissal, and trailing-question
    detectors run only when `stop_hook_active` is `false`. They ask whether
    this response is defective, so re-running them inside a blocked chain
    would re-block on text the agent was already told about. Claude Code sets
    `stop_hook_active` on every stop of a blocked chain and never clears it.
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
   Whether the response-quality detectors apply to this stop.
   
   False for every stop after the first in a blocked chain, so a forced
   continuation is never re-blocked for wording the agent already answered for.
   */
  const responseQualityApplies = !event.stop_hook_active;

  /**
   Final assistant message with code blocks, inline code, and quotes stripped before scanning.
   */
  const prose = stripNonProseRegions(event.last_assistant_message
    ?? '',);
  /**
   First hedging-phrase hit, or `NO_MATCH`; populates the uncertainty reminder when matched.
   */
  const match = responseQualityApplies
    ? findUncertainty(prose,)
    : NO_MATCH;
  /**
   First uncited categorical-dismissal hit, or `NO_MATCH`; populates the dismissal reminder when matched.
   */
  const dismissal = responseQualityApplies
    ? findCategoricalDismissal(prose,)
    : NO_MATCH;
  /**
   Trailing user-directed question hit, or `NO_MATCH`; populates the AskUserQuestion reminder when matched.
   */
  const question = responseQualityApplies
    ? findTrailingQuestion(prose,)
    : NO_MATCH;

  /**
   Phrase that leaves a choice with the user in prose, or `NO_MATCH`.

   Runs inside a blocked chain too, where the other wording detectors are off, but only
   for a phrase the agent has not been told about in this turn, so a block can be answered.
   */
  const parked = findParkedDecision(prose,);
  /**
   Whether the parked-decision reminder applies to this stop.
   */
  const parkedApplies = (parked !== NO_MATCH)
    && (responseQualityApplies
      || (!alreadyRaised({
        phrase: parked.phrase,
        transcriptLines,
      },)));

  /**
   Reminder lines accumulated across the detectors; joined into the final block reason.
   */
  const reasons: string[] = [];

  if (match !== NO_MATCH) {
    reasons.push(
      `Your response contains uncertain language ("${match.phrase}").`,
      'Search for evidence, read the relevant code, or check documentation.',
      'Always research thoroughly before responding.',
      'If you have already investigated and the uncertainty is genuinely warranted,',
      'say so explicitly and continue with your response.',
      'This may be a false positive; use your judgement.',
    );
  }

  if (dismissal !== NO_MATCH) {
    reasons.push(
      `Your response contains an uncited categorical dismissal ("${dismissal.phrase}").`,
      'Categorical dismissals are one rg/find/config-read/AGENTS.md-grep away from being verified.',
      'Cite the search result inline on the same line as the dismissal:',
      'a file path with an extension, a path:line reference, or the literal "AGENTS.md".',
      'If the dismissal was wrong, fold the now-relevant option back into the analysis.',
      'This may be a false positive; use your judgement.',
    );
  }

  if (question !== NO_MATCH) {
    reasons.push(
      `Your response ends with a question to the user ("${question.sentence}").`,
      'Use the AskUserQuestion tool to ask the user instead of ending your response with a question.',
      'The AskUserQuestion tool ensures the user sees and can respond to your question directly.',
      'Rephrase your question as an AskUserQuestion tool call and continue.',
    );
  }

  // A trailing question already routes the agent to the question tool, so a second reminder would repeat it.
  if (parkedApplies && (question === NO_MATCH)) {
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

/**
 Ambient inputs of the handler, injectable so tests never reach the network or the real environment.
 */
type StopRemindersDependencies = {
  /**
   Process environment; carries the opt-in switches and the credential.
   */
  readonly env: NodeJS.ProcessEnv;

  /**
   `fetch` used for the Luna request.
   */
  readonly fetchImpl: typeof fetch;

  /**
   Credential reader; the default asks the environment, then mise once.
   */
  readonly readKey: typeof readLunaKey;
};

/**
 Builds the stop-reminders handler around its ambient inputs.
 
 All environment and filesystem access lives here so the policy stays pure and
 testable. Depth is only computed when forced continuation is enabled, since
 the transcript read is pointless otherwise.
 
 When forced continuation applies, Luna reads the final message and the verdict
 picks the action (see `situation-action.ts`). State alone still ends the
 chain at the depth limit and when the last forced continuation issued no tool
 call, and a trailing question keeps its own reason, so none of those cases
 spends a request. Any Luna failure falls back to the state-based behavior this
 handler had before Luna and says so in a `systemMessage`.
 
 @param dependencies - ambient inputs; each defaults to the real one
 
 @returns handler for the `Stop` event
 
 @example
 ```ts
 const handler = createStopRemindersHandler({ fetchImpl: fakeFetch });
 ```
 */
function createStopRemindersHandler(
  dependencies: {
    readonly env?: StopRemindersDependencies['env'];
    readonly fetchImpl?: StopRemindersDependencies['fetchImpl'];
    readonly readKey?: StopRemindersDependencies['readKey'];
  } = {},
): (event: ReadonlyDeep<StopInput>,) => Promise<StopRemindersOutput> {
  /**
   Resolved ambient inputs.
   */
  const {
    env = process.env,
    fetchImpl = fetch,
    readKey = readLunaKey,
  } = dependencies;
  /**
   Factory logger so every handler line names this hook.
   */
  const rl = tagged({
    tag: createStopRemindersHandler.name,
    l: logger,
  },);

  return async function stopRemindersHandler(event: ReadonlyDeep<StopInput>,): Promise<StopRemindersOutput> {
    /**
     Handler-scoped logger.
     */
    const l = tagged({
      tag: stopRemindersHandler.name,
      l: rl,
    },);
    /**
     Whether the opt-in switch enables forced continuation.
     */
    const enabled = autoContinueEnabled(env[AUTO_CONTINUE_ENV] ?? UNSET_SETTING,);
    /**
     Transcript tail for depth, progress, and task replay.
     
     Claude Code does not reliably end a blocked chain, so the depth bound is the
     only termination guarantee; see `continuation-depth.ts` for the measurement.
     */
    const transcript = (enabled || event.stop_hook_active)
      ? await readTranscriptTail(event.transcript_path,)
      : {
        lines: [],
        truncated: false,
      };
    /**
     Forced continuations already issued for this human turn.
     */
    const depth = continuationDepth(transcript.lines,);
    /**
     Whether the previous forced continuation produced any tool call.
     
     Releases a session that is blocked on something outside the agent control,
     where pushing again yields another restatement rather than work.
     */
    const worked = workedSinceLastForcedContinuation(transcript.lines,);
    /**
     Whether a finished task's notification is queued and will resume the session.

     `background_tasks` omits a task that finished just before the stop, so an agent
     that is right about its wait can see an empty list. The transcript still holds
     the queued notification; see `queued-notification.ts`.
     */
    const notificationQueued = hasQueuedTaskNotification(transcript.lines,);
    /**
     Whether a background task is running or a notification for one is queued.

     The state-based fallback releases on this. With a Luna verdict the table
     decides on running tasks alone, since a wait the agent claims while nothing
     runs cannot resume; a queued notification is the one exception and is
     released before the request, below.
     */
    const waiting = hasRunningBackgroundTask(event.background_tasks,)
      || notificationQueued;
    /**
     Whether every tracked task is finished.
     
     A session with no task list is excluded, since most sessions never create
     one and releasing on an empty list would disable forced continuation
     everywhere. A truncated tail is excluded too: a task created before the
     window and never updated inside it is invisible, so an open list can look
     finished, and this release must not conclude anything from an absence it
     cannot see.
     */
    const nothingTracked = (!transcript.truncated)
      && (taskListState(transcript.lines,) === 'all-finished');
    /**
     Whether state alone allows another forced continuation.
     
     Depth and a lack of progress end the chain without asking anyone; they are the
     releases that read no text and never needed a judge.
     */
    const continuationOpen = enabled
      && (depth < maxContinuationDepth(env[MAX_DEPTH_ENV] ?? '',))
      && worked;
    /**
     The state-based answer used when Luna is off or unavailable.
     */
    const stateBased = continuationOpen && (!waiting)
      && (!nothingTracked);
    /**
     Final assistant message the judge would read.
     */
    const message = event.last_assistant_message ?? '';
    /**
     Whether the trailing-question detector already owns this stop.
     */
    const trailingQuestion = (!event.stop_hook_active)
      && (findTrailingQuestion(stripNonProseRegions(message,),) !== NO_MATCH);

    if ((!continuationOpen) || trailingQuestion
      || (message.trim() === '')
      || (!lunaEnabled(env[LUNA_SWITCH_ENV] ?? '',))) {
      l.debug('Luna not consulted for this stop',);
      return stopRemindersDecision({
        event,
        forcedContinuationAllowed: stateBased,
        transcriptLines: transcript.lines,
      },);
    }

    /**
     Record skeleton shared by the verdict and fallback entries.
     */
    const base = {
      sessionId: event.session_id,
      backgroundTasks: (event.background_tasks ?? []).map(function summarize(task,) {
        return {
          type: task.type,
          status: task.status,
        };
      },),
      depth,
      message,
      permissionMode: event.permission_mode,
      sessionCrons: (event.session_crons ?? []).length,
    };
    /**
     Where this stop's record is appended.
     */
    const path = verdictLogPath(env,);

    /**
     Why this stop is released without a request: the situation cannot be a premature stop.
     
     A running subagent reports back on its own. In plan mode the agent cannot perform the
     work it announces, so announcing it and stopping is the mode working as designed.
     A queued task notification is delivered as a new turn right after the stop.
     The empty string means none applies.
     */
    const releaseWithoutAsking = (event.permission_mode === 'plan')
      ? 'plan mode: the agent cannot act yet'
      : (hasRunningSubagent(event.background_tasks,)
        ? 'a subagent is running'
        : (notificationQueued
          ? 'a task notification is queued'
          : ''));
    if (releaseWithoutAsking !== '') {
      // Recorded so the verdict log still shows what each skip covered.
      l.debug(`Luna skipped: ${releaseWithoutAsking}`,);
      await appendVerdict({
        path,
        record: {
          ...base,
          outcome: 'skipped',
          detail: releaseWithoutAsking,
          action: 'release',
        },
      },);
      return stopRemindersDecision({
        event,
        forcedContinuationAllowed: false,
        transcriptLines: transcript.lines,
      },);
    }

    try {
      /**
       Credential for this request; never logged.
       */
      const key = await readKey({
        env,
        cwd: event.cwd,
      },);
      /**
       Luna's validated reading of the final message.
       */
      const verdict = await askLunaSituation({
        message,
        key,
        fetchImpl,
      },);
      /**
       Action the table assigns this verdict given the hook state.
       */
      const action = situationAction({
        verdict,
        backgroundTasks: event.background_tasks ?? [],
      },);
      // Debug, not info: console.info writes to stdout, which a hook reserves for its JSON output.
      l.debug(`Luna verdict ${verdict.situation} at ${String(verdict.probability,)}: ${action.kind} (${action.why})`,);
      await appendVerdict({
        path,
        record: {
          ...base,
          outcome: 'luna',
          detail: action.why,
          action: action.kind,
          situation: verdict.situation,
          probabilities: verdict.probabilities,
        },
      },);
      return stopRemindersDecision({
        event,
        forcedContinuationAllowed: false,
        situation: action,
        transcriptLines: transcript.lines,
      },);
    }
    catch (error) {
      l.warn(`Luna unavailable, using state-based continuation: ${String(error,)}`,);
      await appendVerdict({
        path,
        record: {
          ...base,
          outcome: 'fallback',
          detail: String(error,),
          action: stateBased ? 'block' : 'release',
        },
      },);
      return stopRemindersDecision({
        event,
        forcedContinuationAllowed: stateBased,
        transcriptLines: transcript.lines,
        notice: `stop-reminders: the Luna check was unavailable (${String(error,)}), `
          + 'so this stop used the state-based rules.',
      },);
    }
  };
}

/**
 Stop-reminders handler wired to the real environment, `fetch`, and credential lookup.
 */
const stopRemindersHandler: (event: ReadonlyDeep<StopInput>,) => Promise<StopRemindersOutput> =
  createStopRemindersHandler();

/**
 Parses raw stdin as a {@link StopInput}.
 
 Input is trusted; it comes from Claude Code's hook dispatch system.
 
 @param raw - JSON payload from Claude Code stdin
 
 @returns parsed Stop event
 
 @example
 ```ts
 const event = stopRemindersParser(await text(process.stdin));
 ```
 */
function stopRemindersParser(raw: string,): StopInput {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- trusted JSON contract from Claude Code hook system
  return JSON.parse(raw,) as StopInput;
}

/**
 Serializes the stop-reminders output for stdout.
 
 No trailing newline; matches Claude Code's wire convention.
 
 @param output - {@link StopRemindersOutput} handler result to serialize
 
 @returns JSON string for stdout
 
 @mutates output - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.
 
 @example
 ```ts
 process.stdout.write(stopRemindersWriter({}));
 ```
 */
function stopRemindersWriter(output: StopRemindersOutput,): string {
  return JSON.stringify(output,);
}

export type {
  StopRemindersDependencies,
  StopRemindersOutput,
};

export {
  createStopRemindersHandler,
  stopRemindersDecision,
  stopRemindersHandler,
  stopRemindersParser,
  stopRemindersWriter,
};
