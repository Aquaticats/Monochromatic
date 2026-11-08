/**
 Shared shapes of the driver: one record of Pi's RPC protocol, how Pi exited, and what a finished run reports.

 @module
 */

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
 */
export type EndReason = 'idle' | 'stop-file' | 'prompt-rejected' | 'pi-exited' | 'turn-cap' | 'wall-clock' | 'driver-failed';

/**
 Where a run stands: over for a reason, or still going.
 */
export type RunProgress = EndReason | 'still-running';

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
};
