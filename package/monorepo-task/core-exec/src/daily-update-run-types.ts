/**
 The shapes the daily update's runner and its real effects share: what a command reports, the outside world the runner
 acts on, and what a group did.

 @module
 */

import type { UpdateCommand, } from './daily-update-plan.ts';

/**
 What a finished command reports.
 */
export type CommandOutcome = {
  /**
   Exit status; any nonzero value, including the one of a command that was stopped, is a failure.
   */
  readonly exitCode: number;
  /**
   Whether the command was stopped for running past its time bound.
   */
  readonly timedOut: boolean;
  /**
   Last lines of the command's output, for the notification.
   */
  readonly outputTail: string;
};

/**
 The outside world the runner acts on.
 */
export type DailyUpdateEffects = {
  /**
   Runs one command in the repository and waits for it, within its own time bound.
   */
  readonly runCommand: (command: UpdateCommand,) => Promise<CommandOutcome>;
  /**
   Repository-relative paths of tracked files whose contents differ from `HEAD`, plus untracked files.
   */
  readonly dirtyPaths: () => Promise<readonly string[]>;
  /**
   Repository-relative paths of every tracked file.
   */
  readonly trackedPaths: () => Promise<readonly string[]>;
  /**
   Reads a file's text; the argument is an absolute path.
   */
  readonly readText: (absolutePath: string,) => Promise<string>;
  /**
   Replaces a file's text.
   */
  readonly writeText: (request: {
    readonly absolutePath: string;
    readonly text: string
  },) => Promise<void>;
  /**
   Commits exactly the given repository-relative paths; rejects when the commit fails.
   */
  readonly commit: (request: {
    readonly message: string;
    readonly paths: readonly string[]
  },) => Promise<void>;
  /**
   Commit `HEAD` names now, as a full object name.
   */
  readonly headCommit: () => Promise<string>;
  /**
   Tells the owner something, on the desktop and in the journal.
   */
  readonly notify: (request: {
    readonly title: string;
    readonly body: string
  },) => Promise<void>;
  /**
   Writes one line to the journal.
   */
  readonly log: (line: string,) => void;
  /**
   Absolute path of the repository.
   */
  readonly repository: string;
};

/**
 What one group did.
 */
export type GroupReport = {
  /**
   Group name.
   */
  readonly name: string;
  /**
   What came of the group: `landed` (committed), `unchanged` (nothing moved), `failed` or `skipped` (files were dirty).
   */
  readonly outcome: 'failed' | 'landed' | 'skipped' | 'unchanged';
  /**
   Repository-relative paths committed, empty unless the outcome is `landed`.
   */
  readonly committed: readonly string[];
  /**
   The commit the group made, empty unless the outcome is `landed`.
   */
  readonly commit: string;
  /**
   Plain-language problems the owner is told about, empty when nothing went wrong.
   */
  readonly problems: readonly string[];
};
