/**
 Refuses a code-mode script that builds an `rm` command at run time.
 
 Code mode is a surface of its own: the script text reaches the guard as one
 `code` field, and the commands it builds do not exist until it runs.
 A script that hands `tools.bash` a command it spelled out is covered by the
 nested call, which the Bash guard refuses on its own.
 A script that builds the command from a value is refused here,
 before any of its text can reach a shell.
 
 @module
 */

import {
  literalRmPathRefusal,
  type LiteralRmPathFinding,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  scriptLiterals,
  type ScriptLiteral,
} from './codemode-script-literals.ts';
import {
  GUARDRAIL_NOT_BLOCKED,
  type GuardrailDecision,
} from './types.ts';
import { isRecord, } from './value.ts';

//region Logging

/**
 Logger root for the code-mode script guard.
 */
const parentLogger = tagged({ tag: 'pi-guardrail-codemode', },);

//endregion Logging

//region Constants

/**
 Programs whose deletions the literal-path rule covers, as this guard reads them.
 */
const DELETING_PROGRAM_NAMES: ReadonlySet<string> = new Set([
  'rm',
  'rmdir',
  'shred',
  'unlink',
]);

/**
 Program whose expression can delete the paths it matched.
 */
const FIND_PROGRAM_NAME = 'find';

/**
 Word that asks `find` to delete what it matched.
 */
const FIND_DELETE_ACTION = '-delete';

/**
 Program word this guard reads when none stands where a command can begin.

 An empty program word cannot be confused with a real one, so this reader needs
 no separate absence value.
 */
const NO_PROGRAM_WORD = '';

/**
 Characters that end one command and start the next inside a command line.
 */
const COMMAND_SEPARATORS: ReadonlySet<string> = new Set([
  '&',
  '(',
  ';',
  '|',
]);

/**
 Character that ends a line, which also starts a new command.
 */
const LINE_BREAK = '\n';

/**
 Character that separates the words of a command line.
 */
const WORD_SEPARATOR = ' ';

/**
 Path separator, so a command word spelled by path is read by its last segment.
 */
const PATH_SEPARATOR = '/';

/**
 Field a code-mode tool call carries its script in.
 */
const CODE_FIELD = 'code';

/**
 Sentence naming what this guard found, put in front of the shared refusal.
 */
const SCRIPT_BUILDS_COMMAND_SENTENCE = 'Blocked: this code-mode script builds a deletion command at run time.';

//endregion Constants

//region Predicates

/**
 Reads the last path segment of one command word.
 
 @param word - word from the start of a command line
 
 @returns final path segment, or the whole word when it carries no separator
 
 @example
 ```ts
 commandProgramName('/usr/bin/rm'); // 'rm'
 ```
 */
function commandProgramName(word: string,): string {
  /**
   Offset of the last separator, or -1 when the word carries none.
   */
  const separatorOffset = word.lastIndexOf(PATH_SEPARATOR,);
  return separatorOffset === (-1)
    ? word
    : word.slice(separatorOffset + 1,);
}

/**
 Reads the program word a piece of text puts in command position.
 
 A literal's text is read as a command line with every substitution replaced by
 a space, so `rm -rf ${d}` names `rm` where the shell would run it.
 
 @param text - text one literal copies
 
 @returns the first program word in command position, or undefined when none stands there
 
 @example
 ```ts
 programInCommandPosition({ text: 'rm -rf  d', }); // 'rm'
 ```
 */
function programInCommandPosition({
  text,
}: {
  readonly text: string;
},): string {
  return (function readsWords(): string {
    /**
     Whether the next word read stands where a command can begin.
     */
    let atCommandStart = true;
    for (const word of text.split(WORD_SEPARATOR,)) {
      if (word.length === 0)
        continue;
      /**
       First character of the word, which may separate this command from the next.
       */
      const firstCharacter = word.charAt(0,);
      if (COMMAND_SEPARATORS.has(firstCharacter,)) {
        atCommandStart = true;
        continue;
      }
      if (firstCharacter === LINE_BREAK)
        atCommandStart = true;
      if (!atCommandStart)
        continue;
      return word;
    }
    return NO_PROGRAM_WORD;
  })();
}

/**
 Reports whether one command word names a program that deletes what it names.
 
 `find` counts only when the same text asks it to delete,
 because the paths that action deletes are never written.
 
 @param programName - command word read by its last path segment
 
 @param text - text of the command line that word stands on
 
 @returns whether that program deletes the paths its command names
 
 @example
 ```ts
 deletesPaths({ programName: 'rm', text: 'rm -rf x' }); // true
 deletesPaths({ programName: 'find', text: 'find . -print' }); // false
 ```
 */
function deletesPaths({
  programName,
  text,
}: {
  readonly programName: string;
  readonly text: string;
},): boolean {
  if (DELETING_PROGRAM_NAMES.has(programName,))
    return true;
  return (programName === FIND_PROGRAM_NAME)
    && text.includes(FIND_DELETE_ACTION,);
}

/**
 Reports whether one literal builds a deletion command at run time.
 
 A template with a substitution or a plus-joined string computes part of its
 text, so the command it names does not exist until the script runs.
 `find` counts when the literal also asks it to delete,
 because the paths that action deletes are never written.
 
 @param literal - literal the script spells
 
 @returns whether the literal names a deletion in command position and computes part of it
 
 @example
 ```ts
 buildsDeletionAtRunTime({ isTemplate: true, text: 'rm -rf  ', holdsSubstitution: true, joinedWithPlus: false }); // true
 ```
 */
function buildsDeletionAtRunTime(literal: ScriptLiteral,): boolean {
  /**
   Program word this literal puts in command position.
   */
  const program = programInCommandPosition({ text: literal.text, },);
  if (program === NO_PROGRAM_WORD)
    return false;
  /**
   Name of that program, read by its last path segment.
   */
  const programName = commandProgramName(program,);
  /**
   Whether that program deletes the paths this command names.
   */
  const deletes = deletesPaths({
    programName,
    text: literal.text,
  },);
  if (!deletes)
    return false;
  return (literal.isTemplate && literal.holdsSubstitution)
    || literal.joinedWithPlus;
}

//endregion Predicates

//region Public reads

/**
 Lists the deletion commands one code-mode script builds at run time.
 
 @param source - script text a model asked code mode to run
 
 @returns one finding per literal that builds such a command
 
 @example
 ```ts
 findRmCommandsBuiltAtRunTime({ source: 'const c = `rm -rf ${d}`;' });
 ```
 */
function findRmCommandsBuiltAtRunTime({ source, }: { readonly source: string; },): readonly LiteralRmPathFinding[] {
  return scriptLiterals(source,)
    .filter(buildsDeletionAtRunTime,)
    .map(function findingOf(literal,): LiteralRmPathFinding {
      /**
       Program word the literal puts in command position.
       */
      const word = programInCommandPosition({
        text: literal.text,
      },);
      return {
        command: literal.text,
        word,
        check: 'shell-computes-the-path',
      };
    },);
}

/**
 Refuses a code-mode script that builds an `rm` command at run time.
 
 @param input - pi `codemode` tool input, read defensively
 
 @returns block decision when the script builds such a command
 
 @example
 ```typescript
 evaluateCodemodeScriptGuard({ input: { code: 'const c = `rm -rf ${d}`; tools.bash({ command: c });' } });
 ```
 */
function evaluateCodemodeScriptGuard({
  input,
}: {
  readonly input: unknown;
},): GuardrailDecision {
  if (!isRecord(input,))
    return GUARDRAIL_NOT_BLOCKED;
  /**
   Script the call asks code mode to run.
   */
  const code = input[CODE_FIELD];
  if ((typeof code) !== 'string')
    return GUARDRAIL_NOT_BLOCKED;

  /**
   Commands the script builds at run time.
   */
  const findings = findRmCommandsBuiltAtRunTime({ source: code, },);
  if (findings.length === 0) {
    /**
     Function-tagged logger reporting a script this guard read and allowed.
     */
    const innerLogger = tagged({
      tag: evaluateCodemodeScriptGuard.name,
      l: parentLogger,
    },);
    innerLogger.debug('code-mode script builds no rm command at run time',);
    return GUARDRAIL_NOT_BLOCKED;
  }
  return {
    block: true,
    reason: `${SCRIPT_BUILDS_COMMAND_SENTENCE} ${literalRmPathRefusal({ findings, },)}`,
  };
}

//endregion Public reads

export {
  buildsDeletionAtRunTime,
  commandProgramName,
  evaluateCodemodeScriptGuard,
  findRmCommandsBuiltAtRunTime,
  programInCommandPosition,
};
