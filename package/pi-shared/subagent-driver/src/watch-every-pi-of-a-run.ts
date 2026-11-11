/**
 Starts Pi for a run and watches it to its end, starting another provider's Pi when a spent quota leaves the run nowhere to go
 on the one it has.

 A run keeps its deadline, its turn cap and its one record across every Pi it starts: the wall clock the caller gave is the outer
 bound of all of it, and the feed and the summary read as one run that moved rather than as two runs that failed.

 @module
 */

import {
  type ConnectionToPi,
  startConnectionToPi,
} from './connection-to-pi.ts';
import {
  FEED_PREVIEW_CHARACTERS,
  NESTED_RUN_VARIABLE,
} from './driver-constants.ts';
import type {
  DriverOptions,
  EndReason,
  RecordOfPiRun,
} from './driver-types.ts';
import { preview, } from './previews-of-pi-messages.ts';
import {
  type RunPutAway,
  putTheRunAway,
} from './put-the-run-away.ts';
import {
  THE_PROMPT_FILE,
  type PromptToSend,
  endReasonOfRun,
} from './take-the-first-prompt.ts';
import type { WatchRequest, } from './watch-request.ts';
import {
  type ProviderMove,
  switchProviderOnSpentQuota,
  switchWarranted,
} from './switch-provider-on-a-spent-quota.ts';

/**
 Builds Pi's command line. `--session` is its own word, so a path holding spaces stays one argument.

 @param options - options of this run

 @param provider - the provider this Pi runs on

 @param model - the model this Pi runs on

 @param session - the session file this Pi resumes, or nothing for a new session

 @returns the arguments for Pi
 */
function piCommandLine({
  options,
  provider,
  model,
  session,
}: {
  readonly options: DriverOptions;
  readonly provider: string;
  readonly model: string;
  readonly session?: string
},): readonly string[] {
  return [
    '--mode',
    'rpc',
    '--provider',
    provider,
    '--model',
    model,
    ...(session === undefined ? [] : [
      '--session',
      session,
    ]),
    ...options.piArguments,
  ];
}

/**
 What watching every Pi of a run left.
 */
export type WatchedRun = {
  /**
  The last Pi the run started, whose statistics the summary carries.
   */
  readonly connection: ConnectionToPi;

  /**
  Why the run ended.
   */
  readonly endReason: EndReason;

  /**
  How that Pi was put away.
   */
  readonly putAway: RunPutAway;
};

/**
 A run's Pi, and what the run has to decide about it.
 */
export type WatchedRunRequest = Omit<WatchRequest, 'connection'> & {
  /**
  Pi's environment, already stripped of every credential-shaped name and already carrying the variable that refuses a nested
  driver.
   */
  readonly environment: NodeJS.ProcessEnv;

  /**
  Where Pi's standard error is kept.
   */
  readonly standardErrorFile: string;
};

/**
 Builds the decliner of a dialog for one Pi: a dialog left waiting is a run that never ends.

 @param connection - the running Pi

 @returns the decliner for that Pi
 */
function dialogDeclinerOf(connection: ConnectionToPi,): (id: unknown) => void {
  return function declineDialog(id: unknown,): void {
    connection.write({
      type: 'extension_ui_response',
      id,
      cancelled: true,
    },);
  };
}

/**
 Watches the Pi of a run, and the Pi a spent quota moves it to.

 The move happens at most as often as a provider named in `switchWarranted` is left behind, which is once: the Pi that follows runs
 on the fallback provider, and a spent quota there ends the run rather than moving it again.

 @param options - options of this run

 @param record - the run's record, which every Pi of the run writes to

 @param inbox - the folder a coordinator drops messages in

 @param deadline - when the run must be over, as milliseconds since the epoch

 @param environment - Pi's environment, already stripped of every credential-shaped name

 @param standardErrorFile - where Pi's standard error is kept

 @returns the last Pi, why the run ended, and how that Pi was put away

 @example
 ```ts
 const watched = await watchEveryPiOfARun({ options, record, inbox, deadline, environment, standardErrorFile, },);
 ```
 */
export async function watchEveryPiOfARun({
  options,
  record,
  inbox,
  deadline,
  environment,
  standardErrorFile,
}: WatchedRunRequest,): Promise<WatchedRun> {
  /**
  What the run's Pi starts on, and what its first prompt says.
   */
  const pi: {
    provider: string;
    model: string;
    session?: string;
    prompt: PromptToSend;
  } = {
    provider: options.provider,
    model: options.model,
    ...(options.session === undefined ? {} : { session: options.session, }),
    prompt: THE_PROMPT_FILE,
  };
  for (;;) {
    /**
    The running Pi.
     */
    const connection: ConnectionToPi = startConnectionToPi({
      piArguments: piCommandLine({
        options,
        provider: pi.provider,
        model: pi.model,
        ...(pi.session === undefined ? {} : { session: pi.session, }),
      },),
      workingFolder: options.workingFolder,
      environment: {
        ...environment,
        [NESTED_RUN_VARIABLE]: '1',
      },
      standardErrorFile,
      listeners: {
        onEvent: function onEvent(event,) {
          record.observe({
            event,
            declineDialog,
          },);
        },
        onUnreadableLine: function onUnreadableLine(line,) {
          record.note({
            kind: 'non-json',
            line: preview({
              value: line,
              length: FEED_PREVIEW_CHARACTERS.unreadableLine,
            },),
          },);
        },
        onExit: function onExit(exit,) {
          record.note({
            kind: 'exit',
            ...exit,
          },);
        },
        onProblem: function onProblem(problem,) {
          record.note({
            kind: 'problem',
            problem,
          },);
        },
      },
    },);
    /**
    Declines a dialog for this Pi, which would otherwise wait for an answer forever.
     */
    const declineDialog = dialogDeclinerOf(connection,);
    /**
    Why this Pi's run is over.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one Pi's end decides whether another is started at all; see doc/lint/sequential-awaits.md
    const { endReason, } = await endReasonOfRun({
      watch: {
        connection,
        record,
        options,
        inbox,
        deadline,
      },
      prompt: pi.prompt,
    },);
    /**
    The error this Pi settled on, when nothing after it recovered.
     */
    const failure = record.unrecoveredModelError();
    if ((endReason !== 'model-error') || (!failure.failed)
      || (!switchWarranted({
        provider: pi.provider,
        failure,
      },))) {
      return {
        connection,
        endReason,
        // oxlint-disable-next-line eslint/no-await-in-loop -- the Pi that ended is put away before this run reports; see doc/lint/sequential-awaits.md
        putAway: await putTheRunAway({
          connection,
          record,
        },),
      };
    }
    /**
    What moving the run to the fallback provider left.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- the move puts this Pi away, and the next one starts only after; see doc/lint/sequential-awaits.md
    const move: ProviderMove = await switchProviderOnSpentQuota({
      connection,
      record,
      provider: pi.provider,
      model: pi.model,
    },);
    if (!move.moved) {
      return {
        connection,
        endReason,
        putAway: move.putAway,
      };
    }
    record.takeFailureAsAnswered();
    pi.provider = move.to
      .provider;
    pi.model = move.to
      .model;
    pi.session = move.sessionFile;
    pi.prompt = {
      kind: 'after-a-provider-switch',
      problem: failure.problem,
      from: options.provider,
      to: move.to,
    };
  }
}
