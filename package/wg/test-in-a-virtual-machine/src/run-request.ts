/**
 What a run was asked to do, and the command line one phase becomes.

 Kept apart from the program's entry point so the reading of arguments and the
 rendering of a phase's command line can be checked without creating a machine.

 @module
 */

import { renderCommandLine, } from './shell-command.ts';
import {
  GUEST_ACCOUNT,
} from './machine.ts';
import {
  type TestDefinition,
  TEST_DEFINITIONS,
  type TestPhase,
} from './test-definitions.ts';

/**
 What one invocation was asked to do.
 */
export type Options = {
  /**
   Identifier of the test to run.
   */
  readonly test: string;

  /**
   Host file the whole output is also written to.
   */
  readonly logPath: string | undefined;
};

/**
 Reads this program's arguments.

 @param argumentsList - arguments after the program name

 @returns what the run should do

 @throws when an argument is missing or unknown
 */
export function parseArguments(argumentsList: readonly string[],): Options {
  /**
   Identifier of the test to run, once one is given.
   */
  let test: string | undefined;
  /**
   Host file the output is also written to, once one is given.
   */
  let logPath: string | undefined;
  for (let index = 0; index < argumentsList.length; index += 1) {
    /**
     The argument at this position.
     */
    const argument = argumentsList[index];
    if (argument === '--test') {
      test = argumentsList[index + 1];
      index += 1;
      continue;
    }
    if (argument === '--log') {
      logPath = argumentsList[index + 1];
      index += 1;
      continue;
    }
    throw new Error(`unknown argument ${String(argument,)}.`,);
  }
  if (test === undefined) {
    throw new Error(`--test names the test to run, one of: ${Object.keys(TEST_DEFINITIONS,).join(', ',)}.`,);
  }
  return {
    test,
    logPath,
  };
}

/**
 Answers the definition one identifier names.

 @param id - identifier from the command line

 @returns the definition

 @throws when no test has that identifier
 */
export function definitionOf(id: string,): TestDefinition {
  /**
   The definition the identifier names, when there is one.
   */
  const definition = TEST_DEFINITIONS[id];
  if (definition === undefined) {
    throw new Error(`no test named ${id}; this package runs ${Object.keys(TEST_DEFINITIONS,).join(', ',)}.`,);
  }
  return definition;
}

/**
 Renders the guest command one phase runs, writing both streams to its log.

 @param phase - the phase to run

 @param logPath - guest file the phase's output is appended to

 @returns one command line for the guest's shell
 */
export function renderPhaseCommand(
  {
    logPath,
    phase,
  }: {
    readonly logPath: string;
    readonly phase: TestPhase;
  },
): string {
  /**
   Variables set for this command alone, as leading assignments.
   */
  const assignments: string[] = Object.entries(phase.environment,)
    .map(function renderAssignment(
      [
        name,
        value,
      ],
    ): string {
      return `${name}=${value}`;
    },);
  /**
   The command itself, in the directory it must start from.
   */
  const inner = `cd ${phase.workingDirectory} && ${
    renderCommandLine({ words: [
      ...assignments,
      ...phase.words,
    ], },)
  }`;
  /**
   Who runs it: the account through runuser, or the machine's root directly.
   */
  const words = phase.caller === 'account'
    ? [
      'runuser',
      '--user',
      GUEST_ACCOUNT,
      '--',
      'sh',
      '-lc',
      inner,
    ]
    : [
      'sh',
      '-lc',
      inner,
    ];
  return `${renderCommandLine({ words, },)} >> ${logPath} 2>&1`;
}
