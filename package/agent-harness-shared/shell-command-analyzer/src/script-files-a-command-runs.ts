/**
 Names the script files one shell command reads from disk.
 
 A guard that reads only the command text misses the file behind it:
 `bash deploy.sh` runs whatever that file holds,
 `source ./env.sh` reads another file into the shell,
 and `./cleanup` runs a file whose first line asks for a shell.
 Each of those is a script the command runs,
 so each is named here for the caller to read.
 
 @module
 */

import { homedir, } from 'node:os';
import { resolve, } from 'node:path';

import { analyzeShellCommand, } from './analyzer.ts';
import { unwrapWrappers, } from './command-wrapper-words.ts';
import {
  lastPathSegment,
  looksLikeExecutablePath,
} from './executable-path-words.ts';
import { rawCommandWordRuns, } from './raw-command-word-runs.ts';
import { shellScriptPayloads, } from './shell-script-payloads.ts';

//region Constants

/**
 Shell programs whose first file operand is a script they read.
 */
const SHELL_PROGRAM_NAMES: ReadonlySet<string> = new Set([
  'ash',
  'bash',
  'dash',
  'ksh',
  'mksh',
  'sh',
  'zsh',
]);

/**
 Shell builtins that read a file into the running shell.
 */
const SOURCE_BUILTIN_NAMES: ReadonlySet<string> = new Set([
  '.',
  'source',
]);

/**
 Shell options whose value is the next word rather than a script file.
 */
const SHELL_OPTIONS_WITH_VALUE: ReadonlySet<string> = new Set([
  '+O',
  '+o',
  '-O',
  '-o',
  '--init-file',
  '--rcfile',
]);

/**
 Shell flag cluster that hands the shell its script as a word instead of a file.
 */
const SHELL_SCRIPT_FLAG = 'c';

/**
 Word that ends a program's own options.
 */
const OPTIONS_TERMINATOR = '--';

/**
 Prefix of a path written from the user's home folder.
 */
const HOME_PREFIX = '~/';

/**
 Separator between the identity parts of one candidate, which no path carries.
 */
const CANDIDATE_IDENTITY_SEPARATOR = '\u0000';

/**
 Sentinel returned when a command hands no program a script file.
 */
const NO_SCRIPT_FILE_OPERAND: unique symbol = Symbol('command names no script file',);

//endregion Constants

//region Types

/**
 One file a command reads as a shell script.
 */
type ScriptFileCandidate = {
  /**
   Absolute path of the file.
   */
  readonly path: string;
  /**
   Whether the command's own words made the shell read this file as a script.
   
   A file named as a bare path word is read only when its first line asks for
   a shell, so its shebang decides.
   */
  readonly shellReadsIt: boolean;
};

//endregion Types

//region Reads

/**
 Reads the file a shell or the `source` builtin was handed.
 
 @param words - innermost program words, program first
 
 @returns the word naming the script file, or {@link NO_SCRIPT_FILE_OPERAND} when the command hands the shell no file
 
 @example
 ```ts
 shellFileOperand(['bash', '-x', 'deploy.sh']); // 'deploy.sh'
 shellFileOperand(['bash', '-c', 'echo hi']); // NO_SCRIPT_FILE_OPERAND
 ```
 */
function shellFileOperand(words: readonly string[],): string | typeof NO_SCRIPT_FILE_OPERAND {
  /**
   Shell program name, whose own option grammar this reader follows.
   */
  const programName = lastPathSegment(words.at(0,)
    ?? '',);
  /**
   Whether a `source` builtin reads the file, which has no script option to skip.
   */
  const readsFileOnly = SOURCE_BUILTIN_NAMES.has(programName,);

  for (let index = 1; index < words.length; index += 1) {
    /**
     Word inspected as option, option value, or file operand.
     */
    const word = words[index];
    if (word === undefined)
      return NO_SCRIPT_FILE_OPERAND;
    if (word === OPTIONS_TERMINATOR)
      continue;
    if (word.startsWith('-',)) {
      if (readsFileOnly)
        continue;
      if (SHELL_OPTIONS_WITH_VALUE.has(word,)) {
        index += 1;
        continue;
      }
      if (word.includes(SHELL_SCRIPT_FLAG,))
        return NO_SCRIPT_FILE_OPERAND;
      continue;
    }
    return word;
  }
  return NO_SCRIPT_FILE_OPERAND;
}

/**
 Resolves one path word the way the shell would, from the tool call's working directory.
 
 @param cwd - working directory the command runs in
 
 @param word - path word as the command spells it
 
 @returns absolute path of the file that word names
 
 @example
 ```ts
 resolvedScriptPath({ cwd: '/repo', word: './clean.sh' }); // '/repo/clean.sh'
 ```
 */
function resolvedScriptPath({
  cwd,
  word,
}: {
  readonly cwd: string;
  readonly word: string;
},): string {
  return word.startsWith(HOME_PREFIX,)
    ? resolve(
      homedir(),
      word.slice(HOME_PREFIX.length,),
    )
    : resolve(
      cwd,
      word,
    );
}

/**
 Names the script file one innermost program launches, when it launches any.
 
 @param words - innermost program words, program first
 
 @param cwd - working directory the command runs in
 
 @returns one candidate when this program reads a script file, none when it reads none
 
 @example
 ```ts
 scriptFileOfProgram({ words: ['bash', 'clean.sh'], cwd: '/repo' });
 ```
 */
function scriptFileOfProgram({
  words,
  cwd,
}: {
  readonly words: readonly string[];
  readonly cwd: string;
},): readonly ScriptFileCandidate[] {
  /**
   Innermost program word, which decides how this command reads a file.
   */
  const program = words.at(0,);
  if (program === undefined)
    return [];
  /**
   Program name, read by its last path segment.
   */
  const programName = lastPathSegment(program,);
  /**
   Whether this program reads the file it is handed as a script.
   */
  const readsScriptFiles = SHELL_PROGRAM_NAMES.has(programName,)
    || SOURCE_BUILTIN_NAMES.has(programName,);
  if (readsScriptFiles) {
    /**
     Word naming the file this program reads.
     */
    const operand = shellFileOperand(words,);
    if ((typeof operand) === 'symbol')
      return [];
    return [{
      path: resolvedScriptPath({
        cwd,
        word: operand,
      },),
      shellReadsIt: true,
    },];
  }
  if (!looksLikeExecutablePath(program,))
    return [];
  return [{
    path: resolvedScriptPath({
      cwd,
      word: program,
    },),
    shellReadsIt: false,
  },];
}

/**
 Collects the script files one command line's words name.
 
 @param words - command word followed by its arguments
 
 @param cwd - working directory the command runs in
 
 @returns the file those words name, when they name one
 
 @example
 ```ts
 scriptFileOfCommandLine({ words: ['env', 'bash', 'clean.sh'], cwd: '/repo' });
 ```
 */
function scriptFileOfCommandLine({
  words,
  cwd,
}: {
  readonly words: readonly string[];
  readonly cwd: string;
},): readonly ScriptFileCandidate[] {
  /**
   Innermost program words, with every wrapper layer the command line spelled stripped.
   */
  const wrapped = unwrapWrappers({ words, },);
  return scriptFileOfProgram({
    words: wrapped.words,
    cwd,
  },);
}

//endregion Reads

//region Public reads

/**
 Names every script file one shell command reads from disk.
 
 Nested sources are read from a work list rather than by recursion,
 and a source text already read is not read twice,
 so a command that launches its own text terminates.
 
 The walk covers the command's own words, the command strings a shell or `eval`
 runs, and the lines `find -exec` and its siblings run.
 It does not follow a file the named script reads in turn.
 
 @param command - shell source from an agent harness Bash tool input
 
 @param cwd - working directory the command runs in
 
 @returns one candidate per distinct file the command names, in reading order
 
 @example
 ```ts
 scriptFilesACommandRuns({ command: 'bash deploy.sh', cwd: '/repo' });
 // [{ path: '/repo/deploy.sh', shellReadsIt: true }]
 ```
 */
function scriptFilesACommandRuns({
  command,
  cwd,
}: {
  readonly command: string;
  readonly cwd: string;
},): readonly ScriptFileCandidate[] {
  /**
   Sources still to read, the command text first.
   */
  const pending: string[] = [command,];
  /**
   Source texts already read, which ends a source that launches its own text.
   */
  const readSources = new Set<string>();
  /**
   Candidates collected, keyed by path so a file read twice is named once.
   */
  const candidates = new Map<string, ScriptFileCandidate>();

  while (pending.length > 0) {
    /**
     Source text read next.
     */
    const source = pending.pop();
    if (source === undefined)
      break;
    if (readSources.has(source,))
      continue;
    readSources.add(source,);
    for (const words of commandLinesOfSource(source,)) {
      for (const candidate of scriptFileOfCommandLine({
        words,
        cwd,
      },)) {
        /**
         Identity of this candidate, which keeps the stricter reading of the two.
         */
        const identity = `${candidate.path}${CANDIDATE_IDENTITY_SEPARATOR}${String(candidate.shellReadsIt,)}`;
        if (!candidates.has(identity,))
          candidates.set(
            identity,
            candidate,
          );
      }
      /**
       Innermost program words of this command line.
       */
      const wrapped = unwrapWrappers({ words, },);
      pending.push(
        ...wrapped.payloads,
        ...wrapped.argumentCommandLines,
        ...shellScriptPayloads({ words: wrapped.words, },),
      );
    }
  }

  return [...candidates.values(),];
}

/**
 Reads one source text as the command lines it holds.
 
 A source the parser rejects is read from its raw word runs,
 because the visible text is the only evidence it carries.
 
 @param source - shell source text
 
 @returns command word lists in reading order
 
 @example
 ```ts
 commandLinesOfSource('cd x && bash clean.sh');
 ```
 */
function commandLinesOfSource(source: string,): readonly (readonly string[])[] {
  /**
   Parsed shell command analysis.
   */
  const analysis = analyzeShellCommand(source,);
  if (!analysis.parsed)
    return rawCommandWordRuns({ source, },);
  return analysis.commands
    .map(function commandWords(info,): readonly string[] {
      return [
        info.name,
        ...info.args,
      ];
    },);
}

//endregion Public reads

export {
  commandLinesOfSource,
  NO_SCRIPT_FILE_OPERAND,
  scriptFileOfCommandLine,
  scriptFileOfProgram,
  scriptFilesACommandRuns,
  shellFileOperand,
};
export type { ScriptFileCandidate, };
