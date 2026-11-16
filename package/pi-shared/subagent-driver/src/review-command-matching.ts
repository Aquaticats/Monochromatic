/**
 The comparison between what a review's statement says it ran and what its session records: each stated command against a recorded
 call, the exit code against the one the call's result carries, and each built program the statement names against the command that
 ran it or a script that command runs.

 A built program is a word a command *runs*, not one it names: a path in a build output folder (`target`, `dist`, or a Cargo target
 directory's `debug`/`release` segment) **under the review's worktree or its own folder**, in the position a shell runs a program or a
 script. A program from the main checkout, or from a build folder tied to no worktree, is not confirmed; the doc states that limit.

 What this comparison cannot see is said as plainly as what it can: a `process` start records no exit code, a command inside a
 `codemode` call is no tool call of its own, a script that builds a program's path from a variable does not hold the path, and a
 statement that states no exit code has none to compare. Those are reported as uncompared or unconfirmed, never as matching.

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
  type CommandComparison,
  NOT_A_BUILT_PROGRAM,
  builtProgramPathIn,
} from './build-path-of-a-review.ts';
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
   Each session command's text with its runs of whitespace collapsed, computed once.
   */
  const recordedTexts = session.commands
    .map(function textOf(recorded,) {
      return collapsed(recorded.command,);
    },);
  /**
   Each script's text, read once for the whole comparison.
   */
  const scriptCache = new Map<string, Promise<string>>();
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
     The command's own text, collapsed.
     */
    const statedText = collapsed(stated.command,);
    /**
     Every session call whose text holds this command and is not already claimed.
     */
    const candidates = recordedTexts
      .flatMap(function atOf(
        text,
        at,
      ) {
        return ((!claimed.has(at,)) && text.includes(statedText,)) ? [at,] : [];
      },);
    /**
     The call this command is paired with: one that exited with the code the statement gives, when there is one, so a command
     run twice (failed, then passed) is not paired with the failure; otherwise the first.
     */
    const recordedAt = candidates.find(function exitsAsStated(at,) {
      return (stated.exitStated !== false)
        && (session.commands[at]
          ?.exitCode
          === stated.exit);
    },) ?? candidates[0]
      ?? (-1);
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
    if (stated.exitStated === false) {
      uncompared.push(
        `the statement states no exit code for "${quoted({ command: stated.command, },)}", so its result cannot be compared`,
      );
    }
    else if (recorded.tool === 'process') {
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
     The program the statement says that command ran, as an absolute path in a folder this review owns.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one stated command's own claim, resolved before the next claim is judged; the same path may be named again and its resolution is cheap. See doc/lint/sequential-awaits.md.
    const program = await builtProgramPathIn({
      path: stated.ranBuiltProgram,
      workingFolder: session.workingFolder,
      reviewFolder,
    },);
    if (program === NOT_A_BUILT_PROGRAM) {
      problems.push(
        `the statement says "${quoted({ command: stated.command, },)}" ran ${stated.ranBuiltProgram}, which is not a program a `
          + 'build wrote in this review\'s worktree or its own folder',
      );
      continue;
    }
    /**
     The text of each script the recorded command runs.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one stated command's scripts, read before that command's claim is judged; a later stated command may need the same script, and the cache shares the read. See doc/lint/sequential-awaits.md.
    const scriptTexts = await scriptTextsRanBy({
      command: recorded.command,
      folders,
      cache: scriptCache,
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
  /**
   Whether a session command ran a build tool, in its own program position or in a script it runs.
   */
  const build = await buildRanInACommand({
    commands: session.commands,
    folders,
    cache: scriptCache,
  },);
  // A statement that confirms one built program, or one build task, is answered as what it shows: a command it does not list is
  // not hunted for. Only when nothing is shown does the session decide whether the statement left out a run a build wrote, which is
  // how a review that ran a program and omitted it is a mismatch rather than `ran nothing`.
  /**
   The session calls to read for an unstated built program, which is none once the statement shows a run.
   */
  const sessionsToLookAt = ((builtPrograms.length === 0) && (!build.ran))
    ? session.commands
      .map(function recordedCall(
        recorded,
        at,
      ) {
        return {
          at,
          recorded,
        };
      },)
    : [];
  /**
   Each session call's build programs, read at once.
   */
  const sessionPrograms = await Promise.all(sessionsToLookAt.map(async function programsOf({
    at,
    recorded,
  }: {
    readonly at: number;
    readonly recorded: SessionCommand;
  },) {
    /**
     Each word this session call runs, with the build program it names.
     */
    const programs = await Promise.all(executedWordsIn({ command: recorded.command, },)
      .map(async function programOf(ran,) {
        return await builtProgramPathIn({
          path: ran.word,
          workingFolder: session.workingFolder,
          reviewFolder,
        },);
      },),);
    return {
      at,
      programs: programs.filter(function isABuiltProgram(one,) {
        return one !== NOT_A_BUILT_PROGRAM;
      },),
    };
  },),);
  for (const {
    at,
    programs,
  } of sessionPrograms) {
    if (programs.length === 0) {
      continue;
    }
    /**
     The session call this entry is.
     */
    const recorded = session.commands[at];
    if (recorded === undefined) {
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
    /**
     The statement's program, when it names one a build wrote.
     */
    const statedBuild = (statedProgram === undefined)
      ? NOT_A_BUILT_PROGRAM
      // oxlint-disable-next-line eslint/no-await-in-loop -- one recorded call's stated program, resolved as that call is judged; the folder is resolved once for the whole comparison. See doc/lint/sequential-awaits.md.
      : await builtProgramPathIn({
        path: statedProgram,
        workingFolder: session.workingFolder,
        reviewFolder,
      },);
    for (const program of programs) {
      if (statedBuild === program) {
        continue;
      }
      problems.push(
        `the session ran "${quoted({ command: recorded.command, },)}", a program a build wrote (${program}), and the `
          + 'statement does not name that command as one that ran a built program',
      );
    }
  }
  return {
    problems,
    uncompared,
    builtPrograms: [...new Set(builtPrograms,)],
    buildCommandRan: build.ran,
    buildTaskCommands: build.commands,
  };
}
