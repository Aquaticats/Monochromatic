/**
 Shared shapes of the driver: one record of Pi's RPC protocol, how Pi exited, and what a finished run reports.

 @module
 */

/**
 One JSON object of Pi's RPC protocol, in either direction.
 */
export type PiRecord = Readonly<Record<string, unknown>>;

/**
 How the Pi process ended.
 */
export type PiExit = {
  readonly code: number | null;
  readonly signal: string | null;

  /**
   Why Pi never started, or null when it did.
   */
  readonly startError: string | null;
};

/**
 Why a run ended. `idle` and `stop-file` are the ends a run chooses; the others fail the driver.
 */
export type EndReason =
  | 'idle'
  | 'stop-file'
  | 'prompt-rejected'
  | 'pi-exited'
  | 'turn-cap'
  | 'wall-clock';

/**
 What `summary.json` holds.
 */
export type RunSummary = {
  readonly endReason: EndReason;
  readonly piExit: PiExit | null;
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
   Session file to resume, or null for a new session.
   */
  readonly session: string | null;

  /**
   Further arguments for Pi, each kept whole.
   */
  readonly piArguments: readonly string[];
  readonly wallSeconds: number;
  readonly idleSeconds: number;
  readonly turnCap: number;
};
