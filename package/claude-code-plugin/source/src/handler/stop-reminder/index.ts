import type { StopInput, } from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ReadonlyDeep, } from 'type-fest';

import {
  AUTO_CONTINUE_ENV,
  autoContinueEnabled,
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
import {
  type StopRemindersOutput,
  stopRemindersDecision,
} from './decision.ts';
import { readLunaKey, } from './luna-key.ts';
import {
  askLunaSituation,
  LUNA_SWITCH_ENV,
  lunaEnabled,
} from './luna-situation.ts';
import { hasQueuedTaskNotification, } from './queued-notification.ts';
import { alreadyRaised, } from './reminder-history.ts';
import { questionLead, } from './reminder-lead.ts';
import { situationAction, } from './situation-action.ts';
import { earlierStopTexts, } from './turn-history.ts';
import {
  appendVerdict,
  verdictLogPath,
} from './verdict-log.ts';
import {
  findTrailingQuestion,
  NO_MATCH,
  stripNonProseRegions,
} from './uncertainty.ts';

/**
 Module logger for the stop-reminders handler.
 */
const logger = tagged({ tag: 'stop-reminders', },);

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
     Trailing question in the final message, or `NO_MATCH`.
     */
    const asked = findTrailingQuestion(stripNonProseRegions(message,),);
    /**
     Whether the trailing-question detector already owns this stop.

     Inside a blocked chain it owns the stop only for a question it has not raised in this turn,
     the same once-per-trigger rule the decision applies.
     */
    const trailingQuestion = (asked !== NO_MATCH)
      && (!(event.stop_hook_active
        && alreadyRaised({
          lead: questionLead(asked.sentence,),
          transcriptLines: transcript.lines,
        },)));

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
      stopHookActive: event.stop_hook_active,
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
        // History removed three repeated false blocks, each with a background task running, and moved the 2026-10-29
        // miss, which had none, under the cutoff. It is sent only on a repeated stop with a task running.
        earlierStops: (event.stop_hook_active && hasRunningBackgroundTask(event.background_tasks,))
          ? earlierStopTexts({
            transcriptLines: transcript.lines,
            finalMessage: message,
          },)
          : [],
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

export type { StopRemindersOutput, } from './decision.ts';

export { stopRemindersDecision, } from './decision.ts';

export type { StopRemindersDependencies, };

export {
  createStopRemindersHandler,
  stopRemindersHandler,
  stopRemindersParser,
  stopRemindersWriter,
};
