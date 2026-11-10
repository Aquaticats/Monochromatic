/**
 What `land-agent-branches --help` says, built from the same table the parser enforces so the printed text and the enforced
 options cannot drift.

 @module
 */

import {
  COMMIT_PAUSE_SECONDS,
  COMMIT_TIMEOUT_SECONDS,
  COMMIT_TRIES,
  GIT_TIMEOUT_SECONDS,
  WORKTREES_FOLDER,
} from './constants.ts';
import { SIDES_TO_KEEP, } from './conflict-regions.ts';

/**
 The name every command line of this package starts with.
 */
export const PROGRAM_NAME = 'land-agent-branches';

/**
 What `--help` says, which is every subcommand and the options it takes.
 */
export const USAGE: string = [
  `Usage: ${PROGRAM_NAME} <subcommand> [options]`,
  '',
  'Lands agents\' branches on main in the main checkout.',
  '',
  'Subcommands:',
  '  land                merge one branch without committing, compile the merged tree when it',
  '                      touches the Git wrapper, commit through the wrapper, and report what is',
  '                      not on main',
  '                      --branch=<name> --message=<text>',
  '  check               merge main into an agent\'s worktree and run what its branch asks for',
  '                      --worktree=<name> --check=wrapper|package|tasks|none [--task=<task> ...]',
  '                      --filter=<text> is accepted from older callers and ignored',
  '  list                land each branch a list file names, each after the check its entry asks',
  '                      --list=<file> [--only=<key> ...]',
  '  commit-open-merge   commit the merge left open in a checkout, retrying while the wrapper stops it',
  '                      --message=<text> [--worktree=<name>] [--path=<path> ...]',
  '  resolve-conflicts   keep one side of each conflict region, stage the files, commit the merge;',
  '                      with no --path it describes the conflicts instead',
  `                      --worktree=<name> [--path=<path>[=${SIDES_TO_KEEP.join('|')}] ...] [--side=<...>]`,
  '  worktrees           report every worktree, its branch, what main lacks, and whether a unit',
  '                      still runs it',
  '  remove-worktrees    remove the worktrees whose work is on main and which nothing would lose;',
  '                      without --apply it only reports',
  '',
  'Every subcommand takes:',
  '  --main-checkout=<folder>        the checkout main is landed on, by default the one this runs in',
  `  --git-timeout-seconds=<n>       how long one Git command may take (default ${String(GIT_TIMEOUT_SECONDS,)})`,
  `  --commit-tries=<n>              how many times a merge's commit is tried (default ${String(COMMIT_TRIES,)})`,
  `  --commit-pause-seconds=<n>      the wait between two tries (default ${String(COMMIT_PAUSE_SECONDS,)})`,
  `  --commit-timeout-seconds=<n>    how long one commit may take (default ${String(COMMIT_TIMEOUT_SECONDS,)})`,
  '  --help                          print this text and exit 0',
  '',
  `Worktrees live under ${WORKTREES_FOLDER}/ of the main checkout. The list file format, the exit codes and the`,
  'checks each branch kind gets are in package/agent-branch-landing/cli/README.md.',
].join('\n',);
