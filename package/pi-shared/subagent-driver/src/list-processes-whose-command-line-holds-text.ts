/**
 `list-processes-whose-command-line-holds-text`: lists every process whose command line holds a text, with its number, its
 parent, its age and the head of its command line. See `README.md` for the options.

 It reads command lines and never an environment, and it never names its own process: a text that would match this command
 itself finds the processes a reader was asking about.

 @module
 */

import {
  argv,
  pid,
  stdout,
} from 'node:process';
import { optionIn, } from './option-in.ts';
import {
  COMMAND_LINE_CHARACTERS,
  matchLineOf,
  processesWhoseCommandLineHolds,
} from './processes-whose-command-line-holds-text.ts';
import { startProcessTable, } from './processes-under-a-pi-run.ts';
import {
  requiredPositiveNumber,
  requiredText,
} from './required-option-values.ts';
import { runCommandWithChildProcess, } from './run-command.ts';

/**
 The command's command line, for the error message.
 */
const PROCESS_LIST_USAGE =
  'list-processes-whose-command-line-holds-text --text=<text> [--command-characters=<n>]';

/**
 The option that names the text a command line must hold.
 */
const TEXT_OPTION = '--text=';

/**
 The option that names how much of each command line to print.
 */
const COMMAND_CHARACTERS_OPTION = '--command-characters=';

/**
 A command line the command cannot use.
 */
class ProcessListOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ProcessListOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw processListOptionsErrorOf('--text is required.',);
 ```
 */
function processListOptionsErrorOf(message: string,): Error {
  return new ProcessListOptionsError(message,);
}

/**
 The words the command was given.
 */
const words = argv.slice(2,);

/**
 The option that names how much of each command line to print, as it was written.
 */
const commandCharactersGiven = optionIn({
  commandLine: words,
  option: COMMAND_CHARACTERS_OPTION,
},);

/**
 What this run of the command was asked for.
 */
const options = {
  text: requiredText({
    value: optionIn({
      commandLine: words,
      option: TEXT_OPTION,
    },),
    name: 'text',
    usage: PROCESS_LIST_USAGE,
    errorOf: processListOptionsErrorOf,
  },),
  commandCharacters: (commandCharactersGiven === '')
    ? COMMAND_LINE_CHARACTERS
    : requiredPositiveNumber({
      value: commandCharactersGiven,
      name: 'command-characters',
      usage: PROCESS_LIST_USAGE,
      errorOf: processListOptionsErrorOf,
    },),
};

/**
 The process table, read once for the whole answer.
 */
const table = await startProcessTable({ run: runCommandWithChildProcess, },)();

/**
 The processes whose command line holds the text, which are this command's answer.
 */
const found = processesWhoseCommandLineHolds({
  table,
  text: options.text,
  excludingPid: pid,
  commandCharacters: options.commandCharacters,
},);

if (found.length === 0) {
  stdout.write(`no process whose command line holds ${options.text}\n`,);
}
else {
  stdout.write(`${found.map(function lineOf(match,) {
    return matchLineOf(match,);
  },)
    .join('\n',)}\n`,);
}
