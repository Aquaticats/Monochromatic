/**
 The shapes of the channel to Pi: what starting it needs, what it tells the run, one command to send, and how it is ended.

 @module
 */

import type {
  CommandAnswer,
  PiExit,
  PiRecord,
} from './driver-types.ts';
import type { TextMasking, } from './secret-value-masking.ts';

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

  /**
   Whether Pi's standard error is added to what the file already holds, which is how a run whose Pi is started again keeps the
   whole of what each Pi printed in one file. It is off by default, so a file a run is given starts empty.
   */
  readonly standardErrorAppends?: boolean;
  readonly listeners: ConnectionListeners;

  /**
   The secret values this run masks, as the text the extension Pi loads reads from its own descriptor.

   Passing it opens that descriptor in the process: the driver writes the values once and closes its end, and Pi's extension reads
   them while it loads. A caller that passes none starts Pi with three descriptors, as every run did before the masking existed, and
   a Pi whose extension reports no values in place refuses to work (`take-the-first-prompt.ts`).
   */
  readonly secretValuesHandover?: string;

  /**
   The masking Pi's standard error goes through as it is kept, which is the one place that text passes before the file holds it.

   A caller that passes none keeps it as Pi wrote it, which is what a test of this seam alone wants.
   */
  readonly mask?: TextMasking;

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
 How ending Pi went: what the driver had to do, and how much of it the ending's own bounds allowed.

 What the driver learned of Pi's leaving is the question this answers, never what it hoped: a Pi whose exit the operating system
 reported is one that left, whichever stage of the ending heard of it. A process a tool of Pi's holds the output of is the case
 that separates the two: Pi has gone, its output has not ended, and the run must still record that Pi left.
 */
export type ShutDownOutcome = {
  /**
   Whether Pi's leaving was known once every stage of the ending was over. False is a process the driver could not end, which is
   the only ending `pi-not-stopped` names.
   */
  readonly stopped: boolean;

  /**
   The signals the driver sent Pi, in the order it sent them; empty when closing Pi's input was enough.
   */
  readonly signalsSent: readonly NodeJS.Signals[];

  /**
   How long each stage of the ending was given, which is the bound the reason names when Pi outlived them.
   */
  readonly stageMilliseconds: number;

  /**
   Whether the last stage of waiting for Pi ran out with its leaving still unseen, so that the ending's own bounds were spent
   before the driver could say Pi had gone.
   */
  readonly outlivedTheLastBound: boolean;

  /**
   Why the operating system refused a signal, when it refused one: a refused signal is no proof that Pi went.
   */
  readonly refusedSignal?: string;
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
   Ends Pi: closes its input, then signals it if it stays, then settles every handle. What the ending took, and whether Pi's
   leaving was known by the time it was over, are what it answers; a signal the operating system refused is not proof that Pi
   went.
   */
  readonly shutDown: (request?: ShutDownRequest,) => Promise<ShutDownOutcome>;
};
