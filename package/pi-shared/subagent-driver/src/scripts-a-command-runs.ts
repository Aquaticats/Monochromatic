/**
 The scripts a command runs and the text they hold, and whether any command a session records ran a build tool.

 A script is a file an interpreter or a shell is given as its program, so it must be a program the command runs: a file merely named
 by `ls`, `cat` or `rg` is not read as one. Each file is read through one descriptor opened without waiting, so a script that is a
 named pipe is passed over rather than waited on.

 @module
 */

import {
  basename,
  isAbsolute,
  join,
} from 'node:path';
import {
  executedWordsIn,
  scriptRunsABuildTool,
  buildToolRanIn,
} from './command-words.ts';
import { readRegularFileAt, } from './reading-a-regular-file.ts';
import type { SessionCommand, } from './session-commands-of-a-run.ts';

/**
 Bytes of one script this reader takes while looking for a built program's path.

 Measured: the reproduction script of the review of 2026-11-12T10:21:26Z was 17,173 bytes, and a script this large is already far
 more than a reproduction needs; 8 MiB is about 488 times the measured script.
 */
const SCRIPT_TEXT_LIMIT_BYTES = 8_388_608;

/**
 Reads the candidate files a word could name, each as an absolute path.

 A word holding a shell variable cannot be resolved, so its own file name is looked for in the folders a command may write from:
 that is how `/bin/bash "$S/run-binary.sh"` names the review's own script.

 @param word - the word as written

 @param folders - the folders a command's relative path may be written from

 @returns each candidate, in order, the word's own path first

 @example
 ```ts
 const candidates = candidatesOf({ word: '$S/run-binary.sh', folders: ['/reviews/a'] });
 ```
 */
function candidatesOf({
  word,
  folders,
}: {
  readonly word: string;
  readonly folders: readonly string[];
},): readonly string[] {
  /**
   The word's own file name, which a word naming a variable leaves.
   */
  const named = basename(word,);
  if (word.includes('$',)) {
    return folders.filter(function isKnown(folder,) {
      return folder !== '';
    },)
      .map(function underTheFolder(folder,) {
        return join(
          folder,
          named,
        );
      },);
  }
  if (isAbsolute(word,)) {
    return [word,];
  }
  return [
    word,
    ...folders.filter(function isKnown(folder,) {
      return folder !== '';
    },)
      .map(function underTheFolder(folder,) {
        return join(
          folder,
          word,
        );
      },),
  ];
}

/**
 Reads the text of each file a command runs as a script, and hands it to the caller.

 The files are read through one descriptor opened without waiting, so a script that is a named pipe is passed over rather than
 waited on, and a file larger than a script is refused rather than held.

 @param command - the command as written

 @param folders - the folders a script's relative path may be written from

 @param cache - what each candidate script's text is, so one script is read once

 @returns the text of every script the command runs, in order

 @example
 ```ts
 const texts = await scriptTextsRanBy({ command: 'node repro.ts', folders: ['/reviews/a'], cache: new Map() });
 ```
 */
export async function scriptTextsRanBy({
  command,
  folders,
  cache = new Map(),
}: {
  readonly command: string;
  readonly folders: readonly string[];
  readonly cache?: Map<string, Promise<string>>;
},): Promise<readonly string[]> {
  /**
   Every script word the command runs.
   */
  const scripts = executedWordsIn({ command, },)
    .filter(function isAScript(ran,) {
      return ran.role === 'script';
    },);
  /**
   What each script holds, or empty text when it cannot be read.
   */
  const texts = await Promise.all(scripts.map(async function scriptText(ran,) {
    /**
     Every file this script word could name.
     */
    const candidates = candidatesOf({
      word: ran.word,
      folders,
    },);
    /**
     What each candidate holds, read once for the whole comparison.
     */
    const read = await Promise.all(candidates.map(function readOne(candidate,): Promise<string> {
      /**
       The text this candidate is already known to hold, or a read that is kept for the next command.
       */
      const kept = cache.get(candidate,);
      if (kept !== undefined) {
        return kept;
      }
      /**
       The read, kept before it is awaited so a later command shares it.
       */
      const reading = (async function textOf(): Promise<string> {
        /**
         What the read answered.
         */
        const one = await readRegularFileAt({
          path: candidate,
          limitBytes: SCRIPT_TEXT_LIMIT_BYTES,
        },);
        return (one.kind === 'text') ? one.text : '';
      })();
      cache.set(
        candidate,
        reading,
      );
      return reading;
    },),);
    return read.find(function isText(text,) {
      return text !== '';
    },) ?? '';
  },),);
  return texts;
}

/**
 Reads a relative word's path without the `./` a command may write before it.

 @param word - the word a command runs

 @returns the word's path as it stands under the folder the command moved to

 @example
 ```ts
 const tail = tailOf({ word: './target/debug/a' });
 ```
 */
function tailOf({ word, }: { readonly word: string },): string {
  return word.startsWith('./',) ? word.slice(2,) : word;
}

/**
 Reads a program's path from its first build-output segment, so a script that holds `target/debug/a` rather than the whole path is
 still recognised, and a one-letter file name alone is not.

 @param path - the program's absolute path

 @returns its path from the first `target`, `dist`, `debug` or `release` segment, or the whole path when none is there

 @example
 ```ts
 const tail = buildTailOf('/w/package/a/target/debug/a');
 ```
 */
export function buildTailOf(path: string,): string {
  /**
   The path's segments.
   */
  const segments = path.split('/',);
  /**
   Where the first build-output segment stands, or minus one when none does.
   */
  const at = segments.findIndex(function isABuildSegment(segment,) {
    return (segment === 'target') || (segment === 'dist')
      || (segment === 'debug')
      || (segment === 'release');
  },);
  return (at === (-1)) ? path : segments.slice(at,)
    .join('/',);
}

/**
 Says whether a command runs a program or holds one in a script it runs.

 @param command - the command as written

 @param program - the built program's absolute path

 @param scriptTexts - the text of each script the command runs

 @param folders - the folders the command's paths may be written from

 @returns whether the program is run by the command, or held by a script it runs

 @example
 ```ts
 const runs = commandRunsTheProgram({ command, program: '/w/target/debug/a', scriptTexts: [], folders: ['/w'] });
 ```
 */
export function commandRunsTheProgram({
  command,
  program,
  scriptTexts,
  folders,
}: {
  readonly command: string;
  readonly program: string;
  readonly scriptTexts: readonly string[];
  readonly folders: readonly string[];
},): boolean {
  /**
   Whether a word the command runs is the program itself.
   */
  const ranItself = executedWordsIn({ command, },)
    .some(function isTheProgram(ran,) {
      return candidatesOf({
        word: ran.word,
        folders,
      },)
        .some(function resolvesTo(candidate,) {
          // A word that names a shell variable cannot be resolved, so the build tail the program's path begins with is what is
          // compared. A relative word names a path under a folder the command moved to, which this reader does not follow, so its
          // own tail is compared only when it holds a folder; a bare word such as `git` is a program found on PATH, never a path.
          return (candidate === program)
            || (ran.word
              .includes(buildTailOf(program,),))
            || ((!isAbsolute(ran.word,))
              && ran.word
                .includes('/',)
              && program.endsWith(`/${tailOf({ word: ran.word, },)}`,));
        },);
    },);
  if (ranItself) {
    return true;
  }
  return scriptTexts.some(function holds(one,) {
    return one.includes(program,) || one.includes(buildTailOf(program,),);
  },);
}

/**
 Says whether any command a session records ran a build tool, directly or in a script it runs, and names those commands.

 @param commands - the commands the session records

 @param folders - the folders a command's script may be written from

 @param cache - what each candidate script's text is, so one script is read once

 @returns whether a build ran, and the commands that ran one

 @example
 ```ts
 const built = await buildRanInACommand({ commands, folders: ['/w', '/reviews/a'], cache: new Map() });
 ```
 */
export async function buildRanInACommand({
  commands,
  folders,
  cache,
}: {
  readonly commands: readonly SessionCommand[];
  readonly folders: readonly string[];
  readonly cache: Map<string, Promise<string>>;
},): Promise<{
  readonly ran: boolean;
  readonly commands: readonly string[]
}> {
  /**
   Each command's own answer, and each of its scripts' answers.
   */
  const answers = await Promise.all(commands.map(async function buildIn(command,) {
    if (buildToolRanIn({ command: command.command, },)) {
      return command.command;
    }
    /**
     The text of each script this command runs.
     */
    const texts = await scriptTextsRanBy({
      command: command.command,
      folders,
      cache,
    },);
    return texts.some(function builds(text,) {
      return scriptRunsABuildTool({ text, },);
    },) ? command.command : '';
  },),);
  /**
   The commands that ran a build tool.
   */
  const built = answers.filter(function isABuild(command,) {
    return command !== '';
  },);
  return {
    ran: built.length > 0,
    commands: built,
  };
}
