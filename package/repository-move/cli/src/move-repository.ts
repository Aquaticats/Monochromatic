/**
 Command-line entry of the copy tool: moves the repository and its worktree subvolumes to the 860 with extent sharing
 kept, failing closed. Without `--run` it prints every step and runs nothing; with `--run` it needs root for the btrfs
 send and receive. The guided runner (`run-repository-move.ts`) calls it for every stage that copies.

 Passes: `first` and `final` (copy), `freeze` and `thaw` (read-only flag), `external-first` and `external-final`
 (plain folders). Required: `--pass --main --snapshots --received --final --state`. Optional: `--inventory`
 (`btrfs subvolume list -u` output, required with `--run`), `--inventory-main`, `--inventory-prefix`,
 `--deadline-seconds` (per transfer), `--total-deadline-seconds`, `--command-deadline-seconds`, `--min-free-gib`,
 `--clone-sources`, `--limit`, `--external-source`, `--external-final`, `--external-deadline-seconds`, `--print`,
 `--run`. Tests only: `--no-scope`, `--subvolume-marker=<file name>`.

 @module
 */

import {
  argv,
  exit,
} from 'node:process';
import { MoveStopError, } from './errors.ts';
import { runMove, } from './run-move.ts';

try {
  await runMove(argv.slice(2,),);
}
catch (error) {
  if (error instanceof MoveStopError) {
    console.error(`${new Date().toISOString()} STOP: ${error.message}`,);
    exit(1,);
  }
  throw error;
}
