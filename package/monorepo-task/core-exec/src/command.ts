#!/usr/bin/env node

/**
 Command executor wrapper for mise tasks.
 
 This script works around task runner limitations where tasks with `allowFailure: true`
 can't be dependencies.
 
 By wrapping commands in this executor, we can control exit codes while still
 showing all output and errors to the user.
 
 Usage:
   task-command --allowFailure; oxlint  # Always exits with 0
   task-command; oxlint                 # Exits with command's exit code
 
 The `--` separator is required to distinguish script args from command args.
 
 @example
 ```bash
 # Always exit with 0
 task-command --allowFailure; oxlint
 
 # Exit with command's exit code
 task-command; oxlint
 
 # Execute through shell
 task-command --shell; "echo hello && echo world"
 
 # Execute with timeout of 5 seconds
 task-command --timeout 5000; npm test
 ```
 */

// TODO: deprecate Optique
import { object, } from '@optique/core/constructs';
// TODO: deprecate Optique
import {
  multiple,
  optional,
} from '@optique/core/modifiers';
// TODO: deprecate Optique
import {
  argument,
  option,
} from '@optique/core/primitives';
// TODO: deprecate Optique
import {
  integer,
  string,
} from '@optique/core/valueparser';
// TODO: deprecate Optique
import { runSync, } from '@optique/run';
import spawn from 'nano-spawn';
import dedent from 'string-dedent';
import { match, } from 'ts-pattern';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';



//region Parser definition: defines CLI flags and rest arguments after --

/**
 TODO: deprecate Optique
 Optique parser for the task-command CLI
 */
const parser = object({
  allowFailure: option(
    '-a',
    '--allowFailure',
  ),
  shell: option(
    '-s',
    '--shell',
  ),
  timeout: optional(option(
    '-t',
    '--timeout',
    integer(),
  ),),
  rest: multiple(
    argument(string(),),
  ),
},);

//endregion Parser definition

/**
 TODO: deprecate Optique
 Parsed CLI arguments from process.argv
 */
const args = runSync(
  parser,
  {
    programName: 'task-command',
    help: 'option',
  },
);

/**
 Destructured command and its arguments from the rest args after `--`
 */
const [command, ...commandArgs] = args.rest;

if ((command === undefined) || (command === '')) {
  throw new Error(
    dedent`
      No command specified after --
      ${
      JSON.stringify(
        args,
        null,
        2,
      )
    }
    `,
  );
}

try {
  // Execute the command with nano-spawn
  await spawn(
    command,
    commandArgs,
    {
      stdout: 'inherit',
      stderr: 'inherit',
      stdin: 'inherit',
      shell: args.shell,
      timeout: ((typeof args.timeout) === 'number') ? args.timeout : undefined,
    },
  );

  // Script ends naturally with exit code 0
}
catch (error) {
  // nano-spawn throws SubprocessError when the process fails
  match(error,)
    .when(
    function isSubprocessError(
      candidate,
    ): candidate is {
      readonly exitCode?: number;
      readonly signalName?: string;
      readonly message: string;
    } {
      return (candidate !== null)
        && ((typeof candidate) === 'object')
        && ('exitCode' in candidate);
    },
    function handleSubprocessError(subprocessError,): void {
      match(subprocessError.signalName,)
        .when(
        function hasSignal(signal,): signal is string {
          return signal !== undefined;
        },
        function logSignal(signal,): void {
          console.error(`Command terminated by signal: ${signal}`,);
        },
        );

        // Exit with 0 if allowFailure is true, otherwise use the command's exit code
      match(args.allowFailure,)
        .with(
        false,
        function exitWithCode(): void {
          process.exitCode = subprocessError.exitCode
            ?? 1;
        },
        )
        .with(
        true,
        function allowFailureNoop(): void {
              // Let script end naturally with exit code 0
        },
        );
    },
    )
    .otherwise(function handleUnknownError(): void {
      console.error(
        `Failed to execute command: ${
          caughtValueText(error,)
        }`,
      );
      match(args.allowFailure,)
        .with(
        false,
        function rethrowError(): void {
          throw error;
        },
        )
        .with(
        true,
        function allowFailureNoop(): void {
            // Let script end naturally with exit code 0
        },
        );
    },);
}
