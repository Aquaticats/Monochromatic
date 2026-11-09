/**
 The shapes of the channel to Pi: what starting it needs, what it tells the run, one command to send, and how it is ended.

 @module
 */

import type {
  CommandAnswer,
  PiExit,
  PiRecord,
} from './driver-types.ts';

/**
 What the connection tells the run about.
 */
export type ConnectionListeners = {
  readonly onEvent: (record: PiRecord,) => void;
  readonly onUnreadableLine: (line: string,) => void;
  readonly onExit: (exit: PiExit,) => void;

  /**
   A failure of the channel itself, such as a write to a Pi that has gone.
   */
  readonly onProblem: (problem: string,) => void;
};

/**
 What starting Pi needs.
 */
export type ConnectionRequest = {
  readonly piArguments: readonly string[];
  readonly workingFolder: string;

  /**
   Pi's environment, passed as given: this seam starts one process and applies no policy. `supervisePiRun` is where the
   credential rule runs, so a caller of this function that builds its own environment keeps whatever it passes.
   */
  readonly environment: NodeJS.ProcessEnv;
  readonly standardErrorFile: string;
  readonly listeners: ConnectionListeners;

  /**
   How long Pi gets at each stage of ending; tests shorten it.
   */
  readonly graceMilliseconds?: number;
};

/**
 A command to send and how long its answer may be waited for.
 */
export type CommandRequest = {
  readonly command: PiRecord;

  /**
   Longest Pi may stay silent while answering. A compaction Pi reports under way is not silence, because Pi is at work then, so
   the bound stops counting until the compaction ends. Its comment in `driver-constants.ts` says where the value comes from.
   */
  readonly answerMilliseconds?: number;

  /**
   When the wait must end whatever Pi is doing, as milliseconds since the epoch. This is the outer bound: a compaction that
   never ends, or one that is reported every few seconds, cannot hold a command past it.
   */
  readonly deadline: number;
};

/**
 What ending Pi needs.
 */
export type ShutDownRequest = {
  /**
   How long each stage of ending Pi may take. The whole ending takes at most `SHUTDOWN_STAGES` of these, so a caller that has its
   own bound divides it by that count.
   */
  readonly graceMilliseconds?: number;
};

/**
 How ending Pi went.
 */
export type ShutDownOutcome =
  | { readonly stopped: true }
    | {
      readonly stopped: false;

    /**
     Why the driver could not confirm that Pi had gone.
     */
      readonly reason: string;
    };

/**
 A running Pi.
 */
export type ConnectionToPi = {
  readonly hasExited: () => boolean;

  /**
   How Pi ended, or nothing while it runs.
   */
  readonly exitDetails: () => PiExit;

  /**
   Writes a command with no answer to wait for.
   */
  readonly write: (command: PiRecord,) => void;

  /**
   Sends a command and waits for its answer, within a bound. Pi's silence or absence comes back as a failed answer.
   */
  readonly send: (request: CommandRequest,) => Promise<CommandAnswer>;

  /**
   Ends Pi: closes its input, then signals it if it stays, then settles every handle. It reports whether Pi's leaving could be
   confirmed, because a signal the operating system refused is not proof that Pi went.
   */
  readonly shutDown: (request?: ShutDownRequest,) => Promise<ShutDownOutcome>;
};
