/**
 The comparison between what a review's statement says it ran and what its session records: each stated command against a
 recorded call, the exit code against the one the call's result carries, and each built program the statement names against the
 command that ran it or a script that command names.

 What this comparison cannot see is said as plainly as what it can: a `process` start records no exit code, a command inside a
 `codemode` call is no tool call of its own, and a script that builds a program's path from a variable does not hold the path.
 Those are reported as uncompared or unconfirmed, never as matching.

 @module
 */

import type { Stats, } from 'node:fs';
import {
  readFile,
  stat,
} from 'node:fs/promises';
import {
  basename,
  isAbsolute,
  join,
} from 'node:path';
import type {
  ReviewCommandRun,
  ReviewStatement,
} from './review-statement-shape.ts';
import type {
  SessionCommand,
  SessionReadings,
} from './session-commands-of-a-run.ts';

/**
 Bytes of one script this reader takes while looking for a built program's path.

 Measured: the reproduction script of the review of 2026-11-12T10:21:26Z was 17,173 bytes, and a script this large is already far
 more than a reproduction needs; 8 MiB is about 488 times the measured script.
 */
const SCRIPT_TEXT_LIMIT_BYTES = 8_388_608;

/**
 How much of a command a reason quotes, so one line stays readable while still naming the command.
 */
export const COMMAND_IN_A_REASON_CHARACTERS = 160;

/**
 The path fragments that mark a folder a build writes into.
 */
const BUILD_OUTPUT_PARTS: readonly string[] = [
  '/target/',
  '/dist/',
];

/**
 No built program at a path, as a symbol so that no text can stand for it.
 */
const NOT_A_BUILT_PROGRAM: unique symbol = Symbol('path outside a worktree build folder',);

/**
 No file at a candidate path, as a symbol so that no record can stand for it.
 */
const NOT_A_FILE: unique symbol = Symbol('candidate script missing at that path',);

/**
 One candidate script's record, or the symbol that says it is no readable file.
 */
type CandidateFile = Stats | typeof NOT_A_FILE;

/**
 One stated command, compared with the session call it names.
 */
export type StatedCommandCheck = {
  /**
   The command the statement lists.
   */
  readonly stated: ReviewCommandRun;

  /**
   The index of the session call it matches, or nothing when the session records no such command.
   */
  readonly recordedAt?: number;
};

/**
 What comparing a statement with a session found.
 */
export type CommandComparison = {
  /**
   One line per claim that does not fit the session.
   */
  readonly problems: readonly string[];

  /**
   One line per claim the session cannot confirm either way.
   */
  readonly uncompared: readonly string[];

  /**
   The built programs the statement names and the session confirms, each once.
   */
  readonly builtPrograms: readonly string[];
};

/**
 Collapses the runs of whitespace in a command, so a reviewer's line breaks and a session's single line compare.

 @param text - the command as written

 @returns the text with each run of whitespace as one space

 @example
 ```ts
 const same = collapsed('  cd a \\\n  && git status ');
 ```
 */
function collapsed(text: string,): string {
  /* oxlint-disable-next-line no-restricted-syntax/no-regex -- a run of whitespace in one command line; the pattern has one
     quantifier, no backreference and no alternation, and the input is one command's own text */
  return text.replaceAll(
    /\s+/gu,
    ' ',
  )
    .trim();
}

/**
 Cuts a command to the room a reason line has.

 @param command - the command as written

 @returns the command, cut with an ellipsis when it is longer

 @example
 ```ts
 const short = quoted({ command: 'mise run //package/a:native:build' });
 ```
 */
function quoted({ command, }: { readonly command: string },): string {
  return (command.length <= COMMAND_IN_A_REASON_CHARACTERS)
    ? command
    : `${command.slice(
      0,
      COMMAND_IN_A_REASON_CHARACTERS,
    )}...`;
}

/**
 Reads one path as an absolute one, resolving a relative path against the folder the session started in.

 @param path - the path as written

 @param workingFolder - the folder the session started in, or empty text when it records none

 @returns the path, absolute when the folder is known

 @example
 ```ts
 const path = absolutePathIn({ path: './target/debug/wrapper', workingFolder: '/worktree' });
 ```
 */
function absolutePathIn({
  path,
  workingFolder,
}: {
  readonly path: string;
  readonly workingFolder: string;
},): string {
  if (isAbsolute(path,) || (workingFolder === '')) {
    return path;
  }
  return join(
    workingFolder,
    path,
  );
}

/**
 Says whether a path is a program a worktree's build wrote.

 @param path - the path as written, absolute or relative to the session's folder

 @param workingFolder - the folder the session started in

 @returns the absolute path, or nothing when it is not under the worktree's build output

 @example
 ```ts
 const built = builtProgramPathIn({ path: '/worktree/package/a/target/debug/a', workingFolder: '/worktree' });
 ```
 */
function builtProgramPathIn({
  path,
  workingFolder,
}: {
  readonly path: string;
  readonly workingFolder: string;
},): string | typeof NOT_A_BUILT_PROGRAM {
  /**
   The path, resolved against the worktree when it is relative.
   */
  const absolute = absolutePathIn({
    path,
    workingFolder,
  },);
  if ((workingFolder !== '') && (!absolute.startsWith(`${workingFolder}/`,))) {
    return NOT_A_BUILT_PROGRAM;
  }
  return BUILD_OUTPUT_PARTS.some(function isABuildFolder(part,) {
    return absolute.includes(part,);
  },)
    ? absolute
    : NOT_A_BUILT_PROGRAM;
}

/**
 Reads the words of a command, each with its quotes and the shell's separators taken off.

 @param command - the command as written

 @returns its words, in order

 @example
 ```ts
 const words = wordsOf({ command: 'node "reproductions/repro.ts" && true' });
 ```
 */
function wordsOf({ command, }: { readonly command: string },): readonly string[] {
  /* oxlint-disable-next-line no-restricted-syntax/no-regex -- the whitespace between one command's words; the pattern has one
     quantifier and no backreference, and the input is one command's own text */
  return command.split(/\s+/u,)
    .map(function withoutQuotes(word,) {
      /* oxlint-disable-next-line no-restricted-syntax/no-regex -- the quotes and separators at a word's two ends; both
         alternatives are anchored and single-character classes, and the input is one word */
      return word.replaceAll(
        /^[('"`]+|[)'"`,;]+$/gu,
        '',
      );
    },)
    .filter(function isAWord(word,) {
      return word !== '';
    },);
}

/**
 Reads one candidate file's own record, or the symbol that says it cannot be read.

 Any failure to read a candidate leaves it no script whose text can hold the path, so the failure is answered as the symbol rather
 than raised: the candidate is one word of a command, not a file the command itself needed.

 @param path - the candidate's path

 @returns its record, or {@link NOT_A_FILE}

 @example
 ```ts
 const details = await candidateFileAt('/reviews/a/reproductions/repro.ts');
 ```
 */
async function candidateFileAt(path: string,): Promise<CandidateFile> {
  try {
    return await stat(path,);
  }
  catch (error) {
    // The failure names why this candidate is not a script, and the caller moves to the next candidate either way.
    return (String(error,)
      .length
      > 0) ? NOT_A_FILE : NOT_A_FILE;
  }
}

/**
 Reads one candidate file's text, or empty text when it cannot be read.

 @param path - the candidate's path

 @returns its text, or empty text when the file cannot be read

 @example
 ```ts
 const shown = await candidateTextAt('/reviews/a/reproductions/repro.ts');
 ```
 */
async function candidateTextAt(path: string,): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    // A file that cannot be read holds no path this check can find, and the caller moves on.
    return (String(error,)
      .length
      > 0) ? '' : '';
  }
}

/**
 Says whether a file a command names holds a path, which is how a script that runs a built program is confirmed.

 Only a regular file at a folder the command could name is read, and only one no larger than a script: a command that names a
 device, a folder or a huge file is passed over rather than waited on.

 @param command - the command as written

 @param path - the built program's absolute path

 @param folders - the folders a command's relative path may be written from

 @returns whether one of the files the command names holds the path

 @example
 ```ts
 const holds = await aScriptHoldsTheProgram({ command: 'node reproductions/repro.ts', path: '/w/target/debug/w', folders: ['/w', '/reviews/a'] });
 ```
 */
async function aScriptHoldsTheProgram({
  command,
  path,
  folders,
}: {
  readonly command: string;
  readonly path: string;
  readonly folders: readonly string[];
},): Promise<boolean> {
  /**
   Every candidate file the command's words could name, each as an absolute path.
   */
  const candidates = wordsOf({ command, },)
    .flatMap(function candidatesOf(word,) {
      return isAbsolute(word,)
        ? [word,]
        : folders.filter(function isKnown(folder,) {
          return folder !== '';
        },)
          .map(function underTheFolder(folder,) {
            return join(
              folder,
              word,
            );
          },);
    },);
  /**
   Whether each candidate is a script that holds the path: the candidates are independent, so each is read at once.
   */
  const held = await Promise.all(candidates.map(async function candidateHolds(candidate,) {
    /**
     The candidate's own record, or the symbol that says it is not there.
     */
    const details = await candidateFileAt(candidate,);
    if ((details === NOT_A_FILE) || (!details.isFile())
      || (details.size > SCRIPT_TEXT_LIMIT_BYTES)) {
      return false;
    }
    /**
     What the file holds, or empty text when it cannot be read.
     */
    const shown = await candidateTextAt(candidate,);
    return (shown.includes(path,)) || (shown.includes(basename(path,),));
  },),);
  return held.some(function isHeld(one,) {
    return one;
  },);
}

/**
 Compares the commands a statement lists with the commands a session records.

 @param statement - the review's statement

 @param session - what the review's session records

 @param reviewFolder - the review's own folder, where a command's script may be written

 @returns what does not fit, what cannot be compared, and the built programs the session confirms

 @example
 ```ts
 const compared = await compareCommands({ statement, session, reviewFolder: '/reviews/a' });
 ```
 */
export async function compareCommands({
  statement,
  session,
  reviewFolder,
}: {
  readonly statement: ReviewStatement;
  readonly session: SessionReadings;
  readonly reviewFolder: string;
},): Promise<CommandComparison> {
  /**
   The session calls already claimed by a stated command.
   */
  const claimed = new Set<number>();
  /**
   The statement entry that matched each session call, by the call's index.
   */
  const statedAtOfRecorded = new Map<number, number>();
  /**
   What does not fit the session.
   */
  const problems: string[] = [];
  /**
   What the session cannot confirm either way.
   */
  const uncompared: string[] = [];
  /**
   The built programs the statement names and the session confirms.
   */
  const builtPrograms: string[] = [];
  for (const stated of statement.commands) {
    /**
     The session call whose text holds this command, which is the first one not already claimed.
     */
    const recordedAt = session.commands
      .findIndex(function holds(
        recorded,
        at,
      ) {
        return (!claimed.has(at,))
          && collapsed(recorded.command,)
            .includes(collapsed(stated.command,),);
      },);
    if (recordedAt === (-1)) {
      problems.push(
        `the statement lists the command "${quoted({ command: stated.command, },)}" and the session records no call whose text `
          + 'holds it',
      );
      continue;
    }
    claimed.add(recordedAt,);
    statedAtOfRecorded.set(
      recordedAt,
      statement.commands
        .indexOf(stated,),
    );
    /**
     The call the command matched.
     */
    const recorded = session.commands[recordedAt];
    if (recorded === undefined) {
      continue;
    }
    if (recorded.tool === 'process') {
      uncompared.push(
        `"${quoted({ command: stated.command, },)}" ran as a process start, whose result records no exit code, so the exit `
          + `code ${String(stated.exit,)} the statement gives cannot be compared`,
      );
    }
    else if (recorded.exitCode === undefined) {
      uncompared.push(
        `the session records no exit code for "${quoted({ command: stated.command, },)}", so the exit code `
          + `${String(stated.exit,)} the statement gives cannot be compared`,
      );
    }
    else if (recorded.exitCode !== stated.exit) {
      problems.push(
        `the statement says "${quoted({ command: stated.command, },)}" exited ${String(stated.exit,)} and the session `
          + `records ${String(recorded.exitCode,)}`,
      );
    }
    if (stated.ranBuiltProgram === undefined) {
      continue;
    }
    /**
     The program the statement says that command ran, as an absolute path.
     */
    const program = builtProgramPathIn({
      path: stated.ranBuiltProgram,
      workingFolder: session.workingFolder,
    },);
    if (program === NOT_A_BUILT_PROGRAM) {
      problems.push(
        `the statement says "${quoted({ command: stated.command, },)}" ran ${stated.ranBuiltProgram}, which is not a path `
          + 'under a build folder of the review\'s worktree',
      );
      continue;
    }
    if (recorded.command
      .includes(program,)) {
      builtPrograms.push(program,);
      continue;
    }
    /**
     Whether a script that command names holds the program's path.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one stated command's own script, read once its text did not hold the path. See doc/lint/sequential-awaits.md.
    const inAScript = await aScriptHoldsTheProgram({
      command: recorded.command,
      path: program,
      folders: [
        session.workingFolder,
        reviewFolder,
      ],
    },);
    if (!inAScript) {
      problems.push(
        `the statement says "${quoted({ command: stated.command, },)}" ran ${program}, and neither the command nor a script `
          + 'it names holds that path',
      );
      continue;
    }
    builtPrograms.push(program,);
  }
  for (const [at, recorded] of session.commands
    .entries()) {
    /**
     The program this session call ran, when its text names one a worktree built.
     */
    const program = wordsOf({ command: recorded.command, },)
      .map(function possibleProgram(word,) {
        return builtProgramPathIn({
          path: word,
          workingFolder: session.workingFolder,
        },);
      },)
      .find(function isTheProgram(found,) {
        return found !== NOT_A_BUILT_PROGRAM;
      },);
    if (program === undefined) {
      continue;
    }
    /**
     The statement's own entry for this call, when the statement lists it.
     */
    const statedAt = statedAtOfRecorded.get(at,);
    /**
     The statement entry itself.
     */
    const statedEntry = (statedAt === undefined) ? undefined : statement.commands[statedAt];
    /**
     The program that entry says it ran, when it says it ran one.
     */
    const statedProgram = statedEntry?.ranBuiltProgram ?? '';
    if (
      (statedProgram !== '')
        && (builtProgramPathIn({
          path: statedProgram,
          workingFolder: session.workingFolder,
        },) === program)
    ) {
      continue;
    }
    problems.push(
      `the session ran "${quoted({ command: recorded.command, },)}", a program built from the commit (${program}), and the `
        + 'statement does not name that command as one that ran a built program',
    );
  }
  return {
    problems,
    uncompared,
    builtPrograms: [...new Set(builtPrograms,)],
  };
}
