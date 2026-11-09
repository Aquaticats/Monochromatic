/**
 Shared shapes of the driver: one record of Pi's RPC protocol, how Pi exited, and what a finished run reports.

 @module
 */

import type { ModelErrorKind, } from './retries-of-a-model-error.ts';

/**
 One JSON object of Pi's RPC protocol, in either direction.
 */
export type PiRecord = Readonly<Record<string, unknown>>;

/**
 How the Pi process ended. A field is absent when it does not apply: a signalled process has no code, and a process that could
 not be started has neither.
 */
export type PiExit = {
  readonly code?: number;
  readonly signal?: string;

  /**
   Why the operating system refused to start or signal Pi.
   */
  readonly processError?: string;
};

/**
 Why a run ended. `idle` and `stop-file` are the ends a run chooses; the others fail the driver.

 `model-error` means Pi settled on a last assistant message that ended in an error nothing recovered from, `pi-not-stopped`
 means Pi was still alive when every bound on ending it ran out, and `driver-failed` covers the driver's own failures.
 */
export type EndReason =
  | 'idle'
    | 'stop-file'
    | 'prompt-rejected'
    | 'pi-exited'
    | 'turn-cap'
    | 'wall-clock'
    | 'driver-failed'
    | 'model-error'
    | 'pi-not-stopped';

/**
 Where a run stands: over for a reason, or still going.
 */
export type RunProgress = EndReason | 'still-running';

/**
 A failure a run can end with, or the reason there is none. Absence travels as a value rather than as a nullish member, so every
 place that reads one has to say what it does when nothing failed.
 */
export type RunFailure =
  | { readonly failed: false }
    | {
      readonly failed: true;

    /**
     What failed, in words the summary and the exit status carry.
     */
      readonly problem: string;
    };

/**
 What `summary.json` holds.
 */
export type RunSummary = {
  readonly endReason: EndReason;

  /**
   How Pi ended; absent only when Pi outlasted every signal.
   */
  readonly piExit?: PiExit;
  readonly totalMilliseconds: number;
  readonly turns: number;
  readonly settled: number;
  readonly toolHistogram: Readonly<Record<string, number>>;
  readonly toolErrors: readonly PiRecord[];
  readonly dialogs: readonly PiRecord[];

  /**
   Pi's own session statistics, which name the session file and hold the session's cost, or the error asking for them gave.
   */
  readonly sessionStats: unknown;
  readonly lastAssistantText: string;

  /**
   The error of the last assistant message, when Pi settled on it and nothing after it recovered, which is why the run ended as
   `model-error`.
   */
  readonly modelError?: string;

  /**
   How many attempts the run made at its last model error: one attempt and each retry after it. It is here only when the run ended
   on that error, and a run whose attempts ran out names them all here.
   */
  readonly modelErrorAttempts?: number;

  /**
   How many attempts after a model error the run made, so a run that needed one reads apart from a run that did not. It is zero
   for every run that never answered a model error with another attempt.
   */
  readonly retries: number;

  /**
   How many model errors named a rate limit or a quota, whether or not another attempt could fix them, because the coordinator
   measures those apart from other failures.
   */
  readonly rateLimitErrors: number;

  /**
   Why the feed is incomplete, when writing it failed.
   */
  readonly feedProblem?: string;

  /**
   What went wrong in the driver itself, when `endReason` is `driver-failed`.
   */
  readonly driverError?: string;
};

/**
 Options of one run of the driver.
 */
export type DriverOptions = {
  readonly runFolder: string;
  readonly workingFolder: string;
  readonly promptFile: string;
  readonly provider: string;
  readonly model: string;

  /**
   Session file to resume; absent for a new session.
   */
  readonly session?: string;

  /**
   Further arguments for Pi, each kept whole.
   */
  readonly piArguments: readonly string[];
  readonly wallSeconds: number;
  readonly idleSeconds: number;
  readonly turnCap: number;

  /**
   How many attempts follow a model error another attempt may fix. Zero ends every such run as it ended before: on the first
   error.
   */
  readonly retryCap: number;

  /**
   Seconds before the first attempt after a model error, doubling with each attempt.
   */
  readonly retryBaseSeconds: number;
};

/**
 One event and how to decline a dialog so it cannot stall the session.
 */
export type EventObservation = {
  readonly event: PiRecord;
  readonly declineDialog: (id: unknown,) => void;
};

/**
 What the summary needs from the end of the run.
 */
export type RunEnding = {
  readonly endReason: EndReason;
  readonly piExit: PiExit;
  readonly piHasExited: boolean;
  readonly sessionStats: unknown;
};

/**
 One attempt after a model error, as the feed and the summary carry it.
 */
export type RetryTaken = {
  /**
   Which attempt this is, counting from one.
   */
  readonly attempt: number;

  /**
   How many attempts the run allows, so a reader sees the bound this one sits under.
   */
  readonly of: number;

  /**
   Milliseconds the driver waited before this attempt.
   */
  readonly delayMilliseconds: number;

  /**
   What kind of failure it answers.
   */
  readonly kind: ModelErrorKind;

  /**
   What the model said when its message ended in the error this attempt answers.
   */
  readonly problem: string;
};

/**
 The feed and the counts of one run.
 */
export type RecordOfPiRun = {
  /**
   Whether Pi is at work. Only `agent_settled` clears this: `agent_end` ends one low-level run, and retries or follow-up work can
   still come.
   */
  readonly isRunning: () => boolean;
  readonly settledCount: () => number;
  readonly turnCount: () => number;

  /**
   How long nothing has been heard or noted.
   */
  readonly silentMilliseconds: () => number;

  /**
   Writes one feed line, which also counts as activity: a message just delivered must not look like a long silence.
   */
  readonly note: (entry: PiRecord,) => void;

  /**
   Counts a prompt Pi handled without starting a run, for which no `agent_settled` will come.
   */
  readonly countHandledPrompt: () => void;

  /**
   Keeps what went wrong in the driver itself, for the summary.
   */
  readonly noteDriverFailure: (failure: string,) => void;

  /**
   Why the feed is incomplete, when writing it failed. A run whose feed failed did not end clean.
   */
  readonly feedProblem: () => RunFailure;

  /**
   The error Pi settled on, when the last assistant message ended in one and nothing after it recovered.
   */
  readonly unrecoveredModelError: () => RunFailure;

  /**
   How many attempts after a model error the run has made, which bounds the retries it may still take.
   */
  readonly retriesTaken: () => number;

  /**
   Notes an attempt after a model error on the feed, counts it for the summary, and takes the error it answers as answered, so the
   same error is never counted twice.
   */
  readonly countRetry: (retry: RetryTaken,) => void;
  readonly observe: (observation: EventObservation,) => void;
  readonly summarize: (ending: RunEnding,) => RunSummary;

  /**
   Finishes writing the feed.
   */
  readonly close: () => Promise<void>;
};
