/**
 Settling a merge by keeping one side of each conflict region of the files named, staging them, and committing the merge; and
 reporting what a worktree's merge holds when no file is named.

 @module
 */

import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  askGit,
  mergeIsOpen,
  type Repository,
  unresolvedPaths,
} from './git-commands.ts';
import {
  stagePaths,
} from './git-changes.ts';
import {
  keepOneSideOfEachRegion,
  type SideToKeep,
} from './conflict-regions.ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { LandingConfigurationError, } from './errors.ts';
import {
  found,
  type Looked,
  missing,
} from './lookups.ts';
import {
  commitTheOpenMerge,
  type CommitOutcome,
} from './open-merge.ts';
import type { Report, } from './report.ts';

/**
 The two-letter status Git puts before a path it could not merge, in any of its unmerged states.
 */
const CONFLICT_MARKERS: ReadonlySet<string> = new Set([
  'UU',
  'AA',
  'DU',
  'UD',
  'AU',
  'UA',
  'DD',
]);

/**
 How many characters of a status line name the merge state, which is `XY ` before the path.
 */
const CONFLICT_MARKER_LENGTH = 2;

/**
 One file to settle, and the side to keep in it.
 */
export type ConflictToSettle = {
  /**
   The file, repository-relative.
   */
  readonly path: string;

  /**
   The side to keep in it.
   */
  readonly side: SideToKeep;
};

/**
 What settling a merge came to.
 */
export type ResolutionOutcome =
  /**
   The worktree's merge was described and nothing was changed.
   */
  | {
    readonly resolved: 'reported';
    readonly conflicts: readonly string[];
    readonly mergeIsOpen: boolean;
  }
  /**
   Every named file was settled and the merge is committed.
   */
    | {
      readonly resolved: 'settled';
      readonly files: readonly {
        readonly path: string;
        readonly regions: number;
        readonly side: SideToKeep;
      }[];
      readonly commit: CommitOutcome;
    };

/**
 Describes the merge a worktree holds: the paths it left conflicted, and whether it holds a merge at all.

 @param repository - the checkout to ask

 @param report - where to report

 @returns what the worktree's merge holds

 @example
 ```ts
 const outcome = await reportOnAConflict({ repository: worktree, report, },);
 ```
 */
export async function reportOnAConflict({
  repository,
  report,
}: {
  readonly repository: Repository;
  readonly report: Report;
},): Promise<ResolutionOutcome> {
  /**
   What Git says is unmerged.
   */
  const asked = await askGit({
    repository,
    words: [
      'status',
      '--porcelain',
    ],
  },);
  /**
   The paths the merge left conflicted, which are the lines whose two status letters include a merge state.
   */
  const conflicts = asked.status === 0
    ? asked.stdout
      .split('\n',)
      .filter(function isConflicted(line: string,) {
        return CONFLICT_MARKERS.has(line.slice(
          0,
          CONFLICT_MARKER_LENGTH
        ),);
      },)
    : [];
  /**
   Whether the worktree holds a merge.
   */
  const open = await mergeIsOpen(repository,);
  report.line(`conflicted: ${String(conflicts.length,)}`,);
  if (conflicts.length > 0) report.line(conflicts.join('\n',),);
  report.line(`merge open: ${String(open,)}`,);
  return {
    resolved: 'reported',
    conflicts,
    mergeIsOpen: open,
  };
}

/**
 Settles the named files by keeping the side each names, stages them, and commits the merge.

 @param repository - the checkout to ask

 @param files - the files to settle

 @param tries - how many times its commit is tried

 @param pauseMilliseconds - milliseconds between two tries

 @param commitTimeoutMilliseconds - milliseconds one commit may take

 @param report - where to report

 @returns what settling came to

 @throws LandingConfigurationError when no merge is open, a named file has no conflict, or a path is still unresolved

 @example
 ```ts
 const outcome = await settleConflicts({ repository: worktree, files: [{ path: 'doc/a.md', side: 'incoming', }], bounds, report, },);
 ```
 */
export async function settleConflicts({
  repository,
  files,
  tries,
  pauseMilliseconds,
  commitTimeoutMilliseconds,
  report,
}: {
  readonly repository: Repository;
  readonly files: readonly ConflictToSettle[];
  readonly tries: number;
  readonly pauseMilliseconds: number;
  readonly commitTimeoutMilliseconds: number;
  readonly report: Report;
},): Promise<ResolutionOutcome> {
  if (!(await mergeIsOpen(repository,))) {
    throw new LandingConfigurationError(`${repository.folder} holds no merge, so there is nothing to settle`,);
  }
  /**
   What each file came to.
   */
  const settled: {
    readonly path: string;
    readonly regions: number;
    readonly side: SideToKeep;
  }[] = [];
  for (const file of files) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- one file at a time, so a refusal names the file it stopped at
    settled.push(await settleOneFile({
      repository,
      file,
      report,
    },),);
  }
  await stagePaths({
    repository,
    paths: files.map(function pathOf(file: ConflictToSettle,) {
      return file.path;
    },),
  },);
  /**
   The paths still unresolved, which a commit cannot be made over.
   */
  const unresolved = await unresolvedPaths(repository,);
  if (unresolved.length > 0) {
    throw new LandingConfigurationError(`still unresolved:\n${unresolved.join('\n',)}`,);
  }
  return {
    resolved: 'settled',
    files: settled,
    commit: await commitTheOpenMerge({
      repository,
      message: '',
      paths: [],
      tries,
      pauseMilliseconds,
      commitTimeoutMilliseconds,
      report,
    },),
  };
}

/**
 Settles one file, refusing when it holds no conflict at all.

 @param repository - the checkout to ask

 @param file - the file to settle

 @param report - where to report

 @returns how many regions were settled and which side was kept

 @throws LandingConfigurationError when the file cannot be read or holds no conflict

 @example
 ```ts
 const settled = await settleOneFile({ repository, file, report, },);
 ```
 */
async function settleOneFile({
  repository,
  file,
  report,
}: {
  readonly repository: Repository;
  readonly file: ConflictToSettle;
  readonly report: Report;
},): Promise<{
  readonly path: string;
  readonly regions: number;
  readonly side: SideToKeep;
}> {
  /**
   The file's path on disk.
   */
  const path = join(
    repository.folder,
    file.path,
  );
  /**
   The settled text and how many regions it held.
   */
  const resolved = await readAndSettle({
    path,
    side: file.side,
  },);
  if (resolved.regions === 0) {
    throw new LandingConfigurationError(`${file.path} holds no conflict, so its side cannot be kept`,);
  }
  await writeFile(
    path,
    resolved.text,
  );
  report.line(`${file.path}: ${String(resolved.regions,)} region(s) resolved as ${file.side}`,);
  return {
    path: file.path,
    regions: resolved.regions,
    side: file.side,
  };
}

/**
 Reads a file and settles its conflict regions.

 @param path - the path

 @param side - the side to keep

 @returns the settled text and how many regions it held

 @throws LandingConfigurationError when the file cannot be read

 @example
 ```ts
 const resolved = await readAndSettle({ path: '/worktree/doc/a.md', side: 'current', },);
 ```
 */
async function readAndSettle({
  path,
  side,
}: {
  readonly path: string;
  readonly side: SideToKeep;
},): Promise<{
  readonly text: string;
  readonly regions: number;
}> {
  /**
   The file's text.
   */
  const text = await textOf(path,);
  if (!text.found) throw new LandingConfigurationError(`${path} cannot be read: ${text.why}`,);
  return keepOneSideOfEachRegion({
    text: text.value,
    side,
  },);
}

/**
 Reads a file as text.

 @param path - the file's path on disk

 @returns its text, or why it could not be read

 @example
 ```ts
 const text = await textOf('/worktree/doc/a.md',);
 ```
 */
async function textOf(path: string,): Promise<Looked<string>> {
  try {
    return found(await readFile(
      path,
      'utf8',
    ),);
  }
  catch (caught) {
    return missing(caughtValueText(caught,),);
  }
}
