/**
 Command-line entry of the table of uncommitted work: lists, for every worktree the repository knows, what a move would
 strand. Usage: `list-uncommitted-work.ts <main worktree> <output file> [per-worktree timeout in seconds]`.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import {
  argv,
  exit,
} from 'node:process';
import { SCAN_DEFAULT_PER_WORKTREE_SECONDS, } from './constants.ts';
import { MoveStopError, } from './errors.ts';
import { buildUncommittedWorkTable, } from './uncommitted-work-table.ts';

/**
 The parts of the value this function uses, taken out of it.
 */
const [mainWorktree = '', outputFile = '', timeoutText = '',] = argv.slice(2,);

try {
  if ((mainWorktree === '') || (outputFile === '')) {
    throw new MoveStopError('usage: list-uncommitted-work.ts <main worktree> <output file> [timeout seconds]',);
  }
  /**
   The table of uncommitted work.
   */
  const table = await buildUncommittedWorkTable({
    mainWorktree,
    perWorktreeSeconds: timeoutText === '' ? SCAN_DEFAULT_PER_WORKTREE_SECONDS : Number(timeoutText,),
  },);
  await writeFile(
    outputFile,
    table.text,
  );
  console.log(
    `${String(table.worktrees,)} worktrees listed, ${
      String(table.dirty,)
    } with something a move would strand; table in ${outputFile}`,
  );
}
catch (error) {
  if (error instanceof MoveStopError) {
    console.error(`${new Date().toISOString()} STOP: ${error.message}`,);
    exit(1,);
  }
  throw error;
}
