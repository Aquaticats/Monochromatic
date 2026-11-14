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

 `idle` means the agent ended its turn with nothing of its own still running: a run whose work is still going when the agent
 settles is kept, and ends on its wall clock instead if the work outlasts it. `model-error` means Pi settled on a last assistant
 message that ended in an error nothing recovered from, `pi-not-stopped`
 means Pi was still alive when every bound on ending it ran out, `compaction-failed` means a compaction Pi reported failing left
 it silent instead of taking the prompt, and `driver-failed` covers the driver's own failures.

 `pi-exited` is Pi leaving a run that had taken its prompt, so work may have been done. `pi-exited-before-prompt` is Pi leaving
 every start the driver made before any of them took the prompt: nothing of the run's work happened, and the summary names what Pi
 printed instead.
 */
export type EndReason =
  | 'idle'
    | 'stop-file'
    | 'prompt-rejected'
    | 'compaction-failed'
    | 'pi-exited'
    | 'pi-exited-before-prompt'
    | 'turn-cap'
    | 'wall-clock'
    | 'driver-failed'
    | 'model-error'
    | 'pi-not-stopped'
    | 'masking-not-armed';

/**
 Every reason a run can end with, so a reader of summaries can ask about all of them without naming any twice.
 */
export const END_REASONS: readonly EndReason[] = [
  'idle',
  'stop-file',
  'prompt-rejected',
  'compaction-failed',
  'pi-exited',
  'pi-exited-before-prompt',
  'turn-cap',
  'wall-clock',
  'driver-failed',
  'model-error',
  'pi-not-stopped',
  'masking-not-armed',
];

/**
 Where a run stands: over for a reason, or still going.
 */
export type RunProgress = EndReason | 'still-running';

/**
 How a command's wait ended.

 `answered` means Pi wrote a response, which may itself report a refusal: what Pi made of the command is in that record.
 `silence` and `deadline` are the driver's own bounds running out, and `pi-exited` is Pi going before it answered.
 */
export type CommandEnding =
  | 'answered'
    | 'silence'
    | 'deadline'
    | 'pi-exited';

/**
 Pi's answer to a command, or the driver's own reason for waiting no longer. Which one it is says where the fields are: an
 answer carries Pi's own record, and every other ending carries the driver's words for it.
 */
export type CommandAnswer =
  | {
    readonly ending: 'answered';

    /**
     Pi's own response record, whose `success` and `error` say what Pi made of the command.
     */
    readonly record: PiRecord;
  }
    | {
      readonly ending: 'silence' | 'deadline' | 'pi-exited';

    /**
     What the driver names for stopping: the bound Pi stayed silent for, or the deadline the run reached.
     */
      readonly problem: string;

    /**
     What Pi reported of a compaction this wait saw: one that failed, or one still under way when the wait ended. A run that
     ends on it did not end because Pi refused anything.
     */
      readonly compactionProblem?: string;
    };

/**
 What Pi reports of its own session: a compaction under way, or nothing.
 */
export type CompactionOfPi =
  | { readonly compacting: false }
    | {
      readonly compacting: true;

    /**
     Why Pi began the compaction, as `compaction_start` reports it: `manual`, `threshold` or `overflow`.
     */
      readonly reason: string;

    /**
     When the driver saw the compaction begin, as milliseconds since the epoch.
     */
      readonly since: number;
    };

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
 What Pi's own automatic retry reported when it stopped retrying the model error the run stands on, or that it reported nothing
 of one. A retry that is turned off in Pi's settings, and an error Pi does not retry, both report nothing.
 */
export type PiRetryGaveUp =
  | { readonly gaveUp: false }
    | {
      readonly gaveUp: true;

    /**
     How many attempts Pi's retry made at the error, as its `auto_retry_end` reported them.
     */
      readonly attempts: number;

    /**
     What Pi said when it stopped, which is that event's `finalError`: the error's own text, or Pi's word for a retry it
     cancelled.
     */
      readonly problem: string;
    };

/**
 Why a run started Pi no more times: the starts it allows were spent, the sequence of them reached the time one start-up may take,
 or the run's own wall clock was too near to start another Pi inside it.
 */
export type NoFurtherStart =
  | 'tries-spent'
    | 'start-sequence-spent'
    | 'wall-clock-too-near';

/**
 One start that ended before its Pi took the prompt, as the summary carries it, so a reader counts the starts a run needed and
 reads what Pi printed on the one that ran out.
 */
export type PiStartFailure = {
  /**
   Which start this was, counting from one.
   */
  readonly attempt: number;

  /**
   How Pi ended, which is an exit rather than a refusal to start: a Pi the operating system would not start is never started
   again, because nothing about the machine changes between two such tries.
   */
  readonly exit: PiExit;

  /**
   Milliseconds the driver waited after this start before it started Pi again, and zero when this start was the run's last.
   */
  readonly pauseMilliseconds: number;

  /**
   The last lines Pi wrote to its standard error on this start, so the cause is in the summary and no reader needs the log.
   */
  readonly standardErrorLines: readonly string[];

  /**
   Why Pi would not go on, in its own words, taken from those lines; absent when they name no reason.
   */
  readonly reason?: string;
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
   What Pi reported of its own automatic retry when it gave up on the model error the run ended with, so a coordinator sees that
   the error was Pi's retry's to answer, that Pi answered it and stopped, and how many attempts it made. It is here only when a
   model error stands and Pi said it gave up on it; a 429 left to Pi is the case it was added for.
   */
  readonly piRetryGaveUp?: {
    readonly attempts: number;
    readonly problem: string;
  };

  /**
   Why the feed is incomplete, when writing it failed.
   */
  readonly feedProblem?: string;

  /**
   What went wrong in the driver itself, when `endReason` is `driver-failed`.
   */
  readonly driverError?: string;

  /**
   What the driver failed to read when it asked what the run left running, which is why it cannot say whether anything of the
   run's still runs. It sits beside the end reason rather than in its place: the agent had ended its turn and Pi had exited by the
   time it was asked, so the run did its work whether or not the answer could be read.
   */
  readonly workProblem?: string;

  /**
   What Pi reported of the compaction it could not finish while the run waited for its prompt: the failure, or the compaction
   still running when the run's own bound ended it. Absent when the run never waited on a compaction.
   */
  readonly compactionProblem?: string;

  /**
   How many times the driver started Pi in this run: one for a run Pi went on with, and more when a start ended before the prompt
   was taken and the driver started Pi again. Every run says, so a reader counts over a runs folder how often this happens.
   */
  readonly piStartAttempts: number;

  /**
   Every start that ended before its Pi took the prompt, in the order they happened. A run that ended as
   `pi-exited-before-prompt` has one per start; a run that went on after a later start has the ones it rode out, and its own
   `endReason` is the one it earned.
   */
  readonly piStartFailures?: readonly PiStartFailure[];

  /**
   Why no further start followed the last of those, which is present only when the run ended as `pi-exited-before-prompt`. It
   tells a caller whether every start the driver allows failed or whether the time one start-up may take was already spent.
   */
  readonly piStartStoppedBy?: NoFurtherStart;
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

  /**
   Seconds the driver waits before it starts Pi again after a start ended before it took the prompt.
   */
  readonly startPauseSeconds: number;
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

  /**
   How many times the driver started Pi, which is one unless a start ended before its Pi took the prompt.
   */
  readonly piStartAttempts?: number;

  /**
   The starts that ended before their Pi took the prompt.
   */
  readonly piStartFailures?: readonly PiStartFailure[];

  /**
   Why no further start followed the last of them, when that is why the run ended as it did.
   */
  readonly piStartStoppedBy?: NoFurtherStart;
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

  /**
   How many replies had succeeded when this attempt was sent, so an attempt is credited to the failures before the reply that
   arrived while its command was being answered rather than to the ones after it.
   */
  readonly answersBefore: number;
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
   Counts that Pi took the run's first prompt, which is the point after which work may have begun: a Pi that leaves after it is
   never started again.
   */
  readonly countPromptTaken: () => void;

  /**
   Whether Pi took the run's first prompt.
   */
  readonly promptTaken: () => boolean;

  /**
   Keeps what went wrong in the driver itself, for the summary.
   */
  readonly noteDriverFailure: (failure: string,) => void;

  /**
   Keeps what the driver failed to read when it asked what the run left running, which the feed notes under its own name and the
   summary carries beside the end reason.
   */
  readonly noteWorkProblem: (failure: string,) => void;

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
   How many assistant messages have replied without an error, which tells an attempt sent before a reply from one sent after it.
   */
  readonly answersTaken: () => number;

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
