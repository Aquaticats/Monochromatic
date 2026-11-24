/**
 Reads every `rm` one Bash tool call would run that names its paths at run time.
 
 The command text answers for itself,
 and each script file the command reads from disk is read as a file so its
 findings name their line.
 A file that cannot be read gives no finding and logs why at debug level,
 because a throwing guard would fail every Bash call.
 
 @module
 */

import { readFile, } from 'node:fs/promises';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  findLiteralRmPaths,
  findLiteralRmPathsInScriptFile,
} from './literal-rm-paths.ts';
import type { LiteralRmPathFinding, } from './literal-rm-path-finding.ts';
import {
  type ScriptFileCandidate,
  scriptFilesACommandRuns,
} from './script-files-a-command-runs.ts';
import { isShellScriptFile, } from './shell-script-file.ts';

//region Logging

/**
 Logger root for the literal-`rm`-path guard.
 */
const parentLogger = tagged({ tag: 'literal-rm-paths', },);

/**
 Sentinel returned when a script file cannot be read as text.
 */
const SCRIPT_FILE_UNREADABLE: unique symbol = Symbol('script file cannot be read as text',);

//endregion Logging

//region File reads

/**
 Reads one script file the command runs.
 
 @param path - absolute path of the file
 
 @returns the file's text, or {@link SCRIPT_FILE_UNREADABLE} when it cannot be read as text
 
 @example
 ```ts
 await scriptFileTextOrNone('/repo/clean.sh');
 ```
 */
async function scriptFileTextOrNone(path: string,): Promise<string | typeof SCRIPT_FILE_UNREADABLE> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error: unknown) {
    /**
     Function-tagged logger reporting a script the guard could not read.
     */
    const innerLogger = tagged({
      tag: scriptFileTextOrNone.name,
      l: parentLogger,
    },);
    innerLogger.debug(
      `script file not read, no finding from it: ${path}: ${caughtValueText(error,)}`,
    );
    return SCRIPT_FILE_UNREADABLE;
  }
}

//endregion File reads

//region Public reads

/**
 Reads every `rm` a Bash tool call would run that names its paths at run time.
 
 @param command - shell source from an agent harness Bash tool input
 
 @param cwd - working directory the command runs in
 
 @returns one finding per offending command and word
 
 @example
 ```ts
 await findLiteralRmPathsInCommand({ command: 'bash clean.sh', cwd: '/repo' });
 ```
 */
async function findLiteralRmPathsInCommand({
  command,
  cwd,
}: {
  readonly command: string;
  readonly cwd: string;
},): Promise<readonly LiteralRmPathFinding[]> {
  /**
   Candidates the command names, each read once.
   */
  const candidates = scriptFilesACommandRuns({
    command,
    cwd,
  },);
  /**
   Findings from the command text and from every script file it reads.
   */
  return [
    ...findLiteralRmPaths({ source: command, },),
    ...(await Promise.all(candidates
      .map(function scriptFileFindings(candidate,): Promise<readonly LiteralRmPathFinding[]> {
        return findingsOfScriptFile(candidate,);
      },)))
      .flat(),
  ];
}

/**
 Reads one candidate script file and reports the `rm` findings it holds.
 
 @param candidate - file the command names
 
 @returns findings of that file, empty when it is unreadable or is no shell script
 
 @example
 ```ts
 await findingsOfScriptFile({ path: '/repo/clean.sh', shellReadsIt: true });
 ```
 */
async function findingsOfScriptFile(candidate: ScriptFileCandidate,): Promise<readonly LiteralRmPathFinding[]> {
  /**
   Text of the script file, absent when it cannot be read.
   */
  const text = await scriptFileTextOrNone(candidate.path,);
  if ((typeof text) === 'symbol')
    return [];
  /**
   Whether a shell reads this file, which the command's own words or its first line settles.
   */
  const shellReadsIt = candidate.shellReadsIt
    || isShellScriptFile({
      path: candidate.path,
      text,
    },);
  if (!shellReadsIt)
    return [];
  return findLiteralRmPathsInScriptFile({
    path: candidate.path,
    source: text,
  },);
}

//endregion Public reads

export { findLiteralRmPathsInCommand, };
