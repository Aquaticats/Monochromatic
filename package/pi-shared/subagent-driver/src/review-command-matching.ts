/**
 The comparison between what a review's statement says it ran and what its session records: each stated command against a recorded
 call, the exit code against the one the call's result carries, and each built program the statement names against the command that
 ran it or a script that command runs.

 A built program is a word a command *runs*, not one it names: a path under a Cargo target directory (`target/`, or a `debug` or
 `release` segment of a target directory wherever it stands) or under `dist/`, in the position a shell runs a program or a script.
 The check cannot read a build's provenance; what it confirms is that a program from a build folder ran, and it reports whether the
 session also records a build command, so the commit tie is said for what it is.

 What this comparison cannot see is said as plainly as what it can: a `process` start records no exit code, a command inside a
 `codemode` call is no tool call of its own, and a script that builds a program's path from a variable does not hold the path.
 Those are reported as uncompared or unconfirmed, never as matching.

 @module
 */

import {
  isAbsolute,
  join,
} from 'node:path';
import {
  collapsed,
  executedWordsIn,
  quoted,
} from './command-words.ts';
import {
  buildRanInACommand,
  commandRunsTheProgram,
  scriptTextsRanBy,
} from './scripts-a-command-runs.ts';
import type {
  ReviewCommandRun,
  ReviewStatement,
} from './review-statement-shape.ts';
import type {
  SessionCommand,
  SessionReadings,
} from './session-commands-of-a-run.ts';

/**
 The path segments that mark a folder a build writes into: a Cargo target directory's `target`, its `debug` or `release` output (the
 shared build directory is a target directory at its own root), and the package's `dist` output.
 */
const BUILD_OUTPUT_SEGMENTS: ReadonlySet<string> = new Set([
  'target',
  'dist',
  'debug',
  'release',
]);

/**
 No built program at a path, as a symbol so that no text can stand for it.
 */
const NOT_A_BUILT_PROGRAM: unique symbol = Symbol('path holds no build output segment',);

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

  /**
   Whether a command the session records ran a build tool, in its own program position or in a script it runs.
   */
  readonly buildCommandRan: boolean;
};

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
 Says whether a path is a program a build wrote, wherever its build folder stands.

 No folder is compared with the worktree or with any alias of it: the path is read for a build output segment alone, so a program a
 reviewer built in its scratch folder or in the shared build directory counts as one built here.

 @param path - the path as written, absolute or relative to the session's folder

 @param workingFolder - the folder the session started in

 @returns the absolute path, or {@link NOT_A_BUILT_PROGRAM} when no segment of it is a build output

 @example
 ```ts
 const built = builtProgramPathIn({ path: '/var/mnt/encrypted/cargo-build/debug/deps/a', workingFolder: '/worktree' });
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
   The path, resolved against the session's folder when it is relative.
   */
  const absolute = absolutePathIn({
    path,
    workingFolder,
  },);
  return absolute.split('/',)
    .some(function isABuildSegment(segment,) {
      return BUILD_OUTPUT_SEGMENTS.has(segment,);
    },)
    ? absolute
    : NOT_A_BUILT_PROGRAM;
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
   The folders a command's relative paths may be written from.
   */
  const folders: readonly string[] = [
    session.workingFolder,
    reviewFolder,
  ];
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
  for (const [statedAt, stated,] of statement.commands
    .entries()) {
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
      statedAt,
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
        `the statement says "${quoted({ command: stated.command, },)}" ran ${stated.ranBuiltProgram}, which no build output `
          + 'segment of its path marks as a program a build wrote',
      );
      continue;
    }
    /**
     The text of each script the recorded command runs.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one stated command's scripts, read before that command's claim is judged; a later stated command may need the same script, and the reads are independent within the command. See doc/lint/sequential-awaits.md.
    const scriptTexts = await scriptTextsRanBy({
      command: recorded.command,
      folders,
    },);
    if (!commandRunsTheProgram({
      command: recorded.command,
      program,
      scriptTexts,
      folders,
    },)) {
      problems.push(
        `the statement says "${quoted({ command: stated.command, },)}" ran ${program}, and neither the command nor a script `
          + 'it runs holds that path',
      );
      continue;
    }
    builtPrograms.push(program,);
  }
  // A statement that confirms one built program is answered as complete: a command it does not list is not hunted for. Only when
  // nothing is confirmed does the session decide whether the statement left out a run a build wrote, which is how a review that
  // ran a program and omitted it is a mismatch rather than `ran nothing`.
  /**
   The session calls to read for an unstated built program, which is none once the statement confirms one.
   */
  const sessionsToLookAt = (builtPrograms.length === 0) ? [...session.commands
    .entries(),] : [];
  for (const [at, recorded,] of sessionsToLookAt) {
    /**
     Each word this session call runs, with the build program it names.
     */
    const programsRan = executedWordsIn({ command: recorded.command, },)
      .flatMap(function programOf(ran,) {
        /**
         The path this word runs, when it is a build program.
         */
        const program = builtProgramPathIn({
          path: ran.word,
          workingFolder: session.workingFolder,
        },);
        return (program === NOT_A_BUILT_PROGRAM) ? [] : [program,];
      },);
    if (programsRan.length === 0) {
      continue;
    }
    /**
     The statement's own entry for this call, when the statement lists it.
     */
    const statedAt = statedAtOfRecorded.get(at,);
    /**
     What that entry says it ran, when it says it ran a program.
     */
    const statedProgram = (statedAt === undefined)
      ? undefined
      : statement.commands[statedAt]
        ?.ranBuiltProgram;
    for (const program of programsRan) {
      if (
        (statedProgram !== undefined)
          && (builtProgramPathIn({
            path: statedProgram,
            workingFolder: session.workingFolder,
          },) === program)
      ) {
        continue;
      }
      problems.push(
        `the session ran "${quoted({ command: recorded.command, },)}", a program a build wrote (${program}), and the `
          + 'statement does not name that command as one that ran a built program',
      );
    }
  }
  /**
   Whether a session command ran a build tool, in its own program position or in a script it runs.
   */
  const buildCommandRan = await buildRanInACommand({
    commands: session.commands,
    folders,
  },);
  return {
    problems,
    uncompared,
    builtPrograms: [...new Set(builtPrograms,)],
    buildCommandRan,
  };
}
